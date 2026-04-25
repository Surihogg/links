mod commands;
mod config;
mod db;
mod fetcher;
#[allow(dead_code)]
mod normalize;

use std::path::PathBuf;
use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use std::str::FromStr;

fn static_data_dir() -> PathBuf {
    dirs::data_dir()
        .expect("failed to resolve system data dir")
        .join("com.links.desktop")
}

fn data_dir(app: &tauri::AppHandle) -> PathBuf {
    app.path().app_data_dir().expect("failed to resolve app data dir")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Ensure data directory exists before any plugin tries to write to it.
    // On Windows, missing directory during log plugin init can cause a hang.
    if let Some(dir) = dirs::data_dir() {
        let _ = std::fs::create_dir_all(dir.join("com.links.desktop"));
    }

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .max_file_size(10 * 1024 * 1024)
                .level(log::LevelFilter::Info)
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Folder { path: static_data_dir(), file_name: None }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _event, _shortcut| {
                    let window = if let Some(w) = app.get_webview_window("quick-add") {
                        w
                    } else {
                        let config = app.config().app.windows.iter().find(|w| w.label == "quick-add").unwrap();
                        let builder = tauri::WebviewWindowBuilder::from_config(app, config).unwrap();
                        #[cfg(target_os = "macos")]
                        let builder = builder.decorations(true).title_bar_style(tauri::TitleBarStyle::Overlay).title(" ");
                        #[cfg(target_os = "windows")]
                        let builder = builder.decorations(false);
                        builder.build().unwrap()
                    };
                    let _ = window.show();
                    let _ = window.set_focus();
                })
                .build(),
        )
        .setup(|app| {
            let dir = data_dir(&app.handle().clone());
            log::info!("[startup] data_dir = {:?}", dir);

            let cfg = config::Config::load(&dir).unwrap_or_else(|_| config::Config::empty());
            let mut window_width: f64 = 900.0;
            let mut window_height: f64 = 600.0;
            if let Some(size_val) = cfg.get_value("window-size") {
                if let (Some(w), Some(h)) = (size_val["width"].as_f64(), size_val["height"].as_f64()) {
                    window_width = w;
                    window_height = h;
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.set_size(tauri::LogicalSize::new(w, h));
                    }
                }
            }
            if let Some(pos_val) = cfg.get_value("window-position") {
                if let (Some(x), Some(y)) = (pos_val["x"].as_f64(), pos_val["y"].as_f64()) {
                    let x = x as i32;
                    let y = y as i32;
                    let w = window_width as i32;
                    let h = window_height as i32;
                    let mut visible = false;
                    if let Ok(monitors) = app.available_monitors() {
                        for monitor in monitors {
                            let area = monitor.work_area();
                            let ax = area.position.x;
                            let ay = area.position.y;
                            let aw = area.size.width as i32;
                            let ah = area.size.height as i32;
                            if x + w > ax && x < ax + aw && y + h > ay && y < ay + ah {
                                visible = true;
                                break;
                            }
                        }
                    }
                    if visible {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(x, y)));
                        }
                    }
                }
            }
            let shortcut_str = cfg.get("global-shortcut").unwrap_or_else(|| commands::DEFAULT_SHORTCUT.to_string());
            app.manage(cfg);

            log::info!("[startup] initializing database...");
            if let Err(e) = commands::init_db(&app.handle().clone()) {
                log::error!("[startup] database initialization failed: {}. App will run without database.", e);
            } else {
                log::info!("[startup] database initialized");
            }

            let show_i = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let icon = app.default_window_icon().cloned().unwrap();
            TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("Links")
                .show_menu_on_left_click(false)
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

            let shortcut = Shortcut::from_str(&shortcut_str).expect("failed to parse shortcut");
            app.global_shortcut().register(shortcut)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::links_list,
            commands::links_create,
            commands::links_update,
            commands::links_delete,
            commands::links_search,
            commands::copy_to_clipboard,
            commands::check_duplicate,
            commands::check_link_status,
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
            commands::get_shortcut,
            commands::set_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
