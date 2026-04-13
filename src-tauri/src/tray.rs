use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, Wry,
};
use rusqlite::Connection;
use crate::{AppState, services::session_service};

const ICON_SIZE: u32 = 32;

// ---------------------------------------------------------------------------
// Tray Menu Data Structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct WorkOrderSummary {
    pub id: String,
    pub name: String,
    pub customer_name: String,
    pub is_favorite: bool,
}

#[derive(Debug)]
pub struct TrayMenuData {
    pub favorites: Vec<WorkOrderSummary>,
    pub recent: Vec<WorkOrderSummary>,
}

/// Query the database for favorites and recent work orders to populate the dynamic tray menu.
pub fn get_tray_menu_data(conn: &Connection) -> Result<TrayMenuData, rusqlite::Error> {
    // Get favorites (up to 5)
    let mut stmt = conn.prepare(
        "SELECT wo.id, wo.name, c.name AS customer_name, wo.is_favorite
         FROM work_orders wo
         JOIN customers c ON wo.customer_id = c.id
         WHERE wo.is_favorite = 1 AND wo.archived_at IS NULL
         ORDER BY wo.updated_at DESC
         LIMIT 5"
    )?;
    
    let favorites = stmt.query_map([], |row| {
        Ok(WorkOrderSummary {
            id: row.get(0)?,
            name: row.get(1)?,
            customer_name: row.get(2)?,
            is_favorite: row.get::<_, i64>(3)? == 1,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    
    // Get recent (up to 10, excluding favorites)
    let mut stmt = conn.prepare(
        "SELECT wo.id, wo.name, c.name AS customer_name, wo.is_favorite
         FROM work_orders wo
         JOIN customers c ON wo.customer_id = c.id
         LEFT JOIN recent_work_orders rwo ON wo.id = rwo.work_order_id
         WHERE wo.is_favorite = 0 AND wo.archived_at IS NULL AND rwo.work_order_id IS NOT NULL
         ORDER BY rwo.last_used_at DESC
         LIMIT 10"
    )?;
    
    let recent = stmt.query_map([], |row| {
        Ok(WorkOrderSummary {
            id: row.get(0)?,
            name: row.get(1)?,
            customer_name: row.get(2)?,
            is_favorite: row.get::<_, i64>(3)? == 1,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    
    Ok(TrayMenuData { favorites, recent })
}

// ---------------------------------------------------------------------------
// Tray Setup
// ---------------------------------------------------------------------------

/// Set up the system tray icon with right-click menu and single-click toggle.
///
/// Menu layout:
///   [Current work order or "Not tracking"]  ← disabled label
///   ─────────────────────────────────────────
///   Pause / Resume  (contextual)
///   Switch Project...
///   ─────────────────────────────────────────
///   Open Work Tracker
///   Quit
///
/// Single left-click: toggle pause/resume if a session is active.
pub fn setup_tray(app: &App) -> tauri::Result<()> {
    let handle = app.handle();
    let menu = build_menu(handle, "Not tracking", false)?;

    let _ = TrayIconBuilder::with_id("main")
        .icon(make_circle_icon(107, 114, 128)) // grey = stopped
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Work Tracker 2 — Not tracking")
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_tray_icon_event)
        .build(app)?;

    Ok(())
}

/// Update the tray icon, tooltip, and menu to reflect current session state.
/// Called via the `update_tray_state` IPC command after any session change.
pub fn update_tray_state(
    app: &AppHandle,
    work_order_name: Option<&str>,
    is_paused: bool,
) -> tauri::Result<()> {
    let tray = match app.tray_by_id("main") {
        Some(t) => t,
        None => return Ok(()),
    };

    // active=green (#16a34a), paused=amber (#f59e0b), stopped=grey (#6b7280)
    let icon = match work_order_name {
        None                  => make_circle_icon(107, 114, 128),
        Some(_) if is_paused  => make_circle_icon(245, 158, 11),
        Some(_)               => make_circle_icon(22, 163, 74),
    };
    tray.set_icon(Some(icon))?;

    let tooltip = match work_order_name {
        None       => "Work Tracker 2 — Not tracking".to_string(),
        Some(name) if is_paused => format!("Work Tracker 2 — ⏸ {}", name),
        Some(name) => format!("Work Tracker 2 — ▶ {}", name),
    };
    tray.set_tooltip(Some(&tooltip))?;

    let menu = build_menu(app, work_order_name.unwrap_or("Not tracking"), is_paused)?;
    tray.set_menu(Some(menu))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Build a 32×32 RGBA image with a colored anti-aliased circle.
fn make_circle_icon(r: u8, g: u8, b: u8) -> tauri::image::Image<'static> {
    let size = ICON_SIZE as usize;
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;
    let rad = size as f32 * 0.4;

    let mut rgba = vec![0u8; size * size * 4];
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let alpha = if dist <= rad - 1.0 {
                255
            } else if dist <= rad {
                ((rad - dist) * 255.0) as u8
            } else {
                0
            };
            let i = (y * size + x) * 4;
            rgba[i]     = r;
            rgba[i + 1] = g;
            rgba[i + 2] = b;
            rgba[i + 3] = alpha;
        }
    }
    tauri::image::Image::new_owned(rgba, ICON_SIZE, ICON_SIZE)
}

fn build_menu(app: &AppHandle, work_order: &str, is_paused: bool) -> tauri::Result<Menu<Wry>> {
    let pause_label = if is_paused { "Resume" } else { "Pause" };
    
    // Get tray menu data from database
    let menu_data = {
        let state = app.state::<AppState>();
        state.db.lock()
            .ok()
            .and_then(|conn| get_tray_menu_data(&conn).ok())
    };
    
    let (favorites, recent) = match menu_data {
        Some(data) => (data.favorites, data.recent),
        None => (Vec::new(), Vec::new()),
    };
    
    // Build menu items dynamically
    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<Wry>>> = vec![
        Box::new(MenuItem::with_id(app, "current-work", work_order, false, None::<&str>)?),
        Box::new(PredefinedMenuItem::separator(app)?),
    ];
    
    // Add favorites section if any exist
    if !favorites.is_empty() {
        items.push(Box::new(MenuItem::with_id(app, "favorites-header", "⭐ Favorites", false, None::<&str>)?));
        for wo in &favorites {
            let label = format!("  • {} ({})", wo.name, wo.customer_name);
            let menu_id = format!("switch-{}", wo.id);
            items.push(Box::new(MenuItem::with_id(app, &menu_id, label, true, None::<&str>)?));
        }
        items.push(Box::new(PredefinedMenuItem::separator(app)?));
    }
    
    // Add recent section if any exist
    if !recent.is_empty() {
        items.push(Box::new(MenuItem::with_id(app, "recent-header", "⏱ Recent", false, None::<&str>)?));
        for wo in &recent {
            let label = format!("  • {} ({})", wo.name, wo.customer_name);
            let menu_id = format!("switch-{}", wo.id);
            items.push(Box::new(MenuItem::with_id(app, &menu_id, label, true, None::<&str>)?));
        }
        items.push(Box::new(PredefinedMenuItem::separator(app)?));
    }
    
    // Add standard menu items
    items.push(Box::new(MenuItem::with_id(app, "pause-resume", pause_label, true, None::<&str>)?));
    items.push(Box::new(MenuItem::with_id(app, "switch-project", "Switch Project...", true, None::<&str>)?));
    items.push(Box::new(MenuItem::with_id(app, "view-reports", "View Reports", true, None::<&str>)?));
    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(app, "open-app", "Open Work Tracker", true, None::<&str>)?));
    items.push(Box::new(MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?));
    
    // Convert Vec<Box<dyn IsMenuItem>> to Vec<&dyn IsMenuItem>
    let item_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> = items.iter().map(|item| item.as_ref()).collect();
    
    Menu::with_items(app, &item_refs)
}

fn on_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let event_id = event.id().as_ref();
    
    // Handle switch-{id} pattern
    if event_id.starts_with("switch-") {
        let work_order_id = &event_id["switch-".len()..];
        
        // Switch to the selected work order
        let success = {
            let state = app.state::<AppState>();
            let result = state.db.lock()
                .ok()
                .and_then(|conn| session_service::switch_to_work_order(&conn, work_order_id).ok());
            result.is_some()
        };
        
        // Emit event to frontend to update UI
        if success {
            let _ = app.emit("tray-action", "switch");
        }
        return;
    }
    
    // Handle other menu events
    match event_id {
        "quit" => {
            // Stop any active session before quitting so time is not lost.
            // Scoped block ensures MutexGuard is dropped before app.exit().
            {
                let state = app.state::<AppState>();
                if let Ok(conn) = state.db.lock() {
                    let _ = session_service::stop_active_session(&conn);
                };
            }
            // Destroy the window before exiting to prevent
            // Chrome_WidgetWin_0 class unregister error 1412 on Windows
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.destroy();
            }
            app.exit(0);
        }
        "open-app" => show_main_window(app),
        "pause-resume" => toggle_pause_resume(app),
        "switch-project" => {
            show_main_window(app);
            let _ = app.emit("open-search-switch", ());
        }
        "view-reports" => {
            show_main_window(app);
            let _ = app.emit("open-reports", ());
        }
        _ => {}
    }
}

fn on_tray_icon_event(tray: &tauri::tray::TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        toggle_pause_resume(tray.app_handle());
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

fn toggle_pause_resume(app: &AppHandle) {
    // Use a local `result` so the match expression (and its temporaries, including
    // the MutexGuard and State borrow) are fully dropped before `app.emit` is called.
    let did_toggle = {
        let state = app.state::<AppState>();
        let result = match state.db.lock() {
            Ok(conn) => {
                let (has_session, is_paused): (i64, i64) = conn
                    .query_row(
                        "SELECT CASE WHEN session_id IS NOT NULL THEN 1 ELSE 0 END, \
                         COALESCE(is_paused, 0) FROM active_session WHERE id = 1",
                        [],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .unwrap_or((0, 0));

                if has_session == 1 {
                    if is_paused == 1 {
                        let _ = session_service::resume_session(&conn);
                    } else {
                        let _ = session_service::pause_session(&conn);
                    }
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        };
        result
    };

    if did_toggle {
        let _ = app.emit("tray-action", "pause-resume");
    }
}
