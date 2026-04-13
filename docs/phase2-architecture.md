# Phase 2 Architecture: Multi-Customer Workflows

**Author**: Han (Lead)  
**Status**: Ready for implementation  
**Date**: 2026-04-12  
**Prerequisite reading**: `docs/phase2-plan.md`, `.squad/decisions.md` (Phase 2 scope section)

---

## 1. Phase 2a vs 2b Scope

| Phase 2a (MVP — ship first) | Phase 2b (if timeline permits) |
|------------------------------|-------------------------------|
| P2-UI-1: Pause button in Timer | P2-TAURI-1: System tray quick menu |
| P2-UI-2: PauseResumeControls component | |
| P2-UI-3: SessionList inline pause/resume | |
| P2-STORE-1: Timer store pause sync | |
| P2-SEARCH-1: SearchSwitch grouping | |
| P2-SEARCH-2: Favorite indicators | |
| P2-HOTKEY-1: Global hotkey Ctrl+Shift+S | |
| P2-TEST-UI-1: Component tests | |
| P2-TEST-BACKEND-1: Backend pause tests | |
| P2-TEST-INT-1: Integration tests | |
| P2-DOCS-1: API reference updates | |
| P2-PERF-1: Performance verification | |

**Rule**: Phase 2b does NOT block Phase 2a shipment.

---

## 2. Pause State Flow — Component Interaction

### 2.1 State Machine

```
                 pause()              stop()
  ┌──────────┐ ──────────► ┌──────────┐ ──────────► ┌──────────┐
  │ RUNNING  │             │  PAUSED  │             │ STOPPED  │
  │ (green)  │ ◄────────── │ (amber)  │             │ (grey)   │
  └──────────┘  resume()   └──────────┘             └──────────┘
       │                                                  ▲
       └──────────────────── stop() ──────────────────────┘
```

**Linear transitions only.** No cycling (Paused → Running → Paused is allowed via resume then re-pause, but each transition is a discrete backend call). Once stopped, session is finalized — no un-stopping.

### 2.2 Data Flow: Pause Action

```
User clicks "⏸ Pause"
        │
        ▼
┌─────────────────────────┐
│  Timer.svelte            │ calls timer.pause()
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│  timer.svelte.ts (store) │ 1. await apiPauseSession()    [Tauri IPC]
│                          │ 2. await timer.refresh()       [re-fetch state]
└────────────┬────────────┘
             │  invoke('pause_session')
             ▼
┌─────────────────────────┐
│  commands/sessions.rs    │ routes to session_service::pause_session()
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│  session_service.rs      │ 1. Validate: active session exists + not paused
│                          │ 2. SET active_session.is_paused = 1
│                          │ 3. SET active_session.paused_session_at = NOW
│                          │ 4. SET time_sessions.paused_at = NOW
│                          │ Returns: Ok(())
└─────────────────────────┘
             │
             ▼  (timer.refresh fetches new state)
┌─────────────────────────┐
│  get_active_session()    │ Returns ActiveSession { isPaused: true, ... }
│                          │ elapsedSeconds excludes current pause interval
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│  timer store $effect     │ isPaused = true → stopTick() (timer freezes)
│                          │ Updates tray tooltip
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│  Timer.svelte re-renders │ Badge: amber "Paused", button: "▶ Resume"
│  SessionList re-renders  │ (if inline pause badges shown)
│  DailySummary unchanged  │ (paused time still counted in totals)
└─────────────────────────┘
```

### 2.3 Data Flow: Resume Action

Same flow, reversed:
1. `timer.resume()` → `invoke('resume_session')`
2. Backend: calculate pause duration, add to `total_paused_seconds`, clear `paused_at`
3. `timer.refresh()` → `isPaused: false`, `elapsedSeconds` updated (excludes paused time)
4. `$effect` fires → `startTick()` resumes

### 2.4 Data Flow: Stop While Paused

Critical path — user can stop from paused state:
1. Backend `stop_active_session()` already handles this: reads `total_paused_seconds`, subtracts from gross duration
2. No special UI logic needed — Stop button works in both Running and Paused states
3. Duration calculation: `effective = (end_time - start_time) - total_paused_seconds`

---

## 3. Timer Store Extension (P2-STORE-1)

### 3.1 Current State (Phase 1 — already implemented)

```typescript
// timer.svelte.ts — ALREADY EXISTS
let activeSession = $state<ActiveSession | null>(null);
let elapsedSeconds = $state(0);
const isPaused = $derived(activeSession?.isPaused ?? false);

// $effect already handles tick start/stop based on isPaused
$effect(() => {
  if (activeSession && !isPaused) { startTick(); }
  else { stopTick(); }
});

// pause() and resume() methods already exist
async pause()  { await apiPauseSession(); await timer.refresh(); }
async resume() { await apiResumeSession(); await timer.refresh(); }
```

### 3.2 What P2-STORE-1 Actually Needs

The store is already wired. The work item is about **verification and edge-case hardening**, not new code:

1. **Verify `$effect` tick restart works on resume** — The reactive effect should fire when `isPaused` flips from `true` → `false`, restarting the tick interval. Already implemented but needs testing.

2. **Verify tray tooltip updates on pause/resume** — `updateTrayTooltip()` is called in `setActive()` but NOT after pause/resume. **FIX NEEDED**: Call `updateTrayTooltip()` at the end of `pause()` and `resume()` methods, or make it reactive via `$effect`.

3. **Heartbeat behavior during pause** — Currently heartbeat runs on 30s interval regardless of pause state. This is acceptable (keeps orphan detection alive). No change needed.

4. **Race condition: rapid pause/resume** — If user clicks pause then immediately resume before first `refresh()` returns, the second call could fail ("session is not paused" because backend hasn't processed pause yet). **Mitigation**: Add a `transitioning` flag to disable buttons during pause/resume:

```typescript
// Add to timer store
let transitioning = $state(false);

export const timer = {
  get transitioning() { return transitioning; },

  async pause() {
    if (transitioning) return;
    transitioning = true;
    try {
      await apiPauseSession();
      await timer.refresh();
    } catch (e: any) {
      alert(e?.message ?? 'Failed to pause');
    } finally {
      transitioning = false;
    }
  },

  async resume() {
    if (transitioning) return;
    transitioning = true;
    try {
      await apiResumeSession();
      await timer.refresh();
    } catch (e: any) {
      alert(e?.message ?? 'Failed to resume');
    } finally {
      transitioning = false;
    }
  }
};
```

5. **Tray tooltip for paused state**: Extend `updateTrayTooltip()` to show paused indicator:

```typescript
function updateTrayTooltip() {
  let tooltip = 'Work Tracker — Not tracking';
  if (activeSession) {
    const state = isPaused ? '⏸' : '⏱';
    tooltip = `${state} Work Tracker — ${activeSession.workOrderName} (${activeSession.customerName})`;
  }
  invoke('update_tray_tooltip', { tooltip }).catch(console.error);
}
```

### 3.3 Constraint for Leia

- **Do not add new Tauri commands** for pause. Backend is complete.
- **Do not change `ActiveSession` type** — `isPaused: boolean` already exists.
- **Do add `transitioning` guard** to prevent double-click race condition.
- **Do make tray tooltip reactive** to pause state changes.

---

## 4. SearchSwitch Refactor Plan (P2-SEARCH-1, P2-SEARCH-2)

### 4.1 Current Behavior

`SearchSwitch.svelte` shows a flat list:
- Empty search → `sessionsStore.recent` (recent work orders)
- With search → filtered `listWorkOrders()` results
- Favorite star toggle exists but no grouping

### 4.2 Target Behavior: Three-Group Display

```
┌─────────────────────────────────┐
│ Search work orders... (Ctrl+K)  │
├─────────────────────────────────┤
│ ★ FAVORITES                     │  ← Group header (always visible)
│   ⭐ Frontend Redesign          │
│   ⭐ API Integration            │
│                                 │
│ ◷ RECENT                        │  ← Group header
│   ☆ Bug fixes                   │
│   ☆ Sprint planning             │
│                                 │
│ ⬡ ALL                           │  ← Group header (search mode only)
│   ☆ Legacy migration            │
│   ☆ Documentation               │
└─────────────────────────────────┘
```

### 4.3 Grouping Algorithm

```typescript
type GroupedItems = {
  favorites: WorkOrder[];
  recent: WorkOrder[];
  all: WorkOrder[];      // only populated during search
};

function groupItems(items: WorkOrder[], isSearching: boolean): GroupedItems {
  const favorites = items.filter(wo => wo.isFavorite);
  const nonFavorites = items.filter(wo => !wo.isFavorite);

  if (isSearching) {
    return { favorites, recent: [], all: nonFavorites };
  }

  // No search: split recent (non-favorite) items
  return { favorites, recent: nonFavorites, all: [] };
}
```

**Sort within each group**: By `last_used` timestamp (most recent first). This requires the backend to return a `last_used` field. Check: Does `getRecentWorkOrders` already include this?

### 4.4 Backend Requirement Check

`get_recent_work_orders` in the reports service returns work orders sorted by most-recent session. The `WorkOrder` type already has `isFavorite`. **No backend changes needed** — grouping is purely frontend.

### 4.5 Refactor Steps (for Leia)

1. **Extract `groupItems()` utility function** (testable, pure)
2. **Replace flat `displayItems` derived** with grouped derived:
   ```typescript
   let grouped = $derived(groupItems(
     query.trim() ? searchResults : sessionsStore.recent,
     !!query.trim()
   ));
   ```
3. **Render groups with headers** — each group gets a `<div class="group">` with `<h4>` label
4. **Keyboard navigation across groups** — `selectedIndex` must span all groups continuously (0..N across favorites + recent + all)
5. **Empty group handling** — hide group header if group has 0 items
6. **Active badge update** — show "Running" (green) or "Paused" (amber) badge next to active work order

### 4.6 Favorite Toggle Pattern (P2-SEARCH-2)

Already implemented in `handleToggleFavorite()`. The refactor only needs:
- Star icon already renders ⭐/☆ based on `isFavorite`
- After toggle, the item moves between groups on next refresh
- **Optimistic update**: Toggle `isFavorite` in local state immediately, refresh from backend async

### 4.7 Constraint for Leia

- **Keep SearchSwitch as single component** — don't split into sub-components yet (premature for 3 groups)
- **Keyboard navigation must work across group boundaries** — ArrowDown from last favorite → first recent
- **Empty state per group** — don't show "No favorites" placeholder; just hide the section
- **Badge colors**: Running = `var(--accent)` green, Paused = `#f59e0b` amber
- **Search mode**: Show favorites matching query at top, then all other matches below

---

## 5. Global Hotkey Integration (P2-HOTKEY-1)

### 5.1 Plugin: `tauri-plugin-global-shortcut`

Tauri 2 provides first-party support via `@tauri-apps/plugin-global-shortcut`.

**Installation**:
```bash
# Rust side
cargo add tauri-plugin-global-shortcut

# JS side
npm install @tauri-apps/plugin-global-shortcut
```

**Cargo.toml addition**:
```toml
tauri-plugin-global-shortcut = "2"
```

**lib.rs registration**:
```rust
.plugin(tauri_plugin_global_shortcut::init())
```

### 5.2 Hotkey Registration (Frontend)

Register in the root `+layout.svelte` or `+page.svelte` on mount:

```typescript
import { register } from '@tauri-apps/plugin-global-shortcut';
import { getCurrentWindow } from '@tauri-apps/api/window';

onMount(async () => {
  await register('CmdOrCtrl+Shift+S', async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.setFocus();
    // Focus the search input in SearchSwitch
    document.querySelector<HTMLInputElement>('.search-input')?.focus();
  });
});
```

### 5.3 Behavior on Hotkey Press

1. Window brought to front and focused (even if minimized)
2. SearchSwitch search input auto-focused
3. User types to search or arrow-keys through recents/favorites
4. Enter to switch, Escape to dismiss (leaves window open)

### 5.4 Cleanup on Window Close

```typescript
import { unregister } from '@tauri-apps/plugin-global-shortcut';

onDestroy(async () => {
  await unregister('CmdOrCtrl+Shift+S');
});
```

### 5.5 Platform Notes

| Platform | Shortcut | Notes |
|----------|----------|-------|
| Windows  | Ctrl+Shift+S | Works natively; may conflict with some apps |
| macOS    | Cmd+Shift+S  | `CmdOrCtrl` maps correctly |
| Linux    | Ctrl+Shift+S | Wayland may have limitations; X11 works |

**Risk**: Some apps (VS Code, browsers) may capture Ctrl+Shift+S. Tauri global shortcuts take priority over app-level shortcuts, but if another global shortcut tool has it registered first, it won't work. Test early.

### 5.6 Constraint for Leia

- Register shortcut in `+layout.svelte` `onMount` (not in individual components)
- Use `CmdOrCtrl+Shift+S` (Tauri's cross-platform syntax)
- Focus search input programmatically — add an `id="search-switch-input"` to the SearchSwitch input
- Don't auto-clear the search field on hotkey — user may want to resume a previous search
- Handle registration failure gracefully (log warning, don't crash)

---

## 6. System Tray Architecture (P2-TAURI-1 — Phase 2b Only)

### 6.1 Current State

Tray icon already configured in `tauri.conf.json`:
```json
"trayIcon": {
  "iconPath": "icons/32x32.png",
  "iconAsTemplate": true
}
```

Tooltip update already works via `update_tray_tooltip` Tauri command.

### 6.2 Phase 2b Enhancement: Right-Click Menu

The tray needs a dynamic context menu built from Rust, updated whenever session state changes.

**Menu structure**:
```
┌──────────────────────────┐
│ ⏸ Pause current session  │  (or ▶ Resume, based on state)
│ ■ Stop tracking          │
│ ─────────────────────── │
│ ★ Frontend Redesign      │  (favorite 1 — click to switch)
│ ★ API Integration        │  (favorite 2)
│ ─────────────────────── │
│ ◷ Bug fixes              │  (recent 1)
│ ◷ Sprint planning        │  (recent 2)
│ ─────────────────────── │
│ Open Work Tracker        │
│ Quit                     │
└──────────────────────────┘
```

### 6.3 Implementation Approach

Tauri 2's tray API supports dynamic menus via Rust:

```rust
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;

// Build menu dynamically based on current state
fn build_tray_menu(app: &tauri::AppHandle, state: &TrayState) -> Menu {
    let menu = Menu::new(app).unwrap();
    // Add items based on active session, favorites, recents
    // ...
    menu
}
```

**Key challenge**: Menu must rebuild when session state changes (pause/resume/stop/switch). Trigger rebuild:
- After every `pause_session`, `resume_session`, `stop_session`, `start_session`
- After `toggle_favorite`
- On periodic heartbeat (fallback)

### 6.4 Constraint for Chewie

- **Phase 2b only** — do not start until Phase 2a ships
- Tray menu actions invoke same Tauri commands as UI (no new backend code)
- Menu rebuild must be < 50ms (it runs on main thread)
- Test on Windows first (primary platform), macOS second
- Use `TrayIconBuilder::on_menu_event` for click handling
- Favorites/recents: query from DB directly (don't rely on frontend state)

---

## 7. Risk: Pause State Sync Race Condition

### 7.1 The Problem

```
Time ──────────────────────────────────►

UI:    [click Pause]     [click Resume]
        │                  │
IPC:    ├─► pause_session  │
        │   (in flight)    ├─► resume_session
        │                  │   FAILS: "not paused yet"
        │   ◄── Ok(())    │
        │                  │
```

User clicks Pause, then immediately clicks Resume before the pause IPC round-trip completes. The resume call hits the backend before the pause has been persisted.

### 7.2 Mitigations (Ranked by Priority)

**M1: UI Transitioning Guard (Required — P2-STORE-1)**

Disable pause/resume buttons during the IPC round-trip:
```typescript
let transitioning = $state(false);
// Wrap pause() and resume() with transitioning = true/false
// Buttons: disabled={timer.transitioning}
```

This is the primary fix. Fast, simple, prevents the race entirely at the UI layer.

**M2: Backend Idempotency (Recommended — P2-TEST-BACKEND-1)**

Make `pause_session` and `resume_session` return success if already in desired state, instead of erroring:
```rust
// Instead of: return Err("already paused")
// Do: return Ok(()) — idempotent
```

Pro: More resilient. Con: Masks potential bugs. **Decision: Keep strict validation in Phase 2a**, switch to idempotent in Phase 2b if needed.

**M3: Optimistic UI State (Optional — skip in Phase 2a)**

Update `isPaused` in the store immediately before the IPC call, then reconcile on refresh. Adds complexity. Not needed if M1 is implemented.

### 7.3 Decision

Implement M1 (transitioning guard) in Phase 2a. Verify M2 viability in P2-TEST-BACKEND-1 but don't change behavior yet. Skip M3.

---

## 8. Key Constraints by Implementer

### Leia (Frontend — P2-UI-1/2/3, P2-STORE-1, P2-SEARCH-1/2, P2-HOTKEY-1)

| Constraint | Reason |
|-----------|--------|
| No new Tauri commands | Backend pause/resume/favorites are complete |
| No changes to `ActiveSession` type | `isPaused` field already exists |
| Add `transitioning` guard to timer store | Prevents race condition (Section 7) |
| Make tray tooltip reactive to pause state | User sees ⏸/⏱ in system tray |
| Use `CmdOrCtrl+Shift+S` for hotkey | Cross-platform, Tauri convention |
| Register hotkey in `+layout.svelte` | Available globally, cleaned up on destroy |
| Keep SearchSwitch as single component | Don't over-split; 3 groups don't warrant sub-components |
| Keyboard nav across group boundaries | ArrowDown/Up must cross favorites→recent→all seamlessly |
| Pause badge color: `#f59e0b` | Amber, matches Running green `var(--accent)` |
| No new npm dependencies except `@tauri-apps/plugin-global-shortcut` | Keep bundle lean |

### Chewie (Backend/Tray — P2-TAURI-1, P2-TEST-BACKEND-1)

| Constraint | Reason |
|-----------|--------|
| P2-TAURI-1 is Phase 2b — do not start until 2a ships | Dependency management |
| Backend tests first (P2-TEST-BACKEND-1) — unblocked now | Can start immediately |
| Test pause/resume state transitions exhaustively | Cover: pause→resume, pause→stop, double-pause (error), resume without pause (error) |
| Test duration calculation with paused intervals | `effective = gross - total_paused_seconds` |
| Tray menu rebuild must be < 50ms | Main thread performance |
| Tray actions use existing commands, no new backend code | Single source of truth |
| Add `tauri-plugin-global-shortcut = "2"` to Cargo.toml | Required for P2-HOTKEY-1 |

### Wedge (Testing — P2-TEST-UI-1, P2-TEST-INT-1, P2-PERF-1)

| Constraint | Reason |
|-----------|--------|
| Component tests: use Vitest + testing-library if Svelte 5 supports it | Known limitation: `$effect` context may block some tests |
| Integration test: full pause→resume→stop flow through store | End-to-end state verification |
| Performance: measure pause/resume round-trip (target < 100ms) | Below user perception threshold |
| Performance: measure SearchSwitch filter with 100+ work orders (target < 100ms) | Phase 2 scales data |

### Mon Mothma (Docs — P2-DOCS-1)

| Constraint | Reason |
|-----------|--------|
| Update `docs/api-reference.md` with pause/resume command docs | Commands exist but undocumented |
| Document global hotkey in `docs/setup.md` | User-facing feature |
| Add pause state to `docs/architecture.md` Section 5 | Architectural completeness |

---

## 9. Definition of Done — Phase 2

### Phase 2a (Must-Have for Ship)

- [ ] **Pause/Resume UI works**: Timer shows pause button (running), resume button (paused), correct badge colors
- [ ] **State transitions correct**: Running→Paused→Stopped, Running→Stopped, no invalid transitions
- [ ] **Timer freezes on pause**: `elapsedSeconds` stops incrementing, resumes correctly
- [ ] **Tray tooltip shows pause state**: ⏸ when paused, ⏱ when running
- [ ] **Transitioning guard active**: Buttons disabled during IPC round-trip
- [ ] **SearchSwitch groups items**: Favorites section, Recent section, All (search mode)
- [ ] **Favorite toggle works**: Star click toggles, item moves between groups
- [ ] **Global hotkey works**: Ctrl+Shift+S brings window + focuses search input
- [ ] **Backend tests pass**: Pause/resume transitions, duration calculations, error cases
- [ ] **No regressions**: All Phase 1 tests still pass
- [ ] **Performance verified**: Pause/resume < 100ms, search filter < 100ms
- [ ] **API docs updated**: Pause/resume commands documented

### Phase 2b (Nice-to-Have)

- [ ] System tray right-click menu with dynamic entries
- [ ] Tray menu shows pause/resume/stop actions
- [ ] Tray menu shows favorites + recent work orders
- [ ] Click tray menu item switches project

---

## 10. File Reference

| File | Phase 2 Changes |
|------|----------------|
| `src/lib/stores/timer.svelte.ts` | Add `transitioning` guard, reactive tray tooltip |
| `src/lib/components/Timer.svelte` | Disable buttons during transition (already has pause/resume UI) |
| `src/lib/components/SearchSwitch.svelte` | Refactor for grouped display, keyboard nav across groups |
| `src/lib/components/SessionList.svelte` | Add pause state badges to today's session entries |
| `src/routes/+layout.svelte` | Register global hotkey |
| `src/routes/+page.svelte` | Update footer shortcuts hint |
| `src-tauri/Cargo.toml` | Add `tauri-plugin-global-shortcut` |
| `src-tauri/src/lib.rs` | Register global-shortcut plugin |
| `src-tauri/src/commands/sessions.rs` | No changes needed |
| `src-tauri/src/services/session_service.rs` | No changes needed |
