-- Work Tracker 2 - Initial Schema
-- Based on architecture.md Section 4

PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;

-- ============================================
-- CUSTOMERS
-- ============================================
CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    code TEXT,
    color TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    archived_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_customers_name ON customers(name);
CREATE INDEX IF NOT EXISTS idx_customers_code ON customers(code);
CREATE INDEX IF NOT EXISTS idx_customers_archived ON customers(archived_at);

-- ============================================
-- WORK ORDERS
-- ============================================
CREATE TABLE IF NOT EXISTS work_orders (
    id TEXT PRIMARY KEY NOT NULL,
    customer_id TEXT NOT NULL,
    name TEXT NOT NULL,
    code TEXT,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    archived_at TEXT,
    
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_orders_customer_id ON work_orders(customer_id);
CREATE INDEX IF NOT EXISTS idx_work_orders_status ON work_orders(status);
CREATE INDEX IF NOT EXISTS idx_work_orders_archived ON work_orders(archived_at);

-- ============================================
-- TIME SESSIONS
-- ============================================
CREATE TABLE IF NOT EXISTS time_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    work_order_id TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    duration_seconds INTEGER,
    duration_override INTEGER,
    activity_type TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    
    FOREIGN KEY (work_order_id) REFERENCES work_orders(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sessions_start_time ON time_sessions(start_time);
CREATE INDEX IF NOT EXISTS idx_sessions_end_time ON time_sessions(end_time);
CREATE INDEX IF NOT EXISTS idx_sessions_work_order_id ON time_sessions(work_order_id);
CREATE INDEX IF NOT EXISTS idx_sessions_date_range ON time_sessions(start_time, end_time);

-- ============================================
-- ACTIVE SESSION (Singleton for crash recovery)
-- ============================================
CREATE TABLE IF NOT EXISTS active_session (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    session_id TEXT,
    work_order_id TEXT,
    started_at TEXT,
    last_heartbeat TEXT,
    
    FOREIGN KEY (session_id) REFERENCES time_sessions(id) ON DELETE SET NULL,
    FOREIGN KEY (work_order_id) REFERENCES work_orders(id) ON DELETE SET NULL
);

-- Initialize the singleton row
INSERT OR IGNORE INTO active_session (id, session_id, work_order_id, started_at, last_heartbeat)
VALUES (1, NULL, NULL, NULL, NULL);

-- ============================================
-- RECENT ITEMS (for quick-switch)
-- ============================================
CREATE TABLE IF NOT EXISTS recent_work_orders (
    work_order_id TEXT PRIMARY KEY NOT NULL,
    last_used_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    use_count INTEGER NOT NULL DEFAULT 1,
    
    FOREIGN KEY (work_order_id) REFERENCES work_orders(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_recent_last_used ON recent_work_orders(last_used_at DESC);
