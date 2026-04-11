use tauri::State;
use rusqlite::params;
use uuid::Uuid;
use chrono::Utc;
use crate::{AppState, models::{customer::*, error::AppError}};

#[tauri::command]
pub fn create_customer(state: State<AppState>, params: CreateCustomerParams) -> Result<Customer, AppError> {
    let conn = state.db.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    conn.execute(
        "INSERT INTO customers (id, name, code, color, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        rusqlite::params![&id, &params.name, &params.code, &params.color, &now, &now]
    )?;
    
    Ok(Customer {
        id,
        name: params.name,
        code: params.code,
        color: params.color,
        created_at: now.clone(),
        updated_at: now,
        archived_at: None,
    })
}

#[tauri::command]
pub fn list_customers(state: State<AppState>, include_archived: Option<bool>) -> Result<Vec<Customer>, AppError> {
    let conn = state.db.lock().unwrap();
    let include_archived = include_archived.unwrap_or(false);
    
    let sql = if include_archived {
        "SELECT id, name, code, color, created_at, updated_at, archived_at FROM customers ORDER BY name"
    } else {
        "SELECT id, name, code, color, created_at, updated_at, archived_at FROM customers WHERE archived_at IS NULL ORDER BY name"
    };
    
    let mut stmt = conn.prepare(sql)?;
    let customers: Result<Vec<_>, _> = stmt.query_map([], |row| {
        Ok(Customer {
            id: row.get(0)?,
            name: row.get(1)?,
            code: row.get(2)?,
            color: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            archived_at: row.get(6)?,
        })
    })?.collect();
    
    customers.map_err(AppError::Database)
}

#[tauri::command]
pub fn update_customer(state: State<AppState>, id: String, params: UpdateCustomerParams) -> Result<Customer, AppError> {
    let conn = state.db.lock().unwrap();
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
    if params.color.is_some() {
        updates.push("color = ?");
        values.push(Box::new(params.color.clone()));
    }
    
    values.push(Box::new(id.clone()));
    
    let sql = format!("UPDATE customers SET {} WHERE id = ?", updates.join(", "));
    let params_refs: Vec<&dyn rusqlite::ToSql> = values.iter().map(|b| b.as_ref()).collect();
    
    let rows_affected = conn.execute(&sql, rusqlite::params_from_iter(params_refs))?;
    
    if rows_affected == 0 {
        return Err(AppError::NotFound(format!("Customer {} not found", id)));
    }
    
    // Fetch updated customer
    conn.query_row(
        "SELECT id, name, code, color, created_at, updated_at, archived_at FROM customers WHERE id = ?",
        params![&id],
        |row| {
            Ok(Customer {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                color: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                archived_at: row.get(6)?,
            })
        }
    ).map_err(AppError::Database)
}

#[tauri::command]
pub fn archive_customer(state: State<AppState>, id: String) -> Result<(), AppError> {
    let conn = state.db.lock().unwrap();
    let now = Utc::now().to_rfc3339();
    
    let rows_affected = conn.execute(
        "UPDATE customers SET archived_at = ?, updated_at = ? WHERE id = ?",
        params![&now, &now, &id]
    )?;
    
    if rows_affected == 0 {
        return Err(AppError::NotFound(format!("Customer {} not found", id)));
    }
    
    Ok(())
}
