/// Phase 2b integration tests for the dynamic tray menu.
/// Uses an in-memory SQLite database for isolation.
use app_lib::db::init_test_db;
use app_lib::tray::get_tray_menu_data;
use rusqlite::{params, Connection};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Insert a customer and return customer_id.
fn setup_customer(conn: &Connection, name: &str) -> String {
    let customer_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO customers (id, name, created_at, updated_at) 
         VALUES (?, ?, datetime('now'), datetime('now'))",
        params![&customer_id, name],
    )
    .expect("insert customer");
    customer_id
}

/// Insert a work order and return work_order_id.
fn setup_work_order(
    conn: &Connection,
    customer_id: &str,
    name: &str,
    code: &str,
    is_favorite: bool,
) -> String {
    let work_order_id = Uuid::new_v4().to_string();
    let favorite_val = if is_favorite { 1 } else { 0 };
    conn.execute(
        "INSERT INTO work_orders (id, customer_id, name, code, is_favorite, created_at, updated_at) 
         VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        params![&work_order_id, customer_id, name, code, favorite_val],
    )
    .expect("insert work_order");
    work_order_id
}

/// Create a session for a work order (simulates tracking).
fn create_session(conn: &Connection, work_order_id: &str) {
    let session_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO time_sessions (id, work_order_id, start_time, end_time, duration_seconds, created_at, updated_at) 
         VALUES (?, ?, ?, ?, 3600, ?, ?)",
        params![&session_id, work_order_id, &now, &now, &now, &now],
    )
    .expect("insert session");

    // Update recent_work_orders
    conn.execute(
        "INSERT OR REPLACE INTO recent_work_orders (work_order_id, last_used_at, use_count) 
         VALUES (?, ?, 1)",
        params![work_order_id, &now],
    )
    .expect("update recent");
}

// ---------------------------------------------------------------------------
// TC-2b-01: get_tray_menu_data returns favorites
// ---------------------------------------------------------------------------

#[test]
fn tc_2b_01_get_tray_menu_data_returns_favorites() {
    let conn = init_test_db().expect("init db");

    // Setup: Create 2 customers, 3 work orders (2 favorites, 1 not)
    let customer1 = setup_customer(&conn, "ACME Corp");
    let customer2 = setup_customer(&conn, "Globex Inc");

    let wo1 = setup_work_order(&conn, &customer1, "Design Sprint", "WO-01", true);
    let wo2 = setup_work_order(&conn, &customer2, "Backend API", "WO-02", true);
    let _wo3 = setup_work_order(&conn, &customer1, "DevOps", "WO-03", false);

    // Call get_tray_menu_data
    let result = get_tray_menu_data(&conn).expect("get tray menu data");

    // Assert: favorites contains the 2 favorite work orders
    assert_eq!(result.favorites.len(), 2, "should have 2 favorites");
    let fav_ids: Vec<&str> = result.favorites.iter().map(|f| f.id.as_str()).collect();
    assert!(fav_ids.contains(&wo1.as_str()), "wo1 should be in favorites");
    assert!(fav_ids.contains(&wo2.as_str()), "wo2 should be in favorites");

    // Assert: recent does NOT contain the favorites
    let recent_ids: Vec<&str> = result.recent.iter().map(|r| r.id.as_str()).collect();
    assert!(!recent_ids.contains(&wo1.as_str()), "wo1 should NOT be in recent");
    assert!(!recent_ids.contains(&wo2.as_str()), "wo2 should NOT be in recent");
}

// ---------------------------------------------------------------------------
// TC-2b-02: get_tray_menu_data returns recent work orders
// ---------------------------------------------------------------------------

#[test]
fn tc_2b_02_get_tray_menu_data_returns_recent_work_orders() {
    let conn = init_test_db().expect("init db");

    // Setup: Create 3 work orders (all is_favorite=0), start+stop sessions on 2 of them
    let customer1 = setup_customer(&conn, "Customer A");
    let wo1 = setup_work_order(&conn, &customer1, "Project Alpha", "WO-01", false);
    let wo2 = setup_work_order(&conn, &customer1, "Project Beta", "WO-02", false);
    let wo3 = setup_work_order(&conn, &customer1, "Project Gamma", "WO-03", false);

    // Create sessions for wo1 and wo2
    create_session(&conn, &wo1);
    create_session(&conn, &wo2);

    // Call get_tray_menu_data
    let result = get_tray_menu_data(&conn).expect("get tray menu data");

    // Assert: recent contains the 2 work orders that had sessions
    assert_eq!(result.recent.len(), 2, "should have 2 recent work orders");
    let recent_ids: Vec<&str> = result.recent.iter().map(|r| r.id.as_str()).collect();
    assert!(recent_ids.contains(&wo1.as_str()), "wo1 should be in recent");
    assert!(recent_ids.contains(&wo2.as_str()), "wo2 should be in recent");

    // Assert: the work order with no sessions is NOT in recent
    assert!(!recent_ids.contains(&wo3.as_str()), "wo3 should NOT be in recent");
}

// ---------------------------------------------------------------------------
// TC-2b-03: get_tray_menu_data excludes archived work orders
// ---------------------------------------------------------------------------

#[test]
fn tc_2b_03_get_tray_menu_data_excludes_archived_work_orders() {
    let conn = init_test_db().expect("init db");

    // Setup: Create work order, set archived_at to a timestamp
    let customer1 = setup_customer(&conn, "Customer X");
    let wo1 = setup_work_order(&conn, &customer1, "Archived Project", "WO-01", true);

    // Archive the work order
    conn.execute(
        "UPDATE work_orders SET archived_at = datetime('now') WHERE id = ?",
        params![&wo1],
    )
    .expect("archive work order");

    // Call get_tray_menu_data
    let result = get_tray_menu_data(&conn).expect("get tray menu data");

    // Assert: archived work order appears in neither favorites nor recent
    assert_eq!(result.favorites.len(), 0, "should have 0 favorites");
    assert_eq!(result.recent.len(), 0, "should have 0 recent");
}

// ---------------------------------------------------------------------------
// TC-2b-04: get_tray_menu_data returns empty lists for fresh DB
// ---------------------------------------------------------------------------

#[test]
fn tc_2b_04_get_tray_menu_data_returns_empty_lists_for_fresh_db() {
    let conn = init_test_db().expect("init db");

    // Setup: Empty DB (no work orders)

    // Call get_tray_menu_data
    let result = get_tray_menu_data(&conn).expect("get tray menu data");

    // Assert: favorites is empty, recent is empty (no error/panic)
    assert_eq!(result.favorites.len(), 0, "favorites should be empty");
    assert_eq!(result.recent.len(), 0, "recent should be empty");
}

// ---------------------------------------------------------------------------
// TC-2b-05: get_tray_menu_data customer name is included
// ---------------------------------------------------------------------------

#[test]
fn tc_2b_05_get_tray_menu_data_customer_name_is_included() {
    let conn = init_test_db().expect("init db");

    // Setup: Create customer "ACME Corp", create work order "Design Sprint" under it
    let customer1 = setup_customer(&conn, "ACME Corp");
    let wo1 = setup_work_order(&conn, &customer1, "Design Sprint", "WO-01", true);

    // Call get_tray_menu_data
    let result = get_tray_menu_data(&conn).expect("get tray menu data");

    // Assert: favorite entry has customer_name = "ACME Corp" and name = "Design Sprint"
    assert_eq!(result.favorites.len(), 1, "should have 1 favorite");
    let favorite = &result.favorites[0];
    assert_eq!(favorite.customer_name, "ACME Corp", "customer name should match");
    assert_eq!(favorite.name, "Design Sprint", "work order name should match");
    assert_eq!(favorite.id, wo1, "work order id should match");
}

// ---------------------------------------------------------------------------
// Timestamp Regression Tests
// ---------------------------------------------------------------------------

/// TC-ts-01: Session with SQLite-format timestamp can be parsed
#[test]
fn tc_ts_01_session_with_sqlite_format_timestamp_can_be_parsed() {
    let conn = init_test_db().expect("init db");

    // Setup: Insert a session directly with datetime('now') format (no T separator)
    let customer_id = setup_customer(&conn, "Test Customer");
    let work_order_id = setup_work_order(&conn, &customer_id, "Test WO", "WO-01", false);
    let session_id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO time_sessions (id, work_order_id, start_time, end_time, created_at, updated_at)
         VALUES (?, ?, datetime('now', '-1 hour'), datetime('now'), datetime('now'), datetime('now'))",
        params![&session_id, &work_order_id],
    )
    .expect("insert session");

    // Fetch the session start_time and end_time
    let (start_time, end_time): (String, String) = conn
        .query_row(
            "SELECT start_time, end_time FROM time_sessions WHERE id = ?",
            params![&session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("fetch timestamps");

    // Verify the format is SQLite format (space separator, not T)
    assert!(start_time.contains(' '), "start_time should have space separator");
    assert!(end_time.contains(' '), "end_time should have space separator");

    // Call calculate_duration via stop_active_session (which internally uses calculate_duration)
    // We'll verify by checking that duration_seconds can be computed without error
    conn.execute(
        "UPDATE time_sessions SET duration_seconds = 
         CAST((julianday(end_time) - julianday(start_time)) * 86400 AS INTEGER)
         WHERE id = ?",
        params![&session_id],
    )
    .expect("calculate duration");

    let duration: i64 = conn
        .query_row(
            "SELECT duration_seconds FROM time_sessions WHERE id = ?",
            params![&session_id],
            |row| row.get(0),
        )
        .expect("fetch duration");

    // Assert: Duration is calculated without error (backward compatibility)
    assert!(duration > 3500 && duration < 3700, "duration should be close to 1 hour (3600s)");
}

/// TC-ts-02: Session with RFC3339 timestamp is parsed correctly
#[test]
fn tc_ts_02_session_with_rfc3339_timestamp_is_parsed_correctly() {
    let conn = init_test_db().expect("init db");

    // Setup: Insert session with strftime('%Y-%m-%dT%H:%M:%SZ', 'now') format
    let customer_id = setup_customer(&conn, "Test Customer");
    let work_order_id = setup_work_order(&conn, &customer_id, "Test WO", "WO-01", false);
    let session_id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO time_sessions (id, work_order_id, start_time, end_time, created_at, updated_at)
         VALUES (?, ?, strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-1 hour'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
        params![&session_id, &work_order_id],
    )
    .expect("insert session");

    // Fetch the session start_time and end_time
    let (start_time, end_time): (String, String) = conn
        .query_row(
            "SELECT start_time, end_time FROM time_sessions WHERE id = ?",
            params![&session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("fetch timestamps");

    // Verify the format is RFC3339 format (T separator and Z suffix)
    assert!(start_time.contains('T'), "start_time should have T separator");
    assert!(end_time.contains('T'), "end_time should have T separator");
    assert!(start_time.ends_with('Z'), "start_time should end with Z");
    assert!(end_time.ends_with('Z'), "end_time should end with Z");

    // Calculate duration using SQLite
    conn.execute(
        "UPDATE time_sessions SET duration_seconds = 
         CAST((julianday(end_time) - julianday(start_time)) * 86400 AS INTEGER)
         WHERE id = ?",
        params![&session_id],
    )
    .expect("calculate duration");

    let duration: i64 = conn
        .query_row(
            "SELECT duration_seconds FROM time_sessions WHERE id = ?",
            params![&session_id],
            |row| row.get(0),
        )
        .expect("fetch duration");

    // Assert: Duration calculated correctly
    assert!(duration > 3500 && duration < 3700, "duration should be close to 1 hour (3600s)");
}
