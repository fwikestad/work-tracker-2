-- Migration 003: Settings table
-- Provides a simple key/value store for user-configurable app settings.

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

-- Default: rounding is off
INSERT OR IGNORE INTO settings (key, value) VALUES ('round_to_half_hour', 'false');
