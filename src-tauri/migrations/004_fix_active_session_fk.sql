-- Migration 004: Fix active_session FK constraints and recover missing singleton
--
-- Migration 003 accidentally changed active_session foreign keys from
-- ON DELETE SET NULL to ON DELETE CASCADE. This caused discard_orphan_session
-- (and delete_session) to cascade-delete the singleton row (id=1), after which
-- all session tracking silently failed (UPDATE WHERE id=1 matched 0 rows).
--
-- This migration:
--   1. Rebuilds active_session with the correct ON DELETE SET NULL constraints
--   2. Re-inserts the singleton row if it was cascade-deleted

CREATE TABLE active_session_fixed (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    session_id TEXT,
    work_order_id TEXT,
    started_at TEXT,
    last_heartbeat TEXT,
    FOREIGN KEY (session_id) REFERENCES time_sessions(id) ON DELETE SET NULL,
    FOREIGN KEY (work_order_id) REFERENCES work_orders(id) ON DELETE SET NULL
);

-- Copy any existing data (singleton may or may not exist)
INSERT OR IGNORE INTO active_session_fixed (id, session_id, work_order_id, started_at, last_heartbeat)
SELECT id, session_id, work_order_id, started_at, last_heartbeat FROM active_session;

-- Recover the singleton if it was deleted by the cascade bug
INSERT OR IGNORE INTO active_session_fixed (id, session_id, work_order_id, started_at, last_heartbeat)
VALUES (1, NULL, NULL, NULL, NULL);

DROP TABLE active_session;
ALTER TABLE active_session_fixed RENAME TO active_session;
