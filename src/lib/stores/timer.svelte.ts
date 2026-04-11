// Reactive state for the active session and running timer
import type { ActiveSession, OrphanSession } from '$lib/types';
import { getActiveSession } from '$lib/api/sessions';

let activeSession = $state<ActiveSession | null>(null);
let elapsedSeconds = $state(0);
let orphanSession = $state<OrphanSession | null>(null);
let timerInterval: ReturnType<typeof setInterval> | null = null;

export const timer = {
  get active() {
    return activeSession;
  },
  get elapsed() {
    return elapsedSeconds;
  },
  get orphan() {
    return orphanSession;
  },
  get isTracking() {
    return activeSession !== null;
  },

  setActive(session: ActiveSession | null) {
    activeSession = session;
    if (session) {
      elapsedSeconds = session.elapsedSeconds;
      startTick();
    } else {
      stopTick();
      elapsedSeconds = 0;
    }
  },

  setOrphan(session: OrphanSession | null) {
    orphanSession = session;
  },

  async refresh() {
    const session = await getActiveSession();
    timer.setActive(session);
  }
};

function startTick() {
  stopTick();
  timerInterval = setInterval(() => {
    elapsedSeconds += 1;
  }, 1000);
}

function stopTick() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
}
