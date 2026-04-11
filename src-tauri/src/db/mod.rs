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
    conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql"))?;
    conn.execute_batch(include_str!("../../migrations/002_phase2_features.sql"))?;
    Ok(())
}
