# Work Tracker 2 — Architecture Document

**Version**: 1.0  
**Author**: Han (Lead)  
**Date**: 2026-04-11  
**Status**: Draft — Ready for Review

---

## Executive Summary

This document defines the architecture for Work Tracker 2, a desktop time-tracking application for consultants. The architecture prioritizes:

1. **Speed** — Context switching under 3 seconds
2. **Reliability** — Zero data loss, even on crashes
3. **Simplicity** — Solo developer + AI team can build and maintain it
4. **Native feel** — Small footprint, responsive UI

---

## 1. Technology Stack Decision

### Recommendation: Tauri 2 + Svelte 5 + TypeScript + SQLite

After evaluating the options against project requirements, I recommend **Option A: Tauri 2 + Svelte 5 + TypeScript + SQLite**.

### Options Evaluated

| Criterion | Tauri + Svelte | Electron + React | Tauri + React |
|-----------|----------------|------------------|---------------|
| Bundle size | ~10-15 MB | ~150-200 MB | ~10-15 MB |
| Memory usage | ~50-80 MB | ~150-300 MB | ~50-80 MB |
| Dev velocity | Medium | High | Medium-High |
| Rust learning curve | Yes | No | Yes |
| Ecosystem maturity | Growing | Mature | Growing |
| Crash safety (backend) | Rust = excellent | Node = good | Rust = excellent |
| Native feel | Excellent | Good | Excellent |

### Why Tauri + Svelte Wins

1. **Bundle size matters for distribution** — A 10MB app is trivial to share. A 150MB app feels heavy. Fredrik may send this to clients or colleagues.

2. **Svelte's simplicity aligns with project goals** — Svelte 5's runes provide reactive state without boilerplate. For a single-developer team, less code = fewer bugs.

3. **Rust backend provides crash safety** — SQLite operations in Rust are memory-safe. The backend won't crash from memory leaks or null pointer issues that can plague Electron's main process.

4. **Performance headroom** — Timer updates, search filtering, and list rendering will be snappy without optimization effort.

5. **Native system tray** — Tauri's system tray support is first-class and cross-platform.

### Tradeoff: Rust Learning Curve

Fredrik may not be deep in Rust. Mitigation:

- **The backend is small** — ~500-800 lines of Rust for Phase 1
- **IPC commands are simple** — Just functions that take params and return results
- **AI assistance** — Copilot excels at generating Rust from clear specs
- **Recommended resources**:
  - [The Rust Book, Chapter 1-6](https://doc.rust-lang.org/book/) (basics)
  - [Tauri v2 Guide](https://v2.tauri.app/start/)
  - [tauri-plugin-sql docs](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/sql)

### SQLite Configuration

Use **tauri-plugin-sql** (official Tauri plugin) for SQLite access:

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-sql = { version = "2", features = ["sqlite"] }
```

This provides:
- Async SQLite operations from Rust commands
- Automatic connection pooling
- Cross-platform path handling

---

## 2. Application Architecture

### Three-Layer Design

```
┌─────────────────────────────────────────────────────────────────┐
│                     FRONTEND (Svelte 5)                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  Components │  │   Stores    │  │   IPC Client (api/)     │  │
│  │  (UI)       │  │   (State)   │  │   invoke('command')     │  │
│  └─────────────┘  └─────────────┘  └──────────┬──────────────┘  │
└────────────────────────────────────────────────┼────────────────┘
                                                 │ Tauri IPC
┌────────────────────────────────────────────────▼────────────────┐
│                   BACKEND (Rust + Tauri)                        │
│  ┌─────────────────────┐  ┌─────────────────────────────────┐   │
│  │  Commands (IPC)     │  │  Domain Logic (services)        │   │
│  │  start_session()    │  │  - Session switching            │   │
│  │  stop_session()     │  │  - Validation                   │   │
│  │  get_daily_summary()│  │  - Calculations                 │   │
│  └──────────┬──────────┘  └────────────────┬────────────────┘   │
│             │                              │                    │
│  ┌──────────▼──────────────────────────────▼────────────────┐   │
│  │                    Data Layer (db/)                      │   │
│  │  - SQLite via tauri-plugin-sql                           │   │
│  │  - WAL mode, synchronous=NORMAL                          │   │
│  │  - Migrations embedded at compile time                   │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  work_tracker.db │
                    │  (SQLite file)  │
                    └─────────────────┘
```

### IPC Boundary (JS → Rust → SQLite)

All backend operations are exposed as Tauri commands:

```typescript
// Frontend: src/lib/api/sessions.ts
import { invoke } from '@tauri-apps/api/core';

export async function startSession(workOrderId: string): Promise<Session> {
  return invoke('start_session', { workOrderId });
}

export async function stopSession(sessionId: string, notes?: string): Promise<Session> {
  return invoke('stop_session', { sessionId, notes });
}
```

```rust
// Backend: src-tauri/src/commands/sessions.rs
#[tauri::command]
async fn start_session(
    work_order_id: String,
    state: State<'_, DbPool>,
) -> Result<Session, AppError> {
    let db = state.inner();
    
    // Transaction: stop any active session, then start new
    db.transaction(|tx| {
        services::sessions::stop_active_session(tx)?;
        services::sessions::create_session(tx, work_order_id)
    }).await
}
```

### State Management (Frontend)

Use **Svelte 5 Runes** for reactive state:

```typescript
// src/lib/stores/timer.svelte.ts

let activeSession = $state<Session | null>(null);
let elapsedSeconds = $state(0);

// Derived state
let isTracking = $derived(activeSession !== null);
let formattedTime = $derived(formatDuration(elapsedSeconds));

// Actions
export function setActiveSession(session: Session | null) {
  activeSession = session;
  if (session) {
    elapsedSeconds = Math.floor((Date.now() - session.startedAt) / 1000);
  }
}
```

For cross-component state, use a simple store module pattern rather than a heavy state management library.

---

## 3. Project Structure

```
work-tracker-2/
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── commands/             # Tauri command handlers
│   │   │   ├── mod.rs
│   │   │   ├── customers.rs
│   │   │   ├── work_orders.rs
│   │   │   ├── sessions.rs
│   │   │   └── reports.rs
│   │   ├── db/                   # Database layer
│   │   │   ├── mod.rs
│   │   │   ├── migrations.rs     # Embedded migrations
│   │   │   ├── pool.rs           # Connection management
│   │   │   └── schema.sql        # Reference schema (embedded)
│   │   ├── services/             # Business logic
│   │   │   ├── mod.rs
│   │   │   ├── session_service.rs
│   │   │   └── summary_service.rs
│   │   ├── models/               # Domain types
│   │   │   ├── mod.rs
│   │   │   ├── customer.rs
│   │   │   ├── work_order.rs
│   │   │   ├── session.rs
│   │   │   └── error.rs
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── migrations/               # SQL migration files
│   │   └── 001_initial_schema.sql
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
│
├── src/                          # Svelte frontend
│   ├── lib/
│   │   ├── components/           # Reusable UI components
│   │   │   ├── Timer.svelte
│   │   │   ├── SessionList.svelte
│   │   │   ├── QuickAdd.svelte
│   │   │   ├── SearchSwitch.svelte
│   │   │   ├── DailySummary.svelte
│   │   │   └── CustomerPicker.svelte
│   │   ├── stores/               # Reactive state
│   │   │   ├── timer.svelte.ts
│   │   │   ├── sessions.svelte.ts
│   │   │   └── ui.svelte.ts
│   │   ├── api/                  # IPC client wrappers
│   │   │   ├── index.ts
│   │   │   ├── customers.ts
│   │   │   ├── workOrders.ts
│   │   │   ├── sessions.ts
│   │   │   └── reports.ts
│   │   └── utils/
│   │       ├── formatters.ts
│   │       └── shortcuts.ts
│   ├── routes/                   # SvelteKit routes (if using)
│   │   ├── +layout.svelte
│   │   ├── +page.svelte          # Main tracking view
│   │   ├── customers/
│   │   │   └── +page.svelte
│   │   └── settings/
│   │       └── +page.svelte
│   ├── app.html
│   ├── app.css
│   └── app.d.ts
│
├── docs/
│   ├── architecture.md           # This document
│   └── api-reference.md          # Command documentation
│
├── .squad/                       # Squad agent state
├── .github/
│   └── copilot-instructions.md   # Framework instructions
│
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
└── README.md
```

---

## 4. Core Data Model (Schema Draft)

### SQLite Schema

```sql
-- migrations/001_initial_schema.sql

-- Enable WAL mode and appropriate sync level for crash safety
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

-- ============================================
-- CUSTOMERS
-- ============================================
CREATE TABLE customers (
    id TEXT PRIMARY KEY NOT NULL,           -- UUID
    name TEXT NOT NULL,
    code TEXT,                              -- Optional short code (e.g., "ACME")
    color TEXT,                             -- Hex color for UI (e.g., "#3B82F6")
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    archived_at TEXT                        -- Soft delete
);

CREATE INDEX idx_customers_name ON customers(name);
CREATE INDEX idx_customers_code ON customers(code);
CREATE INDEX idx_customers_archived ON customers(archived_at);

-- ============================================
-- WORK ORDERS
-- ============================================
CREATE TABLE work_orders (
    id TEXT PRIMARY KEY NOT NULL,           -- UUID
    customer_id TEXT NOT NULL,
    name TEXT NOT NULL,
    code TEXT,                              -- Optional reference code
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',  -- 'active', 'paused', 'closed'
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    archived_at TEXT,                       -- Soft delete
    
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX idx_work_orders_customer_id ON work_orders(customer_id);
CREATE INDEX idx_work_orders_status ON work_orders(status);
CREATE INDEX idx_work_orders_archived ON work_orders(archived_at);

-- ============================================
-- TIME SESSIONS
-- ============================================
CREATE TABLE time_sessions (
    id TEXT PRIMARY KEY NOT NULL,           -- UUID
    work_order_id TEXT NOT NULL,
    start_time TEXT NOT NULL,               -- ISO 8601 timestamp
    end_time TEXT,                          -- NULL if session is active
    duration_seconds INTEGER,               -- Calculated or manual
    duration_override INTEGER,              -- User override (NULL = use calculated)
    activity_type TEXT,                     -- 'meeting', 'development', 'design', 'admin', etc.
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    FOREIGN KEY (work_order_id) REFERENCES work_orders(id) ON DELETE CASCADE
);

CREATE INDEX idx_sessions_start_time ON time_sessions(start_time);
CREATE INDEX idx_sessions_end_time ON time_sessions(end_time);
CREATE INDEX idx_sessions_work_order_id ON time_sessions(work_order_id);

-- Composite index for date range queries (common for daily/weekly summaries)
CREATE INDEX idx_sessions_date_range ON time_sessions(start_time, end_time);

-- ============================================
-- ACTIVE SESSION (Singleton for crash recovery)
-- ============================================
CREATE TABLE active_session (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Enforces singleton
    session_id TEXT,                        -- FK to time_sessions.id (NULL if no active session)
    work_order_id TEXT,                     -- Denormalized for quick display
    started_at TEXT,                        -- When tracking began
    last_heartbeat TEXT,                    -- Updated every 30s while app is running
    
    FOREIGN KEY (session_id) REFERENCES time_sessions(id) ON DELETE SET NULL,
    FOREIGN KEY (work_order_id) REFERENCES work_orders(id) ON DELETE SET NULL
);

-- Initialize the singleton row
INSERT INTO active_session (id, session_id, work_order_id, started_at, last_heartbeat)
VALUES (1, NULL, NULL, NULL, NULL);

-- ============================================
-- RECENT ITEMS (for quick-switch)
-- ============================================
CREATE TABLE recent_work_orders (
    work_order_id TEXT PRIMARY KEY NOT NULL,
    last_used_at TEXT NOT NULL DEFAULT (datetime('now')),
    use_count INTEGER NOT NULL DEFAULT 1,
    
    FOREIGN KEY (work_order_id) REFERENCES work_orders(id) ON DELETE CASCADE
);

CREATE INDEX idx_recent_last_used ON recent_work_orders(last_used_at DESC);
```

### Entity Relationship Diagram

```
┌─────────────┐       ┌──────────────┐       ┌───────────────┐
│  customers  │──1:N──│ work_orders  │──1:N──│ time_sessions │
│             │       │              │       │               │
│ id (PK)     │       │ id (PK)      │       │ id (PK)       │
│ name        │       │ customer_id  │       │ work_order_id │
│ code        │       │ name         │       │ start_time    │
│ color       │       │ code         │       │ end_time      │
│ archived_at │       │ status       │       │ duration_*    │
│             │       │ archived_at  │       │ activity_type │
└─────────────┘       └──────────────┘       │ notes         │
                                             └───────────────┘
                                                    │
                             ┌──────────────────────┘
                             │
                      ┌──────▼────────┐
                      │active_session │  (singleton)
                      │               │
                      │ id = 1        │
                      │ session_id    │
                      │ work_order_id │
                      │ started_at    │
                      │ last_heartbeat│
                      └───────────────┘
```

---

## 5. Key Technical Decisions

### 5.1 Crash Recovery

**Strategy**: WAL mode + heartbeat + startup recovery

1. **WAL Mode**: SQLite WAL ensures writes survive app crashes
2. **Heartbeat**: While a session is active, update `active_session.last_heartbeat` every 30 seconds
3. **Startup Check**: On app launch, query `active_session`:
   - If `session_id` is set but `time_sessions.end_time` is NULL → orphan detected
   - Present recovery dialog: "You have an open session from [time]. Close it or discard?"

```rust
// On startup
async fn check_for_orphan_sessions(db: &DbPool) -> Option<OrphanSession> {
    sqlx::query_as!(
        OrphanSession,
        r#"
        SELECT 
            ts.id, ts.start_time, wo.name as work_order_name, c.name as customer_name
        FROM time_sessions ts
        JOIN work_orders wo ON ts.work_order_id = wo.id
        JOIN customers c ON wo.customer_id = c.id
        WHERE ts.end_time IS NULL
        "#
    )
    .fetch_optional(db)
    .await
    .ok()
    .flatten()
}
```

### 5.2 Session Switching

**Strategy**: Atomic transaction (stop old + start new)

```rust
async fn switch_session(
    db: &DbPool,
    new_work_order_id: &str,
) -> Result<Session, AppError> {
    let now = Utc::now().to_rfc3339();
    
    db.transaction(|tx| async move {
        // 1. Close any active session
        sqlx::query!(
            r#"
            UPDATE time_sessions 
            SET end_time = ?, 
                duration_seconds = (strftime('%s', ?) - strftime('%s', start_time)),
                updated_at = ?
            WHERE end_time IS NULL
            "#,
            now, now, now
        )
        .execute(&mut *tx)
        .await?;

        // 2. Create new session
        let session_id = uuid::Uuid::new_v4().to_string();
        sqlx::query!(
            r#"
            INSERT INTO time_sessions (id, work_order_id, start_time)
            VALUES (?, ?, ?)
            "#,
            session_id, new_work_order_id, now
        )
        .execute(&mut *tx)
        .await?;

        // 3. Update active_session singleton
        sqlx::query!(
            r#"
            UPDATE active_session 
            SET session_id = ?, work_order_id = ?, started_at = ?, last_heartbeat = ?
            WHERE id = 1
            "#,
            session_id, new_work_order_id, now, now
        )
        .execute(&mut *tx)
        .await?;

        // 4. Update recents
        sqlx::query!(
            r#"
            INSERT INTO recent_work_orders (work_order_id, last_used_at, use_count)
            VALUES (?, ?, 1)
            ON CONFLICT(work_order_id) DO UPDATE SET 
                last_used_at = excluded.last_used_at,
                use_count = use_count + 1
            "#,
            new_work_order_id, now
        )
        .execute(&mut *tx)
        .await?;

        Ok(Session { id: session_id, work_order_id: new_work_order_id.to_string(), start_time: now })
    }).await
}
```

### 5.3 Duration Calculation

**Strategy**: Auto-calculate with optional override

```sql
-- Effective duration query
SELECT 
    COALESCE(duration_override, duration_seconds) as effective_duration
FROM time_sessions
WHERE id = ?;
```

- `duration_seconds` is auto-calculated on session stop
- `duration_override` is set when user manually edits
- UI shows calculated duration but allows override via inline edit

### 5.4 Quick-Add (createAndStart)

**Strategy**: Single atomic IPC call creates customer (optional) + work order + starts session

```typescript
// Frontend
export interface QuickAddParams {
  customerName?: string;      // Create new if provided
  customerId?: string;        // Use existing if provided
  workOrderName: string;
  workOrderCode?: string;
}

export async function quickAdd(params: QuickAddParams): Promise<Session> {
  return invoke('quick_add', params);
}
```

```rust
// Backend: Creates customer if needed, creates work order, starts session
#[tauri::command]
async fn quick_add(
    customer_name: Option<String>,
    customer_id: Option<String>,
    work_order_name: String,
    work_order_code: Option<String>,
    state: State<'_, DbPool>,
) -> Result<QuickAddResult, AppError> {
    // Transaction: create all in one atomic operation
    // Returns: { customer, workOrder, session }
}
```

### 5.5 State Management

**Strategy**: Svelte 5 runes in simple store modules

```
┌─────────────────────────────────────────┐
│              App State                  │
├─────────────────────────────────────────┤
│ timer.svelte.ts                         │
│   - activeSession: Session | null       │
│   - elapsedSeconds: number              │
│   - isTracking: boolean (derived)       │
├─────────────────────────────────────────┤
│ sessions.svelte.ts                      │
│   - todaysSessions: Session[]           │
│   - recentWorkOrders: WorkOrder[]       │
├─────────────────────────────────────────┤
│ ui.svelte.ts                            │
│   - showQuickAdd: boolean               │
│   - searchQuery: string                 │
│   - recoverySession: OrphanSession|null │
└─────────────────────────────────────────┘
```

No Redux/Zustand needed — Svelte 5 runes provide reactive state with minimal boilerplate.

### 5.6 Build & Distribution

**Build command**:
```bash
# Development
npm run tauri dev

# Production build
npm run tauri build
```

**Output**:
- Windows: `target/release/bundle/msi/work-tracker-2_1.0.0_x64.msi` (~10-15MB)
- macOS: `target/release/bundle/dmg/work-tracker-2_1.0.0_aarch64.dmg`
- Linux: `target/release/bundle/appimage/work-tracker-2_1.0.0_amd64.AppImage`

**Auto-updates** (Phase 2+): Tauri supports auto-update via `tauri-plugin-updater`.

### 5.7 SvelteKit SSR Disabled (Critical for Tauri)

**Why SSR must be disabled**: Tauri apps are fully client-side. SvelteKit's default behavior attempts to pre-render pages in Node.js at build time, but Tauri IPC commands (e.g., `invoke()`) don't exist in Node.js — they only exist in the Tauri app runtime. This causes build failures and runtime errors in components that call Tauri at initialization time.

**Solution**: Disable SSR and prerendering globally in `src/routes/+layout.ts`:

```typescript
// src/routes/+layout.ts
export const ssr = false;
export const prerender = false;
```

**Impact**:
- SvelteKit routes render only in the browser after the Tauri app starts
- IPC commands are safe to use in component initialization (`onMount`, top-level effects)
- No build-time pre-rendering
- Reduces bundle size slightly (no SSR overhead)

**Related**: This pattern applies to any full-page components that use Tauri IPC (e.g., `ReportView.svelte`). Use `onMount()` to defer data fetching to runtime.

### 5.8 Pause/Resume Pattern (Phase 2 Preparation)

**Phase 1 scope**: Only Running/Stopped states. Pause is Phase 2+.

**When pause becomes relevant**: In Phase 2, sessions will support a paused state where the timer is frozen but the session remains open. This requires special handling in both backend and frontend.

**Backend pattern** (Phase 2):
```rust
// pauseSession command
pub fn pause_session(state: State<AppState>) -> Result<(), AppError> {
    let conn = state.db.lock().unwrap();
    let now = Utc::now().to_rfc3339();
    
    // Record pause time without closing session
    conn.execute(
        "UPDATE time_sessions 
         SET paused_at = ?, total_paused_seconds = total_paused_seconds + 0
         WHERE id = (SELECT session_id FROM active_session)",
        [&now]
    )?;
    Ok(())
}

// resumeSession command
pub fn resume_session(state: State<AppState>) -> Result<(), AppError> {
    let conn = state.db.lock().unwrap();
    
    // Clear pause state
    conn.execute(
        "UPDATE time_sessions 
         SET paused_at = NULL
         WHERE id = (SELECT session_id FROM active_session)",
        []
    )?;
    Ok(())
}
```

**Key point**: `pauseSession` and `resumeSession` return `void` (not `Session`), because they only update the pause state. The frontend must call `timer.refresh()` to query the updated active session state.

**Frontend pattern**:
```typescript
// src/lib/stores/timer.svelte.ts
async pause() {
  try {
    await apiPauseSession();   // Returns void
    await timer.refresh();     // Fetch updated session (isPaused, etc.)
  } catch (e) {
    alert(e?.message ?? 'Failed to pause');
  }
}

async resume() {
  try {
    await apiResumeSession();  // Returns void
    await timer.refresh();     // Fetch updated session
  } catch (e) {
    alert(e?.message ?? 'Failed to resume');
  }
}
```

**Why this matters**: The void return type is intentional—commands should be focused and return only what's necessary. If pause/resume need to update UI, the frontend queries the fresh state via a separate `getActiveSession()` call. This pattern is simpler than trying to return complex data structures.

---

## 6. Phase 1 Deliverables Checklist

### Backend (Rust)

- [ ] **Database initialization**
  - [ ] SQLite file created in app data directory
  - [ ] WAL mode enabled
  - [ ] Schema migrations run on startup
  - [ ] `active_session` singleton initialized

- [ ] **Crash recovery**
  - [ ] Startup check for orphan sessions
  - [ ] Recovery API: `recover_session()`, `discard_session()`

- [ ] **Customer commands**
  - [ ] `create_customer(name, code?, color?)`
  - [ ] `list_customers(include_archived?)`
  - [ ] `update_customer(id, ...)`
  - [ ] `archive_customer(id)`

- [ ] **Work order commands**
  - [ ] `create_work_order(customer_id, name, code?, description?)`
  - [ ] `list_work_orders(customer_id?, include_archived?)`
  - [ ] `update_work_order(id, ...)`
  - [ ] `archive_work_order(id)`

- [ ] **Session commands**
  - [ ] `start_session(work_order_id)` — atomic switch
  - [ ] `stop_session(notes?, activity_type?)`
  - [ ] `get_active_session()`
  - [ ] `update_session(id, duration_override?, notes?, activity_type?)`
  - [ ] `list_sessions(start_date, end_date)`
  - [ ] `delete_session(id)`

- [ ] **Quick-add command**
  - [ ] `quick_add(customer_name?, customer_id?, work_order_name, code?)`

- [ ] **Summary commands**
  - [ ] `get_daily_summary(date)` — totals by customer/work order
  - [ ] `get_recent_work_orders(limit)` — for quick-switch

- [ ] **Export command**
  - [ ] `export_csv(start_date, end_date)` — returns CSV string

### Frontend (Svelte)

- [ ] **Timer component**
  - [ ] Active session display (work order name, customer)
  - [ ] Real-time elapsed time (updates every second)
  - [ ] Stop button with optional notes input
  - [ ] Clear "not tracking" state when idle

- [ ] **Quick-switch interface**
  - [ ] Recent items list (last 10)
  - [ ] Search input with filtering
  - [ ] Click or Enter to switch
  - [ ] Keyboard shortcut (Cmd/Ctrl+K) to focus search

- [ ] **Quick-add overlay**
  - [ ] Keyboard shortcut (Cmd/Ctrl+N) to open
  - [ ] Customer dropdown or "create new"
  - [ ] Work order name (required)
  - [ ] Submit → creates and starts tracking

- [ ] **Today's work log**
  - [ ] List of completed sessions for today
  - [ ] Inline edit (notes, duration override)
  - [ ] Delete with confirmation

- [ ] **Daily summary panel**
  - [ ] Total hours today
  - [ ] Breakdown by customer
  - [ ] Real-time updates

- [ ] **Recovery dialog**
  - [ ] Shows on startup if orphan session detected
  - [ ] "Close now" (uses current time) or "Discard"
  - [ ] Blocks app until resolved

- [ ] **Settings (minimal)**
  - [ ] Customer management (CRUD)
  - [ ] Work order management (CRUD)

- [ ] **Export**
  - [ ] Date range selector
  - [ ] Export to CSV button
  - [ ] Save file dialog

### Infrastructure

- [ ] **Package.json scripts**
  - [ ] `dev` — Start Tauri dev server
  - [ ] `build` — Production build
  - [ ] `test` — Run frontend tests

- [ ] **Documentation**
  - [ ] README with setup instructions
  - [ ] This architecture doc
  - [ ] API reference (command signatures)

---

## 7. Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| App cold start | <2s | Time from click to usable UI |
| Context switch | <3s | Time from trigger to timer running |
| Timer update | <100ms | UI refresh latency |
| Search filter | <50ms | Keystroke to filtered results |
| Session create | <100ms | IPC round trip |
| Daily summary | <100ms | Query + render |
| CSV export (1 month) | <1s | Query + generate + save dialog |

---

## 8. Future Considerations (Phase 2+)

### System Tray
- Always-visible tray icon with current tracking status
- Right-click menu: Recent items, Start/Stop, Quick-add

### Global Hotkey
- System-wide shortcut to open quick-switch (even when app not focused)
- Requires `tauri-plugin-global-shortcut`

### Favorites/Pinning
- Pin work orders for one-click access
- Separate from "recent" — user-curated list

### Color Coding
- Customer colors propagate to UI elements
- Visual grouping in lists and summaries

### Pause State
- Phase 2: Add true pause (timer frozen but session not closed)
- Requires `paused_at` timestamp and pause interval tracking

---

## Appendix A: Rust Crate Dependencies

```toml
# src-tauri/Cargo.toml

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-sql = { version = "2", features = ["sqlite"] }
tauri-plugin-dialog = "2"          # File save dialogs
tauri-plugin-fs = "2"              # File system access
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"                    # Error handling
```

---

## Appendix B: Frontend Dependencies

```json
// package.json (partial)
{
  "devDependencies": {
    "@sveltejs/kit": "^2.0.0",
    "@sveltejs/vite-plugin-svelte": "^3.0.0",
    "@tauri-apps/cli": "^2.0.0",
    "svelte": "^5.0.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0"
  }
}
```

---

## Appendix C: Getting Started Commands

```bash
# Prerequisites
# - Node.js 18+
# - Rust (via rustup)
# - Platform build tools (see Tauri docs)

# Clone and setup
git clone <repo>
cd work-tracker-2
npm install

# Development
npm run tauri dev

# Build for distribution
npm run tauri build
```

---

*Document End*
