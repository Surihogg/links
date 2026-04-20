mod commands;
mod db;
mod fetcher;

use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _event, _shortcut| {
                    if let Some(window) = app.get_webview_window("quick-add") {
                        let _ = window.set_focus();
                    } else {
                        let _ = tauri::WebviewWindowBuilder::new(
                            app,
                            "quick-add",
                            tauri::WebviewUrl::App("index.html".into()),
                        )
                        .title("快速添加")
                        .inner_size(400.0, 350.0)
                        .center()
                        .focused(true)
                        .build();
                    }
                })
                .build(),
        )
        .setup(|app| {
            commands::init_db(&app.handle().clone())?;
            let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyL);
            app.global_shortcut().register(shortcut)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::links_list,
            commands::links_create,
            commands::links_update,
            commands::links_delete,
            commands::links_search,
            commands::categories_list,
            commands::categories_create,
            commands::categories_update,
            commands::categories_delete,
            commands::tags_list,
            commands::tags_autocomplete,
            commands::export_links,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
