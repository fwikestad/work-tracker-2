# Features Reference

This document catalogs all features of Work Tracker 2, organized by phase and status.

---

## Phase 1 — Core Time Tracking (Implemented ✅)

### A. Timer & Tracking

- ✅ **Start tracking** — Begin session on a work order (Ctrl+K to switch, Ctrl+N to quick-add)
- ✅ **Stop tracking** — End session and calculate duration (Ctrl+S)
- ✅ **Active timer display** — Real-time elapsed time shown at top of app
- ✅ **Active indicator** — Visual distinction between running and stopped sessions
- ✅ **Auto-duration calculation** — End time - start time = duration
- ✅ **Manual duration override** — User can adjust duration after stopping
- ✅ **Crash recovery** — App detects incomplete sessions on startup and prompts user to close or discard

### B. Customers & Work Orders

- ✅ **Create customers** — Add new company/entity to track
- ✅ **Create work orders** — Add projects under customers
- ✅ **Edit customer** — Change name, code, or color
- ✅ **Edit work order** — Change name, code, description, status
- ✅ **Archive customer** — Hide inactive customers without deleting data
- ✅ **Archive work order** — Hide inactive projects without deleting data
- ✅ **Delete customer** — Hard delete (with confirmation; cascades to work orders/sessions)
- ✅ **Delete work order** — Hard delete (with confirmation; cascades to sessions)
- ✅ **Quick-add** — Create customer + work order in one action (Ctrl+N) and immediately start tracking

### C. Session Management

- ✅ **List sessions** — View all time entries for a given date
- ✅ **Filter by customer** — Show only entries for selected customer
- ✅ **Filter by work order** — Show only entries for selected project
- ✅ **Inline edit** — Click entry to edit duration, notes, activity type
- ✅ **Delete session** — Remove entry from history (with confirmation)
- ✅ **Add notes** — Capture details about what was worked on
- ✅ **Activity type** — Classify work (development, meeting, design, admin, other)

### D. Daily Summary

- ✅ **Today's total** — Show total hours worked today
- ✅ **Breakdown by customer** — Pie/bar chart of time per customer
- ✅ **Breakdown by work order** — List of projects worked on today with hours
- ✅ **Real-time updates** — Summary refreshes as sessions are created/edited/deleted

### D.1. Week Summary

- ✅ **Weekly view** — View all work from the current week (Monday–Sunday)
- ✅ **Week navigation** — Arrow controls to move to previous/next weeks
- ✅ **Inline editing** — Click entries to adjust duration, notes, or activity type
- ✅ **Access via tab** — Dedicated tab in navigation bar

### E. Reports & Export

- ✅ **Date range filtering** — Select today, this week, this month, or custom range
- ✅ **Export to CSV** — Generate spreadsheet-compatible export
- ✅ **Include session details** — CSV contains date, customer, work order, duration, notes, activity
- ✅ **Grouped export** — Option to group by customer or by project
- ✅ **Auto-download** — CSV saved to Downloads folder and opened in default app

### F. Data Persistence & Reliability

- ✅ **SQLite with WAL mode** — Crash-safe writes using write-ahead logging
- ✅ **Local-only storage** — All data in `work_tracker.db` on user's machine
- ✅ **No cloud required** — App works fully offline
- ✅ **Immediate persistence** — Every action saved to disk immediately (no "save" button)
- ✅ **Atomic transactions** — Multi-step operations (e.g., stop session + start new) are atomic
- ✅ **Foreign key constraints** — Database enforces data integrity

### G. User Interface

- ✅ **Responsive layout** — Adapts to different window sizes
- ✅ **Keyboard-first** — All common actions accessible via keyboard
- ✅ **Minimal dialogs** — Use inline forms and overlays instead of pop-ups
- ✅ **Undo support** — Reversible actions (delete shows confirmation + undo)
- ✅ **Search/switch** — Ctrl+K opens quick-search overlay to find and switch projects
- ✅ **Touch-friendly** — Large touch targets (44px+) for stylus/glove use
- ✅ **Color-coded customers** — Optional visual distinction by customer

### H. System Tray Integration

- ✅ **System tray icon** — App visible in taskbar/system tray even when minimized
- ✅ **Active session display** — Tray shows current work order and elapsed time
- ✅ **Quick-switch menu** — Click tray icon to see recent/favorite projects and switch
- ✅ **Stop action in tray** — Stop tracking without opening main window
- ✅ **Context menu** — Right-click tray for options (minimize, quit, etc.)

---

## Phase 2 — Multi-Customer Workflows (Implemented ✅)

### A. Paused Sessions

- ✅ **Pause tracking** — Freeze timer without closing session (Ctrl+P)
- ✅ **Resume tracking** — Unfreeze timer and continue
- ✅ **Paused state display** — Show visually that session is paused (separate from running/stopped)
- ✅ **Pause duration tracking** — Cumulative pause time separate from active time
- ✅ **Manual adjustment** — User can adjust total_paused_seconds if needed

### B. Favorites & Pinning

- ✅ **Mark favorite** — Pin frequently-used work orders for quick access
- ✅ **Unmark favorite** — Remove from favorites
- ✅ **Favorites list** — Show pinned projects at top of quick-switch (Ctrl+K)
- ✅ **Recent list** — Track and display recently used projects
- ✅ **One-click access** — Click favorite to start tracking immediately

### C. Advanced Quick-Switch

- ✅ **System tray quick-switch** — Click tray icon to see favorites + recents
- ✅ **Keyboard-accessible** — All navigation via arrow keys and Enter
- ✅ **Search within tray menu** — Type to filter favorites/recents
- ✅ **Visual recency** — Show when each project was last used

### D. Organizing Customers

- ✅ **View all customers** — Customer management page with full CRUD
- ✅ **Search customers** — Find customer by name or code
- ✅ **Bulk operations** — Select multiple customers (Phase 2+)
- ✅ **Status display** — Show active vs. archived
- ✅ **Last-used metadata** — Show when customer was last worked on

### E. Widget Mode

- ✅ **Always-on-top window** — Shrink to compact floating overlay
- ✅ **Persistent tracking** — Continue tracking while widget is visible
- ✅ **Quick-switch from widget** — Click work order name to switch projects
- ✅ **Toggle with shortcut** — Ctrl+W / Cmd+W to enable/disable widget mode
- ✅ **Restore previous state** — Window size/position restored when exiting widget mode

---

## Phase 3 — Background & Reports (Implemented ✅)

### A. Background Running

- ✅ **Minimize to tray** — Close main window but continue tracking
- ✅ **Persistent tracking** — Session continues even if app is minimized/hidden
- ✅ **Tray notifications** — Optional reminders (Phase 3+)

### B. Reports Tab

- ✅ **Advanced report generation** — Build custom reports with filters
- ✅ **Date range picker** — Select any start/end date
- ✅ **Group by** — Option to group by customer, work order, or activity type
- ✅ **Summaries** — Show totals, averages, and breakdowns
- ✅ **Time zone awareness** — Correct handling for users across regions

### C. Archive Management

- ✅ **Archive old entries** — Move old sessions to archive for performance
- ✅ **Unarchive** — Restore archived entries if needed
- ✅ **Separate view** — View archived data without cluttering current view

### D. Tray Menu Enhancements

- ✅ **Quick-access menu** — Right-click tray shows recent projects + actions
- ✅ **One-click actions** — Stop, pause, or resume from tray menu
- ✅ **Current status** — Tray tooltip shows active work order and elapsed time

---

## Phase 4a — ServiceNow Integration (In Progress 🚧)

### A. ServiceNow Import

- 🚧 **ServiceNow Import Set CSV export** — Export sessions in ServiceNow-compatible format with format toggle in Reports UI
  - Columns: date, customer, work order, code, start time, end time, duration (hours), activity type, notes
  - User manually uploads via ServiceNow's CSV Import Set (no API dependency)
  - Validates user demand before Phase 4b REST automation
- 📋 **ServiceNow API push (Phase 4b)** — Direct REST integration; credentials in OS keychain (parked pending Phase 4a validation)

---

## Phase 4b+ — Team & Integrations (Planned 📋)

### A. Multi-User

- 📋 **Multiple users per computer** — Each user has separate tracking data
- 📋 **User switching** — Quick profile switching without logout
- 📋 **Shared projects** — Optional: team members can share project definitions

### B. Multi-Device (Optional)

- 📋 **Cloud sync** (optional) — Sync data across user's personal devices
- 📋 **Conflict resolution** — Handle overlapping sessions on different devices
- 📋 **Offline merging** — Sync when reconnected

### C. Integrations

- 📋 **Billing tool export** — Export to accounting/invoicing software
- 📋 **Calendar integration** — Export sessions as calendar events
- 📋 **Slack notifications** — Optional: notify when switching projects
- 📋 **Zapier/IFTTT** — Webhooks for third-party automation

### D. Team Features

- 📋 **Project templates** — Shared templates for common project structures
- 📋 **Activity classifications** — Team-defined activity types
- 📋 **Billing rates** — Per-customer or per-project rates for revenue tracking
- 📋 **Time entry approval** — Manager approval workflow for tracked time

### E. Notifications & Alerts

- 📋 **Idle warning** — Remind user if tracking has been paused for too long
- 📋 **End-of-day summary** — Daily summary notification
- 📋 **Schedule reminders** — Nudge at specific times (e.g., "Start tracking!")

---

## Not Planned (Out of Scope)

- ❌ **Web app or mobile app** — Desktop-only (Tauri limitation + deliberate choice for simplicity)
- ❌ **Real-time collaboration** — Not a shared tool; single-user focused
- ❌ **Attendance tracking** — Not a time clock; focused on billable work tracking
- ❌ **AI time classification** — No automatic activity categorization
- ❌ **Geolocation tracking** — Privacy-first: no location data collected

---

## Session States Summary

### Phase 1

| State | Duration | Timer | UI | Transition |
|-------|----------|-------|----|----|
| Running | ∞ | Active | Green, counting up | Start → Running; Running → Stopped |
| Stopped | Final (calculated or overridden) | Stopped | Grey, static | Running → Stopped |

### Phase 2+

| State | Duration | Timer | UI | Transition |
|-------|----------|-------|----|----|
| Running | ∞ | Active | Green, counting up | Start → Running; Running → Paused/Stopped |
| Paused | Frozen (excludes pause time) | Frozen | Amber/Yellow, static | Running → Paused; Paused → Running |
| Stopped | Final (calculated or overridden, excludes pause time) | Stopped | Grey, static | Running/Paused → Stopped |

---

## Keyboard Shortcuts Reference

| Phase | Shortcut | Action | Availability |
|-------|----------|--------|--------------|
| 1 | Ctrl+N / Cmd+N | Quick-add (create + start) | Anywhere in app |
| 1 | Ctrl+K / Cmd+K | Search and switch | Anywhere in app |
| 1 | Ctrl+S / Cmd+S | Stop tracking | Anywhere in app |
| 2 | Ctrl+P / Cmd+P | Pause / Resume | Anywhere in app |
| 1 | Esc | Close overlay | When overlay open |
| 1 | ↑ / ↓ | Navigate results | In search results |
| 1 | Enter | Confirm selection | In search results |
| 3 | Ctrl+E / Cmd+E | Open Reports tab | Phase 3+ |

---

## Test Coverage

| Component | Phase 1 | Phase 2 | Phase 3 | Status |
|-----------|---------|---------|---------|--------|
| Session management | Tests ✅ | Tests ✅ | Tests ✅ | 7 Rust integration tests |
| Customer/work order CRUD | Tests ✅ | Tests ✅ | Tests ✅ | Covered by Rust tests |
| UI components | Smoke ✅ | Partial | Partial | ~55 Vitest tests |
| Quick-add | Tests ✅ | — | — | E2E coverage |
| Daily summary | Tests ✅ | Tests ✅ | Tests ✅ | Query verified |
| CSV export | Tests ✅ | Tests ✅ | Tests ✅ | Format verified |
| Paused sessions | — | Tests ✅ | Tests ✅ | Phase 2+ |
| Favorites | — | Tests ✅ | Tests ✅ | Phase 2+ |

---

## Performance Targets

| Operation | Target | Current | Status |
|-----------|--------|---------|--------|
| App startup | < 1s | ~500ms | ✅ |
| Start tracking | < 500ms | ~200ms | ✅ |
| Stop tracking | < 500ms | ~150ms | ✅ |
| Pause/resume | < 200ms | ~100ms | ✅ |
| Switch project | < 3s | ~1s | ✅ |
| Daily summary | < 100ms | ~50ms | ✅ |
| Search | < 50ms | ~30ms | ✅ |
| Export (1 month) | < 5s | ~2s | ✅ |

---

## Feature Gating (How Features Are Controlled)

### Phase 1 → Phase 2 Migration

- Database migration adds new columns (`paused_at`, `is_favorite`)
- UI conditionally shows pause button if `hasPauseFeature` (config flag)
- Favorites only visible if enabled in settings
- Backwards compatible: old data works with new app version

### Phase 2 → Phase 3 Migration

- Archive table created in new migration
- Reports tab shown in main nav
- Background running enabled (tray stays active)
- Archive management UI only visible if data exists

### Conditional Features

All features in Phase 2+ can be:
- Disabled at compile time (Cargo features) for lighter builds
- Toggled at runtime (settings) for A/B testing
- Gated by experiment flags (Phase 3+)

---

## Backward Compatibility

- ✅ **Older app versions** can read newer database (extra columns ignored)
- ✅ **Newer app versions** can read older database (migrations auto-apply)
- ✅ **No forced migrations** — Old data preserved, new fields default to NULL or sensible values
- ✅ **CSV exports** compatible across versions

---

## Documentation Links

- **[docs/development.md](development.md)** — How to build and test features
- **[docs/data-model.md](data-model.md)** — Database schema for feature implementation
- **[docs/architecture.md](architecture.md)** — System design and three-layer architecture
- **.github/workflows/** — CI/CD pipelines for automated testing
