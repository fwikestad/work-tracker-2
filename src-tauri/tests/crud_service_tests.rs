/// Phase 1 integration tests for CRUD and summary operations.
/// Uses an in-memory SQLite database for isolation.
use app_lib::db::init_test_db;
use app_lib::services::{session_service, summary_service};
use rusqlite::{Connection, params};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn create_customer(conn: &Connection, name: &str, code: Option<&str>) -> String {
    let customer_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO customers (id, name, code, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        params![&customer_id, name, code, &now, &now],
    )
    .expect("insert customer");
    customer_id
}

fn create_work_order(conn: &Connection, customer_id: &str, name: &str, code: &str) -> String {
    let work_order_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO work_orders (id, customer_id, name, code, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        params![&work_order_id, customer_id, name, code, &now, &now],
    )
    .expect("insert work_order");
    work_order_id
}

fn create_completed_session(
    conn: &Connection,
    work_order_id: &str,
    start_offset_seconds: i64,
    duration_seconds: i64,
) -> String {
    let session_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    // Create session with offset start time
    conn.execute(
        "INSERT INTO time_sessions (id, work_order_id, start_time, created_at, updated_at) VALUES (?, ?, strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?), ?, ?)",
        params![&session_id, work_order_id, format!("{} seconds", start_offset_seconds), &now, &now],
    )
    .expect("insert session");
    
    // Complete it with calculated end_time and duration
    conn.execute(
        "UPDATE time_sessions SET end_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?), duration_seconds = ? WHERE id = ?",
        params![format!("{} seconds", start_offset_seconds + duration_seconds), duration_seconds, &session_id],
    )
    .expect("complete session");
    
    session_id
}

// ---------------------------------------------------------------------------
// TC-CUSTOMER-01: create_customer happy path
// ---------------------------------------------------------------------------

#[test]
fn tc_customer_01_create_customer_happy_path() {
    let conn = init_test_db().expect("DB init failed");
    
    let customer_id = create_customer(&conn, "Test Customer", Some("TC01"));
    
    // Verify customer was created
    let (name, code, created_at, updated_at): (String, Option<String>, String, String) = conn
        .query_row(
            "SELECT name, code, created_at, updated_at FROM customers WHERE id = ?",
            params![&customer_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("query customer");
    
    assert_eq!(name, "Test Customer");
    assert_eq!(code, Some("TC01".to_string()));
    assert!(!created_at.is_empty(), "created_at must be set");
    assert!(!updated_at.is_empty(), "updated_at must be set");
}

// ---------------------------------------------------------------------------
// TC-CUSTOMER-02: list_customers returns all
// ---------------------------------------------------------------------------

#[test]
fn tc_customer_02_list_customers_returns_all() {
    let conn = init_test_db().expect("DB init failed");
    
    // Create 3 customers
    let _c1 = create_customer(&conn, "Customer A", None);
    let _c2 = create_customer(&conn, "Customer B", Some("CB"));
    let _c3 = create_customer(&conn, "Customer C", None);
    
    // Query all customers
    let mut stmt = conn
        .prepare("SELECT id, name FROM customers WHERE archived_at IS NULL ORDER BY name")
        .expect("prepare query");
    
    let customers: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("query")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect");
    
    assert_eq!(customers.len(), 3, "should return all 3 customers");
    assert_eq!(customers[0].1, "Customer A");
    assert_eq!(customers[1].1, "Customer B");
    assert_eq!(customers[2].1, "Customer C");
}

// ---------------------------------------------------------------------------
// TC-CUSTOMER-03: update_customer changes fields
// ---------------------------------------------------------------------------

#[test]
fn tc_customer_03_update_customer_changes_fields() {
    let conn = init_test_db().expect("DB init failed");
    
    let customer_id = create_customer(&conn, "Original Name", Some("ORIG"));
    
    // Fetch original updated_at
    let original_updated_at: String = conn
        .query_row(
            "SELECT updated_at FROM customers WHERE id = ?",
            params![&customer_id],
            |row| row.get(0),
        )
        .expect("query original updated_at");
    
    // Simulate a brief delay
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // Update customer name
    let new_updated_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE customers SET name = ?, updated_at = ? WHERE id = ?",
        params!["Updated Name", &new_updated_at, &customer_id],
    )
    .expect("update customer");
    
    // Verify changes persisted
    let (name, updated_at): (String, String) = conn
        .query_row(
            "SELECT name, updated_at FROM customers WHERE id = ?",
            params![&customer_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query updated customer");
    
    assert_eq!(name, "Updated Name", "name should be updated");
    assert_ne!(
        updated_at, original_updated_at,
        "updated_at should change"
    );
}

// ---------------------------------------------------------------------------
// TC-CUSTOMER-04: delete_customer cascades (soft delete via archived_at)
// ---------------------------------------------------------------------------

#[test]
fn tc_customer_04_archive_customer_preserves_data() {
    let conn = init_test_db().expect("DB init failed");
    
    let customer_id = create_customer(&conn, "Customer to Archive", None);
    let work_order_id = create_work_order(&conn, &customer_id, "Work Order", "WO-01");
    let _session_id = create_completed_session(&conn, &work_order_id, -3600, 1800);
    
    // Archive customer (soft delete)
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE customers SET archived_at = ? WHERE id = ?",
        params![&now, &customer_id],
    )
    .expect("archive customer");
    
    // Verify customer is archived
    let archived_at: Option<String> = conn
        .query_row(
            "SELECT archived_at FROM customers WHERE id = ?",
            params![&customer_id],
            |row| row.get(0),
        )
        .expect("query archived_at");
    
    assert!(archived_at.is_some(), "archived_at must be set");
    
    // Verify work order and session still exist (no cascade delete)
    let wo_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM work_orders WHERE id = ?",
            params![&work_order_id],
            |row| row.get(0),
        )
        .expect("query work_order");
    
    assert_eq!(wo_count, 1, "work order should still exist");
    
    let session_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM time_sessions WHERE work_order_id = ?",
            params![&work_order_id],
            |row| row.get(0),
        )
        .expect("query session");
    
    assert_eq!(session_count, 1, "session should still exist");
}

// ---------------------------------------------------------------------------
// TC-WORKORDER-01: create_work_order requires valid customer
// ---------------------------------------------------------------------------

#[test]
fn tc_workorder_01_create_requires_valid_customer() {
    let conn = init_test_db().expect("DB init failed");
    
    let non_existent_customer = Uuid::new_v4().to_string();
    let work_order_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    // Try to create work order with non-existent customer_id
    let result = conn.execute(
        "INSERT INTO work_orders (id, customer_id, name, code, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        params![&work_order_id, &non_existent_customer, "Test WO", "WO-01", &now, &now],
    );
    
    // Should fail due to foreign key constraint
    assert!(result.is_err(), "should fail with foreign key violation");
}

// ---------------------------------------------------------------------------
// TC-QUICKADD-01: quick_add creates all three entities atomically
// ---------------------------------------------------------------------------

#[test]
fn tc_quickadd_01_creates_all_entities_atomically() {
    let conn = init_test_db().expect("DB init failed");
    
    // Create first session
    let customer_id_1 = create_customer(&conn, "Existing Customer", None);
    let work_order_id_1 = create_work_order(&conn, &customer_id_1, "WO 1", "WO1");
    session_service::switch_to_work_order(&conn, &work_order_id_1)
        .expect("switch to WO1 failed");
    
    // quick_add should auto-stop the previous session and start a new one
    let params = app_lib::models::session::QuickAddParams {
        customer_name: Some("New Customer via QuickAdd".to_string()),
        customer_id: None,
        work_order_name: "New Work Order".to_string(),
        work_order_code: Some("QA-01".to_string()),
    };
    
    let result = session_service::quick_add(&conn, &params).expect("quick_add failed");
    
    // Verify customer was created
    assert!(!result.customer.id.is_empty(), "customer ID should be set");
    assert_eq!(result.customer.name, "New Customer via QuickAdd");
    
    // Verify work order was created
    assert!(!result.work_order.id.is_empty(), "work order ID should be set");
    assert_eq!(result.work_order.name, "New Work Order");
    assert_eq!(result.work_order.code, Some("QA-01".to_string()));
    assert_eq!(
        result.work_order.customer_id, result.customer.id,
        "work order should belong to new customer"
    );
    
    // Verify session was started
    assert!(!result.session.id.is_empty(), "session ID should be set");
    assert!(result.session.end_time.is_none(), "new session should be open");
    assert_eq!(
        result.session.work_order_id, result.work_order.id,
        "session should be for new work order"
    );
    
    // Verify previous session was auto-stopped
    let prev_end_time: Option<String> = conn
        .query_row(
            "SELECT end_time FROM time_sessions WHERE work_order_id = ?",
            params![&work_order_id_1],
            |row| row.get(0),
        )
        .expect("query prev session");
    
    assert!(
        prev_end_time.is_some(),
        "previous session should have been auto-stopped"
    );
    
    // Verify active_session points to new session
    let active_sid: Option<String> = conn
        .query_row(
            "SELECT session_id FROM active_session WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("query active_session");
    
    assert_eq!(
        active_sid.as_deref(),
        Some(result.session.id.as_str()),
        "active_session should point to new session"
    );
}

// ---------------------------------------------------------------------------
// TC-SUMMARY-01: daily summary aggregates correctly
// ---------------------------------------------------------------------------

#[test]
fn tc_summary_01_daily_summary_aggregates_correctly() {
    let conn = init_test_db().expect("DB init failed");
    
    let customer_a = create_customer(&conn, "Customer A", None);
    let customer_b = create_customer(&conn, "Customer B", None);
    
    let wo_a1 = create_work_order(&conn, &customer_a, "WO A1", "A1");
    let wo_a2 = create_work_order(&conn, &customer_a, "WO A2", "A2");
    let wo_b1 = create_work_order(&conn, &customer_b, "WO B1", "B1");
    
    // Create 3 completed sessions today
    // Session 1: WO A1, 1800 seconds (30 min)
    create_completed_session(&conn, &wo_a1, -3600, 1800);
    // Session 2: WO A2, 3600 seconds (60 min)
    create_completed_session(&conn, &wo_a2, -7200, 3600);
    // Session 3: WO B1, 900 seconds (15 min)
    create_completed_session(&conn, &wo_b1, -1800, 900);
    
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let summary = summary_service::get_daily_summary(&conn, &today)
        .expect("get_daily_summary failed");
    
    // Verify total
    assert_eq!(
        summary.total_seconds,
        1800 + 3600 + 900,
        "total should be sum of all sessions"
    );
    
    // Verify entries are grouped by customer and work order
    assert_eq!(summary.entries.len(), 3, "should have 3 distinct entries");
    
    // Find Customer A entries
    let customer_a_entries: Vec<_> = summary
        .entries
        .iter()
        .filter(|e| e.customer_name == "Customer A")
        .collect();
    assert_eq!(customer_a_entries.len(), 2, "Customer A should have 2 WOs");
    
    // Find Customer B entry
    let customer_b_entries: Vec<_> = summary
        .entries
        .iter()
        .filter(|e| e.customer_name == "Customer B")
        .collect();
    assert_eq!(customer_b_entries.len(), 1, "Customer B should have 1 WO");
    
    // Verify session count
    assert_eq!(summary.sessions.len(), 3, "should return 3 sessions");
}

// ---------------------------------------------------------------------------
// TC-SUMMARY-02: report excludes open sessions
// ---------------------------------------------------------------------------

#[test]
fn tc_summary_02_report_excludes_open_sessions() {
    let conn = init_test_db().expect("DB init failed");
    
    let customer_id = create_customer(&conn, "Test Customer", None);
    let work_order_id = create_work_order(&conn, &customer_id, "Test WO", "WO1");
    
    // Create 1 completed session
    create_completed_session(&conn, &work_order_id, -3600, 1800);
    
    // Create 1 open session (no end_time)
    session_service::switch_to_work_order(&conn, &work_order_id)
        .expect("switch failed");
    
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let report = summary_service::get_report(&conn, &today, &today)
        .expect("get_report failed");
    
    // Only completed session should appear in totals
    assert_eq!(
        report.total_seconds, 1800,
        "total should only include completed session"
    );
    
    // Only 1 session in the summary (the completed one)
    let completed_sessions: Vec<_> = report
        .sessions
        .iter()
        .filter(|s| s.end_time.is_some())
        .collect();
    
    assert_eq!(
        completed_sessions.len(),
        1,
        "should only count completed sessions"
    );
}
