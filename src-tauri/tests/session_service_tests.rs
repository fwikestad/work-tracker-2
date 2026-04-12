/// Phase 1 integration tests for the session service layer.
/// Uses an in-memory SQLite database for isolation.
use app_lib::db::{init_test_db, initialize};
use app_lib::services::session_service;
use rusqlite::{Connection, params};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Insert a single customer and work order; return (customer_id, work_order_id).
fn setup_customer_and_work_order(conn: &Connection) -> (String, String) {
    let customer_id = uuid::Uuid::new_v4().to_string();
    let work_order_id = uuid::Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO customers (id, name, created_at, updated_at) \
         VALUES (?, 'Test Customer', datetime('now'), datetime('now'))",
        params![&customer_id],
    )
    .expect("insert customer");

    conn.execute(
        "INSERT INTO work_orders (id, customer_id, name, code, created_at, updated_at) \
         VALUES (?, ?, 'Test Work Order', 'WO-01', datetime('now'), datetime('now'))",
        params![&work_order_id, &customer_id],
    )
    .expect("insert work_order");

    (customer_id, work_order_id)
}

/// Create a second work order under the same customer.
fn add_work_order(conn: &Connection, customer_id: &str, code: &str) -> String {
    let work_order_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO work_orders (id, customer_id, name, code, created_at, updated_at) \
         VALUES (?, ?, 'Work Order B', ?, datetime('now'), datetime('now'))",
        params![&work_order_id, customer_id, code],
    )
    .expect("insert work_order_b");
    work_order_id
}

// ---------------------------------------------------------------------------
// TC-SESSION-01: switch_to_work_order — happy path
// ---------------------------------------------------------------------------

#[test]
fn tc_session_01_switch_to_work_order_happy_path() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    let session = session_service::switch_to_work_order(&conn, &work_order_id)
        .expect("switch_to_work_order failed");

    // Session was created
    assert!(!session.id.is_empty(), "session id should be set");
    assert!(session.end_time.is_none(), "new session must not have end_time");

    // active_session row updated
    let (active_sid, is_paused): (Option<String>, i64) = conn
        .query_row(
            "SELECT session_id, is_paused FROM active_session WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query active_session");

    assert_eq!(
        active_sid.as_deref(),
        Some(session.id.as_str()),
        "active_session.session_id should match created session"
    );
    assert_eq!(is_paused, 0, "is_paused should start at 0");

    // time_sessions row has NULL end_time
    let end_time: Option<String> = conn
        .query_row(
            "SELECT end_time FROM time_sessions WHERE id = ?",
            params![&session.id],
            |row| row.get(0),
        )
        .expect("query time_sessions");

    assert!(end_time.is_none(), "time_sessions.end_time should be NULL");
}

// ---------------------------------------------------------------------------
// TC-SESSION-02: stop_active_session — sets end_time and duration
// ---------------------------------------------------------------------------

#[test]
fn tc_session_02_stop_sets_end_time_and_duration() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    let session = session_service::switch_to_work_order(&conn, &work_order_id)
        .expect("switch failed");

    // Back-date start_time so duration will be > 0
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-5 seconds') WHERE id = ?",
        params![&session.id],
    )
    .expect("back-date start_time");

    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop_current_session failed")
        .expect("should return a stopped session");

    assert!(stopped.end_time.is_some(), "end_time must be set after stop");
    assert!(
        stopped.duration_seconds.unwrap_or(0) > 0,
        "duration_seconds must be > 0"
    );

    // active_session cleared
    let active_sid: Option<String> = conn
        .query_row(
            "SELECT session_id FROM active_session WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("query active_session");

    assert!(
        active_sid.is_none(),
        "active_session.session_id should be NULL after stop"
    );
}

// ---------------------------------------------------------------------------
// TC-SESSION-03: switch auto-stops previous session
// ---------------------------------------------------------------------------

#[test]
fn tc_session_03_switch_auto_stops_previous_session() {
    let conn = init_test_db().expect("DB init failed");
    let (customer_id, work_order_a) = setup_customer_and_work_order(&conn);
    let work_order_b = add_work_order(&conn, &customer_id, "WO-B");

    let session_a = session_service::switch_to_work_order(&conn, &work_order_a)
        .expect("switch to A failed");

    // Back-date so auto-stop will produce duration > 0
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-5 seconds') WHERE id = ?",
        params![&session_a.id],
    )
    .expect("back-date session A");

    // Switch to B — must implicitly stop A
    let _session_b = session_service::switch_to_work_order(&conn, &work_order_b)
        .expect("switch to B failed");

    // Session A now has an end_time
    let end_time_a: Option<String> = conn
        .query_row(
            "SELECT end_time FROM time_sessions WHERE id = ?",
            params![&session_a.id],
            |row| row.get(0),
        )
        .expect("query session A");

    assert!(end_time_a.is_some(), "session A must have end_time after auto-stop");

    // Only one open session remains
    let open_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM time_sessions WHERE end_time IS NULL",
            [],
            |row| row.get(0),
        )
        .expect("count open sessions");

    assert_eq!(open_count, 1, "invariant: at most 1 open session at a time");
}

// ---------------------------------------------------------------------------
// TC-SESSION-04: pause_session sets is_paused = 1
// ---------------------------------------------------------------------------

#[test]
fn tc_session_04_pause_sets_is_paused() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");

    // pause_session must return Ok(())
    let result = session_service::pause_session(&conn);
    assert!(result.is_ok(), "pause_session must return Ok(()): {:?}", result);

    // active_session.is_paused = 1 and paused_session_at IS NOT NULL
    let (is_paused, paused_at): (i64, Option<String>) = conn
        .query_row(
            "SELECT is_paused, paused_session_at FROM active_session WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query active_session");

    assert_eq!(is_paused, 1, "active_session.is_paused should be 1");
    assert!(
        paused_at.is_some(),
        "active_session.paused_session_at must be set"
    );

    // time_sessions.paused_at IS NOT NULL
    let session_paused_at: Option<String> = conn
        .query_row(
            "SELECT paused_at FROM time_sessions WHERE end_time IS NULL",
            [],
            |row| row.get(0),
        )
        .expect("query time_sessions paused_at");

    assert!(
        session_paused_at.is_some(),
        "time_sessions.paused_at must be set after pause"
    );
}

// ---------------------------------------------------------------------------
// TC-SESSION-05: resume_session accumulates paused time and clears pause state
// ---------------------------------------------------------------------------

#[test]
fn tc_session_05_resume_accumulates_paused_time() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    session_service::pause_session(&conn).expect("pause failed");

    // Simulate a 10-second pause by back-dating paused timestamps
    conn.execute(
        "UPDATE active_session SET paused_session_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-10 seconds') WHERE id = 1",
        [],
    )
    .expect("back-date paused_session_at");
    conn.execute(
        "UPDATE time_sessions SET paused_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-10 seconds') WHERE end_time IS NULL",
        [],
    )
    .expect("back-date time_sessions.paused_at");

    // resume_session must return Ok(())
    let result = session_service::resume_session(&conn);
    assert!(result.is_ok(), "resume_session must return Ok(()): {:?}", result);

    // Pause state cleared on active_session
    let (is_paused, paused_at): (i64, Option<String>) = conn
        .query_row(
            "SELECT is_paused, paused_session_at FROM active_session WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query active_session");

    assert_eq!(is_paused, 0, "active_session.is_paused must be 0 after resume");
    assert!(
        paused_at.is_none(),
        "active_session.paused_session_at must be NULL after resume"
    );

    // total_paused_seconds accumulated
    let total_paused: Option<i64> = conn
        .query_row(
            "SELECT total_paused_seconds FROM time_sessions WHERE end_time IS NULL",
            [],
            |row| row.get(0),
        )
        .expect("query total_paused_seconds");

    assert!(
        total_paused.unwrap_or(0) > 0,
        "total_paused_seconds must be > 0 after resume"
    );
}

// ---------------------------------------------------------------------------
// TC-SESSION-06: no overlapping sessions invariant
// ---------------------------------------------------------------------------

#[test]
fn tc_session_06_no_overlapping_sessions() {
    let conn = init_test_db().expect("DB init failed");
    let (customer_id, work_order_a) = setup_customer_and_work_order(&conn);
    let work_order_b = add_work_order(&conn, &customer_id, "WO-OVERLAP");

    session_service::switch_to_work_order(&conn, &work_order_a).expect("switch A");

    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-3 seconds') WHERE end_time IS NULL",
        [],
    )
    .expect("back-date");

    session_service::switch_to_work_order(&conn, &work_order_b).expect("switch B");

    let open_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM time_sessions WHERE end_time IS NULL",
            [],
            |row| row.get(0),
        )
        .expect("count open sessions");

    assert_eq!(
        open_count, 1,
        "invariant: never more than 1 open session"
    );
}

// ---------------------------------------------------------------------------
// TC-DATA-01: WAL mode is enabled for file-based databases
// ---------------------------------------------------------------------------

#[test]
fn tc_data_01_wal_mode_enabled() {
    let dir = std::env::temp_dir();
    let db_path = dir.join(format!("wt2_wal_test_{}.db", uuid::Uuid::new_v4()));

    let conn = initialize(&db_path).expect("initialize failed");

    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .expect("pragma journal_mode");

    assert_eq!(journal_mode, "wal", "WAL mode must be enabled on disk database");

    // Cleanup temp files
    drop(conn);
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(format!("{}-wal", db_path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", db_path.display()));
}

// ---------------------------------------------------------------------------
// TC-DATA-02: migrations create all expected tables and columns
// ---------------------------------------------------------------------------

#[test]
fn tc_data_02_migrations_run_cleanly() {
    let conn = init_test_db().expect("init_test_db failed");

    // All tables expected after both migrations
    let expected_tables = [
        "customers",
        "work_orders",
        "time_sessions",
        "active_session",
        "recent_work_orders",
        "schema_migrations",
    ];

    for table in &expected_tables {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
                params![table],
                |row| row.get(0),
            )
            .expect("query sqlite_master");

        assert_eq!(count, 1, "table '{}' must exist after migrations", table);
    }

    // Migration 002 added is_paused to active_session
    let col_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('active_session') WHERE name='is_paused'",
            [],
            |row| row.get(0),
        )
        .expect("pragma_table_info");

    assert_eq!(
        col_count, 1,
        "active_session.is_paused column must exist (migration 002)"
    );
}
