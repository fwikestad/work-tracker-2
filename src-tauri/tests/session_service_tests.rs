/// Phase 1 + Phase 2 integration tests for the session service layer.
/// Uses an in-memory SQLite database for isolation.
use app_lib::db::{init_test_db, initialize};
use app_lib::services::{session_service, summary_service};
use app_lib::models::error::AppError;
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

// ---------------------------------------------------------------------------
// TC-SESSION-07: pause when already paused — must return error
// ---------------------------------------------------------------------------

#[test]
fn tc_session_07_pause_when_already_paused_errors() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    session_service::pause_session(&conn).expect("first pause failed");

    // Second pause must return Err
    let result = session_service::pause_session(&conn);
    assert!(
        result.is_err(),
        "pause_session must error when already paused"
    );
}

// ---------------------------------------------------------------------------
// TC-SESSION-08: pause when no active session — must return error
// ---------------------------------------------------------------------------

#[test]
fn tc_session_08_pause_when_no_active_session_errors() {
    let conn = init_test_db().expect("DB init failed");

    let result = session_service::pause_session(&conn);
    assert!(
        result.is_err(),
        "pause_session must error when no session is active"
    );
}

// ---------------------------------------------------------------------------
// TC-SESSION-09: resume when session is not paused — must return error
// ---------------------------------------------------------------------------

#[test]
fn tc_session_09_resume_when_not_paused_errors() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    // Session is running but NOT paused — resume must fail
    let result = session_service::resume_session(&conn);
    assert!(
        result.is_err(),
        "resume_session must error when session is not paused"
    );
}

// ---------------------------------------------------------------------------
// TC-SESSION-10: stop a paused session — duration equals gross wall-clock time
// ---------------------------------------------------------------------------

#[test]
fn tc_session_10_stop_paused_session_uses_gross_duration() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");

    // Back-date start by 20 seconds
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-20 seconds') WHERE end_time IS NULL",
        [],
    )
    .expect("back-date start");

    // Pause, simulate a 10-second pause, then resume
    session_service::pause_session(&conn).expect("pause failed");
    conn.execute(
        "UPDATE active_session SET paused_session_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-10 seconds') WHERE id = 1",
        [],
    )
    .expect("back-date paused_session_at");
    conn.execute(
        "UPDATE time_sessions SET paused_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-10 seconds') WHERE end_time IS NULL",
        [],
    )
    .expect("back-date paused_at");
    session_service::resume_session(&conn).expect("resume failed");

    // Stop — should record gross duration (~20 s, includes the 10-second pause)
    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");

    assert!(stopped.end_time.is_some(), "end_time must be set");
    let dur = stopped.duration_seconds.unwrap_or(0);
    assert!(
        dur >= 15,
        "gross duration must be ≥ 15 s (was {}) — paused time should be included",
        dur
    );
}

// ---------------------------------------------------------------------------
// TC-SESSION-11: multiple pause/resume cycles accumulate total_paused_seconds
// ---------------------------------------------------------------------------

#[test]
fn tc_session_11_multiple_pause_resume_cycles_accumulate() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");

    // First cycle: 5-second pause
    session_service::pause_session(&conn).expect("pause 1 failed");
    conn.execute(
        "UPDATE active_session SET paused_session_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-5 seconds') WHERE id = 1",
        [],
    ).expect("back-date cycle 1");
    conn.execute(
        "UPDATE time_sessions SET paused_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-5 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date ts cycle 1");
    session_service::resume_session(&conn).expect("resume 1 failed");

    // Second cycle: 5-second pause
    session_service::pause_session(&conn).expect("pause 2 failed");
    conn.execute(
        "UPDATE active_session SET paused_session_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-5 seconds') WHERE id = 1",
        [],
    ).expect("back-date cycle 2");
    conn.execute(
        "UPDATE time_sessions SET paused_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-5 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date ts cycle 2");
    session_service::resume_session(&conn).expect("resume 2 failed");

    // total_paused_seconds must be ≥ 8 (two 5-second pauses, allow slight timing slack)
    let total_paused: i64 = conn
        .query_row(
            "SELECT COALESCE(total_paused_seconds, 0) FROM time_sessions WHERE end_time IS NULL",
            [],
            |row| row.get(0),
        )
        .expect("query total_paused_seconds");

    assert!(
        total_paused >= 8,
        "total_paused_seconds must accumulate across cycles (was {})",
        total_paused
    );

    // Session must stop cleanly
    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");

    assert!(stopped.end_time.is_some(), "end_time must be set after stop");
}

// ---------------------------------------------------------------------------
// TC-SESSION-12: switch while paused — old session stops, new session starts
// ---------------------------------------------------------------------------

#[test]
fn tc_session_12_switch_while_paused_stops_old_session() {
    let conn = init_test_db().expect("DB init failed");
    let (customer_id, work_order_a) = setup_customer_and_work_order(&conn);
    let work_order_b = add_work_order(&conn, &customer_id, "WO-B2");

    let session_a = session_service::switch_to_work_order(&conn, &work_order_a)
        .expect("switch to A failed");

    // Back-date start and then pause session A
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-10 seconds') WHERE id = ?",
        params![&session_a.id],
    ).expect("back-date A");
    session_service::pause_session(&conn).expect("pause A failed");

    // Switch to B while A is paused — must auto-stop A and start B running
    let session_b = session_service::switch_to_work_order(&conn, &work_order_b)
        .expect("switch to B while A paused");

    // Session A must now have end_time
    let end_time_a: Option<String> = conn
        .query_row(
            "SELECT end_time FROM time_sessions WHERE id = ?",
            params![&session_a.id],
            |row| row.get(0),
        )
        .expect("query session A end_time");
    assert!(end_time_a.is_some(), "session A must have end_time after switch");

    // active_session now points to session B
    let (active_sid, is_paused): (Option<String>, i64) = conn
        .query_row(
            "SELECT session_id, is_paused FROM active_session WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query active_session");

    assert_eq!(
        active_sid.as_deref(),
        Some(session_b.id.as_str()),
        "active_session must point to session B"
    );
    assert_eq!(is_paused, 0, "new session B must not be paused");

    // Invariant: only one open session
    let open_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM time_sessions WHERE end_time IS NULL",
            [],
            |row| row.get(0),
        )
        .expect("count open sessions");
    assert_eq!(open_count, 1, "at most 1 open session after switch");
}

// ---------------------------------------------------------------------------
// TC-SESSION-13: daily summary includes sessions that had paused intervals
// ---------------------------------------------------------------------------

#[test]
fn tc_session_13_daily_summary_includes_paused_session_gross_duration() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");

    // Back-date session start by 60 seconds
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-60 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date start");

    // Pause for ~20 seconds, then resume
    session_service::pause_session(&conn).expect("pause failed");
    conn.execute(
        "UPDATE active_session SET paused_session_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-20 seconds') WHERE id = 1",
        [],
    ).expect("back-date paused_session_at");
    conn.execute(
        "UPDATE time_sessions SET paused_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-20 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date paused_at");
    session_service::resume_session(&conn).expect("resume failed");

    // Stop the session
    session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");

    // Query daily summary for today
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let summary = summary_service::get_daily_summary(&conn, &today)
        .expect("daily summary failed");

    // Session must appear and total must reflect gross duration (~60 s, including 20 s pause)
    assert_eq!(summary.entries.len(), 1, "summary must contain one entry");
    assert!(
        summary.total_seconds >= 50,
        "summary total must include paused intervals (was {} s, expected ≥ 50 s)",
        summary.total_seconds
    );
    assert!(
        summary.entries[0].total_seconds >= 50,
        "entry total_seconds must include paused intervals (was {})",
        summary.entries[0].total_seconds
    );
}

// ---------------------------------------------------------------------------
// TC-SESSION-14: heartbeat during pause does not clear pause state
// ---------------------------------------------------------------------------

#[test]
fn tc_session_14_heartbeat_during_pause_preserves_pause_state() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    session_service::pause_session(&conn).expect("pause failed");

    // Capture heartbeat timestamp before update
    let heartbeat_before: Option<String> = conn
        .query_row(
            "SELECT last_heartbeat FROM active_session WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("query heartbeat before");

    // Heartbeat update must not affect pause state
    session_service::update_heartbeat(&conn).expect("update_heartbeat failed");

    let (is_paused, paused_at, heartbeat_after): (i64, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT is_paused, paused_session_at, last_heartbeat FROM active_session WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("query active_session post-heartbeat");

    assert_eq!(is_paused, 1, "is_paused must remain 1 after heartbeat");
    assert!(
        paused_at.is_some(),
        "paused_session_at must remain set after heartbeat"
    );
    assert!(
        heartbeat_after.is_some(),
        "last_heartbeat must be updated"
    );
    // Heartbeat timestamp should have advanced (or at least be set)
    assert_ne!(
        heartbeat_before, heartbeat_after,
        "last_heartbeat must change after update_heartbeat"
    );
}

// ===========================================================================
// EDIT START/END TIMES FEATURE TESTS (ISSUE #29)
// ===========================================================================

// ---------------------------------------------------------------------------
// TC-EDIT-01: update_session_times — happy path: update start_time
// ---------------------------------------------------------------------------

#[test]
fn tc_edit_01_update_start_time_recalculates_duration() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Create and stop a session
    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    
    // Back-date start by 60 seconds
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-60 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date start");
    
    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");
    
    let original_duration = stopped.duration_seconds.unwrap();
    
    // Update start_time to 120 seconds ago (doubling duration)
    let new_start = chrono::Utc::now() - chrono::Duration::seconds(120);
    let new_start_str = new_start.to_rfc3339();
    
    let updated = session_service::update_session_times(&conn, &stopped.id, Some(&new_start_str), None)
        .expect("update_session_times failed");
    
    // Verify start_time updated
    assert_eq!(updated.start_time, new_start_str, "start_time should be updated");
    
    // Verify duration recalculated (should roughly double)
    let new_duration = updated.duration_seconds.unwrap();
    assert!(new_duration >= original_duration * 2 - 5, 
        "duration should increase from {} to ~{}, got {}", 
        original_duration, original_duration * 2, new_duration);
    
    // Verify updated_at bumped
    assert!(updated.updated_at > stopped.updated_at, 
        "updated_at should be bumped after edit");
}

// ---------------------------------------------------------------------------
// TC-EDIT-02: update_session_times — happy path: update end_time
// ---------------------------------------------------------------------------

#[test]
fn tc_edit_02_update_end_time_recalculates_duration() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Create and stop a session
    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-60 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date start");
    
    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");
    
    let original_duration = stopped.duration_seconds.unwrap();
    let original_end = stopped.end_time.clone().unwrap();
    
    // Update end_time to 30 seconds earlier (halving duration)
    let new_end = chrono::Utc::now() - chrono::Duration::seconds(30);
    let new_end_str = new_end.to_rfc3339();
    
    let updated = session_service::update_session_times(&conn, &stopped.id, None, Some(&new_end_str))
        .expect("update_session_times failed");
    
    // Verify end_time updated
    assert_eq!(updated.end_time.unwrap(), new_end_str, "end_time should be updated");
    
    // Verify duration recalculated (should roughly halve)
    let new_duration = updated.duration_seconds.unwrap();
    assert!(new_duration >= original_duration / 2 - 5 && new_duration <= original_duration / 2 + 5,
        "duration should decrease from {} to ~{}, got {}", 
        original_duration, original_duration / 2, new_duration);
}

// ---------------------------------------------------------------------------
// TC-EDIT-03: update_session_times — happy path: update both start and end
// ---------------------------------------------------------------------------

#[test]
fn tc_edit_03_update_both_start_and_end_times() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Create and stop a session
    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-60 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date start");
    
    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");
    
    // Update both times to a new 2-hour window
    let new_start = chrono::Utc::now() - chrono::Duration::hours(3);
    let new_end = chrono::Utc::now() - chrono::Duration::hours(1);
    let new_start_str = new_start.to_rfc3339();
    let new_end_str = new_end.to_rfc3339();
    
    let updated = session_service::update_session_times(&conn, &stopped.id, Some(&new_start_str), Some(&new_end_str))
        .expect("update_session_times failed");
    
    // Verify both times updated
    assert_eq!(updated.start_time, new_start_str, "start_time should be updated");
    assert_eq!(updated.end_time.unwrap(), new_end_str, "end_time should be updated");
    
    // Verify duration is exactly 2 hours
    let new_duration = updated.duration_seconds.unwrap();
    assert!(new_duration >= 7195 && new_duration <= 7205,
        "duration should be ~7200 seconds (2 hours), got {}", new_duration);
}

// ---------------------------------------------------------------------------
// TC-EDIT-04: update_session_times — validation: start_time >= end_time rejected
// ---------------------------------------------------------------------------

#[test]
fn tc_edit_04_start_time_after_end_time_rejected() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Create and stop a session
    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-60 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date start");
    
    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");
    
    // Try to set start_time AFTER current end_time
    let invalid_start = chrono::Utc::now() + chrono::Duration::hours(1);
    let invalid_start_str = invalid_start.to_rfc3339();
    
    let result = session_service::update_session_times(&conn, &stopped.id, Some(&invalid_start_str), None);
    assert!(result.is_err(), "update_session_times must reject start_time >= end_time");
    
    // Verify error is Validation type
    match result {
        Err(AppError::Validation(msg)) => {
            assert!(msg.contains("start_time") && msg.contains("end_time"),
                "error message should mention start_time and end_time");
        },
        _ => panic!("expected AppError::Validation, got {:?}", result),
    }
}

// ---------------------------------------------------------------------------
// TC-EDIT-05: update_session_times — validation: zero duration rejected
// ---------------------------------------------------------------------------

#[test]
fn tc_edit_05_zero_duration_rejected() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Create and stop a session
    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-60 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date start");
    
    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");
    
    // Try to set end_time == start_time (zero duration)
    let same_time = stopped.start_time.clone();
    
    let result = session_service::update_session_times(&conn, &stopped.id, None, Some(&same_time));
    assert!(result.is_err(), "update_session_times must reject zero duration");
    
    // Verify error is Validation type
    match result {
        Err(AppError::Validation(msg)) => {
            assert!(msg.contains("duration") || msg.contains("zero"),
                "error message should mention duration or zero");
        },
        _ => panic!("expected AppError::Validation, got {:?}", result),
    }
}

// ---------------------------------------------------------------------------
// TC-EDIT-06: update_session_times — validation: future end_time rejected
// ---------------------------------------------------------------------------

#[test]
fn tc_edit_06_future_end_time_rejected() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Create and stop a session
    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-60 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date start");
    
    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");
    
    // Try to set end_time in the future (more than 5 minutes beyond now)
    let future_end = chrono::Utc::now() + chrono::Duration::minutes(10);
    let future_end_str = future_end.to_rfc3339();
    
    let result = session_service::update_session_times(&conn, &stopped.id, None, Some(&future_end_str));
    assert!(result.is_err(), "update_session_times must reject end_time too far in future");
    
    // Verify error is Validation type
    match result {
        Err(AppError::Validation(msg)) => {
            assert!(msg.contains("future") || msg.contains("end_time"),
                "error message should mention future or end_time");
        },
        _ => panic!("expected AppError::Validation, got {:?}", result),
    }
}

// ---------------------------------------------------------------------------
// TC-EDIT-07: update_session_times — validation: cannot edit running session
// ---------------------------------------------------------------------------

#[test]
fn tc_edit_07_cannot_edit_running_session() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Create a RUNNING session (no end_time)
    let session = session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    
    // Try to update start_time of running session
    let new_start = chrono::Utc::now() - chrono::Duration::hours(1);
    let new_start_str = new_start.to_rfc3339();
    
    let result = session_service::update_session_times(&conn, &session.id, Some(&new_start_str), None);
    assert!(result.is_err(), "update_session_times must reject editing running session");
    
    // Verify error is Validation type
    match result {
        Err(AppError::Validation(msg)) => {
            assert!(msg.contains("running") || msg.contains("active") || msg.contains("complete"),
                "error message should mention session state: {}", msg);
        },
        _ => panic!("expected AppError::Validation, got {:?}", result),
    }
}

// ---------------------------------------------------------------------------
// TC-EDIT-08: update_session_times — overlap prevention: editing creates overlap
// ---------------------------------------------------------------------------

#[test]
#[ignore = "TODO: implement update_session_times function"]
fn tc_edit_08_overlap_prevention_on_time_edit() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Create first session: 2 hours ago → 1 hour ago
    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-2 hours'), 
                                  end_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-1 hour'),
                                  duration_seconds = 3600
         WHERE end_time IS NULL",
        [],
    ).expect("create first completed session");
    
    // Clear active session for next session
    conn.execute("UPDATE active_session SET session_id = NULL WHERE id = 1", []).expect("clear active");
    
    // Create second session: now - 30 min → now
    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-30 minutes') WHERE end_time IS NULL",
        [],
    ).expect("back-date second session");
    
    let second_session = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");
    
    // Try to edit second session's start_time to overlap with first session
    // (e.g., 90 minutes ago, which would overlap with the first session's 2hr→1hr window)
    let overlap_start = chrono::Utc::now() - chrono::Duration::minutes(90);
    let overlap_start_str = overlap_start.to_rfc3339();
    
    // TODO: Call update_session_times — should detect overlap
    // let result = session_service::update_session_times(&conn, &second_session.id, Some(&overlap_start_str), None);
    // assert!(result.is_err(), "update_session_times must prevent overlapping sessions");
    
    // Verify error is Validation type mentioning overlap
    // match result {
    //     Err(AppError::Validation(msg)) => {
    //         assert!(msg.contains("overlap"),
    //             "error message should mention overlap: {}", msg);
    //     },
    //     _ => panic!("expected AppError::Validation with overlap message, got {:?}", result),
    // }
}

// ---------------------------------------------------------------------------
// TC-EDIT-09: update_session_times — duration_override interaction: cleared on edit
// ---------------------------------------------------------------------------

#[test]
fn tc_edit_09_duration_override_cleared_on_time_edit() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Create and stop a session
    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-60 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date start");
    
    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");
    
    // Manually set a duration_override
    conn.execute(
        "UPDATE time_sessions SET duration_override = 7200 WHERE id = ?",
        params![&stopped.id],
    ).expect("set duration_override");
    
    // Verify override is set
    let override_before: Option<i64> = conn
        .query_row(
            "SELECT duration_override FROM time_sessions WHERE id = ?",
            params![&stopped.id],
            |row| row.get(0),
        )
        .expect("query duration_override");
    assert_eq!(override_before, Some(7200), "duration_override should be set before edit");
    
    // Update start_time — this should clear duration_override
    let new_start = chrono::Utc::now() - chrono::Duration::hours(2);
    let new_start_str = new_start.to_rfc3339();
    
    let updated = session_service::update_session_times(&conn, &stopped.id, Some(&new_start_str), None)
        .expect("update_session_times failed");
    
    // DECISION: Duration override should be cleared when times are edited
    // Rationale: If user manually edits times, calculated duration is the new source of truth
    // Manual override would be stale/misleading
    
    // Verify duration_override is cleared
    assert!(updated.duration_override.is_none(), 
        "duration_override should be cleared when times are edited");
    
    // Alternative decision: KEEP duration_override (if team decides manual override always wins)
    // assert_eq!(updated.duration_override, Some(7200), 
    //     "duration_override should be preserved when times are edited");
}

// ---------------------------------------------------------------------------
// TC-EDIT-10: update_session_times — audit trail: updated_at bumped
// ---------------------------------------------------------------------------

#[test]
fn tc_edit_10_updated_at_bumped_on_time_edit() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Create and stop a session
    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-60 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date start");
    
    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");
    
    let original_updated_at = stopped.updated_at.clone();
    
    // Wait 1 second to ensure updated_at differs
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    // Update end_time
    let new_end = chrono::Utc::now();
    let new_end_str = new_end.to_rfc3339();
    
    let updated = session_service::update_session_times(&conn, &stopped.id, None, Some(&new_end_str))
        .expect("update_session_times failed");
    
    // Verify updated_at is newer
    assert!(updated.updated_at > original_updated_at,
        "updated_at should be bumped after edit: was {}, now {}",
        original_updated_at, updated.updated_at);
}

// ---------------------------------------------------------------------------
// TC-EDIT-11: update_session_times — validation: session must exist
// ---------------------------------------------------------------------------

#[test]
fn tc_edit_11_nonexistent_session_rejected() {
    let conn = init_test_db().expect("DB init failed");

    let fake_session_id = uuid::Uuid::new_v4().to_string();
    let new_start = chrono::Utc::now() - chrono::Duration::hours(1);
    let new_start_str = new_start.to_rfc3339();
    
    let result = session_service::update_session_times(&conn, &fake_session_id, Some(&new_start_str), None);
    assert!(result.is_err(), "update_session_times must reject nonexistent session");
    
    // Verify error is NotFound
    match result {
        Err(AppError::NotFound(msg)) => {
            assert!(msg.contains(&fake_session_id),
                "error message should mention session ID: {}", msg);
        },
        _ => panic!("expected AppError::NotFound, got {:?}", result),
    }
}

// ---------------------------------------------------------------------------
// TC-EDIT-12: update_session_times — tolerance: allow small future times
// ---------------------------------------------------------------------------

#[test]
fn tc_edit_12_allow_small_future_tolerance() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Create and stop a session
    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-60 seconds') WHERE end_time IS NULL",
        [],
    ).expect("back-date start");
    
    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");
    
    // Try to set end_time 2 minutes in the future (within tolerance for clock skew)
    let slightly_future = chrono::Utc::now() + chrono::Duration::minutes(2);
    let slightly_future_str = slightly_future.to_rfc3339();
    
    let result = session_service::update_session_times(&conn, &stopped.id, None, Some(&slightly_future_str));
    assert!(result.is_ok(), 
        "update_session_times should allow end_time within reasonable tolerance (e.g., 5 minutes): {:?}", 
        result);
    
    // Rationale: Allow small future times to handle:
    // 1. Clock skew between devices
    // 2. User correction when they forgot to stop timer
    // 3. Timezone confusion
    // Suggested tolerance: 5 minutes
}
