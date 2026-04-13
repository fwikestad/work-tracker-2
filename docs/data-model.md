# Data Model & Database Schema

This document describes the database structure that powers Work Tracker 2. All data is stored in SQLite and persisted locally on the user's machine.

---

## Overview

The database uses a relational schema with four main tables:

1. **customers** — Tracked entities (companies, clients, departments)
2. **work_orders** — Discrete work units under a customer
3. **time_sessions** — Individual tracking sessions (start → stop)
4. **active_session** — Singleton tracking the currently active session (for crash recovery)
5. **recent_work_orders** — Metadata for quick-switch optimization

**Phase 2+ additions**:
- Paused sessions (pause state tracking)
- Favorites (pinned work orders)

---

## Core Tables

### Customers

Represents a tracked entity (e.g., "Acme Corp", "Internal", "Open Source").

```sql
CREATE TABLE customers (
    id TEXT PRIMARY KEY NOT NULL,           -- UUID or unique string
    name TEXT NOT NULL,                     -- Display name ("Acme Corp")
    code TEXT,                              -- Optional code or abbreviation
    color TEXT,                             -- Optional hex color (#FF5733)
    created_at TEXT NOT NULL,               -- ISO 8601 timestamp
    updated_at TEXT NOT NULL,               -- ISO 8601 timestamp
    archived_at TEXT                        -- NULL = active, timestamp = archived
);
```

**Indexes**:
- `idx_customers_name` — Fast lookups by name (search, autocomplete)
- `idx_customers_code` — Fast lookups by code
- `idx_customers_archived` — Filter archived vs. active customers

**Constraints**:
- `id`: Unique, never null
- `name`: Required
- `archived_at`: NULL (active) or ISO 8601 timestamp (archived); soft delete pattern

**Lifecycle**:
- Create: User adds a new customer via UI or quick-add
- Update: User edits name, code, or color
- Archive: User hides customer without deleting sessions
- Delete: Hard delete removes customer and all associated work orders/sessions (cascade)

---

### Work Orders

Represents a unit of work under a customer (e.g., "Web Redesign Project", "Code Review").

```sql
CREATE TABLE work_orders (
    id TEXT PRIMARY KEY NOT NULL,           -- UUID or unique string
    customer_id TEXT NOT NULL,              -- Foreign key → customers(id)
    name TEXT NOT NULL,                     -- Display name ("Web Redesign")
    code TEXT,                              -- Optional project code ("PRJ-123")
    description TEXT,                       -- Optional longer description
    status TEXT NOT NULL DEFAULT 'active',  -- 'active', 'paused', or 'closed'
    created_at TEXT NOT NULL,               -- ISO 8601 timestamp
    updated_at TEXT NOT NULL,               -- ISO 8601 timestamp
    archived_at TEXT,                       -- NULL = active, timestamp = archived
    is_favorite INTEGER DEFAULT 0           -- Phase 2: 1 = favorite, 0 = not
);
```

**Indexes**:
- `idx_work_orders_customer_id` — Fast lookup by customer (hierarchical queries)
- `idx_work_orders_status` — Filter by status (active/paused/closed)
- `idx_work_orders_archived` — Filter archived vs. active
- `idx_work_orders_favorite` — Phase 2: quick-access favorites

**Constraints**:
- `customer_id`: Required, foreign key with cascade delete
- `name`: Required
- `status`: Must be 'active', 'paused', or 'closed'
- `archived_at`: Soft delete (same pattern as customers)
- `is_favorite`: 0 or 1 (boolean)

**Foreign Key Relationship**:
```
customers (1) ──has many──> work_orders (N)
  ON DELETE CASCADE
```

Deleting a customer removes all associated work orders and their sessions.

---

### Time Sessions

The core tracking entity: a single period of work on a work order.

```sql
CREATE TABLE time_sessions (
    id TEXT PRIMARY KEY NOT NULL,               -- UUID or unique string
    work_order_id TEXT NOT NULL,                -- Foreign key → work_orders(id)
    start_time TEXT NOT NULL,                   -- ISO 8601 start timestamp
    end_time TEXT,                              -- ISO 8601 end timestamp (NULL if running)
    duration_seconds INTEGER,                   -- Calculated: (end_time - start_time)
    duration_override INTEGER,                  -- Manual override (in seconds) if user provided
    activity_type TEXT,                         -- Optional: "development", "meeting", "design", etc.
    notes TEXT,                                 -- Optional: user notes
    created_at TEXT NOT NULL,                   -- ISO 8601: when entry was created
    updated_at TEXT NOT NULL,                   -- ISO 8601: when entry was last modified
    paused_at TEXT,                             -- Phase 2: timestamp when paused
    total_paused_seconds INTEGER DEFAULT 0      -- Phase 2: cumulative pause duration
);
```

**Indexes**:
- `idx_sessions_start_time` — Fast filtering by start date
- `idx_sessions_end_time` — Find open sessions (end_time IS NULL) and completed sessions
- `idx_sessions_work_order_id` — Queries by work order (hierarchical)
- `idx_sessions_date_range` — Composite index for date range queries (Phase 2+)

**Constraints**:
- `work_order_id`: Required, foreign key with cascade delete
- `start_time`: Required, ISO 8601
- `end_time`: NULL (session running) or ISO 8601 (session stopped)
- `duration_seconds`: Calculated duration in seconds (or NULL if running)
- `duration_override`: User-provided override (takes precedence over calculated)

**Foreign Key Relationship**:
```
work_orders (1) ──has many──> time_sessions (N)
  ON DELETE CASCADE
```

Deleting a work order removes all its sessions.

**Session States**:

| State | end_time | paused_at | Meaning |
|-------|----------|-----------|---------|
| Running | NULL | NULL | Session active, timer running (Phase 1+) |
| Paused | NULL | timestamp | Timer paused but session not closed (Phase 2+) |
| Stopped | timestamp | NULL | Session closed, duration final |

**Duration Calculation**:

1. **Automatic** (default):
   - `duration_seconds = (end_time - start_time)` in seconds
   - Stored on session stop

2. **Manual Override** (user-provided):
   - `duration_override` is set by user after session creation
   - Takes precedence in UI display: `duration_override ?? duration_seconds`

---

### Active Session (Singleton)

Tracks the currently active session for quick resume and crash recovery.

```sql
CREATE TABLE active_session (
    id INTEGER PRIMARY KEY CHECK (id = 1),      -- Always 1 (singleton)
    session_id TEXT,                            -- Foreign key → time_sessions(id), NULL if no active
    work_order_id TEXT,                         -- Foreign key → work_orders(id) (denormalized for speed)
    started_at TEXT,                            -- ISO 8601 timestamp (informational)
    last_heartbeat TEXT,                        -- Last UI heartbeat (Phase 2+, informational)
    is_paused INTEGER DEFAULT 0,                -- Phase 2: 1 = paused, 0 = running
    paused_session_at TEXT                      -- Phase 2: when paused
);
```

**Usage**:
- On app startup: Query this row to find incomplete sessions
- On session start: Update `session_id` and `work_order_id`
- On session stop: Clear `session_id` and `work_order_id`
- On crash: App recovers by checking `session_id` on restart

**Initial State** (created during migration):
```sql
INSERT INTO active_session (id, session_id, work_order_id, started_at, last_heartbeat)
VALUES (1, NULL, NULL, NULL, NULL);
```

**Constraints**:
- `id`: Always 1 (singleton pattern)
- `session_id`: Foreign key, can be NULL
- `work_order_id`: Foreign key, can be NULL (denormalized for UI perf)

**Foreign Key Relationships**:
```
time_sessions (1) ←─ active_session (singleton)
work_orders (1) ←─ active_session (singleton)
  ON DELETE SET NULL (if referenced item deleted, clear the reference)
```

---

### Recent Work Orders

Optimization table for quick-switch feature: tracks usage frequency and recency.

```sql
CREATE TABLE recent_work_orders (
    work_order_id TEXT PRIMARY KEY NOT NULL,    -- Foreign key → work_orders(id)
    last_used_at TEXT NOT NULL,                 -- ISO 8601 timestamp
    use_count INTEGER NOT NULL DEFAULT 1        -- How many times used (for ranking)
);
```

**Indexes**:
- `idx_recent_last_used` — Fast ordering by most recent first

**Foreign Key Relationship**:
```
work_orders (1) ──referenced by──> recent_work_orders (N)
  ON DELETE CASCADE
```

---

## Entity Relationships (ER Diagram)

```
┌─────────────┐
│  customers  │
├─────────────┤
│ id (PK)     │
│ name        │
│ code        │
│ archived_at │
└──────┬──────┘
       │ 1:N
       │ (ON DELETE CASCADE)
       │
┌──────▼──────────────┐
│   work_orders       │
├─────────────────────┤
│ id (PK)             │
│ customer_id (FK)    │◄──────┐
│ name                │       │
│ status              │       │ 1:1
│ is_favorite         │       │ (Phase 2)
│ archived_at         │       │
└──────┬──────────────┘       │
       │ 1:N            ┌─────┴──────────┐
       │ (CASCADE)      │ active_session │
       │                ├────────────────┤
┌──────▼─────────────────┐   │ id = 1 (singleton)
│   time_sessions        │   │ session_id (FK) ►──┐
├────────────────────────┤   │ work_order_id (FK)─┘
│ id (PK)                │   │ is_paused (Phase 2)
│ work_order_id (FK)  ───┼──►│ paused_session_at
│ start_time             │   │
│ end_time               │   └────────────────────┘
│ duration_seconds       │
│ duration_override      │
│ activity_type          │
│ notes                  │
│ paused_at (Phase 2)    │
│ total_paused_seconds   │
└────┬───────────────────┘
     │ N:N (uses)
     │
┌────▼─────────────────┐
│ recent_work_orders   │
├──────────────────────┤
│ work_order_id (FK/PK)├──►(work_orders)
│ last_used_at         │
│ use_count            │
└──────────────────────┘
```

---

## Crash Recovery

When the app starts, it checks for incomplete sessions:

1. **Query for open session**:
   ```sql
   SELECT session_id FROM active_session WHERE id = 1;
   ```

2. **If session_id is NULL**: Normal startup; no recovery needed

3. **If session_id is not NULL**:
   - Fetch the incomplete session:
     ```sql
     SELECT * FROM time_sessions WHERE id = ? AND end_time IS NULL;
     ```
   - Present recovery UI: "You have an open session from [timestamp]. Close it now or discard?"
   - User chooses:
     - **Close**: Set `end_time` to current time, calculate duration, clear `active_session`
     - **Discard**: Delete the orphan session, clear `active_session`
   - Recovery completes before normal app flow starts

4. **Recovery guarantees**:
   - No lost data: Either the session is recovered and closed properly, or explicitly discarded
   - Atomicity: Close/discard operations are single transactions

---

## Durability & Write Safety

### SQLite Configuration

The database is initialized with crash-safe pragmas:

```sql
PRAGMA journal_mode = WAL;       -- Write-Ahead Logging
PRAGMA synchronous = NORMAL;     -- Minimum for durability
PRAGMA foreign_keys = ON;        -- Enforce relationships
PRAGMA busy_timeout = 5000;      -- Wait 5s if database locked
```

**WAL Mode**: Writes are first written to a separate log file, then committed in batches. If the app crashes mid-write:
- Committed changes are safe (in the main DB)
- Uncommitted changes are rolled back (in the WAL log)
- On next startup, SQLite automatically recovers

**Synchronous NORMAL**: After each transaction commits, the filesystem is flushed to ensure the OS writes to disk. This is safe for most applications and faster than FULL mode.

### Write Policy

Every INSERT/UPDATE to `time_sessions` is immediately persisted:
- No batching or deferred writes
- Each operation is a synchronous transaction
- Frontend waits for the response before updating UI
- Result: No "save" button; all changes are durable as soon as the operation completes

---

## Query Patterns

### Daily Summary

Retrieve all completed sessions for a given date, grouped by customer:

```sql
SELECT
    c.id, c.name,
    SUM(COALESCE(s.duration_override, s.duration_seconds)) as total_seconds
FROM time_sessions s
JOIN work_orders w ON s.work_order_id = w.id
JOIN customers c ON w.customer_id = c.id
WHERE DATE(s.start_time) = DATE('now')
    AND s.end_time IS NOT NULL
    AND c.archived_at IS NULL
    AND w.archived_at IS NULL
GROUP BY c.id
ORDER BY total_seconds DESC;
```

**Performance**: < 100ms for typical dataset (50-100 entries)

### Weekly Summary

```sql
SELECT
    c.id, c.name,
    SUM(COALESCE(s.duration_override, s.duration_seconds)) as total_seconds
FROM time_sessions s
JOIN work_orders w ON s.work_order_id = w.id
JOIN customers c ON w.customer_id = c.id
WHERE strftime('%Y-%W', s.start_time) = strftime('%Y-%W', 'now')
    AND s.end_time IS NOT NULL
    AND c.archived_at IS NULL
GROUP BY c.id
ORDER BY total_seconds DESC;
```

**Performance**: < 500ms

### Quick-Switch (Recent Items)

```sql
SELECT w.id, w.name, c.name as customer_name, rwo.last_used_at
FROM recent_work_orders rwo
JOIN work_orders w ON rwo.work_order_id = w.id
JOIN customers c ON w.customer_id = c.id
WHERE w.archived_at IS NULL
ORDER BY rwo.last_used_at DESC
LIMIT 10;
```

**Performance**: < 50ms

### Search Customers/Work Orders

```sql
SELECT id, name FROM customers
WHERE name LIKE '%query%'
    AND archived_at IS NULL
ORDER BY name
LIMIT 20;
```

**Performance**: < 50ms (with `idx_customers_name` index)

---

## Data Validation Rules

### Constraints Enforced by Database

- Foreign key constraints: Child records cannot exist without parent
- Check constraints: `active_session.id = 1` (singleton)
- NOT NULL constraints: Required fields cannot be null
- Unique primary keys: `id` fields cannot be duplicated

### Business Rules Enforced by Application

1. **No overlapping active sessions**: Only one session with `end_time IS NULL` at a time
   - Check before creating new session
   - Stop previous session atomically when switching work orders

2. **Session duration consistency**: `end_time >= start_time`
   - Validated when stopping session
   - User cannot set an end time before start

3. **No orphaned data**: Cascade deletes ensure no dangling foreign keys
   - Delete customer → removes all work orders and sessions
   - Delete work order → removes all sessions

4. **Activity type whitelist** (optional): 'development', 'meeting', 'design', 'admin', or NULL
   - Enforced by frontend validation (not database constraint)

---

## Indexes and Query Performance

### Why These Indexes Exist

| Index | Reason |
|-------|--------|
| `idx_customers_name` | UI search/autocomplete by customer name |
| `idx_customers_code` | Optional code-based lookups |
| `idx_customers_archived` | Filter active vs. archived customers |
| `idx_work_orders_customer_id` | Hierarchical queries (customer → projects) |
| `idx_work_orders_status` | Filter by status (active/paused/closed) |
| `idx_work_orders_archived` | Filter active vs. archived projects |
| `idx_work_orders_favorite` | Quick access to favorites (Phase 2) |
| `idx_sessions_start_time` | Date filtering for summaries |
| `idx_sessions_end_time` | Find open sessions, filter completed |
| `idx_sessions_work_order_id` | Hierarchical queries (project → sessions) |
| `idx_sessions_date_range` | Composite index for date range queries (Phase 2+) |
| `idx_recent_last_used` | Order recent items by most recent |

### Index Trade-offs

- **Pros**: Faster queries, especially for filtering and ordering
- **Cons**: Slower inserts/updates (indexes must be maintained), disk space

For this app:
- **Read-heavy** (users query far more often than they write)
- **Indexes are worth the cost** (small dataset, so disk impact is negligible)

---

## Migrations

Migrations are version-controlled SQL files in `src-tauri/migrations/`. The app runs them automatically on startup.

### Migration 001: Initial Schema

**File**: `001_initial_schema.sql`  
**Applies**: All tables above (customers, work_orders, time_sessions, active_session, recent_work_orders)

### Migration 002: Phase 2+ Features

**File**: `002_phase2_features.sql`  
**Adds**:
- `time_sessions.paused_at` — Timestamp when paused
- `time_sessions.total_paused_seconds` — Cumulative pause duration
- `active_session.is_paused` — Boolean (1/0)
- `active_session.paused_session_at` — When paused
- `work_orders.is_favorite` — Boolean (1/0)
- Index on `work_orders(is_favorite)` — For quick-access favorites

---

## Testing

### Database Tests

Run Rust integration tests to verify schema and queries:

```bash
cd src-tauri
cargo test --test db
```

Tests verify:
- Schema creates without errors
- Foreign keys enforced
- Indexes exist and are used
- Crash recovery logic works
- Duration calculation accurate

---

## Future Extensions (Phase 2+)

- **Audit log**: Track all changes (created/updated/deleted) for compliance
- **Immutability**: Lock entries after 30 days (read-only, can't edit)
- **Denormalization**: Pre-compute daily summaries for faster queries
- **Archive**: Separate table for old entries (> 1 year)
- **Multi-user**: Add `user_id` column to all tables (if team features added)

---

## References

- **SQLite WAL Documentation**: https://www.sqlite.org/wal.html
- **SQLite PRAGMA Reference**: https://www.sqlite.org/pragma.html
- **ISO 8601 Timestamps**: https://www.w3.org/TR/NOTE-datetime
