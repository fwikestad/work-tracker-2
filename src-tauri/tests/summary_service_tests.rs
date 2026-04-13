/// Phase 3 integration tests for summary service (reports).
/// Tests get_report and export_csv functions.
use app_lib::db::init_test_db;
use app_lib::services::summary_service;
use rusqlite::{Connection, params};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Insert a single customer and work order; return (customer_id, work_order_id).
fn setup_customer_and_work_order(conn: &Connection, customer_name: &str, work_order_name: &str) -> (String, String) {
    let customer_id = uuid::Uuid::new_v4().to_string();
    let work_order_id = uuid::Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO customers (id, name, created_at, updated_at) \
         VALUES (?, ?, datetime('now'), datetime('now'))",
        params![&customer_id, customer_name],
    )
    .expect("insert customer");

    conn.execute(
        "INSERT INTO work_orders (id, customer_id, name, code, created_at, updated_at) \
         VALUES (?, ?, ?, 'WO-01', datetime('now'), datetime('now'))",
        params![&work_order_id, &customer_id, work_order_name],
    )
    .expect("insert work_order");

    (customer_id, work_order_id)
}

/// Insert a completed session with a specific date and duration.
fn insert_session(
    conn: &Connection,
    work_order_id: &str,
    date: &str,
    duration_seconds: i64,
) {
    let session_id = uuid::Uuid::new_v4().to_string();
    let start_time = format!("{} 09:00:00", date);
    let end_time = format!("{} 10:00:00", date);

    conn.execute(
        "INSERT INTO time_sessions (id, work_order_id, start_time, end_time, duration_seconds, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        params![&session_id, work_order_id, &start_time, &end_time, duration_seconds],
    )
    .expect("insert session");
}

// ---------------------------------------------------------------------------
// TC-SUMMARY-01: get_report with no data returns empty entries
// ---------------------------------------------------------------------------

#[test]
fn tc_summary_01_get_report_empty_date_range() {
    let conn = init_test_db().expect("DB init failed");
    setup_customer_and_work_order(&conn, "Customer A", "Work Order A");

    let result = summary_service::get_report(&conn, "2025-01-01", "2025-01-31")
        .expect("get_report failed");

    assert_eq!(result.entries.len(), 0, "should return zero entries for empty date range");
    assert_eq!(result.total_seconds, 0, "total_seconds should be 0");
    assert_eq!(result.sessions.len(), 0, "sessions should be empty");
}

// ---------------------------------------------------------------------------
// TC-SUMMARY-02: get_report with data returns correct aggregation
// ---------------------------------------------------------------------------

#[test]
fn tc_summary_02_get_report_aggregates_sessions() {
    let conn = init_test_db().expect("DB init failed");
    let (_, wo_id_1) = setup_customer_and_work_order(&conn, "Customer A", "Work Order 1");
    let (_, wo_id_2) = setup_customer_and_work_order(&conn, "Customer B", "Work Order 2");

    // Insert sessions across multiple days
    insert_session(&conn, &wo_id_1, "2025-04-01", 3600); // 1 hour
    insert_session(&conn, &wo_id_1, "2025-04-02", 1800); // 30 minutes
    insert_session(&conn, &wo_id_2, "2025-04-02", 7200); // 2 hours

    let result = summary_service::get_report(&conn, "2025-04-01", "2025-04-30")
        .expect("get_report failed");

    assert_eq!(result.entries.len(), 2, "should return 2 work order entries");
    assert_eq!(result.total_seconds, 3600 + 1800 + 7200, "total_seconds should sum all sessions");
    assert_eq!(result.sessions.len(), 3, "should return 3 sessions");

    // Check that entries are sorted by total_seconds DESC
    assert!(result.entries[0].total_seconds >= result.entries[1].total_seconds,
            "entries should be sorted by total_seconds DESC");
}

// ---------------------------------------------------------------------------
// TC-SUMMARY-03: export_csv returns valid CSV header
// ---------------------------------------------------------------------------

#[test]
fn tc_summary_03_export_csv_header() {
    let conn = init_test_db().expect("DB init failed");
    setup_customer_and_work_order(&conn, "Customer A", "Work Order A");

    let csv = summary_service::export_csv(&conn, "2025-04-01", "2025-04-30", false)
        .expect("export_csv failed");

    // Check header row
    assert!(csv.starts_with("Date,Customer,Work Order,Start Time,End Time,Duration (minutes),Activity Type,Notes\n"),
            "CSV should start with correct header row");
}

// ---------------------------------------------------------------------------
// TC-SUMMARY-04: export_csv with data includes rows
// ---------------------------------------------------------------------------

#[test]
fn tc_summary_04_export_csv_with_data() {
    let conn = init_test_db().expect("DB init failed");
    let (_, wo_id) = setup_customer_and_work_order(&conn, "Acme Corp", "Project Alpha");

    insert_session(&conn, &wo_id, "2025-04-15", 3600); // 1 hour = 60 minutes

    let csv = summary_service::export_csv(&conn, "2025-04-01", "2025-04-30", false)
        .expect("export_csv failed");

    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 2, "should have header + 1 data row");

    // Check data row contains customer and work order names
    assert!(lines[1].contains("Acme Corp"), "data row should contain customer name");
    assert!(lines[1].contains("Project Alpha"), "data row should contain work order name");
    assert!(lines[1].contains("60"), "data row should contain duration in minutes (60)");
}

// ---------------------------------------------------------------------------
// TC-SUMMARY-05: export_csv escapes commas in names
// ---------------------------------------------------------------------------

#[test]
fn tc_summary_05_export_csv_escapes_commas() {
    let conn = init_test_db().expect("DB init failed");
    let (_, wo_id) = setup_customer_and_work_order(&conn, "Smith, Jones & Co.", "Design, Dev & Deploy");

    insert_session(&conn, &wo_id, "2025-04-15", 1800);

    let csv = summary_service::export_csv(&conn, "2025-04-01", "2025-04-30", false)
        .expect("export_csv failed");

    // CSV escaping: fields with commas should be quoted
    assert!(csv.contains("\"Smith, Jones & Co.\""), "customer name with comma should be quoted");
    assert!(csv.contains("\"Design, Dev & Deploy\""), "work order name with comma should be quoted");
}

// ---------------------------------------------------------------------------
// TC-SUMMARY-06: get_report excludes incomplete sessions (end_time IS NULL)
// ---------------------------------------------------------------------------

#[test]
fn tc_summary_06_get_report_excludes_incomplete_sessions() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn, "Customer A", "Work Order A");

    // Insert completed session
    insert_session(&conn, &work_order_id, "2025-04-15", 3600);

    // Insert incomplete session (start but no end)
    let incomplete_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO time_sessions (id, work_order_id, start_time, end_time, created_at, updated_at) \
         VALUES (?, ?, '2025-04-15 14:00:00', NULL, datetime('now'), datetime('now'))",
        params![&incomplete_id, &work_order_id],
    )
    .expect("insert incomplete session");

    let result = summary_service::get_report(&conn, "2025-04-01", "2025-04-30")
        .expect("get_report failed");

    assert_eq!(result.sessions.len(), 1, "should exclude incomplete session");
    assert_eq!(result.total_seconds, 3600, "should only count completed session");
}

// ---------------------------------------------------------------------------
// TC-SUMMARY-07: get_report respects date range boundaries
// ---------------------------------------------------------------------------

#[test]
fn tc_summary_07_get_report_date_boundaries() {
    let conn = init_test_db().expect("DB init failed");
    let (_, work_order_id) = setup_customer_and_work_order(&conn, "Customer A", "Work Order A");

    // Insert sessions: one before, one inside, one after the range
    insert_session(&conn, &work_order_id, "2025-03-31", 1000); // before
    insert_session(&conn, &work_order_id, "2025-04-15", 2000); // inside
    insert_session(&conn, &work_order_id, "2025-05-01", 3000); // after

    let result = summary_service::get_report(&conn, "2025-04-01", "2025-04-30")
        .expect("get_report failed");

    assert_eq!(result.sessions.len(), 1, "should only include session within range");
    assert_eq!(result.total_seconds, 2000, "should only sum session within range");
}

// ---------------------------------------------------------------------------
// TC-ROUND-01: floor_to_half_hour rounds correctly for all cases
// ---------------------------------------------------------------------------

#[test]
fn tc_round_01_floor_to_half_hour() {
    use app_lib::services::summary_service::floor_to_half_hour;
    use chrono::NaiveDateTime;

    let cases = [
        ("2025-04-15 09:17:00", "2025-04-15 09:00:00"),
        ("2025-04-15 09:47:00", "2025-04-15 09:30:00"),
        ("2025-04-15 10:05:00", "2025-04-15 10:00:00"),
        ("2025-04-15 14:58:00", "2025-04-15 14:30:00"),
        ("2025-04-15 09:00:00", "2025-04-15 09:00:00"), // already on boundary
        ("2025-04-15 09:30:00", "2025-04-15 09:30:00"), // already on boundary
    ];

    for (input, expected) in cases {
        let dt = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").unwrap();
        let floored = floor_to_half_hour(dt);
        let expected_dt = NaiveDateTime::parse_from_str(expected, "%Y-%m-%d %H:%M:%S").unwrap();
        assert_eq!(floored, expected_dt, "floor_to_half_hour({}) should be {}", input, expected);
    }
}

// ---------------------------------------------------------------------------
// TC-ROUND-02: export_csv with rounding extends duration to started half-hour
// ---------------------------------------------------------------------------

#[test]
fn tc_round_02_export_csv_rounds_duration() {
    use rusqlite::params;

    let conn = init_test_db().expect("DB init failed");
    let (_, wo_id) = setup_customer_and_work_order(&conn, "Acme Corp", "Project Beta");

    // Session: 09:17 → 10:20 (63 min raw). With rounding: 09:00 → 10:20 = 80 min.
    let session_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO time_sessions (id, work_order_id, start_time, end_time, duration_seconds, created_at, updated_at) \
         VALUES (?, ?, '2025-04-15 09:17:00', '2025-04-15 10:20:00', 3780, datetime('now'), datetime('now'))",
        params![&session_id, &wo_id],
    ).expect("insert session");

    // Without rounding: 63 minutes
    let csv_plain = summary_service::export_csv(&conn, "2025-04-01", "2025-04-30", false)
        .expect("export_csv failed");
    let plain_row = csv_plain.lines().nth(1).unwrap();
    assert!(plain_row.contains(",63,"), "plain export should show 63 minutes");

    // With rounding: 09:00 → 10:20 = 80 minutes
    let csv_rounded = summary_service::export_csv(&conn, "2025-04-01", "2025-04-30", true)
        .expect("export_csv rounded failed");
    let rounded_row = csv_rounded.lines().nth(1).unwrap();
    assert!(rounded_row.contains(",80,"), "rounded export should show 80 minutes (09:00→10:20)");
}

// ---------------------------------------------------------------------------
// TC-ROUND-03: rounding respects duration_override (manual edits win)
// ---------------------------------------------------------------------------

#[test]
fn tc_round_03_rounding_respects_override() {
    use rusqlite::params;

    let conn = init_test_db().expect("DB init failed");
    let (_, wo_id) = setup_customer_and_work_order(&conn, "Override Corp", "Manual Project");

    // Session with a manual override of 45 minutes (2700s)
    let session_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO time_sessions (id, work_order_id, start_time, end_time, duration_seconds, duration_override, created_at, updated_at) \
         VALUES (?, ?, '2025-04-15 09:17:00', '2025-04-15 10:20:00', 3780, 2700, datetime('now'), datetime('now'))",
        params![&session_id, &wo_id],
    ).expect("insert session with override");

    // Even with rounding enabled, override should win → 45 minutes
    let csv = summary_service::export_csv(&conn, "2025-04-01", "2025-04-30", true)
        .expect("export_csv failed");
    let row = csv.lines().nth(1).unwrap();
    assert!(row.contains(",45,"), "override should win over rounding (45 min)");
}

// ---------------------------------------------------------------------------
// TC-ROUND-04: get/set round_to_half_hour setting persists correctly
// ---------------------------------------------------------------------------

#[test]
fn tc_round_04_setting_persists() {
    use rusqlite::params;

    let conn = init_test_db().expect("DB init failed");

    // Default from migration should be false
    let default_val = summary_service::get_round_to_half_hour(&conn)
        .expect("get_round_to_half_hour failed");
    assert!(!default_val, "default should be false");

    // Set to true
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('round_to_half_hour', 'true')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![],
    ).expect("update setting");

    let updated_val = summary_service::get_round_to_half_hour(&conn)
        .expect("get_round_to_half_hour after update failed");
    assert!(updated_val, "should now be true");
}
