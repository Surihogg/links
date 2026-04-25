use crate::config::Config;
use crate::db::{
    AppError, CreateCategoryPayload, CreateLinkPayload, Db, ExportParams, ListLinksParams,
    PaginatedResult, SearchParams, UpdateCategoryPayload, UpdateLinkPayload,
};
use rusqlite::params;
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

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

#[tauri::command]
pub fn import_bookmarks(db: State<'_, Db>) -> Result<u32, AppError> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("书签文件", &["html", "htm"])
        .set_title("导入浏览器书签")
        .pick_file()
    else {
        return Ok(0);
    };

    let html = std::fs::read_to_string(&path)?;
    let doc = scraper::Html::parse_document(&html);

    let conn = db.0.lock().unwrap();
    let mut count: u32 = 0;

    for node in doc.select(&scraper::Selector::parse("a").unwrap()) {
        let Some(href) = node.value().attr("href") else { continue };
        if href.is_empty() || !href.starts_with("http") {
            continue;
        }

        let title = node.text().collect::<String>().trim().to_string();
        let favicon = node.value().attr("icon").unwrap_or("").to_string();

        if conn.query_row(
            "SELECT COUNT(*) FROM links WHERE url = ?",
            rusqlite::params![href],
            |r| r.get::<_, i64>(0),
        ).unwrap_or(0) > 0 {
            continue;
        }

        conn.execute(
            "INSERT INTO links (url, title, favicon_url) VALUES (?, ?, ?)",
            rusqlite::params![href, title, favicon],
        )?;
        let id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO links_fts (rowid, title, description, notes, url) VALUES (?, '', '', '', ?)",
            rusqlite::params![id, href],
        ).ok();

        count += 1;
    }

    Ok(count)
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

pub const DEFAULT_SHORTCUT: &str = "CmdOrCtrl+Shift+L";

#[tauri::command]
pub fn get_shortcut(config: State<'_, Config>) -> Result<String, AppError> {
    Ok(config.get("global-shortcut").unwrap_or_else(|| DEFAULT_SHORTCUT.to_string()))
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
        if let Some(proxy_url) = crate::fetcher::get_windows_system_proxy() {
            log::info!("[check_link] using system proxy: {}", proxy_url);
            let proxy = reqwest::Proxy::all(&proxy_url)
                .map_err(|e| AppError::General(e.to_string()))?;
            builder = builder.proxy(proxy);
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
