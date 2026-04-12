# Squad Decisions

## Active Decisions

### 2026-04-11: Instruction Framework Review — Fixed

**From**: Han (Lead)  
**Status**: IMPLEMENTED

Fixed 6 issues in the instruction framework to unblock Phase 1 agent spawning.

#### MUST FIX (Blockers) — All Resolved

1. **Crash Recovery Specification** (copilot-instructions.md, database.instructions.md)
   - Added "Crash Recovery & Durability" section requiring WAL mode
   - Specified immediate write policy: all session writes flushed before UI confirmation
   - Defined startup recovery flow: detect orphan sessions, present recovery UI
   - Added SQLite PRAGMA requirements (journal_mode=WAL, synchronous=NORMAL)
   - **Status**: IMPLEMENTED

2. **Quick-Add Workflow** (all 4 files)
   - Added inline quick-add to Phase 1 scope (Cmd/Ctrl+N overlay)
   - Added Quick-Add Component spec to UI instructions (Section 7)
   - Added `createAndStart` atomic operation to backend spec
   - Quick-add requires only a name, immediately starts tracking
   - **Status**: IMPLEMENTED

3. **Quick-Switch Moved to Phase 1** (copilot-instructions.md)
   - Phase 1 now includes: recent items (last 5-10), search-to-switch
   - Phase 2 retains: favorites/pinning, global hotkey, taskbar menu
   - Rationale documented: "context switching IS the core value prop"
   - **Status**: IMPLEMENTED

#### SHOULD FIX — Addressed

4. **Paused State Definition** (copilot-instructions.md, ui-components.instructions.md)
   - Defined paused state explicitly (timer frozen, session not closed)
   - MVP recommendation: skip pause in Phase 1, use stop instead
   - Phase 1 states: Running/Stopped only
   - Phase 2 adds: Paused (amber indicator)

5. **Required Indexes** (database.instructions.md)
   - Added explicit index definitions for MVP performance:
     - idx_sessions_start_time, idx_sessions_work_order_id, idx_sessions_end_time
     - idx_work_orders_customer_id, idx_customers_name

6. **Performance Targets Harmonized** (ui-components.instructions.md)
   - Changed context switch target from <2s to <3s (matches main framework)
   - All files now consistent: <3s context switch, <100ms timer, <50ms search

---

### 2026-04-11: Technology Stack Decision — Approved

**From**: Han (Lead)  
**Status**: APPROVED

#### Decision: Tauri 2 + Svelte 5 + TypeScript + SQLite

**Rationale**:
1. **Bundle size** — ~10-15MB vs Electron's 150MB+. Easy to distribute, feels lightweight.
2. **Crash safety** — Rust backend won't crash from memory issues. SQLite operations are memory-safe.
3. **Svelte simplicity** — Svelte 5 runes provide reactivity without boilerplate. Solo dev + AI team benefits from less code.
4. **Native feel** — Fast startup, low memory, first-class system tray support.
5. **Offline-first** — SQLite embedded with WAL mode. No network required.

**Tradeoff**: Rust learning curve. Mitigated by small backend (~500-800 LOC) and AI assistance.

**Rejected Alternatives**:
- Electron + React: Too heavy for the value. 150MB for a time tracker is excessive.
- Tauri + React: Viable, but Svelte is simpler for this scope.

---

#### Decision: IPC Pattern — Approved

**Choice**: Tauri commands (JS → Rust → SQLite)

**Pattern**:
```
Frontend (invoke) → Rust command handler → Service logic → SQLite
                 ← Result/Error          ←              ←
```

**Key commands**:
- `start_session(work_order_id)` — Atomic switch (stops old, starts new)
- `stop_session(notes?, activity_type?)` — Finalizes active session
- `quick_add(...)` — Creates customer/work order and starts tracking in one call
- `get_daily_summary(date)` — Returns totals by customer/work order

**Rationale**: Tauri's command pattern is idiomatic, type-safe (via TypeScript bindings), and simple. Each command is a discrete operation with clear inputs/outputs.

---

#### Decision: State Management — Approved

**Choice**: Svelte 5 runes in module stores

**Pattern**:
```typescript
// stores/timer.svelte.ts
let activeSession = $state<Session | null>(null);
let elapsedSeconds = $state(0);
let isTracking = $derived(activeSession !== null);
```

**Rationale**:
- Svelte 5 runes eliminate boilerplate (no `writable()`, no `.set()`)
- State is reactive by default
- No external dependencies (no Zustand, no Redux)
- Simple modules can be imported where needed

**Store organization**:
- `timer.svelte.ts` — Active session, elapsed time
- `sessions.svelte.ts` — Today's sessions, recents
- `ui.svelte.ts` — Modal states, search query

---

#### Decision: Database Strategy — Approved

**Choice**: SQLite with tauri-plugin-sql + WAL mode

**Schema highlights**:
- UUIDs for all IDs (portable, no auto-increment issues)
- ISO 8601 timestamps stored as TEXT
- `active_session` singleton table for crash recovery
- Soft deletes via `archived_at` column

**Crash recovery**:
1. WAL mode ensures writes survive crashes
2. Heartbeat updates `active_session.last_heartbeat` every 30s
3. On startup: check for orphan sessions, present recovery dialog

**Migrations**: Embedded at compile time, run on first startup.

---

#### Decision: Project Structure — Approved

```
work-tracker-2/
├── src-tauri/           # Rust backend
│   ├── src/commands/    # IPC handlers
│   ├── src/services/    # Business logic
│   ├── src/db/          # SQLite + migrations
│   └── src/models/      # Domain types
├── src/                 # Svelte frontend
│   ├── lib/components/  # UI components
│   ├── lib/stores/      # Reactive state
│   └── lib/api/         # IPC client wrappers
└── docs/                # Architecture docs
```

**Rationale**: Clear three-layer separation. Backend can be tested independently. Frontend components are decoupled from IPC details.

---

#### Decision: Phase 1 MVP Scope — Approved

**In scope**:
- CRUD for customers and work orders
- Quick-add (create + start in one action)
- Start/stop sessions (atomic switching)
- Active timer with real-time display
- Today's work log with inline edit
- Search-to-switch (recent items + filter)
- Daily summary by customer
- CSV export

**Out of scope (Phase 2+)**:
- Pause state
- Favorites/pinning
- System tray quick-switch
- Color coding (Phase 2+ feature, mockup includes for reference)
- Global hotkeys

---

### 2026-04-11: UI Design — Approved

**From**: Leia (Frontend Dev)  
**Status**: APPROVED

#### Key Design Choices

**Theme**: Dark mode as primary design language
- Professional appearance suitable for consultant/business tool
- Reduces eye strain during long working sessions
- Excellent for taskbar/system tray applications
- Better for focus during deep work periods

**Color Palette**:
```
Background:    #1a1d24 (dark slate)
Surface:       #252932 (lighter slate)
Accent:        #3b82f6 (blue - calm, professional)
Success:       #10b981 (green - active tracking state)
Text Primary:  #e5e7eb (light grey)
Text Muted:    #6b7280 (medium grey)

Customer colors (left borders):
- Acme:        #8b5cf6 (purple)
- GlobalTech:  #ec4899 (pink)
- Innovate:    #14b8a6 (teal)
- Others:      #f59e0b (amber)
```

**Layout**: Two-column desktop with sticky left sidebar
- Left Panel (400px): Active timer, controls, quick actions
- Right Panel (fluid): Today's activity log, summaries
- Responsive: collapses to single column below 1024px

**Visual Hierarchy**:
- Priority 1: Active work (large timer, green accent when running)
- Priority 2: Today's activity log (compact rows, color-coded borders)
- Priority 3: Controls and metadata (supporting elements)

**State Indication**: Badge + color coding
- **Running**: Green dot + "Running" badge + green timer
- **Stopped**: Grey badge + grey timer
- **Paused** (Phase 2): Amber badge

**Typography**: System fonts only (offline-first)
- Timer: 3.5rem (56px)
- Headings: 1.25rem (20px)
- Body: 0.875rem (14px)
- Labels: 0.75rem (12px)
- All interactive elements ≥44px (touch-friendly)

**Interaction Patterns**:
- Quick-Add Overlay: Minimal centered overlay, Cmd/Ctrl+N, auto-dismiss on Esc
- Context Switcher: Search-first overlay, keyboard-navigable, shows recent items
- Inline Editing: Click entry to expand inline editor (no modal dialogs)
- Keyboard Shortcuts: Tab, Arrow keys, Enter to confirm, Esc to cancel
- Global Shortcuts: Ctrl+N (quick-add), Ctrl+F (search), Esc (cancel)

**Accessibility**:
- WCAG AA compliant (4.5:1 contrast minimum)
- All interactions keyboard-accessible
- Color coding paired with text labels (not color-only)
- Focus states visible on all interactive elements

---

### 2026-04-11: Backend Scaffold Decision — rusqlite Pattern

**From**: Chewie (Backend Dev)  
**Status**: IMPLEMENTED

#### Decision: Use rusqlite Directly with Mutex<Connection>

The architecture initially mentioned tauri-plugin-sql, however during implementation research, discovered that tauri-plugin-sql is a **JavaScript-side plugin** that exposes SQLite to the frontend via IPC, not a Rust backend database layer.

For the three-layer architecture (Frontend → Service Layer → Data Layer), we need:
- Direct Rust access to SQLite for service logic
- Transaction control for atomic operations
- WAL mode and PRAGMA configuration
- Connection lifecycle management

**Choice**: rusqlite + Mutex<Connection> as Tauri State

**Implementation patterns**:
1. **Atomic Session Switching**: Use `conn.unchecked_transaction()` to stop current session, create new session, update active_session singleton, and update recent_work_orders all in one transaction
2. **Duration Calculation**: Store both `duration_seconds` (calculated) and `duration_override` (user-specified), query with COALESCE
3. **Crash Recovery**: Singleton table `active_session` with `last_heartbeat`, orphan detection on startup
4. **Error Handling**: Custom AppError enum with Tauri serialization
5. **Quick-Add Workflow**: Atomic multi-step transaction (create customer, work order, start session)

**Rationale**:
- Direct control over transactions and WAL mode
- Mutex ensures thread-safe concurrent access from Tauri commands
- No IPC overhead — commands run in-process with direct DB access
- Standard Rust pattern for shared mutable state

**Alternatives rejected**:
- tauri-plugin-sql: JS-side only, no Rust service layer access
- sqlx: Overkill for MVP (async not needed, compile-time checking adds complexity)
- diesel: Too heavy (ORM overhead, schema.rs boilerplate)

---

### 2026-04-11: Phase 1 Backend Implementation Complete

**From**: Chewie (Backend Dev)  
**Status**: IMPLEMENTED

**Deliverables**: 47 files created

**Frontend Config** (6): package.json, vite.config.ts, svelte.config.js, tsconfig.json, app.html, app.css  
**Frontend Source** (9): app.d.ts, routes/, lib/types.ts, lib/api/ (4 wrappers)  
**Rust Config** (5): Cargo.toml, build.rs, tauri.conf.json, capabilities/default.json, icons/  
**Rust Source** (17): Core (3), Models (5), Services (2), Commands (3 modules with 18 commands total), Database migration  

**Key Implementation Achievements**:
- SQLite with WAL mode (crash-safe)
- 5 tables: customers, work_orders, time_sessions, active_session, recent_work_orders
- 11 performance indexes
- 18 Tauri IPC commands with full error handling
- Atomic session switching
- Crash recovery via orphan detection
- CSV export with proper escaping
- TypeScript types matching Rust models

---

### 2026-04-11: Phase 1 Frontend Implementation Complete

**From**: Leia (Frontend Dev)  
**Status**: IMPLEMENTED

**Deliverables**: Complete Svelte 5 frontend

**Stores** (3): timer.svelte.ts (active session + elapsed), sessions.svelte.ts (today's + recent), ui.svelte.ts (modal states)

**Utilities** (2): formatters.ts (duration/time formatting), shortcuts.ts (global keyboard shortcut registration)

**Components** (8):
- Timer.svelte — Active session display with stop controls
- RecoveryDialog.svelte — Orphan recovery on startup
- QuickAdd.svelte — Ctrl+N overlay for create + start
- SearchSwitch.svelte — Recent + search with keyboard nav
- SessionList.svelte — Today's sessions with inline edit
- DailySummary.svelte — Total hours + customer breakdown
- CustomerList.svelte — CRUD for customers
- WorkOrderList.svelte — CRUD for work orders

**Routes** (3): +layout.svelte (app init), +page.svelte (main tracking), manage/+page.svelte (admin + export)

**Key Technical Decisions**:
- Svelte 5 runes exclusively ($state, $derived, $effect) for reactivity
- Dark theme: #0d0d0d background, #4caf7d accent for active state only
- Single-column layout (max-width 480px)
- Keyboard-first: all actions reachable without mouse
- Inline editing, no modal dialogs for reversible actions
- Real-time updates via reactive stores

---

### 2026-04-11: Documentation Complete

**From**: Mon Mothma (Technical Writer)  
**Status**: IMPLEMENTED

**Deliverables**:
- `README.md` — Developer-friendly quickstart (prerequisites, features, keyboard shortcuts, project structure, data paths, dev workflow)
- `docs/api-reference.md` — Comprehensive IPC command reference (18 commands across customers, work orders, sessions, reports) with TypeScript signatures, parameter descriptions, return types, error codes, and realistic usage examples

**Key Achievements**:
- All 18 IPC commands fully documented with typed examples
- Clear error code reference table
- End-to-end workflow example showing complete user journey
- Local-first guarantee clarified
- Crash recovery protocol documented
- Phase 1 scope locked at 16 commands (later extended to 18 with recover/discard commands)

---

### 2026-04-11: Phase 1 Test Plan Complete

**From**: Wedge (Tester)  
**Status**: IMPLEMENTED

**Deliverable**: `docs/test-plan.md` — 118 comprehensive test cases

**Coverage**:
- 65+ backend Tauri commands
- 23+ frontend Svelte components
- 30+ data integrity & edge cases

**Critical Findings**:
1. **Atomic operations risk** — start_session must be single transaction (TC-027 blocker)
2. **Duration override logic** — COALESCE pattern easy to get wrong (TC-036, 037, 059, 114)
3. **Orphan recovery complexity** — Must block UI and present user choice (TC-050 through TC-053)
4. **Midnight boundary edge case** — Session spanning midnight should count to start_time date (TC-074)
5. **Cascade delete semantics** — Soft vs hard delete not fully specified yet (clarify before dev)

**Recommendations**:
- Implement in order: Core CRUD → Complex features → Frontend integration
- Automate: atomic switch, daily summary, crash recovery (highest ROI)
- Manual checklist: 10-step user flow (<5 min)
- Performance targets: <100ms timer, <3s switch, <50ms search

**Next Steps**: Weekly review during development; update Section 5 as edge cases emerge

---

### 2026-04-12: Phase 1 Code Review — Approved

**From**: Han (Lead)  
**Status**: APPROVED

**Verdict**: Phase 1 implementation APPROVED for shipment.

**Bug Fixes Reviewed** (All Correct ✅):
1. SSR Disable (`src/routes/+layout.ts`) — Standard Tauri + SvelteKit pattern
2. Pause/Resume Return Types (`src/lib/api/sessions.ts`) — Type signature now matches Rust
3. Timer State Refresh (`src/lib/stores/timer.svelte.ts`) — Consistency guaranteed
4. Report Initialization (`src/lib/components/ReportView.svelte`) — Uses `onMount()` correctly

**Code Quality Assessment**:
- Backend (Rust) — Excellent: WAL mode, crash recovery, atomic operations, error handling
- Frontend (Svelte) — Good with one observation: QuickAdd missing `isPaused` field (P2, non-blocking)

**Architecture Review** — Aligned ✅:
- Three-layer separation: Frontend → Service → Data
- Atomic switching via `switch_to_work_order` transactions
- Crash safety: WAL mode + heartbeat recovery
- State management: Svelte 5 runes pattern consistent

**Phase 1 Completeness Check** — All Delivered ✅:
- CRUD for customers and work orders
- Quick-add (create + start atomic)
- Start/stop/pause/resume sessions
- Active timer with real-time display
- Today's work log with inline edit
- Search-to-switch (recent items + filter)
- Daily summary by customer
- CSV export
- Crash recovery (orphan detection + recovery UI)

**Minor Fix Recommended** (Non-Blocking):
- QuickAdd: Add `isPaused: false` to `timer.setActive` call
- Assignee: Leia (Frontend Dev)
- Priority: P2 (nice-to-have before ship)

**Next Steps**:
1. Leia: Fix QuickAdd type (COMPLETED)
2. Wedge: Run test plan to validate performance and edge cases
3. Ship when tests pass ✅

---

### 2026-04-12: Documentation Simplification & Update — Approved

**From**: Mon Mothma (Technical Writer)  
**Status**: COMPLETED

**Objective**: Simplify README as entry point + expand docs to document recent fixes.

**Changes**:

**1. README.md Refactored** (252 → ~90 lines)
- New structure: Pitch → Features → Quick Start → Shortcuts → Doc Links → License
- All hyperlinks to `docs/setup.md`, `docs/architecture.md`, `docs/api-reference.md`, `docs/ui-mockup.html`
- Removed: Platform prerequisites, detailed build instructions, full project structure, troubleshooting
- Rationale: README is entry point; details belong in focused docs

**2. docs/setup.md Created** (~200 lines)
- Platform prerequisites (Windows/macOS/Linux variants)
- Installation walkthrough
- Development workflow (dev server, hot-reload behavior)
- Distribution builds (release output)
- Testing, linting, formatting commands
- Full project structure with explanations
- Data storage and crash recovery
- Troubleshooting FAQ
- Next steps pointers
- Audience: Developers setting up local environment

**3. docs/architecture.md Expanded**

**Section 5.7: SvelteKit SSR Disabled (Critical for Tauri)**
- Problem: SvelteKit pre-renders pages in Node.js at build time via SSR. Tauri's `invoke()` doesn't exist in Node.js—only at runtime. This causes build-time failures in components using IPC.
- Solution: Disable SSR globally in `src/routes/+layout.ts`:
  ```typescript
  export const ssr = false;
  export const prerender = false;
  ```
- Impact: Routes render only in browser after Tauri app starts. IPC calls safe in `onMount()` and effects. No build-time pre-rendering.
- Related fix: ReportView.svelte now uses `onMount()` for data fetching instead of module-level initialization.
- Why documented: This pattern is not obvious to developers new to Tauri + SvelteKit. Critical gotcha that prevents mysterious build/runtime failures.

**Section 5.8: Pause/Resume Pattern (Phase 2 Preparation)**
- Context: Phase 1 has Running/Stopped only. Phase 2 will add Paused.
- Backend pattern: Both `pause_session()` and `resume_session()` return `void`, not `Session`
- Frontend pattern:
  ```typescript
  async pause() {
    await apiPauseSession();   // Returns void
    await timer.refresh();     // Fetch updated state
  }
  ```
- Key insight: Focused commands return void. Frontend queries fresh state separately. Enables cleaner separation of concerns and independent evolution of logic.
- Why documented: Void return is intentional. Prevents future "why doesn't pauseSession return the new session?" confusion. Provides template for Phase 2 implementation.

**4. docs/api-reference.md Verified**
- All Phase 1 IPC commands properly documented
- `stop_session` correctly returns `Session` (not void)
- Phase 2 commands (`pauseSession`, `resumeSession`) intentionally omitted (Phase 1 scope)
- Error codes reference complete

**Documentation Architecture** (Final):
```
README.md (entry point, ~90 lines)
  ↓ hyperlinks to:
  ├─ docs/setup.md (how to install and build)
  ├─ docs/architecture.md (design decisions and patterns)
  ├─ docs/api-reference.md (IPC command reference)
  └─ docs/ui-mockup.html (interactive prototype)
```

**Key Principle**: Simplicity over comprehensiveness. Each doc has one clear audience and purpose. No duplicated content.

**Verification**: All links in README verified to exist. No broken markdown, no placeholder URLs.

---

### 2026-04-12: QuickAdd Type Fix — Completed

**From**: Leia (Frontend Dev)  
**Status**: COMPLETED

**Issue**: QuickAdd.svelte manually constructs `ActiveSession` object, missing required `isPaused: false` field (flagged by Han code review).

**Fix**: Added `isPaused: false` to the object literal:
```typescript
timer.setActive({
  sessionId: result.session.id,
  workOrderId: result.workOrder.id,
  workOrderName: result.workOrder.name,
  customerName: result.customer.name,
  customerColor: result.customer.color,
  startedAt: result.session.startTime,
  elapsedSeconds: 0,
  isPaused: false,  // ← Added
});
```

**Severity**: P2 (type safety improvement; no runtime impact due to guard clauses)

**Outcome**: TypeScript now validates all required fields in `ActiveSession` type. Improves maintainability and IDE autocompletion.

---

## Governance

- All meaningful changes require team consensus
- Document architectural decisions here
- Keep history focused on work, decisions focused on direction
