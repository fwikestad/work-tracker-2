# Skill: Tauri 2 Tray Event Handlers + App State Access

## Problem

In Tauri 2 `on_menu_event` and `on_tray_icon_event` closures, accessing managed state (via `app.state::<T>()`) and then calling `app.emit()` in the same scope produces a borrow checker error:

```
error[E0597]: `state` does not live long enough
```

Root cause: `app.state::<T>()` returns `State<'_, T>` which borrows `app`. The `MutexGuard` from `state.db.lock()` borrows `state`. Both borrows must end before `app.emit()` is called — but the default borrow scope makes them overlap.

---

## Solution Pattern

**Always scope DB access in a block that returns a plain value, then call `app.emit()` outside the block.**

```rust
fn my_menu_handler(app: &AppHandle, event: tauri::menu::MenuEvent) {
    // Scope 1: acquire state + conn, do work, return a plain bool
    let did_work = {
        let state = app.state::<AppState>();
        let result = match state.db.lock() {
            Ok(conn) => {
                // do database work
                some_service::do_thing(&conn).is_ok()
            }
            Err(_) => false,
        };
        result  // ← named variable: ensures all match temporaries drop before `state`
    }; // ← state and MutexGuard are fully dropped here

    // Scope 2: now safe to use `app` for emit / window operations
    if did_work {
        let _ = app.emit("my-event", payload);
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.set_focus();
        }
    }
}
```

**For `if let` without a return value** — add a trailing semicolon to the `if let` block:

```rust
{
    let state = app.state::<AppState>();
    if let Ok(conn) = state.db.lock() {
        let _ = some_service::do_thing(&conn);
    };  // ← semicolon forces expression → statement, temporaries drop sooner
}
app.exit(0); // safe: state already dropped
```

---

## Why the Named Variable Matters

Without a named variable, the match expression's temporary `Result<MutexGuard, ...>` may not be dropped until the end of the enclosing block. With `let result = match ...; result`, the compiler knows `result` is a `bool` (not a borrow), so the `MutexGuard` and `State` temporaries can be dropped before `result` is used.

---

## Tauri 2 Image API

`tauri::image::Image::from_bytes()` does **not** exist in Tauri 2.10.x.

For tray icons from raw pixels use:
```rust
tauri::image::Image::new_owned(rgba_vec: Vec<u8>, width: u32, height: u32)
```

For PNG/ICO files on disk, use `tauri::image::Image::from_path()` (requires the file to exist at runtime).

---

## Required Imports for Tray Event Handlers

```rust
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle,
    Emitter,   // ← required for app.emit(); NOT in prelude
    Manager,   // ← required for app.state(), app.get_webview_window()
    Wry,
};
```

`Emitter` and `Manager` must be explicitly imported — they are not re-exported in `tauri::prelude`.

---

## Reference Implementation

`src-tauri/src/tray.rs` in `work-tracker-2` is the canonical implementation of this pattern.
