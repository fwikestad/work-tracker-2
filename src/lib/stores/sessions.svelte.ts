import type { Session, WorkOrder } from '$lib/types';
import { listSessions } from '$lib/api/sessions';
import { getRecentWorkOrders } from '$lib/api/reports';
import { listWorkOrders } from '$lib/api/workOrders';

export interface WeekDay {
  date: string;
  label: string;
  isToday: boolean;
  sessions: Session[];
}

const MONTH_NAMES = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
const DAY_NAMES = ['Monday','Tuesday','Wednesday','Thursday','Friday','Saturday','Sunday'];

function toIsoDate(date: Date): string {
  return date.toISOString().split('T')[0];
}

function getMondayOfWeek(offset: number): Date {
  const now = new Date();
  const day = now.getDay(); // 0=Sun, 1=Mon, ..., 6=Sat
  const diffToMonday = day === 0 ? -6 : 1 - day;
  const monday = new Date(now);
  monday.setDate(now.getDate() + diffToMonday + offset * 7);
  monday.setHours(0, 0, 0, 0);
  return monday;
}

function buildWeekLabel(offset: number): string {
  const monday = getMondayOfWeek(offset);
  const sunday = new Date(monday);
  sunday.setDate(monday.getDate() + 6);
  const year = sunday.getFullYear();
  const monPart = `${MONTH_NAMES[monday.getMonth()]} ${monday.getDate()}`;
  const sunPart = `${MONTH_NAMES[sunday.getMonth()]} ${sunday.getDate()}`;
  return `${monPart} – ${sunPart}, ${year}`;
}

function buildWeekDays(monday: Date, byDate: Map<string, Session[]>): WeekDay[] {
  const todayStr = toIsoDate(new Date());
  return Array.from({ length: 7 }, (_, i) => {
    const d = new Date(monday);
    d.setDate(monday.getDate() + i);
    const dateStr = toIsoDate(d);
    return {
      date: dateStr,
      label: `${DAY_NAMES[i]} ${MONTH_NAMES[d.getMonth()]} ${d.getDate()}`,
      isToday: dateStr === todayStr,
      sessions: byDate.get(dateStr) ?? []
    };
  });
}

let todaysSessions = $state<Session[]>([]);
let recentWorkOrders = $state<WorkOrder[]>([]);
let allWorkOrders = $state<WorkOrder[]>([]);
let weekOffset = $state(0);
let weekSessions = $state<WeekDay[]>([]);

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
  get weekOffset() {
    return weekOffset;
  },
  get weekSessions() {
    return weekSessions;
  },
  get selectedWeekLabel() {
    return buildWeekLabel(weekOffset);
  },

  async setWeekOffset(n: number) {
    weekOffset = Math.min(0, n);
    await sessionsStore.refreshWeek();
  },

  async refreshToday() {
    const today = toIsoDate(new Date());
    todaysSessions = await listSessions(today, today);
  },

  async refreshWeek(offset?: number) {
    if (offset !== undefined) {
      weekOffset = Math.min(0, offset);
    }
    const monday = getMondayOfWeek(weekOffset);
    const sunday = new Date(monday);
    sunday.setDate(monday.getDate() + 6);
    const sessions = await listSessions(toIsoDate(monday), toIsoDate(sunday));
    const byDate = new Map<string, Session[]>();
    for (const s of sessions) {
      const d = s.startTime.split('T')[0];
      if (!byDate.has(d)) byDate.set(d, []);
      byDate.get(d)!.push(s);
    }
    weekSessions = buildWeekDays(monday, byDate);
    // Keep todaysSessions in sync when viewing the current week
    if (weekOffset === 0) {
      todaysSessions = byDate.get(toIsoDate(new Date())) ?? [];
    }
  },

  async refreshRecent() {
    const [recent, all] = await Promise.all([getRecentWorkOrders(10), listWorkOrders()]);
    recentWorkOrders = recent;
    allWorkOrders = all;
  },

  async refreshAll() {
    await Promise.all([sessionsStore.refreshWeek(), sessionsStore.refreshRecent()]);
    // When on a past week, todaysSessions won't be updated by refreshWeek,
    // so refresh it separately to keep the $effect in +page.svelte working.
    if (weekOffset !== 0) {
      await sessionsStore.refreshToday();
    }
  }
};
