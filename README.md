# Work Tracker 2

**Native desktop time tracker for consultants** — fast context switching, zero data loss.

Built with **Tauri 2** + **Svelte 5** + **TypeScript** + **SQLite** for speed, reliability, and offline-first operation.

---

## What It Does

- **Track time across multiple customers and work orders** — Switch projects instantly without losing data
- **Instant context switching** — Jump between tasks in less than 3 seconds
- **Active timer always visible** — Know exactly what you're working on right now
- **Daily summary** — See how your time breaks down by customer and project
- **CSV export** — Archive and share your tracked time
- **Zero data loss** — Built-in crash recovery ensures your work is never lost

---

## Features (Phase 1 MVP)

✅ **Time Tracking**
- Start and stop sessions with a single click
- Automatic timer display (or manual time entry)
- Inline editing of notes and duration
- Real-time active session indicator

✅ **Context Switching**
- Search to find and switch projects instantly
- Recent items for quick access
- Quick-add overlay to create a new project and start tracking in one action (Ctrl+N)

✅ **Reporting**
- Daily summary by customer and project
- CSV export for any date range
- Session-level metadata (notes, activity type)

✅ **Reliability**
- Local SQLite database with crash recovery
- WAL mode ensures zero data loss
- Orphan session detection on startup

---

## Screenshots

For an interactive UI mockup, see **[docs/ui-mockup.html](docs/ui-mockup.html)** — open in your browser.

---

## Prerequisites

### Required

- **Node.js** 18 or later (LTS recommended)
- **Rust** (stable channel, installed via [rustup.rs](https://rustup.rs))
- **Platform build tools**:

  **Windows**: Microsoft C++ Build Tools (MSVC)
  ```
  Download from: https://visualstudio.microsoft.com/downloads/
  Or install Visual Studio Community with C++ workload
  ```

  **macOS**: Xcode command line tools
  ```bash
  xcode-select --install
  ```

  **Linux**: Build essentials and WebKit development headers
  ```bash
  # Ubuntu/Debian
  sudo apt-get install build-essential libwebkit2gtk-4.1-dev libssl-dev

  # Fedora
  sudo dnf install gcc g++ webkit2-gtk4.1-devel openssl-devel

  # Arch
  sudo pacman -S base-devel webkit2gtk openssl
  ```

### Full Tauri Prerequisites

For complete details, refer to the [Tauri v2 Prerequisites Guide](https://v2.tauri.app/start/prerequisites/).

---

## Getting Started

### 1. Clone and install dependencies

```bash
git clone https://github.com/your-username/work-tracker-2.git
cd work-tracker-2
npm install
```

### 2. Start development server

```bash
npm run tauri:dev
```

The app will launch in development mode. Any changes to frontend code hot-reload automatically.

### 3. Build for distribution

```bash
npm run tauri:build
```

The compiled application will be in `src-tauri/target/release/bundle/`:
- **Windows**: `.msi` installer (~10-15 MB)
- **macOS**: `.dmg` package
- **Linux**: `.AppImage` executable

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| <kbd>Ctrl</kbd>+<kbd>N</kbd> (Windows/Linux) or <kbd>Cmd</kbd>+<kbd>N</kbd> (macOS) | Quick-add: Create and start tracking a new work order |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> / <kbd>Cmd</kbd>+<kbd>K</kbd> | Open search / context switcher |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> / <kbd>Cmd</kbd>+<kbd>S</kbd> | Stop current session |
| <kbd>Escape</kbd> | Close overlay / cancel |
| <kbd>Enter</kbd> | Confirm action or switch to selected item |
| <kbd>Arrow Keys</kbd> | Navigate search results |
| <kbd>Tab</kbd> | Move between form fields |

---

## Project Structure

```
work-tracker-2/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # IPC command handlers (start_session, stop_session, etc.)
│   │   ├── db/             # SQLite interface and migrations
│   │   ├── services/       # Business logic (session switching, summaries)
│   │   ├── models/         # Domain types (Customer, WorkOrder, Session)
│   │   └── main.rs
│   ├── migrations/         # SQL schema migrations
│   └── Cargo.toml
│
├── src/                    # Svelte frontend
│   ├── lib/
│   │   ├── components/     # UI components (Timer, SessionList, QuickAdd, etc.)
│   │   ├── stores/         # Reactive state (timer, sessions, ui)
│   │   ├── api/            # IPC client wrappers
│   │   └── utils/          # Formatters, keyboard shortcuts
│   ├── routes/             # SvelteKit pages
│   ├── app.html
│   ├── app.css
│   └── app.d.ts
│
├── docs/
│   ├── architecture.md     # Technical decisions and system design
│   ├── api-reference.md    # Full command documentation
│   └── ui-mockup.html      # Interactive UI prototype
│
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
└── README.md               # This file
```

**Key directories**:
- **src-tauri/src/commands/** — All Tauri IPC handlers (synchronous with backend)
- **src-tauri/src/services/** — Business logic independent of IPC layer (testable)
- **src/lib/stores/** — Svelte 5 runes for reactive UI state
- **docs/architecture.md** — Definitive reference for technical decisions

---

## Data Storage

All data is stored locally on your machine using SQLite. No cloud sync or internet required.

### Default Data Location

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\work-tracker-2\work_tracker.db` |
| macOS | `~/Library/Application Support/work-tracker-2/work_tracker.db` |
| Linux | `~/.config/work-tracker-2/work_tracker.db` |

To locate the database on your system:
- Open the app's preferences or look in the app data directory above
- The database file is a standard SQLite 3 file (compatible with any SQLite viewer)

---

## Development

### Running Tests

```bash
npm run test
```

### Formatting and Linting

```bash
npm run lint
npm run format
```

### Building Documentation

Docs are in markdown (read in any editor or via GitHub). The architecture document (`docs/architecture.md`) is the source of truth for design decisions.

---

## Contributing

Work Tracker 2 is developed by Fredrik Kristiansen Wikestad with AI-assisted engineering.

Before making changes:
1. Read **docs/architecture.md** for design context
2. Check that changes align with the Phase 1 scope (see checklist in architecture.md)
3. Run tests and linters
4. Follow the existing code style (Svelte components, Rust conventions)

---

## License

[Specify your license here, e.g., MIT, Apache-2.0, etc.]

---

## Support & Feedback

Report bugs or request features via [GitHub Issues](https://github.com/your-username/work-tracker-2/issues).

---

## References

- **Tauri Documentation** — https://v2.tauri.app
- **Svelte 5 Runes** — https://svelte.dev/docs/runes
- **SQLite** — https://www.sqlite.org/
- **Architecture & Design** — See `docs/architecture.md`
- **API Reference** — See `docs/api-reference.md`

---

*Last updated: 2026-04-11*
