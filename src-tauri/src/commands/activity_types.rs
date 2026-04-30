use tauri::State;
use uuid::Uuid;
use chrono::Utc;
use rusqlite::params;
use crate::{AppState, db::get_conn, models::{activity_type::*, error::AppError}};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ActivityType> {
    Ok(ActivityType {
        id: row.get(0)?,
        name: row.get(1)?,
        sort_order: row.get(2)?,
        created_at: row.get(3)?,
    })
}

#[tauri::command]
pub fn list_activity_types(state: State<AppState>) -> Result<Vec<ActivityType>, AppError> {
    let conn = get_conn(&state)?;
    let mut stmt = conn.prepare(
        "SELECT id, name, sort_order, created_at FROM activity_types ORDER BY sort_order, name"
    )?;
    let items: Result<Vec<_>, _> = stmt.query_map([], map_row)?.collect();
    items.map_err(AppError::Database)
}

#[tauri::command]
pub fn create_activity_type(state: State<AppState>, params: CreateActivityTypeParams) -> Result<ActivityType, AppError> {
    let conn = get_conn(&state)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let max_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) FROM activity_types",
        [],
        |row| row.get(0)
    )?;
    conn.execute(
        "INSERT INTO activity_types (id, name, sort_order, created_at) VALUES (?, ?, ?, ?)",
        params![&id, &params.name, max_order + 1, &now]
    )?;
    Ok(ActivityType { id, name: params.name, sort_order: max_order + 1, created_at: now })
}

#[tauri::command]
pub fn update_activity_type(state: State<AppState>, id: String, params: UpdateActivityTypeParams) -> Result<ActivityType, AppError> {
    let conn = get_conn(&state)?;
    if let Some(name) = &params.name {
        conn.execute("UPDATE activity_types SET name = ? WHERE id = ?", rusqlite::params![name, &id])?;
    }
    if let Some(sort_order) = params.sort_order {
        conn.execute("UPDATE activity_types SET sort_order = ? WHERE id = ?", rusqlite::params![sort_order, &id])?;
    }
    let item = conn.query_row(
        "SELECT id, name, sort_order, created_at FROM activity_types WHERE id = ?",
        params![&id],
        map_row
    ).map_err(|_| AppError::NotFound(format!("Activity type {} not found", id)))?;
    Ok(item)
}

#[tauri::command]
pub fn delete_activity_type(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = get_conn(&state)?;
    let rows = conn.execute("DELETE FROM activity_types WHERE id = ?", params![&id])?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("Activity type {} not found", id)));
    }
    Ok(())
}
