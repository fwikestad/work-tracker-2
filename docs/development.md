# Development Guide

This guide covers setting up a local development environment, running tests, and building Work Tracker 2.

---

## Prerequisites

### Required Tools

- **Node.js 18+ (LTS recommended)** — https://nodejs.org/
  - Verify: `node --version` (should be v18 or higher)
- **Rust (stable)** — https://rustup.rs
  - Verify: `rustc --version` and `cargo --version`
- **Git** — https://git-scm.com/

### Platform-Specific Build Tools

#### Windows

1. **Microsoft C++ Build Tools**
   - Download: https://visualstudio.microsoft.com/downloads/ → Look for "Microsoft C++ Build Tools"
   - Alternative: Install Visual Studio Community with C++ workload enabled
   - Verify: `cargo build --help` should work without errors

#### macOS

1. **Xcode Command Line Tools**
   ```bash
   xcode-select --install
   ```
   - Verify: `clang --version` should show "Apple clang"

#### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install build-essential libwebkit2gtk-4.1-dev libssl-dev libjavascriptcoregtk-4.1-dev
```

Verify:
```bash
rustc --version
cargo --version
node --version
```

---

## Clone and Setup

### 1. Clone the Repository

```bash
git clone https://github.com/wikestad/work-tracker-2.git
cd work-tracker-2
```

### 2. Install Dependencies

```bash
npm install
cd src-tauri && cargo build && cd ..
```

This:
- Installs Node.js dependencies (Tauri CLI, Vite, Svelte, TypeScript, test runners)
- Builds the Rust backend to verify the toolchain works

If this step fails, verify your platform tools (see Prerequisites above).

---

## Development Workflow

### Running in Development Mode

```bash
npm run tauri:dev
```

This:
1. Starts the Vite dev server on `http://localhost:1420`
2. Launches the Tauri app window in debug mode
3. Enables hot module reload (HMR) for quick feedback on frontend changes
4. Shows Tauri debug console (right-click → Inspect Element)

The app window opens with your local frontend code. Backend changes require restarting.

### Running Tests

#### Frontend Tests (Vitest)

```bash
npm test
```

Runs all tests in:
- `src/lib/**/*.test.ts`
- `src/lib/__tests__/**/*.test.ts`

Current test count: **~55 tests** covering components, stores, and smoke tests.

Watch mode:
```bash
npm test:watch
```

Coverage report:
```bash
npm run test:coverage
```

#### Backend Tests (Rust)

```bash
cd src-tauri
cargo test
```

Runs all Rust integration and unit tests. Current count: **~7 tests** covering session management, customer CRUD, and database operations.

Test in release mode (slower, more optimizations):
```bash
cd src-tauri
cargo test --release
```

### Linting and Formatting

```bash
npm run lint        # Check TypeScript and Rust style
npm run format      # Auto-format code (if available)
```

For Rust specifically:
```bash
cd src-tauri
cargo clippy -- -D warnings  # Lint with strict warnings
cargo fmt -- --check         # Check formatting
cargo fmt                    # Auto-format
```

---

## Building for Release

### Production Build

```bash
npm run tauri:build
```

This:
1. Compiles the frontend (Vite, TypeScript)
2. Compiles the Rust backend in release mode
3. Bundles everything with Tauri
4. Creates platform-specific installers:
   - **Windows**: `.msi` + `.exe`
   - **macOS**: `.dmg` + `.app`
   - **Linux**: `.AppImage` + `.deb`

Output location: `src-tauri/target/release/bundle/`

### Build Time

- Cold build (first time): ~8-10 minutes
- Warm build (cached artifacts): ~3-4 minutes
- Most time spent on Rust compilation

---

## Project Structure

```
work-tracker-2/
├── src/                          # Frontend (TypeScript + Svelte)
│   ├── lib/
│   │   ├── components/           # Reusable UI components
│   │   ├── stores/               # Svelte stores (timer, session, UI state)
│   │   ├── api/                  # Tauri IPC client (invoke commands)
│   │   ├── __tests__/            # Test files
│   │   └── ...
│   ├── routes/                   # SvelteKit pages and layouts
│   └── app.html, app.css
│
├── src-tauri/                    # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── commands/             # Tauri IPC command handlers
│   │   ├── services/             # Business logic (sessions, summaries)
│   │   ├── models/               # Data models (Session, Customer, etc.)
│   │   ├── db/                   # Database initialization and connection
│   │   ├── lib.rs, main.rs       # Tauri app setup
│   │   └── tray.rs               # System tray integration
│   ├── migrations/               # Database schema SQL
│   ├── Cargo.toml                # Rust dependencies
│   └── target/                   # Build artifacts (ignored)
│
├── docs/                         # Documentation
│   ├── development.md            # This file
│   ├── architecture.md           # System design
│   ├── data-model.md             # Database schema
│   ├── features.md               # Feature reference
│   └── ...
│
├── .github/workflows/            # CI/CD pipelines
│   ├── ci.yml                    # Lint, test, build check
│   ├── coverage.yml              # Code coverage reporting
│   ├── release.yml               # Multi-platform builds
│   ├── audit.yml                 # Security audits
│   └── dependabot.yml            # Dependency updates
│
├── package.json                  # Node.js scripts and deps
├── tsconfig.json                 # TypeScript configuration
├── vite.config.ts                # Vite bundler config
├── vitest.config.ts              # Vitest test runner config
└── README.md                      # User-facing documentation
```

---

## Key Conventions

### TypeScript / Frontend

- **File naming**: camelCase (e.g., `sessionStore.svelte.ts`, `TimerDisplay.svelte`)
- **Component naming**: PascalCase (e.g., `<SearchSwitch />`)
- **Imports**: Use `@` alias for `src/lib` (configured in `svelte.config.js`)
- **Stores**: Use Svelte 5 runes (`let foo = $state(...)`)
- **IPC calls**: Use `src/lib/api/*` modules, not direct `invoke` calls

### Rust / Backend

- **Module organization**: Commands → Services → Models → Database
- **Tauri invoke parameter names**: Use camelCase (e.g., `work_order_id` in params is passed as `workOrderId` from frontend)
- **Error handling**: Return `Result<T, AppError>` where `AppError` implements `Serialize`
- **Database**: Always use transaction for multi-step operations
- **Session rules**: At most one active session at a time; switching stops the previous one

### Database

- **Schema**: Defined in `src-tauri/migrations/` (SQL files)
- **Timestamps**: ISO 8601 UTC format (e.g., `2026-04-13T12:34:56Z`)
- **Foreign keys**: Enabled with `PRAGMA foreign_keys = ON`
- **Indexes**: Created for frequently queried columns (start_time, customer_id, work_order_id, end_time)
- **Crash safety**: WAL mode + synchronous NORMAL pragma

---

## Common Development Tasks

### Add a New Command (Backend → Frontend)

1. **Define the handler in Rust** (`src-tauri/src/commands/my_command.rs`):
   ```rust
   #[tauri::command]
   pub async fn my_command(param: String, state: State<'_, AppState>) -> Result<String, AppError> {
       // Implementation
       Ok(result)
   }
   ```

2. **Register in** `src-tauri/src/commands/mod.rs`:
   ```rust
   pub mod my_command;
   // In lib.rs: .invoke_handler(tauri::generate_handler![my_command::my_command])
   ```

3. **Call from Frontend** (`src/lib/api/commands.ts`):
   ```typescript
   import { invoke } from '@tauri-apps/api/core';
   
   export async function myCommand(param: string): Promise<string> {
       return invoke('my_command', { param });
   }
   ```

4. **Use in Components**:
   ```svelte
   import { myCommand } from '$lib/api/commands';
   let result = await myCommand('test');
   ```

### Add a Database Migration

1. Create a new SQL file in `src-tauri/migrations/` (e.g., `003_my_migration.sql`)
2. Define the schema changes (e.g., `ALTER TABLE ...`, `CREATE INDEX ...`)
3. Update the migration runner in `src-tauri/src/db/mod.rs` to include the new file
4. Rebuild: `cd src-tauri && cargo build`

### Write a Frontend Component Test

```typescript
// src/lib/components/MyComponent.test.ts
import { render, screen } from '@testing-library/svelte';
import MyComponent from './MyComponent.svelte';

describe('MyComponent', () => {
  it('renders correctly', () => {
    render(MyComponent);
    expect(screen.getByText('Expected text')).toBeTruthy();
  });
});
```

Run: `npm test`

### Write a Rust Test

```rust
// src-tauri/src/services/my_service.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        let result = my_function(42);
        assert_eq!(result, expected_value);
    }
}
```

Run: `cd src-tauri && cargo test`

---

## Debugging

### Frontend Debugging

1. In dev mode (`npm run tauri:dev`), right-click the window → **Inspect Element**
2. Opens the browser dev tools (Chrome DevTools)
3. Use console, network tab, debugger as normal

### Rust Debugging

1. Set a breakpoint in VS Code (red dot on line number)
2. Run with debugger:
   ```bash
   cd src-tauri
   cargo build
   # Use a Rust debugger like `rust-gdb` or VS Code's CodeLLDB extension
   ```

Alternative: Add debug logging with `eprintln!` macros and observe in dev console.

### Database Inspection

SQLite database location:
- **Development**: `src-tauri/target/debug/bundle/*/` or wherever Tauri places it in debug mode
- **After production build**: Check platform-specific paths in README.md

Use a SQLite browser (e.g., https://sqlitebrowser.org/) to inspect the database directly.

---

## Troubleshooting

### `npm install` fails

**Symptom**: Error installing native dependencies (sharp, node-gyp)  
**Solution**: Ensure build tools are installed (see Prerequisites). On Linux, run `apt-get install python3-dev`.

### `npm run tauri:dev` crashes immediately

**Symptom**: Tauri app window doesn't open  
**Solution**: 
1. Check the terminal for error messages
2. Verify Rust and Node.js are installed: `rustc --version && node --version`
3. Try rebuilding: `cd src-tauri && cargo clean && cargo build && cd ..`

### Tests fail with "cannot find module"

**Symptom**: Vitest/Jest import errors  
**Solution**: Run `npm install` and restart the test runner. Clear cache: `rm -rf node_modules/.vite`

### `cargo test` panics with "database lock"

**Symptom**: Rust tests fail with SQLITE_BUSY  
**Solution**: This can happen if tests run in parallel on the same in-memory database. Add `#[serial]` to test functions (requires `serial_test` crate, or run with `--test-threads=1`).

### Changes in Rust backend aren't reflected in dev mode

**Solution**: Restart `npm run tauri:dev`. Frontend HMR works, but backend changes require a rebuild.

---

## Performance Tips

- **Frontend**: Use Chrome DevTools Performance tab to profile rendering
- **Rust**: Use `cargo build --release` for faster execution; profile with `cargo profiling`
- **Database**: Add indexes for frequently queried columns; use `EXPLAIN QUERY PLAN` to verify
- **Build time**: Cache will speed up subsequent builds; use `sccache` for Rust (Phase 3+)

---

## CI/CD & Release

### Running CI Locally

Simulate the CI pipeline on your machine:

```bash
npm run lint              # TypeScript + Rust linting
npm test                  # Frontend tests
cd src-tauri && cargo test && cd ..  # Backend tests
npm run tauri:build       # Full release build
```

### Creating a Release

1. Update `package.json` version (e.g., `0.2.0`)
2. Commit: `git commit -m "v0.2.0: description"`
3. Tag: `git tag v0.2.0`
4. Push: `git push && git push --tags`
5. CI/CD automatically builds and publishes to GitHub Releases

See `.github/workflows/release.yml` for details.

---

## Further Reading

- **[docs/architecture.md](architecture.md)** — System design and three-layer architecture
- **[docs/data-model.md](data-model.md)** — Database schema and queries
- **[docs/features.md](features.md)** — Complete feature inventory and roadmap
- **[Tauri v2 Guide](https://v2.tauri.app/)** — Official Tauri documentation
- **[Svelte 5 Docs](https://svelte.dev/)** — Svelte and reactive runes
- **[Rust Book](https://doc.rust-lang.org/book/)** — Rust fundamentals
