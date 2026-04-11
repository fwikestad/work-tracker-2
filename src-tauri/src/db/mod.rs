use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Connection>,
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

    Ok(())
}
