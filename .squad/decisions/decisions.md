# Work Tracker 2 — Decisions Log

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
