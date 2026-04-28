mod commands;
mod config;
mod db;
mod fetcher;
mod http_server;
#[allow(dead_code)]
mod normalize;

use std::path::PathBuf;
use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri_plugin_deep_link::DeepLinkExt;
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
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // 单实例回调：第二个实例启动时，将焦点转到主窗口
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
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
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_process::init())?;

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

            // 深度链接处理：支持浏览器 Bookmarklet 一键收藏
            app.manage(commands::PendingDeepLink(std::sync::Mutex::new(None)));

            // 启动本地 HTTP 服务（固定端口 48927，token 持久化到 config）
            {
                let cfg = app.state::<Config>();
                let existing_token = cfg.get("http-server-token");
                let http_handle = app.handle().clone();
                match http_server::start(existing_token, move |url, title| {
                    handle_browser_capture(&http_handle, &url, &title);
                }) {
                    Ok((port, token)) => {
                        log::info!("[http] 本地服务已启动: 127.0.0.1:{}", port);
                        // 持久化 token 到 config，跨重启复用（bookmarklet 不用重新拖）
                        let _ = cfg.set("http-server-token", &serde_json::to_string(&token).unwrap_or_default());
                        let dir = data_dir(&app.handle());
                        let _ = cfg.save(&dir);
                        app.manage(commands::LocalServerInfo { port, token });
                    }
                    Err(e) => {
                        log::error!("[http] 本地服务启动失败: {}", e);
                        // 即使启动失败也注册一个占位，避免前端命令找不到 state
                        app.manage(commands::LocalServerInfo { port: 0, token: String::new() });
                    }
                }
            }

            {
                let app_handle = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    if let Some(url_str) = event.urls().first() {
                        handle_deep_link(&app_handle, url_str.as_str());
                    }
                });

                // 冷启动时通过 get_current 捕获（Windows 必需，macOS 也作为兜底）
                if let Ok(Some(urls)) = app.deep_link().get_current() {
                    if let Some(url) = urls.first() {
                        handle_deep_link(app.handle(), url.as_str());
                    }
                }

                // 开发模式下运行时注册协议（Windows/Linux 需要运行时注册）
                #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
                {
                    if let Err(e) = app.deep_link().register_all() {
                        log::warn!("[deep-link] 运行时注册失败: {}", e);
                    }
                }
            }

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
            commands::get_system_proxy,
            commands::pop_pending_deep_link,
            commands::get_local_server_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 纯逻辑版本的深度链接解析（不依赖 Tauri runtime），用于测试
fn parse_deep_link(raw_url: &str) -> Option<(String, String)> {
    let parsed = url::Url::parse(raw_url).ok()?;
    if parsed.scheme() != "links" {
        return None;
    }
    let host = parsed.host_str().unwrap_or("");
    let path = parsed.path();
    if host != "add" && path != "/add" && path != "add" {
        return None;
    }
    let params: std::collections::HashMap<String, String> = parsed
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let link_url = params.get("url")?.clone();
    let link_title = params.get("title").cloned().unwrap_or_default();
    Some((link_url, link_title))
}

fn handle_deep_link(app: &tauri::AppHandle, raw_url: &str) {
    let (link_url, link_title) = match parse_deep_link(raw_url) {
        Some(result) => result,
        None => {
            log::warn!("[deep-link] 无法解析: {}", raw_url);
            return;
        }
    };
    log::info!("[deep-link] 收藏链接: url={}, title={}", link_url, link_title);
    handle_browser_capture(app, &link_url, &link_title);
}

fn handle_browser_capture(app: &tauri::AppHandle, link_url: &str, link_title: &str) {
    let payload = serde_json::json!({
        "url": link_url,
        "title": link_title,
    });

    if let Some(pending) = app.try_state::<commands::PendingDeepLink>() {
        let mut guard = pending.0.lock().unwrap();
        // 双轨并行去重：如果已有未消费的相同 URL，不重复写入
        let is_dup = guard.as_ref().map_or(false, |existing| {
            existing.get("url").and_then(|v| v.as_str()) == Some(link_url)
        });
        if !is_dup {
            *guard = Some(payload);
        }
    }

    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
    }
    if let Some(window) = app.get_webview_window("quick-add") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.set_always_on_top(true);
    }
    let _ = app.emit("quick-add-shown", ());
}

#[cfg(test)]
mod deep_link_tests {
    use super::*;

    #[test]
    fn test_parse_basic_deep_link() {
        let result = parse_deep_link("links://add?url=https%3A%2F%2Fexample.com&title=My%20Page");
        assert_eq!(result, Some(("https://example.com".into(), "My Page".into())));
    }

    #[test]
    fn test_parse_deep_link_with_special_chars() {
        let result = parse_deep_link(
            "links://add?url=https%3A%2F%2Fexample.com%2Fpath%3Fq%3Dhello%26lang%3D%E4%B8%AD%E6%96%87&title=Test%20%26%20More",
        );
        assert_eq!(
            result,
            Some(("https://example.com/path?q=hello&lang=中文".into(), "Test & More".into()))
        );
    }

    #[test]
    fn test_parse_deep_link_no_title() {
        let result = parse_deep_link("links://add?url=https%3A%2F%2Fexample.com");
        assert_eq!(result, Some(("https://example.com".into(), "".into())));
    }

    #[test]
    fn test_parse_deep_link_wrong_scheme() {
        let result = parse_deep_link("http://add?url=https://example.com");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_deep_link_wrong_host() {
        let result = parse_deep_link("links://other?url=https://example.com");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_deep_link_missing_url() {
        let result = parse_deep_link("links://add?title=NoURL");
        assert_eq!(result, None);
    }
}
