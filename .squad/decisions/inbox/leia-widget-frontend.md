# Widget Mode — Frontend Component Structure & API Surface

_For Wedge to write accurate integration tests_

---

## Overview

Widget mode shrinks the main Tauri window to 320×150 px and shows a compact always-on-top overlay. Toggling OFF restores the previous window size/position.

---

## Files

| File | Role |
|---|---|
| `src/lib/api/window.ts` | Tauri invoke wrapper |
| `src/lib/stores/widget.svelte.ts` | Widget mode state store |
| `src/lib/components/WidgetOverlay.svelte` | Compact overlay component |
| `src/routes/+page.svelte` | Toggle button + conditional render + event listener |

---

## API: `src/lib/api/window.ts`

```typescript
export async function toggleWidgetMode(enable: boolean): Promise<boolean>
```

- Invokes Tauri command `toggle_widget_mode` with `{ enable: boolean }`
- Returns `boolean` (the new widget mode state from backend)
- Should be called by UI before updating `widgetStore`

---

## Store: `src/lib/stores/widget.svelte.ts`

```typescript
export const widgetStore = {
  get isWidgetMode(): boolean   // $state — true when in widget mode
  setWidgetMode(value: boolean): void
}
```

**Mock shape for tests:**
```typescript
vi.mock('$lib/stores/widget.svelte', () => ({
  widgetStore: {
    isWidgetMode: false,        // override as needed per test
    setWidgetMode: vi.fn(),
  },
}));
```

---

## Component: `WidgetOverlay.svelte`

**Reads from:**
- `timer.isTracking` — `boolean`
- `timer.isPaused` — `boolean`
- `timer.elapsed` — `number` (seconds)
- `timer.active` — `ActiveSession | null`
  - `.workOrderName: string`
  - `.customerName: string`
  - `.customerColor: string | null`

**Rendered structure (idle / not tracking):**
```
[⊘ Stopped badge]  [✕ exit button]
00:00              (elapsed)
Not tracking
```

**Rendered structure (active):**
```
[🟢 Running badge]  [✕ exit button]
1:23:45             (elapsed, large)
Work Order Name
Customer Name
```

**Interactions:**
- Exit button (`title="Exit widget mode (Ctrl+Alt+W)"`) calls `toggleWidgetMode(false)` then `widgetStore.setWidgetMode(false)`
- No other interactive controls — pause/stop must be done from normal mode

**State badges:**
| State | Icon | Text | CSS class |
|---|---|---|---|
| Not tracking | ⊘ | Stopped | `.stopped` |
| Tracking, paused | 🟡 | Paused | `.paused` |
| Tracking, running | 🟢 | Running | `.running` |

---

## Toggle flow in `+page.svelte`

1. User clicks 📌 button → `handleWidgetToggle()`
2. `toggleWidgetMode(!isWidgetMode)` invoked
3. On success → `widgetStore.setWidgetMode(next)`
4. `{#if widgetStore.isWidgetMode}` renders `<WidgetOverlay />` instead of normal layout

**Backend event listener (Ctrl+Alt+W global shortcut):**
```typescript
listen('toggle-widget-mode', (event) => {
  widgetStore.setWidgetMode(event.payload as boolean);
});
```
Note: when the event fires, the backend has already resized the window. The listener only syncs the store state — it does NOT call `toggleWidgetMode()` again.

---

## Mocks needed for integration tests

```typescript
vi.mock('$lib/stores/widget.svelte', () => ({
  widgetStore: { isWidgetMode: false, setWidgetMode: vi.fn() },
}));

vi.mock('$lib/api/window', () => ({
  toggleWidgetMode: vi.fn().mockResolvedValue(false),
}));
```

---

## Key assertions for integration tests

- When `widgetStore.isWidgetMode = false`: normal layout renders (nav, timer, etc.)
- When `widgetStore.isWidgetMode = true`: `WidgetOverlay` renders; normal layout does NOT render
- 📌 button has `aria-pressed="false"` when off, `aria-pressed="true"` when on
- Clicking 📌 calls `toggleWidgetMode(true)` (first time)
- `WidgetOverlay` exit button calls `toggleWidgetMode(false)` and `setWidgetMode(false)`
- With no active session: "Not tracking" text visible, badge reads "Stopped"
- With active running session: elapsed time visible, badge reads "Running"
- With paused session: badge reads "Paused"
