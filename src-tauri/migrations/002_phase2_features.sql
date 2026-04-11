-- Phase 2+3 Backend Features Migration
-- Adds: Paused state, Favorites, and supporting columns

-- ============================================
-- PAUSED STATE (Phase 2)
-- ============================================

-- Add pause tracking to time_sessions
ALTER TABLE time_sessions ADD COLUMN paused_at TEXT;
ALTER TABLE time_sessions ADD COLUMN total_paused_seconds INTEGER DEFAULT 0;

-- Add pause state to active_session
ALTER TABLE active_session ADD COLUMN is_paused INTEGER DEFAULT 0;
ALTER TABLE active_session ADD COLUMN paused_session_at TEXT;

-- ============================================
-- FAVORITES / PINNING (Phase 2)
-- ============================================

-- Add favorite flag to work_orders
ALTER TABLE work_orders ADD COLUMN is_favorite INTEGER DEFAULT 0;

-- Index for favorite filtering
CREATE INDEX IF NOT EXISTS idx_work_orders_favorite ON work_orders(is_favorite);
