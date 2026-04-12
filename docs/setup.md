# Setup Guide

Complete guide for setting up Work Tracker 2 for development and building for distribution.

---

## Prerequisites

### Required

- **Node.js 18+** (LTS recommended) — https://nodejs.org/
- **Rust** (stable channel) — https://rustup.rs
- **Platform-specific build tools**

### Platform Setup

#### Windows

1. **Microsoft C++ Build Tools** (MSVC):
   - Download from https://visualstudio.microsoft.com/downloads/
   - Look for "Microsoft C++ Build Tools"
   - OR install Visual Studio Community with C++ workload selected

2. Verify installation:
   ```powershell
   rustc --version
   cargo --version
   node --version
   ```

#### macOS

1. **Xcode Command Line Tools**:
   ```bash
   xcode-select --install
   ```

2. Verify:
   ```bash
   rustc --version
   cargo --version
   node --version
   ```

#### Linux (Ubuntu/Debian)

```bash
sudo apt-get install build-essential libwebkit2gtk-4.1-dev libssl-dev
rustc --version
cargo --version
node --version
```

#### Linux (Fedora)

```bash
sudo dnf install gcc g++ webkit2-gtk4.1-devel openssl-devel
rustc --version
cargo --version
node --version
```

#### Linux (Arch)

```bash
sudo pacman -S base-devel webkit2gtk openssl
rustc --version
cargo --version
node --version
```

For complete details, see [Tauri v2 Prerequisites](https://v2.tauri.app/start/prerequisites/).

---

## Installation

### Clone Repository

```bash
git clone https://github.com/wikestad/work-tracker-2.git
cd work-tracker-2
```

### Install Dependencies

```bash
npm install
```

This installs:
- Frontend: Svelte, SvelteKit, Vite, TypeScript
- Tauri CLI and plugins
- Development tools (linters, formatters)

---

## Development Workflow

### Start Development Server

```bash
npm run tauri:dev
```

**What this does**:
1. Builds Rust backend
2. Starts Vite dev server
3. Launches app in development window
4. Enables hot-reload for frontend changes
5. Rust changes require app restart

**First run**: May take 2-5 minutes while Rust dependencies compile.

### Code Changes

**Frontend (Svelte)**: Hot-reloads automatically.  
**Backend (Rust)**: Restart app (`Ctrl+C` and run `npm run tauri:dev` again).

---

## Building for Distribution

### macOS / Linux / Windows

```bash
npm run tauri:build
```

**Output location**: `src-tauri/target/release/bundle/`

**Platforms**:
- **Windows**: `.msi` installer (~10-15 MB)
- **macOS**: `.dmg` package
- **Linux**: `.AppImage` executable

Build time: 3-10 minutes depending on platform and cache.

---

## Testing and Quality

### Run Tests

```bash
npm run test
```

### Lint Code

```bash
npm run lint
```

### Format Code

```bash
npm run format
```

---

## Project Structure

```
work-tracker-2/
│
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── main.rs          # Tauri app entry
│   │   ├── commands/        # IPC command handlers
│   │   │   ├── customers.rs
│   │   │   ├── sessions.rs
│   │   │   └── reports.rs
│   │   ├── services/        # Business logic
│   │   ├── db/              # SQLite connection and setup
│   │   └── models/          # Domain types
│   │
│   ├── migrations/          # SQL schema versions
│   │   ├── 001_init.sql
│   │   └── 002_add_pause.sql
│   │
│   ├── Cargo.toml           # Rust dependencies
│   ├── tauri.conf.json      # Tauri configuration
│   └── capabilities/        # Tauri permission scopes
│
├── src/                     # Svelte frontend
│   ├── lib/
│   │   ├── components/      # UI components
│   │   ├── stores/          # Reactive state (Svelte runes)
│   │   ├── api/             # IPC client wrappers
│   │   ├── types.ts         # TypeScript interfaces
│   │   └── utils/           # Helpers (formatting, shortcuts)
│   │
│   ├── routes/              # SvelteKit pages
│   │   ├── +layout.ts       # Global layout (SSR disabled)
│   │   ├── +layout.svelte
│   │   ├── +page.svelte     # Main tracking view
│   │   └── manage/          # Management routes
│   │
│   ├── app.html             # HTML template
│   ├── app.css              # Global styles
│   └── app.d.ts             # Type definitions
│
├── docs/                    # Documentation
│   ├── architecture.md      # Design decisions and patterns
│   ├── api-reference.md     # IPC command reference
│   ├── setup.md             # This file
│   └── ui-mockup.html       # Interactive prototype
│
├── .github/                 # GitHub workflows and issues
├── package.json             # Frontend + Tauri dependencies
├── vite.config.ts           # Vite bundler config
├── svelte.config.js         # Svelte config
├── tsconfig.json            # TypeScript config
└── README.md                # Quick start
```

---

## Data Storage

### Database Location

Work Tracker stores all data locally in SQLite:

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\work-tracker-2\work_tracker.db` |
| macOS | `~/Library/Application Support/work-tracker-2/work_tracker.db` |
| Linux | `~/.config/work-tracker-2/work_tracker.db` |

### Crash Recovery

If the app crashes while a session is active:

1. On next launch, a recovery dialog appears
2. Choose: **Resume** (close session with current time) or **Discard** (delete orphan)
3. App continues normally after recovery

The database uses **WAL mode** (Write-Ahead Logging) to ensure no data loss, even on unexpected shutdown.

---

## Troubleshooting

### Build fails: "MSVC not found" (Windows)

**Solution**: Install Microsoft C++ Build Tools from https://visualstudio.microsoft.com/

### Build fails: "WebKit development headers not found" (Linux)

**Ubuntu/Debian**:
```bash
sudo apt-get install libwebkit2gtk-4.1-dev libssl-dev
```

**Fedora**:
```bash
sudo dnf install webkit2-gtk4.1-devel openssl-devel
```

### App starts but shows blank window

This is usually a frontend build error. Check the console:
```bash
npm run tauri:dev
# Look for error messages in output
```

### "npm: command not found"

Install Node.js from https://nodejs.org/ (includes npm).

### Rust compilation very slow

Rust compiles in debug mode during `npm run tauri:dev`, which is slower than release builds. This is normal. Subsequent runs reuse compiled cache and are faster.

---

## Next Steps

- Read **[docs/architecture.md](architecture.md)** for system design
- Read **[docs/api-reference.md](api-reference.md)** for IPC command details
- Open **[docs/ui-mockup.html](ui-mockup.html)** in a browser to see the UI design
- Check the main **[README.md](../README.md)** for quick start and keyboard shortcuts
