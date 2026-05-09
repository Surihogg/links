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
