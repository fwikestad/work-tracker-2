# Work Tracker 2

**Time tracking for consultants. Switch between customers and projects instantly. Never lose a minute of work.**

Work Tracker 2 is a desktop app that helps busy consultants track billable time across multiple customers and projects throughout the day. No cloud accounts, no syncing delays—just you, your work, and a local database that's always there.

---

## What You Can Do

- **Track time with one keystroke** — Switch between customers/projects in seconds (Ctrl+K)
- **Always know what you're working on** — Active timer visible at all times
- **Keyboard-only workflow** — Ctrl+N to add, Ctrl+K to switch, Ctrl+S to stop (optional mouse support)
- **See your day at a glance** — Daily summary breaks down hours by customer and project
- **Export your time** — Generate CSV reports for billing, invoicing, or archiving
- **No cloud, no friction** — All data stored locally; works offline; starts instantly

---

## Getting Started

### System Requirements

- **Windows 10+** (x64) — Download `.msi` installer
- **macOS 10.13+** (Intel or Apple Silicon) — Download `.dmg` installer  
- **Linux (Ubuntu/Debian or similar)** — Download `.AppImage` or `.deb` package

No installation required for portable versions (AppImage).

### Installation

1. Download the latest release for your platform from [GitHub Releases](https://github.com/wikestad/work-tracker-2/releases)
2. Run the installer (or extract `.AppImage`)
3. Launch Work Tracker 2
4. Start tracking!

### First Session (30 seconds)

1. **Create a customer** — Press <kbd>Ctrl+N</kbd>, type the company name (e.g., "Acme Corp"), press <kbd>Enter</kbd>
2. **Create a work order** — Type the project name (e.g., "Web redesign"), press <kbd>Enter</kbd>
3. **Start tracking** — Timer begins immediately
4. **Stop when done** — Press <kbd>Ctrl+S</kbd>, add notes if needed
5. **View your day** — Click "Today" tab to see hours by customer

---

## Key Features

### Timer & Tracking
- Start/stop/pause work sessions with keyboard shortcuts
- Active session always visible at the top
- Running timer updates in real-time
- Pause a session to take a break, resume later

### Customers & Projects
- Create unlimited customers and projects (work orders)
- Organize work by customer; pin favorites for quick access
- Archive inactive projects; unarchive anytime
- Color-code customers for visual recognition (optional)

### Daily Summary
- View total hours worked today, grouped by customer
- See breakdown by project within each customer
- Real-time updates as you track

### Reports & Export
- Filter by date range (today, this week, this month, custom)
- Export to CSV for spreadsheets, billing tools, or personal archive
- Includes session details: dates, times, notes, activity types

### Data & Reliability
- All data stored on your computer (`work_tracker.db`)
- Crash-safe: even if the app crashes, you won't lose time entries
- No account setup; no cloud; no syncing
- Works offline; no internet required

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| <kbd>Ctrl+N</kbd> / <kbd>Cmd+N</kbd> | Quick-add: create and start tracking |
| <kbd>Ctrl+K</kbd> / <kbd>Cmd+K</kbd> | Search and switch projects |
| <kbd>Ctrl+S</kbd> / <kbd>Cmd+S</kbd> | Stop current session |
| <kbd>Ctrl+P</kbd> / <kbd>Cmd+P</kbd> | Pause/resume current session |
| <kbd>Esc</kbd> | Close overlays |
| <kbd>↑↓</kbd> | Navigate search results |
| <kbd>Enter</kbd> | Confirm selection |

---

## Where Is My Data?

All your tracking data is stored in a single file on your computer. You own it.

| Platform | Location |
|----------|----------|
| Windows | `%APPDATA%\work-tracker-2\work_tracker.db` |
| macOS | `~/Library/Application Support/work-tracker-2/work_tracker.db` |
| Linux | `~/.config/work-tracker-2/work_tracker.db` |

You can:
- **Backup** — Copy the `.db` file to cloud storage (Google Drive, Dropbox, etc.)
- **Export** — Use the CSV export feature for long-term archiving
- **Share** — Send reports via email or copy data to spreadsheets

---

## Frequently Asked Questions

**Q: Is my data secure? Will you access it?**  
A: No. Work Tracker 2 is desktop-only. Your data never leaves your computer. No cloud accounts, no servers, no analytics.

**Q: What if the app crashes?**  
A: You won't lose time entries. The app uses write-ahead logging (WAL mode) to ensure every time entry is saved to disk immediately. On restart, the app will recover gracefully.

**Q: Can I use this with a team?**  
A: Currently, Work Tracker 2 is single-user per computer. Phase 2+ may add multi-user features. For now, each team member runs their own copy.

**Q: How do I export my data?**  
A: Click the "Reports" tab, select your date range, and click "Export to CSV". The file downloads to your Downloads folder and opens in Excel or Google Sheets.

**Q: Can I edit past entries?**  
A: Yes. Click any entry in the list to edit duration, notes, activity type, or tags. Changes are saved immediately.

**Q: What happens if I close the app while tracking?**  
A: The active session stays open. When you restart the app, it asks if you want to close that session or discard it.

---

## Developer Setup

For developers contributing to Work Tracker 2:

**Prerequisites**:
- Node.js 18+ (LTS)
- Rust (stable)
- Platform-specific tools (see [docs/development.md](docs/development.md))

**Get Started**:
```bash
git clone https://github.com/wikestad/work-tracker-2.git
cd work-tracker-2
npm install
npm run tauri:dev
```

Full setup instructions: **[docs/development.md](docs/development.md)**

---

## Documentation

- **[docs/development.md](docs/development.md)** — Developer setup, testing, and contribution guide
- **[docs/architecture.md](docs/architecture.md)** — System design, three-layer architecture, tech stack decisions
- **[docs/data-model.md](docs/data-model.md)** — Database schema, tables, indexes, and relationships
- **[docs/features.md](docs/features.md)** — Complete feature list (implemented and planned)

---

## Contributing

Contributions welcome! Before starting:

1. Read **[docs/development.md](docs/development.md)** for setup and conventions
2. Fork the repo, create a feature branch
3. Make changes and test locally: `npm run tauri:dev`
4. Run the full test suite: `npm test && (cd src-tauri && cargo test)`
5. Submit a pull request with a clear description

---

## Roadmap

**Phase 1 (Current)** — Core time tracking for single consultant  
✅ Timer, customers/projects, daily summary, reports, exports, crash recovery

**Phase 2** — Multi-customer workflows & favorites  
📋 Pinned favorites, system tray quick-switch, color-coded customers, paused sessions

**Phase 3** — Advanced reporting  
📋 Background running, report generation, archive management

**Phase 4+** — Team & integrations  
📋 Multi-user per computer, third-party integrations, local backups, notifications

---

## Issues, Questions, or Suggestions?

- **GitHub Issues** — Report bugs and feature requests: [github.com/wikestad/work-tracker-2/issues](https://github.com/wikestad/work-tracker-2/issues)
- **Discussions** — Ask questions or share ideas: [github.com/wikestad/work-tracker-2/discussions](https://github.com/wikestad/work-tracker-2/discussions)

---

## License

[Add your license here (e.g., MIT, GPL-3.0)]

---

## Built With

- **[Tauri 2](https://v2.tauri.app/)** — Cross-platform desktop framework
- **[Rust](https://www.rust-lang.org/)** — Fast, memory-safe backend
- **[Svelte 5](https://svelte.dev/)** — Lightweight reactive UI
- **[SQLite](https://www.sqlite.org/)** — Reliable local database
- **[TypeScript](https://www.typescriptlang.org/)** — Type-safe frontend
