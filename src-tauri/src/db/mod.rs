use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use crate::models::error::AppError;

pub struct AppState {
    pub db: Mutex<Connection>,
}

/// Safely acquire database lock with graceful poison error handling.
///
/// Returns a `MutexGuard` to the database connection. Unlike direct `.unwrap()` calls,
/// this returns an `AppError` if the mutex is poisoned (thread panicked while holding lock),
/// preventing app crashes in production.
///
/// # Errors
/// Returns `AppError::Database` with SQLITE_BUSY status if the mutex is poisoned.
///
/// # Example
/// ```ignore
/// let conn = get_conn(&state)?;
/// conn.execute("UPDATE ...", [])?;
/// ```
pub fn get_conn<'a>(state: &'a tauri::State<AppState>) -> Result<MutexGuard<'a, Connection>, AppError> {
    state.db.lock().map_err(|e| {
        AppError::Database(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
            Some(format!("Database lock poisoned: {}", e))
        ))
    })
}

/// Create an in-memory database with all migrations applied. For use in tests only.
pub fn init_test_db() -> SqlResult<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("
        PRAGMA foreign_keys = ON;
    ")?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// Create an in-memory database for dev mode. No persistence — fresh DB on each app start.
pub fn init_dev_db() -> SqlResult<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("
        PRAGMA foreign_keys = ON;
    ")?;
    run_migrations(&conn)?;
    Ok(conn)
}

pub fn initialize(db_path: &Path) -> SqlResult<Connection> {
    let conn = Connection::open(db_path)?;
    
    // WAL mode and pragma setup
    conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 5000;
    ")?;
    
    // Run migrations
    run_migrations(&conn)?;
    
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY);"
    )?;

    // If customers table already exists, migration 001 was applied before versioning was added
    let customers_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='customers'",
        [], |r| r.get(0))?;

    let v1: i64 = conn.query_row(
        "SELECT COUNT(*) FROM schema_migrations WHERE version = 1", [], |r| r.get(0))?;
    if v1 == 0 {
        if customers_exists == 0 {
            conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql"))?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (1)", [])?;
    }

    // If paused_at column already exists, migration 002 was applied before versioning was added
    let paused_at_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('time_sessions') WHERE name='paused_at'",
        [], |r| r.get(0))?;

    let v2: i64 = conn.query_row(
        "SELECT COUNT(*) FROM schema_migrations WHERE version = 2", [], |r| r.get(0))?;
    if v2 == 0 {
        if paused_at_exists == 0 {
            conn.execute_batch(include_str!("../../migrations/002_phase2_features.sql"))?;
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (2)", [])?;
    }

    // Migration 003: Remove pause functionality and duration_override
    let v3: i64 = conn.query_row(
        "SELECT COUNT(*) FROM schema_migrations WHERE version = 3", [], |r| r.get(0))?;
    if v3 == 0 {
        conn.execute_batch(include_str!("../../migrations/003_remove_pause.sql"))?;
        conn.execute("INSERT INTO schema_migrations (version) VALUES (3)", [])?;
    }

    // Migration 004: Fix active_session FK (ON DELETE CASCADE → ON DELETE SET NULL)
    // and recover the singleton row if it was accidentally cascade-deleted by migration 003.
    let v4: i64 = conn.query_row(
        "SELECT COUNT(*) FROM schema_migrations WHERE version = 4", [], |r| r.get(0))?;
    if v4 == 0 {
        conn.execute_batch(include_str!("../../migrations/004_fix_active_session_fk.sql"))?;
        conn.execute("INSERT INTO schema_migrations (version) VALUES (4)", [])?;
    }

    // Migration 005: Add servicenow_task_id to work_orders
    let v5: i64 = conn.query_row(
        "SELECT COUNT(*) FROM schema_migrations WHERE version = 5", [], |r| r.get(0))?;
    if v5 == 0 {
        conn.execute_batch(include_str!("../../migrations/005_servicenow.sql"))?;
        conn.execute("INSERT INTO schema_migrations (version) VALUES (5)", [])?;
    }

    // Migration 006: Create activity_types table with default seed data
    let v6: i64 = conn.query_row(
        "SELECT COUNT(*) FROM schema_migrations WHERE version = 6", [], |r| r.get(0))?;
    if v6 == 0 {
        conn.execute_batch(include_str!("../../migrations/006_activity_types.sql"))?;
        conn.execute("INSERT INTO schema_migrations (version) VALUES (6)", [])?;
    }

    Ok(())
}
