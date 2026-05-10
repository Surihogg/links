//! SQL 行 → Rust 结构的映射函数。
//!
//! 集中放置共享的 SELECT 列字符串与 row 转换逻辑，避免在多个 CRUD 方法中
//! 重复写 `row.get(0..N)` 列表（重复时极易因列序变更而出 bug）。

use rusqlite::Connection;

use super::models::{Category, Link, Tag};

/// 链接表的标准 SELECT 列顺序，所有读链接 SQL 必须严格遵循此顺序，
/// 与 [`row_to_link`] 的字段读取一一对应。
pub(crate) const LINK_COLUMNS: &str = "l.id, l.url, l.title, l.description, l.notes, l.favicon_url, l.og_image_url, l.category_id, l.is_favorite, l.is_broken, l.click_count, l.last_opened_at, l.created_at, l.updated_at";

/// 将链接表行映射为 [`Link`]。tags 字段需调用方另行通过 [`load_tags_for_link`] 填充。
pub(crate) fn row_to_link(row: &rusqlite::Row) -> rusqlite::Result<Link> {
    Ok(Link {
        id: row.get(0)?,
        url: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        notes: row.get(4)?,
        favicon_url: row.get(5)?,
        og_image_url: row.get(6)?,
        category_id: row.get(7)?,
        is_favorite: row.get::<_, i32>(8)? != 0,
        is_broken: row.get::<_, i32>(9)? != 0,
        click_count: row.get(10)?,
        last_opened_at: row.get(11)?,
        tags: vec![],
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

/// 将分类表行映射为 [`Category`]。SELECT 列顺序约定：
/// `id, name, parent_id, sort_order, created_at, updated_at`。
pub(crate) fn row_to_category(row: &rusqlite::Row) -> rusqlite::Result<Category> {
    Ok(Category {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
        sort_order: row.get(3)?,
        children: vec![],
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

/// 将标签表行映射为 [`Tag`]。SELECT 列顺序约定：`id, name, updated_at`。
pub(crate) fn row_to_tag(row: &rusqlite::Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        updated_at: row.get(2)?,
    })
}

/// 加载某条链接的全部标签名。
///
/// 单条查询版本；批量场景请用 [`load_tags_for_links`] 避免 N+1。
pub(crate) fn load_tags_for_link(conn: &Connection, link_id: i64) -> Vec<String> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT t.name FROM tags t JOIN link_tags lt ON lt.tag_id = t.id WHERE lt.link_id = ?",
    ) else {
        return vec![];
    };
    let tags: Vec<String> =
        match stmt.query_map(rusqlite::params![link_id], |row| row.get::<_, String>(0)) {
            Ok(rows) => rows.flatten().collect(),
            Err(_) => vec![],
        };
    drop(stmt);
    tags
}

/// 批量为多条链接加载标签，把每条 link 的 tags 字段就地填充。
///
/// 一次 SQL 拉回所有 (link_id, tag_name) 对，按 link_id 分组后填回。
/// 替代 list/search/export 路径里 `for link in items { link.tags = load_tags_for_link(...) }`
/// 的 N+1 模式：30 条/页 时从 31 次查询降为 1 次。
///
/// 空 links 时直接返回，不发起查询。
pub(crate) fn load_tags_for_links(conn: &Connection, links: &mut [super::models::Link]) {
    if links.is_empty() {
        return;
    }

    // SQLite 单条 IN (?, ?, ?, ...) 用占位符上限 999；本应用单页最多 100 条，
    // 远低于上限，直接拼一次 IN 即可。如未来分页变大可分批。
    let placeholders: String = std::iter::repeat("?")
        .take(links.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT lt.link_id, t.name \
         FROM link_tags lt JOIN tags t ON t.id = lt.tag_id \
         WHERE lt.link_id IN ({}) \
         ORDER BY lt.link_id, t.name",
        placeholders
    );

    let params: Vec<rusqlite::types::Value> =
        links.iter().map(|l| rusqlite::types::Value::from(l.id)).collect();

    let Ok(mut stmt) = conn.prepare(&sql) else { return };
    let rows = match stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    }) {
        Ok(r) => r,
        Err(_) => return,
    };

    let mut by_link: std::collections::HashMap<i64, Vec<String>> =
        std::collections::HashMap::with_capacity(links.len());
    for row in rows.flatten() {
        by_link.entry(row.0).or_default().push(row.1);
    }
    drop(stmt);

    for link in links {
        if let Some(tags) = by_link.remove(&link.id) {
            link.tags = tags;
        }
    }
}

/// 确保给定标签名都存在于 `tags` 表，返回它们对应的 ID。
/// 同时刷新 `updated_at`，使新近触达的标签在 `list_tags` 中靠前展示。
pub(crate) fn ensure_tags(conn: &Connection, tags: &[String]) -> Vec<i64> {
    let mut ids = Vec::new();
    for tag in tags {
        let tag = tag.trim().to_string();
        if tag.is_empty() {
            continue;
        }
        conn.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?)",
            rusqlite::params![tag],
        )
        .ok();
        conn.execute(
            "UPDATE tags SET updated_at = datetime('now','localtime') WHERE name = ?",
            rusqlite::params![tag],
        )
        .ok();
        if let Ok(id) = conn.query_row(
            "SELECT id FROM tags WHERE name = ?",
            rusqlite::params![tag],
            |r| r.get::<_, i64>(0),
        ) {
            ids.push(id);
        }
    }
    ids
}

/// 由扁平的分类列表（已带 `children = []` 占位）构建树形结构。
/// 返回的 roots 是 parent_id IS NULL 的分类，children 链接好。
pub(crate) fn build_category_tree(mut roots: Vec<Category>, mut others: Vec<Category>) -> Vec<Category> {
    let mut children_of: std::collections::HashMap<i64, Vec<Category>> =
        std::collections::HashMap::new();
    others.drain(..).for_each(|cat| {
        if let Some(pid) = cat.parent_id {
            children_of.entry(pid).or_default().push(cat);
        } else {
            roots.push(cat);
        }
    });

    fn build(
        parent_id: i64,
        children_of: &std::collections::HashMap<i64, Vec<Category>>,
    ) -> Vec<Category> {
        children_of
            .get(&parent_id)
            .map(|cs| {
                cs.iter()
                    .map(|c| Category {
                        children: build(c.id, children_of),
                        ..c.clone()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    for root in &mut roots {
        root.children = build(root.id, &children_of);
    }

    roots
}
