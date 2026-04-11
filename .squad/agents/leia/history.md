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

**Integration notes**:
- All Tauri IPC calls imported from '$lib/api'
- Error handling follows AppError structure (code, message, details)
- Timer updates happen every 1s via setInterval
- Real-time summary updates via $effect reactivity

