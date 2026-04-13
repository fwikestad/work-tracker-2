use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, Wry,
};
use crate::{AppState, services::session_service};

const ICON_SIZE: u32 = 32;

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
        .icon_as_template(true)
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
    Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "current-work", work_order, false, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "pause-resume", pause_label, true, None::<&str>)?,
            &MenuItem::with_id(app, "switch-project", "Switch Project...", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "open-app", "Open Work Tracker", true, None::<&str>)?,
            &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
        ],
    )
}

fn on_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "quit" => {
            // Stop any active session before quitting so time is not lost.
            // Scoped block ensures MutexGuard is dropped before app.exit().
            {
                let state = app.state::<AppState>();
                if let Ok(conn) = state.db.lock() {
                    let _ = session_service::stop_active_session(&conn);
                };
            }
            app.exit(0);
        }
        "open-app" => show_main_window(app),
        "pause-resume" => toggle_pause_resume(app),
        "switch-project" => {
            show_main_window(app);
            let _ = app.emit("open-search-switch", ());
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
