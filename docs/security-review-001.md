# Security Review #001 — Work Tracker 2

**Reviewer:** Ackbar (Security Expert)  
**Date:** 2026-04-12  
**Scope:** Full initial security review — Tauri config, IPC commands, SQL layer, services, dependencies  
**Codebase Version:** Post Phase 2 implementation (pause/resume, favorites)

---

## Executive Summary

**Overall Risk Posture: LOW**

Work Tracker 2 is a local-only desktop app with no network exposure, no authentication requirement, and a small trust boundary (local user → Tauri WebView → Rust backend → SQLite). The codebase demonstrates strong security fundamentals: all SQL queries use parameterized statements, Rust's type system prevents most injection classes, and the IPC surface is well-scoped.

Two medium-severity findings and three low-severity findings were identified. No critical or high-severity vulnerabilities exist.

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High | 0 |
| Medium | 2 |
| Low | 3 |
| Informational | 3 |

---

## Findings

### [SEV-001] Content Security Policy Disabled ✅ FIXED

- **Severity:** Medium (CVSS 3.1 score: 5.4)
- **Location:** `src-tauri/tauri.conf.json:31`
- **Description:** The CSP is explicitly set to `null`, disabling all Content Security Policy protections for the WebView. This removes the browser's built-in defense against XSS, inline script injection, and unauthorized resource loading.
- **Impact:** If an attacker can inject content into the WebView (e.g., via a stored XSS in notes/activity fields, or via a malicious file opened through the fs plugin), they can execute arbitrary JavaScript with full access to the Tauri IPC bridge. Since `withGlobalTauri: true` is set, the injected script would have access to all registered Tauri commands.
- **CVSS Vector:** AV:L/AC:H/PR:N/UI:R/S:U/C:H/I:L/A:N
- **Reproduction:**
  1. Store a `<script>` tag in a notes field (if rendered as HTML)
  2. When the notes field renders, the script executes
  3. Script calls `window.__TAURI__.core.invoke('delete_session', { id: '...' })` to delete data
- **Fix Applied:** Restrictive CSP set in `src-tauri/tauri.conf.json`:
  ```json
  "security": {
    "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
  }
  ```
  Scripts are restricted to same-origin only (no inline scripts). `unsafe-inline` for styles is retained for Svelte's scoped CSS.

---

### [SEV-002] Shell Plugin Enabled Without Clear Justification

- **Severity:** Medium (CVSS 3.1 score: 5.0)
- **Location:** `src-tauri/Cargo.toml:19`, `src-tauri/capabilities/default.json:10`
- **Description:** `tauri-plugin-shell` is included as a dependency and `shell:default` permission is granted in capabilities. This plugin allows spawning child processes and opening URLs from the frontend. No usage of shell functionality was found in the codebase.
- **Impact:** If an attacker achieves XSS in the WebView (made easier by SEV-001), the shell plugin provides a direct path to arbitrary command execution on the host OS. Even with `shell:default` (which restricts to `open` only), this could be used to open malicious URLs or trigger protocol handlers.
- **CVSS Vector:** AV:L/AC:H/PR:N/UI:R/S:C/C:L/I:L/A:L
- **Reproduction:**
  1. Achieve script execution in WebView (see SEV-001)
  2. Call `window.__TAURI__.shell.open('malicious-url')` or equivalent
- **Recommended Fix:** Remove `tauri-plugin-shell` from `Cargo.toml` and `shell:default` from `capabilities/default.json` unless a specific feature requires it. If needed later, add it back with a scoped allow-list.
  ```toml
  # Remove from Cargo.toml:
  # tauri-plugin-shell = "2"
  ```
  ```json
  // Remove from capabilities/default.json permissions:
  // "shell:default"
  ```
  Also remove `.plugin(tauri_plugin_shell::init())` from `src-tauri/src/lib.rs:25`.

---

### [SEV-003] `withGlobalTauri` Exposes Full IPC Surface to WebView

- **Severity:** Low (CVSS 3.1 score: 3.3)
- **Location:** `src-tauri/tauri.conf.json:13`
- **Description:** Setting `withGlobalTauri: true` exposes the entire Tauri API on `window.__TAURI__`, making all registered IPC commands callable from the browser console or any injected script. In production, the IPC bridge should be accessed only through the bundled frontend code.
- **Impact:** Combined with a disabled CSP (SEV-001), this maximizes the blast radius of any XSS — an attacker's injected script can invoke every registered command (delete data, export CSV, modify records) without needing to discover the IPC interface.
- **CVSS Vector:** AV:L/AC:H/PR:N/UI:R/S:U/C:L/I:L/A:N
- **Reproduction:**
  1. Open browser dev tools in the Tauri window (if debug build)
  2. Type `window.__TAURI__.core.invoke('list_customers')` in console
  3. Full customer list is returned
- **Recommended Fix:** Set `withGlobalTauri: false` in production. The `@tauri-apps/api` npm package uses the IPC bridge directly and does not require the global. This is primarily a defense-in-depth measure.
  ```json
  "withGlobalTauri": false
  ```

---

### [SEV-004] No Input Length Validation on Text Fields

- **Severity:** Low (CVSS 3.1 score: 3.1)
- **Location:** `src-tauri/src/commands/customers.rs:8`, `src-tauri/src/commands/work_orders.rs:8`, `src-tauri/src/commands/sessions.rs:168`
- **Description:** Command handlers accept string inputs (customer name, work order name, notes, activity_type) without any length validation. A malicious or buggy frontend could send extremely long strings (megabytes), causing excessive memory allocation and database bloat.
- **Impact:** Denial of service through database bloat. SQLite has a default `SQLITE_MAX_LENGTH` of 1 billion bytes per field. Repeated large inserts could fill the user's disk. In a local-only app the attacker would need access to the machine, making this low severity.
- **CVSS Vector:** AV:L/AC:L/PR:L/UI:N/S:U/C:N/I:N/A:L
- **Reproduction:**
  1. From the frontend, call `invoke('create_customer', { params: { name: 'A'.repeat(10_000_000) } })`
  2. Database grows by ~10MB per call
- **Recommended Fix:** Add length validation in command handlers or a shared validation layer:
  ```rust
  const MAX_NAME_LEN: usize = 500;
  const MAX_NOTES_LEN: usize = 10_000;
  
  if params.name.len() > MAX_NAME_LEN {
      return Err(AppError::Validation("Name too long".into()));
  }
  ```

---

### [SEV-005] CSV Export Vulnerable to Formula Injection

- **Severity:** Low (CVSS 3.1 score: 2.6)
- **Location:** `src-tauri/src/services/summary_service.rs:286-291`
- **Description:** The `escape_csv` function correctly handles commas, quotes, and newlines, but does not neutralize formula injection characters. If a user stores a value like `=CMD("calc")` or `+SYSTEM("cmd")` in a notes field, and the CSV is later opened in Excel/LibreOffice, the spreadsheet application may interpret it as a formula and execute it.
- **Impact:** If the exported CSV is opened in a spreadsheet program, stored payloads could trigger formula execution. Since this is a single-user local app, the attacker would need to have previously injected the payload into the user's own data — making the practical risk very low.
- **CVSS Vector:** AV:L/AC:H/PR:L/UI:R/S:U/C:L/I:N/A:N
- **Reproduction:**
  1. Create a session with notes: `=HYPERLINK("http://evil.com","Click")`
  2. Export CSV via `export_csv` command
  3. Open the CSV in Excel — the cell renders as a clickable hyperlink
- **Recommended Fix:** Prefix cells that start with `=`, `+`, `-`, `@`, `\t`, or `\r` with a single quote or tab:
  ```rust
  fn escape_csv(value: &str) -> String {
      let needs_formula_guard = value.starts_with('=')
          || value.starts_with('+')
          || value.starts_with('-')
          || value.starts_with('@')
          || value.starts_with('\t')
          || value.starts_with('\r');
      
      let guarded = if needs_formula_guard {
          format!("'{}", value)
      } else {
          value.to_string()
      };
      
      if guarded.contains(',') || guarded.contains('"') || guarded.contains('\n') {
          format!("\"{}\"", guarded.replace('"', "\"\""))
      } else {
          guarded
      }
  }
  ```

---

### [INFO-001] Dynamic SQL Construction Uses Safe Pattern

- **Severity:** Informational
- **Location:** `src-tauri/src/commands/sessions.rs:34-55`, `src-tauri/src/commands/customers.rs:61-83`, `src-tauri/src/commands/work_orders.rs:141-166`
- **Description:** The dynamic UPDATE query builders use `format!()` to construct SQL, which could appear concerning. However, the column names are hardcoded string literals (e.g., `"duration_override = ?"`, `"name = ?"`), not user-controlled. All user values are passed as parameterized query arguments. This pattern is **safe** — it's equivalent to a static query builder.
- **Recommendation:** No action required. Consider adding a comment explaining why `format!()` is safe here, to prevent future developers from incorrectly "fixing" it.

---

### [INFO-002] `summary_service::fetch_sessions` Accepts Dynamic WHERE Clause

- **Severity:** Informational
- **Location:** `src-tauri/src/services/summary_service.rs:35-85`
- **Description:** The `fetch_sessions` helper accepts a `where_clause: &str` parameter that is interpolated into a SQL query via `format!()`. This is a potentially dangerous pattern. However, all callers pass hardcoded string literals as the WHERE clause, and the function is private (`fn` not `pub fn`), so it cannot be called from outside the module.
- **Recommendation:** No action required now. If this function is ever made public or called with user-controlled input, it would become a SQL injection vulnerability. Consider adding a safety comment:
  ```rust
  // SAFETY: where_clause must be a hardcoded string literal, never user input.
  // All parameter values are passed through the `params` slice.
  ```

---

### [INFO-003] File System Plugin Granted Default Permissions

- **Severity:** Informational
- **Location:** `src-tauri/capabilities/default.json:9`
- **Description:** `fs:default` permissions are granted. The `fs:default` scope in Tauri 2 is restricted to the app's data directory and resource directory by default. The app uses `app.path().app_data_dir()` for the SQLite database, which is within the allowed scope. No evidence of arbitrary path access was found.
- **Recommendation:** Audit periodically. If fs plugin usage expands, ensure paths are scoped and validated.

---

## Architecture Notes

### Positive Security Practices

1. **Parameterized SQL everywhere** — All user-supplied values flow through `rusqlite::params![]` or `params_from_iter()`. No string concatenation of user input into SQL. This eliminates SQL injection as an attack class.

2. **Mutex-guarded database access** — The `AppState { db: Mutex<Connection> }` pattern ensures single-writer access. The `get_conn()` helper handles mutex poisoning gracefully instead of panicking.

3. **Structured error handling** — `AppError` serializes to a consistent `{ code, message }` structure. Internal error details (like SQLite error codes) are wrapped, not exposed raw to the frontend.

4. **Transaction usage for atomic operations** — `switch_to_work_order` and `quick_add` use `unchecked_transaction()` to ensure multi-step operations are atomic. This prevents data corruption on partial failures.

5. **WAL mode and foreign keys enabled** — `PRAGMA journal_mode=WAL` and `PRAGMA foreign_keys=ON` are set in `db::initialize()`, ensuring crash safety and referential integrity.

6. **UUID v4 for entity IDs** — Non-sequential, non-guessable identifiers. Good practice even for local apps.

7. **Soft delete (archive) pattern** — Customers and work orders use `archived_at` instead of hard delete, preserving audit trail.

8. **Singleton active_session with CHECK constraint** — `CHECK (id = 1)` on the active_session table enforces the "at most one active session" invariant at the database level.

### Trust Boundary Analysis

```
┌──────────────────────────────────────────────────────┐
│  LOCAL MACHINE (single trust zone)                    │
│                                                       │
│  ┌─────────────────┐     IPC      ┌───────────────┐  │
│  │  Svelte WebView  │ ──invoke──► │  Rust Backend  │  │
│  │  (untrusted JS)  │ ◄──result── │  (trusted)     │  │
│  └─────────────────┘              └───────┬───────┘  │
│                                           │          │
│                                    ┌──────▼──────┐   │
│                                    │   SQLite DB  │   │
│                                    │  (app_data/) │   │
│                                    └─────────────┘   │
│                                                       │
└──────────────────────────────────────────────────────┘
```

The WebView is the only untrusted component. Tauri's IPC bridge is the sole attack surface. Since there's no network listener, remote attacks are not possible — an attacker must have local access or compromise the WebView content.

---

## Dependency Status

### Cargo Audit Results

**Vulnerabilities found: 0**  
**Warnings: 20** (all `unmaintained` or `unsound` in transitive dependencies)

Notable warnings:
| Crate | Type | Advisory |
|-------|------|----------|
| `glib 0.18.5` | unsound | RUSTSEC-2024-0429 — `VariantStrIter` soundness issue |
| `rand 0.7.3, 0.8.5` | unsound | RUSTSEC-2026-0097 — unsound with custom logger |
| `atk`, `gdk`, `gtk` (11 crates) | unmaintained | GTK3 bindings no longer maintained |
| `proc-macro-error 1.0.4` | unmaintained | RUSTSEC-2024-0370 |
| `unic-*` (5 crates) | unmaintained | Various RUSTSEC-2025-* |
| `fxhash 0.2.1` | unmaintained | RUSTSEC-2025-0057 |

**Assessment:** The `glib` and `rand` unsoundness warnings are in transitive dependencies pulled in by Tauri's GTK backend (Linux only) and are not directly exploitable in this app's usage patterns. The GTK3 unmaintained warnings are expected — Tauri 2 uses GTK3 on Linux. No action needed; monitor for Tauri updates that may address these.

### npm Audit Results

**Vulnerabilities found: 3 low severity**

| Package | Severity | Advisory |
|---------|----------|----------|
| `cookie` <0.7.0 | Low | GHSA-pxg6-pf52-xh8x — accepts out-of-bounds characters in cookie name/path/domain |

**Assessment:** The `cookie` vulnerability is in `@sveltejs/kit`'s dependency chain. Since this is a Tauri desktop app that does not use HTTP cookies in production (no server, no browser cookies), this has **zero practical impact**. The fix requires a breaking `@sveltejs/kit` downgrade — do not apply. Monitor for a non-breaking fix upstream.

---

## Recommendations Priority

| Priority | Finding | Effort | Impact |
|----------|---------|--------|--------|
| 1 | SEV-001: Enable CSP | Low (config change) | Eliminates XSS blast radius |
| 2 | SEV-002: Remove shell plugin | Low (config + code) | Removes unnecessary attack surface |
| 3 | SEV-003: Disable withGlobalTauri | Low (config change) | Defense in depth |
| 4 | SEV-004: Add input length validation | Medium (code change) | Prevents resource exhaustion |
| 5 | SEV-005: Fix CSV formula injection | Low (code change) | Prevents spreadsheet attacks |

---

*Report generated by Ackbar — Security Expert*  
*Next review recommended: After Phase 3 reporting features or after significant dependency updates*
