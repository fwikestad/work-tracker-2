-- Phase 4a: Customizable activity types
-- Replaces free-text activity_type with a managed list

CREATE TABLE IF NOT EXISTS activity_types (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_activity_types_sort ON activity_types(sort_order);

-- Seed default activity types
INSERT OR IGNORE INTO activity_types (id, name, sort_order) VALUES
    ('at-development',  'Development',   0),
    ('at-meeting',      'Meeting',       1),
    ('at-code-review',  'Code Review',   2),
    ('at-documentation','Documentation', 3),
    ('at-admin',        'Admin',         4),
    ('at-testing',      'Testing',       5),
    ('at-support',      'Support',       6);
