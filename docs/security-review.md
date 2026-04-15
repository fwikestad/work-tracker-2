# Security Review — Work Tracker 2

**Date:** 2026-04-13  
**Reviewer:** Han (Lead)  
**Scope:** Pre-Delivery Audit (Tauri 2 desktop app)

---

## Executive Summary

This security review covers the Tauri 2 desktop application (SvelteKit frontend + Rust backend + SQLite). The application is designed as a **local-only, single-user desktop app** with no network connectivity requirements for core features.

**Overall Risk Assessment:** LOW for intended use case (local desktop app for single consultant)

| Severity | Count | Status |
|----------|-------|--------|
| Critical | 0 | — |
| High | 1 | **FIXED** (shell plugin removed) |
| Medium | 2 | 1 FIXED (shell:default), 1 NOTED (CSP) |
| Low/Info | 5 | Documented |

---

## 1. Critical Findings

**None identified.**

All critical security paths (SQL injection, command injection, sensitive data exposure) were reviewed and found to be properly handled.

---

## 2. High Severity Findings

### 2.1 Unused Shell Plugin Registered ✅ FIXED

**Location:** `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`

**Finding:** The `tauri-plugin-shell` was included in dependencies, initialized in the Rust backend, and `shell:default` permission was granted in capabilities — but the shell plugin is **completely unused** by the application.

**Risk:** If the webview were compromised (e.g., via XSS if CSP were disabled and remote content loaded), an attacker could potentially spawn shell processes on the user's system.

**Evidence:**
- `Cargo.toml` line 19: `tauri-plugin-shell = "2"`
- `lib.rs` line 33: `.plugin(tauri_plugin_shell::init())`
- `capabilities/default.json` line 12: `"shell:default"`
- No imports of `@tauri-apps/plugin-shell` in frontend
- No shell-related IPC calls anywhere in codebase

**Fix Applied:**
1. Removed `tauri-plugin-shell = "2"` from `Cargo.toml`
2. Removed `.plugin(tauri_plugin_shell::init())` from `lib.rs`
3. Removed `"shell:default"` from `capabilities/default.json`

**Verification:** Build passes after removal, no functionality affected.

---

## 3. Medium Severity Findings

### 3.1 CSP Disabled (`csp: null`) — NOTED (Recommendation Only)

**Location:** `src-tauri/tauri.conf.json` line 27

**Finding:** Content Security Policy is explicitly disabled.

**Risk Assessment for This App: LOW**

For a **local-only desktop app** that:
- Loads no remote URLs
- Has no external API integrations
- Serves all content from local SvelteKit build
- Has no user-generated HTML rendering (`{@html}` not used)

The risk of XSS exploitation is minimal because:
1. No vector for injecting remote scripts (no remote content loaded)
2. No `{@html}` directives that could render user input as HTML
3. User data (names, notes) is rendered as text content, not HTML

**Recommendation:** For defense-in-depth, consider enabling a permissive CSP:

```json
"security": {
  "csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
}
```

**Action Required:** Manual testing needed before enabling. SvelteKit may require `unsafe-inline` for styles. **Not implemented in this review** — requires user approval and testing.

### 3.2 File System Permissions Not Scoped ✅ FIXED (Partial)

**Location:** `src-tauri/capabilities/default.json`

**Finding:** `fs:allow-write-text-file` is granted without path scope restrictions.

**Risk:** The app can write to any location the user has access to, not just app data or user-selected paths.

**Current Usage:** The fs plugin is used only for CSV export via the save dialog. The dialog plugin already restricts writes to user-selected paths.

**Fix Applied:** Removed unneeded permissions:
- `fs:default` removed (not used)
- `fs:allow-write-text-file` retained (needed for CSV export, but scoped by dialog selection)

**Recommendation for Future:** If file operations expand, add explicit path scopes:
```json
{
  "identifier": "fs:allow-write-text-file",
  "allow": [{ "path": "$DOCUMENT/**" }]
}
```

---

## 4. Low/Informational Findings

### 4.1 `withGlobalTauri` — **FIXED**

**Location:** `src-tauri/tauri.conf.json`

**Finding:** Previously `withGlobalTauri: true` exposed the entire Tauri IPC layer via `window.__TAURI__` to all scripts in the WebView, amplifying the impact of any XSS vulnerability.

**Resolution:** Set to `withGlobalTauri: false`. All frontend code uses explicit imports (`import { invoke } from '@tauri-apps/api/core'`), so no functional change was required. The `window.__TAURI__` global is no longer exposed.

### 4.2 No Input Length Validation on User Strings

**Location:** All command handlers in `src-tauri/src/commands/`

**Finding:** User-supplied strings (customer names, work order names, notes) have no maximum length validation.

**Risk Assessment: LOW** because:
- SQLite handles arbitrarily long strings
- No buffer overflow risk in Rust
- DoS via extremely long strings is mitigated by Tauri's IPC size limits

**Recommendation:** Consider adding reasonable limits (e.g., 255 chars for names, 10KB for notes) for UX consistency and to prevent accidental large data entry.

### 4.3 Console Logging of Errors in Frontend

**Location:** Various Svelte components

**Finding:** Error details are logged to console:
- `DailySummary.svelte`: `console.error('Failed to refresh daily summary:', e)`
- `QuickAdd.svelte`: `console.error('Quick add failed:', e)`
- `timer.svelte.ts`: Multiple error logs

**Risk Assessment: INFORMATIONAL**
- No sensitive data logged
- Errors are application errors, not user secrets
- Acceptable for desktop app debugging

**Recommendation:** Consider adding a debug mode toggle for production builds.

### 4.4 Unmaintained GTK Dependencies (cargo audit warnings)

**Finding:** `cargo audit` reports multiple `RUSTSEC-2024-04xx` warnings for GTK3 bindings (atk, cairo, gdk, etc.) marked as unmaintained.

**Risk Assessment: LOW**
- These are transitive dependencies from Tauri's Linux support
- Not security vulnerabilities, just maintenance warnings
- Tauri team will address in future releases

**Action Required:** None. Monitor Tauri releases for updates.

### 4.5 Low-Severity npm Vulnerability (cookie package)

**Finding:** `npm audit` reports a low-severity issue in the `cookie` package (GHSA-pxg6-pf52-xh8x).

**Risk Assessment: NEGLIGIBLE**
- The cookie package is a transitive dependency of SvelteKit
- This app doesn't use cookies (it's a desktop app)
- The vulnerability relates to out-of-bounds characters in cookie parsing

**Action Required:** None. Will be resolved when SvelteKit updates dependencies.

---

## 5. Safe to Ship As-Is

The following items were reviewed and found to be properly implemented:

### 5.1 SQL Injection Protection ✅

All SQL queries use parameterized statements via `rusqlite::params![]`. No string concatenation of user input into SQL.

**Verified in:**
- `src-tauri/src/commands/customers.rs`
- `src-tauri/src/commands/work_orders.rs`
- `src-tauri/src/commands/sessions.rs`
- `src-tauri/src/services/session_service.rs`
- `src-tauri/src/services/summary_service.rs`

### 5.2 IPC Command Validation ✅

All registered commands properly validate:
- Work order existence before session start
- Customer existence before work order creation
- Session existence before operations
- State validity (can't pause already-paused session)

### 5.3 Database Location ✅

SQLite database is stored in the app data directory (`app.path().app_data_dir()`), which is:
- Windows: `%APPDATA%\com.work-tracker-2.app\work_tracker.db`
- macOS: `~/Library/Application Support/com.work-tracker-2.app/work_tracker.db`
- Linux: `~/.local/share/com.work-tracker-2.app/work_tracker.db`

This is the correct secure location for app data.

### 5.4 No XSS Vectors in Frontend ✅

Searched for `{@html}` directive — none found. All user content is rendered as text.

### 5.5 No Hardcoded Secrets ✅

Searched for API keys, tokens, passwords — none found. App has no external integrations.

### 5.6 Error Handling ✅

All commands return structured `Result<T, AppError>` types. Mutex poisoning is handled gracefully via `get_conn()` helper.

---

## 6. Changes Made in This Review

| File | Change | Reason |
|------|--------|--------|
| `src-tauri/Cargo.toml` | Removed `tauri-plugin-shell = "2"` | Unused, security risk |
| `src-tauri/src/lib.rs` | Removed `.plugin(tauri_plugin_shell::init())` | Unused |
| `src-tauri/capabilities/default.json` | Removed `"shell:default"`, `"fs:default"` | Unused, reduce attack surface |

---

## 7. Recommendations for Future

1. **Enable CSP** after testing SvelteKit compatibility
2. **Add input length validation** for better UX
3. **Scope fs permissions** if file operations expand
4. **Monitor Tauri releases** for GTK dependency updates
5. **Consider code signing** for production distribution

---

## 8. Conclusion

The application is **safe to ship** for its intended use case as a local-only desktop time tracker. The primary security concern (unused shell plugin) has been fixed. The remaining items are informational or require user approval before implementation.

**Verdict:** ✅ APPROVED FOR DELIVERY (with fixes applied)
