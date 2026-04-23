mod commands;
mod db;
mod fetcher;

use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir { file_name: None }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .build(),
        )
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

            if let Some(db) = app.try_state::<db::Db>() {
                if let Ok(Some(size_json)) = db.get_setting("window-size") {
                    if let Ok(size) = serde_json::from_str::<serde_json::Value>(&size_json) {
                        if let (Some(w), Some(h)) = (size["width"].as_f64(), size["height"].as_f64()) {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.set_size(tauri::LogicalSize::new(w, h));
                            }
                        }
                    }
                }
            }

            let show_i = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let icon = app.default_window_icon().cloned().unwrap();
            TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("Links")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

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
            commands::tags_create,
            commands::tags_delete,
            commands::tags_autocomplete,
            commands::export_links,
            commands::open_url,
            commands::save_file,
            commands::fetch_metadata,
            commands::import_bookmarks,
            commands::get_setting,
            commands::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
