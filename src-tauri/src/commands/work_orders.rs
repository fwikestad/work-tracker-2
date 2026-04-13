use tauri::State;
use rusqlite::params;
use uuid::Uuid;
use chrono::Utc;
use crate::{AppState, db::get_conn, models::{work_order::*, error::AppError}};

#[tauri::command]
pub fn create_work_order(state: State<AppState>, params: CreateWorkOrderParams) -> Result<WorkOrder, AppError> {
    let conn = get_conn(&state)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    // Verify customer exists
    let customer_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM customers WHERE id = ? AND archived_at IS NULL",
        params![&params.customer_id],
        |row| row.get(0)
    )?;
    
    if customer_exists == 0 {
        return Err(AppError::NotFound(format!("Customer {} not found", params.customer_id)));
    }
    
    conn.execute(
        "INSERT INTO work_orders (id, customer_id, name, code, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![&id, &params.customer_id, &params.name, &params.code, &params.description, &now, &now]
    )?;
    
    // Fetch with customer info
    conn.query_row(
        "SELECT wo.id, wo.customer_id, c.name, c.color, wo.name, wo.code, wo.description, wo.status, wo.is_favorite, wo.created_at, wo.updated_at, wo.archived_at 
         FROM work_orders wo 
         JOIN customers c ON wo.customer_id = c.id 
         WHERE wo.id = ?",
        params![&id],
        |row| {
            Ok(WorkOrder {
                id: row.get(0)?,
                customer_id: row.get(1)?,
                customer_name: row.get(2)?,
                customer_color: row.get(3)?,
                name: row.get(4)?,
                code: row.get(5)?,
                description: row.get(6)?,
                status: row.get(7)?,
                is_favorite: row.get::<_, i64>(8)? == 1,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                archived_at: row.get(11)?,
            })
        }
    ).map_err(AppError::Database)
}

#[tauri::command]
pub fn list_work_orders(
    state: State<AppState>, 
    customer_id: Option<String>, 
    favorites_only: Option<bool>,
    include_archived: Option<bool>
) -> Result<Vec<WorkOrder>, AppError> {
    let conn = get_conn(&state)?;
    let include_archived = include_archived.unwrap_or(false);
    
    let archived_clause = if include_archived { "" } else { "AND wo.archived_at IS NULL " };
    let favorites_clause = if favorites_only.unwrap_or(false) { "AND wo.is_favorite = 1 " } else { "" };
    let customer_clause = if customer_id.is_some() { "AND wo.customer_id = ? " } else { "" };
    let order_by = if customer_id.is_some() { "ORDER BY wo.name" } else { "ORDER BY c.name, wo.name" };
    
    let sql = format!(
        "SELECT wo.id, wo.customer_id, c.name, c.color, wo.name, wo.code, wo.description, 
                wo.status, wo.is_favorite, wo.created_at, wo.updated_at, wo.archived_at
         FROM work_orders wo
         JOIN customers c ON wo.customer_id = c.id
         WHERE 1=1 {archived_clause}{favorites_clause}{customer_clause}
         {order_by}"
    );
    
    let mut stmt = conn.prepare(&sql)?;
    
    let work_orders: Result<Vec<_>, _> = if let Some(cid) = customer_id {
        stmt.query_map(params![cid], |row| {
            Ok(WorkOrder {
                id: row.get(0)?,
                customer_id: row.get(1)?,
                customer_name: row.get(2)?,
                customer_color: row.get(3)?,
                name: row.get(4)?,
                code: row.get(5)?,
                description: row.get(6)?,
                status: row.get(7)?,
                is_favorite: row.get::<_, i64>(8)? == 1,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                archived_at: row.get(11)?,
            })
        })?.collect()
    } else {
        stmt.query_map([], |row| {
            Ok(WorkOrder {
                id: row.get(0)?,
                customer_id: row.get(1)?,
                customer_name: row.get(2)?,
                customer_color: row.get(3)?,
                name: row.get(4)?,
                code: row.get(5)?,
                description: row.get(6)?,
                status: row.get(7)?,
                is_favorite: row.get::<_, i64>(8)? == 1,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                archived_at: row.get(11)?,
            })
        })?.collect()
    };
    
    work_orders.map_err(AppError::Database)
}

#[tauri::command]
pub fn update_work_order(state: State<AppState>, id: String, params: UpdateWorkOrderParams) -> Result<WorkOrder, AppError> {
    let conn = get_conn(&state)?;
    let now = Utc::now().to_rfc3339();
    
    // Build dynamic UPDATE query
    let mut updates = vec!["updated_at = ?"];
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.clone())];
    
    if let Some(name) = &params.name {
        updates.push("name = ?");
        values.push(Box::new(name.clone()));
    }
    if params.code.is_some() {
        updates.push("code = ?");
        values.push(Box::new(params.code.clone()));
    }
    if params.description.is_some() {
        updates.push("description = ?");
        values.push(Box::new(params.description.clone()));
    }
    if let Some(status) = &params.status {
        updates.push("status = ?");
        values.push(Box::new(status.clone()));
    }
    
    values.push(Box::new(id.clone()));
    
    let sql = format!("UPDATE work_orders SET {} WHERE id = ?", updates.join(", "));
    let params_refs: Vec<&dyn rusqlite::ToSql> = values.iter().map(|b| b.as_ref()).collect();
    
    let rows_affected = conn.execute(&sql, rusqlite::params_from_iter(params_refs))?;
    
    if rows_affected == 0 {
        return Err(AppError::NotFound(format!("Work order {} not found", id)));
    }
    
    // Fetch updated work order
    conn.query_row(
        "SELECT wo.id, wo.customer_id, c.name, c.color, wo.name, wo.code, wo.description, wo.status, wo.is_favorite, wo.created_at, wo.updated_at, wo.archived_at 
         FROM work_orders wo 
         JOIN customers c ON wo.customer_id = c.id 
         WHERE wo.id = ?",
        params![&id],
        |row| {
            Ok(WorkOrder {
                id: row.get(0)?,
                customer_id: row.get(1)?,
                customer_name: row.get(2)?,
                customer_color: row.get(3)?,
                name: row.get(4)?,
                code: row.get(5)?,
                description: row.get(6)?,
                status: row.get(7)?,
                is_favorite: row.get::<_, i64>(8)? == 1,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                archived_at: row.get(11)?,
            })
        }
    ).map_err(AppError::Database)
}

#[tauri::command]
pub fn archive_work_order(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = get_conn(&state)?;
    let now = Utc::now().to_rfc3339();
    
    let rows_affected = conn.execute(
        "UPDATE work_orders SET archived_at = ?, updated_at = ? WHERE id = ?",
        params![&now, &now, &id]
    )?;
    
    if rows_affected == 0 {
        return Err(AppError::NotFound(format!("Work order {} not found", id)));
    }
    
    Ok(())
}

#[tauri::command]
pub fn toggle_favorite(state: State<AppState>, work_order_id: String) -> Result<WorkOrder, AppError> {
    let conn = get_conn(&state)?;
    let now = Utc::now().to_rfc3339();
    
    // Toggle the is_favorite flag
    conn.execute(
        "UPDATE work_orders SET is_favorite = NOT is_favorite, updated_at = ? WHERE id = ?",
        params![&now, &work_order_id]
    )?;
    
    // Fetch updated work order
    conn.query_row(
        "SELECT wo.id, wo.customer_id, c.name, c.color, wo.name, wo.code, wo.description, wo.status, wo.is_favorite, wo.created_at, wo.updated_at, wo.archived_at 
         FROM work_orders wo 
         JOIN customers c ON wo.customer_id = c.id 
         WHERE wo.id = ?",
        params![&work_order_id],
        |row| {
            Ok(WorkOrder {
                id: row.get(0)?,
                customer_id: row.get(1)?,
                customer_name: row.get(2)?,
                customer_color: row.get(3)?,
                name: row.get(4)?,
                code: row.get(5)?,
                description: row.get(6)?,
                status: row.get(7)?,
                is_favorite: row.get::<_, i64>(8)? == 1,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                archived_at: row.get(11)?,
            })
        }
    ).map_err(AppError::Database)
}
