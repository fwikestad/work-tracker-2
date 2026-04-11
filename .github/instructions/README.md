# Work Tracker 2 - Instruction Set Overview

This directory contains comprehensive Copilot instructions for building a consultant work-tracking desktop application.

## File Organization

### Main Workspace Instructions
- **[copilot-instructions.md](../copilot-instructions.md)** — Framework for the entire project
  - Project overview and core problem statement
  - Architectural layer model (not prescriptive)
  - Domain entities and constraints (not schema)
  - Feature workflows and success criteria
  - Design principles (not specific layouts)
  - Implementation phases with clear goals
  - High-level guidance for architecture decisions

### Specialized Domain Instructions
Located in this folder (`.github/instructions/`):

#### 1. **[ui-components.instructions.md](./ui-components.instructions.md)**
Applied to: Frontend files (`frontend/**/*.{ts,svelte,tsx,jsx}`)

**Use when**: Building UI components, designing layouts, implementing user interactions

**Key topics**:
- Component architecture patterns (Container, Presentational, Form)
- 7 key feature areas: Taskbar quick switch/status, active work, context switching, daily log, entry creation, summary, management
- Design principles: visual hierarchy, keyboard-first, responsive
- Performance targets and benchmarks
- Testing checklist

---

#### 2. **[database.instructions.md](./database.instructions.md)**
Applied to: Backend data files (`backend/**/*.{rs,py,sql,ts}`, `backend/db/**`)

**Use when**: Designing data persistence, writing queries, managing data access layers

**Key topics**:
- Core entities and relationships (not SQL schema)
- Fundamental constraints and their rationale
- Querying patterns (by date, customer, activity, etc.)
- Consistency rules (duration calculation, session uniqueness, cascading)
- Performance considerations and optimization strategies
- Data validation patterns
- Migration and schema evolution guidance
- Transaction design for atomic operations

---

#### 3. **[api-backend.instructions.md](./api-backend.instructions.md)**
Applied to: Backend service files (`backend/**/*.{rs,py,ts}`, `backend/src/**`)

**Use when**: Designing APIs, implementing business logic, building operations

**Key topics**:
- API design principles (stateless, clear contracts, consistent errors)
- Core workflows: customer management, work order management, session/entry management
- Business logic patterns: no overlapping sessions, duration handling, summary calculations
- Data access patterns and query optimization
- Service layer responsibilities
- Communication patterns (works with REST, GraphQL, RPC, IPC)
- Session/state management
- Testing checklist

---

## Philosophy: Framework Not Specification

These instructions provide **guidance** on **what** to build and **why**, not **how** to build it:

✅ **We prescribe**:
- Core problems: fast context switching, accurate time tracking, no data loss
- Constraints: at most one active session, duration integrity, cascading deletes
- Workflows: start work, stop work, switch context, export reports
- UX requirement: taskbar/system tray shortcut for quick switching and active tracking visibility
- Success criteria: <3 seconds to switch, <100ms queries, 100% data persistence
- Runtime model: self-contained local app with no required cloud dependency

❌ **We don't prescribe**:
- Specific database schema (SQL vs NoSQL, table names, column types)
- Specific endpoints (/api/v1/... vs /graphql vs RPC methods)
- Specific languages or frameworks
- Specific UI components or styling
- Specific deployment or packaging

---

## How Copilot Uses These Instructions

### Automatic Loading
When you work on files, Copilot automatically loads relevant instructions:
- Editing `frontend/components/*`? → `ui-components.instructions.md` loads
- Writing database code? → `database.instructions.md` loads
- Implementing backend logic? → `api-backend.instructions.md` loads

### Manual Discovery
You can explicitly reference instructions in queries:
- "Build the active work indicator following the UI guidelines"
- "Design the data model for storing sessions and ensuring no overlaps"
- "Implement the start work operation with proper transaction handling"

### Context Clues
Copilot uses keywords from instruction descriptions to decide relevance:
- "Use when: designing database schema..." — Triggers on schema questions
- "Use when: building UI components..." — Triggers on component questions
- "Use when: designing APIs..." — Triggers on API questions

---

## Document Structure

Each instruction file follows this pattern:

```yaml
---
name: Clear name
applyTo: "glob pattern for files"
description: "Use when: [trigger keywords] - What this helps with"
---

# Topic

## Subsection
[Guidance, patterns, constraints, checklist]
```

The `applyTo` glob determines which files trigger auto-loading. The `description` contains discovery keywords.

---

## Quick Start for Agents

### Phase 1: Core Time Tracking

Ask Copilot:
1. "Set up the data persistence layer. Follow the database instructions. Support customers, work orders, and time sessions."
2. "Implement the backend service layer. Follow the API guidelines. Need operations for start/stop/list work samples."
3. "Build the UI. Follow the component guidelines. Need active work indicator, daily log, and entry creation."

### Phase 2: Multi-Customer Workflows

Ask Copilot:
1. "Add customer/project management CRUD operations."
2. "Implement quick-switch interface for fast context switching."
3. "Add keyboard shortcuts following the UI guidelines."

### Phase 3: Reporting

Ask Copilot:
1. "Implement daily/weekly summary queries."
2. "Add activity and duration summary calculations."
3. "Build CSV export for personal review and archive."

---

## Key Concepts (Not Implementations)

### No Overlapping Sessions
**What**: Only one work order should be active at a time  
**Why**: Prevents confusion, ensures accurate personal tracking  
**How**: Agents choose (database constraint, application validation, or both)

### Duration Flexibility
**What**: Support both calculated (end - start) and manual duration entry  
**Why**: UX - users sometimes forget to stop timers  
**How**: Agents choose (store both, track source of truth, etc)

### Cascade Safety
**What**: Deleting a customer removes associated work orders and entries  
**Why**: No orphaned data, clean model  
**How**: Agents choose (hard delete vs soft delete, warning UI, etc.)

### Summary Accuracy
**What**: Accurate reporting of tracked time and activity summaries  
**Why**: Core value for personal review and daily planning  
**How**: Agents choose (aggregation queries, caching, real-time calc)

---

## Files Structure Agents Will Create

Agents have freedom to organize as follows:

```
project-root/
├── .github/
│   └── instructions/              # This framework
├── backend/ (or src/)
│   ├── domain/                    # Business logic, entities
│   ├── persistence/               # Data access, queries
│   ├── services/                  # Operations, workflows
│   ├── api/                       # Communication layer
│   └── tests/
├── frontend/                      
│   ├── components/                # UI pieces
│   ├── state/                     # State management
│   ├── services/                  # API client, backend comm
│   └── tests/
└── docs/
    ├── API.md                     # API specification
    ├── DATA_MODEL.md              # Entity relationships
    └── DEPLOYMENT.md              # How to run
```

---

## Common Questions

**Q: Should I follow the prescriptive details?**
A: No - this is a framework, not a specification. Skip any prescriptive sections (like "create `Timer.svelte`") and focus on the goals and constraints.

**Q: What if I want to use a different architecture?**
A: Great! As long as you satisfy the core constraints (no overlapping sessions, fast context switching, data persistence), the architecture is flexible. Document your choices.

**Q: How do I know if my implementation is correct?**
A: Check against the success criteria in each workflow:
- Can consultant track a full day without data loss? ✓
- Can they switch context in <3 seconds? ✓
- Can they export summary data for personal review? ✓

**Q: Can I add more features beyond Phase 1-4?**
A: Absolutely. The framework is extensible. Document new features and their constraints.

---

## When to Reference Which Instruction

| I need to... | Reference file |
|---|---|
| Understand the overall vision | `../copilot-instructions.md` |
| Build UI for a feature | `ui-components.instructions.md` |
| Design data persistence | `database.instructions.md` |
| Implement a workflow/operation | `api-backend.instructions.md` |
| Check if my design is sound | All three + success criteria |
| Debug a constraint issue | Domain entity constraints section |

---

## Maintaining Instructions

### When to Update
- Add a new core workflow (update all 3 files)
- Change a fundamental constraint (update relevant files + main framework)
- Clarify ambiguous guidance (update relevant file)
- Discover a new pattern (add as subsection in relevant file)

### Version Control
- Keep instructions in `.github/` for team visibility
- Changes to instructions = pull request (review by team)
- Track rationale for changes (why was this constraint added?)

### Staying Aligned
- Instructions should match actual codebase
- If implementation diverges from instructions, update one or both
- Use instructions to onboard new agents to the project

---

## Support & Feedback

If an instruction is:
- **Too vague**: Add examples or anti-patterns
- **Too specific**: Generalize to framework/principles
- **Inaccurate**: Correct based on actual implementation
- **Missing**: Add if it's a core concern

Remember: These instructions exist to enable agents to build the right thing, not to constrain their creativity.

