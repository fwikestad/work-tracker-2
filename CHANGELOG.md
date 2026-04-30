# Changelog

All notable changes to Work Tracker 2 are documented here.

---

## [Phase 4a] — ServiceNow Export & Activity Types

### Added
- ServiceNow-compatible CSV export (Reports → Export ServiceNow)
- Optional ServiceNow Task ID field on Work Orders (Manage → Work Orders)
- Customizable Activity Types with full CRUD (Manage → Activity Types)
- Expandable session rows in Report view — click a work order to see individual sessions with time range and notes
- "Last week" quick select in Reports toolbar
- Sticky "Back to tracking" footer in Manage view

### Changed
- Stop-session dialog now shows a dropdown for Activity Type (populated from the managed list instead of free text)

### Database
- Migration 005: `servicenow_task_id` column added to `work_orders`
- Migration 006: `activity_types` table with 7 seeded defaults (Development, Meeting, Code Review, Documentation, Admin, Testing, Support)

---

## [Phase 3] — Reports & Summaries

### Added
- Weekly and monthly summary views
- CSV export of time entries
- Date range filtering (today / this week / this month / custom)
- Session grouping by customer and work order

---

## [Phase 2] — Pause, Favorites & Widget Mode

### Added
- Pause and resume session support
- Favorite / pin work orders for one-click access
- Always-on-top widget mode (compact timer overlay)
- Week summary panel
- Global hotkey support (Ctrl+Shift+S)

---

## [Phase 1] — Core Time Tracking (MVP)

### Added
- Start / stop time tracking sessions
- Active timer display (real-time elapsed)
- Customer and work order management (full CRUD)
- Quick-add overlay (Ctrl+N) — create customer + work order and start tracking immediately
- Search-to-switch (Ctrl+K) — type to filter, Enter to switch work order
- Today's session list with inline editing
- Manual duration override
- Crash recovery — detects incomplete sessions on startup
- SQLite persistence with WAL mode (crash-safe writes)
- Daily summary totals
