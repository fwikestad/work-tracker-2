use tauri::State;
use rusqlite::params;
use chrono::Utc;
use crate::{AppState, db::get_conn, models::{session::*, error::AppError}, services::session_service};

#[tauri::command]
pub fn start_session(state: State<AppState>, work_order_id: String) -> Result<Session, AppError> {
    let conn = get_conn(&state)?;
    session_service::switch_to_work_order(&conn, &work_order_id)
}

#[tauri::command]
pub fn stop_session(state: State<AppState>, notes: Option<String>, activity_type: Option<String>) -> Result<Option<Session>, AppError> {
    let conn = get_conn(&state)?;
    session_service::stop_current_session(
        &conn,
        notes.as_deref(),
        activity_type.as_deref()
    )
}

#[tauri::command]
pub fn get_active_session(state: State<AppState>) -> Result<Option<ActiveSession>, AppError> {
    let conn = get_conn(&state)?;
    session_service::get_active_session(&conn)
}

#[tauri::command]
pub fn update_session(state: State<AppState>, id: String, params: UpdateSessionParams) -> Result<Session, AppError> {
    let conn = get_conn(&state)?;
    let now = Utc::now().to_rfc3339();
    
    // If start_time or end_time is being updated, use the specialized function
    if params.start_time.is_some() || params.end_time.is_some() {
        // Call update_session_times for validation and duration recalculation
        let updated = session_service::update_session_times(
            &conn,
            &id,
            params.start_time.as_deref(),
            params.end_time.as_deref(),
        )?;
        
        // If other fields (notes, activity_type) are provided,
        // apply them in a second update
        if params.notes.is_some() || params.activity_type.is_some() {
            let mut updates = vec!["updated_at = ?"];
            let mut values: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.clone())];
            
            if params.activity_type.is_some() {
                updates.push("activity_type = ?");
                values.push(Box::new(params.activity_type.clone()));
            }
            if params.notes.is_some() {
                updates.push("notes = ?");
                values.push(Box::new(params.notes.clone()));
            }
            
            values.push(Box::new(id.clone()));
            
            let sql = format!("UPDATE time_sessions SET {} WHERE id = ?", updates.join(", "));
            let params_refs: Vec<&dyn rusqlite::ToSql> = values.iter().map(|b| b.as_ref()).collect();
            
            conn.execute(&sql, rusqlite::params_from_iter(params_refs))?;
            
            // Fetch and return the final updated session
            return conn.query_row(
                "SELECT 
                    ts.id,
                    ts.work_order_id,
                    wo.name,
                    c.name,
                    c.color,
                    ts.start_time,
                    ts.end_time,
                    ts.duration_seconds,
                    ts.duration_seconds,
                    ts.activity_type,
                    ts.notes,
                    ts.created_at,
                    ts.updated_at
                FROM time_sessions ts
                JOIN work_orders wo ON ts.work_order_id = wo.id
                JOIN customers c ON wo.customer_id = c.id
                WHERE ts.id = ?",
                params![&id],
                |row| {
                    Ok(Session {
                        id: row.get(0)?,
                        work_order_id: row.get(1)?,
                        work_order_name: row.get(2)?,
                        customer_name: row.get(3)?,
                        customer_color: row.get(4)?,
                        start_time: row.get(5)?,
                        end_time: row.get(6)?,
                        duration_seconds: row.get(7)?,
                        effective_duration: row.get(8)?,
                        activity_type: row.get(9)?,
                        notes: row.get(10)?,
                        created_at: row.get(11)?,
                        updated_at: row.get(12)?,
                    })
                }
            ).map_err(AppError::Database);
        }
        
        return Ok(updated);
    }
    
    // Original path: only updating notes or activity_type
    // Build dynamic UPDATE query
    let mut updates = vec!["updated_at = ?"];
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.clone())];
    
    if params.activity_type.is_some() {
        updates.push("activity_type = ?");
        values.push(Box::new(params.activity_type.clone()));
    }
    if params.notes.is_some() {
        updates.push("notes = ?");
        values.push(Box::new(params.notes.clone()));
    }
    
    values.push(Box::new(id.clone()));
    
    let sql = format!("UPDATE time_sessions SET {} WHERE id = ?", updates.join(", "));
    let params_refs: Vec<&dyn rusqlite::ToSql> = values.iter().map(|b| b.as_ref()).collect();
    
    let rows_affected = conn.execute(&sql, rusqlite::params_from_iter(params_refs))?;
    
    if rows_affected == 0 {
        return Err(AppError::NotFound(format!("Session {} not found", id)));
    }
    
    // Fetch updated session
    conn.query_row(
        "SELECT 
            ts.id,
            ts.work_order_id,
            wo.name,
            c.name,
            c.color,
            ts.start_time,
            ts.end_time,
            ts.duration_seconds,
            ts.duration_seconds,
            ts.activity_type,
            ts.notes,
            ts.created_at,
            ts.updated_at
        FROM time_sessions ts
        JOIN work_orders wo ON ts.work_order_id = wo.id
        JOIN customers c ON wo.customer_id = c.id
        WHERE ts.id = ?",
        params![&id],
        |row| {
            Ok(Session {
                id: row.get(0)?,
                work_order_id: row.get(1)?,
                work_order_name: row.get(2)?,
                customer_name: row.get(3)?,
                customer_color: row.get(4)?,
                start_time: row.get(5)?,
                end_time: row.get(6)?,
                duration_seconds: row.get(7)?,
                effective_duration: row.get(8)?,
                activity_type: row.get(9)?,
                notes: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        }
    ).map_err(AppError::Database)
}

#[tauri::command]
pub fn list_sessions(state: State<AppState>, start_date: String, end_date: String) -> Result<Vec<Session>, AppError> {
    let conn = get_conn(&state)?;
    
    let mut stmt = conn.prepare("
        SELECT 
            ts.id,
            ts.work_order_id,
            wo.name,
            c.name,
            c.color,
            ts.start_time,
            ts.end_time,
            ts.duration_seconds,
            ts.duration_seconds,
            ts.activity_type,
            ts.notes,
            ts.created_at,
            ts.updated_at
        FROM time_sessions ts
        JOIN work_orders wo ON ts.work_order_id = wo.id
        JOIN customers c ON wo.customer_id = c.id
        WHERE date(ts.start_time) >= date(?)
          AND date(ts.start_time) <= date(?)
        ORDER BY ts.start_time
    ")?;
    
    let sessions: Result<Vec<_>, _> = stmt.query_map(params![&start_date, &end_date], |row| {
        Ok(Session {
            id: row.get(0)?,
            work_order_id: row.get(1)?,
            work_order_name: row.get(2)?,
            customer_name: row.get(3)?,
            customer_color: row.get(4)?,
            start_time: row.get(5)?,
            end_time: row.get(6)?,
            duration_seconds: row.get(7)?,
            effective_duration: row.get(8)?,
            activity_type: row.get(9)?,
            notes: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?.collect();
    
    sessions.map_err(AppError::Database)
}

#[tauri::command]
pub fn delete_session(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = get_conn(&state)?;
    
    let rows_affected = conn.execute("DELETE FROM time_sessions WHERE id = ?", params![&id])?;
    
    if rows_affected == 0 {
        return Err(AppError::NotFound(format!("Session {} not found", id)));
    }
    
    Ok(())
}

#[tauri::command]
pub fn quick_add(state: State<AppState>, params: QuickAddParams) -> Result<QuickAddResult, AppError> {
    let conn = get_conn(&state)?;
    session_service::quick_add(&conn, &params)
}

#[tauri::command]
pub fn recover_session(state: State<AppState>, session_id: String) -> Result<Session, AppError> {
    let conn = get_conn(&state)?;
    session_service::recover_session(&conn, &session_id)
}

#[tauri::command]
pub fn discard_orphan_session(state: State<AppState>, session_id: String) -> Result<(), AppError> {
    let conn = get_conn(&state)?;
    session_service::discard_orphan_session(&conn, &session_id)
}

#[tauri::command]
pub fn get_last_stopped_work_order(state: State<AppState>) -> Result<Option<String>, AppError> {
    let conn = get_conn(&state)?;
    session_service::get_last_stopped_work_order(&conn)
}

#[tauri::command]
pub fn update_heartbeat(state: State<AppState>) -> Result<(), AppError> {
    let conn = get_conn(&state)?;
    session_service::update_heartbeat(&conn)
}

#[tauri::command]
pub fn check_for_orphan_session(state: State<AppState>) -> Result<Option<OrphanSession>, AppError> {
    let conn = get_conn(&state)?;
    session_service::check_for_orphan_session(&conn)
}
