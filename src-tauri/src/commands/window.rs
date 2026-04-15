use std::sync::Mutex;
use crate::WindowState;

#[tauri::command]
pub fn resize_widget(
    window: tauri::Window,
    state: tauri::State<'_, Mutex<WindowState>>,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let phys_pos = window.outer_position().map_err(|e| e.to_string())?;
    let phys_size = window.outer_size().map_err(|e| e.to_string())?;

    let current_logical_height = phys_size.height as f64 / scale;
    let current_logical_x = phys_pos.x as f64 / scale;
    let current_logical_y = phys_pos.y as f64 / scale;

    let mut ws = state.lock().map_err(|e| e.to_string())?;

    if height > current_logical_height {
        // Expanding: save the current physical Y so we can restore it on collapse.
        ws.widget_pre_expand_y = Some(phys_pos.y);

        // Reposition the window upward if the expanded height would push it off-screen.
        if let Ok(Some(monitor)) = window.current_monitor() {
            let monitor_logical_y = monitor.position().y as f64 / scale;
            let monitor_logical_height = monitor.size().height as f64 / scale;
            let monitor_logical_bottom = monitor_logical_y + monitor_logical_height;

            let new_bottom = current_logical_y + height;
            if new_bottom > monitor_logical_bottom {
                let new_logical_y = (monitor_logical_bottom - height).max(monitor_logical_y);
                window
                    .set_position(tauri::Position::Logical(tauri::LogicalPosition {
                        x: current_logical_x,
                        y: new_logical_y,
                    }))
                    .map_err(|e| e.to_string())?;
            }
        }
    } else if height < current_logical_height {
        // Collapsing: restore the physical Y position saved before the expansion.
        if let Some(pre_y) = ws.widget_pre_expand_y.take() {
            window
                .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: phys_pos.x,
                    y: pre_y,
                }))
                .map_err(|e| e.to_string())?;
        }
    }

    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_widget_mode(
    window: tauri::Window,
    state: tauri::State<'_, Mutex<WindowState>>,
    enable: bool,
) -> Result<bool, String> {
    let mut ws = state.lock().map_err(|e| e.to_string())?;

    if enable {
        let size = window.outer_size().map_err(|e| e.to_string())?;
        let position = window.outer_position().map_err(|e| e.to_string())?;
        ws.previous_size = Some((size.width, size.height));
        ws.previous_position = Some((position.x, position.y));

        window
            .set_size(tauri::Size::Logical(tauri::LogicalSize {
                width: 320.0,
                height: 100.0,
            }))
            .map_err(|e| e.to_string())?;
        window.set_resizable(false).map_err(|e| e.to_string())?;
        window.set_always_on_top(true).map_err(|e| e.to_string())?;
    } else {
        window.set_always_on_top(false).map_err(|e| e.to_string())?;
        window.set_resizable(true).map_err(|e| e.to_string())?;

        if let Some((w, h)) = ws.previous_size.take() {
            window
                .set_size(tauri::Size::Physical(tauri::PhysicalSize {
                    width: w,
                    height: h,
                }))
                .map_err(|e| e.to_string())?;
        }
        if let Some((x, y)) = ws.previous_position.take() {
            window
                .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
                .map_err(|e| e.to_string())?;
        }
    }

    ws.is_widget_mode = enable;
    Ok(enable)
}
