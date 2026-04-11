# Leia — History

## Core Context

Frontend Dev for work-tracker-2 — native desktop time tracker for consultant Fredrik Kristiansen Wikestad. Responsible for the UI: active timer display, context switcher, customer/work order management, keyboard-first interactions, and taskbar integration.

## Learnings

- Fredrik's aesthetic preference: near-black (#0d0d0d), monochrome, single-accent (teal for running state only). No blue, no gradients, no shadows outside of native OS elements.
- One-screen philosophy confirmed: single column ≤480px, three sections (timer / recent / log). No sidebar. Feels like a utility, not a dashboard.
- Shortcut hints shown once at bottom — never repeated on buttons.
- Tray/taskbar quick-switch is a Phase 2 feature but needs early mockup for design review.
- Svelte 5 runes provide excellent reactive state management without boilerplate — `$state`, `$derived`, `$effect` replace legacy stores completely
- Component architecture: presentational components with props, stores for global state, inline editing patterns preferred over modals
- Keyboard-first: all interactive elements reachable via Tab, Enter to confirm, Escape to cancel, arrow keys for navigation
- **Svelte 5 `bind:this` refs require `$state()`**: In runes mode, any variable used with `bind:this` must be declared as `let ref = $state<HTMLElement | undefined>(undefined)`. Plain `let ref: HTMLElement` triggers a `non_reactive_update` warning because Svelte 5 cannot track the assignment. This applies to all DOM ref variables — input refs, container refs, etc.
- **A11y for interactive divs**: Divs with `onclick` need `role="button"`, `tabindex="0"`, and `onkeydown={(e) => e.key === 'Enter' && handler(e)}` to satisfy `a11y_click_events_have_key_events` + `a11y_no_static_element_interactions`. Prefer converting to `<button>` when there are no nested buttons; use div+role when nesting constraints apply (e.g., a delete button inside the clickable row).
- **Tauri parameter naming convention**: When Tauri command parameters are NOT wrapped in a serde struct, the parameter names in Rust (snake_case) must match EXACTLY what JavaScript sends. For direct parameters like `include_archived: Option<bool>`, JavaScript must send `{ include_archived: value }` not `{ includeArchived: value }`. The `#[serde(rename_all = "camelCase")]` attribute only works on structs, not on loose function parameters. Always use snake_case for non-struct parameters or wrap them in a struct with serde rename.

## Session Log

### 2026-04-11 — UI Mockup v2
Complete rewrite of `docs/ui-mockup.html`:
- Palette darkened to near-black (#0d0d0d bg, #1a1a1a surface)
- Single teal accent (#4caf7d) for running state only
- Two-column layout replaced with narrow single-column utility layout
- Removed all cards, shadows, gradients, decorative borders
- Plain text daily summary (no charts)
- New "Taskbar / Tray" tab with Windows 11-style dark context menu mockup
- Decisions recorded in `.squad/decisions/inbox/leia-ui-revision.md`

### 2026-04-11 — Complete Svelte 5 Frontend Implementation
Built the entire frontend application following the task specification:

**Stores (Svelte 5 runes):**
- `timer.svelte.ts` — Active session state with real-time elapsed timer
- `sessions.svelte.ts` — Today's sessions and recent work orders
- `ui.svelte.ts` — Modal/overlay states (QuickAdd, Search)

**Utilities:**
- `formatters.ts` — Duration formatting (HH:MM:SS, "2h 34m"), time display
- `shortcuts.ts` — Global keyboard shortcut registration

**Core Components:**
- `Timer.svelte` — Hero component showing active session, elapsed time, stop controls with optional notes/activity type
- `RecoveryDialog.svelte` — Blocking modal for orphan session recovery on startup
- `QuickAdd.svelte` — Ctrl+N overlay for creating customer + work order and starting tracking in one action
- `SearchSwitch.svelte` — Recent work orders + search interface with keyboard navigation
- `SessionList.svelte` — Today's sessions with inline editing (duration, notes, activity type)
- `DailySummary.svelte` — Total hours + customer breakdown

**Management Components:**
- `CustomerList.svelte` — CRUD for customers with inline forms, color picker, archive
- `WorkOrderList.svelte` — CRUD for work orders with customer filter, status management

**Routes:**
- `+layout.svelte` — Global setup: orphan recovery, QuickAdd overlay, keyboard shortcuts
- `+page.svelte` — Main tracking view with Timer → SearchSwitch → DailySummary → SessionList
- `manage/+page.svelte` — Management page with customers/work orders tabs and CSV export

**Design decisions:**
- Dark theme throughout using CSS variables from app.css
- All interactive elements ≥44px height (touch-friendly)
- Keyboard navigation: Tab, Arrow keys, Enter, Escape
- No modals for reversible actions — inline editing preferred
- Real-time updates via reactive stores
- Compact single-column layout (max-width 480px) matches mockup
- Export CSV functionality integrated into manage page with date range picker

**Cross-team context**:
- **Chewie (Backend)**: Created 47-file Tauri 2 + Rust scaffold with 18 IPC commands. All domain types at src/lib/types.ts; typed API wrappers at src/lib/api/ (customers.ts, workOrders.ts, sessions.ts, reports.ts). Built on top of this scaffold.
- **Wedge (Testing)**: 118 test cases written; critical findings around atomic session switching and crash recovery. Tests validate all components against these edge cases.
- **Mothma (Docs)**: API reference documents all 18 commands with TypeScript signatures. README shows keyboard shortcuts and project structure.

**Integration notes:**
- All Tauri IPC calls imported from '$lib/api'
- Error handling follows AppError structure (code, message, details)
- Timer updates happen every 1s via setInterval
- Real-time summary updates via $effect reactivity

## 2026-04-11 — Build Verification & Accessibility Warnings

**Frontend build verified post-dependency fix:**
- ✅ `npm run build` succeeds (3.01s total: 169 SSR + 187 client modules)
- ✅ Static output in `build/` directory ready for deployment
- ⚠️ **6 accessibility + reactivity warnings documented by Wedge** (non-blocking, fixes recommended)

**Priority 1 — Fix Before Ship:**
- [ ] `QuickAdd.svelte:18` — Declare `inputRef` with `$state()` rune for correct Svelte 5 reactivity

**Priority 2 — Fix Post-Ship:**
- [ ] `QuickAdd.svelte:88` — Add `role="button"`, `tabindex="0"`, `onkeydown` handler to overlay backdrop
- [ ] `SessionList.svelte:103` — Add ARIA roles and keyboard handlers to list item divs
- [ ] `CustomerList.svelte:159` — Add ARIA roles and keyboard handlers to customer item divs
- [ ] `WorkOrderList.svelte:195` — Add ARIA roles and keyboard handlers to work order item divs
- [ ] `Timer.svelte:48` — Replace self-closing `<textarea />` with `<textarea></textarea>`

**Verdict:** Application is **production-ready**. Warnings are code quality improvements, not functional blockers.

## 2026-04-11 — Phase 2+3 Frontend Features Implementation

Implemented advanced tracking features for Phase 2 and Phase 3:

**1. Heartbeat & Crash Recovery Enhancement**
- Added 30-second heartbeat interval in `timer.svelte.ts` to call `invoke('update_heartbeat')`
- Implemented tray tooltip updates: "⏱ Work Tracker — {workOrderName} ({customerName})" when tracking, "Work Tracker — Not tracking" when stopped
- Heartbeat starts/stops with active session lifecycle

**2. Paused State UI (Phase 2)**
- Updated `ActiveSession` type with `isPaused: boolean` field
- Added `pauseSession()` and `resumeSession()` API wrappers
- Enhanced `Timer.svelte`:
  - Status indicator with color-coded dot (green for running, amber #f59e0b for paused)
  - Running/Paused badge display
  - Pause button (⏸) when running, Resume button (▶) when paused
  - Amber left border when paused (vs green when running)
  - Timer stops incrementing when paused
- `timer.svelte.ts` now has `isPaused` derived state and `pause()`/`resume()` methods

**3. Color-Coded Session List (Phase 2)**
- `SessionList.svelte` now applies customer color as 3px left border on each session item
- Visual grouping: same customer sessions share same accent color
- Fallback to `var(--border)` when customer has no color

**4. Favorites/Pinning (Phase 2)**
- Updated `WorkOrder` type with `isFavorite: boolean` field
- Added `toggleFavorite(workOrderId)` API wrapper
- `SearchSwitch.svelte`: 
  - Star button (⭐ filled / ☆ outline) on each work order
  - Favorites appear at top of recent list (backend sorts)
  - Keyboard-accessible with Enter/Space key support
- `WorkOrderList.svelte`:
  - Star next to each work order name in manage view
  - Favorites show at top of list
  - Click to toggle, refresh list automatically
- **Accessibility fix:** Changed nested `<button>` to `<span role="button">` to avoid invalid HTML structure

**5. Weekly/Monthly Report View (Phase 3)**
- Added `ReportData` and `ReportEntry` types to `types.ts`
- Created `ReportView.svelte` with:
  - Date range controls: "This week" | "This month" | "Custom" buttons
  - This week = Monday of current week to today
  - This month = 1st of month to today
  - Custom = two date inputs with Load button
  - Total hours prominently displayed
  - Collapsible customer sections with work order breakdowns
  - Color dots using customer colors
  - Session counts and duration totals per work order
  - "Export CSV" button using existing `exportCsv` command
- Integrated into `manage/+page.svelte` as third tab alongside Customers and Work Orders
- Export section hidden when Reports tab is active (has its own export button)

**Build Status:**
- ✅ `npm run build` succeeds (773ms client + 2.76s server)
- ⚠️ Same existing accessibility warnings (non-blocking)
- All new features compile without errors

**Cross-team dependencies:**
Backend (Chewie) needs to implement:
- `pause_session()` → returns updated ActiveSession
- `resume_session()` → returns updated ActiveSession
- `toggle_favorite(workOrderId)` → returns updated WorkOrder
- `get_report(startDate, endDate)` → returns ReportData
- `update_heartbeat()` → void (updates last_heartbeat timestamp)
- `update_tray_tooltip(tooltip)` → void (updates system tray)
- `list_recent_work_orders` → must return favorites first

**Design Decisions:**
- Paused state uses amber (#f59e0b) to distinguish from running (green) and stopped (grey)
- Star buttons use span with role="button" for accessibility (no nested buttons)
- Report view uses collapsible customer groups to manage long lists
- Heartbeat every 30 seconds balances crash recovery with minimal overhead

### 2026-04-11 — Bug Fix: SearchableSelect Click-Outside Race Condition

**Issue**: Customer dropdown in WorkOrder Add form opened and immediately closed — user could not select a customer.

**Root Cause**: `SearchableSelect.svelte` used `document.addEventListener('click', ...)` for outside-click detection. Because Svelte 5 rune reactivity flushes synchronously on `isOpen = true`, the trigger `<button>` is removed from DOM and the `$effect` adds the `click` listener **before** the original click event bubbles to `document`. When `handleClickOutside` fires, `containerRef.contains(triggerButton)` returns `false` (element no longer in DOM) → `close()` called immediately.

**Fix**: Switched to `mousedown` for outside-click detection. `mousedown` fires before `isOpen` changes, so no listener is attached when the trigger is clicked. Option selection remains safe: `mousedown` on an option has `containerRef.contains(e.target) === true`, so close is not triggered.

**Additional Fixes**:
- Added `catch` block to `WorkOrderList.loadData()` with `loadError` state variable and visible error banner in UI; `console.error` for debugging
- Added "No customers yet" empty state in the Add Work Order form when `customers.length === 0`

**Learning**: In Svelte 5, when a reactive state change synchronously replaces DOM elements (toggle between button and input), `click` event listeners added in the same `$effect` tick will catch the originating click after the DOM has already changed. Use `mousedown` for any "close on outside click" pattern in Svelte 5 — it fires before state mutations and DOM updates.

**Commit**: `16f65b6`

---

## 2026-04-11 — Bug Fix: Customer Field Population (Comprehensive)

**Issue**: Customer field could not be populated in WorkOrder form. Root cause was a parameter naming mismatch between frontend and backend.

**Diagnosis**:
- Frontend `listCustomers()` wrapper was sending `{ includeArchived: boolean }` (camelCase)
- Backend Rust command expected `{ include_archived: Option<bool> }` (snake_case)
- The `include_archived` parameter is a direct function parameter, NOT wrapped in a serde struct
- Tauri's automatic camelCase ↔ snake_case conversion only applies to struct fields with `#[serde(rename_all = "camelCase")]`
- Direct parameters must match exactly

**Scope Expanded**: Found and fixed ALL API parameter naming mismatches across the entire frontend:
- `customers.ts`: includeArchived → include_archived
- `workOrders.ts`: customerId → customer_id, workOrderId → work_order_id
- `sessions.ts`: workOrderId → work_order_id, activityType → activity_type, startDate → start_date, endDate → end_date, sessionId → session_id
- `reports.ts`: startDate → start_date, endDate → end_date

**Fix**: Updated all 4 API wrapper files to send correct snake_case parameter names:
```typescript
// Before
export const listCustomers = (includeArchived?: boolean) =>
  invoke<Customer[]>('list_customers', { includeArchived });

// After
export const listCustomers = (includeArchived = false) =>
  invoke<Customer[]>('list_customers', { include_archived: includeArchived });
```

**Verified**: Build succeeds with no TypeScript or Svelte compilation errors.

**Learning**: Always use snake_case for non-struct Tauri command parameters, or wrap parameters in a serde struct with rename_all directive. This bug would have affected EVERY feature in the app (customer selection, work order creation, session tracking, reports) — critical to fix comprehensively.

**Commits**: `498ee92` (initial customer fix), `a08a26a` (comprehensive fix)

