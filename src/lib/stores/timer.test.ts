import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { ActiveSession } from '$lib/types';

// KNOWN LIMITATION:
// The timer.svelte.ts module uses top-level $effect() which requires a Svelte
// component context. This prevents normal Vitest testing. The tests below are
// SKIPPED until we can either:
// 1. Extract the effect logic to a testable form
// 2. Use @testing-library/svelte to provide component context
// 
// Assigned to: Leia (Frontend) for Phase 2

// Mocks are hoisted before any imports by Vitest — must be declared before the
// imports they affect, but Vitest processes them first regardless.

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('$lib/api/sessions', () => ({
  getActiveSession: vi.fn(),
  pauseSession: vi.fn().mockResolvedValue(undefined),
  resumeSession: vi.fn().mockResolvedValue(undefined),
}));

import { timer } from '$lib/stores/timer.svelte';
import * as sessionsApi from '$lib/api/sessions';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeSession(isPaused: boolean): ActiveSession {
  return {
    sessionId: 'test-session-1',
    workOrderId: 'wo-1',
    workOrderName: 'Test Work Order',
    customerName: 'Test Customer',
    customerColor: '#ff0000',
    startedAt: new Date().toISOString(),
    elapsedSeconds: 10,
    isPaused,
  };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('timer store — pause/resume fix', () => {
  beforeEach(async () => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    timer.setActive(null);
  });

  afterEach(() => {
    vi.clearAllMocks();
    vi.useRealTimers();
  });

  /**
   * TC-TIMER-01
   * After the fix, pause() calls timer.refresh() which re-fetches the active
   * session. The active session must NOT be null and must show isPaused = true.
   * (Before the fix, it called setActive(null) — wiping the session entirely.)
   */
  it('TC-TIMER-01: pause() calls refresh() — active session stays set with isPaused=true', async () => {
    const runningSession = makeSession(false);
    const pausedSession = makeSession(true);

    // Start with a running session
    timer.setActive(runningSession);
    expect(timer.active).not.toBeNull();

    // After pause, getActiveSession will return a paused session
    vi.mocked(sessionsApi.getActiveSession).mockResolvedValue(pausedSession);

    await timer.pause();

    expect(timer.active).not.toBeNull();
    expect(timer.active?.isPaused).toBe(true);
  });

  /**
   * TC-TIMER-02
   * After the fix, resume() calls timer.refresh() which re-fetches the active
   * session. The active session must NOT be null and must show isPaused = false.
   */
  it('TC-TIMER-02: resume() calls refresh() — active session stays set with isPaused=false', async () => {
    const pausedSession = makeSession(true);
    const runningSession = makeSession(false);

    // Start with a paused session
    timer.setActive(pausedSession);
    expect(timer.active?.isPaused).toBe(true);

    // After resume, getActiveSession will return a running session
    vi.mocked(sessionsApi.getActiveSession).mockResolvedValue(runningSession);

    await timer.resume();

    expect(timer.active).not.toBeNull();
    expect(timer.active?.isPaused).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// Phase 2 Timer Store Tests
//
// All tests in this block are SKIPPED pending resolution of the $effect
// context issue. Once Leia (Frontend) provides a testable wrapper or
// @testing-library/svelte integration, uncomment and enable these.
//
// These tests act as the SPECIFICATION for Phase 2 timer behaviour.
// Assigned to: Leia (Frontend) to implement the testable surface.
// ---------------------------------------------------------------------------

describe('timer store — Phase 2 pause/resume state', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    timer.setActive(null);
  });

  afterEach(() => {
    vi.clearAllMocks();
    vi.useRealTimers();
  });

  /**
   * TC-P2-TIMER-01
   * pause() should set isPaused=true AND freeze elapsedSeconds so that the
   * setInterval tick no longer increments the counter.
   */
  it('TC-P2-TIMER-01: pause() sets isPaused=true and freezes elapsedSeconds', async () => {
    const runningSession = makeSession(false);
    const pausedSession = makeSession(true); // isPaused: true

    vi.mocked(sessionsApi.getActiveSession).mockResolvedValue(pausedSession);
    timer.setActive(runningSession);

    const elapsedBefore = timer.elapsed;
    await timer.pause();
    vi.advanceTimersByTime(3000); // 3 ticks

    expect(timer.isPaused).toBe(true);
    // elapsedSeconds must NOT have advanced during the 3 fake ticks
    expect(timer.elapsed).toBe(elapsedBefore);
  });

  /**
   * TC-P2-TIMER-02
   * resume() should set isPaused=false AND restart the tick so that
   * elapsedSeconds begins incrementing again from the frozen value.
   */
  it('TC-P2-TIMER-02: resume() sets isPaused=false and restarts tick', async () => {
    const pausedSession = makeSession(true);
    const runningSession = makeSession(false);

    // Start in paused state
    vi.mocked(sessionsApi.getActiveSession).mockResolvedValue(pausedSession);
    timer.setActive(pausedSession);
    const frozenElapsed = timer.elapsed;

    // Now resume
    vi.mocked(sessionsApi.getActiveSession).mockResolvedValue(runningSession);
    await timer.resume();
    vi.advanceTimersByTime(3000); // 3 ticks

    expect(timer.isPaused).toBe(false);
    // elapsedSeconds should have advanced by ~3 seconds since resume
    expect(timer.elapsed).toBeGreaterThan(frozenElapsed);
    expect(timer.elapsed).toBe(frozenElapsed + 3);
  });

  /**
   * TC-P2-TIMER-03
   * Calling pause() when already paused must be idempotent — the second call
   * should not change any state or throw an error.
   */
  it('TC-P2-TIMER-03: pause() when already paused is idempotent', async () => {
    const pausedSession = makeSession(true);
    vi.mocked(sessionsApi.getActiveSession).mockResolvedValue(pausedSession);
    timer.setActive(pausedSession);

    const elapsedBefore = timer.elapsed;
    await timer.pause(); // first pause — no-op (already paused)
    await timer.pause(); // second pause — still idempotent

    expect(timer.isPaused).toBe(true);
    expect(timer.elapsed).toBe(elapsedBefore);
    // API should have been called twice (refresh each time) but state unchanged
  });

  /**
   * TC-P2-TIMER-04
   * setActive() with a session where isPaused=true should leave isPaused=true
   * immediately (not start the tick). Timer should be frozen from the start.
   */
  it('TC-P2-TIMER-04: setActive() with isPaused=true starts in paused state', () => {
    const pausedSession = makeSession(true);
    timer.setActive(pausedSession);

    expect(timer.active).not.toBeNull();
    expect(timer.isPaused).toBe(true);

    const elapsed = timer.elapsed;
    vi.advanceTimersByTime(5000); // 5 ticks
    // elapsedSeconds must not advance — tick must be stopped
    expect(timer.elapsed).toBe(elapsed);
  });

  /**
   * TC-P2-TIMER-05
   * clear() (or setActive(null)) must reset ALL state:
   *   - active → null
   *   - elapsedSeconds → 0
   *   - isPaused → false
   */
  it('TC-P2-TIMER-05: clear/setActive(null) resets all state including isPaused', () => {
    const pausedSession = makeSession(true);
    timer.setActive(pausedSession);
    expect(timer.active).not.toBeNull();
    expect(timer.isPaused).toBe(true);
    expect(timer.elapsed).toBeGreaterThan(0);

    timer.setActive(null); // or timer.clear() once added

    expect(timer.active).toBeNull();
    expect(timer.isPaused).toBe(false);
    expect(timer.elapsed).toBe(0);
    expect(timer.isTracking).toBe(false);
  });
});
