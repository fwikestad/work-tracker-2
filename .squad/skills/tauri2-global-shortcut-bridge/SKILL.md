# Skill: Tauri 2 Global Shortcut → Frontend Event Bridge

**Stack:** Tauri 2 + Svelte 5 (TypeScript)

---

## Problem

Register a system-wide keyboard shortcut that works even when the app is minimized or out of focus, then open a UI overlay in the frontend.

---

## Solution

### 1. Add dependency (Cargo.toml)

```toml
tauri-plugin-global-shortcut = "2"
```

### 2. Register plugin and shortcut (lib.rs)

```rust
use tauri::Emitter;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.unminimize();
                            let _ = win.set_focus();
                        }
                        let _ = app.emit("my-event", ());
                    }
                })
                .build(),
        )
        .setup(|app| {
            let shortcut = Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::SHIFT),
                Code::KeyS,
            );
            app.handle().global_shortcut().register(shortcut)?;
            Ok(())
        })
}
```

### 3. Listen in frontend (+layout.svelte)

```typescript
import { listen } from '@tauri-apps/api/event';
import { onMount } from 'svelte';

onMount(async () => {
    const unlisten = await listen('my-event', () => {
        uiStore.openSearch();
    });
    return () => unlisten();
});
```

---

## Key Details

- Rust handles native window focus; frontend handles overlay state (clean separation).
- `use tauri::Emitter` must be imported for `app.emit()` to resolve.
- `use tauri_plugin_global_shortcut::GlobalShortcutExt` must be in scope for `.global_shortcut()`.
- Register handler in `Builder::with_handler`, register shortcut(s) in `setup`.
- No capabilities.json changes needed for Rust-to-JS event direction.
- `listen()` returns an `UnlistenFn` — return it from `onMount` to clean up on component destroy.

---

## Common Modifiers

```rust
Modifiers::CONTROL | Modifiers::SHIFT  // Ctrl+Shift+...
Modifiers::SUPER                        // Win/Cmd key
```
