# Work Tracker 2 — Phase 1 Test Plan

**Version**: 1.0  
**Author**: Wedge (Tester)  
**Date**: 2026-04-11  
**Status**: Ready for Development

---

## Overview

This test plan covers all Phase 1 functionality for Work Tracker 2. It is designed to:
- Guide manual testing during development
- Serve as the foundation for automated test suite (Rust integration + Svelte component tests)
- Verify data integrity, crash recovery, and performance targets
- Identify regressions before release

**Scope**: Phase 1 MVP (see architecture.md §6 Deliverables)

---

## Test Coverage Map

```
Backend (Rust/Tauri Commands)
├── Customer Management
├── Work Order Management  
├── Time Session Management
├── Quick-Add (atomic operation)
├── Crash Recovery
├── Summary & Reporting
└── Data Integrity & Invariants

Frontend (Svelte Integration)
├── Timer Component & Real-time Updates
├── Quick-Switch Interface
├── Quick-Add Overlay
├── Today's Work Log
├── Daily Summary Panel
├── Recovery Dialog
└── CSV Export

Performance & Non-Functional
├── Timer Update Latency
├── Context Switch Time
├── Search/Filter Response
├── Query Performance
└── Session Persistence Under Load
```

---

# 1. BACKEND IPC COMMAND TESTS (Rust Integration Tests)

These tests verify Tauri command handlers and database operations. Run with:
```bash
# Rust integration tests
cargo test --test '*' -- --nocapture

# Database tests (SQLite)
cargo test db:: -- --nocapture
```

---

## 1.1 Customer Management Commands

### TC-001: Create customer — Happy path
**Given:** Clean database with no customers  
**When:** `create_customer(name="Acme Corp", code="ACME", color="#8b5cf6")`  
**Then:**
- Command returns success with new customer ID (UUID format)
- Database contains customer with all fields set correctly
- `created_at` and `updated_at` are recent timestamps
- `archived_at` is NULL
**Priority:** P0

### TC-002: Create customer — Minimal fields
**Given:** Clean database  
**When:** `create_customer(name="Simple Co")`  
**Then:**
- Command succeeds
- `code` and `color` are NULL
- All other fields populated correctly
**Priority:** P1

### TC-003: Create customer — Name validation (required)
**Given:** Clean database  
**When:** `create_customer(name="")`  
**Then:**
- Command returns error: "Customer name is required"
- No database change
**Priority:** P0

### TC-004: Create customer — Name too long (300+ chars)
**Given:** Clean database  
**When:** `create_customer(name="A" * 300)`  
**Then:**
- Command succeeds (or truncates to reasonable limit)
- Database accepts without corruption
**Priority:** P2

### TC-005: Create customer — Special characters in name
**Given:** Clean database  
**When:** `create_customer(name="O'Reilly & Associates (Ltd.)")`  
**Then:**
- Command succeeds
- Name stored correctly (no SQL injection, no escaping issues)
- Retrieved name matches input exactly
**Priority:** P1

### TC-006: List customers — Empty database
**Given:** Database with no customers  
**When:** `list_customers(include_archived=false)`  
**Then:**
- Command returns empty array `[]`
- No error
**Priority:** P0

### TC-007: List customers — Multiple customers
**Given:** Database with 5 customers (2 archived, 3 active)  
**When:** `list_customers(include_archived=false)`  
**Then:**
- Returns only 3 active customers
- Results sorted alphabetically by name (or by created_at DESC)
**Priority:** P0

### TC-008: List customers — Include archived flag
**Given:** Database with 5 customers (2 archived, 3 active)  
**When:** `list_customers(include_archived=true)`  
**Then:**
- Returns all 5 customers
- Archived customers identifiable (have `archived_at` value)
**Priority:** P1

### TC-009: Update customer — Name change
**Given:** Customer "Old Name" exists  
**When:** `update_customer(id, name="New Name")`  
**Then:**
- Command succeeds
- `updated_at` is newer than before
- Other fields unchanged
**Priority:** P0

### TC-010: Update customer — Non-existent customer
**Given:** Clean database  
**When:** `update_customer(id="nonexistent-uuid")`  
**Then:**
- Command returns error: "Customer not found"
- No database changes
**Priority:** P0

### TC-011: Archive customer — Soft delete
**Given:** Customer "Acme" with 3 work orders exists  
**When:** `archive_customer(id)`  
**Then:**
- Command succeeds
- Customer `archived_at` is set to current time
- Customer no longer appears in `list_customers(include_archived=false)`
- Customer still appears in `list_customers(include_archived=true)`
- Associated work orders are NOT deleted (only customer archived)
**Priority:** P0

### TC-012: Archive customer — Already archived
**Given:** Customer already archived  
**When:** `archive_customer(id)`  
**Then:**
- Command succeeds (idempotent)
- `archived_at` remains unchanged (or updated to now)
**Priority:** P1

---

## 1.2 Work Order Management Commands

### TC-013: Create work order — Happy path
**Given:** Customer "Acme" exists  
**When:** `create_work_order(customer_id, name="Q2 Development", code="Q2-DEV", description="Q2 development work")`  
**Then:**
- Command returns new work order ID
- All fields stored correctly
- `status` defaults to 'active'
- `archived_at` is NULL
**Priority:** P0

### TC-014: Create work order — Required fields validation
**Given:** Customer exists  
**When:** `create_work_order(customer_id="", name="")`  
**Then:**
- Command returns error for missing customer_id and name
- No database change
**Priority:** P0

### TC-015: Create work order — Non-existent customer
**Given:** Database  
**When:** `create_work_order(customer_id="nonexistent-uuid", name="Work")`  
**Then:**
- Command returns error: "Customer not found" (or FK constraint violation)
- No work order created
**Priority:** P0

### TC-016: Create work order — Special characters in name/description
**Given:** Customer exists  
**When:** `create_work_order(customer_id, name="Q2's \"Tricky\" Work", description="Emoji test: 😀✅ 🚀")`  
**Then:**
- Command succeeds
- Special characters stored and retrieved without corruption
**Priority:** P1

### TC-017: List work orders — All customers
**Given:** 3 customers with 2, 1, 3 work orders (5 total, 1 archived)  
**When:** `list_work_orders(customer_id=None, include_archived=false)`  
**Then:**
- Returns 4 active work orders across all customers
- Sorted by customer, then by name or created_at
**Priority:** P0

### TC-018: List work orders — Filter by customer
**Given:** 3 customers with 2, 1, 3 work orders  
**When:** `list_work_orders(customer_id="acme-id", include_archived=false)`  
**Then:**
- Returns 2 work orders for Acme only
- Excludes other customers' work orders
**Priority:** P0

### TC-019: List work orders — Empty customer
**Given:** Customer with no work orders  
**When:** `list_work_orders(customer_id="id", include_archived=false)`  
**Then:**
- Returns empty array
- No error
**Priority:** P1

### TC-020: Update work order — Change name and status
**Given:** Work order exists with status='active'  
**When:** `update_work_order(id, name="New Name", status="closed")`  
**Then:**
- Command succeeds
- `name` and `status` updated
- `updated_at` is newer
- Other fields unchanged
**Priority:** P0

### TC-021: Update work order — Invalid status transition
**Given:** Work order with status='active'  
**When:** `update_work_order(id, status="invalid_status")`  
**Then:**
- Command returns error: "Invalid status value"
- Status unchanged
**Priority:** P1

### TC-022: Archive work order — Soft delete
**Given:** Work order with 5 sessions exists  
**When:** `archive_work_order(id)`  
**Then:**
- Command succeeds
- Work order `archived_at` is set
- Associated sessions are NOT deleted
- Work order no longer in `list_work_orders(include_archived=false)`
**Priority:** P0

### TC-023: Archive work order — Non-existent
**Given:** Database  
**When:** `archive_work_order(id="nonexistent")`  
**Then:**
- Command returns error: "Work order not found"
**Priority:** P1

---

## 1.3 Session Management Commands

### TC-024: Start session — Happy path
**Given:** Work order exists, no active session  
**When:** `start_session(work_order_id)`  
**Then:**
- Command returns session with ID, work_order_id, start_time
- `end_time` is NULL (session is active/incomplete)
- `active_session` singleton updated with session_id, work_order_id, started_at, last_heartbeat
- `recent_work_orders` entry created/updated for this work order
**Priority:** P0

### TC-025: Start session — Session created with correct timestamp
**Given:** Work order exists  
**When:** `start_session(work_order_id)` at known time T  
**Then:**
- Session `start_time` is ISO 8601 timestamp close to T (within 1 second)
**Priority:** P0

### TC-026: Start session — Non-existent work order
**Given:** Database  
**When:** `start_session(work_order_id="nonexistent")`  
**Then:**
- Command returns error: "Work order not found"
- No session created
- `active_session` unchanged
**Priority:** P0

### TC-027: Start session — Atomic switch (no overlapping sessions)
**Given:** Session A is active (no end_time)  
**When:** `start_session(work_order_id_B)` (different work order)  
**Then:**
- Command succeeds and returns session B
- Session A has end_time set to ~now
- Duration_seconds calculated for session A
- Session B is now active (end_time NULL)
- `active_session` now points to session B
- All updates in single transaction (all-or-nothing)
**Priority:** P0

### TC-028: Start session — Double-start protection
**Given:** Session A is active  
**When:** `start_session(work_order_id_A)` (same work order)  
**Then:**
- Command either:
  - a) Returns the existing active session (idempotent), OR
  - b) Stops A and starts new (creates new session record)
- Behavior documented and consistent
- No orphaned sessions
**Priority:** P1

### TC-029: Stop session — Happy path
**Given:** Session is active (end_time NULL)  
**When:** `stop_session(notes="Completed feature X", activity_type="development")`  
**Then:**
- Command succeeds
- Session end_time set to ~now
- duration_seconds calculated (end_time - start_time)
- notes and activity_type stored
- updated_at is current time
- `active_session` set to NULL (no active session)
**Priority:** P0

### TC-030: Stop session — Manual duration override
**Given:** Session is active  
**When:** `stop_session(notes="...", manual_duration_seconds=1800)`  
**Then:**
- Command succeeds
- duration_override set to 1800
- end_time set to now
- Effective duration is 1800 (override takes precedence)
**Priority:** P1

### TC-031: Stop session — No active session
**Given:** No active session (active_session.session_id is NULL)  
**When:** `stop_session()`  
**Then:**
- Command returns error: "No active session"
- Database unchanged
**Priority:** P1

### TC-032: Stop session — Session already stopped
**Given:** Session S with end_time already set  
**When:** `stop_session()` (try to stop again)  
**Then:**
- Command returns error: "Session already stopped" OR
- Command succeeds without changes (idempotent)
- Behavior documented
**Priority:** P1

### TC-033: Get active session — Session is running
**Given:** Session is active with work_order  
**When:** `get_active_session()`  
**Then:**
- Command returns session info
- Includes: id, work_order_id, start_time, elapsed_seconds (calculated from start_time to now)
- elapsed_seconds updates on each call (reflects actual elapsed time)
**Priority:** P0

### TC-034: Get active session — No active session
**Given:** No active session  
**When:** `get_active_session()`  
**Then:**
- Command returns null or empty result
- No error
**Priority:** P0

### TC-035: Update session — Edit notes
**Given:** Completed session S  
**When:** `update_session(id=S.id, notes="Updated notes")`  
**Then:**
- Command succeeds
- notes field updated
- updated_at is now
- Other fields unchanged
**Priority:** P0

### TC-036: Update session — Edit duration override
**Given:** Completed session with duration_seconds=3600 (1 hour calculated)  
**When:** `update_session(id, duration_override=5400)` (1.5 hours)  
**Then:**
- Command succeeds
- duration_override set to 5400
- Effective duration becomes 5400
- duration_seconds unchanged (original calculation preserved)
**Priority:** P0

### TC-037: Update session — Clear duration override
**Given:** Session with duration_override=5400  
**When:** `update_session(id, duration_override=null)`  
**Then:**
- Command succeeds
- duration_override cleared
- Effective duration reverts to duration_seconds (calculated)
**Priority:** P1

### TC-038: Update session — Non-existent session
**Given:** Database  
**When:** `update_session(id="nonexistent", notes="...")`  
**Then:**
- Command returns error: "Session not found"
**Priority:** P1

### TC-039: List sessions — Date range filter
**Given:** Sessions:
- 2026-04-10 09:00-10:00 (completed)
- 2026-04-11 08:00-12:00 (completed)
- 2026-04-11 14:00-17:00 (completed)
- 2026-04-12 09:00-??? (active, no end_time)

**When:** `list_sessions(start_date="2026-04-11", end_date="2026-04-11")`  
**Then:**
- Returns 2 sessions (both on 2026-04-11)
- Excludes 2026-04-10 and 2026-04-12 entries
- Date filtering uses start_time for comparison
**Priority:** P0

### TC-040: List sessions — Include incomplete (active) sessions
**Given:** Sessions as TC-039  
**When:** `list_sessions(start_date="2026-04-12", end_date="2026-04-12", include_incomplete=true)`  
**Then:**
- Returns active session from 2026-04-12
- Session has start_time but end_time=NULL
**Priority:** P1

### TC-041: List sessions — Exclude incomplete sessions
**Given:** Sessions as TC-039  
**When:** `list_sessions(start_date="2026-04-12", end_date="2026-04-12", include_incomplete=false)`  
**Then:**
- Returns empty array (active session excluded)
**Priority:** P1

### TC-042: List sessions — Sort order
**Given:** Multiple completed sessions on same day  
**When:** `list_sessions(start_date, end_date)`  
**Then:**
- Sessions sorted by start_time ascending (earliest first)
**Priority:** P1

### TC-043: Delete session — Removes session and updates aggregates
**Given:** Completed session S, summary cached  
**When:** `delete_session(id=S.id)`  
**Then:**
- Command succeeds
- Session removed from database
- Dependent aggregate data (daily summary, recents) updated
**Priority:** P1

### TC-044: Delete session — Non-existent session
**Given:** Database  
**When:** `delete_session(id="nonexistent")`  
**Then:**
- Command returns error: "Session not found"
**Priority:** P1

---

## 1.4 Quick-Add Command (Atomic Multi-Step Operation)

### TC-045: Quick-add — Create new customer + new work order + start session
**Given:** Clean database  
**When:** `quick_add(customer_name="GlobalTech", work_order_name="API Development", work_order_code="API-001")`  
**Then:**
- Command succeeds, returns { customer, work_order, session }
- New customer created with name="GlobalTech", archived_at=NULL
- New work order created under that customer, status='active'
- New session created and active (start_time set, end_time NULL)
- `active_session` now points to this session
- All 3 changes in single atomic transaction (all-or-nothing)
**Priority:** P0

### TC-046: Quick-add — Use existing customer + create new work order
**Given:** Customer "Acme" exists  
**When:** `quick_add(customer_id="acme-id", work_order_name="New Project", work_order_code="NEW-001")`  
**Then:**
- Command succeeds, returns { customer, work_order, session }
- No new customer created (existing Acme used)
- New work order created under Acme
- Session started for new work order
**Priority:** P0

### TC-047: Quick-add — Minimal parameters (work order name only)
**Given:** Database  
**When:** `quick_add(work_order_name="Quick Task")`  
**Then:**
- Command returns error: "customer_name or customer_id required"
- No changes (all-or-nothing transaction)
**Priority:** P1

### TC-048: Quick-add — Invalid customer ID
**Given:** Database  
**When:** `quick_add(customer_id="nonexistent-uuid", work_order_name="Task")`  
**Then:**
- Command returns error: "Customer not found"
- No work order or session created (transaction rolled back)
**Priority:** P0

### TC-049: Quick-add — Atomic failure (mid-transaction)
**Given:** Database with validation that fails on work order creation  
**When:** `quick_add(customer_name="X", work_order_name="")` (empty name)  
**Then:**
- Command returns error
- No customer created (transaction rolled back)
- Database remains clean
**Priority:** P0

---

## 1.5 Crash Recovery Commands

### TC-050: Recover session — Orphan detected on startup
**Given:**
- App was tracking session S on work order W
- App crashed without calling `stop_session()`
- Database has session S with start_time set but end_time=NULL

**When:** App starts and calls `check_for_orphan_sessions()`  
**Then:**
- Command detects orphan (end_time IS NULL)
- Returns OrphanSession with: id, start_time, work_order_name, customer_name, elapsed_seconds (now - start_time)
- Frontend presents recovery dialog
**Priority:** P0

### TC-051: Recover session — No orphan (clean shutdown)
**Given:** App properly stopped all sessions before last shutdown  
**When:** App starts and calls `check_for_orphan_sessions()`  
**Then:**
- Command returns null/None (no orphan)
- App proceeds normally, no recovery dialog
**Priority:** P0

### TC-052: Recover session — User chooses to close orphan
**Given:** Orphan session S detected, recovery dialog shown  
**When:** User clicks "Close now" → `recover_session(session_id=S.id)`  
**Then:**
- Command succeeds
- Session S gets end_time set to current time
- Duration calculated
- active_session set to NULL
- Session finalized with user's current time
**Priority:** P0

### TC-053: Recover session — User chooses to discard orphan
**Given:** Orphan session S detected  
**When:** User clicks "Discard" → `discard_orphan_session(session_id=S.id)`  
**Then:**
- Command succeeds
- Session S deleted from database (or soft-deleted)
- active_session set to NULL
- No time recorded for orphan
**Priority:** P1

### TC-054: Recover session — Multiple orphans (edge case)
**Given:** Database with 2 sessions both having end_time=NULL (corrupted state)  
**When:** `check_for_orphan_sessions()`  
**Then:**
- Returns all orphans or first one with flag indicating multiple
- Recovery handles all consistently
**Priority:** P2

---

## 1.6 Summary & Reporting Commands

### TC-055: Get daily summary — Empty day
**Given:** Date 2026-04-13 has no sessions  
**When:** `get_daily_summary(date="2026-04-13")`  
**Then:**
- Command returns { total_seconds: 0, by_customer: [], by_work_order: [] }
- No error
**Priority:** P0

### TC-055: Get daily summary — Single customer, single work order
**Given:**
- 2026-04-11: 1 session on "Acme/Project A" from 09:00 to 10:00 (3600 seconds = 1 hour)

**When:** `get_daily_summary(date="2026-04-11")`  
**Then:**
- Returns:
  ```json
  {
    "total_seconds": 3600,
    "by_customer": [
      { "customer_name": "Acme", "total_seconds": 3600, "work_orders": [...] }
    ]
  }
  ```
- Total is 1 hour
- Breakdown shows Acme with 1 hour
**Priority:** P0

### TC-056: Get daily summary — Multiple customers and work orders
**Given:**
- 2026-04-11 sessions:
  - Acme/ProjectA: 09:00-10:00 (1 hour)
  - Acme/ProjectB: 10:00-11:00 (1 hour)
  - GlobalTech/ProjectC: 14:00-16:00 (2 hours)

**When:** `get_daily_summary(date="2026-04-11")`  
**Then:**
- total_seconds = 14400 (4 hours)
- by_customer:
  - Acme: 7200 (2 hours), with 2 work orders
  - GlobalTech: 7200 (2 hours), with 1 work order
**Priority:** P0

### TC-057: Get daily summary — Includes incomplete (active) session
**Given:**
- 2026-04-11 sessions:
  - Completed: 09:00-10:00 (1 hour)
  - Active (started at 14:00, no end_time)

**When:** `get_daily_summary(date="2026-04-11", include_incomplete=true)` called at 14:30  
**Then:**
- Completed: 3600 seconds
- Active: calculated as (14:30 - 14:00) = 1800 seconds
- total_seconds = 5400 (1.5 hours)
**Priority:** P1

### TC-058: Get daily summary — Excludes active session
**Given:** Same as TC-057  
**When:** `get_daily_summary(date="2026-04-11", include_incomplete=false)`  
**Then:**
- Returns only completed session (3600 seconds)
- Active session ignored
**Priority:** P1

### TC-059: Get daily summary — Duration override respected
**Given:** Session with duration_seconds=3600 but duration_override=5400  
**When:** `get_daily_summary(date)`  
**Then:**
- Total includes 5400 (override value), not 3600 (calculated)
**Priority:** P0

### TC-060: Get recent work orders — Ordered by last_used_at
**Given:** recent_work_orders table:
- ProjectB: last_used_at=2026-04-11 14:00
- ProjectA: last_used_at=2026-04-11 10:00
- ProjectC: last_used_at=2026-04-11 12:00

**When:** `get_recent_work_orders(limit=10)`  
**Then:**
- Returns [ProjectB, ProjectC, ProjectA] (most recent first)
- Limit respected
**Priority:** P0

### TC-061: Get recent work orders — Limit parameter
**Given:** 10 recent work orders  
**When:** `get_recent_work_orders(limit=5)`  
**Then:**
- Returns top 5 by last_used_at descending
- 5 items exactly
**Priority:** P1

### TC-062: Get recent work orders — Empty recents
**Given:** recent_work_orders table is empty  
**When:** `get_recent_work_orders(limit=10)`  
**Then:**
- Returns empty array
- No error
**Priority:** P1

---

## 1.7 Export Command

### TC-063: Export CSV — Happy path
**Given:**
- 2026-04-10: Acme/ProjectA 09:00-10:00 (meeting)
- 2026-04-10: GlobalTech/ProjectB 14:00-16:00 (development)
- 2026-04-11: Acme/ProjectA 09:00-12:00 (development, 3 hours)

**When:** `export_csv(start_date="2026-04-10", end_date="2026-04-11")`  
**Then:**
- Returns CSV string with header: `Customer,Work Order,Activity Type,Start Time,End Time,Duration (hours),Notes`
- 3 data rows (one per session)
- Duration calculated or overridden correctly
- CSV is valid (parseable by standard CSV readers)
- All 3 sessions included
**Priority:** P0

### TC-064: Export CSV — Date filtering
**Given:** Sessions on 2026-04-10, 2026-04-11, 2026-04-12  
**When:** `export_csv(start_date="2026-04-11", end_date="2026-04-11")`  
**Then:**
- Returns 1 row (only 2026-04-11 sessions)
- 2026-04-10 and 2026-04-12 excluded
**Priority:** P0

### TC-065: Export CSV — Empty date range
**Given:** No sessions in date range 2026-04-20 to 2026-04-25  
**When:** `export_csv(start_date="2026-04-20", end_date="2026-04-25")`  
**Then:**
- Returns CSV with header only (no data rows)
- No error
**Priority:** P1

### TC-066: Export CSV — Special characters in fields
**Given:** Session with notes="Quote: \"important\", apostrophe: it's tricky, emoji: 🎉"  
**When:** `export_csv(start_date, end_date)` includes this session  
**Then:**
- CSV properly escapes special characters
- No parsing errors when re-imported
- All content preserved exactly
**Priority:** P1

---

# 2. DATA INTEGRITY & INVARIANT TESTS

These tests verify system-level invariants and crash resilience.

---

### TC-067: Invariant — No overlapping active sessions
**Setup:**
1. Create customer "Acme", work order "ProjectA"
2. Start session on ProjectA
3. Wait 2 seconds
4. Create work order "ProjectB" under Acme
5. Try to start session on ProjectB while ProjectA is still active

**Expected:**
- Session on ProjectA automatically stops
- Session on ProjectB starts
- Both sessions have valid start/end times
- No time lost
**Priority:** P0

### TC-068: Invariant — No orphaned sessions after atomic switch failure
**Setup:**
1. Start session on ProjectA (successful)
2. Database failure during start_session(ProjectB) midway through transaction
   (e.g., connection reset after ProjectA is stopped but before ProjectB session inserted)

**Expected:**
- Transaction rolls back completely
- ProjectA session remains active (with no end_time) OR is in a valid state
- Database is consistent (no partial updates)
- Application can recover from error
**Priority:** P0

### TC-069: Invariant — Cascade delete (customer → work orders → sessions)
**Setup:**
1. Create customer "OldCo"
2. Create 2 work orders under OldCo
3. Create 5 sessions under those work orders
4. Archive customer "OldCo"

**Expected:**
- Customer archived (soft delete, not hard delete)
- Associated work orders NOT deleted (remain in DB)
- Sessions NOT deleted
- Customer no longer appears in active lists
**Priority:** P0

### TC-070: WAL mode verification — Writes persist on crash
**Setup:**
1. Start application
2. Create customer, work order, start session
3. Immediately kill app process (simulated crash via test harness or task manager)
4. Restart application
5. Query database

**Expected:**
- All 3 entities exist in database
- No data loss
- active_session.session_id set to the created session (orphan detection)
- App presents recovery dialog
**Priority:** P0

### TC-071: SQLite integrity — Database remains valid after unexpected shutdown
**Setup:**
1. Multiple rapid operations (create customer, work order, start session, stop session)
2. Crash during I/O
3. Restart app and run `PRAGMA integrity_check`

**Expected:**
- `PRAGMA integrity_check` returns "ok" (no corruption)
- Database is readable
- No orphaned pointers or foreign key violations
**Priority:** P0

### TC-072: Transaction rollback — Partial operation failure
**Setup:**
1. Attempt quick_add with:
   - Valid customer_name
   - Valid work_order_name
   - Invalid customer_id (will fail on FK check)

**Expected:**
- Entire operation rolls back
- No customer created
- No work order created
- No session created
- Database state unchanged
**Priority:** P0

### TC-073: Rapid context switching — No lost sessions
**Setup:**
1. Create 5 different work orders
2. Switch between them rapidly (5 switches in 10 seconds)
3. Each switch: start session on new work order

**Expected:**
- All 5 sessions created with correct work_order_ids
- Each session has valid start_time and end_time (except last one, which is active)
- No duplicate sessions
- No lost time
- Durations for stopped sessions calculated correctly
**Priority:** P1

### TC-074: Date boundary — Session spanning midnight
**Setup:**
1. Create session starting at 23:55 on 2026-04-11
2. Stop session at 00:05 on 2026-04-12
3. Query daily summary for both dates

**Expected:**
- `get_daily_summary(date="2026-04-11")` includes session (start_date filtered by start_time)
- `get_daily_summary(date="2026-04-12")` does NOT include session (ends after midnight but started previous day)
- Session's duration correctly calculated (10 minutes)
- No duplicate counting across days
**Priority:** P1

---

# 3. FRONTEND INTEGRATION TESTS (Svelte Component Tests)

These tests verify UI components and user interactions with the backend. Run with:
```bash
npm run test
```

Or use Svelte Testing Library + Tauri mock.

---

### TC-075: Timer component — Real-time update every second
**Setup:**
1. Start active session at known time T
2. Render Timer component
3. Wait 5 seconds

**Expected:**
- Timer displays "00:00" at T+0
- Timer displays "00:01" at T+1
- Timer displays "00:02" at T+2
- Timer displays "00:05" at T+5
- Updates smooth (no jitter, no skipped seconds)
**Priority:** P0

### TC-076: Timer component — Stop button triggers stop_session
**Setup:**
1. Active session running
2. Timer component rendered with "Stop" button

**When:** User clicks "Stop"  
**Then:**
- Modal or form appears for notes/activity type (optional)
- User can skip or fill in
- `stop_session()` command invoked
- UI shows "Not tracking" state
- Timer stops updating
**Priority:** P0

### TC-077: Timer component — Inactive state (no active session)
**Setup:**
1. No active session

**When:** Timer component rendered  
**Then:**
- Displays "00:00" or "Not tracking"
- No update loop running
- No CPU usage
- Color indicates inactive state (grey)
**Priority:** P1

### TC-078: Quick-switch interface — Recent items display
**Setup:**
1. 5 work orders with recent history
2. App renders quick-switch component

**When:** Component loads  
**Then:**
- Shows top 5 recent work orders (or as many as configured)
- Each item shows: customer name + work order name
- Ordered by last_used_at descending (most recent first)
**Priority:** P0

### TC-079: Quick-switch interface — Search filter
**Setup:**
1. 10 work orders with various names
2. Quick-switch component with search input

**When:** User types "proj"  
**Then:**
- List filtered to show only items matching "proj" (case-insensitive)
- Updates within 50ms (performance target)
- "No results" message if no matches
**Priority:** P0

### TC-080: Quick-switch interface — Keyboard navigation
**Setup:**
1. Quick-switch component showing results

**When:**
1. User presses Down arrow key
2. User presses Down again
3. User presses Up arrow
4. User presses Enter

**Then:**
- First Down: Selection moves to result #1 (highlighted)
- Second Down: Selection moves to result #2
- Up arrow: Selection moves to result #1
- Enter: `start_session()` called for selected work order
- UI switches to timer for new work order
**Priority:** P0

### TC-081: Quick-switch interface — Global hotkey trigger
**Setup:**
1. App running with quick-switch component hidden

**When:** User presses Ctrl+K (or Cmd+K on macOS)  
**Then:**
- Quick-switch overlay appears
- Search input focused
- User can immediately type to filter
**Priority:** P1

### TC-082: Quick-add overlay — Keyboard shortcut opens
**Setup:**
1. App running, not in quick-add

**When:** User presses Ctrl+N (or Cmd+N on macOS)  
**Then:**
- Quick-add overlay appears (centered, modal)
- First input field (customer name or customer picker) focused
- Backdrop dims main app
**Priority:** P0

### TC-083: Quick-add overlay — Create new customer
**Setup:**
1. Quick-add overlay open
2. No customer selected

**When:**
1. User types "New Customer" in customer field
2. User tabs to work order name field
3. User types "New Project"
4. User presses Enter

**Then:**
- `quick_add(customer_name="New Customer", work_order_name="New Project")` called
- Overlay closes
- Timer starts on new project
- "New Customer" / "New Project" displayed
**Priority:** P0

### TC-084: Quick-add overlay — Use existing customer
**Setup:**
1. Quick-add overlay open
2. Database has customers: "Acme", "GlobalTech"

**When:**
1. User clicks customer dropdown
2. Selects "Acme"
3. Types "Website Redesign" in work order name
4. Presses Enter

**Then:**
- `quick_add(customer_id=acme_id, work_order_name="Website Redesign")` called
- Overlay closes
- Timer starts on "Acme / Website Redesign"
**Priority:** P0

### TC-085: Quick-add overlay — Escape closes without action
**Setup:**
1. Quick-add overlay open with partial input

**When:** User presses Escape  
**Then:**
- Overlay closes
- No command sent
- App state unchanged
**Priority:** P1

### TC-086: Today's work log — Displays all completed sessions for today
**Setup:**
1. Today (2026-04-11) has 4 completed sessions:
   - 09:00-10:00 Acme/ProjectA (1 hour)
   - 10:00-11:30 Acme/ProjectB (1.5 hours)
   - 14:00-16:00 GlobalTech/ProjectC (2 hours)
   - 16:30-17:00 Acme/ProjectA (0.5 hours)

**When:** Today's work log component rendered  
**Then:**
- Shows all 4 rows
- Each row displays: customer (color-coded), work order name, duration, start/end times
- Sorted chronologically (earliest first)
- Optional: notes/activity type icons
**Priority:** P0

### TC-087: Today's work log — Inline edit session
**Setup:**
1. Session displayed in log

**When:** User clicks on session row  
**Then:**
- Row expands to show editable fields
- Fields: duration (with override toggle), notes, activity_type
- User can modify any field
- User clicks "Save" or presses Ctrl+Enter
**Then:**
- `update_session()` called
- Row collapses and reflects new data
- No page reload
**Priority:** P0

### TC-088: Today's work log — Delete session
**Setup:**
1. Session displayed in log

**When:** User clicks delete/trash icon  
**Then:**
- Confirmation dialog: "Delete this session? Time will be lost."
- User confirms
- `delete_session()` called
- Session removed from list
- Daily summary updated
**Priority:** P1

### TC-089: Today's work log — Empty day
**Setup:**
1. Today has no completed sessions

**When:** Work log component rendered  
**Then:**
- Displays "No sessions yet" or empty message
- No error
**Priority:** P1

### TC-090: Daily summary panel — Correct totals by customer
**Setup:**
1. Today: 4 sessions as TC-086
   - Acme: 2 hours total (1 + 1.5 = 2.5 hours, typo in TC should be 1 + 0.5 + 1 = 2.5 hours... let's correct: ProjectA(1h) + ProjectB(1.5h) + ProjectA(0.5h) = 2.5 hours)
   - GlobalTech: 2 hours

**When:** Daily summary component rendered  
**Then:**
- Shows:
  - Total: 4.5 hours (2.5 + 2)
  - Acme: 2.5 hours
  - GlobalTech: 2 hours
- Format: "Acme — 2h 30m" (or decimal 2.5h)
**Priority:** P0

### TC-091: Daily summary panel — Real-time update (active session)
**Setup:**
1. Daily summary shows 3 hours completed + 0 hours active
2. User starts new session

**When:** Timer counts up and new session is active  
**Then:**
- Daily summary total updates in real-time (3.0 → 3.1 → 3.2 as seconds elapsed)
- Category for that work order updates
- Smooth update (no lag)
**Priority:** P0

### TC-092: Daily summary panel — Update on stop
**Setup:**
1. Daily summary shows 3 hours
2. Session is active (not counted in summary)

**When:** User stops session and notifies UI  
**Then:**
- Summary updates to include the stopped session
- Total reflects new duration
- Activity icon/type displays
**Priority:** P1

### TC-093: Recovery dialog — Orphan detected on startup
**Setup:**
1. App launch with orphan session in database

**When:** App initializes  
**Then:**
- Recovery dialog appears (modal, blocks interaction)
- Shows: "Open session from [time]. Close it or discard?"
- Displays customer and work order names
- Elapsed time since session started
- Two buttons: "Close Now" and "Discard"
**Priority:** P0

### TC-094: Recovery dialog — User chooses "Close Now"
**Setup:**
1. Recovery dialog showing orphan

**When:** User clicks "Close Now"  
**Then:**
- Dialog closes
- `recover_session(session_id)` called
- Session finalized with current time
- App proceeds normally
- No active session (timer at "00:00")
**Priority:** P0

### TC-095: Recovery dialog — User chooses "Discard"
**Setup:**
1. Recovery dialog showing orphan

**When:** User clicks "Discard"  
**Then:**
- Dialog closes
- `discard_orphan_session(session_id)` called
- Orphan removed
- App proceeds normally
- No active session
**Priority:** P0

### TC-096: CSV export — Save file dialog
**Setup:**
1. App showing export form (date range selector)

**When:** User selects date range and clicks "Export to CSV"  
**Then:**
- `export_csv()` command invoked
- File save dialog appears (native file picker)
- Default filename: "work-tracker-2026-04-11.csv" (today's date)
- User selects location and confirms
- CSV file saved
**Priority:** P0

### TC-097: CSV export — Verify CSV content
**Setup:**
1. CSV exported from TC-096

**When:** User opens CSV in spreadsheet app  
**Then:**
- Header row present: Customer,Work Order,Activity Type,Start Time,End Time,Duration (hours),Notes
- Data rows match sessions from selected date range
- No parsing errors
- All special characters preserved
**Priority:** P0

---

# 4. PERFORMANCE TEST CASES

Verify system meets performance targets defined in architecture.md §7.

---

### TC-098: Timer update latency — <100ms per update
**Measurement:** 
1. Start active session
2. Record timestamp when Timer component receives elapsed_seconds update
3. Measure time from actual elapsed second to UI refresh
4. Repeat 10 times, calculate average

**Target:** <100ms average, no single update >200ms  
**Tool:** Browser DevTools Performance panel or Tauri profiler  
**Priority:** P0

### TC-099: Context switch latency — <3s end-to-end
**Measurement:**
1. Start session on ProjectA
2. Record time T1 (when context switch initiated)
3. Trigger start_session(ProjectB)
4. Record time T2 (when timer shows ProjectB name + starts counting)
5. T2 - T1 = context switch time

**Target:** <3 seconds  
**Repeat:** 5 times, average  
**Priority:** P0

### TC-100: Search filter latency — <50ms
**Measurement:**
1. Open quick-switch with 50 work orders
2. Record T1 (keystroke "p")
3. Record T2 (when results filtered to show matches)
4. T2 - T1 = latency

**Target:** <50ms  
**Repeat:** 10 times, keystroke → result  
**Tool:** Browser DevTools or profiler  
**Priority:** P1

### TC-101: Session create latency — <100ms
**Measurement:**
1. Call `create_work_order()` and measure time to response
2. Repeat 20 times with different inputs

**Target:** <100ms average  
**Tool:** Tauri debug console or profiler  
**Priority:** P1

### TC-102: Daily summary query latency — <100ms
**Measurement:**
1. Database has 50+ sessions
2. Call `get_daily_summary(date)`
3. Measure query + serialization time

**Target:** <100ms  
**Repeat:** 10 times  
**Tool:** SQLite EXPLAIN QUERY PLAN, profiler  
**Priority:** P1

### TC-103: CSV export latency — <1s for 1-month export
**Measurement:**
1. 30 days of data, ~200 sessions
2. Call `export_csv(start_date, end_date)`
3. Measure time to CSV string generated

**Target:** <1 second  
**Tool:** Profiler  
**Priority:** P1

### TC-104: App cold start — <2s to usable UI
**Measurement:**
1. Kill app
2. Start app (measure from executable launch)
3. Measure time to: UI visible, timer component responsive

**Target:** <2 seconds on typical developer machine  
**Priority:** P1

---

# 5. EDGE CASES & BOUNDARY CONDITIONS

### TC-105: Empty customer name
**Given:** Attempt to create customer with empty string or whitespace only  
**When:** `create_customer(name="")`  
**Then:** Error: "Customer name is required"  
**Priority:** P0

### TC-106: Very long customer name (1000+ chars)
**Given:** Create customer with very long name  
**When:** `create_customer(name="A" * 1000)`  
**Then:** Either succeeds and stores full name, or truncates gracefully (document behavior)  
**Priority:** P2

### TC-107: Unicode characters in names
**Given:** Create customer with unicode: "北京 (Beijing)"  
**When:** `create_customer(name="北京 (Beijing)")`  
**Then:** Succeeds, stored and retrieved correctly  
**Priority:** P1

### TC-108: Emoji in notes
**Given:** Update session with emoji in notes: "Completed 🎉 Feature X"  
**When:** `update_session(id, notes="Completed 🎉 Feature X")`  
**Then:** Succeeds, emoji stored and CSV export preserves emoji  
**Priority:** P1

### TC-109: SQL injection attempt in name
**Given:** Create customer with malicious string: `name="'; DROP TABLE customers; --"`  
**When:** `create_customer(name="'; DROP TABLE customers; --")`  
**Then:**
- Command succeeds (string treated as literal)
- Table NOT dropped (parameterized queries prevent injection)
- Name stored as-is (with quotes and semicolons)
**Priority:** P0

### TC-110: Apostrophe in customer name
**Given:** Create customer "O'Reilly & Associates"  
**When:** `create_customer(name="O'Reilly & Associates")`  
**Then:** Succeeds, quotes and ampersands stored correctly, no escaping issues  
**Priority:** P1

### TC-111: Work order with no sessions
**Given:** Work order created but never used (no sessions)  
**When:** List work orders, query summaries  
**Then:** Work order appears in lists but contributes 0 hours to summary  
**Priority:** P1

### TC-112: Customer with no work orders
**Given:** Customer created but no work orders added  
**When:** `list_work_orders(customer_id)`  
**Then:** Returns empty array, no error  
**Priority:** P1

### TC-113: Zero-duration session
**Given:** Session starts and ends at exact same timestamp (or within 1 second)  
**When:** `stop_session()`  
**Then:** duration_seconds = 0, not negative, displayed as "0h 0m" or omitted  
**Priority:** P1

### TC-114: Manual duration override = 0
**Given:** Session with calculated duration = 3600 (1 hour)  
**When:** `update_session(id, duration_override=0)`  
**Then:** Effective duration = 0, stored correctly, not reverted to calculated  
**Priority:** P2

### TC-115: Negative manual override (invalid)
**Given:** Session with positive calculated duration  
**When:** `update_session(id, duration_override=-3600)`  
**Then:** Error: "Duration cannot be negative" or silently rejected (document)  
**Priority:** P1

### TC-116: Session with extreme duration (24+ hours)
**Given:** Session starts at 09:00 on day 1, ends at 10:00 on day 2  
**When:** `stop_session()`  
**Then:**
- duration_seconds calculated correctly (>86400)
- Displayed as "25 hours" or similar (not wrapped to 1 hour)
- Summary includes full duration
**Priority:** P1

### TC-117: DST boundary — Session spanning timezone change
**Given:** Session starting at 01:59 and ending at 03:01 (during spring DST change +1 hour)  
**When:** Calculate duration and summary  
**Then:**
- Duration reflects actual wall-clock time (60 minutes, not 2 hours or 0 hours)
- Timestamps handled correctly (document timezone assumption: UTC or local)
**Priority:** P2

### TC-118: All work sessions deleted for a day
**Given:** 5 sessions on 2026-04-11, all deleted  
**When:** `get_daily_summary(date="2026-04-11")`  
**Then:** Returns empty summary (total=0, by_customer=[], no error)  
**Priority:** P1

---

# 6. TEST EXECUTION & REPORTING

## Running Tests

### Rust Backend Tests
```bash
cd src-tauri
cargo test --lib           # Unit tests
cargo test --test '*'      # Integration tests
cargo test --doc           # Doc tests
```

### Svelte Frontend Tests
```bash
npm run test               # Vitest + Testing Library
npm run test:unit         # Unit tests only
npm run test:integration  # Integration tests (with Tauri mock)
```

### Manual Testing Checklist
- [ ] Create new customer (manual form)
- [ ] Create work order under customer
- [ ] Start session (timer should count)
- [ ] Stop session with notes
- [ ] View today's summary
- [ ] Switch between 3 work orders rapidly (verify no loss)
- [ ] Close app during active session, restart (verify recovery)
- [ ] Edit a past session's notes
- [ ] Delete a session
- [ ] Export CSV for past week
- [ ] Verify CSV opens in Excel/Google Sheets without errors

## Test Report Template

```
TEST RUN: Phase 1 — [Date] — [Tester]

BACKEND TESTS:
- Total: 65 tests
- Passed: XX
- Failed: XX
- Skipped: XX
- Coverage: XX%

FRONTEND TESTS:
- Total: 23 tests
- Passed: XX
- Failed: XX
- Skipped: XX

PERFORMANCE TESTS:
- Context Switch: XX ms (target <3000ms) ✓/✗
- Timer Update: XX ms (target <100ms) ✓/✗
- Search Filter: XX ms (target <50ms) ✓/✗
- Daily Summary Query: XX ms (target <100ms) ✓/✗

CRITICAL ISSUES:
- [List any P0 failures]

KNOWN LIMITATIONS:
- [List any deferred P1/P2 issues]

NOTES:
- [Any observations, edge cases discovered, etc.]
```

---

# 7. REGRESSION TEST SUITE (To Be Automated)

Once Phase 1 dev is complete, convert key tests to automated CI/CD:

### Must-Automate (P0)
- TC-024: Start session happy path
- TC-027: Atomic switch (no overlaps)
- TC-029: Stop session
- TC-055: Get daily summary (correct totals)
- TC-067: Invariant — no overlapping sessions
- TC-070: WAL mode verification (crash recovery)
- TC-098: Timer latency <100ms

### Should-Automate (P1)
- TC-039: List sessions date filtering
- TC-056: Daily summary (multiple customers)
- TC-075: Timer real-time updates
- TC-090: Daily summary panel totals

### Nice-to-Automate (P2)
- TC-005: Special characters in names
- TC-107: Unicode handling
- TC-117: DST boundaries

---

# 8. KNOWN LIMITATIONS & OPEN QUESTIONS

- **Timezone handling**: Document whether app uses UTC or local time (affects DST edge case TC-117)
- **CSV delimiter**: Confirm comma vs semicolon for non-US locales
- **Pause state**: Deferred to Phase 2 (not in MVP, but referenced in some test notes)
- **Favorites/pinning**: Phase 2+ feature (quick-switch shows recents only in Phase 1)
- **Color coding**: Phase 2+ feature (test plan assumes colors may be stored but not critical)

---

*End of Test Plan*
