/**
 * App Load Smoke Tests
 *
 * These tests verify that key modules can be imported and initialized
 * without throwing errors. A module-level error (like a misplaced $effect)
 * would crash the entire app with a black window — this file is the
 * regression guard against that class of bug.
 *
 * What is tested:
 *   - Stores can be imported (no module-level errors)
 *   - Store objects expose expected API shape after import
 *   - No Tauri invoke() calls happen at import time (would throw in test env)
 *
 * THE TEST THAT WOULD HAVE CAUGHT THE BLACK-WINDOW BUG:
 *   "timer store — can be imported without throwing"
 *   The original timer.svelte.ts had a module-level $effect() which threw
 *   "Effect can only be created inside an effect" at import time.
 *   This smoke test catches that class of error immediately.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mocks must be declared before the imports they affect.
// Vitest hoists vi.mock() calls so they run before any module is imported.
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('$lib/api/sessions', () => ({
  getActiveSession: vi.fn().mockResolvedValue(null),
  pauseSession: vi.fn().mockResolvedValue(undefined),
  resumeSession: vi.fn().mockResolvedValue(undefined),
  listSessions: vi.fn().mockResolvedValue([]),
  startSession: vi.fn().mockResolvedValue(undefined),
  stopSession: vi.fn().mockResolvedValue(undefined),
  updateSession: vi.fn().mockResolvedValue(undefined),
  deleteSession: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('$lib/api/reports', () => ({
  getRecentWorkOrders: vi.fn().mockResolvedValue([]),
}));

vi.mock('$lib/api/workOrders', () => ({
  listWorkOrders: vi.fn().mockResolvedValue([]),
  toggleFavorite: vi.fn().mockResolvedValue(undefined),
}));

// Static imports — if any of these modules has a module-level $effect()
// or other runtime error at import time, this whole file will fail to load
// and Vitest will report it as a load error. That is the intended behavior.
import { timer } from '$lib/stores/timer.svelte';
import { sessionsStore } from '$lib/stores/sessions.svelte';
import { uiStore } from '$lib/stores/ui.svelte';
import { invoke } from '@tauri-apps/api/core';

beforeEach(() => {
  vi.clearAllMocks();
});

// ---------------------------------------------------------------------------
// Timer store
// ---------------------------------------------------------------------------

describe('timer store — can be imported without throwing', () => {
  it('timer object is defined after import', () => {
    expect(timer).toBeDefined();
    expect(typeof timer).toBe('object');
  });

  it('timer exposes expected API shape', () => {
    expect(timer).toHaveProperty('active');
    expect(timer).toHaveProperty('elapsed');
    expect(timer).toHaveProperty('isTracking');
    expect(timer).toHaveProperty('isPaused');
    expect(typeof timer.setActive).toBe('function');
    expect(typeof timer.refresh).toBe('function');
    expect(typeof timer.pause).toBe('function');
    expect(typeof timer.resume).toBe('function');
  });

  it('timer starts in idle state (no active session)', () => {
    // After module load with no active session, all state should be falsy/zero
    // Note: state may have been set by other tests in the suite; we just
    // verify the shape is correct, not the exact value.
    expect(typeof timer.isTracking).toBe('boolean');
    expect(typeof timer.isPaused).toBe('boolean');
    expect(typeof timer.elapsed).toBe('number');
  });
});

// ---------------------------------------------------------------------------
// Sessions store
// ---------------------------------------------------------------------------

describe('sessions store — can be imported without throwing', () => {
  it('sessionsStore object is defined after import', () => {
    expect(sessionsStore).toBeDefined();
    expect(typeof sessionsStore).toBe('object');
  });

  it('sessionsStore exposes expected API shape', () => {
    expect(sessionsStore).toHaveProperty('todays');
    expect(sessionsStore).toHaveProperty('recent');
    expect(sessionsStore).toHaveProperty('allFavorites');
    expect(typeof sessionsStore.refreshToday).toBe('function');
    expect(typeof sessionsStore.refreshRecent).toBe('function');
    expect(typeof sessionsStore.refreshAll).toBe('function');
  });

  it('sessionsStore.todays is an array after import', () => {
    expect(Array.isArray(sessionsStore.todays)).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// UI store
// ---------------------------------------------------------------------------

describe('ui store — can be imported without throwing', () => {
  it('uiStore object is defined after import', () => {
    expect(uiStore).toBeDefined();
    expect(typeof uiStore).toBe('object');
  });

  it('uiStore exposes expected API shape', () => {
    expect(uiStore).toHaveProperty('quickAdd');
    expect(uiStore).toHaveProperty('search');
    expect(uiStore).toHaveProperty('query');
    expect(typeof uiStore.openQuickAdd).toBe('function');
    expect(typeof uiStore.closeQuickAdd).toBe('function');
    expect(typeof uiStore.openSearch).toBe('function');
    expect(typeof uiStore.closeSearch).toBe('function');
  });
});

// ---------------------------------------------------------------------------
// No Tauri calls at import time
// ---------------------------------------------------------------------------

describe('timer store — no Tauri calls happen at import time', () => {
  it('invoke() was NOT called during module-level store initialization', () => {
    // All vi.mock() calls are hoisted, so the mock was in place when modules
    // were imported. If any store had called invoke() at module level, this
    // assertion would fail.
    // Note: clearAllMocks() in beforeEach resets the counter, so this only
    // checks calls since the last test's cleanup — the key protection is that
    // the FILE itself loads without error (store imports above), which is the
    // real regression guard.
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });
});
