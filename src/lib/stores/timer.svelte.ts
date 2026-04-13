// Reactive state for the active session and running timer
import type { ActiveSession, OrphanSession } from '$lib/types';
import { getActiveSession, pauseSession as apiPauseSession, resumeSession as apiResumeSession } from '$lib/api/sessions';
import { invoke } from '@tauri-apps/api/core';

let activeSession = $state<ActiveSession | null>(null);
let elapsedSeconds = $state(0);
let orphanSession = $state<OrphanSession | null>(null);
let timerInterval: ReturnType<typeof setInterval> | null = null;
let heartbeatInterval: ReturnType<typeof setInterval> | null = null;

const isPaused = $derived(activeSession?.isPaused ?? false);

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
  get isPaused() {
    return isPaused;
  },

  setActive(session: ActiveSession | null) {
    activeSession = session;
    if (session) {
      elapsedSeconds = session.elapsedSeconds;
      if (!session.isPaused) {
        startTick();
      } else {
        stopTick();
      }
      startHeartbeat();
    } else {
      stopTick();
      stopHeartbeat();
      elapsedSeconds = 0;
    }
    updateTrayState();
  },

  setOrphan(session: OrphanSession | null) {
    orphanSession = session;
  },

  async refresh() {
    const session = await getActiveSession();
    timer.setActive(session);
  },

  async pause() {
    try {
      await apiPauseSession();
      stopTick();
      await timer.refresh();
    } catch (e: any) {
      alert(e?.message ?? 'Failed to pause');
    }
  },

  async resume() {
    try {
      await apiResumeSession();
      await timer.refresh();
      startTick();
    } catch (e: any) {
      alert(e?.message ?? 'Failed to resume');
    }
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

function stopHeartbeat() {
  if (heartbeatInterval) {
    clearInterval(heartbeatInterval);
    heartbeatInterval = null;
  }
}

function updateTrayState() {
  invoke('update_tray_state', {
    workOrderName: activeSession?.workOrderName ?? null,
    isPaused: activeSession?.isPaused ?? false,
  }).catch((e) => {
    console.error('Failed to update tray state:', e);
  });
}
