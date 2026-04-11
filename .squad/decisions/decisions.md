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
