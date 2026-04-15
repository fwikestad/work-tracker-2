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

---

### 2026-04-13: Pre-Public Security & Privacy Audit — PASSED ✅

**Purpose**: Comprehensive audit before making repository public on GitHub.

**Scope**: Privacy-focused review for PII, local paths, credentials, secrets, and internal information that could compromise developer or machine.

**Result**: **SAFE TO GO PUBLIC** — No blocking issues found.

**What Was Checked**:
- All source files (`*.ts`, `*.rs`, `*.svelte`, `*.json`, `*.toml`, `*.yml`, `*.md`)
- Git-tracked files list
- .gitignore coverage
- .squad/ tracked content
- Dependency audits (npm audit, cargo audit)

**Findings Summary**:
- ✅ No credentials, API keys, tokens, or passwords
- ✅ No local paths (`C:\Users\`, `/home/`, `frewikes`)
- ✅ No machine names or hostnames
- ✅ No private IPs or internal URLs
- ✅ Developer name appears only in expected metadata (Cargo.toml authors, project docs) — acceptable
- ✅ .gitignore properly excludes .env, node_modules, build artifacts, sensitive .squad dirs
- ✅ npm audit: 3 low (cookie in sveltejs/kit, not exploitable)
- ✅ cargo audit: 0 vulnerabilities

**Previously Flagged (Non-Blocking)**:
- CSP disabled — code-level concern, not privacy risk
- withGlobalTauri: true — code-level concern, not privacy risk

**Document**: `.squad/decisions/inbox/ackbar-pre-public-security-review.md`

---

### 2026-04-13: Full Security Audit #002 — Complete

**Purpose**: Comprehensive security audit of the work-tracker-2 codebase.

**Result**: **LOW RISK** — 0 Critical, 0 High, 2 Medium, 2 Low findings.

**Automated Scans**:
- `cargo audit`: 0 vulnerabilities (20 GTK3 warnings, transitive, not actionable)
- `npm audit`: 3 low (cookie in @sveltejs/kit, not exploitable in desktop)

**Findings**:

| # | Severity | Title | GitHub Issue |
|---|----------|-------|--------------|
| 1 | Medium | CSP Disabled | #6 |
| 2 | Medium | withGlobalTauri: true | #9 |
| 3 | Low | CSV Formula Injection | — |
| 4 | Low | No Input Length Validation | — |

**Positive Observations**:
- ✅ All SQL parameterized (no injection)
- ✅ No unsafe DOM ops (innerHTML, eval)
- ✅ No hardcoded secrets
- ✅ Shell plugin removed (previous finding fixed)
- ✅ Mutex-guarded DB access
- ✅ UUID v4 for IDs
- ✅ Capability scope appropriate

**Recommendations**:
1. **P1**: Set restrictive CSP and `withGlobalTauri: false`
2. **P2**: Add input length validation (255/2000 char limits)
3. **P2**: Add CSV formula prefix sanitization

**Document**: `.squad/decisions.md` (merged from inbox)

### 2026-04-15: Security Audit Results Consolidated

All findings merged into .squad/decisions.md. GitHub Issues #6 and #9 tracking P1 fixes. No code changes required for MVP; findings are configuration/design-level.

**Next steps**: Monitor GitHub for implementation status on CSP and withGlobalTauri fixes.
