use rusqlite::{Connection, params};
use uuid::Uuid;
use chrono::Utc;
use crate::models::{session::*, customer::*, work_order::*, error::AppError};

/// Stop the currently active session (if any). Returns the stopped session id.
pub fn stop_active_session(conn: &Connection) -> Result<Option<String>, AppError> {
    // Get active session
    let mut stmt = conn.prepare("SELECT session_id FROM active_session WHERE id = 1")?;
    let session_id: Option<String> = stmt.query_row([], |row| row.get(0)).ok();
    
    if let Some(sid) = session_id {
        let now = Utc::now().to_rfc3339();
        
        // Calculate duration
        let start_time: String = conn.query_row(
            "SELECT start_time FROM time_sessions WHERE id = ?",
            params![&sid],
            |row| row.get(0)
        )?;
        
        let duration = calculate_duration(&start_time, &now)?;
        
        // Update session
        conn.execute(
            "UPDATE time_sessions SET end_time = ?, duration_seconds = ?, updated_at = ? WHERE id = ?",
            params![&now, duration, &now, &sid]
        )?;
        
        // Clear active session
        conn.execute(
            "UPDATE active_session SET session_id = NULL, work_order_id = NULL, started_at = NULL, last_heartbeat = NULL WHERE id = 1",
            params![]
        )?;
        
        Ok(Some(sid))
    } else {
        Ok(None)
    }
}

/// Create and start a new session for the given work_order_id. Stops any active session first.
pub fn switch_to_work_order(conn: &Connection, work_order_id: &str) -> Result<Session, AppError> {
    // Verify work order exists
    let _exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM work_orders WHERE id = ? AND archived_at IS NULL",
        params![work_order_id],
        |row| row.get(0)
    )?;
    
    if _exists == 0 {
        return Err(AppError::NotFound(format!("Work order {} not found", work_order_id)));
    }
    
    // Start transaction
    let tx = conn.unchecked_transaction()?;
    
    // Stop current session if any
    stop_active_session(&tx)?;
    
    // Create new session
    let session_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    tx.execute(
        "INSERT INTO time_sessions (id, work_order_id, start_time, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        params![&session_id, work_order_id, &now, &now, &now]
    )?;
    
    // Update active session
    tx.execute(
        "UPDATE active_session SET session_id = ?, work_order_id = ?, started_at = ?, last_heartbeat = ? WHERE id = 1",
        params![&session_id, work_order_id, &now, &now]
    )?;
    
    // Update recent work orders
    tx.execute(
        "INSERT OR REPLACE INTO recent_work_orders (work_order_id, last_used_at, use_count) 
         VALUES (?, ?, COALESCE((SELECT use_count FROM recent_work_orders WHERE work_order_id = ?), 0) + 1)",
        params![work_order_id, &now, work_order_id]
    )?;
    
    tx.commit()?;
    
    // Fetch and return the created session
    get_session_by_id(conn, &session_id)
}

/// Stop current session with metadata
pub fn stop_current_session(conn: &Connection, notes: Option<&str>, activity_type: Option<&str>) -> Result<Option<Session>, AppError> {
    let session_id = stop_active_session(conn)?;
    
    if let Some(sid) = session_id {
        let now = Utc::now().to_rfc3339();
        
        // Update notes and activity type if provided
        if notes.is_some() || activity_type.is_some() {
            let mut sql = "UPDATE time_sessions SET updated_at = ?".to_string();
            // Own the values so they live long enough for params_vec borrows
            let notes_owned = notes.map(|s| s.to_string());
            let activity_owned = activity_type.map(|s| s.to_string());
            let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&now];

            if let Some(ref n) = notes_owned {
                sql.push_str(", notes = ?");
                params_vec.push(n);
            }
            if let Some(ref a) = activity_owned {
                sql.push_str(", activity_type = ?");
                params_vec.push(a);
            }
            sql.push_str(" WHERE id = ?");
            params_vec.push(&sid);

            conn.execute(&sql, rusqlite::params_from_iter(params_vec))?;
        }
        
        Ok(Some(get_session_by_id(conn, &sid)?))
    } else {
        Ok(None)
    }
}

/// Get active session with joined display data
pub fn get_active_session(conn: &Connection) -> Result<Option<ActiveSession>, AppError> {
    let mut stmt = conn.prepare("
        SELECT 
            ts.id,
            ts.work_order_id,
            wo.name,
            c.name,
            c.color,
            ts.start_time
        FROM active_session a
        JOIN time_sessions ts ON a.session_id = ts.id
        JOIN work_orders wo ON ts.work_order_id = wo.id
        JOIN customers c ON wo.customer_id = c.id
        WHERE a.id = 1 AND a.session_id IS NOT NULL
    ")?;
    
    let result = stmt.query_row([], |row| {
        let start_time: String = row.get(5)?;
        let elapsed = calculate_elapsed(&start_time).unwrap_or(0);
        
        Ok(ActiveSession {
            session_id: row.get(0)?,
            work_order_id: row.get(1)?,
            work_order_name: row.get(2)?,
            customer_name: row.get(3)?,
            customer_color: row.get(4)?,
            started_at: start_time,
            elapsed_seconds: elapsed,
        })
    });
    
    match result {
        Ok(active) => Ok(Some(active)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

/// Check for orphan sessions on startup
pub fn check_for_orphan_session(conn: &Connection) -> Result<Option<OrphanSession>, AppError> {
    let mut stmt = conn.prepare("
        SELECT 
            ts.id,
            wo.name,
            c.name,
            ts.start_time
        FROM active_session a
        JOIN time_sessions ts ON a.session_id = ts.id
        JOIN work_orders wo ON ts.work_order_id = wo.id
        JOIN customers c ON wo.customer_id = c.id
        WHERE a.id = 1 
          AND a.session_id IS NOT NULL 
          AND ts.end_time IS NULL
          AND (a.last_heartbeat IS NULL OR datetime(a.last_heartbeat) < datetime('now', '-2 minutes'))
    ")?;
    
    let result = stmt.query_row([], |row| {
        Ok(OrphanSession {
            session_id: row.get(0)?,
            work_order_name: row.get(1)?,
            customer_name: row.get(2)?,
            started_at: row.get(3)?,
        })
    });
    
    match result {
        Ok(orphan) => Ok(Some(orphan)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

/// Recover an orphan session (close it at current time)
pub fn recover_session(conn: &Connection, session_id: &str) -> Result<Session, AppError> {
    let now = Utc::now().to_rfc3339();
    
    // Get start time
    let start_time: String = conn.query_row(
        "SELECT start_time FROM time_sessions WHERE id = ?",
        params![session_id],
        |row| row.get(0)
    )?;
    
    let duration = calculate_duration(&start_time, &now)?;
    
    // Update session
    conn.execute(
        "UPDATE time_sessions SET end_time = ?, duration_seconds = ?, updated_at = ? WHERE id = ?",
        params![&now, duration, &now, session_id]
    )?;
    
    // Clear active session
    conn.execute(
        "UPDATE active_session SET session_id = NULL, work_order_id = NULL, started_at = NULL, last_heartbeat = NULL WHERE id = 1",
        params![]
    )?;
    
    get_session_by_id(conn, session_id)
}

/// Discard an orphan session (delete it)
pub fn discard_orphan_session(conn: &Connection, session_id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM time_sessions WHERE id = ?", params![session_id])?;
    conn.execute(
        "UPDATE active_session SET session_id = NULL, work_order_id = NULL, started_at = NULL, last_heartbeat = NULL WHERE id = 1",
        params![]
    )?;
    Ok(())
}

/// quick_add: create customer (optional) + work order + start session atomically
pub fn quick_add(conn: &Connection, params: &QuickAddParams) -> Result<QuickAddResult, AppError> {
    let tx = conn.unchecked_transaction()?;
    let now = Utc::now().to_rfc3339();
    
    // Determine customer
    let customer_id = if let Some(cid) = &params.customer_id {
        cid.clone()
    } else if let Some(cname) = &params.customer_name {
        // Create new customer
        let new_cid = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO customers (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)",
            params![&new_cid, cname, &now, &now]
        )?;
        new_cid
    } else {
        return Err(AppError::Validation("Either customer_id or customer_name required".into()));
    };
    
    // Create work order
    let work_order_id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO work_orders (id, customer_id, name, code, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        params![&work_order_id, &customer_id, &params.work_order_name, &params.work_order_code, &now, &now]
    )?;
    
    tx.commit()?;
    
    // Start session using switch
    let session = switch_to_work_order(conn, &work_order_id)?;
    
    // Fetch customer and work order
    let customer = get_customer_by_id(conn, &customer_id)?;
    let work_order = get_work_order_by_id(conn, &work_order_id)?;
    
    Ok(QuickAddResult {
        customer,
        work_order,
        session,
    })
}

// Helper functions

fn calculate_duration(start: &str, end: &str) -> Result<i64, AppError> {
    let start_dt = chrono::DateTime::parse_from_rfc3339(start)
        .map_err(|_| AppError::Validation("Invalid start time".into()))?;
    let end_dt = chrono::DateTime::parse_from_rfc3339(end)
        .map_err(|_| AppError::Validation("Invalid end time".into()))?;
    
    let duration = end_dt.signed_duration_since(start_dt);
    Ok(duration.num_seconds())
}

fn calculate_elapsed(start: &str) -> Result<i64, AppError> {
    let now = Utc::now().to_rfc3339();
    calculate_duration(start, &now)
}

fn get_session_by_id(conn: &Connection, id: &str) -> Result<Session, AppError> {
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
            ts.duration_override,
            COALESCE(ts.duration_override, ts.duration_seconds),
            ts.activity_type,
            ts.notes,
            ts.created_at,
            ts.updated_at
        FROM time_sessions ts
        JOIN work_orders wo ON ts.work_order_id = wo.id
        JOIN customers c ON wo.customer_id = c.id
        WHERE ts.id = ?
    ")?;
    
    stmt.query_row(params![id], |row| {
        Ok(Session {
            id: row.get(0)?,
            work_order_id: row.get(1)?,
            work_order_name: row.get(2)?,
            customer_name: row.get(3)?,
            customer_color: row.get(4)?,
            start_time: row.get(5)?,
            end_time: row.get(6)?,
            duration_seconds: row.get(7)?,
            duration_override: row.get(8)?,
            effective_duration: row.get(9)?,
            activity_type: row.get(10)?,
            notes: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    }).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("Session {} not found", id)),
        e => AppError::Database(e),
    })
}

fn get_customer_by_id(conn: &Connection, id: &str) -> Result<Customer, AppError> {
    conn.query_row(
        "SELECT id, name, code, color, created_at, updated_at, archived_at FROM customers WHERE id = ?",
        params![id],
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
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("Customer {} not found", id)),
        e => AppError::Database(e),
    })
}

fn get_work_order_by_id(conn: &Connection, id: &str) -> Result<WorkOrder, AppError> {
    conn.query_row(
        "SELECT wo.id, wo.customer_id, c.name, c.color, wo.name, wo.code, wo.description, wo.status, wo.created_at, wo.updated_at, wo.archived_at 
         FROM work_orders wo 
         JOIN customers c ON wo.customer_id = c.id 
         WHERE wo.id = ?",
        params![id],
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
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                archived_at: row.get(10)?,
            })
        }
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("Work order {} not found", id)),
        e => AppError::Database(e),
    })
}
