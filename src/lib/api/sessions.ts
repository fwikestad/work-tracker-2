import { invoke } from '@tauri-apps/api/core';
import type {
  Session,
  ActiveSession,
  OrphanSession,
  QuickAddParams,
  QuickAddResult,
  UpdateSessionParams,
} from '../types';

/** Starts tracking time on a specific work order. Stops any active session first. */
export const startSession = (workOrderId: string) =>
  invoke<Session>('start_session', { workOrderId });

/** Stops the currently active session and saves it with optional notes and activity type. */
export const stopSession = (notes?: string, activityType?: string) =>
  invoke<Session | null>('stop_session', { notes, activityType });

/** Fetches the currently active tracking session, if any. */
export const getActiveSession = () =>
  invoke<ActiveSession | null>('get_active_session');

/** Updates an existing session's details (duration, notes, activity type). */
export const updateSession = (id: string, params: UpdateSessionParams) =>
  invoke<Session>('update_session', { id, params });

/** Fetches all sessions within a date range (inclusive). Dates are ISO strings (YYYY-MM-DD). */
export const listSessions = (startDate: string, endDate: string) =>
  invoke<Session[]>('list_sessions', { startDate, endDate });

/** Permanently deletes a session. */
export const deleteSession = (id: string) =>
  invoke<void>('delete_session', { id });

/** Creates a customer + work order and starts tracking in one operation. */
export const quickAdd = (params: QuickAddParams) =>
  invoke<QuickAddResult>('quick_add', { params });

/** Recovers an orphaned session by closing it with the current timestamp. */
export const recoverSession = (sessionId: string) =>
  invoke<Session>('recover_session', { sessionId });

/** Discards an orphaned session without saving it. */
export const discardOrphanSession = (sessionId: string) =>
  invoke<void>('discard_orphan_session', { sessionId });

/** Checks for any orphaned session that needs recovery on app startup. */
export const checkForOrphanSession = () =>
  invoke<OrphanSession | null>('check_for_orphan_session');

/** Gets the work order ID of the most recently stopped session. */
export const getLastStoppedWorkOrder = async (): Promise<string | null> => {
  return await invoke<string | null>('get_last_stopped_work_order');
};

/** Continues tracking on the most recently stopped work order. */
export const continueLastSession = async (): Promise<void> => {
  const workOrderId = await getLastStoppedWorkOrder();
  if (workOrderId) {
    await startSession(workOrderId);
  }
};
