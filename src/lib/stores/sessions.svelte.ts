import type { Session, WorkOrder } from '$lib/types';
import { listSessions } from '$lib/api/sessions';
import { getRecentWorkOrders } from '$lib/api/reports';

let todaysSessions = $state<Session[]>([]);
let recentWorkOrders = $state<WorkOrder[]>([]);

export const sessionsStore = {
  get todays() {
    return todaysSessions;
  },
  get recent() {
    return recentWorkOrders;
  },

  async refreshToday() {
    const today = new Date().toISOString().split('T')[0];
    todaysSessions = await listSessions(today, today);
  },

  async refreshRecent() {
    recentWorkOrders = await getRecentWorkOrders(10);
  },

  async refreshAll() {
    await Promise.all([sessionsStore.refreshToday(), sessionsStore.refreshRecent()]);
  }
};
