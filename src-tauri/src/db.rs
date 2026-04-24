use rusqlite::{Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

impl Db {
    pub fn find_by_url(&self, url: &str, exclude_id: Option<i64>) -> Result<Option<Link>, AppError> {
        let conn = self.0.lock().unwrap();
        let normalized = url.trim_end_matches('/');
        let with_slash = format!("{}/", normalized);
        let sql = if exclude_id.is_some() {
            format!("SELECT {} FROM links l WHERE (l.url = ?1 OR l.url = ?2) AND l.id != ?3 LIMIT 1", LINK_COLUMNS)
        } else {
            format!("SELECT {} FROM links l WHERE l.url = ?1 OR l.url = ?2 LIMIT 1", LINK_COLUMNS)
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
    pub fn open(path: &Path) -> SqlResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Db(Mutex::new(conn)))
    }

    pub fn migrate(&self) -> SqlResult<()> {
        let conn = self.0.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                parent_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );

            CREATE TABLE IF NOT EXISTS links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                favicon_url TEXT NOT NULL DEFAULT '',
                og_image_url TEXT NOT NULL DEFAULT '',
                category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );

            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS link_tags (
                link_id INTEGER NOT NULL REFERENCES links(id) ON DELETE CASCADE,
                tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (link_id, tag_id)
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS links_fts USING fts5(
                title, description, notes, url,
                content=links, content_rowid=id
            );

            CREATE TRIGGER IF NOT EXISTS links_ai AFTER INSERT ON links BEGIN
                INSERT INTO links_fts(rowid, title, description, notes, url)
                VALUES (new.id, new.title, new.description, new.notes, new.url);
            END;

            CREATE TRIGGER IF NOT EXISTS links_ad AFTER DELETE ON links BEGIN
                INSERT INTO links_fts(links_fts, rowid, title, description, notes, url)
                VALUES ('delete', old.id, old.title, old.description, old.notes, old.url);
            END;

            CREATE TRIGGER IF NOT EXISTS links_au AFTER UPDATE ON links BEGIN
                INSERT INTO links_fts(links_fts, rowid, title, description, notes, url)
                VALUES ('delete', old.id, old.title, old.description, old.notes, old.url);
                INSERT INTO links_fts(rowid, title, description, notes, url)
                VALUES (new.id, new.title, new.description, new.notes, new.url);
            END;
            ",
        )?;
        // Ensure is_broken column exists for existing databases
        conn.execute_batch("ALTER TABLE links ADD COLUMN is_broken INTEGER NOT NULL DEFAULT 0").ok();
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub description: String,
    pub notes: String,
    pub favicon_url: String,
    pub og_image_url: String,
    pub category_id: Option<i64>,
    pub is_favorite: bool,
    pub is_broken: bool,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkPayload {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub category_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub favicon_url: Option<String>,
    pub og_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLinkPayload {
    pub id: i64,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub category_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_broken: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub sort_order: i32,
    pub children: Vec<Category>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryPayload {
    pub name: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryPayload {
    pub id: i64,
    pub name: Option<String>,
    pub parent_id: Option<Option<i64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ListLinksParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub category_id: Option<Option<i64>>,
    pub tag: Option<String>,
    pub query: Option<String>,
    pub favorite_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub query: String,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub category_id: Option<Option<i64>>,
    pub tag: Option<String>,
    pub favorite_only: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResult<T: Serialize> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Deserialize)]
pub struct ExportParams {
    pub format: String,
    pub category_id: Option<i64>,
    pub tag: Option<String>,
    pub favorite_only: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Database(#[from] rusqlite::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Fetch(#[from] reqwest::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    General(String),
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

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
        tags: vec![],
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

pub(crate) fn load_tags_for_link(conn: &Connection, link_id: i64) -> Vec<String> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT t.name FROM tags t JOIN link_tags lt ON lt.tag_id = t.id WHERE lt.link_id = ?",
    ) else {
        return vec![];
    };
    let tags: Vec<String> = match stmt.query_map(rusqlite::params![link_id], |row| row.get::<_, String>(0)) {
        Ok(rows) => rows.flatten().collect(),
        Err(_) => vec![],
    };
    drop(stmt);
    tags
}

pub(crate) fn ensure_tags(conn: &Connection, tags: &[String]) -> Vec<i64> {
    let mut ids = Vec::new();
    for tag in tags {
        let tag = tag.trim().to_string();
        if tag.is_empty() {
            continue;
        }
        conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", rusqlite::params![tag])
            .ok();
        if let Ok(id) = conn.query_row("SELECT id FROM tags WHERE name = ?", rusqlite::params![tag], |r| r.get::<_, i64>(0)) {
            ids.push(id);
        }
    }
    ids
}

const LINK_COLUMNS: &str = "l.id, l.url, l.title, l.description, l.notes, l.favicon_url, l.og_image_url, l.category_id, l.is_favorite, l.is_broken, l.created_at, l.updated_at";

impl Db {
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
            conn.execute("DELETE FROM link_tags WHERE link_id = ?", rusqlite::params![payload.id])?;
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

        if let Some(Some(cid)) = params.category_id {
            sql_parts.push("l.category_id = ?".into());
            count_parts.push("l.category_id = ?".into());
            p.push(Box::new(cid));
            cp.push(Box::new(cid));
        }
        if let Some(ref tag) = params.tag {
            let clause = "EXISTS (SELECT 1 FROM link_tags lt JOIN tags t ON t.id = lt.tag_id WHERE lt.link_id = l.id AND t.name = ?)".to_string();
            sql_parts.push(clause.clone());
            count_parts.push(clause);
            p.push(Box::new(tag.clone()));
            cp.push(Box::new(tag.clone()));
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

        let query_sql = format!(
            "SELECT {} FROM links l {} ORDER BY l.updated_at DESC LIMIT ? OFFSET ?",
            LINK_COLUMNS, where_sql
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

        for link in &mut items {
            link.tags = load_tags_for_link(&conn, link.id);
        }

        Ok(PaginatedResult { items, total, page, per_page })
    }

    pub fn search_links(&self, params: &SearchParams) -> Result<PaginatedResult<Link>, AppError> {
        let conn = self.0.lock().unwrap();
        let query = &params.query;
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(30).min(100);
        let offset = (page - 1) * per_page;

        let like_pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let fts_query = format!("{}*", query.replace('"', "\"\""));

        let fts_sql = format!("SELECT {} FROM links l JOIN links_fts fts ON fts.rowid = l.id WHERE links_fts MATCH ?", LINK_COLUMNS);
        let tag_sql = format!("SELECT DISTINCT {} FROM links l JOIN link_tags lt ON lt.link_id = l.id JOIN tags t ON t.id = lt.tag_id WHERE t.name LIKE ?", LINK_COLUMNS);
        let like_sql = format!("SELECT {} FROM links l WHERE l.title LIKE ? OR l.description LIKE ? OR l.notes LIKE ? OR l.url LIKE ?", LINK_COLUMNS);

        let base_union = format!("{} UNION {} UNION {}", fts_sql, tag_sql, like_sql);

        let mut filter_parts: Vec<String> = Vec::new();
        if let Some(Some(cid)) = params.category_id {
            filter_parts.push("category_id = ?".into());
        }
        if let Some(ref tag) = params.tag {
            filter_parts.push("EXISTS (SELECT 1 FROM link_tags lt2 JOIN tags t2 ON t2.id = lt2.tag_id WHERE lt2.link_id = sub.id AND t2.name = ?)".into());
        }
        if params.favorite_only.unwrap_or(false) {
            filter_parts.push("is_favorite = 1".into());
        }

        let (union_sql, count_sql) = if filter_parts.is_empty() {
            (format!("{} ORDER BY updated_at DESC", base_union), format!("SELECT COUNT(*) FROM ({})", base_union))
        } else {
            let w = filter_parts.join(" AND ");
            (format!("SELECT * FROM ({}) AS sub WHERE {} ORDER BY updated_at DESC", base_union, w),
             format!("SELECT COUNT(*) FROM ({}) AS sub WHERE {}", base_union, w))
        };

        let build_search_params = |search_q: &str, like_p: &str| -> Vec<Box<dyn rusqlite::types::ToSql>> {
            let mut p: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
                Box::new(search_q.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
            ];
            if let Some(Some(cid)) = params.category_id { p.push(Box::new(cid)); }
            if let Some(ref tag) = params.tag { p.push(Box::new(tag.clone())); }
            p
        };

        let build_like_params = |like_p: &str| -> Vec<Box<dyn rusqlite::types::ToSql>> {
            let mut p: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
            ];
            if let Some(Some(cid)) = params.category_id { p.push(Box::new(cid)); }
            if let Some(ref tag) = params.tag { p.push(Box::new(tag.clone())); }
            p
        };

        let full_query = format!("{} LIMIT ? OFFSET ?", union_sql);
        let query_p = build_search_params(&fts_query, &like_pattern);
        let count_p = build_search_params(&fts_query, &like_pattern);

        let mut all_p: Vec<Box<dyn rusqlite::types::ToSql>> = query_p;
        all_p.push(Box::new(per_page));
        all_p.push(Box::new(offset));

        let res: Result<Vec<Link>, _> = {
            let mut stmt = conn.prepare(&full_query)?;
            let r = stmt.query_map(
                rusqlite::params_from_iter(all_p.iter().map(|v| v.as_ref())),
                row_to_link,
            )?.collect();
            drop(stmt);
            r
        };

        match res {
            Ok(items) => {
                let total: u32 = conn.query_row(
                    &count_sql,
                    rusqlite::params_from_iter(count_p.iter().map(|v| v.as_ref())),
                    |r| r.get(0),
                ).unwrap_or(items.len() as u32);

                let mut items = items;
                for link in &mut items { link.tags = load_tags_for_link(&conn, link.id); }
                Ok(PaginatedResult { items, total, page, per_page })
            }
            Err(_) => {
                let fallback_union = format!("{} UNION {} ORDER BY updated_at DESC", tag_sql, like_sql);
                let (fb_query, fb_count) = if filter_parts.is_empty() {
                    (format!("{} LIMIT ? OFFSET ?", fallback_union), format!("SELECT COUNT(*) FROM ({})", fallback_union))
                } else {
                    let w = filter_parts.join(" AND ");
                    (format!("SELECT * FROM ({}) AS sub WHERE {} ORDER BY updated_at DESC LIMIT ? OFFSET ?", fallback_union, w),
                     format!("SELECT COUNT(*) FROM ({}) AS sub WHERE {}", fallback_union, w))
                };

                let mut fb_p = build_like_params(&like_pattern);
                let fb_count_p = build_like_params(&like_pattern);
                fb_p.push(Box::new(per_page));
                fb_p.push(Box::new(offset));

                let mut fb_stmt = conn.prepare(&fb_query)?;
                let items: Vec<Link> = fb_stmt.query_map(
                    rusqlite::params_from_iter(fb_p.iter().map(|v| v.as_ref())),
                    row_to_link,
                )?.collect::<Result<Vec<_>, _>>()?;
                drop(fb_stmt);

                let total: u32 = conn.query_row(
                    &fb_count,
                    rusqlite::params_from_iter(fb_count_p.iter().map(|v| v.as_ref())),
                    |r| r.get(0),
                ).unwrap_or(items.len() as u32);

                let mut items = items;
                for link in &mut items { link.tags = load_tags_for_link(&conn, link.id); }
                Ok(PaginatedResult { items, total, page, per_page })
            }
        }
    }

    pub fn list_categories(&self) -> Result<Vec<Category>, AppError> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at FROM categories ORDER BY sort_order, name",
        )?;
        let rows: Vec<Category> = stmt
            .query_map([], |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    parent_id: row.get(2)?,
                    sort_order: row.get(3)?,
                    children: vec![],
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut by_id: std::collections::HashMap<i64, Category> = std::collections::HashMap::new();
        let mut children_of: std::collections::HashMap<i64, Vec<Category>> = std::collections::HashMap::new();
        let mut roots = Vec::new();
        for cat in rows {
            if let Some(pid) = cat.parent_id {
                children_of.entry(pid).or_default().push(cat);
            } else {
                roots.push(cat);
            }
        }
        for root in &roots {
            by_id.insert(root.id, Category { children: vec![], ..root.clone() });
        }

        fn build_tree(
            parent_id: i64,
            children_of: &std::collections::HashMap<i64, Vec<Category>>,
        ) -> Vec<Category> {
            let Some(children) = children_of.get(&parent_id) else {
                return vec![];
            };
            children.iter().map(|c| {
                Category {
                    children: build_tree(c.id, children_of),
                    ..c.clone()
                }
            }).collect()
        }

        for root in &mut roots {
            root.children = build_tree(root.id, &children_of);
        }

        Ok(roots)
    }

    pub fn create_category(&self, payload: &CreateCategoryPayload) -> Result<Category, AppError> {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "INSERT INTO categories (name, parent_id) VALUES (?, ?)",
            rusqlite::params![payload.name, payload.parent_id],
        )?;
        let id = conn.last_insert_rowid();
        let cat = conn.query_row(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at FROM categories WHERE id = ?",
            rusqlite::params![id],
            |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    parent_id: row.get(2)?,
                    sort_order: row.get(3)?,
                    children: vec![],
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )?;
        Ok(cat)
    }

    pub fn update_category(&self, payload: &UpdateCategoryPayload) -> Result<Category, AppError> {
        let conn = self.0.lock().unwrap();
        if let Some(ref name) = payload.name {
            conn.execute(
                "UPDATE categories SET name = ?, updated_at = datetime('now','localtime') WHERE id = ?",
                rusqlite::params![name, payload.id],
            )?;
        }
        if let Some(ref parent_id_opt) = payload.parent_id {
            conn.execute(
                "UPDATE categories SET parent_id = ?, updated_at = datetime('now','localtime') WHERE id = ?",
                rusqlite::params![parent_id_opt, payload.id],
            )?;
        }
        let cat = conn.query_row(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at FROM categories WHERE id = ?",
            rusqlite::params![payload.id],
            |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    parent_id: row.get(2)?,
                    sort_order: row.get(3)?,
                    children: vec![],
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )?;
        Ok(cat)
    }

    pub fn delete_category(&self, id: i64) -> Result<(), AppError> {
        let conn = self.0.lock().unwrap();
        conn.execute("DELETE FROM categories WHERE id = ?", rusqlite::params![id])?;
        Ok(())
    }

    pub fn list_tags(&self) -> Result<Vec<Tag>, AppError> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM tags ORDER BY name")?;
        let tags = stmt
            .query_map([], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    pub fn create_tag(&self, name: &str) -> Result<Tag, AppError> {
        let conn = self.0.lock().unwrap();
        conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", rusqlite::params![name])?;
        let tag = conn.query_row(
            "SELECT id, name FROM tags WHERE name = ?",
            rusqlite::params![name],
            |row| Ok(Tag { id: row.get(0)?, name: row.get(1)? }),
        )?;
        Ok(tag)
    }

    pub fn delete_tag(&self, id: i64) -> Result<(), AppError> {
        let conn = self.0.lock().unwrap();
        conn.execute("DELETE FROM tags WHERE id = ?", rusqlite::params![id])?;
        Ok(())
    }

    pub fn autocomplete_tags(&self, prefix: &str) -> Result<Vec<Tag>, AppError> {
        let conn = self.0.lock().unwrap();
        let pattern = format!("%{}%", prefix.replace('%', "\\%").replace('_', "\\_"));
        let mut stmt = conn.prepare("SELECT id, name FROM tags WHERE name LIKE ? ORDER BY name LIMIT 10")?;
        let tags = stmt
            .query_map(rusqlite::params![pattern], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    pub fn export_links(&self, params: &ExportParams) -> Result<String, AppError> {
        let conn = self.0.lock().unwrap();

        let mut where_clauses = Vec::new();
        let mut p: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(cid) = params.category_id {
            where_clauses.push("l.category_id = ?".to_string());
            p.push(Box::new(cid));
        }
        if let Some(ref tag) = params.tag {
            where_clauses.push(
                "EXISTS (SELECT 1 FROM link_tags lt JOIN tags t ON t.id = lt.tag_id WHERE lt.link_id = l.id AND t.name = ?)".to_string(),
            );
            p.push(Box::new(tag.clone()));
        }
        if params.favorite_only.unwrap_or(false) {
            where_clauses.push("l.is_favorite = 1".to_string());
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let sql = format!(
            "SELECT {} FROM links l {} ORDER BY l.updated_at DESC",
            LINK_COLUMNS, where_sql
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut links: Vec<Link> = stmt
            .query_map(
                rusqlite::params_from_iter(p.iter().map(|v| v.as_ref())),
                row_to_link,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);

        for link in &mut links {
            link.tags = load_tags_for_link(&conn, link.id);
        }

        match params.format.as_str() {
            "json" => Ok(serde_json::to_string_pretty(&links)?),
            "markdown" => {
                let mut md = String::from("# Links Export\n\n");
                for link in &links {
                    md.push_str(&format!("- [{}]({})", link.title, link.url));
                    if !link.tags.is_empty() {
                        md.push_str(&format!(" `{}`", link.tags.join("` `")));
                    }
                    if !link.description.is_empty() {
                        md.push_str(&format!("\n  {}", link.description));
                    }
                    md.push('\n');
                }
                Ok(md)
            }
            "csv" => {
                let mut csv = String::from("title,url,description,tags,category_id,favorite,created_at\n");
                for link in &links {
                    csv.push_str(&format!(
                        "\"{}\",\"{}\",\"{}\",\"{}\",{},\"{}\",{}\n",
                        link.title.replace('"', "\"\""),
                        link.url.replace('"', "\"\""),
                        link.description.replace('"', "\"\""),
                        link.tags.join(","),
                        link.category_id.map(|i| i.to_string()).unwrap_or_default(),
                        link.is_favorite,
                        link.created_at,
                    ));
                }
                Ok(csv)
            }
            _ => Err(AppError::General(format!(
                "Unsupported export format: {}",
                params.format
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Db {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        let db = Db(Mutex::new(conn));
        db.migrate().unwrap();
        db
    }

    fn make_link(url: &str) -> CreateLinkPayload {
        CreateLinkPayload {
            url: url.to_string(),
            title: None,
            description: None,
            notes: None,
            category_id: None,
            tags: None,
            favicon_url: None,
            og_image_url: None,
        }
    }

    fn make_link_full(url: &str, title: &str, desc: &str, tags: Vec<&str>) -> CreateLinkPayload {
        CreateLinkPayload {
            url: url.to_string(),
            title: Some(title.to_string()),
            description: Some(desc.to_string()),
            notes: None,
            category_id: None,
            tags: Some(tags.iter().map(|s| s.to_string()).collect()),
            favicon_url: None,
            og_image_url: None,
        }
    }

    #[test]
    fn test_create_link_minimal() {
        let db = test_db();
        let link = db.create_link(&make_link("https://example.com")).unwrap();
        assert_eq!(link.url, "https://example.com");
        assert_eq!(link.title, "");
        assert!(link.category_id.is_none());
        assert!(link.tags.is_empty());
    }

    #[test]
    fn test_create_link_full() {
        let db = test_db();
        let cat = db.create_category(&CreateCategoryPayload { name: "Rust".into(), parent_id: None }).unwrap();
        let payload = CreateLinkPayload {
            url: "https://rust-lang.org".into(),
            title: Some("Rust Language".into()),
            description: Some("Systems programming language".into()),
            notes: Some("My notes".into()),
            category_id: Some(cat.id),
            tags: Some(vec!["rust".into(), "programming".into()]),
            favicon_url: Some("https://rust-lang.org/favicon.ico".into()),
            og_image_url: None,
        };
        let link = db.create_link(&payload).unwrap();
        assert_eq!(link.title, "Rust Language");
        assert_eq!(link.description, "Systems programming language");
        assert_eq!(link.category_id, Some(cat.id));
        let mut tag_names = link.tags.clone();
        tag_names.sort();
        assert_eq!(tag_names, vec!["programming", "rust"]);
    }

    #[test]
    fn test_category_id_filter_negative() {
        let db = test_db();
        let payload = CreateLinkPayload {
            category_id: Some(-1),
            ..make_link("https://example.com")
        };
        let link = db.create_link(&payload).unwrap();
        assert!(link.category_id.is_none());
    }

    #[test]
    fn test_category_id_filter_zero() {
        let db = test_db();
        let payload = CreateLinkPayload {
            category_id: Some(0),
            ..make_link("https://example.com")
        };
        let link = db.create_link(&payload).unwrap();
        assert!(link.category_id.is_none());
    }

    #[test]
    fn test_list_links_pagination() {
        let db = test_db();
        for i in 0..5 {
            db.create_link(&make_link(&format!("https://example.com/{}", i))).unwrap();
        }
        let result = db.list_links(&ListLinksParams {
            page: Some(1), per_page: Some(3), category_id: None, tag: None, query: None, favorite_only: None,
        }).unwrap();
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.total, 5);
        assert_eq!(result.page, 1);
    }

    #[test]
    fn test_list_links_filter_by_category() {
        let db = test_db();
        let cat = db.create_category(&CreateCategoryPayload { name: "News".into(), parent_id: None }).unwrap();
        db.create_link(&CreateLinkPayload { category_id: Some(cat.id), ..make_link("https://news.com") }).unwrap();
        db.create_link(&make_link("https://other.com")).unwrap();

        let result = db.list_links(&ListLinksParams {
            category_id: Some(Some(cat.id)), page: None, per_page: None, tag: None, query: None, favorite_only: None,
        }).unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].url, "https://news.com");
    }

    #[test]
    fn test_list_links_filter_by_tag() {
        let db = test_db();
        db.create_link(&make_link_full("https://rust-lang.org", "Rust", "Lang", vec!["rust"])).unwrap();
        db.create_link(&make_link("https://other.com")).unwrap();

        let result = db.list_links(&ListLinksParams {
            tag: Some("rust".into()), page: None, per_page: None, category_id: None, query: None, favorite_only: None,
        }).unwrap();
        assert_eq!(result.items.len(), 1);
    }

    #[test]
    fn test_list_links_filter_favorites() {
        let db = test_db();
        let link = db.create_link(&make_link("https://example.com")).unwrap();
        db.update_link(&UpdateLinkPayload { id: link.id, is_favorite: Some(true), ..Default::default() }).unwrap();
        db.create_link(&make_link("https://other.com")).unwrap();

        let result = db.list_links(&ListLinksParams {
            favorite_only: Some(true), page: None, per_page: None, category_id: None, tag: None, query: None,
        }).unwrap();
        assert_eq!(result.items.len(), 1);
        assert!(result.items[0].is_favorite);
    }

    #[test]
    fn test_update_link_title() {
        let db = test_db();
        let link = db.create_link(&make_link("https://example.com")).unwrap();
        let updated = db.update_link(&UpdateLinkPayload {
            id: link.id,
            title: Some("New Title".into()),
            ..Default::default()
        }).unwrap();
        assert_eq!(updated.title, "New Title");
    }

    #[test]
    fn test_update_link_clear_category() {
        let db = test_db();
        let cat = db.create_category(&CreateCategoryPayload { name: "Test".into(), parent_id: None }).unwrap();
        let link = db.create_link(&CreateLinkPayload { category_id: Some(cat.id), ..make_link("https://example.com") }).unwrap();
        assert_eq!(link.category_id, Some(cat.id));

        let updated = db.update_link(&UpdateLinkPayload {
            id: link.id,
            category_id: Some(-1),
            ..Default::default()
        }).unwrap();
        assert!(updated.category_id.is_none());
    }

    #[test]
    fn test_update_link_replace_tags() {
        let db = test_db();
        let link = db.create_link(&make_link_full("https://example.com", "T", "D", vec!["old1", "old2"])).unwrap();
        assert_eq!(link.tags, vec!["old1", "old2"]);

        let updated = db.update_link(&UpdateLinkPayload {
            id: link.id,
            tags: Some(vec!["new1".into()]),
            ..Default::default()
        }).unwrap();
        assert_eq!(updated.tags, vec!["new1"]);
    }

    #[test]
    fn test_delete_link_removes_tag_associations() {
        let db = test_db();
        let link = db.create_link(&make_link_full("https://example.com", "T", "D", vec!["tag1"])).unwrap();
        db.delete_link(link.id).unwrap();

        let tags = db.list_tags().unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "tag1");
    }

    #[test]
    fn test_toggle_favorite() {
        let db = test_db();
        let link = db.create_link(&make_link("https://example.com")).unwrap();
        assert!(!link.is_favorite);

        let updated = db.update_link(&UpdateLinkPayload {
            id: link.id,
            is_favorite: Some(true),
            ..Default::default()
        }).unwrap();
        assert!(updated.is_favorite);
    }

    #[test]
    fn test_category_tree() {
        let db = test_db();
        let parent = db.create_category(&CreateCategoryPayload { name: "Parent".into(), parent_id: None }).unwrap();
        let child = db.create_category(&CreateCategoryPayload { name: "Child".into(), parent_id: Some(parent.id) }).unwrap();

        let tree = db.list_categories().unwrap();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].id, parent.id);
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].id, child.id);
    }

    #[test]
    fn test_category_tree_deterministic() {
        let db = test_db();
        let a = db.create_category(&CreateCategoryPayload { name: "A_Root".into(), parent_id: None }).unwrap();
        let b = db.create_category(&CreateCategoryPayload { name: "B_Root".into(), parent_id: None }).unwrap();
        let c = db.create_category(&CreateCategoryPayload { name: "C_Child".into(), parent_id: Some(a.id) }).unwrap();

        let tree = db.list_categories().unwrap();
        assert_eq!(tree.len(), 2);
        let a_cat = tree.iter().find(|t| t.id == a.id).unwrap();
        assert_eq!(a_cat.children.len(), 1);
        assert_eq!(a_cat.children[0].id, c.id);
        let b_cat = tree.iter().find(|t| t.id == b.id).unwrap();
        assert!(b_cat.children.is_empty());
    }

    #[test]
    fn test_delete_parent_category_nullifies_child_ref() {
        let db = test_db();
        let parent = db.create_category(&CreateCategoryPayload { name: "Parent".into(), parent_id: None }).unwrap();
        let _child = db.create_category(&CreateCategoryPayload { name: "Child".into(), parent_id: Some(parent.id) }).unwrap();
        let link = db.create_link(&CreateLinkPayload { category_id: Some(parent.id), ..make_link("https://example.com") }).unwrap();

        db.delete_category(parent.id).unwrap();

        let link_after = db.list_links(&ListLinksParams {
            page: None, per_page: None, category_id: None, tag: None, query: None, favorite_only: None,
        }).unwrap();
        assert!(link_after.items[0].category_id.is_none());

        let cats = db.list_categories().unwrap();
        assert_eq!(cats.len(), 1);
        assert!(cats[0].parent_id.is_none());
    }

    #[test]
    fn test_tag_crud() {
        let db = test_db();
        let tag = db.create_tag("rust").unwrap();
        assert_eq!(tag.name, "rust");

        let dup = db.create_tag("rust").unwrap();
        assert_eq!(dup.id, tag.id);

        let tags = db.list_tags().unwrap();
        assert_eq!(tags.len(), 1);

        db.delete_tag(tag.id).unwrap();
        assert!(db.list_tags().unwrap().is_empty());
    }

    #[test]
    fn test_autocomplete_tags() {
        let db = test_db();
        db.create_tag("rust").unwrap();
        db.create_tag("ruby").unwrap();
        db.create_tag("javascript").unwrap();

        let results = db.autocomplete_tags("ru").unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|t| t.name.contains("ru")));
    }

    #[test]
    fn test_ensure_tags_trims_and_ignores_empty() {
        let db = test_db();
        let conn = db.0.lock().unwrap();
        let ids = ensure_tags(&conn, &["  hello  ".into(), "".into(), "world".into()]);
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_search_fts() {
        let db = test_db();
        db.create_link(&make_link_full("https://example.com", "Rust Programming Language", "A systems language", vec![])).unwrap();

        let results = db.search_links(&SearchParams { query: "Rust".into(), page: None, per_page: None, category_id: None, tag: None, favorite_only: None }).unwrap();
        assert!(!results.items.is_empty());
        assert!(results.items[0].title.contains("Rust"));
    }

    #[test]
    fn test_search_chinese() {
        let db = test_db();
        db.create_link(&make_link_full("https://example.com", "Rust 编程语言指南", "学习 Rust", vec![])).unwrap();

        let results = db.search_links(&SearchParams { query: "编程".into(), page: None, per_page: None, category_id: None, tag: None, favorite_only: None }).unwrap();
        assert!(!results.items.is_empty());
    }

    #[test]
    fn test_search_by_tag_name() {
        let db = test_db();
        db.create_link(&make_link_full("https://example.com", "Some Title", "desc", vec!["webassembly"])).unwrap();

        let results = db.search_links(&SearchParams { query: "webassembly".into(), page: None, per_page: None, category_id: None, tag: None, favorite_only: None }).unwrap();
        assert!(!results.items.is_empty());
    }

    #[test]
    fn test_search_like_fallback() {
        let db = test_db();
        db.create_link(&make_link_full("https://example.com", "Hello World", "desc", vec![])).unwrap();

        let results = db.search_links(&SearchParams { query: "ello".into(), page: None, per_page: None, category_id: None, tag: None, favorite_only: None }).unwrap();
        assert!(!results.items.is_empty());
    }

    #[test]
    fn test_export_json() {
        let db = test_db();
        db.create_link(&make_link_full("https://example.com", "Test", "Desc", vec!["tag1"])).unwrap();

        let json = db.export_links(&ExportParams { format: "json".into(), category_id: None, tag: None, favorite_only: None }).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("tag1"));
    }

    #[test]
    fn test_export_markdown() {
        let db = test_db();
        db.create_link(&make_link_full("https://example.com", "Test", "Desc", vec![])).unwrap();

        let md = db.export_links(&ExportParams { format: "markdown".into(), category_id: None, tag: None, favorite_only: None }).unwrap();
        assert!(md.starts_with("# Links Export"));
        assert!(md.contains("[Test](https://example.com)"));
    }

    #[test]
    fn test_export_csv() {
        let db = test_db();
        db.create_link(&make_link_full("https://example.com", "Test", "Desc", vec![])).unwrap();

        let csv = db.export_links(&ExportParams { format: "csv".into(), category_id: None, tag: None, favorite_only: None }).unwrap();
        assert!(csv.starts_with("title,url,description"));
        assert!(csv.contains("Test"));
    }

    #[test]
    fn test_export_unsupported_format() {
        let db = test_db();
        let result = db.export_links(&ExportParams { format: "xml".into(), category_id: None, tag: None, favorite_only: None });
        assert!(result.is_err());
    }
}

impl Default for UpdateLinkPayload {
    fn default() -> Self {
        UpdateLinkPayload {
            id: 0,
            url: None,
            title: None,
            description: None,
            notes: None,
            category_id: None,
            tags: None,
            is_favorite: None,
            is_broken: None,
        }
    }
}
