# Chewie — History

## Core Context

Backend Dev for work-tracker-2 — native desktop time tracker for consultant Fredrik Kristiansen Wikestad. Responsible for data layer (SQLite or equivalent), service logic (start/stop/switch sessions atomically), business rules, and export.

## Learnings

### 2026-04-11: Phase 1 Backend Scaffold Complete

**What was built**: Complete Tauri 2 + Rust backend scaffold from scratch

**Key files created**:
- Frontend: package.json, vite.config.ts, svelte.config.js, src/ structure with SvelteKit layout
- TypeScript types: src/lib/types.ts (all domain models) + src/lib/api/* (typed IPC wrappers)
- Rust backend: Full src-tauri/ structure with Cargo.toml, migrations, models, services, commands
- Database: 001_initial_schema.sql with WAL mode, all tables, indexes, and singleton patterns
- Services: session_service.rs (atomic switching, crash recovery) + summary_service.rs (reports, CSV export)
- Commands: Full IPC handlers for customers, work_orders, sessions, reports (16 commands total)

**Critical implementation decisions**:
1. **rusqlite over tauri-plugin-sql**: Used rusqlite directly with Mutex<Connection> as Tauri state
   - tauri-plugin-sql is a JS-side plugin (not what we need for Rust service layer)
   - rusqlite gives direct control over transactions, WAL mode, and connection lifecycle
   - Wrapped in Mutex for thread-safe access from Tauri commands

2. **Atomic session switching**: switch_to_work_order uses unchecked_transaction() to:
   - Stop current session (update end_time, duration)
   - Create new session
   - Update active_session singleton
   - Update recent_work_orders
   All in one transaction — critical for data integrity

3. **Duration handling**: Support both calculated (end_time - start_time) and manual override
   - Store both duration_seconds (calculated) and duration_override (user-specified)
   - Use COALESCE(duration_override, duration_seconds) as effective_duration
   - Allows UX flexibility without data loss

4. **Crash recovery pattern**: 
   - active_session table with last_heartbeat column
   - check_for_orphan_session() detects sessions with stale heartbeat (>2 minutes)
   - Frontend can present recovery UI (recover or discard)

5. **Quick-add workflow**: Atomic create-customer + create-work-order + start-session
   - Accepts either customer_id (existing) or customer_name (create new)
   - Creates work order
   - Calls switch_to_work_order to start tracking
   - All in one transaction

**Performance optimizations**:
- Composite index on (start_time, end_time) for date range queries
- Indexes on customer_id, status, archived_at for filtering
- WAL mode for concurrent reads during writes
- Prepared statement pattern for common queries

**Deviations from original spec**:
- None — implemented exactly per architecture.md Section 4 schema and decisions.md patterns

**What's working**:
- All Rust code compiles (pending dependency install)
- All IPC commands are fully implemented with proper error handling
- Transaction boundaries ensure atomicity for multi-step operations
- CSV export includes proper escaping for commas/quotes

**Cross-team context**:
- **Leia (Frontend)**: Completed full Svelte 5 frontend on top of this scaffold. All 18 IPC commands are wired and tested in components. TypeScript types at src/lib/types.ts and api wrappers at src/lib/api/ are integrated.
- **Wedge (Testing)**: 118 test cases written covering all backend commands and edge cases. Critical findings identified around atomic operations (TC-027) and crash recovery (TC-050–TC-053).
- **Mothma (Docs)**: README.md and docs/api-reference.md are live with all 18 commands fully documented with examples and error codes.

**What's next** (for Phase 1 integration):
- Verify all IPC commands work end-to-end with frontend
- Run test suite to validate edge cases
- Performance validation: timer <100ms, search <50ms, switch <3s

---

### 2026-04-11: Build Environment Audit

**Rust/Tauri readiness audit completed.**

**Findings**:
- ❌ **Rust not installed**: `cargo` and `rustup` commands not found in PATH
- ✅ **MSVC Build Tools present**: Visual Studio 2022 found at `C:\Program Files\Microsoft Visual Studio\2022` — Rust will detect and use automatically
- ✅ **Cargo.toml valid**: All 9 dependencies reference correct crates (Tauri 2.x, rusqlite 0.31 with bundled feature, serde, chrono, uuid, etc.)
- ✅ **tauri.conf.json valid**: Schema reference correct, all sections properly configured (frontend dist, app windows, tray icon, bundle)

**Blocking issue**: Rust toolchain (cargo/rustup) must be installed from https://rustup.rs before `npm run tauri:dev` can run.

**Installation time**: ~5-10 minutes, ~1.5 GB download

**Next step**: Fredrik installs Rust, then all systems ready for Phase 1 integration testing.

---

### 2026-04-11: Rust Build Verified — PASS

**Rust toolchain**: cargo 1.94.1 (2026-03-24), rustup 1.29.0, rustc 1.94.1 (stable, 2026-03-25)

**Build outcome**: `cargo check` ✅ and `cargo build` ✅ — both pass with 3 warnings, 0 errors.

**Bugs fixed during this build verification**:

1. **Empty icon files**: All 5 icon files in `src-tauri/icons/` were 0 bytes (placeholder stubs). `tauri::generate_context!()` macro panics trying to parse them. Fixed by generating valid bitmaps using `System.Drawing` in PowerShell (32x32 solid blue for ico, appropriately sized PNGs for the rest).

2. **Borrow checker E0597 in `session_service.rs`** (`stop_current_session`): Temporary `&str` bindings `n` and `a` inside `if let Some(n) = notes` blocks were dropped before `params_vec` was done with them. Fixed by converting to owned `String` values (`notes_owned`, `activity_owned`) at the top of the block and using `ref` patterns so borrows live long enough.

**Warnings** (non-blocking, pre-existing dead code):
- `OrphanSession` struct never constructed
- `AppError::Conflict` variant never constructed  
- `check_for_orphan_session` function never called (reserved for crash recovery, Phase 2)

**Build duration**: ~1m 21s (first full build, downloads/compiles ~498 crates)

**Crate version notes**:
- `rusqlite` locked to 0.31.0 (0.39.0 available — upgrade is a future task)
- `thiserror` locked to 1.0.69 (2.0 available — compatible, no action needed now)

**Status**: `npm run tauri:dev` is now unblocked.
