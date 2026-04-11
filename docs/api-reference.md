# Tauri IPC Command Reference

This document describes all Tauri IPC commands available in Work Tracker 2. These are the primary interface between the Svelte frontend and Rust backend.

---

## Invocation Pattern

All commands are invoked from the frontend using the Tauri `invoke` function:

```typescript
import { invoke } from '@tauri-apps/api/core';

// Example: invoke a command and handle result
const result = await invoke<ReturnType>('command_name', { param: value });
```

**Error Handling**:
All commands return either a successful result or an `AppError` with a code and message:

```typescript
interface AppError {
  code: string;        // Error code (e.g., "NOT_FOUND", "INVALID_DURATION")
  message: string;     // Human-readable error message
  details?: Record<string, unknown>;  // Optional additional context
}
```

---

## Customer Commands

### create_customer

Create a new customer (tracked entity).

**TypeScript**: `invoke<Customer>('create_customer', params)`

**Parameters**:
- `name` (string, required): Customer name (e.g., "Acme Corp", "Internal")
- `code` (string, optional): Short reference code (e.g., "ACME", "INT")
- `color` (string, optional): Hex color for UI (e.g., "#3B82F6"). Default: system-assigned.

**Returns**: `Customer` object
```typescript
interface Customer {
  id: string;           // UUID
  name: string;
  code: string | null;
  color: string | null;
  created_at: string;   // ISO 8601 timestamp
  updated_at: string;
}
```

**Errors**:
- `INVALID_NAME` — Customer name is empty or too long
- `DUPLICATE_CODE` — Code already exists for another customer

**Example**:
```typescript
const customer = await invoke<Customer>('create_customer', {
  name: 'Acme Corp',
  code: 'ACME',
  color: '#8b5cf6'
});
```

---

### list_customers

Retrieve all customers, optionally filtered.

**TypeScript**: `invoke<Customer[]>('list_customers', params)`

**Parameters**:
- `include_archived` (boolean, optional): Whether to include soft-deleted customers. Default: false.

**Returns**: Array of `Customer` objects

**Errors**:
- `DB_ERROR` — Database query failed

**Example**:
```typescript
const customers = await invoke<Customer[]>('list_customers', {
  include_archived: false
});
```

---

### update_customer

Update a customer's name, code, or color.

**TypeScript**: `invoke<Customer>('update_customer', params)`

**Parameters**:
- `id` (string, required): Customer UUID
- `name` (string, optional): New customer name
- `code` (string, optional): New reference code
- `color` (string, optional): New hex color

**Returns**: Updated `Customer` object

**Errors**:
- `NOT_FOUND` — Customer ID does not exist
- `INVALID_NAME` — New name is empty or invalid
- `DUPLICATE_CODE` — New code already assigned to another customer

**Example**:
```typescript
const updated = await invoke<Customer>('update_customer', {
  id: 'customer-uuid-here',
  color: '#ec4899'  // Change customer color
});
```

---

### archive_customer

Soft-delete a customer (mark as archived). Associated work orders and sessions are preserved.

**TypeScript**: `invoke<void>('archive_customer', params)`

**Parameters**:
- `id` (string, required): Customer UUID

**Returns**: `null` (or void on success)

**Errors**:
- `NOT_FOUND` — Customer ID does not exist
- `ALREADY_ARCHIVED` — Customer is already archived

**Example**:
```typescript
await invoke('archive_customer', { id: 'customer-uuid' });
```

---

## Work Order Commands

### create_work_order

Create a new work order (project/task) under a customer.

**TypeScript**: `invoke<WorkOrder>('create_work_order', params)`

**Parameters**:
- `customer_id` (string, required): Parent customer UUID (must exist)
- `name` (string, required): Work order name (e.g., "API Development", "UI Design Phase 2")
- `code` (string, optional): Reference code (e.g., "API-001", "UI-P2")
- `description` (string, optional): Long-form description

**Returns**: `WorkOrder` object
```typescript
interface WorkOrder {
  id: string;           // UUID
  customer_id: string;
  name: string;
  code: string | null;
  description: string | null;
  status: 'active' | 'paused' | 'closed';
  created_at: string;   // ISO 8601 timestamp
  updated_at: string;
}
```

**Errors**:
- `CUSTOMER_NOT_FOUND` — Customer ID does not exist
- `INVALID_NAME` — Work order name is empty or invalid
- `DUPLICATE_CODE` — Code already exists for this customer

**Example**:
```typescript
const workOrder = await invoke<WorkOrder>('create_work_order', {
  customer_id: 'acme-uuid',
  name: 'Backend API Refactor',
  code: 'REFACTOR-001',
  description: 'Refactor monolith into microservices'
});
```

---

### list_work_orders

Retrieve all work orders, optionally filtered by customer.

**TypeScript**: `invoke<WorkOrder[]>('list_work_orders', params)`

**Parameters**:
- `customer_id` (string, optional): Filter by customer UUID. If omitted, returns all work orders.
- `include_archived` (boolean, optional): Whether to include archived work orders. Default: false.

**Returns**: Array of `WorkOrder` objects

**Errors**:
- `DB_ERROR` — Database query failed

**Example**:
```typescript
// Get all work orders for a customer
const workOrders = await invoke<WorkOrder[]>('list_work_orders', {
  customer_id: 'acme-uuid'
});

// Get all active work orders across all customers
const allActive = await invoke<WorkOrder[]>('list_work_orders', {
  include_archived: false
});
```

---

### update_work_order

Update a work order's name, code, description, or status.

**TypeScript**: `invoke<WorkOrder>('update_work_order', params)`

**Parameters**:
- `id` (string, required): Work order UUID
- `name` (string, optional): New name
- `code` (string, optional): New reference code
- `description` (string, optional): New description
- `status` (string, optional): New status ('active', 'paused', or 'closed')

**Returns**: Updated `WorkOrder` object

**Errors**:
- `NOT_FOUND` — Work order ID does not exist
- `INVALID_NAME` — New name is empty or invalid
- `DUPLICATE_CODE` — New code already assigned to another work order in this customer
- `INVALID_STATUS` — Status not one of 'active', 'paused', 'closed'

**Example**:
```typescript
const updated = await invoke<WorkOrder>('update_work_order', {
  id: 'workorder-uuid',
  status: 'closed'
});
```

---

### archive_work_order

Soft-delete a work order. Associated time sessions are preserved but the work order cannot be used for new sessions.

**TypeScript**: `invoke<void>('archive_work_order', params)`

**Parameters**:
- `id` (string, required): Work order UUID

**Returns**: `null` (or void on success)

**Errors**:
- `NOT_FOUND` — Work order ID does not exist
- `ALREADY_ARCHIVED` — Work order is already archived

**Example**:
```typescript
await invoke('archive_work_order', { id: 'workorder-uuid' });
```

---

## Session Commands

### start_session

Start tracking time on a work order. If another session is active, it is stopped first (atomic).

**TypeScript**: `invoke<Session>('start_session', params)`

**Parameters**:
- `work_order_id` (string, required): Work order UUID (must exist and not be archived)

**Returns**: `Session` object
```typescript
interface Session {
  id: string;                // UUID
  work_order_id: string;
  start_time: string;        // ISO 8601 timestamp
  end_time: string | null;   // NULL if session is active
  duration_seconds: number | null;
  duration_override: number | null;
  activity_type: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}
```

**Errors**:
- `WORK_ORDER_NOT_FOUND` — Work order ID does not exist
- `WORK_ORDER_ARCHIVED` — Work order is archived
- `DB_ERROR` — Database operation failed

**Example**:
```typescript
const session = await invoke<Session>('start_session', {
  work_order_id: 'workorder-uuid'
});
console.log(`Started tracking: ${session.id}`);
```

---

### stop_session

Stop the active session and save optional metadata.

**TypeScript**: `invoke<Session>('stop_session', params)`

**Parameters**:
- `notes` (string, optional): Notes or summary of work done
- `activity_type` (string, optional): Activity classification ('meeting', 'development', 'design', 'admin', etc.)
- `duration_override` (number, optional): Manual duration in seconds (overrides auto-calculated duration)

**Returns**: Updated `Session` object with `end_time` set

**Errors**:
- `NO_ACTIVE_SESSION` — No session is currently running
- `INVALID_DURATION` — Duration override is negative or zero
- `DB_ERROR` — Database operation failed

**Example**:
```typescript
const completed = await invoke<Session>('stop_session', {
  notes: 'Completed API endpoint',
  activity_type: 'development'
});
```

---

### get_active_session

Retrieve the currently active session (if any).

**TypeScript**: `invoke<Session | null>('get_active_session', {})`

**Parameters**: None

**Returns**: `Session` object if a session is active, or `null` if idle

**Errors**: None (always succeeds)

**Example**:
```typescript
const active = await invoke<Session | null>('get_active_session', {});
if (active) {
  console.log(`Tracking: ${active.work_order_id}`);
} else {
  console.log('Idle');
}
```

---

### update_session

Update a completed session's notes, activity type, or duration.

**TypeScript**: `invoke<Session>('update_session', params)`

**Parameters**:
- `id` (string, required): Session UUID
- `notes` (string, optional): Updated notes
- `activity_type` (string, optional): Updated activity type
- `duration_override` (number, optional): Manual duration in seconds

**Returns**: Updated `Session` object

**Errors**:
- `NOT_FOUND` — Session ID does not exist
- `SESSION_STILL_ACTIVE` — Cannot edit a session that hasn't been stopped yet
- `INVALID_DURATION` — Duration override is negative or zero

**Example**:
```typescript
const updated = await invoke<Session>('update_session', {
  id: 'session-uuid',
  notes: 'Fixed critical bug in payment processing',
  activity_type: 'development',
  duration_override: 3600  // Override to exactly 1 hour
});
```

---

### list_sessions

Retrieve sessions within a date range, optionally filtered by work order.

**TypeScript**: `invoke<Session[]>('list_sessions', params)`

**Parameters**:
- `start_date` (string, required): ISO 8601 date string (e.g., "2026-04-11")
- `end_date` (string, required): ISO 8601 date string
- `work_order_id` (string, optional): Filter by work order UUID

**Returns**: Array of `Session` objects

**Errors**:
- `INVALID_DATE_RANGE` — Start date is after end date
- `DB_ERROR` — Database query failed

**Example**:
```typescript
// Get today's sessions
const today = '2026-04-11';
const sessions = await invoke<Session[]>('list_sessions', {
  start_date: today,
  end_date: today
});

// Get sessions for a specific work order this week
const thisWeek = await invoke<Session[]>('list_sessions', {
  start_date: '2026-04-07',
  end_date: '2026-04-13',
  work_order_id: 'workorder-uuid'
});
```

---

### delete_session

Delete a session permanently.

**TypeScript**: `invoke<void>('delete_session', params)`

**Parameters**:
- `id` (string, required): Session UUID

**Returns**: `null` (or void on success)

**Errors**:
- `NOT_FOUND` — Session ID does not exist

**Example**:
```typescript
await invoke('delete_session', { id: 'session-uuid' });
```

---

### quick_add

Create a customer (if needed) and work order, then immediately start tracking. Single atomic operation.

**TypeScript**: `invoke<QuickAddResult>('quick_add', params)`

**Parameters**:
- `customer_name` (string, optional): Create a new customer with this name. Required if `customer_id` is not provided.
- `customer_id` (string, optional): Use an existing customer by UUID. Required if `customer_name` is not provided.
- `work_order_name` (string, required): New work order name
- `work_order_code` (string, optional): Work order reference code

**Returns**: `QuickAddResult` object
```typescript
interface QuickAddResult {
  customer: Customer;
  work_order: WorkOrder;
  session: Session;
}
```

**Errors**:
- `INVALID_CUSTOMER_NAME` — Customer name is empty
- `INVALID_WORK_ORDER_NAME` — Work order name is empty
- `CUSTOMER_NOT_FOUND` — Customer ID provided but does not exist
- `DB_ERROR` — Database operation failed

**Example**:
```typescript
// Create new customer and start tracking
const result = await invoke<QuickAddResult>('quick_add', {
  customer_name: 'New Client Inc',
  work_order_name: 'Website Redesign'
});

// Use existing customer
const result2 = await invoke<QuickAddResult>('quick_add', {
  customer_id: 'existing-customer-uuid',
  work_order_name: 'Monthly Retainer Work'
});
```

---

### recover_session

Resume an orphan session (detected on startup after a crash). This closes the orphaned session with the current time.

**TypeScript**: `invoke<Session>('recover_session', params)`

**Parameters**:
- `session_id` (string, required): Session UUID of the orphan

**Returns**: Closed `Session` object with `end_time` set

**Errors**:
- `NOT_FOUND` — Session ID does not exist
- `SESSION_NOT_ORPHAN` — Session is not orphaned (has an end_time)
- `DB_ERROR` — Database operation failed

**Example**:
```typescript
// Called when recovery dialog is shown on startup
const recovered = await invoke<Session>('recover_session', {
  session_id: 'orphan-session-uuid'
});
```

---

### discard_orphan_session

Permanently discard an orphan session (detected on startup). Use with caution.

**TypeScript**: `invoke<void>('discard_orphan_session', params)`

**Parameters**:
- `session_id` (string, required): Session UUID of the orphan

**Returns**: `null` (or void on success)

**Errors**:
- `NOT_FOUND` — Session ID does not exist
- `SESSION_NOT_ORPHAN` — Session is not orphaned

**Example**:
```typescript
// Called when user chooses to discard on startup
await invoke('discard_orphan_session', { session_id: 'orphan-uuid' });
```

---

## Report & Export Commands

### get_daily_summary

Generate a summary of time tracked for a specific date.

**TypeScript**: `invoke<DailySummary>('get_daily_summary', params)`

**Parameters**:
- `date` (string, required): ISO 8601 date string (e.g., "2026-04-11")

**Returns**: `DailySummary` object
```typescript
interface DailySummary {
  date: string;                    // ISO 8601 date
  total_seconds: number;           // Total tracked time
  by_customer: {                   // Grouped by customer
    customer_id: string;
    customer_name: string;
    total_seconds: number;
    work_orders: {
      work_order_id: string;
      work_order_name: string;
      total_seconds: number;
      sessions: number;            // Count of sessions
    }[];
  }[];
}
```

**Errors**:
- `INVALID_DATE` — Date is not a valid ISO 8601 string
- `DB_ERROR` — Database query failed

**Example**:
```typescript
const summary = await invoke<DailySummary>('get_daily_summary', {
  date: '2026-04-11'
});

console.log(`Total today: ${summary.total_seconds / 3600} hours`);
summary.by_customer.forEach(c => {
  console.log(`  ${c.customer_name}: ${c.total_seconds / 3600}h`);
});
```

---

### get_recent_work_orders

Retrieve frequently-used work orders for quick-switch UI.

**TypeScript**: `invoke<RecentWorkOrder[]>('get_recent_work_orders', params)`

**Parameters**:
- `limit` (number, optional): Maximum number of results. Default: 10.

**Returns**: Array of `RecentWorkOrder` objects (ordered by most recent first)
```typescript
interface RecentWorkOrder {
  id: string;                 // UUID
  customer_id: string;
  customer_name: string;
  work_order_id: string;
  work_order_name: string;
  work_order_code: string | null;
  last_used_at: string;       // ISO 8601 timestamp
  use_count: number;          // Total times used
}
```

**Errors**:
- `INVALID_LIMIT` — Limit is negative or zero
- `DB_ERROR` — Database query failed

**Example**:
```typescript
const recent = await invoke<RecentWorkOrder[]>('get_recent_work_orders', {
  limit: 10
});

// Render quick-switch dropdown
recent.forEach(r => {
  console.log(`${r.customer_name} / ${r.work_order_name}`);
});
```

---

### export_csv

Generate a CSV export of sessions within a date range.

**TypeScript**: `invoke<string>('export_csv', params)`

**Parameters**:
- `start_date` (string, required): ISO 8601 date string
- `end_date` (string, required): ISO 8601 date string
- `group_by` (string, optional): Grouping level ('customer', 'work_order', 'day'). Default: 'customer'.

**Returns**: CSV string (header + rows)

**Errors**:
- `INVALID_DATE_RANGE` — Start date is after end date
- `DB_ERROR` — Database query failed

**CSV Format** (example):
```
Date,Customer,Work Order,Duration (hours),Activity Type,Notes
2026-04-11,Acme Corp,API Development,2.5,development,Implemented payment endpoint
2026-04-11,Acme Corp,Meetings,1.0,meeting,Sprint planning
2026-04-11,Internal,Admin,0.5,admin,Status report
```

**Example**:
```typescript
const csv = await invoke<string>('export_csv', {
  start_date: '2026-04-01',
  end_date: '2026-04-30',
  group_by: 'customer'
});

// Save to file (using file dialog or other mechanism)
console.log(csv);
```

---

## Error Codes Reference

Common error codes returned by commands:

| Code | Meaning |
|------|---------|
| `NOT_FOUND` | Entity (customer, work order, session) does not exist |
| `INVALID_NAME` | Name field is empty, too long, or invalid format |
| `INVALID_DATE` | Date string is not ISO 8601 format |
| `INVALID_DATE_RANGE` | Start date is after end date |
| `DUPLICATE_CODE` | Code is already in use for another entity |
| `DB_ERROR` | General database operation error |
| `NO_ACTIVE_SESSION` | No session is currently tracking |
| `SESSION_STILL_ACTIVE` | Cannot edit a session that's still running |
| `INVALID_DURATION` | Duration is negative, zero, or invalid |
| `WORK_ORDER_NOT_FOUND` | Work order UUID does not exist |
| `WORK_ORDER_ARCHIVED` | Work order is archived and cannot be used |
| `CUSTOMER_NOT_FOUND` | Customer UUID does not exist |
| `ALREADY_ARCHIVED` | Entity is already archived |
| `SESSION_NOT_ORPHAN` | Session is not an orphan (has proper end_time) |
| `INVALID_LIMIT` | Limit parameter is invalid |
| `INVALID_STATUS` | Status is not one of: active, paused, closed |
| `INVALID_CUSTOMER_NAME` | Customer name is empty or invalid |
| `INVALID_WORK_ORDER_NAME` | Work order name is empty or invalid |

---

## Frontend Usage Example

Complete example showing typical app flow:

```typescript
import { invoke } from '@tauri-apps/api/core';

// 1. Create customer
const customer = await invoke<Customer>('create_customer', {
  name: 'Acme Corp',
  code: 'ACME'
});

// 2. Create work order
const workOrder = await invoke<WorkOrder>('create_work_order', {
  customer_id: customer.id,
  name: 'API Development',
  code: 'API-001'
});

// 3. Start tracking
const session = await invoke<Session>('start_session', {
  work_order_id: workOrder.id
});
console.log(`Tracking: ${session.id}`);

// ... work happens ...

// 4. Stop tracking
const completed = await invoke<Session>('stop_session', {
  notes: 'Completed auth endpoints',
  activity_type: 'development'
});

// 5. View daily summary
const summary = await invoke<DailySummary>('get_daily_summary', {
  date: '2026-04-11'
});
console.log(`Today: ${summary.total_seconds / 3600} hours`);

// 6. Export data
const csv = await invoke<string>('export_csv', {
  start_date: '2026-04-01',
  end_date: '2026-04-30'
});
// Save csv to file...
```

---

*Last updated: 2026-04-11*
