//! Session management service layer.
//!
//! Provides atomic operations for time tracking sessions including:
//! - Starting/stopping/switching work sessions
//! - Pause/resume functionality (Phase 2)
//! - Crash recovery for orphaned sessions
//! - Quick-add workflow for creating customers + work orders atomically
//!
//! All session operations maintain data integrity through transactions and ensure
//! at most one active session exists at any time.

use rusqlite::{Connection, params};
use uuid::Uuid;
use chrono::Utc;
use crate::models::{session::*, customer::*, work_order::*, error::AppError};
use rusqlite::OptionalExtension;

/// Stop the currently active session (if any).
///
/// Sets `end_time` to current timestamp, calculates `duration_seconds` (gross wall-clock time),
/// and clears the `active_session` singleton. No-op if no session is active.
///
/// # Arguments
///
/// * `conn` - Database connection
///
/// # Returns
///
/// The session ID that was stopped, or `None` if no session was active.
///
/// # Errors
///
/// Returns `AppError::Database` if the query fails.
pub fn stop_active_session(conn: &Connection) -> Result<Option<String>, AppError> {
    // Get active session
    let mut stmt = conn.prepare("SELECT session_id FROM active_session WHERE id = 1")?;
    let session_id: Option<String> = stmt.query_row([], |row| row.get(0)).ok();
    
    if let Some(sid) = session_id {
        let now = Utc::now().to_rfc3339();
        
        // Fetch start time; total_paused_seconds is informational only since we store gross duration
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

/// Switch to tracking a different work order.
///
/// Atomically stops the current session (if any) and starts a new session for the specified
/// work order. Updates the `active_session` singleton and recent work orders tracking.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `work_order_id` - UUID of the work order to start tracking
///
/// # Returns
///
/// The newly created `Session` with `end_time = None` (running state).
///
/// # Errors
///
/// Returns `AppError::NotFound` if the work order does not exist or is archived.
/// Returns `AppError::Database` on transaction failure.
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
    
    // Update active session — use upsert so a missing singleton is recovered automatically
    let rows = tx.execute(
        "UPDATE active_session SET session_id = ?, work_order_id = ?, started_at = ?, last_heartbeat = ? WHERE id = 1",
        params![&session_id, work_order_id, &now, &now]
    )?;
    if rows == 0 {
        tx.execute(
            "INSERT INTO active_session (id, session_id, work_order_id, started_at, last_heartbeat) VALUES (1, ?, ?, ?, ?)",
            params![&session_id, work_order_id, &now, &now]
        )?;
    }
    
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

/// Stop the current session and optionally add notes and activity type.
///
/// Convenience wrapper around `stop_active_session` that also updates session metadata.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `notes` - Optional session notes to save
/// * `activity_type` - Optional activity classification (e.g., "meeting", "development")
///
/// # Returns
///
/// The stopped `Session` with all metadata, or `None` if no session was active.
///
/// # Errors
///
/// Returns `AppError::Database` on query failure.
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

/// Get the currently active session with customer and work order details.
///
/// Joins `active_session` → `time_sessions` → `work_orders` → `customers` to retrieve
/// full display context. Calculates `elapsed_seconds` accounting for pause intervals.
///
/// # Arguments
///
/// * `conn` - Database connection
///
/// # Returns
///
/// An `ActiveSession` with all display data, or `None` if no session is active.
///
/// # Errors
///
/// Returns `AppError::Database` on query failure.
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

/// Detect orphaned sessions on application startup.
///
/// An orphaned session is one that has `end_time = NULL` and a stale heartbeat
/// (last_heartbeat > 2 minutes ago). This indicates the app crashed or was force-quit
/// without properly closing the session.
///
/// # Arguments
///
/// * `conn` - Database connection
///
/// # Returns
///
/// An `OrphanSession` with display context if found, or `None` if no orphan exists.
///
/// # Errors
///
/// Returns `AppError::Database` on query failure.
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

/// Recover an orphaned session by closing it with the current timestamp.
///
/// User chooses to "accept" the orphan — close it now and preserve the tracked time.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `session_id` - UUID of the orphaned session
///
/// # Returns
///
/// The recovered `Session` with `end_time` set to now.
///
/// # Errors
///
/// Returns `AppError::NotFound` if the session does not exist.
/// Returns `AppError::Database` on update failure.
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

/// Discard an orphaned session by deleting it from the database.
///
/// User chooses to reject the orphan — delete it without preserving any time.
/// Clears the `active_session` singleton.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `session_id` - UUID of the orphaned session to delete
///
/// # Errors
///
/// Returns `AppError::Database` on deletion failure.
pub fn discard_orphan_session(conn: &Connection, session_id: &str) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;
    // Clear active_session reference FIRST so the subsequent DELETE does not
    // cascade-delete the singleton row (guard against future schema issues).
    tx.execute(
        "UPDATE active_session SET session_id = NULL, work_order_id = NULL, started_at = NULL, last_heartbeat = NULL WHERE id = 1",
        params![]
    )?;
    tx.execute("DELETE FROM time_sessions WHERE id = ?", params![session_id])?;
    tx.commit()?;
    Ok(())
}

/// Quick-add workflow: create customer + work order + start session atomically.
///
/// Phase 1 feature for inline creation without navigating away from active timer view.
/// Either uses an existing customer or creates a new one based on parameters.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `params` - QuickAddParams with either `customer_id` (existing) or `customer_name` (create new)
///
/// # Returns
///
/// A `QuickAddResult` containing the customer, work order, and newly started session.
///
/// # Errors
///
/// Returns `AppError::Validation` if neither `customer_id` nor `customer_name` is provided.
/// Returns `AppError::Database` on transaction failure.
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

/// Get the work order ID of the most recently stopped session.
///
/// Used by the "Continue" feature to start a new session on the last work order.
/// Only considers sessions that have been completed (end_time IS NOT NULL).
///
/// # Arguments
///
/// * `conn` - Database connection
///
/// # Returns
///
/// The work order ID of the most recent stopped session, or `None` if no sessions exist.
///
/// # Errors
///
/// Returns `AppError::Database` on query failure.
pub fn get_last_stopped_work_order(conn: &Connection) -> Result<Option<String>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT work_order_id FROM time_sessions 
         WHERE end_time IS NOT NULL 
         ORDER BY end_time DESC 
         LIMIT 1"
    )?;
    
    let result = stmt.query_row([], |row| row.get(0)).optional()?;
    Ok(result)
}

/// Update the heartbeat timestamp for crash detection.
///
/// Called periodically (e.g., every 30 seconds) by the frontend to signal the app is alive.
/// If heartbeat becomes stale (>2 minutes), `check_for_orphan_session` detects it on next startup.
///
/// # Arguments
///
/// * `conn` - Database connection
///
/// # Errors
///
/// Returns `AppError::Database` on update failure.
pub fn update_heartbeat(conn: &Connection) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    
    conn.execute(
        "UPDATE active_session SET last_heartbeat = ? WHERE id = 1 AND session_id IS NOT NULL",
        params![&now]
    )?;
    
    Ok(())
}

// Helper functions

/// Parse a timestamp string supporting both RFC3339 and SQLite datetime formats.
///
/// Provides backward compatibility with older sessions that may have SQLite-format timestamps
/// while new sessions use RFC3339 (current standard).
///
/// # Supported Formats
///
/// - RFC3339: "2024-01-15T10:30:00Z" or "2024-01-15T10:30:00+00:00" (current)
/// - SQLite: "2024-01-15 10:30:00" (legacy, converted internally)
///
/// # Arguments
///
/// * `timestamp` - Timestamp string in either format
///
/// # Returns
///
/// Parsed `chrono::DateTime<FixedOffset>` with UTC timezone.
///
/// # Errors
///
/// Returns `AppError::Validation` if the format is unrecognized.
fn parse_timestamp(timestamp: &str) -> Result<chrono::DateTime<chrono::FixedOffset>, AppError> {
    // Try RFC3339 first (current format)
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        return Ok(dt);
    }
    
    // Try SQLite datetime format: "YYYY-MM-DD HH:MM:SS"
    // Convert to RFC3339 by replacing space with 'T' and adding 'Z'
    if timestamp.len() == 19 && timestamp.chars().nth(10) == Some(' ') {
        let rfc3339 = format!("{}T{}Z", &timestamp[..10], &timestamp[11..]);
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&rfc3339) {
            return Ok(dt);
        }
    }
    
    Err(AppError::Validation(format!("Invalid timestamp format: {}", timestamp)))
}

/// Calculate duration in seconds between two timestamps.
///
/// Supports mixed formats (one RFC3339, one SQLite) via `parse_timestamp`.
/// Returns gross wall-clock time (no pause subtraction — handled elsewhere).
///
/// # Arguments
///
/// * `start` - Start timestamp (any supported format)
/// * `end` - End timestamp (any supported format)
///
/// # Returns
///
/// Duration in seconds as `i64`.
///
/// # Errors
///
/// Returns `AppError::Validation` if either timestamp is invalid.
fn calculate_duration(start: &str, end: &str) -> Result<i64, AppError> {
    let start_dt = parse_timestamp(start)?;
    let end_dt = parse_timestamp(end)?;
    
    let duration = end_dt.signed_duration_since(start_dt);
    Ok(duration.num_seconds())
}

/// Calculate elapsed seconds from a start timestamp to now.
///
/// Helper for real-time elapsed time calculations in active sessions.
///
/// # Arguments
///
/// * `start` - Start timestamp (any supported format)
///
/// # Returns
///
/// Elapsed seconds as `i64`.
fn calculate_elapsed(start: &str) -> Result<i64, AppError> {
    calculate_duration(start, &Utc::now().to_rfc3339())
}

/// Fetch a session by ID with joined customer and work order details.
///
/// Internal helper used by public functions to retrieve full session context.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `id` - Session UUID
///
/// # Returns
///
/// A `Session` with all fields populated including `effective_duration`.
///
/// # Errors
///
/// Returns `AppError::NotFound` if the session does not exist.
/// Returns `AppError::Database` on query failure.
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
            ts.duration_seconds,
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
            effective_duration: row.get(8)?,
            activity_type: row.get(9)?,
            notes: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    }).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("Session {} not found", id)),
        e => AppError::Database(e),
    })
}

/// Fetch a customer by ID.
///
/// Internal helper for quick-add workflow.
///
/// # Errors
///
/// Returns `AppError::NotFound` if the customer does not exist.
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

/// Fetch a work order by ID with joined customer details.
///
/// Internal helper for quick-add workflow and session retrieval.
///
/// # Errors
///
/// Returns `AppError::NotFound` if the work order does not exist.
fn get_work_order_by_id(conn: &Connection, id: &str) -> Result<WorkOrder, AppError> {
    conn.query_row(
        "SELECT wo.id, wo.customer_id, c.name, c.color, wo.name, wo.code, wo.description, wo.status, wo.is_favorite, wo.created_at, wo.updated_at, wo.archived_at 
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
                is_favorite: row.get::<_, i64>(8)? == 1,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                archived_at: row.get(11)?,
            })
        }
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("Work order {} not found", id)),
        e => AppError::Database(e),
    })
}

/// Update start and/or end times of a completed session.
///
/// Validates that the session is not currently active, parses timestamps,
/// ensures start < end, and recalculates duration_seconds.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `session_id` - UUID of the session to update
/// * `start_time` - Optional new start time (ISO 8601 / RFC3339 string)
/// * `end_time` - Optional new end time (ISO 8601 / RFC3339 string)
///
/// # Returns
///
/// The updated `Session` with recalculated duration.
///
/// # Errors
///
/// Returns `AppError::NotFound` if the session does not exist.
/// Returns `AppError::Validation` if:
/// - Session is currently active (running)
/// - start_time >= end_time
/// - end_time is too far in the future (>5 minutes)
/// - Timestamp formats are invalid
pub fn update_session_times(
    conn: &Connection,
    session_id: &str,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Session, AppError> {
    // Fetch current session
    let current = get_session_by_id(conn, session_id)?;
    
    // Check if session is active (running)
    let active_session_id: Option<String> = conn
        .query_row(
            "SELECT session_id FROM active_session WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .ok();
    
    if active_session_id.as_deref() == Some(session_id) {
        return Err(AppError::Validation(
            "Cannot edit times of a running session. Stop the session first.".to_string()
        ));
    }
    
    // Ensure session is completed (has end_time)
    if current.end_time.is_none() {
        return Err(AppError::Validation(
            "Cannot edit times of an incomplete session.".to_string()
        ));
    }
    
    // Determine effective start and end times
    let effective_start = start_time.unwrap_or(&current.start_time);
    let effective_end = end_time.unwrap_or(current.end_time.as_ref().unwrap());
    
    // Parse timestamps
    let start_dt = parse_timestamp(effective_start)?;
    let end_dt = parse_timestamp(effective_end)?;
    
    // Validate: start < end
    if start_dt >= end_dt {
        if start_dt == end_dt {
            return Err(AppError::Validation(
                "Session duration must be greater than zero".to_string()
            ));
        }
        return Err(AppError::Validation(
            "start_time must be before end_time".to_string()
        ));
    }
    
    // Validate: end_time not too far in future (allow 5 min tolerance for clock skew)
    let now = Utc::now();
    let future_tolerance = chrono::Duration::minutes(5);
    if end_dt.with_timezone(&Utc) > now + future_tolerance {
        return Err(AppError::Validation(
            "end_time cannot be more than 5 minutes in the future".to_string()
        ));
    }
    
    // Calculate new duration
    let new_duration = calculate_duration(effective_start, effective_end)?;
    
    if new_duration <= 0 {
        return Err(AppError::Validation(
            "Session duration must be greater than zero".to_string()
        ));
    }
    
    // Update session
    let now_str = Utc::now().to_rfc3339();
    let mut updates = vec!["duration_seconds = ?", "updated_at = ?"];
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(new_duration),
        Box::new(now_str.clone()),
    ];
    
    if start_time.is_some() {
        updates.push("start_time = ?");
        values.push(Box::new(effective_start.to_string()));
    }
    
    if end_time.is_some() {
        updates.push("end_time = ?");
        values.push(Box::new(effective_end.to_string()));
    }
    
    values.push(Box::new(session_id.to_string()));
    
    let sql = format!("UPDATE time_sessions SET {} WHERE id = ?", updates.join(", "));
    let params_refs: Vec<&dyn rusqlite::ToSql> = values.iter().map(|b| b.as_ref()).collect();
    
    conn.execute(&sql, rusqlite::params_from_iter(params_refs))?;
    
    // Return updated session
    get_session_by_id(conn, session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp_rfc3339() {
        // RFC3339 with Z suffix
        let result = parse_timestamp("2024-01-15T10:30:00Z");
        assert!(result.is_ok());
        
        // RFC3339 with timezone offset
        let result = parse_timestamp("2024-01-15T10:30:00+00:00");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_timestamp_sqlite_format() {
        // SQLite datetime format: "YYYY-MM-DD HH:MM:SS"
        let result = parse_timestamp("2024-01-15 10:30:00");
        assert!(result.is_ok());
        
        // Verify it parses correctly
        let dt = result.unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M:%S").to_string(), "2024-01-15 10:30:00");
    }

    #[test]
    fn test_calculate_duration_mixed_formats() {
        // SQLite format start, RFC3339 end
        let result = calculate_duration("2024-01-15 10:00:00", "2024-01-15T11:00:00Z");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3600); // 1 hour = 3600 seconds
        
        // RFC3339 start, SQLite format end
        let result = calculate_duration("2024-01-15T10:00:00Z", "2024-01-15 11:00:00");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3600);
        
        // Both RFC3339
        let result = calculate_duration("2024-01-15T10:00:00Z", "2024-01-15T11:00:00Z");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3600);
        
        // Both SQLite format
        let result = calculate_duration("2024-01-15 10:00:00", "2024-01-15 11:00:00");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3600);
    }

    #[test]
    fn test_parse_timestamp_invalid() {
        let result = parse_timestamp("invalid");
        assert!(result.is_err());
        
        let result = parse_timestamp("2024-01-15");
        assert!(result.is_err());
    }
}

