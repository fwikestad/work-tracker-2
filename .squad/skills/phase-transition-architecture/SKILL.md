# Skill: Phase Transition Architecture

## When to Use
When planning a new project phase where the previous phase built infrastructure for upcoming features. Applies when writing architecture docs that bridge "already built" → "needs wiring."

## Pattern

1. **Audit existing code first** — Map every Phase N+1 feature to existing Phase N code. Categorize as: fully implemented / partially implemented / not started.
2. **Document data flows, not just features** — Draw the full path: user action → component → store → IPC → backend → DB → response → store update → re-render. Implementers need the chain, not just the endpoints.
3. **Identify race conditions at layer boundaries** — Anywhere UI optimistically updates before backend confirms, there's a race. Document the specific sequence and the mitigation (transitioning guard, idempotency, or optimistic rollback).
4. **Constrain each implementer explicitly** — Per-agent constraint tables prevent scope creep and duplicated work. "Do NOT add new backend commands" is more useful than "backend is complete."
5. **Split phases at dependency boundaries** — If Feature A doesn't depend on Feature B, they can ship independently. Make this explicit (Phase 2a / 2b).

## Anti-Patterns
- Writing architecture docs that repeat the plan document (plan = what/who/when, architecture = how/why/constraints)
- Documenting backend changes that don't exist (verify code first)
- Skipping race condition analysis for IPC-heavy flows

## Example
See `docs/phase2-architecture.md` in work-tracker-2 for a complete example applying this pattern.
