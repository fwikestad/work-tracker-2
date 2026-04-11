use tauri::Manager;
use std::sync::Mutex;

mod commands;
mod db;
mod models;
mod services;

pub use db::AppState;

#[tauri::command]
fn update_tray_tooltip(app: tauri::AppHandle, tooltip: String) -> Result<(), String> {
    // The tray icon configured in tauri.conf.json uses "main" as the default ID
    app.tray_by_id("main")
        .ok_or_else(|| "Tray not found".to_string())?
        .set_tooltip(Some(&tooltip))
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("work_tracker.db");
            let conn = db::initialize(&db_path)?;
            app.manage(AppState { db: Mutex::new(conn) });
            
            // Tray icon is auto-created from tauri.conf.json
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::customers::create_customer,
            commands::customers::list_customers,
            commands::customers::update_customer,
            commands::customers::archive_customer,
            commands::work_orders::create_work_order,
            commands::work_orders::list_work_orders,
            commands::work_orders::update_work_order,
            commands::work_orders::archive_work_order,
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
            update_tray_tooltip,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {}
        });
}
