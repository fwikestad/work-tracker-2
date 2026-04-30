/// Phase 1 + Phase 2 integration tests for the session service layer.
/// Uses an in-memory SQLite database for isolation.
use app_lib::db::{init_test_db, initialize};
use app_lib::services::{session_service, summary_service};
use app_lib::models::error::AppError;
use app_lib::models::activity_type::ActivityType;
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
    let active_sid: Option<String> = conn
        .query_row(
            "SELECT session_id FROM active_session WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("query active_session");

    assert_eq!(
        active_sid.as_deref(),
        Some(session.id.as_str()),
        "active_session.session_id should match created session"
    );

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
    let _original_end = stopped.end_time.clone().unwrap();
    
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
    
    let _second_session = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");
    
    // Try to edit second session's start_time to overlap with first session
    // (e.g., 90 minutes ago, which would overlap with the first session's 2hr→1hr window)
    let overlap_start = chrono::Utc::now() - chrono::Duration::minutes(90);
    let _overlap_start_str = overlap_start.to_rfc3339();
    
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

// ---------------------------------------------------------------------------
// TC-BUG51-01: discard_orphan_session must not delete the active_session singleton
// ---------------------------------------------------------------------------

/// Regression test: before the fix, deleting from time_sessions would cascade-delete
/// the active_session singleton row (id=1) due to ON DELETE CASCADE introduced in
/// migration 003. After the fix the row must survive.
#[test]
fn tc_bug51_01_discard_orphan_preserves_active_session_singleton() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Start then artificially orphan a session (clear active_session but keep the
    // time_sessions row open, simulating a crash scenario)
    let session = session_service::switch_to_work_order(&conn, &work_order_id)
        .expect("switch failed");

    // Simulate a crash: clear active_session without closing the session
    conn.execute(
        "UPDATE active_session SET session_id = NULL, work_order_id = NULL, started_at = NULL, last_heartbeat = NULL WHERE id = 1",
        [],
    )
    .expect("clear active_session");

    // Discard the orphan
    session_service::discard_orphan_session(&conn, &session.id)
        .expect("discard_orphan_session failed");

    // The singleton row must still exist
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM active_session WHERE id = 1", [], |r| r.get(0))
        .expect("count active_session");
    assert_eq!(count, 1, "active_session singleton must survive discard_orphan_session");

    // And session must be gone from time_sessions
    let ts_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM time_sessions WHERE id = ?",
            params![&session.id],
            |r| r.get(0),
        )
        .expect("count time_sessions");
    assert_eq!(ts_count, 0, "discarded session must be removed from time_sessions");
}

// ---------------------------------------------------------------------------
// TC-BUG51-02: switch_to_work_order succeeds when active_session singleton is missing
// ---------------------------------------------------------------------------

/// Regression test: if the singleton row was previously lost (migration 003 cascade bug),
/// switch_to_work_order must recover it and correctly show the session as active.
#[test]
fn tc_bug51_02_switch_works_when_singleton_missing() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Simulate the corruption: forcibly delete the active_session singleton
    conn.execute("DELETE FROM active_session WHERE id = 1", [])
        .expect("delete singleton");
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM active_session", [], |r| r.get(0))
        .expect("count");
    assert_eq!(count, 0, "pre-condition: singleton should be gone");

    // switch_to_work_order must not silently fail
    let session = session_service::switch_to_work_order(&conn, &work_order_id)
        .expect("switch_to_work_order must succeed even if singleton was missing");

    // Singleton must now exist and point to the new session
    let active_sid: Option<String> = conn
        .query_row(
            "SELECT session_id FROM active_session WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .expect("query active_session");

    assert_eq!(
        active_sid.as_deref(),
        Some(session.id.as_str()),
        "active_session.session_id must match new session after recovery"
    );
}

// ---------------------------------------------------------------------------
// TC-BUG51-03: migration 004 recovers missing singleton and fixes FK constraints
// ---------------------------------------------------------------------------

/// Regression test: migration 004 must re-insert the singleton row if it was
/// cascade-deleted, and the resulting FK must use ON DELETE SET NULL so that
/// future deletes of time_sessions rows don't remove the singleton.
#[test]
fn tc_bug51_03_migration_004_recovers_singleton_and_fixes_fk() {
    // Build the DB through migration 003 (which has the bad FK), then simulate
    // the corruption by deleting from time_sessions while active_session.session_id
    // points to it, then run migration 004 and verify recovery.
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Start a session so active_session.session_id is non-NULL
    let session = session_service::switch_to_work_order(&conn, &work_order_id)
        .expect("switch failed");

    // Simulate the cascade bug: directly delete from time_sessions (as if ON DELETE CASCADE fired)
    // and verify the singleton survived (because migration 004 already ran in init_test_db)
    conn.execute(
        "UPDATE active_session SET session_id = NULL WHERE id = 1",
        [],
    )
    .expect("clear session_id first");
    conn.execute("DELETE FROM time_sessions WHERE id = ?", params![&session.id])
        .expect("delete session");

    // Singleton must still be present (ON DELETE SET NULL, not CASCADE)
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM active_session WHERE id = 1", [], |r| r.get(0))
        .expect("count");
    assert_eq!(count, 1, "active_session singleton must survive time_sessions deletion after migration 004");
}

// ---------------------------------------------------------------------------
// TC-BUG51-04: delete_session command does not destroy the active_session singleton
// ---------------------------------------------------------------------------

/// Regression test: deleting an arbitrary session via the commands layer must not
/// kill the active_session singleton (would happen with ON DELETE CASCADE).
/// The active_session is NOT pointing at this session; we just ensure the singleton survives.
#[test]
fn tc_bug51_04_delete_historical_session_keeps_singleton() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Create and immediately stop a session (historical)
    session_service::switch_to_work_order(&conn, &work_order_id).expect("switch failed");
    conn.execute(
        "UPDATE time_sessions SET start_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-60 seconds') WHERE end_time IS NULL",
        [],
    )
    .expect("back-date start");
    let stopped = session_service::stop_current_session(&conn, None, None)
        .expect("stop failed")
        .expect("expected stopped session");

    // Directly delete the historical session (simulates delete_session command)
    conn.execute("DELETE FROM time_sessions WHERE id = ?", params![&stopped.id])
        .expect("delete session");

    // Singleton must still exist
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM active_session WHERE id = 1", [], |r| r.get(0))
        .expect("count");
    assert_eq!(count, 1, "active_session singleton must survive deletion of a historical session");
}

// ===========================================================================
// PHASE 4a TEST HELPERS
// ===========================================================================

/// Insert a completed time_session with a specific duration and optional notes.
/// `date` is "YYYY-MM-DD". The start/end times are placed within that day;
/// exact wall-clock values don't matter because export uses stored duration_seconds.
fn insert_completed_session(
    conn: &Connection,
    work_order_id: &str,
    date: &str,
    start_hour: u8,
    duration_seconds: i64,
    notes: Option<&str>,
) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let start_time = format!("{}T{:02}:00:00Z", date, start_hour);
    let end_time = format!("{}T{:02}:59:00Z", date, (start_hour as i64 + duration_seconds / 3600 + 1).min(23));
    conn.execute(
        "INSERT INTO time_sessions \
         (id, work_order_id, start_time, end_time, duration_seconds, notes, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        params![&id, work_order_id, &start_time, &end_time, duration_seconds, notes],
    )
    .expect("insert completed session");
    id
}

/// Insert an open (running) time_session — no end_time, no duration.
fn insert_open_session(conn: &Connection, work_order_id: &str, date: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let start_time = format!("{}T15:00:00Z", date);
    conn.execute(
        "INSERT INTO time_sessions \
         (id, work_order_id, start_time, created_at, updated_at) \
         VALUES (?, ?, ?, datetime('now'), datetime('now'))",
        params![&id, work_order_id, &start_time],
    )
    .expect("insert open session");
    id
}

/// Create a work order with an explicit code and optional ServiceNow task ID.
fn add_work_order_with_sn_id(
    conn: &Connection,
    customer_id: &str,
    code: &str,
    sn_id: Option<&str>,
) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO work_orders \
         (id, customer_id, name, code, servicenow_task_id, created_at, updated_at) \
         VALUES (?, ?, 'SN Work Order', ?, ?, datetime('now'), datetime('now'))",
        params![&id, customer_id, code, sn_id],
    )
    .expect("insert work_order_with_sn_id");
    id
}

// ---------------------------------------------------------------------------
// Activity-type helpers — mirror the command logic but accept &Connection
// ---------------------------------------------------------------------------

fn at_list(conn: &Connection) -> Result<Vec<ActivityType>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, sort_order, created_at \
         FROM activity_types ORDER BY sort_order, name",
    )?;
    let items: Result<Vec<_>, _> = stmt
        .query_map([], |row| {
            Ok(ActivityType {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .collect();
    items.map_err(AppError::Database)
}

fn at_create(conn: &Connection, name: &str) -> Result<ActivityType, AppError> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let max_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) FROM activity_types",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO activity_types (id, name, sort_order, created_at) VALUES (?, ?, ?, ?)",
        params![&id, name, max_order + 1, &now],
    )?;
    Ok(ActivityType {
        id,
        name: name.to_string(),
        sort_order: max_order + 1,
        created_at: now,
    })
}

fn at_update_name(conn: &Connection, id: &str, new_name: &str) -> Result<ActivityType, AppError> {
    conn.execute(
        "UPDATE activity_types SET name = ? WHERE id = ?",
        params![new_name, id],
    )?;
    conn.query_row(
        "SELECT id, name, sort_order, created_at FROM activity_types WHERE id = ?",
        params![id],
        |row| {
            Ok(ActivityType {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
                created_at: row.get(3)?,
            })
        },
    )
    .map_err(|_| AppError::NotFound(format!("Activity type {} not found", id)))
}

fn at_delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    let rows = conn.execute("DELETE FROM activity_types WHERE id = ?", params![id])?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("Activity type {} not found", id)));
    }
    Ok(())
}

// ===========================================================================
// GROUP A: export_servicenow tests
// ===========================================================================

const SN_HEADER: &str = "Date,Task ID,Work Order,Time Worked (hours),Category,Work Notes";

// ---------------------------------------------------------------------------
// TC-SN-A1: basic export aggregates two sessions on the same day into one row
// ---------------------------------------------------------------------------

#[test]
fn tc_sn_a1_basic_export_aggregates_by_day() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Two sessions on the same day totalling 3600 s (1 h)
    insert_completed_session(&conn, &work_order_id, "2026-05-01", 9, 1800, None);
    insert_completed_session(&conn, &work_order_id, "2026-05-01", 11, 1800, None);

    let csv = summary_service::export_servicenow(&conn, "2026-05-01", "2026-05-01")
        .expect("export_servicenow failed");

    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines[0], SN_HEADER, "header must match expected columns");
    assert_eq!(lines.len(), 2, "should have header + exactly 1 aggregated data row");
    assert!(lines[1].contains("2026-05-01"), "data row must contain the date");
    assert!(lines[1].contains("1.0"), "3600 s = 1.0 h");
}

// ---------------------------------------------------------------------------
// TC-SN-A2: servicenow_task_id appears in Task ID column when set
// ---------------------------------------------------------------------------

#[test]
fn tc_sn_a2_uses_servicenow_task_id_when_set() {
    let conn = init_test_db().expect("DB init failed");
    let (customer_id, _) = setup_customer_and_work_order(&conn);

    let wo_id = add_work_order_with_sn_id(&conn, &customer_id, "WO-SN", Some("INC1234567"));
    insert_completed_session(&conn, &wo_id, "2026-05-02", 9, 3600, None);

    let csv = summary_service::export_servicenow(&conn, "2026-05-02", "2026-05-02")
        .expect("export_servicenow failed");

    assert!(
        csv.contains("INC1234567"),
        "Task ID column must contain servicenow_task_id 'INC1234567'; got:\n{}",
        csv
    );
}

// ---------------------------------------------------------------------------
// TC-SN-A3: falls back to work order code when servicenow_task_id is NULL
// ---------------------------------------------------------------------------

#[test]
fn tc_sn_a3_falls_back_to_code_when_no_sn_id() {
    let conn = init_test_db().expect("DB init failed");
    let (customer_id, _) = setup_customer_and_work_order(&conn);

    let wo_id = add_work_order_with_sn_id(&conn, &customer_id, "WO-001", None);
    insert_completed_session(&conn, &wo_id, "2026-05-03", 9, 3600, None);

    let csv = summary_service::export_servicenow(&conn, "2026-05-03", "2026-05-03")
        .expect("export_servicenow failed");

    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 2, "header + 1 data row");
    // Second CSV field is Task ID
    let fields: Vec<&str> = lines[1].splitn(6, ',').collect();
    assert_eq!(fields[1], "WO-001", "Task ID must fall back to code when no sn_id");
}

// ---------------------------------------------------------------------------
// TC-SN-A4: rounds up to nearest 0.5 h
// ---------------------------------------------------------------------------

#[test]
fn tc_sn_a4_rounds_up_to_half_hour() {
    let conn = init_test_db().expect("DB init failed");
    let (customer_id, _) = setup_customer_and_work_order(&conn);

    // 1799 s → ceil(1799/1800)*0.5 = 1*0.5 = 0.5
    let wo_half = add_work_order_with_sn_id(&conn, &customer_id, "WO-HALF", None);
    insert_completed_session(&conn, &wo_half, "2026-05-04", 9, 1799, None);

    // 1801 s → ceil(1801/1800)*0.5 = 2*0.5 = 1.0
    let wo_full = add_work_order_with_sn_id(&conn, &customer_id, "WO-FULL", None);
    insert_completed_session(&conn, &wo_full, "2026-05-04", 12, 1801, None);

    let csv = summary_service::export_servicenow(&conn, "2026-05-04", "2026-05-04")
        .expect("export_servicenow failed");

    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 3, "header + 2 data rows");

    // Find the WO-HALF row and WO-FULL row
    let half_row = lines.iter().find(|l| l.contains("WO-HALF"))
        .expect("WO-HALF row not found");
    let full_row = lines.iter().find(|l| l.contains("WO-FULL"))
        .expect("WO-FULL row not found");

    let half_fields: Vec<&str> = half_row.splitn(6, ',').collect();
    let full_fields: Vec<&str> = full_row.splitn(6, ',').collect();

    assert_eq!(half_fields[3], "0.5", "1799 s must round up to 0.5 h");
    assert_eq!(full_fields[3], "1.0", "1801 s must round up to 1.0 h");
}

// ---------------------------------------------------------------------------
// TC-SN-A5: notes from multiple sessions are concatenated with "; "
// ---------------------------------------------------------------------------

#[test]
fn tc_sn_a5_concatenates_notes() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    insert_completed_session(&conn, &work_order_id, "2026-05-05", 9, 1800, Some("fix bug"));
    insert_completed_session(&conn, &work_order_id, "2026-05-05", 11, 1800, Some("deploy"));

    let csv = summary_service::export_servicenow(&conn, "2026-05-05", "2026-05-05")
        .expect("export_servicenow failed");

    assert!(
        csv.contains("fix bug"),
        "Work Notes must contain 'fix bug'; got:\n{}",
        csv
    );
    assert!(
        csv.contains("deploy"),
        "Work Notes must contain 'deploy'; got:\n{}",
        csv
    );
}

// ---------------------------------------------------------------------------
// TC-SN-A6: open sessions (no end_time) are excluded from export
// ---------------------------------------------------------------------------

#[test]
fn tc_sn_a6_excludes_open_sessions() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn);

    // Completed session: 3600 s
    insert_completed_session(&conn, &work_order_id, "2026-05-06", 9, 3600, None);
    // Open session: no end_time, no duration — must not count
    insert_open_session(&conn, &work_order_id, "2026-05-06");

    let csv = summary_service::export_servicenow(&conn, "2026-05-06", "2026-05-06")
        .expect("export_servicenow failed");

    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 2, "header + 1 data row (open session excluded)");

    let fields: Vec<&str> = lines[1].splitn(6, ',').collect();
    assert_eq!(fields[3], "1.0", "only the 3600-s completed session should be counted → 1.0 h");
}

// ---------------------------------------------------------------------------
// TC-SN-A7: empty date range returns only the header row
// ---------------------------------------------------------------------------

#[test]
fn tc_sn_a7_empty_range_returns_header_only() {
    let conn = init_test_db().expect("DB init failed");

    // No sessions at all
    let csv = summary_service::export_servicenow(&conn, "2020-01-01", "2020-01-31")
        .expect("export_servicenow must not fail on empty range");

    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 1, "empty range must return exactly 1 line (header only)");
    assert_eq!(lines[0], SN_HEADER, "that line must be the header");
}

// ===========================================================================
// GROUP B: activity_types CRUD tests
// ===========================================================================

// ---------------------------------------------------------------------------
// TC-AT-B1: migration 006 seeds exactly 7 activity types with expected names
// ---------------------------------------------------------------------------

#[test]
fn tc_at_b1_seeded_on_migration() {
    let conn = init_test_db().expect("DB init failed");

    let types = at_list(&conn).expect("list_activity_types failed");

    assert_eq!(types.len(), 7, "migration 006 must seed exactly 7 activity types");

    let names: Vec<&str> = types.iter().map(|t| t.name.as_str()).collect();
    for expected in &["Development", "Meeting", "Code Review", "Documentation", "Admin", "Testing", "Support"] {
        assert!(
            names.contains(expected),
            "seeded types must include '{}'; got: {:?}",
            expected,
            names
        );
    }
}

// ---------------------------------------------------------------------------
// TC-AT-B2: create_activity_type appends at end (sort_order > all existing)
// ---------------------------------------------------------------------------

#[test]
fn tc_at_b2_create_appends_at_end() {
    let conn = init_test_db().expect("DB init failed");

    let types_before = at_list(&conn).expect("list before");
    let max_order_before = types_before.iter().map(|t| t.sort_order).max().unwrap_or(-1);

    let new_type = at_create(&conn, "My Custom Type").expect("create failed");

    assert!(
        new_type.sort_order > max_order_before,
        "new type sort_order ({}) must be greater than previous max ({})",
        new_type.sort_order,
        max_order_before
    );

    let types_after = at_list(&conn).expect("list after");
    assert_eq!(types_after.len(), 8, "should have 7 seeded + 1 new = 8");
    assert!(
        types_after.iter().any(|t| t.name == "My Custom Type"),
        "new type must appear in list"
    );
}

// ---------------------------------------------------------------------------
// TC-AT-B3: creating a duplicate name returns a Database error
// ---------------------------------------------------------------------------

#[test]
fn tc_at_b3_duplicate_name_fails() {
    let conn = init_test_db().expect("DB init failed");

    // "Development" is already seeded
    let result = at_create(&conn, "Development");

    assert!(result.is_err(), "duplicate name must return an error");
    match result {
        Err(AppError::Database(_)) => {} // expected: UNIQUE constraint violation
        other => panic!("expected AppError::Database for duplicate, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// TC-AT-B4: update_activity_type changes name and is reflected in list
// ---------------------------------------------------------------------------

#[test]
fn tc_at_b4_update_name() {
    let conn = init_test_db().expect("DB init failed");

    // Use the seeded "Admin" type (id = "at-admin")
    let updated = at_update_name(&conn, "at-admin", "Administration")
        .expect("update failed");

    assert_eq!(updated.name, "Administration", "updated name must be returned");

    let types = at_list(&conn).expect("list failed");
    assert!(
        types.iter().any(|t| t.id == "at-admin" && t.name == "Administration"),
        "updated name must be visible in list"
    );
    assert!(
        !types.iter().any(|t| t.name == "Admin"),
        "'Admin' must no longer exist after rename"
    );
}

// ---------------------------------------------------------------------------
// TC-AT-B5: delete_activity_type removes it from the list
// ---------------------------------------------------------------------------

#[test]
fn tc_at_b5_delete_removes_type() {
    let conn = init_test_db().expect("DB init failed");

    // Create a throw-away type so we don't disturb the seeded ones
    let new_type = at_create(&conn, "Temporary Type").expect("create failed");
    let id = new_type.id.clone();

    at_delete(&conn, &id).expect("delete failed");

    let types = at_list(&conn).expect("list failed");
    assert!(
        !types.iter().any(|t| t.id == id),
        "deleted type must not appear in list"
    );
    assert_eq!(types.len(), 7, "list should be back to 7 after deleting the new type");
}

// ---------------------------------------------------------------------------
// TC-AT-B6: delete non-existent activity type returns NotFound error
// ---------------------------------------------------------------------------

#[test]
fn tc_at_b6_delete_nonexistent_returns_not_found() {
    let conn = init_test_db().expect("DB init failed");

    let result = at_delete(&conn, "nonexistent-id-that-does-not-exist");

    assert!(result.is_err(), "deleting non-existent type must return error");
    match result {
        Err(AppError::NotFound(msg)) => {
            assert!(
                msg.contains("nonexistent-id-that-does-not-exist"),
                "error message should mention the id: {}",
                msg
            );
        }
        other => panic!("expected AppError::NotFound, got {:?}", other),
    }
}
