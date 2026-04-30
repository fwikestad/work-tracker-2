-- Migration 003: Remove pause functionality and duration_override column
-- This is a graceful migration that auto-stops any paused sessions before removing columns

-- Step 1: Auto-stop any paused sessions before dropping columns
-- Calculate duration for any running paused session
UPDATE time_sessions 
SET end_time = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
    duration_seconds = CAST((julianday('now') - julianday(start_time)) * 86400 AS INTEGER)
WHERE end_time IS NULL 
  AND EXISTS (SELECT 1 FROM active_session WHERE session_id = time_sessions.id AND is_paused = 1);

-- Clear the active_session for any paused session
UPDATE active_session 
SET session_id = NULL, 
    work_order_id = NULL, 
    started_at = NULL, 
    last_heartbeat = NULL,
    is_paused = 0, 
    paused_session_at = NULL 
WHERE is_paused = 1;

-- Step 2: Remove pause-related columns from time_sessions
-- SQLite doesn't support DROP COLUMN directly, so we need to recreate the table
-- First, create the new schema without pause columns and duration_override
CREATE TABLE time_sessions_new (
    id TEXT PRIMARY KEY,
    work_order_id TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    duration_seconds INTEGER,
    activity_type TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (work_order_id) REFERENCES work_orders(id) ON DELETE CASCADE
);

-- Copy data from old table to new (excluding paused_at, total_paused_seconds, duration_override)
INSERT INTO time_sessions_new (id, work_order_id, start_time, end_time, duration_seconds, activity_type, notes, created_at, updated_at)
SELECT id, work_order_id, start_time, end_time, duration_seconds, activity_type, notes, created_at, updated_at
FROM time_sessions;

-- Drop old table and rename new table
DROP TABLE time_sessions;
ALTER TABLE time_sessions_new RENAME TO time_sessions;

-- Recreate indexes on the new table
CREATE INDEX idx_time_sessions_start_time ON time_sessions(start_time);
CREATE INDEX idx_time_sessions_work_order ON time_sessions(work_order_id);

-- Step 3: Remove pause columns from active_session
-- SQLite doesn't support DROP COLUMN directly, so we need to recreate the table
CREATE TABLE active_session_new (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    session_id TEXT,
    work_order_id TEXT,
    started_at TEXT,
    last_heartbeat TEXT,
    FOREIGN KEY (session_id) REFERENCES time_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (work_order_id) REFERENCES work_orders(id) ON DELETE CASCADE
);

-- Copy data from old table to new (excluding is_paused and paused_session_at)
INSERT INTO active_session_new (id, session_id, work_order_id, started_at, last_heartbeat)
SELECT id, session_id, work_order_id, started_at, last_heartbeat
FROM active_session;

-- Drop old table and rename new table
DROP TABLE active_session;
ALTER TABLE active_session_new RENAME TO active_session;
