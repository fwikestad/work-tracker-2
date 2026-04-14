use tauri::Manager;
use tauri::Emitter;
use std::sync::Mutex;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod commands;
pub mod db;
pub mod models;
pub mod services;
pub mod tray;

pub use db::AppState;

/// Tracks always-on-top widget mode state and the previous window geometry for restoration.
pub struct WindowState {
    pub is_widget_mode: bool,
    pub previous_size: Option<(u32, u32)>,
    pub previous_position: Option<(i32, i32)>,
}

/// Update the tray icon, tooltip, and menu to reflect the current session state.
/// Frontend calls this after every start/stop/pause/resume/switch action.
///
/// - `work_order_name`: Some("...") if a session is active, None if stopped.
/// - `is_paused`: true when the active session is paused.
#[tauri::command]
fn update_tray_state(
    app: tauri::AppHandle,
    work_order_name: Option<String>,
    is_paused: bool,
) -> Result<(), String> {
    tray::update_tray_state(&app, work_order_name.as_deref(), is_paused)
        .map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        match shortcut.key {
                            Code::KeyS => {
                                // Ctrl+Shift+S: bring window to front + open search overlay
                                if let Some(win) = app.get_webview_window("main") {
                                    let _ = win.show();
                                    let _ = win.unminimize();
                                    let _ = win.set_focus();
                                }
                                let _ = app.emit("focus-search", ());
                            }
                            Code::KeyW => {
                                // Ctrl+Alt+W: toggle widget mode — let frontend call toggle_widget_mode
                                let _ = app.emit("toggle-widget-mode", ());
                            }
                            _ => {}
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            let app_dir = app.path().app_data_dir()
                .map_err(|e| Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Failed to get app data dir: {}", e)
                )) as Box<dyn std::error::Error>)?;
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("work_tracker.db");
            let conn = db::initialize(&db_path)?;
            app.manage(AppState { db: Mutex::new(conn) });
            app.manage(Mutex::new(WindowState {
                is_widget_mode: false,
                previous_size: None,
                previous_position: None,
            }));

            // Register Ctrl+Shift+S → bring window to front + open search overlay
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyS);
            app.handle().global_shortcut().register(shortcut)
                .map_err(|e| format!("Failed to register global shortcut: {}", e))?;

            // Register Ctrl+Alt+W → toggle always-on-top widget mode
            let widget_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyW);
            app.handle().global_shortcut().register(widget_shortcut)
                .map_err(|e| format!("Failed to register widget shortcut: {}", e))?;

            // Set up system tray with icon and right-click menu
            tray::setup_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::customers::create_customer,
            commands::customers::list_customers,
            commands::customers::update_customer,
            commands::customers::archive_customer,
            commands::customers::unarchive_customer,
            commands::work_orders::create_work_order,
            commands::work_orders::list_work_orders,
            commands::work_orders::update_work_order,
            commands::work_orders::archive_work_order,
            commands::work_orders::unarchive_work_order,
            commands::work_orders::toggle_favorite,
            commands::sessions::start_session,
            commands::sessions::stop_session,
            commands::sessions::get_active_session,
            commands::sessions::update_session,
            commands::sessions::list_sessions,
            commands::sessions::delete_session,
            commands::sessions::quick_add,
            commands::sessions::recover_session,
            commands::sessions::discard_orphan_session,
            commands::sessions::pause_session,
            commands::sessions::resume_session,
            commands::sessions::update_heartbeat,
            commands::sessions::check_for_orphan_session,
            commands::reports::get_daily_summary,
            commands::reports::get_recent_work_orders,
            commands::reports::export_csv,
            commands::reports::get_report,
            update_tray_state,
            commands::window::toggle_widget_mode,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {}
        });
}
