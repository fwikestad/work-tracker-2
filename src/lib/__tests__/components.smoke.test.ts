/**
 * Component Mount Smoke Tests
 *
 * These tests render key UI components and verify they mount without throwing.
 * They catch runtime errors in $effect, onMount, or template logic that
 * would not be caught by store-level unit tests alone.
 *
 * All stores and Tauri APIs are mocked with inert values so components
 * render their idle/empty states without side effects.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';

// ---------------------------------------------------------------------------
// Mocks — must be declared before component imports.
// Vitest hoists vi.mock() so these are in place before any import runs.
// ---------------------------------------------------------------------------

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

vi.mock('$lib/stores/timer.svelte', () => ({
  timer: {
    active: null,
    elapsed: 0,
    isTracking: false,
    isPaused: false,
    orphan: null,
    setActive: vi.fn(),
    setOrphan: vi.fn(),
    refresh: vi.fn(),
    pause: vi.fn(),
    resume: vi.fn(),
  },
}));

vi.mock('$lib/stores/sessions.svelte', () => ({
  sessionsStore: {
    todays: [],
    recent: [],
    allFavorites: [],
    weekOffset: 0,
    weekSessions: [],
    selectedWeekLabel: 'Apr 14 – Apr 20, 2026',
    setWeekOffset: vi.fn().mockResolvedValue(undefined),
    refreshToday: vi.fn().mockResolvedValue(undefined),
    refreshWeek: vi.fn().mockResolvedValue(undefined),
    refreshRecent: vi.fn().mockResolvedValue(undefined),
    refreshAll: vi.fn().mockResolvedValue(undefined),
  },
}));

vi.mock('$lib/stores/ui.svelte', () => ({
  uiStore: {
    quickAdd: false,
    search: false,
    query: '',
    openQuickAdd: vi.fn(),
    closeQuickAdd: vi.fn(),
    openSearch: vi.fn(),
    closeSearch: vi.fn(),
    setQuery: vi.fn(),
  },
}));

vi.mock('$lib/api/sessions', () => ({
  getActiveSession: vi.fn().mockResolvedValue(null),
  startSession: vi.fn().mockResolvedValue(undefined),
  stopSession: vi.fn().mockResolvedValue(undefined),
  pauseSession: vi.fn().mockResolvedValue(undefined),
  resumeSession: vi.fn().mockResolvedValue(undefined),
  updateSession: vi.fn().mockResolvedValue(undefined),
  deleteSession: vi.fn().mockResolvedValue(undefined),
  listSessions: vi.fn().mockResolvedValue([]),
}));

vi.mock('$lib/api/workOrders', () => ({
  listWorkOrders: vi.fn().mockResolvedValue([]),
  toggleFavorite: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('$lib/api/reports', () => ({
  getRecentWorkOrders: vi.fn().mockResolvedValue([]),
}));

vi.mock('$lib/stores/widget.svelte', () => ({
  widgetStore: {
    isWidgetMode: false,
    setWidgetMode: vi.fn(),
  },
}));

vi.mock('$lib/api/window', () => ({
  toggleWidgetMode: vi.fn().mockResolvedValue(false),
}));

// Stub browser APIs that components may call on error paths
vi.stubGlobal('alert', vi.fn());

// Component imports (after all mocks)
import Timer from '$lib/components/Timer.svelte';
import SearchSwitch from '$lib/components/SearchSwitch.svelte';
import SessionList from '$lib/components/SessionList.svelte';
import WidgetOverlay from '$lib/components/WidgetOverlay.svelte';

beforeEach(() => {
  vi.clearAllMocks();
  cleanup();
});

// ---------------------------------------------------------------------------
// Timer component
// ---------------------------------------------------------------------------

describe('Timer component — mount smoke tests', () => {
  it('mounts without throwing', () => {
    expect(() => render(Timer)).not.toThrow();
  });

  it('renders "Not tracking" state when no active session', () => {
    render(Timer);
    expect(screen.getByText('Not tracking')).toBeTruthy();
  });

  it('renders hint text in idle state', () => {
    render(Timer);
    expect(screen.getByText(/Ctrl\+N|Ctrl\+K/)).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// SearchSwitch component
// ---------------------------------------------------------------------------

describe('SearchSwitch component — mount smoke tests', () => {
  it('mounts without throwing', () => {
    expect(() => render(SearchSwitch)).not.toThrow();
  });

  it('renders search input', () => {
    render(SearchSwitch);
    const input = screen.getByRole('textbox');
    expect(input).toBeTruthy();
  });

  it('renders empty state message when no work orders', () => {
    render(SearchSwitch);
    expect(screen.getByText('No work orders yet')).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// SessionList component
// ---------------------------------------------------------------------------

describe('SessionList component — mount smoke tests', () => {
  it('mounts without throwing', () => {
    expect(() => render(SessionList)).not.toThrow();
  });

  it('renders week navigation', () => {
    render(SessionList);
    expect(screen.getByLabelText('Previous week')).toBeTruthy();
    expect(screen.getByLabelText('Next week')).toBeTruthy();
  });

  it('renders empty state when no sessions this week', () => {
    render(SessionList);
    expect(screen.getByText('No sessions this week')).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// WidgetOverlay component
// ---------------------------------------------------------------------------

describe('WidgetOverlay component — mount smoke tests', () => {
  it('mounts without throwing', () => {
    expect(() => render(WidgetOverlay)).not.toThrow();
  });

  it('renders "Not tracking" state when no active session', () => {
    render(WidgetOverlay);
    expect(screen.getByText('Not tracking')).toBeTruthy();
  });

  it('renders Stopped badge when not tracking', () => {
    render(WidgetOverlay);
    expect(screen.getByText(/Stopped/i)).toBeTruthy();
  });

  it('renders exit button', () => {
    render(WidgetOverlay);
    expect(screen.getByTitle(/Exit widget mode/i)).toBeTruthy();
  });
});
