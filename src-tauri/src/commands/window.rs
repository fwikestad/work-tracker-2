use std::sync::Mutex;
use crate::WindowState;

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
            .set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: 320,
                height: 150,
            }))
            .map_err(|e| e.to_string())?;
        window.set_always_on_top(true).map_err(|e| e.to_string())?;
    } else {
        window.set_always_on_top(false).map_err(|e| e.to_string())?;

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
