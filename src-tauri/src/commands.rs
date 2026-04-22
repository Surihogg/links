use crate::db::{
    AppError, Category, CreateCategoryPayload, CreateLinkPayload, Db, ExportParams, Link,
    ListLinksParams, PaginatedResult, Tag, UpdateCategoryPayload, UpdateLinkPayload,
};
use rusqlite::params;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

fn get_db_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir");
    dir.join("links.db")
}

pub fn init_db(app: &AppHandle) -> Result<(), AppError> {
    let path = get_db_path(app);
    let db = Db::open(&path)?;
    db.migrate()?;
    app.manage(db);
    Ok(())
}

fn row_to_link(row: &rusqlite::Row) -> rusqlite::Result<Link> {
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
        tags: vec![],
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn load_tags_for_link(conn: &rusqlite::Connection, link_id: i64) -> Vec<String> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT t.name FROM tags t JOIN link_tags lt ON lt.tag_id = t.id WHERE lt.link_id = ?",
    ) else {
        return vec![];
    };
    let tags: Vec<String> = match stmt.query_map(params![link_id], |row| row.get::<_, String>(0)) {
        Ok(rows) => rows.flatten().collect(),
        Err(_) => vec![],
    };
    drop(stmt);
    tags
}

fn ensure_tags(conn: &rusqlite::Connection, tags: &[String]) -> Vec<i64> {
    let mut ids = Vec::new();
    for tag in tags {
        let tag = tag.trim().to_string();
        if tag.is_empty() {
            continue;
        }
        conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", params![tag])
            .ok();
        if let Ok(id) = conn.query_row("SELECT id FROM tags WHERE name = ?", params![tag], |r| r.get::<_, i64>(0)) {
            ids.push(id);
        }
    }
    ids
}

#[tauri::command]
pub fn links_list(
    app: AppHandle,
    db: State<'_, Db>,
    params: ListLinksParams,
) -> Result<PaginatedResult<Link>, AppError> {
    let conn = db.0.lock().unwrap();
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
        "SELECT l.id, l.url, l.title, l.description, l.notes, l.favicon_url, l.og_image_url, l.category_id, l.is_favorite, l.created_at, l.updated_at FROM links l {} ORDER BY l.updated_at DESC LIMIT ? OFFSET ?",
        where_sql
    );

    p.push(Box::new(per_page));
    p.push(Box::new(offset));

    let mut stmt = conn.prepare(&query_sql)?;
    let items: Vec<Link> = stmt
        .query_map(
            rusqlite::params_from_iter(p.iter().map(|v| v.as_ref())),
            row_to_link,
        )?
        .collect::<Result<Vec<_>, _>>()?;

    let mut items = items;
    for link in &mut items {
        link.tags = load_tags_for_link(&conn, link.id);
    }

    Ok(PaginatedResult {
        items,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub fn links_create(
    db: State<'_, Db>,
    app: AppHandle,
    payload: CreateLinkPayload,
) -> Result<Link, AppError> {
    let conn = db.0.lock().unwrap();
    let title = payload.title.unwrap_or_default();
    let description = payload.description.unwrap_or_default();
    let notes = payload.notes.unwrap_or_default();
    let favicon_url = payload.favicon_url.unwrap_or_default();
    let og_image_url = payload.og_image_url.unwrap_or_default();
    let category_id = payload.category_id.filter(|&id| id > 0);

    conn.execute(
        "INSERT INTO links (url, title, description, notes, favicon_url, og_image_url, category_id) VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![payload.url, title, description, notes, favicon_url, og_image_url, category_id],
    )?;
    let id = conn.last_insert_rowid();

    let tags = payload.tags.unwrap_or_default();
    let tag_ids = ensure_tags(&conn, &tags);
    for tid in tag_ids {
        conn.execute(
            "INSERT OR IGNORE INTO link_tags (link_id, tag_id) VALUES (?, ?)",
            params![id, tid],
        )?;
    }

    let link = conn.query_row(
        "SELECT id, url, title, description, notes, favicon_url, og_image_url, category_id, is_favorite, created_at, updated_at FROM links WHERE id = ?",
        params![id],
        row_to_link,
    )?;

    let mut link = link;
    link.tags = load_tags_for_link(&conn, link.id);
    drop(conn);

    let url_for_fetch = link.url.clone();
    let link_id = link.id;
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        match crate::fetcher::fetch_metadata(&url_for_fetch).await {
            Ok(meta) => {
                let db_state = app_clone.state::<Db>();
                let Ok(c) = db_state.0.lock() else { return; };
                c.execute(
                    "UPDATE links SET title = CASE WHEN title = '' THEN ?1 ELSE title END, description = CASE WHEN description = '' THEN ?2 ELSE description END, favicon_url = CASE WHEN favicon_url = '' THEN ?3 ELSE favicon_url END, og_image_url = CASE WHEN og_image_url = '' THEN ?4 ELSE og_image_url END, updated_at = datetime('now','localtime') WHERE id = ?5",
                    params![meta.title, meta.description, meta.favicon_url, meta.og_image_url, link_id],
                ).ok();
            }
            Err(e) => {
                log::warn!("metadata fetch failed for {}: {}", url_for_fetch, e);
            }
        }
    });

    Ok(link)
}

#[tauri::command]
pub fn links_update(db: State<'_, Db>, payload: UpdateLinkPayload) -> Result<Link, AppError> {
    let conn = db.0.lock().unwrap();

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
            params![payload.id],
        )?;
        let tag_ids = ensure_tags(&conn, tags);
        for tid in tag_ids {
            conn.execute(
                "INSERT OR IGNORE INTO link_tags (link_id, tag_id) VALUES (?, ?)",
                params![payload.id, tid],
            )?;
        }
    }

    let link = conn.query_row(
        "SELECT id, url, title, description, notes, favicon_url, og_image_url, category_id, is_favorite, created_at, updated_at FROM links WHERE id = ?",
        params![payload.id],
        row_to_link,
    )?;
    let mut link = link;
    link.tags = load_tags_for_link(&conn, link.id);
    Ok(link)
}

#[tauri::command]
pub fn links_delete(db: State<'_, Db>, id: i64) -> Result<(), AppError> {
    let conn = db.0.lock().unwrap();
    conn.execute("DELETE FROM links WHERE id = ?", params![id])?;
    Ok(())
}

#[tauri::command]
pub fn links_search(db: State<'_, Db>, query: String) -> Result<Vec<Link>, AppError> {
    let conn = db.0.lock().unwrap();
    let like_pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));

    let fts_query = format!("{}*", query.replace('"', "\"\""));
    let fts_sql = "SELECT l.id, l.url, l.title, l.description, l.notes, l.favicon_url, l.og_image_url, l.category_id, l.is_favorite, l.created_at, l.updated_at FROM links l JOIN links_fts fts ON fts.rowid = l.id WHERE links_fts MATCH ?";
    let tag_sql = "SELECT DISTINCT l.id, l.url, l.title, l.description, l.notes, l.favicon_url, l.og_image_url, l.category_id, l.is_favorite, l.created_at, l.updated_at FROM links l JOIN link_tags lt ON lt.link_id = l.id JOIN tags t ON t.id = lt.tag_id WHERE t.name LIKE ?";
    let like_sql = "SELECT l.id, l.url, l.title, l.description, l.notes, l.favicon_url, l.og_image_url, l.category_id, l.is_favorite, l.created_at, l.updated_at FROM links l WHERE l.title LIKE ? OR l.description LIKE ? OR l.notes LIKE ?";

    let items: Vec<Link> = {
        let full_sql = format!(
            "{} UNION {} UNION {} ORDER BY updated_at DESC LIMIT 50",
            fts_sql, tag_sql, like_sql
        );
        let fallback_sql = format!(
            "{} UNION {} ORDER BY updated_at DESC LIMIT 50",
            tag_sql, like_sql
        );
        let mut stmt = conn.prepare(&full_sql)?;
        let res: Result<Vec<Link>, _> = stmt.query_map(
            rusqlite::params![fts_query, like_pattern, like_pattern, like_pattern, like_pattern],
            row_to_link,
        )?.collect();
        drop(stmt);
        match res {
            Ok(v) => Ok(v),
            Err(_) => {
                let mut fb = conn.prepare(&fallback_sql)?;
                let res: Result<Vec<Link>, _> = fb.query_map(
                    rusqlite::params![like_pattern, like_pattern, like_pattern],
                    row_to_link,
                )?.collect();
                drop(fb);
                res
            }
        }?
    };

    let mut items = items;
    for link in &mut items {
        link.tags = load_tags_for_link(&conn, link.id);
    }
    Ok(items)
}

#[tauri::command]
pub fn categories_list(db: State<'_, Db>) -> Result<Vec<Category>, AppError> {
    let conn = db.0.lock().unwrap();
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

    let mut map = std::collections::HashMap::new();
    let mut roots = Vec::new();
    for cat in rows {
        map.insert(cat.id, cat);
    }
    let ids: Vec<i64> = map.keys().cloned().collect();
    for id in ids {
        if let Some(cat) = map.remove(&id) {
            if let Some(pid) = cat.parent_id {
                if let Some(parent) = map.get_mut(&pid) {
                    parent.children.push(cat);
                } else {
                    roots.push(cat);
                }
            } else {
                roots.push(cat);
            }
        }
    }
    Ok(roots)
}

#[tauri::command]
pub fn categories_create(db: State<'_, Db>, payload: CreateCategoryPayload) -> Result<Category, AppError> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO categories (name, parent_id) VALUES (?, ?)",
        params![payload.name, payload.parent_id],
    )?;
    let id = conn.last_insert_rowid();
    let cat = conn.query_row(
        "SELECT id, name, parent_id, sort_order, created_at, updated_at FROM categories WHERE id = ?",
        params![id],
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

#[tauri::command]
pub fn categories_update(db: State<'_, Db>, payload: UpdateCategoryPayload) -> Result<Category, AppError> {
    let conn = db.0.lock().unwrap();
    if let Some(ref name) = payload.name {
        conn.execute(
            "UPDATE categories SET name = ?, updated_at = datetime('now','localtime') WHERE id = ?",
            params![name, payload.id],
        )?;
    }
    if let Some(ref parent_id_opt) = payload.parent_id {
        conn.execute(
            "UPDATE categories SET parent_id = ?, updated_at = datetime('now','localtime') WHERE id = ?",
            params![parent_id_opt, payload.id],
        )?;
    }
    let cat = conn.query_row(
        "SELECT id, name, parent_id, sort_order, created_at, updated_at FROM categories WHERE id = ?",
        params![payload.id],
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

#[tauri::command]
pub fn categories_delete(db: State<'_, Db>, id: i64) -> Result<(), AppError> {
    let conn = db.0.lock().unwrap();
    conn.execute("DELETE FROM categories WHERE id = ?", params![id])?;
    Ok(())
}

#[tauri::command]
pub fn tags_list(db: State<'_, Db>) -> Result<Vec<Tag>, AppError> {
    let conn = db.0.lock().unwrap();
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

#[tauri::command]
pub fn tags_delete(db: State<'_, Db>, id: i64) -> Result<(), AppError> {
    let conn = db.0.lock().unwrap();
    conn.execute("DELETE FROM tags WHERE id = ?", params![id])?;
    Ok(())
}

#[tauri::command]
pub fn tags_create(db: State<'_, Db>, name: String) -> Result<Tag, AppError> {
    let conn = db.0.lock().unwrap();
    conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", params![name])?;
    let tag = conn.query_row(
        "SELECT id, name FROM tags WHERE name = ?",
        params![name],
        |row| Ok(Tag { id: row.get(0)?, name: row.get(1)? }),
    )?;
    Ok(tag)
}

#[tauri::command]
pub fn tags_autocomplete(db: State<'_, Db>, prefix: String) -> Result<Vec<Tag>, AppError> {
    let conn = db.0.lock().unwrap();
    let pattern = format!("%{}%", prefix.replace('%', "\\%").replace('_', "\\_"));
    let mut stmt = conn.prepare("SELECT id, name FROM tags WHERE name LIKE ? ORDER BY name LIMIT 10")?;
    let tags = stmt
        .query_map(params![pattern], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tags)
}

#[tauri::command]
pub async fn fetch_metadata(url: String) -> Result<crate::fetcher::PageMeta, AppError> {
    crate::fetcher::fetch_metadata(&url)
        .await
        .map_err(|e| {
            log::warn!("metadata fetch failed for {}: {}", url, e);
            AppError::General(e.to_string())
        })
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), AppError> {
    open::that(&url).map_err(|e| AppError::General(e.to_string()))
}

#[tauri::command]
pub fn save_file(content: String, filename: String) -> Result<(), AppError> {
    let Some(path) = rfd::FileDialog::new()
        .set_file_name(&filename)
        .save_file()
    else {
        return Ok(());
    };
    std::fs::write(&path, content)?;
    Ok(())
}

#[tauri::command]
pub fn export_links(db: State<'_, Db>, params: ExportParams) -> Result<String, AppError> {
    let links = {
        let conn = db.0.lock().unwrap();

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
            "SELECT l.id, l.url, l.title, l.description, l.notes, l.favicon_url, l.og_image_url, l.category_id, l.is_favorite, l.created_at, l.updated_at FROM links l {} ORDER BY l.updated_at DESC",
            where_sql
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut links: Vec<Link> = stmt
            .query_map(
                rusqlite::params_from_iter(p.iter().map(|v| v.as_ref())),
                row_to_link,
            )?
            .collect::<Result<Vec<_>, _>>()?;

        for link in &mut links {
            link.tags = load_tags_for_link(&conn, link.id);
        }

        links
    };

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
