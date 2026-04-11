# Squad Decisions

## Active Decisions

### 2026-04-11: Instruction Framework Review — Blockers Identified

**From**: Han (Lead)  
**Status**: REQUIRES LEAD DECISION

Three critical blockers preventing Phase 1 agent spawning:

1. **Crash Recovery Not Specified**
   - Requirement: Core problem #4 ("never lose time on crash")
   - Gap: All files say "persist immediately" but no implementation guidance
   - Missing: WAL mode specification, sync behavior, recovery flow
   - Impact: Agents will implement crash safety inconsistently or skip it

2. **Quick-Add Workflow Missing**
   - Requirement: Core problem #1 ("quickly create customers/work orders")
   - Gap: CRUD management defined, but inline quick-add UX undefined
   - Missing: UI component spec, combined backend endpoint, phase scope
   - Impact: Tracking workflow incomplete; users can't create work orders mid-session in <30s

3. **Quick-Switch Phase Boundary Blurred**
   - Requirement: Core problem #2 ("instantly switch context" in <3 seconds)
   - Issue: Quick-switch interface marked Phase 2, but required for Phase 1 MVP
   - Missing: Recent/search-to-switch in Phase 1 scope
   - Impact: Phase 1 MVP won't meet core need; users can't efficiently switch projects

**Secondary issues** (should-fix before Phase 2):
- "Paused" state undefined across all files (DB, UI, backend)
- Default indexes not specified (agents will guess, performance degrades)
- Performance targets inconsistent (3s vs 2s context switch)

**Recommended Action**:
- Fix blockers 1–3 before spawning Phase 1 builders
- Can defer secondary issues to phase gates
- Estimated fix time: 2–4 hours

## Governance

- All meaningful changes require team consensus
- Document architectural decisions here
- Keep history focused on work, decisions focused on direction
