# Decision: Round-to-Started-Half-Hour Setting

**Author**: Chewie (Backend Dev)  
**Date**: 2026-04-14  
**Status**: Implemented  

---

## Context

Fredrik requested a company-policy setting: time registrations should be scoped to the *started* half-hour of the day. This means when exporting, session duration is calculated from the floor of `start_time` to the nearest 30-minute boundary, not from the raw stored start.

Examples:
- 09:17 → 09:00 (floor to start of hour)
- 09:47 → 09:30 (floor to half-past)
- 14:58 → 14:30 (floor to half-past)

**Constraint**: The raw `start_time` in the database must never be modified. Rounding is a presentation/export concern only.

---

## Settings Mechanism

### Choice: `settings` table (key/value)

Created a new `settings` table via migration 003:

```sql
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
INSERT OR IGNORE INTO settings (key, value) VALUES ('round_to_half_hour', 'false');
```

**Why key/value over a typed settings row**:
- Extensible: future settings (e.g. `default_activity_type`, `export_date_format`) can be added without schema changes
- Simple: no new Tauri plugins, no JSON config file, no Tauri Store dependency
- Consistent: all settings live in the same SQLite DB as all other data (single source of truth, survives app reinstall if DB is backed up)
- Queryable: settings can be read in SQL joins if needed in the future

**Why not Tauri Store plugin**:
- Would require adding `tauri-plugin-store` dependency
- Stores data separately from the SQLite DB (inconsistent backup/restore story)
- No advantage for simple boolean/string settings

---

## IPC Commands

Two generic commands added to `commands/settings.rs`:

```rust
get_setting(key: String) -> Result<Option<String>, AppError>
set_setting(key: String, value: String) -> Result<(), AppError>
```

**Pattern**: Generic key/value commands avoid adding a new command pair for every future setting. The frontend can call `get_setting("round_to_half_hour")` and `set_setting("round_to_half_hour", "true")`.

---

## Rounding Implementation

### Location: `services/summary_service.rs`

Three pure utility functions:

1. **`floor_to_half_hour(dt: NaiveDateTime) -> NaiveDateTime`** (public for testing):  
   `(minutes / 30) * 30` — integer division floors to nearest 30-min boundary; seconds and nanoseconds are zeroed.

2. **`parse_datetime(s: &str) -> Option<NaiveDateTime>`** (private):  
   Accepts RFC3339 (`"2024-01-15T09:17:00Z"`) and SQLite format (`"2024-01-15 09:17:00"`). Reuses the dual-format parsing pattern established in session_service.

3. **`compute_export_duration(..., round: bool) -> i64`** (private):  
   Precedence: `duration_override` → rounded calculation (if round=true) → stored `duration_seconds`.

4. **`get_round_to_half_hour(conn) -> Result<bool, AppError>`** (public):  
   Reads the `round_to_half_hour` setting; defaults to `false` if row not found.

### Where rounding is applied

- `export_csv()` — standard CSV export (Duration (minutes) column)
- `export_servicenow_csv()` — ServiceNow Import Set (duration_hours column)

Both functions now accept `round_to_half_hour: bool`. The commands layer (`commands/reports.rs`) reads the setting from DB and passes it to the service.

### What is NOT affected by rounding

- The `opened_at` / `start_time` display columns in exports — these remain the raw stored value
- The `get_daily_summary` and `get_report` aggregated queries — these show actual tracked time
- Stored `duration_seconds` or `start_time` in the database — never modified
- Sessions with `duration_override` set — manual edits always win

---

## Override Precedence

When `duration_override` is set (user manually edited the duration), that value is used as-is — even when `round_to_half_hour` is true. Rationale: the user explicitly chose a different duration, and the rounding policy should not silently override that deliberate decision.

---

## Files Changed

| File | Change |
|------|--------|
| `src-tauri/migrations/003_settings.sql` | NEW: settings table + default row |
| `src-tauri/src/db/mod.rs` | Added migration v3 to `run_migrations()` |
| `src-tauri/src/commands/settings.rs` | NEW: `get_setting` / `set_setting` commands |
| `src-tauri/src/commands/mod.rs` | Added `pub mod settings` |
| `src-tauri/src/services/summary_service.rs` | Rounding utilities + updated export signatures |
| `src-tauri/src/commands/reports.rs` | Reads setting, passes to service functions |
| `src-tauri/src/lib.rs` | Registered `get_setting` / `set_setting` commands |
| `src-tauri/tests/summary_service_tests.rs` | Updated 3 existing calls + 4 new rounding tests |
