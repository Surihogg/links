use crate::config::Config;
use crate::db::{
    AppError, CreateCategoryPayload, CreateLinkPayload, Db, ExportParams, ListLinksParams,
    PaginatedResult, SearchParams, UpdateCategoryPayload, UpdateLinkPayload, UpdateTagPayload,
};
use rusqlite::params;
use serde_json::Value;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub struct PendingDeepLink(pub Mutex<Option<Value>>);

#[tauri::command]
pub fn pop_pending_deep_link(state: State<'_, PendingDeepLink>) -> Option<Value> {
    state.0.lock().unwrap().take()
}

/// 本地 HTTP 服务的端口与 token，前端用来动态生成 Bookmarklet
pub struct LocalServerInfo {
    pub port: u16,
    pub token: String,
}

#[derive(serde::Serialize)]
pub struct LocalServerInfoDto {
    pub port: u16,
    pub token: String,
}

#[tauri::command]
pub fn get_local_server_info(state: State<'_, LocalServerInfo>) -> LocalServerInfoDto {
    LocalServerInfoDto { port: state.port, token: state.token.clone() }
}

fn log_fetch_failure(app: &AppHandle, url: &str, error: &str) {
    let Ok(dir) = app.path().app_data_dir() else { return };
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("fail_links.log");

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let os = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    let proxy_info = [
        ("HTTP_PROXY", std::env::var("HTTP_PROXY").ok()),
        ("HTTPS_PROXY", std::env::var("HTTPS_PROXY").ok()),
        ("ALL_PROXY", std::env::var("ALL_PROXY").ok()),
        ("NO_PROXY", std::env::var("NO_PROXY").ok()),
    ]
    .iter()
    .filter_map(|(k, v)| v.as_ref().map(|val| format!("{}={}", k, val)))
    .collect::<Vec<_>>()
    .join(", ");
    let proxy_line = if proxy_info.is_empty() { "None".into() } else { proxy_info };

    let entry = format!(
        "[{}] {}\n  URL: {}\n  OS: {}\n  Proxy: {}\n---\n",
        timestamp, error, url, os, proxy_line
    );

    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(entry.as_bytes());
    }
}

fn get_db_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir");
    dir.join("links.db")
}

pub fn init_db(app: &AppHandle) -> Result<(), AppError> {
    let path = get_db_path(app);
    log::info!("[init_db] opening database at {:?}", path);
    match Db::open(&path) {
        Ok(db) => {
            log::info!("[init_db] database opened, running migrations...");
            db.migrate()?;
            log::info!("[init_db] migrations done");
            app.manage(db);
            Ok(())
        }
        Err(e) => {
            log::warn!("[init_db] failed to open database: {}, recreating...", e);
            // DB file is missing or corrupt — remove stale files and recreate.
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_file(path.with_extension("db-wal"));
            let _ = std::fs::remove_file(path.with_extension("db-shm"));
            let db = Db::open(&path)?;
            db.migrate()?;
            log::info!("[init_db] database recreated and migrated");
            app.manage(db);
            Ok(())
        }
    }
}

#[tauri::command]
pub fn links_list(
    _app: AppHandle,
    db: State<'_, Db>,
    params: ListLinksParams,
) -> Result<PaginatedResult<crate::db::Link>, AppError> {
    log::info!("[cmd] links_list called");
    db.list_links(&params)
}

#[tauri::command]
pub fn links_create(
    db: State<'_, Db>,
    app: AppHandle,
    payload: CreateLinkPayload,
) -> Result<crate::db::Link, AppError> {
    let link = db.create_link(&payload)?;

    let url_for_fetch = link.url.clone();
    let link_id = link.id;
    let app_clone = app.clone();
    // Only re-fetch metadata if frontend didn't already provide it.
    // Frontend fetches title/description/favicon via fetchMeta before submit;
    // if title is already populated, skip the redundant backend fetch.
    let needs_meta_fetch = link.title.is_empty() && link.description.is_empty() && link.favicon_url.is_empty();

    // Spawn link status check after metadata fetch is attempted
    let url_for_check = link.url.clone();
    let link_id_check = link.id;
    let app_clone_check = app.clone();
    tauri::async_runtime::spawn(async move {
        // Check reachability in background and mark is_broken if needed
        let reachable = do_check_link(&url_for_check).await.unwrap_or(false);
        if !reachable {
            if let Ok(db_guard) = app_clone_check.state::<Db>().0.lock() {
                let _ = db_guard.execute(
                    "UPDATE links SET is_broken = 1 WHERE id = ?",
                    rusqlite::params![link_id_check],
                );
            }
        }
        if !needs_meta_fetch { return; }
        match crate::fetcher::fetch_metadata(&url_for_fetch).await {
            Ok(meta) => {
                let db_state = app_clone.state::<Db>();
                let Ok(c) = db_state.0.lock() else { return };
                c.execute(
                    "UPDATE links SET title = CASE WHEN title = '' THEN ?1 ELSE title END, description = CASE WHEN description = '' THEN ?2 ELSE description END, favicon_url = CASE WHEN favicon_url = '' THEN ?3 ELSE favicon_url END, og_image_url = CASE WHEN og_image_url = '' THEN ?4 ELSE og_image_url END, updated_at = datetime('now','localtime') WHERE id = ?5",
                    params![meta.title, meta.description, meta.favicon_url, meta.og_image_url, link_id],
                ).ok();
            }
            Err(e) => {
                log::warn!("metadata fetch failed for {}: {}", url_for_fetch, e);
                log_fetch_failure(&app_clone, &url_for_fetch, &e.to_string());
            }
        }
    });

    Ok(link)
}

#[tauri::command]
pub fn links_update(db: State<'_, Db>, payload: UpdateLinkPayload) -> Result<crate::db::Link, AppError> {
    db.update_link(&payload)
}

#[tauri::command]
pub fn links_delete(db: State<'_, Db>, id: i64) -> Result<(), AppError> {
    db.delete_link(id)
}

#[tauri::command]
pub fn links_search(db: State<'_, Db>, params: SearchParams) -> Result<PaginatedResult<crate::db::Link>, AppError> {
    db.search_links(&params)
}

#[tauri::command]
pub fn categories_list(db: State<'_, Db>) -> Result<Vec<crate::db::Category>, AppError> {
    log::info!("[cmd] categories_list called");
    db.list_categories()
}

#[tauri::command]
pub fn categories_create(db: State<'_, Db>, payload: CreateCategoryPayload) -> Result<crate::db::Category, AppError> {
    db.create_category(&payload)
}

#[tauri::command]
pub fn categories_update(db: State<'_, Db>, payload: UpdateCategoryPayload) -> Result<crate::db::Category, AppError> {
    db.update_category(&payload)
}

#[tauri::command]
pub fn categories_delete(db: State<'_, Db>, id: i64) -> Result<(), AppError> {
    db.delete_category(id)
}

#[tauri::command]
pub fn tags_list(db: State<'_, Db>) -> Result<Vec<crate::db::Tag>, AppError> {
    log::info!("[cmd] tags_list called");
    db.list_tags()
}

#[tauri::command]
pub fn tags_delete(db: State<'_, Db>, id: i64) -> Result<(), AppError> {
    db.delete_tag(id)
}

#[tauri::command]
pub fn tags_create(db: State<'_, Db>, name: String) -> Result<crate::db::Tag, AppError> {
    db.create_tag(&name)
}

#[tauri::command]
pub fn tags_update(db: State<'_, Db>, payload: UpdateTagPayload) -> Result<crate::db::Tag, AppError> {
    db.update_tag(&payload)
}

#[tauri::command]
pub fn tags_autocomplete(db: State<'_, Db>, prefix: String) -> Result<Vec<crate::db::Tag>, AppError> {
    db.autocomplete_tags(&prefix)
}

#[tauri::command]
pub async fn fetch_metadata(app: AppHandle, url: String) -> Result<crate::fetcher::PageMeta, AppError> {
    crate::fetcher::fetch_metadata(&url)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            log::warn!("metadata fetch failed for {}: {}", url, msg);
            log_fetch_failure(&app, &url, &msg);
            AppError::General(msg)
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
    db.export_links(&params)
}

#[derive(Debug, Default)]
struct ImportStats {
    links_imported: u32,
    categories_created: u32,
}

#[derive(Debug, Clone)]
struct BookmarkEntry {
    url: String,
    title: String,
    favicon: String,
    folder_path: Vec<String>,
}

fn parse_bookmark_html(html: &str) -> Vec<BookmarkEntry> {
    let mut entries = Vec::new();
    let mut folder_stack: Vec<String> = Vec::new();

    for line in html.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lower = trimmed.to_lowercase();

        if lower.starts_with("<dt><h3") {
            if let Some(name) = extract_folder_name(trimmed) {
                folder_stack.push(name);
            }
        } else if lower.starts_with("</dl>") {
            folder_stack.pop();
        } else if lower.starts_with("<dt><a ") {
            if let Some(href) = extract_href(trimmed) {
                if href.starts_with("http") {
                    let title = extract_link_title(trimmed);
                    let favicon = extract_icon(trimmed);
                    entries.push(BookmarkEntry {
                        url: href,
                        title,
                        favicon,
                        folder_path: folder_stack.clone(),
                    });
                }
            }
        }
    }

    entries
}

fn extract_folder_name(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let h3_start = lower.find("<h3")?;
    let after_h3 = &lower[h3_start..];
    let start = after_h3.find('>')? + h3_start + 1;
    let end = lower.rfind("</h3>")?;
    let name = text[start..end].trim();
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
}

fn extract_href(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let start = lower.find("href=\"")? + 6;
    let rest = &text[start..];
    let end = rest.find('"')?;
    let href = rest[..end].to_string();
    if href.is_empty() {
        return None;
    }
    Some(href)
}

fn extract_icon(text: &str) -> String {
    let lower = text.to_lowercase();
    let Some(start) = lower.find("icon=\"") else {
        return String::new();
    };
    let rest = &text[start + 6..];
    let Some(end) = rest.find('"') else {
        return String::new();
    };
    rest[..end].to_string()
}

fn extract_link_title(text: &str) -> String {
    let lower = text.to_lowercase();
    let a_start = lower.find("<a");
    let Some(a_start) = a_start else {
        return String::new();
    };
    let after_a = &lower[a_start..];
    let Some(start) = after_a.find('>') else {
        return String::new();
    };
    let start = start + a_start + 1;
    let Some(end) = lower.rfind("</a>") else {
        return String::new();
    };
    text[start..end].trim().to_string()
}

fn get_or_create_category(
    name: &str,
    parent_id: Option<i64>,
    conn: &rusqlite::Connection,
) -> Result<(i64, bool), AppError> {
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM categories WHERE name = ? AND (parent_id IS ? OR (parent_id IS NULL AND ? IS NULL))",
            rusqlite::params![name, parent_id, parent_id],
            |r| r.get(0),
        )
        .ok();

    if let Some(id) = existing {
        return Ok((id, false));
    }

    conn.execute(
        "INSERT INTO categories (name, parent_id) VALUES (?, ?)",
        rusqlite::params![name, parent_id],
    )?;

    Ok((conn.last_insert_rowid(), true))
}

fn import_bookmark_entries(
    entries: &[BookmarkEntry],
    conn: &rusqlite::Connection,
) -> Result<ImportStats, AppError> {
    let mut stats = ImportStats::default();
    let tx = conn.unchecked_transaction()?;

    let mut category_cache: std::collections::HashMap<(String, Option<i64>), i64> =
        std::collections::HashMap::new();

    for entry in entries {
        let mut parent_id: Option<i64> = None;
        for folder_name in &entry.folder_path {
            let cache_key = (folder_name.clone(), parent_id);

            let category_id = if let Some(&id) = category_cache.get(&cache_key) {
                id
            } else {
                let (id, created) = get_or_create_category(folder_name, parent_id, &tx)?;
                if created {
                    stats.categories_created += 1;
                }
                category_cache.insert(cache_key, id);
                id
            };

            parent_id = Some(category_id);
        }

        let exists: bool = tx
            .query_row(
                "SELECT COUNT(*) FROM links WHERE url = ?",
                rusqlite::params![&entry.url],
                |r| Ok(r.get::<_, i64>(0)? > 0),
            )
            .unwrap_or(false);

        if !exists {
            tx.execute(
                "INSERT INTO links (url, title, favicon_url, category_id) VALUES (?, ?, ?, ?)",
                rusqlite::params![&entry.url, &entry.title, &entry.favicon, parent_id],
            )?;
            let id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO links_fts (rowid, title, description, notes, url) VALUES (?, '', '', '', ?)",
                rusqlite::params![id, &entry.url],
            )
            .ok();
            stats.links_imported += 1;
        }
    }

    tx.commit()?;
    Ok(stats)
}

#[tauri::command]
pub fn import_bookmarks(db: State<'_, Db>) -> Result<(u32, u32), AppError> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("书签文件", &["html", "htm"])
        .set_title("导入浏览器书签")
        .pick_file()
    else {
        return Ok((0, 0));
    };

    let html = std::fs::read_to_string(&path)?;
    let entries = parse_bookmark_html(&html);

    let conn = db.0.lock().unwrap();
    let stats = import_bookmark_entries(&entries, &conn)?;

    Ok((stats.links_imported, stats.categories_created))
}

#[tauri::command]
pub fn get_setting(config: State<'_, Config>, key: String) -> Result<Option<String>, AppError> {
    Ok(config.get(&key))
}

#[tauri::command]
pub fn set_setting(app: AppHandle, config: State<'_, Config>, key: String, value: String) -> Result<(), AppError> {
    config.set(&key, &value)?;
    let dir = app.path().app_data_dir().expect("failed to resolve app data dir");
    config.save(&dir)?;
    Ok(())
}

#[tauri::command]
pub fn exit_app(app: AppHandle) {
    app.exit(0);
}

/// 获取 Windows 系统代理 URL，供 updater 等场景使用
/// 非 Windows 平台始终返回 None
#[tauri::command]
pub fn get_system_proxy() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        crate::fetcher::get_windows_system_proxy()
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

pub const DEFAULT_SHORTCUT: &str = "CmdOrCtrl+Shift+L";
pub const DEFAULT_MAIN_SHORTCUT: &str = "CmdOrCtrl+Shift+J";

#[tauri::command]
pub fn get_shortcut(config: State<'_, Config>) -> Result<String, AppError> {
    Ok(config.get("global-shortcut").unwrap_or_else(|| DEFAULT_SHORTCUT.to_string()))
}

#[tauri::command]
pub fn get_main_shortcut(config: State<'_, Config>) -> Result<String, AppError> {
    Ok(config.get("main-shortcut").unwrap_or_else(|| DEFAULT_MAIN_SHORTCUT.to_string()))
}

#[tauri::command]
pub fn set_main_shortcut(app: AppHandle, config: State<'_, Config>, shortcut: String) -> Result<String, AppError> {
    let parsed: tauri_plugin_global_shortcut::Shortcut = shortcut
        .parse()
        .map_err(|e: <tauri_plugin_global_shortcut::Shortcut as std::str::FromStr>::Err| AppError::General(e.to_string()))?;
    let old = config.get("main-shortcut").unwrap_or_else(|| DEFAULT_MAIN_SHORTCUT.to_string());
    let _ = app.global_shortcut().unregister(old.as_str());
    app.global_shortcut()
        .register(parsed)
        .map_err(|e| AppError::General(e.to_string()))?;
    config.set("main-shortcut", &shortcut)?;
    let dir = app.path().app_data_dir().expect("failed to resolve app data dir");
    config.save(&dir)?;
    Ok(parsed.to_string())
}

#[tauri::command]
pub fn set_shortcut(app: AppHandle, config: State<'_, Config>, shortcut: String) -> Result<String, AppError> {
    let parsed: tauri_plugin_global_shortcut::Shortcut = shortcut.parse()
        .map_err(|e: <tauri_plugin_global_shortcut::Shortcut as std::str::FromStr>::Err| AppError::General(e.to_string()))?;

    let old = config.get("global-shortcut").unwrap_or_else(|| DEFAULT_SHORTCUT.to_string());
    let _ = app.global_shortcut().unregister(old.as_str());

    app.global_shortcut().register(parsed)
        .map_err(|e| AppError::General(e.to_string()))?;

    config.set("global-shortcut", &shortcut)?;
    let dir = app.path().app_data_dir().expect("failed to resolve app data dir");
    config.save(&dir)?;

    Ok(parsed.to_string())
}
use arboard::Clipboard;
#[tauri::command]
pub fn copy_to_clipboard(content: String) -> Result<(), AppError> {
    let mut clipboard = Clipboard::new().map_err(|e| AppError::General(e.to_string()))?;
    clipboard
        .set_text(content)
        .map_err(|e| AppError::General(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn check_duplicate(db: State<'_, Db>, url: String, exclude_id: Option<i64>) -> Result<Option<crate::db::Link>, AppError> {
    db.find_by_url(&url, exclude_id)
}

async fn do_check_link(url: &str) -> Result<bool, AppError> {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8));

    // Use system proxy on Windows (same logic as fetcher)
    #[cfg(target_os = "windows")]
    {
        // Check if URL should bypass proxy based on ProxyOverride rules
        if !crate::fetcher::should_bypass_proxy(url) {
            if let Some(proxy_url) = crate::fetcher::get_windows_system_proxy() {
                log::info!("[check_link] using system proxy: {}", proxy_url);
                let proxy = reqwest::Proxy::all(&proxy_url)
                    .map_err(|e| AppError::General(e.to_string()))?;
                builder = builder.proxy(proxy);
            }
        } else {
            log::info!("[check_link] bypassing proxy for internal URL: {}", url);
        }
    }

    let client = builder
        .build()
        .map_err(|e| AppError::General(e.to_string()))?;

    if let Ok(resp) = client.head(url).send().await {
        return Ok(resp.status().is_success());
    }
    // Head failed, try GET
    if let Ok(resp) = client.get(url).send().await {
        return Ok(resp.status().is_success());
    }
    log::warn!("link status check failed for {}", url);
    Ok(false)
}

#[tauri::command]
pub async fn check_link_status(app: AppHandle, url: String) -> Result<bool, AppError> {
    let result = do_check_link(&url).await;
    if let Ok(false) | Err(_) = &result {
        log_fetch_failure(&app, &url, "link status check failed");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bookmark_html_nested_folders() {
        let html = r#"
<!DOCTYPE NETSCAPE-Bookmark-file-1>
<TITLE>Bookmarks</TITLE>
<H1>Bookmarks</H1>
<DL><p>
    <DT><H3 ADD_DATE="1639328511" LAST_MODIFIED="1777320905">书签栏</H3>
    <DL><p>
        <DT><H3 ADD_DATE="1777307977" LAST_MODIFIED="1777320920">测试</H3>
        <DL><p>
            <DT><A HREF="https://www.iconfont.cn/" ADD_DATE="1777308003" ICON="...">扩展程序</A>
            <DT><H3 ADD_DATE="1777307985" LAST_MODIFIED="1777320913">人才</H3>
            <DL><p>
                <DT><A HREF="https://opencode.ai/docs/zh-cn" ADD_DATE="1777320905" ICON="...">简介 | OpenCode</A>
            </DL><p>
        </DL><p>
        <DT><A HREF="https://www.bilibili.com/video/BV1zmSoBnEYM/" ADD_DATE="1777320712" ICON="...">ddd</A>
        <DT><A HREF="https://opencode.ai/docs/zh-cn/lsp/" ADD_DATE="1777320725" ICON="...">LSP 服务器 | OpenCode</A>
    </DL><p>
</DL><p>
"#;

        let entries = parse_bookmark_html(html);
        assert_eq!(entries.len(), 4);

        assert_eq!(entries[0].folder_path, vec!["书签栏", "测试"]);
        assert_eq!(entries[0].url, "https://www.iconfont.cn/");
        assert_eq!(entries[0].title, "扩展程序");

        assert_eq!(entries[1].folder_path, vec!["书签栏", "测试", "人才"]);
        assert_eq!(entries[1].url, "https://opencode.ai/docs/zh-cn");
        assert_eq!(entries[1].title, "简介 | OpenCode");

        assert_eq!(entries[2].folder_path, vec!["书签栏"]);
        assert_eq!(entries[2].url, "https://www.bilibili.com/video/BV1zmSoBnEYM/");
        assert_eq!(entries[2].title, "ddd");

        assert_eq!(entries[3].folder_path, vec!["书签栏"]);
        assert_eq!(entries[3].url, "https://opencode.ai/docs/zh-cn/lsp/");
        assert_eq!(entries[3].title, "LSP 服务器 | OpenCode");
    }

    #[test]
    fn test_parse_bookmark_html_skips_non_http() {
        let html = r#"
<DL><p>
    <DT><A HREF="javascript:void(0)">JS Link</A>
    <DT><A HREF="https://example.com">Valid Link</A>
    <DT><A HREF="about:blank">About Blank</A>
</DL><p>
"#;

        let entries = parse_bookmark_html(html);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].url, "https://example.com");
    }

    #[test]
    fn test_extract_href() {
        assert_eq!(
            extract_href(r#"<A HREF="https://example.com" ICON="...">Title</A>"#),
            Some("https://example.com".to_string())
        );
        assert_eq!(
            extract_href(r#"<A HREF="">Empty</A>"#),
            None
        );
        assert_eq!(
            extract_href(r#"<A NAME="anchor">No HREF</A>"#),
            None
        );
    }

    #[test]
    fn test_extract_link_title() {
        assert_eq!(
            extract_link_title(r#"<A HREF="https://example.com">Title Text</A>"#),
            "Title Text"
        );
        assert_eq!(
            extract_link_title(r#"<A HREF="https://example.com">  Trimmed  </A>"#),
            "Trimmed"
        );
    }

    #[test]
    fn test_extract_folder_name() {
        assert_eq!(
            extract_folder_name(r#"<DT><H3 ADD_DATE="123">Folder Name</H3>"#),
            Some("Folder Name".to_string())
        );
        assert_eq!(
            extract_folder_name(r#"<DT><H3></H3>"#),
            None
        );
    }

    #[test]
    fn test_import_bookmark_entries_creates_tree() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();

        conn.execute_batch(
            "
            CREATE TABLE categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                parent_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );
            CREATE TABLE links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                favicon_url TEXT NOT NULL DEFAULT '',
                og_image_url TEXT NOT NULL DEFAULT '',
                category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                is_broken INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );
            CREATE VIRTUAL TABLE links_fts USING fts5(
                title, description, notes, url,
                content=links, content_rowid=id
            );
            
            CREATE TRIGGER links_ai AFTER INSERT ON links BEGIN
                INSERT INTO links_fts(rowid, title, description, notes, url)
                VALUES (new.id, new.title, new.description, new.notes, new.url);
            END;
            "
        ).unwrap();

        let entries = vec![
            BookmarkEntry {
                url: "https://example.com/1".to_string(),
                title: "Link 1".to_string(),
                favicon: "".to_string(),
                folder_path: vec!["Folder A".to_string(), "Subfolder".to_string()],
            },
            BookmarkEntry {
                url: "https://example.com/2".to_string(),
                title: "Link 2".to_string(),
                favicon: "".to_string(),
                folder_path: vec!["Folder A".to_string()],
            },
            BookmarkEntry {
                url: "https://example.com/3".to_string(),
                title: "Link 3".to_string(),
                favicon: "".to_string(),
                folder_path: vec![],
            },
        ];

        let stats = import_bookmark_entries(&entries, &conn).unwrap();
        assert_eq!(stats.links_imported, 3);
        assert_eq!(stats.categories_created, 2);

        let mut stmt = conn.prepare("SELECT name, parent_id FROM categories ORDER BY id").unwrap();
        let categories: Vec<(String, Option<i64>)> = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].0, "Folder A");
        assert!(categories[0].1.is_none());
        assert_eq!(categories[1].0, "Subfolder");
        assert_eq!(categories[1].1, Some(1));

        let mut stmt = conn.prepare("SELECT url, title, category_id FROM links ORDER BY id").unwrap();
        let links: Vec<(String, String, Option<i64>)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(links.len(), 3);
        assert_eq!(links[0].0, "https://example.com/1");
        assert_eq!(links[0].2, Some(2));
        assert_eq!(links[1].0, "https://example.com/2");
        assert_eq!(links[1].2, Some(1));
        assert_eq!(links[2].0, "https://example.com/3");
        assert!(links[2].2.is_none());
    }

    #[test]
    fn test_import_bookmark_entries_deduplicates_urls() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();

        conn.execute_batch(
            "
            CREATE TABLE categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                parent_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );
            CREATE TABLE links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                favicon_url TEXT NOT NULL DEFAULT '',
                og_image_url TEXT NOT NULL DEFAULT '',
                category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                is_broken INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );
            CREATE VIRTUAL TABLE links_fts USING fts5(
                title, description, notes, url,
                content=links, content_rowid=id
            );
            CREATE TRIGGER links_ai AFTER INSERT ON links BEGIN
                INSERT INTO links_fts(rowid, title, description, notes, url)
                VALUES (new.id, new.title, new.description, new.notes, new.url);
            END;
            "
        ).unwrap();

        let entries = vec![
            BookmarkEntry {
                url: "https://example.com".to_string(),
                title: "First".to_string(),
                favicon: "".to_string(),
                folder_path: vec!["Folder".to_string()],
            },
            BookmarkEntry {
                url: "https://example.com".to_string(),
                title: "Duplicate".to_string(),
                favicon: "".to_string(),
                folder_path: vec!["Other".to_string()],
            },
        ];

        let stats = import_bookmark_entries(&entries, &conn).unwrap();
        assert_eq!(stats.links_imported, 1);
        assert_eq!(stats.categories_created, 2);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM links", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
