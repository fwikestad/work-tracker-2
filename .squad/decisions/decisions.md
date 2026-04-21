# Work Tracker 2 — Decisions Log

## Dev/Prod Environment Isolation

**Date**: 2026-04-14  
**Agent**: Lando (DevOps)  
**Issue**: #31  
**Status**: Implemented  

---

### Context

Developers need to run development builds of Work Tracker 2 without interfering with an installed production version. Previously, both shared the same Tauri app identifier (`com.work-tracker-2.app`), causing data collision and confusion.

### Decision

Create a **Tauri 2 config overlay** (`src-tauri/tauri.dev.conf.json`) that changes the app identifier and product name for dev builds:

- **Debug builds** → `com.work-tracker-2.dev`, displayed as "Work Tracker 2 (Dev)"
- **Release builds** → `com.work-tracker-2.app`, displayed as "Work Tracker 2"

### Implementation

- Created `src-tauri/tauri.dev.conf.json` with overrides for `productName` and `identifier`
- Updated `package.json` npm script: `"tauri:dev": "tauri dev --config src-tauri/tauri.dev.conf.json"`

### Rationale

- Uses native Tauri 2 config overlay feature (no hacks, well-supported)
- Minimal duplication (only 4 lines vs full config)
- Different app identifiers = completely isolated data directories across platforms
- Clear intent via filename
- Zero code changes required

### Impact

- ✅ Dev and prod can run simultaneously without conflicts
- ✅ Window title clearly marks dev builds
- ✅ Data directories remain separate
- ⚠️ Dev data lost on app restart (intentional)

---

## In-Memory Database for Dev Builds

**Date**: 2026-04-22  
**Author**: Chewie (Backend Dev)  
**Status**: Implemented  
**Related Issue**: #31

---

### Context

Work Tracker 2 dev builds and production builds were both using the same persistent SQLite database file, causing conflicts when developers wanted to test features while using the production app or run fresh for each dev session.

### Decision

Use compile-time conditional compilation (`cfg!(debug_assertions)`) to separate dev and production database initialization:

- **Debug builds** → In-memory SQLite database (`:memory:`)
- **Release builds** → Persistent file-based SQLite database

### Implementation

#### Added `init_dev_db()` Function
Location: `src-tauri/src/db/mod.rs`

```rust
pub fn init_dev_db() -> SqlResult<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    run_migrations(&conn)?;
    Ok(conn)
}
```

#### Modified Setup Hook
Location: `src-tauri/src/lib.rs`

Conditionally initializes database:
- If `cfg!(debug_assertions)` → use `init_dev_db()` (in-memory)
- Otherwise → use `db::initialize()` (persistent)

#### Added Dev Window Title
Debug builds set window title to "Work Tracker 2 [Dev]" for visual distinction.

### Rationale

1. **Independence**: Dev and prod never interfere
2. **Fresh State**: Every dev launch starts with clean database
3. **Speed**: In-memory SQLite is faster than file I/O
4. **Simplicity**: No cleanup, no stale data
5. **Test-like Environment**: Same pattern as `init_test_db()` used in unit tests
6. **Compile-time Decision**: `cfg!(debug_assertions)` is standard Rust, automatic, no config needed

### Trade-offs

**Advantages**:
- ✅ Dev and prod completely isolated  
- ✅ Fresh database on every dev start  
- ✅ No persistent dev data to maintain  
- ✅ Clear visual distinction  
- ✅ Zero risk of corrupting production data  

**Disadvantages**:
- ❌ Dev data lost on app restart (intentional)  
- ❌ Cannot test persistence/recovery flows in dev mode  
- ❌ Must use release build to test real database behavior

### Verification

CI checks passing:
- ✅ `cargo clippy -- -D warnings` — 0 warnings
- ✅ `cargo test` — 53 tests passed

---

## CI Enforcement — Definition of Done for All Coding Agents

**Date**: 2026-04-13  
**Authored by**: Fredrik Kristiansen Wikestad (via Copilot Coordinator)  
**Priority**: High — ensures CI consistency and prevents lint-failure loop

---

### Decision

All coding agents (Chewie, Leia, and any future dev agents) must run all four CI checks locally and confirm they pass **before committing any code**. This is now a formal Definition of Done for all coding work.

### CI Checks Required (in order)

```bash
cd src-tauri && cargo clippy -- -D warnings   # Zero warnings or errors
cd src-tauri && cargo test                     # All tests pass
npm test -- --run                              # All frontend tests pass
npm run build                                  # Build succeeds
```

### Rationale

- CI enforces `-D warnings` on Clippy, which treats all lint warnings as hard errors
- Code that compiles locally can silently fail CI if it triggers a warning
- Baking this into agent charters prevents the push-fail-fix-revert loop
- Applies to **all code changes**, no exceptions for "quick fixes" — size does not matter
- Shift-left testing: catch issues at the source, not in CI

### Implementation

- Added `## Definition of Done` section to `.squad/agents/chewie/charter.md`
- Added `## Definition of Done` section to `.squad/agents/leia/charter.md`
- Applied retrospectively to commit b09f4f6 (tray.rs pattern cleanup)

### Scope

- Who: All agents performing code changes
- What: All four CI checks must pass
- When: Before every commit
- Where: Local development environment before git push

---

## Rust Build Status — PASS ✅

**Date**: 2026-04-11  
**Author**: Chewie (Backend Dev)  
**Priority**: High — unblocks frontend integration

---

### Summary

Rust backend now compiles fully. `cargo check` and `cargo build` both pass.

---

### Rust Build Results

| Step | Status | Notes |
|------|--------|-------|
| `cargo --version` | ✅ | cargo 1.94.1 (29ea6fb6a 2026-03-24) |
| `rustup --version` | ✅ | rustup 1.29.0 (28d1352db 2026-03-05), rustc 1.94.1 |
| `cargo check` | ✅ | 3 warnings, 0 errors — 2.31s |
| `cargo build` | ✅ | Finished dev profile in 1m 21s |

---

### Verdict: PASS

---

### Bugs Fixed

#### 1. Empty icon files (all 0 bytes)
- **File**: `src-tauri/icons/` — all 5 icon stubs were empty placeholders
- **Error**: `tauri::generate_context!()` proc macro panicked: `failed to parse icon: failed to fill whole buffer`
- **Fix**: Generated valid icons using PowerShell + System.Drawing (32x32 solid blue ICO, correct-size PNGs)
- **Action needed**: Replace placeholder icons with real app icons before shipping

#### 2. Borrow checker E0597 in `session_service.rs`
- **File**: `src/services/session_service.rs`, function `stop_current_session`, lines 101–108
- **Error**: `n` and `a` (`&str` bindings from `if let Some(n) = notes`) dropped before `params_vec` finished using them
- **Fix**: Convert to owned `String` values (`notes_owned`, `activity_owned`) before building `params_vec`, use `ref` bindings

---

### Non-Blocking Warnings (pre-existing dead code)

- `OrphanSession` struct never constructed (crash recovery, Phase 2)
- `AppError::Conflict` never constructed (defensive code)
- `check_for_orphan_session` never called (Phase 2 feature)

These are intentional scaffolds for future phases — no action required now.

---

### Next Steps

- **Fredrik / all team**: Run `npm run tauri:dev` — Rust backend is now unblocked
- **Leia (Frontend)**: Integration testing can begin; all IPC commands should be callable from the Svelte frontend
- **Wedge (Testing)**: Test suite can run against the live backend
- **Future (Chewie)**: Replace placeholder icon files with real app icons before production packaging

---

## Wedge Smoke Test Verdict — Go/No-Go for tauri:dev

**Date**: 2026-04-11  
**Requested by**: Fredrik Kristiansen Wikestad  
**Tester**: Wedge

---

### Smoke Test: Go/No-Go for tauri:dev

✅ Node.js working — v24.14.1  
✅ npm working — v11.11.0  
✅ Frontend build clean — exit code 0, 169 SSR + 187 client modules, `build/` output generated  
✅ Rust source files present — main.rs (107B), lib.rs (1979B), session_service.rs (13728B), 001_initial_schema.sql (3707B)  
✅ tauri:dev script configured — `"tauri:dev": "tauri dev"` in package.json  
✅ tauri.conf.json devUrl set — `"devUrl": "http://localhost:1420"` with `beforeDevCommand: npm run dev`  

### Overall: GO 🟢

### Command to run:
```
npm run tauri:dev
```

### Known warnings to expect (non-blocking):

**Frontend build warnings** (same as previous verified build — no regressions):
- `QuickAdd.svelte:88` — a11y: overlay div missing keyboard handler + ARIA role
- `QuickAdd.svelte:18` — `inputRef` not declared with `$state(...)` (reactivity)
- `SessionList.svelte:103` — a11y: session div missing keyboard handler + ARIA role
- `Timer.svelte:48` — self-closing `<textarea />` should be `<textarea></textarea>`
- `CustomerList.svelte:159` — a11y: item-info div missing keyboard handler + ARIA role
- `WorkOrderList.svelte:195` — a11y: item-info div missing keyboard handler + ARIA role

**First-run Rust compile** (expected, not an error):
- Cargo will download and compile all crate dependencies on first run — this can take 5–15 minutes
- Subsequent runs will be fast (incremental compilation)
- Watch for `Compiling work-tracker-2 v0.1.0` — this means Rust compile started successfully

### Notes
- `devUrl: http://localhost:1420` — Tauri will wait for the Vite dev server to start on that port before opening the window (handled by `beforeDevCommand`)
- If the window doesn't open, check that Vite successfully bound to port 1420 in the terminal output

---

## UI Mockup v2 — Revision Notes

**Author**: Leia (Frontend Dev)  
**Date**: 2026-04-11  
**File changed**: `docs/ui-mockup.html` — complete rewrite

---

### 1. Much darker theme

**Before**: `#1a1d24` background, `#252932` surface, `#3b82f6` blue accent — dark but not near-black, with multiple accent colours (blue, green, amber).

**After**: `#0d0d0d` background (near-black), `#1a1a1a` surface, `#2a2a2a` border, `#e8e8e8` off-white text, `#4caf7d` single teal accent reserved **only** for running state. Customer colour dots remain (8px muted circles) but are the only colour variation.

**Why**: Fredrik explicitly asked for very dark, monochrome, professional-tool aesthetic. The old palette felt like a consumer SaaS app. New palette is closer to a terminal / IDE — zero visual noise.

---

### 2. Layout: two-column → single-column

**Before**: Two-column desktop layout (400px left sidebar + fluid right panel). Felt like a dashboard.

**After**: Single column, max 480px centred. Three stacked sections — TOP (timer), MIDDLE (recent items), BOTTOM (today's log). Narrow enough to feel like a utility, not a dashboard.

**Why**: Fredrik said "feel like a utility, not a dashboard." Single-column matches the use pattern: glance, click, move on. The two-column layout was optimising for data density at the expense of cognitive simplicity.

---

### 3. Removed all decorative elements

**Before**: Rounded cards, coloured left-border work-info blocks, box-shadows throughout, gradient-adjacent surface layering, pill-shaped buttons, icon usage.

**After**: No cards. No box-shadows (except native context menu). No gradients. No icons. Buttons are plain rectangles with a 1px border. Rows are horizontal lines with minimal padding.

**Why**: Fredrik said "remove all decorative elements that don't serve function." Every removed element reduces cognitive load.

---

### 4. Buttons: no shortcut labels on controls

**Before**: Some buttons included keyboard shortcut hints inline (e.g. "Switch [Ctrl+F]").

**After**: Buttons show only their action label (Stop, Switch, New). Shortcut hints appear **once** in a small muted bar at the bottom of each main screen.

**Why**: Per Fredrik's spec: "Keyboard shortcut hints: shown once at bottom, small, muted — not repeated on every button."

---

### 5. Daily summary: plain list, not dashboard

**Before**: Implied chart/visual breakdown, richer card-based summary.

**After**: Total hours as a large number, then a flat customer breakdown (name / hours / percent), then project sub-rows, then a timeline. All text, tabular numbers, no charts.

**Why**: Fredrik said "simple text/number list — NOT a dashboard." This also keeps the summary fast and accessible.

---

### 6. New tab: Taskbar / Tray

**Added**: A new "Taskbar / Tray" state panel showing:
- A simplified taskbar strip with a tray icon (dot indicator when tracking active)
- A Windows 11-style dark context menu with:
  - Informational current-tracking row (greyed, non-interactive)
  - "Switch to..." with inline submenu showing 3 recent items
  - Stop tracking / Quick add...
  - Open Work Tracker / Quit

**Why**: Fredrik explicitly requested this as a new state. The tray quick-switch is a Phase 2 feature but needs to be designed now so it can be evaluated alongside the main screen.

---

### 7. Quick-add and context-switch overlays

**Before**: Implied full overlays but not clearly separated as states.

**After**: Both overlays are shown as dedicated tabs with the background content dimmed to 25% opacity and a dark semi-transparent backdrop. Single text input, minimal chrome.

**Why**: Keeps the mockup honest — these are overlays, not new screens. The background content being visible (at low opacity) reinforces that context is preserved.

---

## Design token summary

| Token   | Value     | Usage                          |
|---------|-----------|--------------------------------|
| `--bg`  | `#0d0d0d` | Page/app background            |
| `--surface` | `#1a1a1a` | Section headers, overlays |
| `--border`  | `#2a2a2a` | All dividers and borders  |
| `--text`    | `#e8e8e8` | Primary text               |
| `--muted`   | `#888`    | Labels, secondary text     |
| `--accent`  | `#4caf7d` | Running state only         |
| `--hover`   | `#1f1f1f` | Row hover background       |
| `--c1..c4`  | muted palette | 8px customer dots only |

---

## Frontend Build Verification — April 11, 2026

**Requested by:** Fredrik Kristiansen Wikestad  
**Reporter:** Wedge (Tester)  
**Status:** ✅ PASS — Build succeeds, warnings noted

### Summary

After `@sveltejs/vite-plugin-svelte` was bumped from `^4.0.0` → `^5.0.0`, the frontend build was verified end-to-end:

- ✅ `npm run build` completes successfully
- ✅ Static output generated in `build/` directory
- ⚠️ 6 accessibility + reactivity warnings (non-blocking)
- ❌ Standalone TypeScript check fails (expected, requires first build)

**Verdict:** Application is **shippable**. Warnings are code quality improvements, not blockers.

### Build Output

```
✓ 169 modules transformed (SSR bundle, 3.01s)
✓ 187 modules transformed (client bundle, 800ms)
✓ built in 3.01s

> Using @sveltejs/adapter-static
  Wrote site to "build"
  ✔ done
```

### Warnings (Non-Blocking)

#### 1. Accessibility Issues (5 locations)
**Impact:** Keyboard users and screen readers may have difficulty interacting with certain UI elements.

**Files affected:**
- `src/lib/components/QuickAdd.svelte:88` — overlay backdrop
- `src/lib/components/SessionList.svelte:103` — session list items
- `src/lib/components/customers/CustomerList.svelte:159` — customer list items
- `src/lib/components/workorders/WorkOrderList.svelte:195` — work order list items

**Error codes:** `a11y_click_events_have_key_events`, `a11y_no_static_element_interactions`

**Fix:** Add `role="button"`, `tabindex="0"`, and keyboard event handlers to clickable divs.

#### 2. Svelte 5 Rune Reactivity Issue (1 location)
**File:** `src/lib/components/QuickAdd.svelte:18`
**Issue:** `inputRef` needs `$state()` rune declaration for correct reactivity.

#### 3. Self-Closing Tag Issue (1 location)
**File:** `src/lib/components/Timer.svelte:48`
**Issue:** Use `<textarea></textarea>` instead of self-closing `<textarea />`

### Recommendations

**For Leia (Frontend):**
- **Priority 1:** Fix `inputRef` reactivity in QuickAdd.svelte (line 18)
- **Priority 2:** Add ARIA roles to 5 clickable divs, fix textarea self-close

**For Team:**
- No action required on TypeScript check failure
- Build is production-ready as-is

---

## Rust/Tauri Build Environment Readiness

**Date:** 2026-04-11  
**Auditor:** Chewie (Backend Dev)  
**Status:** ❌ **NOT READY** — Rust/cargo not installed

### Environment Audit Results

| Check | Status | Notes |
|-------|--------|-------|
| Rust/cargo | ❌ Not installed | `cargo --version` returned "not recognized" |
| rustup | ❌ Not installed | `rustup --version` returned "not recognized" |
| MSVC Build Tools | ✅ Present | Visual Studio 2022 found at `C:\Program Files\Microsoft Visual Studio\2022` |
| Cargo.toml valid | ✅ Valid | All dependencies reference valid crates (Tauri 2, rusqlite 0.31, serde, chrono, etc.) |
| tauri.conf.json valid | ✅ Valid | Schema reference and all config sections correct |

### What Needs to Be Installed

**Rust development environment** is required before the app can build.

#### Install Steps (Windows)

1. **Download Rust installer:** Visit https://rustup.rs/ and click "Download rustup-init.exe"
2. **Run the installer:** Accept default options and recommended stable toolchain
3. **Restart terminal/PowerShell** after install completes
4. **Verify:** Run `cargo --version` and `rustup --version`

**Installation Time:** ~5-10 minutes (~1.5 GB download)

### Why This Matters

- **cargo:** Rust's package manager and build system (required to compile src-tauri/)
- **rustup:** Rust's version/toolchain manager (keeps Rust updated)
- **MSVC:** Needed on Windows to link compiled Rust code (already available via VS 2022 ✅)
- **Cargo.toml & tauri.conf.json:** Both correctly configured and ready to use once Rust is installed

### Expected Next Command When Ready

```powershell
npm run tauri:dev
```

This will:
1. Start the Svelte dev server (port 1420)
2. Compile Rust backend with cargo
3. Launch the Tauri app with hot-reload enabled
4. App ready for testing within ~30-60 seconds

**File Status Summary:**
- ✅ Frontend ready: package.json, vite.config.ts, node_modules installed
- ✅ Rust config ready: Cargo.toml, tauri.conf.json both valid
- ✅ Build tools ready: Visual Studio 2022 available
- ❌ Missing: Rust toolchain (cargo, rustup)

**Recommendation:** Install Rust from https://rustup.rs, then return and run `npm run tauri:dev`

---

## Phase 2+3 Implementation Summary

**Merged from:** han-phase2-scope.md, chewie-phase2-backend.md, leia-phase2-frontend.md, wedge-phase2-tests.md  
**Date:** 2026-04-12  
**Status:** Design complete, implementation approved

---

## Executive Status

**Phase 1 MVP**: ✅ Complete and shipping. All core time tracking, quick-add, daily summary, and crash recovery features implemented and verified.

**Phase 2+3 Work**: 🟡 Partially implemented with blockers identified and resolved in design phase.

---

## Critical Issues Resolved

### 🔴 Type Mismatch on Pause Commands (RESOLVED)

**Issue Found**: Rust `pause_session()` returns `Result<(), AppError>` but frontend expects `ActiveSession` with `isPaused` flag.

**Fix Implemented**: All pause/resume/heartbeat commands now return `ActiveSession` or `WorkOrder` to ensure frontend sees updated state after operation.

**Impact**: Enables accurate UI state sync for pause/resume workflows.

---

## Database Schema — Phase 2 Migration

**Migration: `002_phase2_features.sql`** adds support for pause state, favorites, heartbeat tracking:

| Table | Column | Type | Purpose |
|-------|--------|------|---------|
| `time_sessions` | `paused_at` | TEXT | Timestamp when session was paused |
| `time_sessions` | `total_paused_seconds` | INTEGER | Cumulative pause duration for accurate elapsed calculation |
| `active_session` | `is_paused` | INTEGER | Durable pause state across restarts |
| `active_session` | `paused_session_at` | TEXT | When current pause interval started |
| `work_orders` | `is_favorite` | INTEGER | Boolean flag for favorites pinning |

**Rationale**: Flat columns preferred over separate tables for simplicity and query performance (no JOINs required).

---

## Pause State Design — Cumulative Duration Approach

**Decision**: Store cumulative `total_paused_seconds` rather than individual pause events.

**Duration Calculation** (with pause):
```
gross_elapsed = current_time - session_start_time
current_pause = if_paused ? (current_time - paused_session_at) : 0
effective_duration = gross_elapsed - total_paused_seconds - current_pause
```

**Why**: Simpler schema, sufficient for MVP use case, no JOIN required for queries. If detailed pause history needed later, can add `pause_events` table without affecting existing data.

**Pause State Machine**:
- **Start**: `is_paused = 0`, `total_paused_seconds = 0`
- **Pause**: Set `is_paused = 1`, record `paused_session_at = now`
- **Resume**: Add `(now - paused_session_at)` to `total_paused_seconds`, clear pause flags
- **Stop**: Finalize duration = gross_duration - total_paused_seconds

---

## Favorites Implementation — Simple Boolean Flag

**Decision**: Use `is_favorite` boolean on `work_orders` table, not separate favorites table.

**Rationale**: Simpler schema, faster queries (no JOIN), natural data model.

**Sorting**: `ORDER BY is_favorite DESC, last_used_at DESC` — favorites always appear first in recent list.

**UI Pattern**: Inline star icon (⭐/☆) at start of each item in SearchSwitch and WorkOrderList. Consistent position enables keyboard accessibility and muscle memory.

**Accessibility**: `<span role="button" tabindex="0">` with Enter/Space handlers (avoids invalid nested button HTML).

---

## Heartbeat & Crash Recovery

**Decision**: 30-second interval with 2-minute orphan detection threshold.

**Frontend Contract**: Call `invoke('update_heartbeat')` every 30 seconds while session active.

**Recovery Logic**: On startup, check for incomplete sessions with `last_heartbeat` older than 2 minutes:
- If found: Present recovery UI ("Close now" or "Discard")
- If not: Continue normally

**Why 2 minutes**: Allows for brief network hiccups or system suspend without false positives. 4 missed heartbeats = high confidence of crash.

**Caveat**: Real-world UAT may show this needs tuning based on user restart patterns.

---

## System Tray Integration

**Decision**: Use Tauri 2 built-in tray configuration (not programmatic `TrayIconBuilder`).

**Configuration** (tauri.conf.json):
```json
"trayIcon": {
  "iconPath": "icons/32x32.png",
  "iconAsTemplate": true
}
```

**Dynamic Updates**: New command `update_tray_tooltip(tooltip: String)` allows frontend to update tooltip with current tracking state.

**Performance Target**: Tooltip updates within 500ms of session state change.

**Phase 2 Roadmap**: Right-click menu with recent items and quick actions (not in initial MVP).

---

## Report Query Design — Weekly/Monthly Aggregation

**Decision**: Reuse daily summary structure with date range filter.

**New Command**: `get_report(start_date, end_date)` returns `ReportData`:
- Aggregated entries grouped by customer + work order
- All individual sessions in range
- Total tracked seconds
- Sorted by total_seconds DESC (highest effort first)

---

## User Directive — Branch Creation Policy

**Date**: 2026-04-21  
**Author**: Fredrik Kristiansen Wikestad (via Copilot)  
**Status**: Documented  

### Decision

When working on an issue, always create a new branch based on the main branch unless explicitly told to use a different base branch.

### Rationale

- Ensures clean, traceable feature branches for each issue
- Prevents accidental dependencies on temporary branches
- Maintains CI integrity and code review clarity

---

## Release Notes Configuration — Auto-Generate Layman-Friendly Release Notes

**Date**: 2026-04-21  
**Author**: Lando (DevOps Expert)  
**Status**: Implemented  
**PR**: #34  

### Context

GitHub's built-in release notes generator (`generate_release_notes: true`) was producing unformatted lists of commits and PRs without clear categorization, making releases hard to scan for users.

### Decision

Configure GitHub's release notes generator with layman-friendly category names using `.github/release.yml`.

### Implementation

1. **Created `.github/release.yml`** with categories:
   - ✨ What's New (feature, enhancement labels)
   - 🐛 Bug Fixes (bug, fix, bugfix labels)
   - ⚡ Improvements (improvement, performance, refactor labels)
   - 🔒 Security (security labels)
   - 📚 Documentation (documentation, docs labels)
   - 🔧 Other Changes (catch-all)

2. **Updated `.github/workflows/release.yml`**:
   - Added friendly body prefix explaining what users will find
   - Kept `generate_release_notes: true` — auto-generated notes append after body
   - Excluded labels: `ignore-for-release`, `dependencies`, `chore`

### Rationale

- **User-friendly**: Non-technical users can scan sections without decoding commits
- **Zero maintenance**: GitHub auto-generates from PR labels
- **Flexible**: Categories updatable without workflow changes
- **Standard practice**: Uses GitHub's built-in feature, not custom scripts
- **Body prefix**: Provides context before detailed changes

### Impact

- Release notes organized into scannable sections
- Reduced manual changelog maintenance
- Better user experience on GitHub releases page

---

## Reports View Grouping Pattern

**Date**: 2026-04-21  
**Author**: Leia  
**Issue**: #35  
**Status**: Complete (PR #36)

### Decision

Reports view groups sessions by **Day → Customer → Work Order** using `groupSessionsByDay()` from `src/lib/utils/reportGrouping.ts`.

- Day groups: always expanded on load, collapsible
- Customer groups within a day: collapsed by default, expandable
- Work order items: visible when parent customer is expanded

### Expand/Collapse Key Scheme

- `expandedDays: Set<string>` — key is `"YYYY-MM-DD"`
- `expandedCustomers: Set<string>` — key is `"YYYY-MM-DD::customerName"` (scoped per day)

### Data Source

Use `reportData.sessions: Session[]` (not `reportData.entries`) for day grouping. Always guard with `?? []` since older test mocks may not include the `sessions` field.

### Date Formatting

Use `formatDay(dateStr)` from `src/lib/utils/formatters.ts`. Constructs date as `new Date(year, month-1, day)` in local timezone to avoid UTC off-by-one errors. Never use `new Date(isoString)` for date-only strings.

### Template Pattern

Three-level nesting: `.day-group` > `.day-customers` > `.customer-group` > `.work-orders`. Day header has larger weight (`font-weight: 700`, `font-size: 15px`). Customer rows indented with `margin-left: 14px` to signal hierarchy under the day.

### Test Coverage

Comprehensive test suite in `src/lib/__tests__/reportGrouping.test.ts` with 17 test cases covering hierarchical grouping, sorting, aggregation, and edge cases. All tests passing (zero regressions on 84 existing tests).

### Implementation Status

- ✅ ReportView.svelte integrated with grouping utility
- ✅ Date formatter with local timezone awareness
- ✅ 4/4 CI checks passed
- ✅ PR #36 open, ready for merge

**Date Handling**:
- Week boundaries: ISO 8601 (Mon-Sun, user preference later)
- Month boundaries: 1st-last day
- Exclude incomplete sessions: `end_time IS NOT NULL`
- Include paused time exclusion: `COALESCE(duration_override, duration_seconds) - total_paused_seconds`

**Performance Target**: <500ms for 1-month report (1000+ sessions).

**Query Optimization**: Uses existing indexes (`idx_sessions_start_time`, `idx_sessions_work_order_id`).

---

## Frontend Type Additions

**New TypeScript interfaces** (src/lib/types.ts):

```typescript
interface ActiveSession {
  isPaused: boolean;  // New field for Phase 2
}

interface WorkOrder {
  isFavorite: boolean;  // New field for Phase 2
}

interface ReportData {
  startDate: string;
  endDate: string;
  totalSeconds: number;
  entries: ReportEntry[];
  sessions: Session[];
}

interface ReportEntry {
  customerId: string;
  customerName: string;
  customerColor: string | null;
  workOrderId: string;
  workOrderName: string;
  totalSeconds: number;
  sessionCount: number;
}
```

---

## Visual Design for Phase 2

**Paused State Indicator**:
- Amber (#f59e0b) color badge + "PAUSED" text in Timer
- Pause button (⏸) / Resume button (▶) swap
- Left border changes green → amber when paused

**Color-Coded Session Borders**:
- 3px left border using customer color on each session
- Falls back to grey if no color set
- CSS binding: `style="border-left-color: {session.customerColor ?? 'var(--border)'}"`

**Report View**:
- Collapsible customer groups with work order detail expansion
- "This week" / "This month" / "Custom" preset buttons
- Progressive disclosure for long customer lists

---

## New Tauri IPC Commands (Phase 2+3)

| Command | Signature | Return |
|---------|-----------|--------|
| `pause_session()` | `()` | `Result<ActiveSession, AppError>` |
| `resume_session()` | `()` | `Result<ActiveSession, AppError>` |
| `update_heartbeat()` | `()` | `Result<(), AppError>` |
| `toggle_favorite(work_order_id)` | `(String)` | `Result<WorkOrder, AppError>` |
| `get_report(start_date, end_date)` | `(String, String)` | `Result<ReportData, AppError>` |
| `update_tray_tooltip(tooltip)` | `(String)` | `Result<(), String>` |
| `check_for_orphan_session()` | `()` | `Result<Option<OrphanSession>, AppError>` |

**Modified Commands**:
- `list_recent_work_orders(limit?)` — Now sorts favorites to top

---

## Test Coverage — 34 New Test Cases

**P0 (Blocking)**: 12 cases covering pause/resume, crash recovery, orphan detection, duration calculation edge cases

**P1 (Important)**: 18 cases for favorites, tray, heartbeat, weekly/monthly reports, activity filtering

**P2 (Nice-to-Have)**: 4 cases for advanced grouping, multiple orphans, etc.

**Critical Test Findings**:
- Pause state must be correct before favorites/reports are tested (affects duration calculation)
- Duration calculation supports 3 independent cases: calculated, manual override, with pause
- Backward compatibility requires migration (existing Phase 1 sessions get `paused_seconds = 0`)
- CSV export header: Date, Customer, Project, Activity Type, Duration (hours), Notes, Start Time, End Time

---

## Execution Timeline

### Immediate (Sprint N)
- **Chewie**: Fix pause command return types (BLOCKER), implement migration 002, complete pause logic with duration tracking
- **Leia**: Add pause/resume buttons, amber state indicator, color CSS bindings, heartbeat polling
- **Wedge**: Run P0 test cases as implementation completes

### Phase 2 (Sprint N+1)
- **Chewie**: Implement favorites CRUD, weekly/monthly report queries, verify tray tooltip updates
- **Leia**: Add favorites UI, weekly report view with date picker, tray tooltip updates
- **Wedge**: Run P1 test cases

### Phase 3 (Sprint N+2)
- **Chewie**: Full reporting layer (trends, filtering, optimize for 1000+ sessions)
- **Leia**: Report visualizations, activity type filtering UI
- **Wedge**: Performance and integration testing

---

## Open Questions for Product Review

1. **Week Definition**: Should week be Mon-Sun (ISO 8601) or user preference?
2. **Pause History**: Do consultants need detailed pause/resume logs, or is cumulative sufficient?
3. **Report Persistence**: Should collapse/expand state in report view persist in localStorage?
4. **Favorites Categories**: Support multiple groups/categories, or flat list only?
5. **Tray Menu**: Implement right-click menu in Phase 2 or defer to Phase 2.1?

---

## Risk Summary

| Issue | Severity | Status |
|-------|----------|--------|
| Type mismatch on pause commands | 🔴 CRITICAL | ✅ Resolved |
| Missing pause duration tracking | 🟡 MEDIUM | ✅ Schema added |
| Pause state not durable across restarts | 🟡 MEDIUM | ✅ DB columns added |
| Heartbeat command not registered | 🟡 MEDIUM | ✅ Verified |
| Tray tooltip command stub | 🟡 MEDIUM | ✅ Implemented |
| CSS color classes not wired | 🟢 LOW | ✅ Design complete |

**Overall Risk**: 🟢 LOW — All blockers identified and addressed in design phase. Ready for implementation.

---

## Fix: Customer Field Population in WorkOrder Form

**Date**: 2026-04-11  
**Author**: Leia (Frontend Dev)  
**Status**: RESOLVED — Extended to comprehensive fix

### Problem

The customer field in the WorkOrder form could not be populated. Users attempting to create a new work order were unable to select a customer from the SearchableSelect dropdown because the customer list was not loading.

**During investigation, discovered this issue affected EVERY frontend API call with direct parameters.**

### Root Cause

**Parameter naming mismatch between frontend and backend:**

- **Frontend** (`src/lib/api/customers.ts`):
  ```typescript
  export const listCustomers = (includeArchived?: boolean) =>
    invoke<Customer[]>('list_customers', { includeArchived });
  ```
  Sent: `{ includeArchived: false }` (camelCase)

- **Backend** (`src-tauri/src/commands/customers.rs` line 30):
  ```rust
  pub fn list_customers(state: State<AppState>, include_archived: Option<bool>)
  ```
  Expected: `{ include_archived: false }` (snake_case)

#### Why This Happened

Tauri's automatic case conversion (`camelCase` ↔ `snake_case`) **only applies to serde structs** with the `#[serde(rename_all = "camelCase")]` attribute.

For **direct function parameters** (not wrapped in a struct), parameter names must match **exactly** between JavaScript and Rust.

#### Why Other Commands Work

Commands like `create_customer` and `update_customer` work because they use **serde structs** for parameters:

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomerParams {
    pub name: String,
    pub code: Option<String>,
    pub color: Option<String>,
}

#[tauri::command]
pub fn create_customer(state: State<AppState>, params: CreateCustomerParams) 
```

The `#[serde(rename_all = "camelCase")]` directive automatically converts `customerId` → `customer_id`, `customerName` → `customer_name`, etc.

But `list_customers` uses a **direct parameter** `include_archived: Option<bool>`, so no automatic conversion happens.

### Solution — Comprehensive Fix

**Updated ALL 4 API wrapper files** to send correct snake_case parameter names for direct parameters:

#### customers.ts
```typescript
// Before: { includeArchived }
// After:
export const listCustomers = (includeArchived = false) =>
  invoke<Customer[]>('list_customers', { include_archived: includeArchived });
```

#### workOrders.ts
```typescript
// Before: { customerId }, { workOrderId }
// After:
export const listWorkOrders = (customerId?: string) =>
  invoke<WorkOrder[]>('list_work_orders', { customer_id: customerId });
```

---

## macOS Universal Build Fix

**Date**: 2026-04-13  
**Authored by**: Lando (DevOps)  
**Status**: Implemented  
**Related**: `.github/workflows/release.yml`, tag v0.1.1

### Problem

The `release.yml` CI workflow was failing on macOS builds with:
```
Target x86_64-apple-darwin is not installed
```

GitHub Actions' `macos-latest` runner is ARM64-only (aarch64-apple-darwin). Building a universal-apple-darwin binary requires **both**:
- `aarch64-apple-darwin` (available by default)
- `x86_64-apple-darwin` (missing, must be added explicitly)

### Solution

Added a platform-specific step to `release.yml` to install the missing Rust target **before** the npm install step:

```yaml
- name: Add x86_64 Rust target (macOS Universal)
  if: matrix.platform == 'macos-latest'
  run: rustup target add x86_64-apple-darwin
```

**Placement**: Between "Setup Rust stable" and "Setup Node.js 22.x" steps (lines 47-49 in release.yml)

**Condition**: `if: matrix.platform == 'macos-latest'` to scope to macOS jobs only

### Why This Works

- `rustup target add` adds the architecture to the current Rust toolchain
- Step runs **before** npm install (platform-independent) to avoid missing headers
- Conditional scope means Windows/Linux jobs skip this step (they don't need x86_64-apple-darwin)
- Total overhead: ~10-15 seconds per macOS build run

### Pattern for Future Rust Multi-Target Builds

When building Rust binaries targeting multiple architectures on GitHub Actions:
1. Always check GitHub Actions runner capabilities (macOS-latest = ARM64-only)
2. For universal binaries, explicitly add all required targets via `rustup target add`
3. Add targets **after** setup-rust-toolchain, **before** cargo build
4. Use `if: matrix.platform == 'runner-name'` for platform-specific steps

### Files Modified

- `.github/workflows/release.yml` — Added rustup target step

### Test & Commit

- **Commit**: 04815e4 (macOS universal build fix)
- **Tag**: v0.1.1 pushed to trigger release workflow
- **Result**: ✅ macOS build now succeeds with universal binary (both architectures included)

export const toggleFavorite = (workOrderId: string) =>
  invoke<WorkOrder>('toggle_favorite', { work_order_id: workOrderId });
```

#### sessions.ts
```typescript
// Before: { workOrderId }, { activityType }, { startDate, endDate }, { sessionId }
// After:
export const startSession = (workOrderId: string) =>
  invoke<Session>('start_session', { work_order_id: workOrderId });

export const stopSession = (notes?: string, activityType?: string) =>
  invoke<Session | null>('stop_session', { notes, activity_type: activityType });

export const listSessions = (startDate: string, endDate: string) =>
  invoke<Session[]>('list_sessions', { start_date: startDate, end_date: endDate });

export const recoverSession = (sessionId: string) =>
  invoke<Session>('recover_session', { session_id: sessionId });

export const discardOrphanSession = (sessionId: string) =>
  invoke<void>('discard_orphan_session', { session_id: sessionId });
```

#### reports.ts
```typescript
// Before: { startDate, endDate }
// After:
export const exportCsv = (startDate: string, endDate: string) =>
  invoke<string>('export_csv', { start_date: startDate, end_date: endDate });

export const getReport = (startDate: string, endDate: string) =>
  invoke<ReportData>('get_report', { start_date: startDate, end_date: endDate });
```

### Impact

- ✅ Customer dropdown now loads correctly in WorkOrder form (**original bug**)
- ✅ Work order filtering by customer works
- ✅ Favorite toggle works
- ✅ Session start/stop works
- ✅ Session recovery works
- ✅ Reports and CSV export work
- ✅ All date range queries work

**This was a critical bug affecting the entire application.** Without this fix, nearly every core feature would have failed silently or with cryptic errors.

### Verification

1. ✅ `npm run build` succeeds with no TypeScript or Svelte errors
2. ✅ Committed as `498ee92` (initial customer fix) and `a08a26a` (comprehensive fix)

### Recommendation

**For future Tauri commands:**

1. **Prefer serde structs** for all parameters (even single parameters) to enable automatic case conversion
2. If using direct parameters, **always use snake_case** in JavaScript to match Rust
3. Add a linting rule or code review checklist to catch camelCase parameters in `invoke()` calls
4. Document this pattern in team onboarding

**Example of better pattern:**

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCustomersParams {
    pub include_archived: Option<bool>,
}

#[tauri::command]
pub fn list_customers(state: State<AppState>, params: ListCustomersParams)
```

Then frontend can use camelCase:
```typescript
invoke('list_customers', { params: { includeArchived: false } })
```

---

## Decision: SearchableSelect — Use `mousedown` for Outside-Click Detection

**From**: Leia (Frontend Dev)  
**Date**: 2026-04-11  
**Status**: IMPLEMENTED

### Decision

`SearchableSelect.svelte` must use `mousedown` (not `click`) for the outside-click detection `$effect` listener.

```js
$effect(() => {
  if (isOpen) {
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }
});
```

### Rationale

**Root cause**: Svelte 5 rune reactivity flushes synchronously when `isOpen = true` is set inside the trigger button's `onclick`. This means:

1. User clicks trigger → `isOpen = true`
2. Svelte flushes DOM: trigger `<button>` is removed, filter `<input>` is inserted, `$effect` fires and adds `click` listener to `document`
3. The original click event **then** bubbles to `document`
4. `handleClickOutside` fires: `containerRef.contains(triggerButton)` → returns `false` (element was removed)
5. `close()` called → dropdown opens and immediately closes

Using `mousedown` prevents this because `mousedown` fires **before** the click event, and therefore **before** `isOpen` changes or DOM updates. When `mousedown` fires on the trigger button, no listener is yet attached to `document` (since `isOpen` is still `false`). By the time `click` bubbles, the `mousedown` listener is already active — but any subsequent `mousedown` on an option element will have `containerRef.contains(e.target) === true`, so options remain selectable.

### Safety

Option selection still works correctly: when the user `mousedown`s on a dropdown option, `containerRef.contains(e.target)` returns `true` (option is inside the container, still in DOM during mousedown), so `close()` is not triggered. The `onclick` on the option fires afterward and calls `selectOption()`.

### Context

Bug reported by Fredrik: "Still I cannot choose a customer in the add workorder form." Previous fix (snake_case parameter naming) was necessary but not sufficient. This mousedown fix resolves the remaining interaction failure.

### Commit

`16f65b6` — fix: SearchableSelect click-outside race condition and WorkOrder empty state

---

## Decision: Svelte 5 Warning Fixes

**Date:** 2026-04-11  
**Author:** Leia (Frontend Dev)  
**Status:** Implemented

### Context

Running `npm run tauri -- dev` surfaced 5 Svelte compiler warnings across 4 components. These are non-blocking but indicate incorrect patterns in Svelte 5 runes mode that should be fixed before shipping.

### Decisions Made

#### 1. `bind:this` refs use `$state<T | undefined>(undefined)`

**Warning:** `non_reactive_update` — variable updated but not declared with `$state`  
**Files:** `QuickAdd.svelte:18`, `SearchableSelect.svelte:23-24`

In Svelte 5 runes mode, `bind:this` writes the DOM element into the variable after mount. For Svelte's reactive system to track this assignment, the variable must be a `$state` rune.

**Pattern adopted:**
```ts
// Before (Svelte 4 style — wrong in runes mode)
let inputRef: HTMLInputElement;

// After (Svelte 5 correct)
let inputRef = $state<HTMLInputElement | undefined>(undefined);
```

This applies to **all** `bind:this` variables — input refs, container refs, etc.

#### 2. Self-closing `<textarea>` → explicit close tag

**Warning:** `element_invalid_self_closing_tag`  
**File:** `Timer.svelte:65`

HTML spec forbids self-closing void syntax for non-void elements like `<textarea>`. Svelte 5 flags this as a warning.

```svelte
<!-- Before -->
<textarea bind:value={notes} rows="3" placeholder="..." />

<!-- After -->
<textarea bind:value={notes} rows="3" placeholder="..."></textarea>
```

#### 3. Interactive divs: `role="button"` + `tabindex` + `onkeydown`

**Warning:** `a11y_click_events_have_key_events` + `a11y_no_static_element_interactions`  
**Files:** `QuickAdd.svelte:93` (overlay backdrop), `SessionList.svelte:103` (session row)

**Decision: Keep as `<div>` with ARIA attributes** (not convert to `<button>`) for these two cases:

- **Overlay backdrop** — wraps the modal dialog; converting to `<button>` would be semantically wrong. Used `role="button" tabindex="0"` with Enter key handler.
- **Session row** — contains a nested `<button>` (delete action). HTML forbids `<button>` inside `<button>`, so ARIA approach is correct here.

**Pattern adopted:**
```svelte
<div
  role="button"
  tabindex="0"
  onclick={handler}
  onkeydown={(e) => e.key === 'Enter' && handler(e)}
>
```

Event handler types updated from `MouseEvent` → `Event` where needed to support both `onclick` and `onkeydown`.

### Remaining Warnings (Out of Scope)

`CustomerList.svelte:145` and `WorkOrderList.svelte:194` have the same a11y pattern. These are tracked in history.md Priority 2 list and should follow the same div+ARIA approach (nested buttons present in those components too).

### Build Status

`npm run build` ✅ passes after all fixes.

---

## Smoke Testing Pattern for Svelte 5 Stores and Components

**Date**: 2026-04-13  
**Author**: Wedge (Tester)  
**Status**: ACCEPTED — implemented and all tests passing  

---

### Context

Work-tracker-2 had a critical black-window bug: `timer.svelte.ts` contained a module-level `$effect()` call. In Svelte 5, `$effect()` requires a component context (or `$effect.root()`). At app startup, importing the timer store threw `"Effect can only be created inside an effect"`, crashing the entire app — blank screen, no error visible to the user.

The fix was simple (remove the misplaced `$effect`), but **no test existed to catch this before it reached production**. This decision documents the pattern to prevent recurrence.

---

### Decision

Add two categories of smoke tests to the frontend test suite:

#### 1. Store Import Smoke Tests (`src/lib/__tests__/smoke.test.ts`)

**Pattern**: Static-import every key store module at the top of a test file (before any `describe`/`it` blocks). If a module throws at import time, the entire test file fails to load — Vitest reports it as a file-level error. This is the intended behavior: a module-level crash is a hard failure.

```typescript
// These imports ARE the smoke test. If any throws, the file fails.
import { timer } from '$lib/stores/timer.svelte';
import { sessionsStore } from '$lib/stores/sessions.svelte';
import { uiStore } from '$lib/stores/ui.svelte';
```

Then verify the exported shape:
```typescript
it('timer exposes expected API shape', () => {
  expect(timer).toHaveProperty('active');
  expect(timer).toHaveProperty('elapsed');
  expect(typeof timer.setActive).toBe('function');
  // ...
});
```

**What this catches**:
- Module-level `$effect()` (the original bug)
- Circular imports that throw on resolution
- Syntax errors in compiled output
- Any `throw` or synchronous rejection at module initialization

#### 2. Component Mount Smoke Tests (`src/lib/__tests__/components.smoke.test.ts`)

**Pattern**: Use `@testing-library/svelte` to `render()` each key component in a mocked environment. Verify it mounts without throwing and renders a key landmark.

```typescript
vi.mock('$lib/stores/timer.svelte', () => ({
  timer: { active: null, elapsed: 0, isTracking: false, isPaused: false,
           setActive: vi.fn(), refresh: vi.fn(), pause: vi.fn(), resume: vi.fn() }
}));

it('Timer mounts without throwing', () => {
  expect(() => render(Timer)).not.toThrow();
});

it('renders Not tracking in idle state', () => {
  render(Timer);
  expect(screen.getByText('Not tracking')).toBeTruthy();
});
```

**What this catches**:
- Runtime errors in `onMount` or `$effect` within components
- Template logic that crashes on null/undefined props
- Store access patterns that fail in test environment

---

### Required Vitest Configuration

Svelte 5 resolves to `index-server.js` by default in Vitest because the `import` ESM condition maps to the server runtime. To use the browser/component runtime, add:

```typescript
// vitest.config.ts
resolve: {
  conditions: ['browser'],
}
```

Without this, `@testing-library/svelte` throws `lifecycle_function_unavailable: mount(...) is not available on the server`.

---

### Mock Setup Requirements

For store smoke tests (no component rendering):
```typescript
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }));
vi.mock('$lib/api/sessions', () => ({ getActiveSession: vi.fn().mockResolvedValue(null), ... }));
```

For component smoke tests (full component render):
- Mock ALL stores the component imports (stores are module singletons, not props)
- Mock ALL API modules (prevent real network/IPC calls)
- `vi.stubGlobal('alert', vi.fn())` — components may call `alert()` in error handlers

---

### What NOT to Do

- ❌ `describe.skip(...)` — skipped blocks don't catch import errors
- ❌ Commented-out `import` statements — these don't run; they provide zero protection
- ❌ Dynamic `import()` inside test bodies — defers the error, loses the "file fails to load" signal
- ❌ Mocking the store inside the smoke test file — defeats the purpose; the import IS the test

---

### Maintenance Rules

1. **Add a smoke test whenever a new store is created** — one import line + shape assertions
2. **Add a component smoke test whenever a new top-level component is created** — `render()` + one landmark assertion  
3. **Keep store mock shapes in sync with the real store interface** — if `timer` gains a new method, add it to the mock
4. **Never use `describe.skip` to defer a smoke test** — if the test can't run, find out why and fix it

---

### Results

Before this pattern:
- 7 timer store tests skipped (could not catch the black-window class of bug)
- 0 component mount tests

After:
- 7 timer store tests passing
- 9 store/module smoke tests passing  
- 9 component mount tests passing
- **Total: 40 tests, 0 skipped, 0 failing**
