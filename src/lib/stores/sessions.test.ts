import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { Session } from '$lib/types';

// ---------------------------------------------------------------------------
// Mock all external dependencies before any store imports.
// Vitest hoists vi.mock() calls, so these run before the import statements.
// ---------------------------------------------------------------------------

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

vi.mock('$lib/api/sessions', () => ({
  listSessions: vi.fn().mockResolvedValue([]),
  getActiveSession: vi.fn().mockResolvedValue(null),
  stopSession: vi.fn().mockResolvedValue(null),
  startSession: vi.fn().mockResolvedValue(null),
  deleteSession: vi.fn().mockResolvedValue(undefined),
  updateSession: vi.fn().mockResolvedValue(null),
  quickAdd: vi.fn().mockResolvedValue(null),
  recoverSession: vi.fn().mockResolvedValue(null),
  discardOrphanSession: vi.fn().mockResolvedValue(undefined),
  checkForOrphanSession: vi.fn().mockResolvedValue(null),
}));

vi.mock('$lib/api/reports', () => ({
  getRecentWorkOrders: vi.fn().mockResolvedValue([]),
}));

vi.mock('$lib/api/workOrders', () => ({
  listWorkOrders: vi.fn().mockResolvedValue([]),
}));

import { sessionsStore } from '$lib/stores/sessions.svelte';
import * as sessionsApi from '$lib/api/sessions';

// ---------------------------------------------------------------------------
// Reference dates (all verified against JS Date calculations)
//
// NOTE: The task spec used example dates with an off-by-one error — it listed
// "Monday=2026-04-14" but April 14, 2026 is actually a Tuesday. The dates
// below are the CORRECT values for 2026 where April 15 = Wednesday.
//
// ANCHOR_DATE  : Wednesday, April 15, 2026  (new Date(2026,3,15).getDay() = 3)
// WEEK_0_START : Monday,    April 13, 2026  (ANCHOR_DATE - 2 days)
// WEEK_0_END   : Sunday,    April 19, 2026  (WEEK_0_START + 6 days)
// WEEK_N1_START: Monday,    April  6, 2026  (WEEK_0_START - 7 days)
// WEEK_N1_END  : Sunday,    April 12, 2026  (WEEK_N1_START + 6 days)
// WEEK_N7_START: Monday,    Feb    23, 2026 (WEEK_0_START - 49 days)
// WEEK_N7_END  : Sunday,    March   1, 2026 (WEEK_N7_START + 6 days)
// SUNDAY_TODAY : Sunday,    April 12, 2026  (= WEEK_N1_END)
// ---------------------------------------------------------------------------

// Use local-time constructors (year, month-1, day) so getDay() is always
// correct regardless of the test runner's timezone.
const ANCHOR_DATE    = new Date(2026, 3, 15, 12, 0, 0);   // Wed Apr 15 2026
const SUNDAY_TODAY   = new Date(2026, 3, 12, 12, 0, 0);   // Sun Apr 12 2026
const MONDAY_TODAY   = new Date(2026, 3, 13, 12, 0, 0);   // Mon Apr 13 2026

const WEEK_0_START   = '2026-04-13';  // Monday
const WEEK_0_END     = '2026-04-19';  // Sunday
const WEEK_N1_START  = '2026-04-06';  // Monday
const WEEK_N1_END    = '2026-04-12';  // Sunday
const WEEK_N7_START  = '2026-02-23';  // Monday (7 weeks before Apr 13)
const WEEK_N7_END    = '2026-03-01';  // Sunday

// ---------------------------------------------------------------------------
// Pure week date math helpers
//
// These replicate the logic that sessionsStore will implement internally.
// Defined here so we can test the math independently of the store.
// Tests in "pure calculations" describe block use these directly and PASS NOW.
// Tests in the "store integration" blocks test via sessionsStore (skip until
// Leia's implementation lands).
// ---------------------------------------------------------------------------

function getWeekStart(date: Date): Date {
  const d = new Date(date);
  // getDay(): 0=Sun, 1=Mon, ..., 6=Sat
  // For Mon-start week: go back (day-1) days, except Sunday goes back 6 days
  const day = d.getDay();
  const daysBack = day === 0 ? 6 : day - 1;
  d.setDate(d.getDate() - daysBack);
  d.setHours(0, 0, 0, 0);
  return d;
}

function weekRangeForOffset(offset: number, today: Date): { start: string; end: string } {
  const weekStart = getWeekStart(today);
  weekStart.setDate(weekStart.getDate() + offset * 7);
  const weekEnd = new Date(weekStart);
  weekEnd.setDate(weekEnd.getDate() + 6);
  return { start: toLocalDateStr(weekStart), end: toLocalDateStr(weekEnd) };
}

function toLocalDateStr(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

const MONTH_NAMES = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];

function formatWeekLabel(startStr: string, endStr: string): string {
  // Parse YYYY-MM-DD as local time (avoid UTC shift from new Date(isoString))
  const [sy, sm, sd] = startStr.split('-').map(Number);
  const [ey, em, ed] = endStr.split('-').map(Number);
  const start = new Date(sy, sm - 1, sd);
  const end   = new Date(ey, em - 1, ed);
  return `${MONTH_NAMES[start.getMonth()]} ${start.getDate()} \u2013 ${MONTH_NAMES[end.getMonth()]} ${end.getDate()}, ${end.getFullYear()}`;
}

// ---------------------------------------------------------------------------
// Session factory for grouping tests
// ---------------------------------------------------------------------------

function makeSession(overrides: Partial<Session> = {}): Session {
  return {
    id: Math.random().toString(36).slice(2),
    workOrderId: 'wo-1',
    workOrderName: 'Test Work Order',
    customerName: 'Test Customer',
    customerColor: '#ff0000',
    startTime: '2026-04-15T09:00:00Z',
    endTime: '2026-04-15T10:00:00Z',
    durationSeconds: 3600,
    activityType: null,
    notes: null,
    createdAt: '2026-04-15T09:00:00Z',
    updatedAt: '2026-04-15T10:00:00Z',
    ...overrides,
  };
}

// ===========================================================================
// BLOCK 1: Pure date/week math — no store dependency, passes immediately
// ===========================================================================

describe('week date math — pure calculations', () => {
  /**
   * TC-WK-MATH-01
   * A Wednesday (Apr 15, 2026) with offset=0 should give Mon Apr 13 – Sun Apr 19.
   * Verifies the "days back to Monday" formula and 6-day forward span.
   */
  it('TC-WK-MATH-01: Wednesday offset=0 → Monday of same week is start, Sunday is end', () => {
    const { start, end } = weekRangeForOffset(0, ANCHOR_DATE);
    expect(start).toBe(WEEK_0_START);  // '2026-04-13' (Monday)
    expect(end).toBe(WEEK_0_END);      // '2026-04-19' (Sunday)
  });

  /**
   * TC-WK-MATH-02
   * offset=-1 from Wednesday Apr 15 → previous Mon Apr 6 – Sun Apr 12.
   */
  it('TC-WK-MATH-02: Wednesday offset=-1 → previous Mon–Sun range', () => {
    const { start, end } = weekRangeForOffset(-1, ANCHOR_DATE);
    expect(start).toBe(WEEK_N1_START);  // '2026-04-06'
    expect(end).toBe(WEEK_N1_END);      // '2026-04-12'
  });

  /**
   * TC-WK-MATH-03
   * offset=-7 from Wednesday Apr 15 → 7 weeks ago: Mon Feb 23 – Sun Mar 1.
   * Tests that large negative offsets calculate correctly across month boundaries.
   */
  it('TC-WK-MATH-03: offset=-7 gives correct Mon–Sun range 7 weeks ago', () => {
    const { start, end } = weekRangeForOffset(-7, ANCHOR_DATE);
    expect(start).toBe(WEEK_N7_START);  // '2026-02-23'
    expect(end).toBe(WEEK_N7_END);      // '2026-03-01'
  });

  /**
   * TC-WK-MATH-04
   * Sunday is the LAST day of its Mon–Sun week (not the first day of the next).
   * Sunday Apr 12, 2026 with offset=0 → Mon Apr 6 – Sun Apr 12.
   * This is the critical "Sunday boundary" edge case.
   */
  it('TC-WK-MATH-04: Sunday is end of its week (not start of next week)', () => {
    const { start, end } = weekRangeForOffset(0, SUNDAY_TODAY);
    expect(start).toBe(WEEK_N1_START);  // '2026-04-06' (Monday)
    expect(end).toBe(WEEK_N1_END);      // '2026-04-12' (Sunday = today)
  });

  /**
   * TC-WK-MATH-05
   * Monday is the FIRST day of its Mon–Sun week.
   * Monday Apr 13, 2026 with offset=0 → Mon Apr 13 – Sun Apr 19.
   * Ensures the "days back to Monday" calculation returns 0 for Monday itself.
   */
  it('TC-WK-MATH-05: Monday is start of its own week (0 days back)', () => {
    const { start, end } = weekRangeForOffset(0, MONDAY_TODAY);
    expect(start).toBe(WEEK_0_START);  // '2026-04-13'
    expect(end).toBe(WEEK_0_END);      // '2026-04-19'
  });

  /**
   * TC-WK-MATH-06
   * selectedWeekLabel format for offset=0 from Wednesday Apr 15, 2026.
   * Expected: "Apr 13 – Apr 19, 2026"
   */
  it('TC-WK-MATH-06: week label for offset=0 uses "Mon D – Mon D, YYYY" format', () => {
    const label = formatWeekLabel(WEEK_0_START, WEEK_0_END);
    expect(label).toBe('Apr 13 \u2013 Apr 19, 2026');
  });

  /**
   * TC-WK-MATH-07
   * selectedWeekLabel format for offset=-1 (previous week).
   * Expected: "Apr 6 – Apr 12, 2026"
   */
  it('TC-WK-MATH-07: week label for offset=-1 uses "Mon D – Mon D, YYYY" format', () => {
    const label = formatWeekLabel(WEEK_N1_START, WEEK_N1_END);
    expect(label).toBe('Apr 6 \u2013 Apr 12, 2026');
  });

  /**
   * TC-WK-MATH-08
   * Week spanning month boundary (Feb 23 – Mar 1): label crosses months correctly.
   */
  it('TC-WK-MATH-08: week label spanning month boundary is formatted correctly', () => {
    const label = formatWeekLabel(WEEK_N7_START, WEEK_N7_END);
    expect(label).toBe('Feb 23 \u2013 Mar 1, 2026');
  });
});

// ===========================================================================
// BLOCK 2: sessionsStore — weekOffset state management
//
// These tests require Leia's implementation of weekOffset / setWeekOffset.
// Marked it.skip() so the full test body is preserved as spec documentation.
// Remove .skip when the implementation lands.
// ===========================================================================

describe('sessionsStore — weekOffset state management', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    vi.setSystemTime(ANCHOR_DATE);
  });

  afterEach(() => {
    vi.clearAllMocks();
    vi.useRealTimers();
  });

  /**
   * TC-WK-STORE-01
   * setWeekOffset(0) sets weekOffset to 0 (current week).
   */
  it.skip('TC-WK-STORE-01: setWeekOffset(0) → weekOffset is 0', () => {
    sessionsStore.setWeekOffset(0);
    expect(sessionsStore.weekOffset).toBe(0);
  });

  /**
   * TC-WK-STORE-02
   * setWeekOffset(-1) sets weekOffset to -1 (last week).
   */
  it.skip('TC-WK-STORE-02: setWeekOffset(-1) → weekOffset is -1', () => {
    sessionsStore.setWeekOffset(-1);
    expect(sessionsStore.weekOffset).toBe(-1);
  });

  /**
   * TC-WK-STORE-03
   * setWeekOffset(1) should be capped at 0 — no future weeks allowed.
   * This is the "no future" invariant.
   */
  it.skip('TC-WK-STORE-03: setWeekOffset(1) is capped to 0 (cannot go into future)', () => {
    sessionsStore.setWeekOffset(1);
    expect(sessionsStore.weekOffset).toBe(0);

    sessionsStore.setWeekOffset(100);
    expect(sessionsStore.weekOffset).toBe(0);
  });

  /**
   * TC-WK-STORE-04
   * setWeekOffset(-7) sets weekOffset to -7 (7 weeks ago).
   * Large negative values should be accepted.
   */
  it.skip('TC-WK-STORE-04: setWeekOffset(-7) → weekOffset is -7', () => {
    sessionsStore.setWeekOffset(-7);
    expect(sessionsStore.weekOffset).toBe(-7);
  });
});

// ===========================================================================
// BLOCK 3: sessionsStore — refreshWeek & weekSessions grouping
//
// Requires Leia's implementation of refreshWeek() and weekSessions.
// ===========================================================================

describe('sessionsStore — refreshWeek session grouping', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    vi.setSystemTime(ANCHOR_DATE);
  });

  afterEach(() => {
    vi.clearAllMocks();
    vi.useRealTimers();
  });

  /**
   * TC-WK-GROUP-01
   * Sessions on different days within the same week are grouped by their
   * local date key (YYYY-MM-DD). Each date key maps to only its own sessions.
   */
  it.skip('TC-WK-GROUP-01: sessions on different days are grouped under their date keys', async () => {
    const monday = makeSession({ startTime: '2026-04-13T09:00:00', endTime: '2026-04-13T10:00:00' });
    const wednesday = makeSession({ startTime: '2026-04-15T14:00:00', endTime: '2026-04-15T15:00:00' });
    const friday = makeSession({ startTime: '2026-04-17T11:00:00', endTime: '2026-04-17T12:00:00' });

    vi.mocked(sessionsApi.listSessions).mockResolvedValue([monday, wednesday, friday]);

    await sessionsStore.refreshWeek(0);
    const grouped = sessionsStore.weekSessions;

    expect(grouped['2026-04-13']).toHaveLength(1);
    expect(grouped['2026-04-13'][0].id).toBe(monday.id);
    expect(grouped['2026-04-15']).toHaveLength(1);
    expect(grouped['2026-04-15'][0].id).toBe(wednesday.id);
    expect(grouped['2026-04-17']).toHaveLength(1);
    expect(grouped['2026-04-17'][0].id).toBe(friday.id);
  });

  /**
   * TC-WK-GROUP-02
   * Multiple sessions on the same day are all placed in the same group.
   */
  it.skip('TC-WK-GROUP-02: multiple sessions on same day go into the same group', async () => {
    const session1 = makeSession({ startTime: '2026-04-15T09:00:00', endTime: '2026-04-15T10:00:00' });
    const session2 = makeSession({ startTime: '2026-04-15T11:00:00', endTime: '2026-04-15T12:30:00' });
    const session3 = makeSession({ startTime: '2026-04-15T14:00:00', endTime: '2026-04-15T15:00:00' });

    vi.mocked(sessionsApi.listSessions).mockResolvedValue([session1, session2, session3]);

    await sessionsStore.refreshWeek(0);
    const grouped = sessionsStore.weekSessions;

    expect(grouped['2026-04-15']).toHaveLength(3);
    const ids = grouped['2026-04-15'].map((s: Session) => s.id);
    expect(ids).toContain(session1.id);
    expect(ids).toContain(session2.id);
    expect(ids).toContain(session3.id);
  });

  /**
   * TC-WK-GROUP-03
   * Empty days (no sessions) produce an empty array or are absent from the map.
   * The store must not fabricate sessions for days with none.
   */
  it.skip('TC-WK-GROUP-03: days with no sessions are empty or absent from weekSessions', async () => {
    const wednesday = makeSession({ startTime: '2026-04-15T09:00:00', endTime: '2026-04-15T10:00:00' });
    vi.mocked(sessionsApi.listSessions).mockResolvedValue([wednesday]);

    await sessionsStore.refreshWeek(0);
    const grouped = sessionsStore.weekSessions;

    // Tuesday (Apr 14) had no sessions
    const tuesday = grouped['2026-04-14'];
    expect(tuesday == null || tuesday.length === 0).toBe(true);
  });

  /**
   * TC-WK-GROUP-04
   * refreshWeek(offset) calls listSessions with the correct Mon–Sun date range
   * for that offset. For offset=0 from Apr 15, 2026 → listSessions('2026-04-13', '2026-04-19').
   */
  it.skip('TC-WK-GROUP-04: refreshWeek(0) calls listSessions with correct Mon–Sun range', async () => {
    vi.mocked(sessionsApi.listSessions).mockResolvedValue([]);

    await sessionsStore.refreshWeek(0);

    expect(sessionsApi.listSessions).toHaveBeenCalledWith(WEEK_0_START, WEEK_0_END);
  });

  /**
   * TC-WK-GROUP-05
   * refreshWeek(-1) calls listSessions with the previous week's Mon–Sun range.
   */
  it.skip('TC-WK-GROUP-05: refreshWeek(-1) calls listSessions with previous week range', async () => {
    vi.mocked(sessionsApi.listSessions).mockResolvedValue([]);

    await sessionsStore.refreshWeek(-1);

    expect(sessionsApi.listSessions).toHaveBeenCalledWith(WEEK_N1_START, WEEK_N1_END);
  });
});

// ===========================================================================
// BLOCK 4: sessionsStore — selectedWeekLabel
//
// Requires Leia's implementation of selectedWeekLabel derived value.
// ===========================================================================

describe('sessionsStore — selectedWeekLabel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    vi.setSystemTime(ANCHOR_DATE);
  });

  afterEach(() => {
    vi.clearAllMocks();
    vi.useRealTimers();
  });

  /**
   * TC-WK-LABEL-01
   * selectedWeekLabel for offset=0 from Wednesday Apr 15, 2026.
   * Expected: "Apr 13 – Apr 19, 2026"
   */
  it.skip('TC-WK-LABEL-01: selectedWeekLabel for offset=0 shows current week range', () => {
    sessionsStore.setWeekOffset(0);
    expect(sessionsStore.selectedWeekLabel).toBe('Apr 13 \u2013 Apr 19, 2026');
  });

  /**
   * TC-WK-LABEL-02
   * selectedWeekLabel for offset=-1 from Wednesday Apr 15, 2026.
   * Expected: "Apr 6 – Apr 12, 2026"
   */
  it.skip('TC-WK-LABEL-02: selectedWeekLabel for offset=-1 shows previous week range', () => {
    sessionsStore.setWeekOffset(-1);
    expect(sessionsStore.selectedWeekLabel).toBe('Apr 6 \u2013 Apr 12, 2026');
  });
});
