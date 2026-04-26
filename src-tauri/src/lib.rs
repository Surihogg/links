mod commands;
mod config;
mod db;
mod fetcher;
#[allow(dead_code)]
mod normalize;

use std::path::PathBuf;
use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use std::str::FromStr;
use crate::config::Config;

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
                .with_handler(move |app, shortcut, _event| {
                    let cfg = app.state::<Config>();
                    let quick_add_str = cfg
                        .get("global-shortcut")
                        .unwrap_or_else(|| commands::DEFAULT_SHORTCUT.to_string());
                    let main_str = cfg
                        .get("main-shortcut")
                        .unwrap_or_else(|| commands::DEFAULT_MAIN_SHORTCUT.to_string());
                    drop(cfg);

                    let quick_add: Shortcut = match quick_add_str.parse() {
                        Ok(s) => s,
                        Err(_) => return,
                    };
                    let main_shortcut: Shortcut = match main_str.parse() {
                        Ok(s) => s,
                        Err(_) => return,
                    };

                    if *shortcut == quick_add {
                        if let Some(window) = app.get_webview_window("quick-add") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = app.emit("quick-add-shown", ());
                        }
                    } else if *shortcut == main_shortcut {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                            let _ = app.emit("main-shown", ());
                        }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let dir = data_dir(&app.handle().clone());
            log::info!("[startup] data_dir = {:?}", dir);

            let cfg = config::Config::load(&dir).unwrap_or_else(|_| config::Config::empty());

            // Windows 上 quick-add 窗口需要移除原生标题栏（macOS 通过 titleBarStyle: Overlay 隐藏）
            #[cfg(target_os = "windows")]
            if let Some(quick_add) = app.get_webview_window("quick-add") {
                let _ = quick_add.set_decorations(false);
            }

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
            // Initialize main shortcut on startup as well
            let main_shortcut_str = cfg
                .get("main-shortcut")
                .unwrap_or_else(|| commands::DEFAULT_MAIN_SHORTCUT.to_string());
            let main_shortcut = Shortcut::from_str(&main_shortcut_str).unwrap_or_else(|_| {
                // Fallback to default if parsing fails
                Shortcut::from_str(commands::DEFAULT_MAIN_SHORTCUT).expect("invalid default shortcut")
            });
            app.global_shortcut().register(main_shortcut).ok();
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
            commands::tags_update,
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
            commands::get_main_shortcut,
            commands::set_main_shortcut,
            commands::exit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
