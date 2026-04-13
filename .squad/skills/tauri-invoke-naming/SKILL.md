# Skill: Tauri 2 invoke() Parameter Naming Convention

**Stack:** Tauri 2 + Svelte 5 (TypeScript)  
**Confidence:** high  
**Domain:** frontend, api, tauri  
**Source:** earned (discovered through repeated production bugs)

---

## The Rule

**ALL `invoke()` calls in `src/lib/api/*.ts` MUST use camelCase keys. No exceptions.**

---

## Why This Matters

### Root Cause: Tauri 2 Automatic Conversion Behavior

Tauri 2 automatically converts **camelCase** → **snake_case** before passing parameters to Rust command handlers.

**Critical detail:** If the frontend sends **snake_case directly**, Tauri does NOT convert it — the key arrives as-is, and Rust's `#[command]` macro can't match it to its snake_case parameter.

### What Goes Wrong

```typescript
// ❌ WRONG — Tauri leaves snake_case untouched
await invoke('create_session', {
  work_order_id: 123,      // Arrives as "work_order_id" in Rust
  activity_type: 'Dev'     // Arrives as "activity_type" in Rust
});

// Rust command signature:
#[command]
fn create_session(work_order_id: i64, activity_type: String) { ... }

// Result: ERROR — "missing required key workOrderId"
```

**Why?** Rust command macro expects Tauri to send `work_order_id` (after converting `workOrderId` → `work_order_id`). But we sent `work_order_id` directly, so Tauri left it unchanged. The macro is looking for the converted form from camelCase input.

```typescript
// ✅ CORRECT — Tauri converts camelCase → snake_case
await invoke('create_session', {
  workOrderId: 123,        // Tauri converts → "work_order_id"
  activityType: 'Dev'      // Tauri converts → "activity_type"
});

// Result: SUCCESS — parameters match Rust signature
```

---

## The Pattern

### All invoke() Parameters Must Be camelCase

| Frontend (invoke param) | Tauri Converts To | Rust Param Name |
|-------------------------|-------------------|-----------------|
| `workOrderId` | `work_order_id` | `work_order_id` |
| `activityType` | `activity_type` | `activity_type` |
| `sessionId` | `session_id` | `session_id` |
| `customerId` | `customer_id` | `customer_id` |
| `startDate` | `start_date` | `start_date` |
| `endDate` | `end_date` | `end_date` |
| `favoritesOnly` | `favorites_only` | `favorites_only` |
| `includeArchived` | `include_archived` | `include_archived` |

---

## All Known Parameters (Current API Surface)

These are all parameters used in `src/lib/api/*.ts` as of 2026-04-13.

### sessions.ts
- `workOrderId` → `work_order_id`
- `activityType` → `activity_type`
- `sessionId` → `session_id`
- `startDate` → `start_date`
- `endDate` → `end_date`
- `durationSeconds` → `duration_seconds`

### workOrders.ts
- `customerId` → `customer_id`
- `workOrderId` → `work_order_id`
- `favoritesOnly` → `favorites_only`

### customers.ts
- `customerId` → `customer_id`
- `includeArchived` → `include_archived`

### reports.ts
- `startDate` → `start_date`
- `endDate` → `end_date`

---

## Examples

### ✅ Correct Patterns

```typescript
// sessions.ts
export async function createSession(
  workOrderId: number,
  activityType?: string,
  notes?: string
): Promise<TimeSession> {
  return invoke('create_session', {
    workOrderId,        // ✅ camelCase
    activityType,       // ✅ camelCase
    notes
  });
}

export async function updateSession(
  sessionId: number,
  durationSeconds?: number,
  notes?: string
): Promise<void> {
  return invoke('update_session', {
    sessionId,          // ✅ camelCase
    durationSeconds,    // ✅ camelCase
    notes
  });
}
```

### ❌ Anti-Patterns (Bugs This Rule Prevents)

```typescript
// ❌ WRONG — snake_case bypasses Tauri conversion
export async function createSession(
  work_order_id: number,
  activity_type?: string
): Promise<TimeSession> {
  return invoke('create_session', {
    work_order_id,      // ❌ BREAKS — Tauri won't convert
    activity_type       // ❌ BREAKS — Tauri won't convert
  });
}

// ❌ WRONG — mixing camelCase and snake_case
export async function getReport(
  startDate: string,
  end_date: string       // ❌ Inconsistent
): Promise<ReportData> {
  return invoke('get_report', {
    startDate,           // ✅ Works
    end_date             // ❌ BREAKS — Tauri won't convert
  });
}
```

---

## Checklist for New invoke() Calls

When adding a new Tauri command:

1. ✅ **Frontend parameter names are camelCase** (e.g., `workOrderId`, not `work_order_id`)
2. ✅ **Rust signature uses snake_case** (e.g., `work_order_id: i64`)
3. ✅ **Let Tauri do the conversion** — never manually convert naming conventions
4. ✅ **Test the command** — verify parameters are received correctly on Rust side
5. ✅ **Update this skill doc** — add new params to the "All Known Parameters" table

---

## History: Bugs This Rule Prevents

This naming convention has been violated **8+ times** across all API files, causing production errors every time:

### Real Production Bugs

1. **sessions.ts** (2026-04-13)
   - `work_order_id` → `workOrderId` ✅ fixed
   - `activity_type` → `activityType` ✅ fixed
   - `session_id` → `sessionId` ✅ fixed
   - `duration_seconds` → `durationSeconds` ✅ fixed

2. **workOrders.ts** (2026-04-13)
   - `work_order_id` → `workOrderId` ✅ fixed
   - `customer_id` → `customerId` ✅ fixed
   - `favorites_only` → `favoritesOnly` ✅ fixed

3. **customers.ts** (2026-04-13)
   - `customer_id` → `customerId` ✅ fixed
   - `include_archived` → `includeArchived` ✅ fixed

4. **reports.ts** (2026-04-13)
   - `start_date` → `startDate` ✅ fixed (first pass)
   - `end_date` → `endDate` ✅ fixed (first pass)
   - **Then `getReport` was missed** — caught on second review ✅ fixed

### Pattern

Every snake_case parameter sent to `invoke()` caused:
- ❌ Runtime error: `"missing required key <expectedCamelCaseName>"`
- ❌ Silent failures in UI (forms/buttons not responding)
- ❌ Debugging confusion (parameters visually present but not recognized)

**Lesson:** Tauri 2's conversion is **silent and automatic** — you only find out it's broken at runtime, not compile time.

---

## Enforcement

- **Code Review:** All new API files (`src/lib/api/*.ts`) must be reviewed for camelCase compliance before merge
- **Testing:** All `invoke()` calls must have corresponding integration tests that verify Rust receives parameters correctly
- **Linting:** Consider adding an ESLint rule to detect snake_case in invoke() calls (future enhancement)

---

## References

- Tauri 2 IPC Documentation: [https://v2.tauri.app/develop/calling-rust/](https://v2.tauri.app/develop/calling-rust/)
- Team decision: `.squad/decisions/inbox/chewie-tauri-naming-rule.md`
- All API wrappers: `src/lib/api/*.ts`
