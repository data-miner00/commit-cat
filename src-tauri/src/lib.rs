mod commands;
mod events;
mod models;
pub mod platform;
mod services;
mod utils;

use tauri::menu::MenuBuilder;
use tauri::Manager;

#[cfg(target_os = "macos")]
#[allow(deprecated)]
use cocoa::appkit::{NSColor, NSWindow};
#[cfg(target_os = "macos")]
#[allow(deprecated)]
use cocoa::base::{id, nil, NO};

#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub(crate) fn setup_macos_window(window: &tauri::WebviewWindow) {
    #[allow(unused_imports)]
    use tauri::Emitter;

    if let Ok(ns_window) = window.ns_window() {
        unsafe {
            let ns_win = ns_window as id;
            // 윈도우를 불투명하지 않게 설정
            ns_win.setOpaque_(NO);
            // 배경색을 완전 투명으로 설정
            ns_win.setBackgroundColor_(NSColor::clearColor(nil));
            // 그림자 비활성화 (잔상 방지)
            ns_win.setHasShadow_(NO);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // ── Plugins ──
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        // ── State ──
        .manage(services::activity::SharedActivityState::default())
        .manage(std::sync::Arc::new(commands::pomodoro::PomodoroState::default()))
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Initialize local data storage
            services::storage::init(&app_handle)?;

            // 날짜가 바뀌었으면 DailySummary 리셋 (앱 시작 시 1회)
            let _ = services::storage::check_daily_reset(&app_handle);

            // macOS 투명 윈도우 설정
            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("cat-overlay") {
                setup_macos_window(&window);
            }

            // 서브 고양이 윈도우 투명 설정은 setup_sub_cat_window 커맨드로 처리

            // ── config 기반 트레이 아이콘 설정 ──
            if let Some(tray) = app.tray_by_id("main-tray") {
                let menu = MenuBuilder::new(app)
                    .text("settings", "Settings...")
                    .text("check-update", "Check for Updates")
                    .separator()
                    .text("quit", "Quit")
                    .build()?;
                tray.set_menu(Some(menu))?;
                tray.set_show_menu_on_left_click(false)?;

                // Streak 정보로 트레이 툴팁 설정
                if let Ok(data) = services::storage::load(&app_handle) {
                    let streak = data.cat.streak_days;
                    if streak > 0 {
                        let _ =
                            tray.set_tooltip(Some(&format!("CommitCat — {} day streak", streak)));
                    } else {
                        let _ = tray.set_tooltip(Some("CommitCat"));
                    }
                }

                // 좌클릭: 고양이 보이기/숨기기
                tray.on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("cat-overlay") {
                            let should_hide = window.is_visible().unwrap_or(false);
                            if should_hide {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                            }
                            // Also toggle sub-cat windows
                            for (label, win) in app.webview_windows() {
                                if label.starts_with("cat-sub-") {
                                    if should_hide {
                                        let _ = win.hide();
                                    } else {
                                        let _ = win.show();
                                    }
                                }
                            }
                        }
                    }
                });

                // 메뉴 이벤트: 색상 변경 + 종료
                tray.on_menu_event(|app, event| {
                    use tauri::Emitter;
                    match event.id().as_ref() {
                        "cat-orange" => {
                            let _ = app.emit("change-cat-color", "orange");
                        }
                        "cat-brown" => {
                            let _ = app.emit("change-cat-color", "brown");
                        }
                        "cat-white" => {
                            let _ = app.emit("change-cat-color", "white");
                        }
                        "settings" => {
                            // 이미 열려있으면 포커스, 아니면 새로 생성
                            if let Some(win) = app.get_webview_window("settings") {
                                let _ = win.set_focus();
                            } else {
                                let _ = tauri::webview::WebviewWindowBuilder::new(
                                    app,
                                    "settings",
                                    tauri::WebviewUrl::App("/".into()),
                                )
                                .title("CommitCat Settings")
                                .inner_size(500.0, 600.0)
                                .center()
                                .resizable(false)
                                .build();
                            }
                        }
                        "check-update" => {
                            let handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = services::update::check_update(&handle).await;
                            });
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                });
            }

            // 활동 모니터 시작
            let monitor_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                services::activity::start_monitor(monitor_handle).await;
            });

            // Git 커밋 감지 시작
            let git_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                services::git::start_watcher(git_handle).await;
            });

            // GitHub 폴링 시작
            let gh_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                services::github::start_github_watcher(gh_handle).await;
            });

            // Docker 활동 감지 시작
            let docker_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                services::docker::start_watcher(docker_handle).await;
            });

            // 플러그인 HTTP 서버 시작 (VS Code 등 외부 연동)
            let server_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                services::plugin_server::start_server(server_handle).await;
            });

            // 업데이트 체크 시작
            let update_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                services::update::start_update_checker(update_handle).await;
            });

            // 클라우드 싱크 서비스 시작 (5분 간격, opt-in)
            let sync_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                services::cloud_sync::start_sync_service(sync_handle).await;
            });

            Ok(())
        })
        // ── Commands (frontend ↔ backend) ──
        .invoke_handler(tauri::generate_handler![
            // Cat state
            commands::cat::get_cat_state,
            commands::cat::click_cat,
            commands::cat::quit_app,
            commands::cat::setup_sub_cat_window,
            // Activity
            commands::activity::get_today_summary,
            commands::activity::get_today_events,
            commands::activity::get_coding_status,
            commands::activity::add_coding_minute,
            // Growth
            commands::growth::get_level_info,
            commands::growth::get_exp_breakdown,
            // Pomodoro
            commands::pomodoro::start_pomodoro,
            commands::pomodoro::stop_pomodoro,
            commands::pomodoro::get_pomodoro_status,
            // Git
            commands::git::get_today_commits,
            commands::git::register_repo,
            commands::git::get_watched_repos,
            commands::git::remove_repo,
            commands::git::clone_repo,
            // Settings
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::update_settings_patch,
            // Fullscreen
            commands::fullscreen::check_fullscreen,
            // XP
            commands::xp::get_xp_status,
            commands::xp::add_xp,
            commands::xp::get_streak_info,
            commands::xp::equip_hat,
            commands::xp::get_hat_info,
            commands::xp::check_event_equip,
            commands::xp::unlock_all_hats,
            // GitHub
            commands::github::verify_github_token,
            commands::github::disconnect_github,
            // Update
            commands::update::check_for_update,
            // AI
            commands::ai::chat_with_cat,
            commands::ai::get_ai_provider_catalog,
            commands::ai::get_codex_provider_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Commit Cat");
}
