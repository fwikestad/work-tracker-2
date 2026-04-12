# Work Tracker 2

**Native desktop time tracker for consultants** — fast context switching, zero data loss.  
Built with **Tauri 2** + **Svelte 5** + **TypeScript** + **SQLite**.

---

## What It Does

- **Track time across customers and projects** — Switch instantly without data loss
- **Active timer always visible** — Know what you're working on right now
- **Keyboard-first interface** — Ctrl+N to quick-add, Ctrl+K to search, Ctrl+S to stop
- **Daily summary and exports** — See your time breakdown by customer/project, export to CSV
- **Offline-first** — All data stored locally; no cloud required

---

## Quick Start

```bash
# Install and run
git clone https://github.com/wikestad/work-tracker-2.git
cd work-tracker-2
npm install
npm run tauri:dev
```

**See [docs/setup.md](docs/setup.md) for detailed prerequisites and build instructions.**

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| <kbd>Ctrl+N</kbd> / <kbd>Cmd+N</kbd> | Quick-add: create and start tracking |
| <kbd>Ctrl+K</kbd> / <kbd>Cmd+K</kbd> | Search and switch projects |
| <kbd>Ctrl+S</kbd> / <kbd>Cmd+S</kbd> | Stop current session |
| <kbd>Esc</kbd> | Close overlay |
| <kbd>↑↓</kbd> | Navigate results |

---

## Documentation

- **[docs/setup.md](docs/setup.md)** — Prerequisites, platform-specific tools, detailed setup
- **[docs/architecture.md](docs/architecture.md)** — System design, database schema, technical decisions
- **[docs/api-reference.md](docs/api-reference.md)** — Tauri IPC commands with TypeScript examples
- **[docs/ui-mockup.html](docs/ui-mockup.html)** — Interactive UI prototype (open in browser)

---

## Local Data Storage

All tracking data lives on your machine. No internet required.

| Platform | Location |
|----------|----------|
| Windows | `%APPDATA%\work-tracker-2\work_tracker.db` |
| macOS | `~/Library/Application Support/work-tracker-2/work_tracker.db` |
| Linux | `~/.config/work-tracker-2/work_tracker.db` |

---

## Project Structure

- **src-tauri/** — Rust backend (commands, business logic, SQLite)
- **src/** — Svelte frontend (components, stores, routes)
- **docs/** — Architecture, API reference, UI prototype

---

## License

[Specify your license here]

---

## Contributing

Before making changes, read **[docs/architecture.md](docs/architecture.md)** for design context and run tests:

```bash
npm run test        # Run test suite
npm run lint        # Check code style
npm run format      # Auto-format code
```
