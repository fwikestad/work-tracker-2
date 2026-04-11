# Work Tracker 2 - Framework Instructions

## Project Overview

**Work Tracker 2** is a desktop application for consultants to intuitively and efficiently track time and work across multiple customers and projects throughout the day.

### Core Problem Statement

Consultants often juggle multiple projects from different customers within a single working day. They need a system that:
- Minimizes friction when switching contexts between customers/projects
- Accurately records and persists tracked time
- Allows quick logging of work activities, notes, and metadata
- Supports generation of summaries and export output
- Maintains data integrity across app sessions and unexpected shutdowns

### Guiding Principles

1. **Simplicity > Features** — Optimize every interaction for minimal cognitive load and keystrokes
2. **Speed First** — Context switching and data entry should feel instant
3. **Strong Data Integrity** — No lost time entries; support for corrections without destructive rewrites
4. **Desktop-Native** — Responsive UI, offline-capable, native platform feel
5. **Self-Contained Local App** — Must run fully on a single local computer with no required cloud services
6. **Extensibility** — Architecture should still allow future evolution without requiring a core rewrite

---

## Architecture Framework

### Conceptual Layers

The system should separate concerns across layers for independence and testability:

```
┌─────────────────────────────────────┐
│     Desktop User Interface          │  Fast, responsive, native
│     (Renders state, handles input)  │  Real-time updates
└────────────────┬────────────────────┘
                 │ Async messages / events
┌────────────────▼────────────────────┐
│  Application Service Layer          │  Business logic, decisions
│  (Coordinates data, validates rules)│  
└────────────────┬────────────────────┘
                 │ Queries / Mutations
┌────────────────▼────────────────────┐
│  Data Persistence Layer             │  Storage & retrieval
│  (Queries, transactions, constraints)│
└─────────────────────────────────────┘
```

**Why This Structure?**
- UI remains responsive; heavy operations don't block rendering
- Service layer can be tested independently
- Data layer can be swapped locally (embedded DB ↔ local service DB)
- Multiple UIs can share the same backend
- Clear separation enables future distributed architectures

### Local-Only Runtime Requirement

The application must be fully functional without internet access or cloud dependencies.

- All core features run locally: tracking, editing, summaries, and exports
- Data is persisted on the local machine only
- Startup and normal operation must not require remote APIs or SaaS services
- Optional integrations may exist, but must be disabled by default and never block core workflows

### Technology Agnostic

The framework does **not** mandate:
- Specific language or framework choices
- Database implementation (SQL, NoSQL, hybrid)
- API protocol (REST, GraphQL, RPC, local IPC)
- UI framework or styling approach
- Deployment or packaging method

**Rationale**: These choices should be made by the implementing agent based on constraints and preferences, not locked into the framework.

---

## Domain Model & Entities

### Core Concepts (Not Prescriptive Schema)

The system models work as hierarchical relationships. Agents may implement these as database tables, documents, objects, or other structures as appropriate.

**Customers** represent tracked entities (companies, clients, departments). Each customer may have multiple projects/work units.

**Work Orders** (or Projects/Tasks) represent discrete units of work under a customer. Each has contextual metadata: code, description, status, and notes.

**Time Sessions** represent continuous periods of work on a single work order. A session has:
- A start point (timestamp)
- Metadata about the work (activity type, notes, tags)
- An end point (timestamp, or open if in-progress)
- A calculated or user-corrected duration

**The Primary Constraint**: At most one active session at a time. Switching to a new work order implicitly closes the previous session.

**Session States** (Phase 1: Running/Stopped only):
- **Running** = session has start_time, no end_time, timer actively accumulating
- **Stopped** = session has both start_time and end_time, duration finalized
- **Paused** (Phase 2 only) = session has start_time, timer frozen but session not closed; requires tracking pause intervals or a `paused_at` timestamp. MVP recommendation: skip true pause in Phase 1 — use "stop" instead (simpler, less error-prone)

### Data Integrity Requirements

Regardless of implementation, the system must:
1. **Persist all sessions** — No time entry should be lost due to app crash or unexpected shutdown
2. **Support correction** — Allow users to adjust duration, notes, and activity metadata after entry creation
3. **Maintain audit trail** — Track when entries were created/modified (for future compliance)
4. **Prevent overlaps** — No two sessions for the same user can have overlapping time ranges
5. **Cascade cleanly** — Deleting a customer or work order should handle associated sessions gracefully
6. **Query efficiently** — Daily summaries, weekly reports, and filtering must be fast (<100ms target)

### Crash Recovery & Durability

The system must survive crashes and unexpected shutdowns without data loss:

1. **WAL Mode Required**: SQLite must use Write-Ahead Logging (`PRAGMA journal_mode=WAL`) for crash-safe writes
2. **Immediate Write Policy**: Every session INSERT/UPDATE must be flushed to disk before returning success to the UI — no "save" button, no batching
3. **Synchronous Mode**: Use `PRAGMA synchronous=NORMAL` minimum (FULL for maximum safety)
4. **Recovery Flow on Startup**:
   - Query for incomplete sessions (`end_time IS NULL`)
   - If found, present recovery UI: "You have an open session from [timestamp]. Close it now or discard?"
   - User can accept (close with current time) or discard (delete the orphan)
   - Recovery must complete before normal app usage begins
5. **No "Save" Button**: All writes are immediate and durable — the user never manually saves

### Optional Metadata

Depending on feature scope, entities may track:
- Color coding for visual recognition
- Activity classification (meeting, development, design, admin)
- Custom fields and tags

---

## Feature Framework

### Core User Workflows

#### 1. Start Work Session
**Goal**: User switches context to begin work on a specific customer/project with minimal friction.

**Principles**:
- Operation should complete in <3 seconds
- Minimal input required: identify the work order (by recent, search, or shortcut)
- Quick switch should be available from app UI and taskbar/system tray shortcuts
- Actively working state should be visually obvious
- If a session already active, it should be stopped/saved automatically

**Success Criteria**: Consultant can switch between 3 different work orders in <10 seconds, no data loss.

---

#### 2. Stop / Pause Active Work
**Goal**: User marks the current session as complete and optionally captures notes/metadata.

**Principles**:
- Stopping a session calculates duration (either auto from timestamps or user-provided)
- User can add/edit notes and classify activity type in one action
- No confirmation dialogs for reversible actions

**Success Criteria**: Stop action completes in <1 second, entry is immediately persisted.

---

#### 3. Manage Work Orders & Customers
**Goal**: User can create, view, and update the catalog of available customers and projects.

**Principles**:
- CRUD operations (create, read, update, delete)
- Search/filter by name or code
- Bulk visibility of active vs archived projects
- Cascading deletes should warn before execution
- **Inline Quick-Add** (Phase 1): Create a new customer + work order from the active timer view without navigating away
  - Quick-add requires only a name — all other fields optional and can be filled later
  - Keyboard shortcut (Cmd/Ctrl+N) opens quick-add overlay from anywhere in the app
  - After quick-add completes, immediately start tracking against the new work order

**Success Criteria**: Create new customer + work order in <30 seconds.

---

#### 4. View Daily Summary
**Goal**: User sees aggregated time breakdown for the current day at a glance.

**Principles**:
- Summary should update in real-time as sessions are created/edited
- Show by customer (most common), by project (alternative view)
- Show totals and distribution by activity/project

**Success Criteria**: Summary loads on app start in <1 second, accurate to within 1 minute.

---

#### 5. Generate Reports & Export
**Goal**: User exports time entries for personal review, archive, or sharing.

**Principles**:
- Support date range filtering (today, this week, this month, custom)
- Group by customer and/or project
- Include duration totals and session notes
- Export format(s) suitable for archive/sharing (CSV minimum)

**Success Criteria**: Generate 1-month report in <5 seconds, matches manual verification.

---

#### 6. Correct / Edit Past Sessions
**Goal**: User adjusts duration, notes, or activity metadata for historical entries.

**Principles**:
- Support inline edit (click entry, modify field)
- Support undo/revert for reversible actions
- Optional: lock old entries to prevent accidental changes
- Show timestamp of when entry was created and last modified

**Success Criteria**: Edit entry and save in <3 seconds, changes reflected immediately.

---

### Optional (Phase 2+) Workflows

- **Favorites/Pinning**: Pin frequently-used work orders for one-click access (Phase 2)
- **Notifications**: Alert if consultant forgets to stop timer
- **Local Backups**: Scheduled local backup/export workflows
- **Team Features**: Shared workflows only if implemented as optional and non-blocking
- **Integrations**: Optional import/export with external billing/accounting tools

---

## Design Principles

### Speed & Responsiveness
- **Context switching**: Change projects in <3 seconds without losing data
- **Data persistence**: Every entry auto-saved; no "save" button needed
- **Keyboard shortcuts**: All common actions accessible via keyboard (no mouse required)
- **Minimize dialogs**: Use inline editing and inline forms where possible
- **No confirmations for reversible actions**: Support undo instead

### Visual Clarity
- **Always visible**: "What am I working on right now?" is immediately obvious
- **Active state prominent**: Currently running session should dominate the screen
- **Hierarchy**: Active work → today's entries → past/archived work
- **Color coding**: Visual grouping by customer or project recommended (Phase 2)
- **State distinction**: Running and stopped entries should be clearly marked (Phase 1); add paused state in Phase 2

### Data Accuracy
- **Time persists**: No data loss on app crash or close (write to disk/DB on every change)
- **User control**: Support both auto-calculated duration and manual override
- **Audit visibility**: When was entry created? When was it last edited?
- **Prevention over correction**: Warn before risky operations (delete, bulk change)

### Responsive Layout
- **Mobile-friendly**: Works on small screens and with touch/stylus input
- **Large touch targets**: Accommodate gloved hands and stylus use
- **Single-column preferred**: Minimize scrolling; important info visible on first load
- **Adapt to screen size**: Flexible layout for desktop, tablet, (future) mobile

---

## Database Design Framework

The data layer should support:

### Core Requirements
1. **Entity relationships**: Organize time sessions to customers/projects hierarchically
2. **Transactionality**: Atomic operations (e.g., stop session + start new = single transaction)
3. **Concurrency**: Support user corrections without lost writes
4. **Queryability**: Efficient filtering by date, customer, project, and activity type
5. **Cascading**: Safe deletion of customers/projects (clean up child sessions)

### Performance Targets
- Single-entry create: <100ms
- Daily summary query: <100ms
- Weekly report query: <500ms
- Search/autocomplete: <50ms

### Data Consistency Rules
- **No overlaps**: At most one active (end_time = null) session per session
- **Duration consistency**: Either calculated from timestamps OR user-provided, with clear source of truth
- **Incomplete session logic**: Open sessions (no end_time) should have clear behavior rules
- **Cascade behavior**: Deleting a project cascades to delete all its sessions

---

## Service Layer Framework

### Service Layer Responsibilities
- **Business logic**: Rules for starting/stopping sessions, switching projects
- **Validation**: Enforce data constraints (no overlaps, duration validity, relationships)
- **Calculations**: Tracked hours and summaries
- **Querying**: Provide domain-specific queries (daily totals, weekly reports)
- **Persistence**: Coordinate with data layer, transactions

### API/Communication Framework
Whatever protocol chosen (REST, GraphQL, RPC, local IPC):
- **Structured responses**: Every response should have consistent error/success structure
- **Idempotency**: Retry-safe operations where possible
- **Clear contracts**: Document what each operation requires and returns
- **Error detail**: Include enough context for debugging (validation failures, conflicts, etc.)

---

## Implementation Phases

### Phase 1: Core Time Tracking (MVP)
**Goal**: Single consultant, one day of tracking accuracy

**Scope**:
1. Data layer (persistence of sessions, customers, work orders) with WAL mode and crash recovery
2. Service layer (start, stop, query operations) including `createAndStart` for quick-add
3. UI (active timer, today's entries list, basic entry management)
4. Integration (UI ↔ service ↔ data layer communication)
5. **Quick-switch (basic)**: Recent items list (last 5-10 work orders), search-to-switch (type to filter, Enter to switch)
6. **Quick-add**: Inline overlay to create customer + work order and immediately start tracking (Cmd/Ctrl+N)

**Rationale**: Context switching IS the core value prop — MVP must demonstrate it.

**Success Criteria**: 
- Consultant can track a full day without data loss
- Context switch in <3 seconds
- Can export tracked time for personal review
- Session states: Running/Stopped only (no pause in Phase 1)

### Phase 2: Multi-Customer Workflows
**Goal**: Smooth switching across customers and projects

**Scope**:
1. Customer/project management (full CRUD, bulk operations)
2. **Advanced quick-switch**: Favorites/pinning, global hotkey, taskbar menu with recents
3. Visual organization (color-coding, grouping)
4. Advanced keyboard shortcuts for power users
5. **Paused state**: Timer frozen but session not closed (requires pause interval tracking)

**Success Criteria**:
- Switch between 3 projects smoothly
- No data loss during switch
- Can quickly find frequently-used projects
- Session states: Running/Paused/Stopped

### Phase 3: Reporting & Analysis
**Goal**: Generate personal activity summaries and exports

**Scope**:
1. Daily/weekly summary queries
2. Activity/project filtering
3. Duration trend calculations
4. Export formats (CSV at minimum)

**Success Criteria**:
- Generate monthly summary report in <5 seconds
- Report matches manual verification

### Phase 4: Advanced (Post-MVP)
**Goal**: Team features, multi-device, integration

**Scope**:
- Local backup/restore workflows
- Multi-user / team features
- Companion experiences that do not compromise local-first behavior
- Third-party integrations (billing, accounting systems)
- Notifications and alerts

---

## Architecture Guidance: Implementation Patterns

### Recommended Separation
The three-layer model ensures flexibility:

**Presentation Layer**:
- Handles UI state and rendering
- Sends user intents as commands/events to service layer
- Receives updates and re-renders

**Service Layer** (Business Logic):
- Enforces rules (no overlapping sessions, cascade logic)
- Coordinates multiple data operations (transactional)
- Provides domain methods (startSession, stopSession, dailySummary)

**Data Layer** (Persistence):
- Stores and retrieves entities
- Enforces constraints and relationships
- Provides efficient queries

### Key Decisions Agents Will Make
- **Data storage**: Embedded SQL DB? Embedded document DB? File-based store?
- **UI framework**: Electron + React? Tauri + Svelte? Other desktop-capable UI?
- **API style**: Local REST? Local GraphQL? Local IPC? CLI?
- **Language**: Python? Rust? TypeScript? Go?

These are not prescribed; agents should choose based on constraints, team expertise, and project requirements.

---

## Development Guidance

### Structure: Suggested Layout
Agents should organize code to support the three-layer separation:

```
project-root/
├── .github/
│   └── instructions/          # This framework and guidance
├── backend/ (or src/)         # Service + Data layers
│   ├── domain/                # Business logic, rules
│   ├── persistence/           # Data access, queries
│   ├── api/                   # External communication
│   └── tests/
├── frontend/                  # Presentation layer  
│   ├── components/
│   ├── state/
│   ├── api-client/
│   └── tests/
└── docs/                      # Implementation decisions, design docs
```

### Core Development Principles
- **Separation of concerns**: Each layer has clear responsibility
- **Testability**: Service layer can be tested independently
- **Error handling**: Structured, actionable errors throughout
- **Transactions**: Multi-step operations atomic
- **Performance**: Target the metrics in this framework

### Documentation Requirements
- **How to start**: Setup instructions and first-run guide
- **API contract**: What each service provides
- **Data model**: Entity definitions and relationships
- **Operations**: Deployment, backup, recovery procedures

---

## Common Questions

**Q: How do I start implementing the timer?**
A: Begin in Phase 1. Create a way to persist a session on a work order and display a running timer. Start with a mocked backend if needed.

**Q: Should I allow overlapping time entries?**
A: No. Only one active session at a time. Stopping one before starting another is enforced by rules and UX.

**Q: How do I future-proof while staying local-first?**
A: Keep the service and data layers modular, but require local persistence and local runtime by default. Optional external integrations can be added behind feature flags without affecting core workflows.

**Q: Can team members work on the same project?**
A: Phase 4 feature. Start with single-user, local-only assumption.

---

## Success Metrics

- **Speed**: Start → Timer running in <5 seconds, switch customer in <3 seconds
- **Accuracy**: 100% of entered time persists; no data loss on crashes
- **Usability**: Consultant prefers app over spreadsheet/notebook for daily tracking
- **Reporting**: Can generate personal monthly summary in <1 minute
