# Work Routing

How to decide who handles what.

## Routing Table

| Work Type | Route To | Examples |
|-----------|----------|----------|
| UI, components, timer display, UX | Leia | Active indicator, context switcher, keyboard shortcuts, taskbar tray |
| Data layer, services, API | Chewie | Schema, migrations, start/stop session, daily summary, export |
| Architecture, tech decisions, scope | Han | Stack selection, phase planning, code review, issue triage |
| Tests, quality, edge cases | Wedge | Session overlap tests, midnight boundary, crash recovery, perf assertions |
| Code review | Han | Review PRs, architectural judgment, approve/reject with feedback |
| Testing | Wedge | Write tests, find edge cases, reviewer gating before ship |
| Scope & priorities | Han | What to build next, trade-offs, decisions |
| Documentation, setup guides, API docs, changelogs, inline comments | Mon Mothma | Architecture docs, command contracts, user guides, release notes |
| Session logging | Scribe | Automatic — never needs routing |
| Final approvals, scope decisions, design sign-off | Fredrik | Human in the loop — coordinator pauses and presents for review |

## Issue Routing

| Label | Action | Who |
|-------|--------|-----|
| `squad` | Triage: analyze issue, assign `squad:{member}` label | Lead |
| `squad:{name}` | Pick up issue and complete the work | Named member |

### How Issue Assignment Works

1. When a GitHub issue gets the `squad` label, the **Lead** triages it — analyzing content, assigning the right `squad:{member}` label, and commenting with triage notes.
2. When a `squad:{member}` label is applied, that member picks up the issue in their next session.
3. Members can reassign by removing their label and adding another member's label.
4. The `squad` label is the "inbox" — untriaged issues waiting for Lead review.

## Rules

1. **Eager by default** — spawn all agents who could usefully start work, including anticipatory downstream work.
2. **Scribe always runs** after substantial work, always as `mode: "background"`. Never blocks.
3. **Quick facts → coordinator answers directly.** Don't spawn an agent for "what port does the server run on?"
4. **When two agents could handle it**, pick the one whose domain is the primary concern.
5. **"Team, ..." → fan-out.** Spawn all relevant agents in parallel as `mode: "background"`.
6. **Anticipate downstream work.** If a feature is being built, spawn the tester to write test cases from requirements simultaneously.
7. **Issue-labeled work** — when a `squad:{member}` label is applied to an issue, route to that member. The Lead handles all `squad` (base label) triage.
