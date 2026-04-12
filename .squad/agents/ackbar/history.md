# Ackbar — History

## Project Context

**Project:** work-tracker-2 — Native desktop time tracker for consultants
**User:** Fredrik Kristiansen Wikestad
**Stack:** Tauri 2 + Rust + SQLite (rusqlite) + Svelte 5 + TypeScript
**Joined:** 2026-04-12

The app is a local-only desktop app. No cloud services, no network exposure. Data lives in a SQLite DB on the user's machine. Tauri 2 provides the native wrapper — Rust backend handles all DB operations via IPC commands, Svelte 5 frontend renders the UI.

Key architecture facts:
- IPC commands in `src-tauri/src/commands/` — these are the attack surface boundary
- DB access via `get_conn()` helper (introduced 2026-04-12 refactor) — safe Mutex lock acquisition
- `tauri-plugin-fs` and `tauri-plugin-dialog` are in use — file system access present
- WAL mode enabled, foreign keys enforced
- No authentication layer (single-user local app)
- 16 Rust integration tests passing, Vitest frontend tests present

## Session: Team Expansion (2026-04-12)

Charter and history files created and committed to repo.

Commit: b6f5341 — team: add Ackbar (Security) and Lando (DevOps)

Ready to begin security reviews and threat modeling.

## Learnings

### 2026-04-12: Security Review #001 — Complete & Approved

First comprehensive security review completed. Overall risk: **LOW** (0 critical, 0 high, 2 medium, 3 low findings).

**Key Findings**:
- **[Medium] CSP disabled** (tauri.conf.json:31) — Fix: Set restrictive CSP
- **[Medium] Shell plugin unused** (Cargo.toml) — Fix: Remove from dependencies
- **[Low] `withGlobalTauri: true`** — Fix: Set to false
- **[Low] No input length validation** — Fix: Add max length checks (255–2000 chars)
- **[Low] CSV formula injection** — Fix: Prefix formula-starting cells with quote

**Positive observations**:
- ✅ All SQL parameterized (no injection possible)
- ✅ Mutex-guarded DB access
- ✅ Transactions for atomic operations
- ✅ WAL mode + foreign keys
- ✅ UUID v4 for all IDs
- ✅ Structured error serialization

**Dependency audits**:
- cargo audit: 0 vulnerabilities, 20 warnings (GTK3 transitive, expected)
- npm audit: 3 low (cookie in @sveltejs/kit, not exploitable in desktop)

**Action plan**:
- Priority 1: CSP + shell plugin removal (immediate, high impact)
- Priority 2: Input validation, CSV guards (Phase 2)

Document: docs/security-review-001.md. Decisions merged into squad/decisions.md.

---

### 2026-04-12: Security Review #001

- **SQL layer is solid.** All queries use `rusqlite::params![]` parameterization. Dynamic UPDATE builders use hardcoded column names — safe pattern. `fetch_sessions` in summary_service uses a private `where_clause` param called only with literals.
- **CSP is disabled** (`"csp": null` in tauri.conf.json). This is the #1 thing to fix. Combined with `withGlobalTauri: true`, any XSS gives full IPC access.
- **Shell plugin is unused dead weight.** `tauri-plugin-shell` is declared in Cargo.toml, registered in lib.rs, and granted `shell:default` capability, but no code uses it. Should be removed.
- **No input length validation anywhere.** All command handlers accept unbounded strings. Need a validation layer or per-field checks.
- **CSV export has a formula injection gap.** The `escape_csv` function handles delimiter escaping but not formula-prefix characters (`=`, `+`, `-`, `@`).
- **cargo audit found 0 vulnerabilities.** 20 warnings are all transitive deps (GTK3 unmaintained, glib unsound, rand unsound). Not actionable at app level.
- **npm audit found 3 low** (`cookie` in @sveltejs/kit). Not exploitable in a desktop app with no HTTP cookies. Do not apply the breaking fix.
- **Tauri capabilities are reasonable.** `fs:default` is scoped to app directories. `dialog:default` is safe. Only `shell:default` is unnecessary.
- **Trust boundary is simple.** WebView → IPC → Rust → SQLite. No network listeners. Attack requires local access or WebView compromise.
