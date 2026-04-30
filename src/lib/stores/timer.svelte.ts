/**
 * Reactive store for managing the active time tracking session and timer state.
 *
 * This store manages the currently active work session, timer updates, and heartbeat
 * communication with the Rust backend. It maintains a single source of truth for
 * what the user is currently tracking.
 *
 * @module timer
 */
import type { ActiveSession, OrphanSession } from '$lib/types';
import { getActiveSession } from '$lib/api/sessions';
import { invoke } from '@tauri-apps/api/core';

/** The currently active tracking session, or null if no session is running. */
let activeSession = $state<ActiveSession | null>(null);

/** Elapsed time in seconds for the active session. */
let elapsedSeconds = $state(0);

/** Orphaned session detected on startup that needs user recovery action. */
let orphanSession = $state<OrphanSession | null>(null);

/** Interval handle for the 1-second timer tick. */
let timerInterval: ReturnType<typeof setInterval> | null = null;

/** Interval handle for the 30-second heartbeat to the backend. */
let heartbeatInterval: ReturnType<typeof setInterval> | null = null;

/**
 * Timer store for managing active time tracking sessions.
 *
 * Provides reactive access to the active session, elapsed time, and pause state.
 * Handles timer tick updates, backend heartbeats, and tray state synchronization.
 */
export const timer = {
  /**
   * The currently active tracking session.
   *
   * @returns The active session with work order details and elapsed time, or null if no session is active.
   */
  get active() {
    return activeSession;
  },

  /**
   * Total elapsed seconds for the current session.
   *
   * This value increments every second when a session is running and not paused.
   *
   * @returns Elapsed seconds as an integer.
   */
  get elapsed() {
    return elapsedSeconds;
  },

  /**
   * Orphaned session discovered on app startup that needs recovery.
   *
   * An orphan session is one that was left running when the app crashed or closed unexpectedly.
   * The user must choose to recover (close it) or discard it before normal tracking can resume.
   *
   * @returns The orphan session details, or null if no orphan exists.
   */
  get orphan() {
    return orphanSession;
  },

  /**
   * Whether any session is currently being tracked.
   *
   * @returns True if a session exists, false otherwise.
   */
  get isTracking() {
    return activeSession !== null;
  },

  /**
   * Sets the active session and updates timer/heartbeat state accordingly.
   *
   * When a session is provided, starts the timer tick and heartbeat.
   * When null, stops all timers and resets elapsed time.
   *
   * @param session - The new active session, or null to clear the active session.
   */
  setActive(session: ActiveSession | null) {
    activeSession = session;
    if (session) {
      elapsedSeconds = session.elapsedSeconds;
      startTick();
      startHeartbeat();
    } else {
      stopTick();
      stopHeartbeat();
      elapsedSeconds = 0;
    }
    updateTrayState();
  },

  /**
   * Sets the orphan session that needs user recovery action.
   *
   * @param session - The orphaned session details, or null to clear.
   */
  setOrphan(session: OrphanSession | null) {
    orphanSession = session;
  },

  /**
   * Refreshes the active session from the backend.
   *
   * Fetches the current active session state and updates the timer accordingly.
   * Use this after operations that modify the session (start, stop).
   */
  async refresh() {
    const session = await getActiveSession();
    timer.setActive(session);
  }
};

/**
 * Starts the 1-second timer tick that increments elapsed seconds.
 * Stops any existing tick interval before starting a new one.
 */
function startTick() {
  stopTick();
  timerInterval = setInterval(() => {
    elapsedSeconds += 1;
  }, 1000);
}

/**
 * Stops the timer tick interval.
 */
function stopTick() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
}

/**
 * Starts the 30-second heartbeat that notifies the backend of continued activity.
 * This allows the backend to detect unexpected app termination.
 * Stops any existing heartbeat before starting a new one.
 */
function startHeartbeat() {
  stopHeartbeat();
  heartbeatInterval = setInterval(() => {
    if (activeSession) {
      invoke('update_heartbeat').catch((e) => {
        console.error('Heartbeat failed:', e);
      });
    }
  }, 30000); // 30 seconds
}

/**
 * Stops the heartbeat interval.
 */
function stopHeartbeat() {
  if (heartbeatInterval) {
    clearInterval(heartbeatInterval);
    heartbeatInterval = null;
  }
}

/**
 * Updates the system tray icon and tooltip to reflect current tracking state.
 * Sends work order name to the Rust backend.
 */
function updateTrayState() {
  invoke('update_tray_state', {
    workOrderName: activeSession?.workOrderName ?? null,
  }).catch((e) => {
    console.error('Failed to update tray state:', e);
  });
}
