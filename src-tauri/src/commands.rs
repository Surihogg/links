use crate::db::{
    AppError, CreateCategoryPayload, CreateLinkPayload, Db, ExportParams, ListLinksParams,
    PaginatedResult, UpdateCategoryPayload, UpdateLinkPayload,
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

#[tauri::command]
pub fn links_list(
    _app: AppHandle,
    db: State<'_, Db>,
    params: ListLinksParams,
) -> Result<PaginatedResult<crate::db::Link>, AppError> {
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
    tauri::async_runtime::spawn(async move {
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
pub fn links_search(db: State<'_, Db>, query: String) -> Result<Vec<crate::db::Link>, AppError> {
    db.search_links(&query)
}

#[tauri::command]
pub fn categories_list(db: State<'_, Db>) -> Result<Vec<crate::db::Category>, AppError> {
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
    db.export_links(&params)
}
