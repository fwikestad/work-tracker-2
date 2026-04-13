# Phase 4 Implementation Plan: ServiceNow Integration & Team Features

**Status**: Phase 4a in progress 🚧

This document outlines Phase 4 work across two sub-phases: ServiceNow integration (4a, now implementing) and future team/multi-user features (4b+).

---

## Goals

### Phase 4a: ServiceNow CSV Export
Deliver real value to consultants using ServiceNow by enabling time entry export in a ServiceNow-compatible format, with minimal complexity. Users can manually upload the CSV file into ServiceNow's native Import Set mechanism.

**Why CSV first**: Zero network dependency, works with any ServiceNow instance, validates user demand before investing in REST API plumbing.

### Phase 4b+: Team & Integrations (Future)
Enable multi-user tracking on shared machines and third-party integrations while maintaining local-first architecture.

---

## Phase 4a — ServiceNow Import Set CSV Export

### Background

See [`.squad/decisions/inbox/han-servicenow-exploration.md`](../.squad/decisions/inbox/han-servicenow-exploration.md) for full analysis. Key recommendation: **CSV export now, REST API later**.

### Scope (Phase 4a MVP)

| Feature | Status | Details |
|---------|--------|---------|
| CSV format toggle | 🚧 New | Reports UI: "Standard CSV" or "ServiceNow Import Set" dropdown |
| ServiceNow columns | 🚧 New | `date`, `customer_name`, `work_order_name`, `work_order_code`, `start_datetime`, `end_datetime`, `duration_hours` (float), `activity_type`, `notes` |
| Duration in hours | 🚧 New | Instead of minutes; 2 decimal places (e.g., 1.75 hours) |
| Manual upload workflow | 📋 Doc only | ServiceNow admin guide: how to set up Import Set transform to accept these columns |

### Technical Changes

**Backend (Rust)**:
- Extend existing `export_csv()` in `summary_service.rs` with format parameter
- Add ServiceNow-specific column mapping and duration conversion
- Reuse existing queries; no new database logic needed

**Frontend (Svelte)**:
- Add format selector to Reports export dialog (dropdown or radio buttons)
- Pass format selection to export command
- Auto-download as `work-tracker-export-servicenow.csv` when ServiceNow format selected

**Database**: No schema changes required

### Implementation Checklist

- [ ] **Backend**: Add `export_csv_servicenow()` or extend `export_csv(format)` with ServiceNow format option (~2-3 hours)
- [ ] **Frontend**: Add format selector to Reports export UI (~3 hours)
- [ ] **Testing**: Verify CSV output against ServiceNow Import Set schema (~1-2 hours)
- [ ] **Documentation**: Write ServiceNow admin import guide (in `docs/`) (~1-2 hours)
- [ ] **Validation**: Test with sample ServiceNow Developer Instance (~2 hours)

**Estimated total**: 9-11 hours (~1.5 days)

### Success Criteria

- ✅ CSV exports correctly in both Standard and ServiceNow formats
- ✅ ServiceNow format is valid for Import Set upload (no transform errors)
- ✅ Columns match agreed schema (confirmed with ServiceNow admin, if applicable)
- ✅ Duration calculation verified (COALESCE(override, calculated) ÷ 3600)
- ✅ All export tests passing

---

## Phase 4b — ServiceNow REST API Push (Parked)

### Background

Full REST integration adds complexity: OAuth 2.0, credential storage, network dependency, error handling. Defer until we validate Phase 4a usage.

### Scope (When Approved)

| Feature | Est. Hours | Notes |
|---------|-----------|-------|
| Settings UI: ServiceNow credentials | 6-8 | Instance URL, auth type (Basic or OAuth 2.0) |
| Credential storage (OS keychain) | 4-5 | Never plaintext in SQLite |
| `push_to_servicenow` Tauri command | 8-10 | Batch push via Import Sets API |
| Error handling + retry logic | 4-5 | Graceful failure, clear feedback |
| OAuth 2.0 client credentials flow | 8-10 | If required by target instance |

**Estimated total (without OAuth)**: 22-28 hours (~3-4 days)  
**Estimated total (with OAuth)**: 30-38 hours (~4-5 days)

### Architecture Decisions (For Phase 4b)

1. **Use Import Sets API** (not direct Table API) — Admin owns the transform; WT2 doesn't need to know internal sys_ids
2. **Batch push**: ~50 records per request
3. **Dry-run**: Optional "test connection" button in settings to validate credentials before first push
4. **Local retry**: Queue failed pushes in SQLite for retry (don't lose data if network fails)
5. **Strict opt-in**: Disabled by default; enable in Settings only after user confirms credentials

### Why Not Yet

- **Validation needed**: Does anyone actually use Phase 4a CSV export? If adoption is low, 4b is wasted effort.
- **Complexity risk**: OAuth 2.0 adds ~5 days. Better to ship CSV, validate demand, then decide on API.
- **Credential safety**: Requires OS keychain integration (new dependency). Phase 4a has zero risk.

---

## Phase 4b+ — Team & Integrations

### Multi-User Tracking (Future)

| Feature | Status | Notes |
|---------|--------|-------|
| User profiles | 📋 Planned | Each user has separate `work_tracker.db` in their own home directory |
| User switching | 📋 Planned | Profile menu in tray or main window (no logout) |
| Shared work order library | 📋 Optional | Team can publish template projects (Phase 4c+) |

**Why deferred**: Single-user works well; multi-user adds UI complexity and testing scope.

### Other Integrations (Future)

- 📋 **Billing tool export** — CSV for accounting software (Freshbooks, Wave, etc.)
- 📋 **Calendar integration** — Export sessions as calendar events (ICS format)
- 📋 **Slack notifications** — Optional alerts on project switches
- 📋 **Zapier/IFTTT** — Webhooks for automation (requires server; local-first limit)

---

## Architecture Principles (Phase 4+)

### Local-First Constraint

ServiceNow integration is inherently network-dependent, but **must remain optional**:
- Feature is disabled by default
- Core tracking works identically with ServiceNow disabled or unreachable
- Failure to push is a visible warning, never silent data loss
- All writes are local-first; push is non-blocking sync

### Credential Security

- Never store passwords, API keys, or tokens in plaintext SQLite
- Use OS keychain (Windows Credential Manager, macOS Keychain, Linux Secret Service)
- OAuth 2.0 tokens refresh automatically; old tokens discarded after use

---

## Testing Strategy

### Phase 4a (CSV)

- [ ] Unit tests: CSV format generation with sample data
- [ ] Validation: Output matches ServiceNow Import Set schema
- [ ] Edge cases: Empty notes, special characters in customer names, midnight timestamps
- [ ] Integration: Export command called from Reports UI returns correct format

### Phase 4b (REST API) — When Ready

- [ ] Mock ServiceNow API responses for testing (no real instance required)
- [ ] Test auth flows: Basic Auth, OAuth 2.0 token refresh
- [ ] Test error scenarios: invalid credentials, network timeout, instance not found
- [ ] Test credential storage: secrets never logged or leaked in error messages

---

## Timeline

### Phase 4a (Next Sprint)

- Sprint start: Engineer 4a work items
- Target completion: 1-2 weeks (depending on ServiceNow admin coordination)
- Release: Include in next major release or point release (TBD)

### Phase 4b (Parked)

- Decision point: After Phase 4a ships, review adoption metrics
- If >30% of users export to ServiceNow: Start Phase 4b planning
- If <10% adoption: Phase 4b may be indefinitely parked
- Decision made by Fredrik + team (Q2/Q3 2026)

---

## Success Metrics

### Phase 4a
- ✅ CSV export format matches ServiceNow admin requirements (zero rejections)
- ✅ Adoption: >20% of users try ServiceNow export within first month

### Phase 4b (If Approved)
- ✅ One-click push without manual CSV upload
- ✅ Zero credential leaks or plaintext storage
- ✅ Graceful error handling (push failure doesn't break local tracking)

---

## Decision Log

| Date | Decision | Owner | Status |
|------|----------|-------|--------|
| 2025-01-29 | CSV-first approach approved (vs. REST API now) | Han | ✅ Approved |
| TBD | Phase 4b approval + scope confirmation | Fredrik | ⏳ Pending |
| TBD | ServiceNow admin coordination (field names) | Fredrik | ⏳ Pending |

---

## Next Steps

1. **Immediate (Phase 4a MVP)**:
   - Chewie: Implement ServiceNow CSV export format in backend
   - Leia: Add format selector to Reports export UI
   - Wedge: Write CSV validation tests
   - Mon Mothma: Document ServiceNow admin import guide

2. **Post-MVP**:
   - Fredrik: Confirm target ServiceNow instance field names (if applicable)
   - Team: Gather Phase 4a adoption metrics after 1 month
   - Han: Review Phase 4b feasibility based on demand

3. **Phase 4b (If Approved)**:
   - Han: Document OAuth 2.0 / credential storage architecture
   - Chewie: Implement REST push + keychain integration
   - Leia: Build Settings UI for ServiceNow credentials
   - Wedge: Integration tests (mock API)

---

## References

- **Phase 4a Background**: [`.squad/decisions/inbox/han-servicenow-exploration.md`](../.squad/decisions/inbox/han-servicenow-exploration.md)
- **Architecture**: [`docs/architecture.md`](architecture.md)
- **API Reference**: [`docs/api-reference.md`](api-reference.md)
- **Feature Catalog**: [`docs/features.md`](features.md)
