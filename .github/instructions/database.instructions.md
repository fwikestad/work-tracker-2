---
name: Database & Data Layer
applyTo: "backend/**/*.{rs,py,sql,ts} backend/db/**"
description: "Use when: designing database schema, writing queries, implementing data access layers, managing migrations, handling data validation. Focus on consistency, performance, and audit trails for personal tracking history."
---

# Database & Data Layer

## Local Persistence Requirement

- Core data must be stored locally on the consultant's computer
- Reads/writes for core workflows must not depend on any cloud database
- The app must start and operate normally without internet connectivity
- Any remote sync/export is optional and must never block local writes

## Design Principles

### Core Entities & Relationships

The system must store:

**Customers**: Represent tracked entities (companies, departments)
- Identity (unique identifier, name)
- Reference code (optional, for internal lookup)
- Contact information and metadata
- Timestamps for audit trail

**Work Orders/Projects**: Represent discrete work units under a customer
- Relationship to parent customer (1:N)
- Identity and code (for reference)
- Status tracking (active, paused, closed)
- Optional: rate/budget information
- Timestamps for audit and reporting

**Time Sessions**: Represent work done
- Relationship to parent work order (1:N)
- Start and end points (for duration calculation)
- Optional: manual duration override
- Metadata: activity type, notes, timestamp of creation
- Timestamps tracking when created/modified

**Current Session State**: Singleton tracking active work
- What work order is currently active (if any)
- Timer state (running, paused)
- Supporting data for quick resume

### Fundamental Constraints

1. **At most one active session per user/worker**
   - Only one work order can be "in progress" at a time
   - Switching to new work implicitly closes previous one

2. **Duration integrity**
   - either calculated from start/end times
   - or manually specified by user
   - System should support both for UX flexibility

3. **No orphaned data on deletion**
   - Deleting a customer must handle all associated work orders and sessions
   - Deleting a work order must handle all associated sessions
   - Consider soft-delete vs hard-delete implications

4. **Audit trail for tracking history**
   - Track when entries were created/modified
   - Support viewing historical state (optional: immutable after certain age)

---

## Data Storage Patterns

### Entity Relationships

Design should support these hierarchies regardless of storage technology:

```
Customer (1)
    ├── WorkOrder (1→N)
    │   ├── TimeSession (1→N)
    │   └── ...more sessions
    └── ...more work orders

ActiveSession (Singleton)
└── Current WorkOrder (0→1)
```

### Querying Requirements

The system must efficiently support:

**By Date/Time**:
- "Show me all work on 2026-04-11"
- "Show me work for week of 2026-04-07"
- Include proper index/query planning

**By Customer/Project**:
- "Show total hours for customer X"
- "Show breakdown by project"
- Support aggregation (SUM, COUNT, GROUP BY equivalent)

**By Status**:
- "Show only active/paused/completed entries"
- Support filtering and sorting

**Search/Lookup**:
- "Find work order by code"
- "Search customers by name"
- Should support partial matches

---

## Consistency & Integrity

### Duration Calculation

**Design choice**: Support two sources of truth for duration:

1. **Calculated**: Derived from end_time - start_time
2. **Manual override**: User specifies duration directly

**Implementation**:
- On session start: record start_time
- On session stop: calculate automatic duration OR allow user override
- Store both timestamps and duration to support both approaches
- Clear indication of which source was used (for auditing)

---

### Session Uniqueness

**Constraint**: No two sessions should have overlapping time ranges for the same worker

- Enforce at application layer (check before insert/update)
- Consider database constraints if supported
- Test edge cases: exact boundaries, timezone handling

---

### Cascade Deletion

**Pattern**: When deleting parent entity, handle children gracefully

- **Soft delete**: Mark as deleted, don't remove data (preserves audit trail)
- **Hard delete**: Remove data immediately
- **Warning before delete**: Inform user of impact (e.g., "Deleting this removes 42 time entries")

Choose based on compliance/audit requirements.

---

## Durability & Crash Recovery

### SQLite Configuration (Required)

```sql
PRAGMA journal_mode=WAL;       -- Write-Ahead Logging for crash safety
PRAGMA synchronous=NORMAL;     -- Minimum for durability (use FULL for maximum safety)
```

### Write Policy

- All INSERT/UPDATE to `time_sessions` table must complete and flush to disk before UI confirms the action
- No batching or delayed writes for session data
- Use synchronous database operations for session mutations

### Startup Recovery Flow

On application startup:
1. Query for open sessions: `SELECT * FROM time_sessions WHERE end_time IS NULL`
2. If results found, present recovery UI before normal app flow
3. Recovery options:
   - **Close now**: Set `end_time = CURRENT_TIMESTAMP`, calculate duration
   - **Discard**: Delete the orphan session
4. Recovery must complete before user can start tracking

---

## Performance Considerations

### Target Metrics
- Single entry create/update: <100ms
- Daily summary query: <100ms
- Weekly report: <500ms
- Search: <50ms
- List rendering: <200ms for 50+ items

### Optimization Strategies

**Indexing**:
- Index on frequently queried columns (date, customer_id, activity type)
- Composite indexes for common filter combinations
- Balance indexing against write performance

**Required Indexes for MVP**:
```sql
CREATE INDEX idx_sessions_start_time ON time_sessions(start_time);
CREATE INDEX idx_sessions_work_order_id ON time_sessions(work_order_id);
CREATE INDEX idx_sessions_end_time ON time_sessions(end_time);  -- for finding open sessions
CREATE INDEX idx_work_orders_customer_id ON work_orders(customer_id);
CREATE INDEX idx_customers_name ON customers(name);  -- for search
```

**Caching**:
- Cache customer/project lists in memory (refresh on add/delete)
- Consider pre-computing daily summaries (optional)
- Don't cache individual entries (must be real-time)

**Query Design**:
- Prefer aggregated queries (GROUP BY, SUM) over row-by-row
- Use pagination for large result sets
- Consider denormalizing for reporting (e.g., cache session totals per project)

---

## Data Validation

### Application-Level Rules

- Date validation: "today or past" for new entries
- Time validation: end_time > start_time
- Customer/project relationship: work order must have valid parent
- Required fields: some fields always needed (others optional)

### Database-Level Rules

- Foreign key constraints (parent must exist)
- Unique constraints (project code unique per customer)
- Check constraints (duration > 0, end_time > start_time)
- Not-null constraints (required fields)

Balance application validation (for UX) with database constraints (for safety).

---

## Migration & Schema Evolution

### Pattern for Safe Changes

1. **Plan**: Document what's changing and why
2. **Create migration**: Version-controlled schema change
3. **Backwards compatibility**: Support old/new formats during transition (if needed)
4. **Test**: Verify with existing data
5. **Deploy**: Run migration
6. **Cleanup**: Remove compatibility code after all clients updated

### Examples of Safe Changes
- Adding optional column
- Adding new index
- Renaming column (through migration script, not direct)
- Archiving old data

### Examples of Risky Changes
- Removing required column (must ensure all data valid first)
- Changing column type (verify all values convertible)
- Removing index (verify it's not critical for queries)

---

## Transaction Design

### Atomic Operations

Use transactions for multi-step operations to ensure consistency:

**Example: Switching work orders**
- Stop current session: set end_time, calculate duration
- Update parent project summary cache
- Create new session
- All must succeed or all must fail

**Pattern**:
```
BEGIN TRANSACTION
  [Step 1]
  [Step 2]
  [Step 3]
COMMIT  // or ROLLBACK if any step fails
```

### Conflict Resolution

- Use optimistic locking where appropriate (version/timestamp)
- Handle "duplicate entry" gracefully (user tries to create same entry twice)
- Resolve local write conflicts deterministically (last-write-wins or version checks)

---

## Audit & Compliance

### Immutability Requirements

Consider: Should old entries be read-only?

- **Option 1**: Lock entries after N days (read-only, can't edit)
- **Option 2**: Support full history (can edit, but keep audit log)
- **Option 3**: Hard delete after retention period

Choose based on compliance needs (billing, tax, industry regulations).

---

## Testing Checklist

- [ ] Schema loads without errors
- [ ] Foreign key relationships enforced
- [ ] Unique constraints prevent duplicates
- [ ] Cascade behavior (delete) works correctly
- [ ] Daily summary query returns correct totals
- [ ] Duration calculation accurate (edge cases: midnight, DST)
- [ ] No orphaned entries after parent delete
- [ ] Queries execute within performance targets
- [ ] Transactions rollback on constraint violation
- [ ] Search works with partial matches
- [ ] Timezone handling correct (if applicable)
- [ ] Data validation catches invalid inputs
- [ ] Indexes created and used in queries
