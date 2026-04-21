# Chewie — History

## Core Context

Backend Dev for work-tracker-2 — native desktop time tracker for consultant Fredrik Kristiansen Wikestad. Responsible for data layer (SQLite or equivalent), service logic (start/stop/switch sessions atomically), business rules, and export.

## Project Summary (Phases 1-4 Complete)

**Tech Stack**: Tauri 2, Rust 1.75+, SQLite (WAL mode), rusqlite 0.31, Tokio async

**Key Achievements**:
- ✅ **Phase 1 (MVP)**: Core time tracking, session switching, crash recovery, SQLite persistence
- ✅ **Phase 2**: Paused sessions, favorites, advanced quick-switch, global hotkeys, widget mode
- ✅ **Phase 3**: Background running, reports tab, archive management, enhanced tray integration
- ✅ **Phase 4**: ServiceNow CSV export format, edit session times, dev/prod isolation

**Critical Patterns**:
1. **Atomic Operations**: Multi-step operations (stop + start, create + start) use unchecked_transaction()
2. **Duration Handling**: Support both calculated (end_time - start_time) and manual override via duration_override
3. **Crash Recovery**: active_session singleton with heartbeat monitoring (2-min timeout)
4. **Error Handling**: AppError enum with structured Result<T> returns, JSON serialization for Tauri IPC
5. **Database**: WAL mode, foreign keys ON, PRAGMA synchronous=NORMAL, run_migrations on every init
6. **Transactions**: Use db.unchecked_transaction() for multi-step atomicity

**Performance Targets (Verified)**:
- Single entry create/update: <100ms ✅
- Daily summary query: <100ms ✅
- Weekly report query: <500ms ✅
- Search/autocomplete: <50ms ✅
- Context switch: <3s ✅

**Service Layer** (~85% documented):
- `session_service.rs`: 21 functions (start, stop, switch, pause, resume, quick-add, crash recovery)
- `summary_service.rs`: 8 functions (daily totals, reports, CSV export, recent/favorites)
- `command handlers`: 20+ Tauri IPC commands for customers, work orders, sessions, reports

**Database Schema** (001 initial + 002 Phase 2+):
- Customers, WorkOrders, TimeSessions, ActiveSession singleton, RecentWorkOrders
- Full indexes on start_time, work_order_id, customer_id, archived_at
- Foreign key constraints enabled, cascading deletes, duplicate detection on insert

**Definition of Done** (Charter requirement):
- ✅ `cargo clippy -- -D warnings` — zero warnings
- ✅ `cargo test` — all tests pass (53/53 as of Apr-22)
- ✅ `npm test -- --run` — all frontend tests pass
- ✅ `npm run build` — full build succeeds

**Last Known State** (2026-04-22):
- In-memory dev database implemented (Issue #31)
- All 53 Rust tests passing, 0 Clippy warnings
- Window title "[Dev]" for debug builds
- Persistent file-based DB for release builds
- Ready for developer testing

---

## Learnings

### Critical Production Patterns (Read First)

**Mutex Safety**: Never `.unwrap()` on lock() — thread panic corrupts all future locking. Always: `.map_err(|e| AppError::...)?`

**Atomic Transactions**: Multi-step operations use `db.unchecked_transaction()?` wrapping all steps. All-or-nothing execution prevents partial corruption.

**Duration Calculation**: Store gross time (end - start) in `duration_seconds`. Use `COALESCE(duration_override, duration_seconds)` for effective (no subtraction). Supports auto + manual override.

**Timestamp Format**: RFC3339 (`"2024-01-15T10:30:00Z"`) is standard. parse_timestamp() handles both RFC3339 and SQLite format for backward compat.

**Error Handling**: All commands return `Result<T>` with AppError. JSON serialization → Tauri IPC. Never swallow errors in catch blocks.

---

### Early Sessions: Key Technical Decisions (Apr 11-15)

**Phase 1 Scaffold** — Complete Tauri 2 + Rust:
- 16 IPC commands, atomic session switching, crash recovery (heartbeat monitor)
- session_service.rs (21 functions), summary_service.rs (8 functions)
- Database schema: 001 initial + 002 Phase 2 migrations
- WAL mode, foreign keys ON, composite indexes, SQLite default timestamps

**Documentation** (Apr 14):
- service_layer ~85% coverage (both service files fully documented)
- Module-level `//!` docs, all public functions + helpers

**Widget Mode Backend** (Apr 14):
- toggle_widget_mode command with Ctrl+Alt+W global shortcut
- Always-on-top 320×150px window with size/position state tracking

**Archive Filtering** (Phase 3):
- include_archived parameter for list commands
- unarchive_customer + unarchive_work_order commands
- Recent work orders exclude archived customers

---

### Recent Sessions: Phase Completions & Bug Fixes

**2026-04-13: Pre-Release Bugs Fixed**:
- Export permissions: Added dialog:allow-save + fs:allow-write-text-file to capabilities
- Tray icon grey on Windows: Removed .icon_as_template(true) — now green/amber/grey render correctly
- Graceful exit: Added window.destroy() before app.exit(0) to avoid ERROR_CLASS_HAS_WINDOWS (1412)

**2026-04-15: Session Time Editing** (Leia implemented most, Chewie fixed validation):
- Allows editing start/end times of completed sessions
- Validation split: zero-duration vs invalid ordering with specific error messages
- 27 tests pass, 1 ignored (Phase 2 overlap prevention)

**2026-04-21: Dev/Prod Isolation** (Issue #31):
- In-memory SQLite for debug builds (`:memory:`, fresh on each start)
- Persistent file DB for release builds (production data)
- `cfg!(debug_assertions)` compile-time decision
- Window title "[Dev]" for visual distinction in debug mode
- All 53 Rust tests pass, 0 Clippy warnings

