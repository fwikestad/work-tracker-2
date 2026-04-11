import { invoke } from '@tauri-apps/api/core';
import type {
  Session,
  ActiveSession,
  OrphanSession,
  QuickAddParams,
  QuickAddResult,
  UpdateSessionParams,
} from '../types';

export const startSession = (workOrderId: string) =>
  invoke<Session>('start_session', { workOrderId });

export const stopSession = (notes?: string, activityType?: string) =>
  invoke<Session | null>('stop_session', { notes, activityType });

export const getActiveSession = () =>
  invoke<ActiveSession | null>('get_active_session');

export const updateSession = (id: string, params: UpdateSessionParams) =>
  invoke<Session>('update_session', { id, params });

export const listSessions = (startDate: string, endDate: string) =>
  invoke<Session[]>('list_sessions', { startDate, endDate });

export const deleteSession = (id: string) =>
  invoke<void>('delete_session', { id });

export const quickAdd = (params: QuickAddParams) =>
  invoke<QuickAddResult>('quick_add', { params });

export const recoverSession = (sessionId: string) =>
  invoke<Session>('recover_session', { sessionId });

export const discardOrphanSession = (sessionId: string) =>
  invoke<void>('discard_orphan_session', { sessionId });

export const checkForOrphanSession = () =>
  invoke<OrphanSession | null>('check_for_orphan_session');

export const pauseSession = () =>
  invoke<ActiveSession>('pause_session');

export const resumeSession = () =>
  invoke<ActiveSession>('resume_session');
