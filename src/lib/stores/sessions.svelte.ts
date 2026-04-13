import type { Session, WorkOrder } from '$lib/types';
import { listSessions } from '$lib/api/sessions';
import { getRecentWorkOrders } from '$lib/api/reports';
import { listWorkOrders } from '$lib/api/workOrders';

let todaysSessions = $state<Session[]>([]);
let recentWorkOrders = $state<WorkOrder[]>([]);
let allWorkOrders = $state<WorkOrder[]>([]);

export const sessionsStore = {
  get todays() {
    return todaysSessions;
  },
  get recent() {
    return recentWorkOrders;
  },
  get allFavorites() {
    return allWorkOrders.filter((wo) => wo.isFavorite);
  },

  async refreshToday() {
    const today = new Date().toISOString().split('T')[0];
    todaysSessions = await listSessions(today, today);
  },

  async refreshRecent() {
    const [recent, all] = await Promise.all([getRecentWorkOrders(10), listWorkOrders()]);
    recentWorkOrders = recent;
    allWorkOrders = all;
  },

  async refreshAll() {
    await Promise.all([sessionsStore.refreshToday(), sessionsStore.refreshRecent()]);
  }
};
