# Phase 2 Implementation Plan: Multi-Customer Workflows

**Status**: ✅ **COMPLETED** (Phase 3 now current)

This document describes Phase 2 work, which has been implemented and shipped. See [features.md](features.md) for current feature status and [docs/architecture.md](architecture.md) for system overview.

---

## Goal (Completed)

Smooth context switching across customers and projects with advanced quick-access patterns, visual organization, and paused session state support.

## Overlap from Phase 1 → Phase 2

The following Phase 2 items were already completed or partially completed in Phase 1:

| Feature | Phase 1 Completion | Phase 2 Delta |
|---------|-------------------|---------------|
| **Pause/Resume Commands** | ✅ `pause_session()` and `resume_session()` commands exist in backend | Implement UI tier + timer state sync |
| **Favorites Infrastructure** | ✅ `is_favorite` column added to `work_orders` in migration 002 | Implement UI filters + favorite toggle in SearchSwitch |
| **Pause Schema** | ✅ `paused_at`, `total_paused_seconds` columns in migration 002 | Pause logic business rules already in `session_service` |
| **Recent Work Orders** | ✅ `sessionsStore.recent` store populated from daily sessions | Phase 2: sort recent by favorites + last-used timestamp |
| **Search/Switch** | ✅ SearchSwitch component exists with search results | Phase 2: add favorite pinning, better grouping (recent/favorites) |
| **Toggle Favorite API** | ✅ `toggleFavorite()` command and Tauri binding exist | Already wired in SearchSwitch; needs testing |

**Bottom Line**: Phase 1 built the foundation. Phase 2 is primarily **UI implementation + business logic wiring + testing**.

---

## Detailed Work Items

| Item ID | Title | Owner | Depends | Complexity | Est. Hours |
|---------|-------|-------|---------|-----------|-----------|
| **P2-ARCH-1** | Document Phase 2 architecture decisions | Han | — | S | 1.5 |
| **P2-UI-1** | Extend Timer component: pause button + paused badge | Leia | — | M | 3 |
| **P2-UI-2** | Extract pause/resume controls to reusable component | Leia | P2-UI-1 | S | 1.5 |
| **P2-UI-3** | Update SessionList for inline pause/resume actions | Leia | P2-UI-2 | M | 3 |
| **P2-STORE-1** | Extend timer store: track paused state + handle pause/resume sync | Leia | — | M | 2 |
| **P2-SEARCH-1** | Refactor SearchSwitch: group items (favorites, recent, search results) | Leia | — | M | 3 |
| **P2-SEARCH-2** | Add visual favorite indicator + star toggle in search results | Leia | P2-SEARCH-1 | S | 1.5 |
| **P2-HOTKEY-1** | Implement global hotkey for quick-switch (Cmd+Option+S / Ctrl+Shift+S) | Leia | P2-SEARCH-1 | M | 2.5 |
| **P2-TAURI-1** | Add taskbar/system tray support + quick menu (toggle pause, favorites) | Chewie | P2-UI-1 | L | 4 |
| **P2-TEST-UI-1** | Write Svelte component tests: pause/resume lifecycle | Wedge | P2-UI-1, P2-STORE-1 | M | 2.5 |
| **P2-TEST-UI-2** | Write Svelte component tests: SearchSwitch grouping + favorites | Wedge | P2-SEARCH-1, P2-SEARCH-2 | M | 2.5 |
| **P2-TEST-BACKEND-1** | Backend tests: pause state transitions + pause interval calculations | Chewie | — | M | 2 |
| **P2-INTEGRATION-1** | Test pause/resume across timer ↔ SessionList ↔ store sync | Wedge | P2-UI-3, P2-STORE-1, P2-TEST-UI-1 | M | 2 |
| **P2-DOCS-1** | Update API reference for pause/resume commands | Mon Mothma | P2-ARCH-1 | S | 1 |
| **P2-PERF-1** | Verify performance targets: pause/resume (< 100ms), search filtering | Wedge | All UI items | S | 1.5 |

---

## Critical Path (First 3 Items to Unblock Team)

### 1. **P2-ARCH-1**: Document Phase 2 architecture decisions

**Assigned to**: Han  
**Goal**: Define how pause state integrates with existing session lifecycle and UI patterns  
**Deliverable**: `.squad/decisions/inbox/han-phase2-scope.md`

**Items to address**:
- Pause state transitions (Running → Paused → Stopped only, or Running ↔ Paused → Stopped?)
- Visual state indicators (colors, badges) for all 3 states
- How pause interacts with daily summaries (count paused time in totals?)
- System tray / global hotkey integration scope (Phase 2a or defer?)

**Unblocks**: P2-UI-1, P2-SEARCH-1, P2-TEST-BACKEND-1

**Completion Criteria**:
- Decisions document written with clear trade-offs
- Team reviewed and approved

---

### 2. **P2-UI-1**: Extend Timer component: pause button + paused badge

**Assigned to**: Leia  
**Goal**: Add pause/resume UI controls to active timer display  
**Dependencies**: P2-ARCH-1 (for pause state design)

**Scope**:
- Add pause/resume button to Timer component (next to stop button)
- Show paused badge (amber/orange) when session is paused
- Wire `timer.pause()` and `timer.resume()` to backend commands
- Show running badge (green) when not paused
- Keyboard shortcut: Ctrl+P to toggle pause

**Acceptance Criteria**:
- Pause button appears only when session is running/paused
- Button changes to "Resume" when paused
- Paused badge renders in correct color
- Pause/resume completes in < 500ms
- Timer continues to display correctly in both states

**Completion**: Submit PR with design review from Han

---

### 3. **P2-STORE-1**: Extend timer store: track paused state + handle pause/resume sync

**Assigned to**: Leia  
**Goal**: Sync pause state between backend and timer store in real-time  
**Dependencies**: P2-ARCH-1 (for state semantics)

**Scope**:
- Add `isPaused: boolean` to timer store state
- Implement `pause()` and `resume()` methods that call backend + update store
- Handle crash recovery: on `refresh()`, check backend pause state and sync
- Ensure pause state persists across component remounts

**Acceptance Criteria**:
- Timer correctly reflects paused state after pause/resume
- Pause state survives navigation and remount
- `refresh()` syncs pause state from backend
- No data loss on network error (client-side validation)

**Completion**: Unit tests pass, component tests in P2-TEST-UI-1 can proceed

---

## Scope Notes

### In Phase 2

- ✅ Pause/resume UI and state management
- ✅ Favorites filtering and pinning in SearchSwitch
- ✅ Better recent-items sorting (favorites first, then by last-used)
- ✅ Global hotkey for quick-switch (Cmd+Option+S / Ctrl+Shift+S)
- ✅ Inline pause/resume in SessionList (edit existing sessions)
- ✅ System tray icon + quick menu (optional: defer to 2b if timeline tight)

### Deferred to Phase 3+

- Color-coding by customer (visual organization)
- Bulk operations on work orders / sessions
- Advanced keyboard shortcuts beyond pause/hotkey/search
- Multi-user / team collaboration features
- Integrations (billing, accounting, calendar)

---

## Architecture Decisions Needed

Before implementation starts, agree on:

1. **Pause State Transitions**: Is it Running ↔ Paused ↔ Stopped, or only Running → Paused → Stopped?
   - Affects UI layout, timer display logic, and backend state tracking

2. **Paused Time Tracking**: How does paused time count in daily summaries?
   - Option A: Exclude paused intervals (count only running)
   - Option B: Include paused intervals (count as work)
   - Option C: Show separately (e.g., "1h 30m running, 45m paused")

3. **System Tray Scope**: Is quick-menu in Phase 2a (MVP) or Phase 2b (nice-to-have)?
   - Affects timeline; Chewie has P2-TAURI-1 blocked pending decision

4. **Visual State Indicators**: Exact badge colors and positioning
   - Running (green ●), Paused (amber ●), Stopped (grey ● or hidden)

---

## Estimated Timeline

- **Architecture decisions**: 1 day (Han)
- **UI tier (pause/resume + pause state store)**: 5-6 days (Leia) — P2-UI-1, 2, 3, P2-STORE-1, P2-SEARCH-1, 2
- **Hotkey implementation**: 2-3 days (Leia) — P2-HOTKEY-1
- **System tray (optional)**: 4 days (Chewie) — P2-TAURI-1
- **Backend testing**: 2 days (Chewie) — P2-TEST-BACKEND-1
- **Frontend testing**: 4-5 days (Wedge) — P2-TEST-UI-1, 2, P2-INTEGRATION-1
- **Performance validation**: 1-2 days (Wedge) — P2-PERF-1
- **Documentation**: 1 day (Mon Mothma) — P2-DOCS-1

**Total (if all items): 20-24 days**  
**MVP (pause/resume + favorites): 10-12 days**  
**MVP + Hotkey: 12-15 days**  
**Full Phase 2: 20-24 days**

---

## Testing Strategy

### Unit Tests (Wedge)
- Pause state transitions in `session_service` (backend)
- Pause interval calculations in summary queries
- Favorite filtering in SearchSwitch component

### Integration Tests (Wedge)
- Pause action in Timer → store sync → SessionList update
- Resume action triggers correct UI changes
- Pause persists across app restart (crash recovery)
- Daily summary correctly handles paused sessions

### Manual Testing Checklist
- [ ] Pause button appears only when session active
- [ ] Pause/resume completes in < 500ms
- [ ] Paused sessions survive app crash + recovery
- [ ] Favorite toggle updates search results immediately
- [ ] Ctrl+Shift+S global hotkey switches project (no focus required)
- [ ] System tray menu shows favorites + recent
- [ ] Performance: search with 100+ work orders < 100ms

---

## Success Criteria

**Phase 2 Complete When**:
1. ✅ Pause/resume working in UI with backend sync
2. ✅ Favorites pinned in SearchSwitch and working
3. ✅ Global hotkey functional (Ctrl+Shift+S on Windows, Cmd+Option+S on Mac)
4. ✅ All tests passing (unit + integration)
5. ✅ Performance targets met (< 100ms pause/resume, < 100ms search)
6. ✅ Manual testing checklist complete
7. ✅ Documentation updated (API reference, architecture notes)

---

## Next Steps

1. Han: Write Phase 2 scope decisions to `.squad/decisions/inbox/han-phase2-scope.md`
2. Han: Schedule design review with Leia + Chewie for architecture sign-off
3. Leia: Pick up P2-UI-1 (Timer pause button) upon architecture approval
4. Chewie: Pick up P2-TEST-BACKEND-1 (pause state tests) in parallel
5. Wedge: Prepare test plan and tooling for Svelte component testing

