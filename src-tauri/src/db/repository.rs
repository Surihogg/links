//! 链接 / 分组 / 标签的 CRUD 与列表查询。
//!
//! 搜索（`search_links`）与统计（`get_stats`）见 `super::search`；
//! 导出（`export_links`）见 `super::export`。

use super::error::AppError;
use super::models::{
    Category, CreateCategoryPayload, CreateLinkPayload, Link, ListLinksParams, PaginatedResult,
    Tag, UpdateCategoryPayload, UpdateLinkPayload, UpdateTagPayload,
};
use super::row_mapping::{
    build_category_tree, ensure_tags, load_tags_for_link, load_tags_for_links, row_to_category,
    row_to_link, row_to_tag, LINK_COLUMNS,
};
use super::Db;

// ---- 链接 CRUD ----

impl Db {
    /// 按 URL 查找已存在的链接（用于去重）。
    /// 兼容尾部斜杠：将传入与 `/` 结尾两种形式都查一遍。
    pub fn find_by_url(&self, url: &str, exclude_id: Option<i64>) -> Result<Option<Link>, AppError> {
        let conn = self.0.lock().unwrap();
        let normalized = url.trim_end_matches('/');
        let with_slash = format!("{}/", normalized);
        let sql = if exclude_id.is_some() {
            format!(
                "SELECT {} FROM links l WHERE (l.url = ?1 OR l.url = ?2) AND l.id != ?3 LIMIT 1",
                LINK_COLUMNS
            )
        } else {
            format!(
                "SELECT {} FROM links l WHERE l.url = ?1 OR l.url = ?2 LIMIT 1",
                LINK_COLUMNS
            )
        };
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = if let Some(ex) = exclude_id {
            stmt.query(rusqlite::params![normalized, with_slash, ex])?
        } else {
            stmt.query(rusqlite::params![normalized, with_slash])?
        };
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_link(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn create_link(&self, payload: &CreateLinkPayload) -> Result<Link, AppError> {
        let conn = self.0.lock().unwrap();
        let title = payload.title.as_deref().unwrap_or("");
        let description = payload.description.as_deref().unwrap_or("");
        let notes = payload.notes.as_deref().unwrap_or("");
        let favicon_url = payload.favicon_url.as_deref().unwrap_or("");
        let og_image_url = payload.og_image_url.as_deref().unwrap_or("");
        let category_id = payload.category_id.filter(|&id| id > 0);

        conn.execute(
            "INSERT INTO links (url, title, description, notes, favicon_url, og_image_url, category_id) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![payload.url, title, description, notes, favicon_url, og_image_url, category_id],
        )?;
        let id = conn.last_insert_rowid();

        let tags = payload.tags.as_deref().unwrap_or(&[]);
        let tag_ids = ensure_tags(&conn, &tags.iter().cloned().collect::<Vec<_>>());
        for tid in tag_ids {
            conn.execute(
                "INSERT OR IGNORE INTO link_tags (link_id, tag_id) VALUES (?, ?)",
                rusqlite::params![id, tid],
            )?;
        }

        let mut link = conn.query_row(
            &format!("SELECT {} FROM links l WHERE id = ?", LINK_COLUMNS),
            rusqlite::params![id],
            row_to_link,
        )?;
        link.tags = load_tags_for_link(&conn, link.id);
        Ok(link)
    }

    pub fn update_link(&self, payload: &UpdateLinkPayload) -> Result<Link, AppError> {
        let conn = self.0.lock().unwrap();

        let mut sets = Vec::new();
        let mut p: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref v) = payload.url {
            sets.push("url = ?".to_string());
            p.push(Box::new(v.clone()));
        }
        if let Some(ref v) = payload.title {
            sets.push("title = ?".to_string());
            p.push(Box::new(v.clone()));
        }
        if let Some(ref v) = payload.description {
            sets.push("description = ?".to_string());
            p.push(Box::new(v.clone()));
        }
        if let Some(ref v) = payload.notes {
            sets.push("notes = ?".to_string());
            p.push(Box::new(v.clone()));
        }
        if let Some(v) = payload.category_id {
            if v == -1 {
                sets.push("category_id = NULL".to_string());
            } else {
                sets.push("category_id = ?".to_string());
                p.push(Box::new(v));
            }
        }
        if let Some(v) = payload.is_favorite {
            sets.push("is_favorite = ?".to_string());
            p.push(Box::new(v as i32));
        }
        if let Some(v) = payload.is_broken {
            sets.push("is_broken = ?".to_string());
            p.push(Box::new(v as i32));
        }
        if let Some(ref v) = payload.favicon_url {
            sets.push("favicon_url = ?".to_string());
            p.push(Box::new(v.clone()));
        }
        if let Some(ref v) = payload.og_image_url {
            sets.push("og_image_url = ?".to_string());
            p.push(Box::new(v.clone()));
        }

        if !sets.is_empty() {
            sets.push("updated_at = datetime('now','localtime')".to_string());
            let sql = format!("UPDATE links SET {} WHERE id = ?", sets.join(", "));
            p.push(Box::new(payload.id));
            conn.execute(
                &sql,
                rusqlite::params_from_iter(p.iter().map(|v| v.as_ref())),
            )?;
        }

        if let Some(ref tags) = payload.tags {
            conn.execute(
                "DELETE FROM link_tags WHERE link_id = ?",
                rusqlite::params![payload.id],
            )?;
            let tag_ids = ensure_tags(&conn, tags);
            for tid in tag_ids {
                conn.execute(
                    "INSERT OR IGNORE INTO link_tags (link_id, tag_id) VALUES (?, ?)",
                    rusqlite::params![payload.id, tid],
                )?;
            }
        }

        let mut link = conn.query_row(
            &format!("SELECT {} FROM links l WHERE id = ?", LINK_COLUMNS),
            rusqlite::params![payload.id],
            row_to_link,
        )?;
        link.tags = load_tags_for_link(&conn, link.id);
        Ok(link)
    }

    pub fn delete_link(&self, id: i64) -> Result<(), AppError> {
        let conn = self.0.lock().unwrap();
        conn.execute("DELETE FROM links WHERE id = ?", rusqlite::params![id])?;
        Ok(())
    }

    pub fn list_links(&self, params: &ListLinksParams) -> Result<PaginatedResult<Link>, AppError> {
        let conn = self.0.lock().unwrap();
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(30).min(100);
        let offset = (page - 1) * per_page;

        let mut sql_parts: Vec<String> = Vec::new();
        let mut count_parts: Vec<String> = Vec::new();
        let mut p: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut cp: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref cid_opt) = params.category_id {
            match cid_opt {
                Some(cid) => {
                    sql_parts.push("l.category_id = ?".into());
                    count_parts.push("l.category_id = ?".into());
                    p.push(Box::new(*cid));
                    cp.push(Box::new(*cid));
                }
                None => {
                    sql_parts.push("l.category_id IS NULL".into());
                    count_parts.push("l.category_id IS NULL".into());
                }
            }
        }
        if let Some(ref tag) = params.tag {
            let clause = "EXISTS (SELECT 1 FROM link_tags lt JOIN tags t ON t.id = lt.tag_id WHERE lt.link_id = l.id AND t.name = ?)".to_string();
            sql_parts.push(clause.clone());
            count_parts.push(clause);
            p.push(Box::new(tag.clone()));
            cp.push(Box::new(tag.clone()));
        }
        if params.untagged_only.unwrap_or(false) {
            sql_parts
                .push("NOT EXISTS (SELECT 1 FROM link_tags lt WHERE lt.link_id = l.id)".into());
            count_parts
                .push("NOT EXISTS (SELECT 1 FROM link_tags lt WHERE lt.link_id = l.id)".into());
        }
        if params.uncategorized_only.unwrap_or(false) {
            sql_parts.push("l.category_id IS NULL".into());
            count_parts.push("l.category_id IS NULL".into());
        }
        if params.favorite_only.unwrap_or(false) {
            sql_parts.push("l.is_favorite = 1".into());
            count_parts.push("l.is_favorite = 1".into());
        }

        let where_sql = if sql_parts.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", sql_parts.join(" AND "))
        };
        let count_where = if count_parts.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", count_parts.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM links l {}", count_where);
        let total: u32 = conn.query_row(
            &count_sql,
            rusqlite::params_from_iter(cp.iter().map(|v| v.as_ref())),
            |r| r.get(0),
        )?;

        let order_by = match params.sort_by.as_deref() {
            Some("click_count") => "l.click_count DESC, l.updated_at DESC",
            Some("last_opened_at") => "l.last_opened_at DESC, l.updated_at DESC",
            _ => "l.updated_at DESC",
        };
        let query_sql = format!(
            "SELECT {} FROM links l {} ORDER BY {} LIMIT ? OFFSET ?",
            LINK_COLUMNS, where_sql, order_by
        );

        p.push(Box::new(per_page));
        p.push(Box::new(offset));

        let mut stmt = conn.prepare(&query_sql)?;
        let mut items: Vec<Link> = stmt
            .query_map(
                rusqlite::params_from_iter(p.iter().map(|v| v.as_ref())),
                row_to_link,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);

        // 批量加载标签：单条 IN 查询代替原 N 次独立 SELECT
        load_tags_for_links(&conn, &mut items);

        Ok(PaginatedResult {
            items,
            total,
            page,
            per_page,
        })
    }

    pub fn track_click(&self, url: &str) -> Result<(), AppError> {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "UPDATE links SET click_count = click_count + 1, last_opened_at = CAST(strftime('%s','now') AS INTEGER) WHERE url = ?1",
            rusqlite::params![url],
        )?;
        Ok(())
    }
}

// ---- 分组 CRUD ----

impl Db {
    pub fn list_categories(&self) -> Result<Vec<Category>, AppError> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at FROM categories ORDER BY updated_at DESC",
        )?;
        let rows: Vec<Category> = stmt
            .query_map([], row_to_category)?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);

        // 复用统一的树构建函数
        Ok(build_category_tree(Vec::new(), rows))
    }

    pub fn create_category(
        &self,
        payload: &CreateCategoryPayload,
    ) -> Result<Category, AppError> {
        let conn = self.0.lock().unwrap();

        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM categories WHERE name = ? AND IFNULL(parent_id, -1) = IFNULL(?, -1)",
                rusqlite::params![payload.name, payload.parent_id],
                |r| r.get(0),
            )
            .ok();
        if let Some(id) = existing {
            return conn
                .query_row(
                    "SELECT id, name, parent_id, sort_order, created_at, updated_at FROM categories WHERE id = ?",
                    rusqlite::params![id],
                    row_to_category,
                )
                .map_err(AppError::from);
        }

        conn.execute(
            "INSERT INTO categories (name, parent_id) VALUES (?, ?)",
            rusqlite::params![payload.name, payload.parent_id],
        )?;
        let id = conn.last_insert_rowid();
        let cat = conn.query_row(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at FROM categories WHERE id = ?",
            rusqlite::params![id],
            row_to_category,
        )?;
        Ok(cat)
    }

    pub fn update_category(
        &self,
        payload: &UpdateCategoryPayload,
    ) -> Result<Category, AppError> {
        let conn = self.0.lock().unwrap();
        if let Some(ref name) = payload.name {
            conn.execute(
                "UPDATE categories SET name = ?, updated_at = datetime('now','localtime') WHERE id = ?",
                rusqlite::params![name, payload.id],
            )?;
        }
        if payload.unset_parent {
            conn.execute(
                "UPDATE categories SET parent_id = NULL, updated_at = datetime('now','localtime') WHERE id = ?",
                rusqlite::params![payload.id],
            )?;
        } else if let Some(parent_id) = payload.parent_id {
            conn.execute(
                "UPDATE categories SET parent_id = ?, updated_at = datetime('now','localtime') WHERE id = ?",
                rusqlite::params![parent_id, payload.id],
            )?;
        }
        let cat = conn.query_row(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at FROM categories WHERE id = ?",
            rusqlite::params![payload.id],
            row_to_category,
        )?;
        Ok(cat)
    }

    pub fn delete_category(&self, id: i64) -> Result<(), AppError> {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "DELETE FROM categories WHERE id = ?",
            rusqlite::params![id],
        )?;
        Ok(())
    }
}

// ---- 标签 CRUD ----

impl Db {
    pub fn list_tags(&self) -> Result<Vec<Tag>, AppError> {
        let conn = self.0.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, name, updated_at FROM tags ORDER BY updated_at DESC")?;
        let tags = stmt
            .query_map([], row_to_tag)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    pub fn create_tag(&self, name: &str) -> Result<Tag, AppError> {
        let conn = self.0.lock().unwrap();
        if let Ok(mut stmt) = conn.prepare("SELECT id, name, updated_at FROM tags WHERE name = ?") {
            let mut rows = stmt.query(rusqlite::params![name])?;
            if let Some(row) = rows.next()? {
                let id: i64 = row.get(0)?;
                conn.execute(
                    "UPDATE tags SET updated_at = datetime('now','localtime') WHERE id = ?",
                    rusqlite::params![id],
                )
                .ok();
                return Ok(Tag {
                    id,
                    name: row.get(1)?,
                    updated_at: row.get(2)?,
                });
            }
        }
        conn.execute("INSERT INTO tags (name) VALUES (?)", rusqlite::params![name])?;
        let id = conn.last_insert_rowid();
        conn.execute(
            "UPDATE tags SET updated_at = datetime('now','localtime') WHERE id = ?",
            rusqlite::params![id],
        )
        .ok();
        let tag = conn.query_row(
            "SELECT id, name, updated_at FROM tags WHERE id = ?",
            rusqlite::params![id],
            row_to_tag,
        )?;
        Ok(tag)
    }

    pub fn delete_tag(&self, id: i64) -> Result<(), AppError> {
        let conn = self.0.lock().unwrap();
        conn.execute("DELETE FROM tags WHERE id = ?", rusqlite::params![id])?;
        Ok(())
    }

    pub fn update_tag(&self, payload: &UpdateTagPayload) -> Result<Tag, AppError> {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "UPDATE tags SET name = ?, updated_at = datetime('now','localtime') WHERE id = ?",
            rusqlite::params![payload.name, payload.id],
        )?;
        let tag = conn.query_row(
            "SELECT id, name, updated_at FROM tags WHERE id = ?",
            rusqlite::params![payload.id],
            row_to_tag,
        )?;
        Ok(tag)
    }

    pub fn autocomplete_tags(&self, prefix: &str) -> Result<Vec<Tag>, AppError> {
        let conn = self.0.lock().unwrap();
        let pattern = format!(
            "%{}%",
            prefix.replace('%', "\\%").replace('_', "\\_")
        );
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.updated_at FROM tags t \
             LEFT JOIN link_tags lt ON t.id = lt.tag_id \
             WHERE t.name LIKE ? \
             GROUP BY t.id \
             ORDER BY COUNT(lt.link_id) DESC, t.name \
             LIMIT 10",
        )?;
        let tags = stmt
            .query_map(rusqlite::params![pattern], row_to_tag)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }
}
