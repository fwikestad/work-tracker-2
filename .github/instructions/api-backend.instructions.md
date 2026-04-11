---
name: Backend API & Services
applyTo: "backend/**/*.{rs,py,ts} backend/src/**"
description: "Use when: designing REST endpoints, implementing business logic, building summaries, handling background tasks. Focus on stateless design, error handling, and audit trails."
---

# Backend API & Services

## API Design Principles

### 0. Local-Only Runtime
- The backend runs on the same local machine as the app
- No required outbound network calls for core features
- Local IPC/loopback HTTP/in-process calls are all acceptable
- Core operations must continue to work when offline

### 1. Stateless Operations
- No session state on server; all state in persistent storage or request context
- Each request is independent; could be routed to different process
- Frontend manages UI state; backend manages persistence only

### 2. Clear Contracts
- Structured request/response formats
- Consistent error handling across all operations
- Version API for forward compatibility

### 3. Error Handling
**Standard Error Response Pattern**:
- Consistent structure (code, message, details)
- Actionable information for client
- Appropriate HTTP status codes

---

## Core Workflows & Operations

### Customer Management

**Retrieve customers**:
- All customers or filtered subset
- Support filtering/searching
- Return paginated results if applicable

**Create customer**:
- Require: name
- Optional: code, contact info, metadata
- Return created entity with ID

**Update customer**:
- Allow partial updates
- Validate relationships (can't delete if active work exists)
- Optional: warn before risky operations

**Delete customer**:
- Handle cascading (delete associated work orders and sessions)
- Option for soft-delete (mark inactive) vs hard-delete
- Warn user of impact before deleting

---

### Work Order Management

**Retrieve work orders**:
- All or filtered (by customer, status)
- Include aggregate data (hours spent, budget remaining)
- Support sorting and pagination

**Create work order**:
- Require: customer (must exist), code, title
- Optional: rate, budget, due date
- Ensure code is unique within customer

**Update work order**:
- Allow status transitions (active → paused → closed)
- Allow updating rates/budgets
- Validate no invalid state transitions

**Delete work order**:
- Cascade to time sessions
- Warn of impact
- Consider soft-delete for audit trail

---

### Time Entry/Session Management

**Start work session**:
- Identify work order to track
- If existing session active, stop it first (atomic)
- Create new incomplete session
- Return session info immediately

**Stop work session**:
- Set end time
- Calculate duration (auto or user-provided)
- Add optional metadata (notes, activity type)
- Persist immediately

**List entries**:
- Filter by date range, customer, work order, activity type
- Sort by time, customer, or other criteria
- Return with aggregate summary (totals, durations)

**Update entry**:
- Allow editing: duration, notes, activity type, tags
- Validate: end_time is consistent, no overlaps
- Prevent edits to locked entries (if applicable)

**Delete entry**:
- Remove from history
- Update aggregate caches as needed
- Option: soft-delete for audit trail

---

### Quick-Add: Combined Create and Start

**`createAndStart` operation** (Phase 1 required):
- Creates customer (if new, matches existing if name exists) + work order + immediately starts a session
- All steps in one atomic transaction
- Parameters:
  - `customer_name` (required): Creates new customer if not exists, matches existing if name found
  - `work_order_name` (required): Name for the new work order
  - Optional: `activity_type`, `notes`
- Returns: The new active session with full context (customer ID, work order ID, session ID, start_time)
- On success: Previous active session (if any) is stopped automatically
- Use case: Quick-add from UI without navigating to management screens

---

## Business Logic Patterns

### No Overlapping Sessions

**Constraint**: Only one incomplete/active session at a time

**Implementation**:
- Check for existing active session before creating new one
- Atomic transaction: stop old + create new (essential for switching)
- Prevent race conditions (use locks if needed)

---

### Duration Calculation

**Two supported approaches**:

1. **Automatic**: duration = end_time - start_time
2. **Manual override**: user specifies in form

**Implementation**:
- Show calculated duration as suggestion
- Allow user to override
- Track source of truth (for auditing)
- Support both in storage/query

---

### Time Summary Calculations

**Required**:
- Aggregate tracked hours by customer/project for date range
- Support activity-based breakdowns
- Exclude incomplete entries from finalized summaries

**Pattern**:
```
For date range [start, end]:
  Query all sessions where:
    date >= start AND date <= end
    end_time is NOT null (completed)
  
  Group by customer/work_order
  SUM(duration_minutes) / 60 = tracked_hours
  
  Return aggregated report
```

---

## Data Access Patterns

### Query Optimization

**Daily Summary**: Retrieve today's work grouped by customer/project
- Needs: Date filtering, customer grouping, SUM/COUNT aggregation
- Performance: Must be <100ms

**Weekly Report**: Retrieve week's work for personal summary
- Needs: Date range, activity filtering, grouping
- Performance: Must be <500ms

**Search**: Find customers or work orders
- Needs: Partial text matching
- Performance: Must be <50ms

**List**: Retrieve all xxx (customers/projects/entries)
- Needs: Pagination, optional filtering, sorting
- Performance: First page <100ms

---

## Service Layer Responsibilities

- **Business logic**: Rules for starting/stopping sessions, switching projects
- **Validation**: Enforce data constraints (no overlaps, duration validity)
- **Calculations**: Tracked hours and summaries
- **Querying**: Provide domain-specific queries (daily totals, reports)
- **Persistence**: Coordinate with data layer, handle transactions
- **Error handling**: Catch database/network errors, return structured responses

---

## Communication Patterns

### Request/Response Structure

**Success Response**:
```
{
  "data": { /* entity or entities */ },
  "metadata": {
    "timestamp": "ISO8601",
    "request_id": "unique_id"
  }
}
```

**Error Response**:
```
{
  "error": {
    "code": "ERROR_CODE",
    "message": "User-friendly message",
    "details": {
      "field": "optional_details",
      "reason": "optional_reason"
    }
  }
}
```

---

### Protocol Choices

Agents can choose:
- **REST**: HTTP verbs (GET, POST, PATCH, DELETE)
- **GraphQL**: Query/mutation language
- **RPC**: Function call style (local or network)
- **Local IPC**: In-process or socket communication

For this project, default to local communication between UI and backend. Remote endpoints are optional extensions only.

Requirements remain the same regardless of protocol:
- Structured contracts
- Error handling
- Transaction support
- Queryability

---

## Session/State Management

### Current Session Tracking

**Singleton**: Track what user is actively working on

- Current work order ID (or null)
- Timer state (running, paused, stopped)
- Elapsed time (calculated or stored)
- Last update timestamp

**Usage**: Quick resume, active indicator, prevent loss of work

**Consistency**: Update atomically with session create/stop

---

### Session Switching Logic

**Atomic operation**:
1. Query current incomplete session (if any)
2. If found: set end_time, calculate duration
3. Update session state to null (no active session)
4. Create new incomplete session
5. Update session state to point to new session
6. All 5 steps in single transaction

**Why atomic**: Ensures consultant never loses active session, no race conditions

---

## Testing Checklist

- [ ] All operations handle missing/invalid input gracefully
- [ ] No overlapping sessions can exist
- [ ] Switching projects stops old project automatically
- [ ] Duration calculated correctly (end_time - start_time)
- [ ] Manual duration override takes precedence
- [ ] Time summaries match manual verification
- [ ] Daily summary totals are accurate
- [ ] Date filtering works (inclusive ranges)
- [ ] Activity/tag filtering works
- [ ] Queries execute within performance targets (<100ms, <500ms, <50ms)
- [ ] Transactions rollback on error (no partial updates)
- [ ] Cascade deletes remove all related data
- [ ] Search supports partial matches
- [ ] Error responses have consistent structure
- [ ] No data loss on network failure (writes persisted before response)
