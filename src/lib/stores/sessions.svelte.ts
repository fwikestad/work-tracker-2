/**
 * Reactive store for managing sessions, recent work orders, and week view navigation.
 *
 * This store handles fetching and organizing time tracking data by day/week,
 * maintaining lists of recent work orders and favorites, and managing the week
 * view offset for browsing historical data.
 *
 * @module sessionsStore
 */
import type { Session, WorkOrder } from '$lib/types';
import { listSessions } from '$lib/api/sessions';
import { getRecentWorkOrders } from '$lib/api/reports';
import { listWorkOrders } from '$lib/api/workOrders';

/**
 * Represents a single day in the week view with its sessions.
 */
export interface WeekDay {
  /** ISO date string (YYYY-MM-DD). */
  date: string;
  /** Human-readable day label, e.g., "Monday Apr 14". */
  label: string;
  /** True if this day is today. */
  isToday: boolean;
  /** All sessions tracked on this day. */
  sessions: Session[];
}

const MONTH_NAMES = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
const DAY_NAMES = ['Monday','Tuesday','Wednesday','Thursday','Friday','Saturday','Sunday'];

/**
 * Converts a Date object to an ISO date string (YYYY-MM-DD).
 *
 * @param date - The date to convert.
 * @returns ISO date string.
 */
function toIsoDate(date: Date): string {
  return date.toISOString().split('T')[0];
}

/**
 * Gets the Monday of the week relative to today.
 *
 * @param offset - Week offset from current week (0 = this week, -1 = last week).
 * @returns Date object set to Monday at midnight.
 */
function getMondayOfWeek(offset: number): Date {
  const now = new Date();
  const day = now.getDay(); // 0=Sun, 1=Mon, ..., 6=Sat
  const diffToMonday = day === 0 ? -6 : 1 - day;
  const monday = new Date(now);
  monday.setDate(now.getDate() + diffToMonday + offset * 7);
  monday.setHours(0, 0, 0, 0);
  return monday;
}

/**
 * Builds a human-readable label for a week.
 *
 * @param offset - Week offset from current week.
 * @returns Label like "Apr 7 – Apr 13, 2025".
 */
function buildWeekLabel(offset: number): string {
  const monday = getMondayOfWeek(offset);
  const sunday = new Date(monday);
  sunday.setDate(monday.getDate() + 6);
  const year = sunday.getFullYear();
  const monPart = `${MONTH_NAMES[monday.getMonth()]} ${monday.getDate()}`;
  const sunPart = `${MONTH_NAMES[sunday.getMonth()]} ${sunday.getDate()}`;
  return `${monPart} – ${sunPart}, ${year}`;
}

/**
 * Builds an array of 7 WeekDay objects starting from Monday.
 *
 * @param monday - The Monday date for the week.
 * @param byDate - Map of ISO date strings to sessions.
 * @returns Array of 7 WeekDay objects (Monday through Sunday).
 */
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

/** All sessions tracked today (continuously updated). */
let todaysSessions = $state<Session[]>([]);

/** Recently tracked work orders (last 10 by default). */
let recentWorkOrders = $state<WorkOrder[]>([]);

/** All work orders (for favorites filtering). */
let allWorkOrders = $state<WorkOrder[]>([]);

/** Current week offset (0 = this week, -1 = last week, etc.). */
let weekOffset = $state(0);

/** Array of 7 days (Monday-Sunday) for the currently selected week. */
let weekSessions = $state<WeekDay[]>([]);

/**
 * Sessions store for managing time tracking data and week navigation.
 *
 * Provides reactive access to today's sessions, recent work orders, favorites,
 * and week-based session browsing.
 */
export const sessionsStore = {
  /**
   * All sessions tracked today.
   *
   * Updated by `refreshToday()` and kept in sync with `refreshWeek()` when viewing the current week.
   *
   * @returns Array of today's sessions.
   */
  get todays() {
    return todaysSessions;
  },

  /**
   * Recently tracked work orders (typically last 10).
   *
   * Used for quick-switch dropdown and favorites list.
   *
   * @returns Array of recent work orders.
   */
  get recent() {
    return recentWorkOrders;
  },

  /**
   * All work orders marked as favorites.
   *
   * Filtered from the full work orders list maintained by `refreshRecent()`.
   *
   * @returns Array of favorite work orders.
   */
  get allFavorites() {
    return allWorkOrders.filter((wo) => wo.isFavorite);
  },

  /**
   * Current week offset from today's week.
   *
   * @returns 0 for this week, -1 for last week, -2 for two weeks ago, etc.
   */
  get weekOffset() {
    return weekOffset;
  },

  /**
   * Array of 7 days (Monday through Sunday) for the currently selected week.
   *
   * Each day includes its sessions, date, label, and whether it's today.
   *
   * @returns Array of WeekDay objects.
   */
  get weekSessions() {
    return weekSessions;
  },

  /**
   * Human-readable label for the currently selected week.
   *
   * @returns Label like "Apr 7 – Apr 13, 2025".
   */
  get selectedWeekLabel() {
    return buildWeekLabel(weekOffset);
  },

  /**
   * Sets the week offset and refreshes the week view.
   *
   * The offset is clamped to 0 (current week) or negative values (past weeks).
   * Future weeks are not supported.
   *
   * @param n - Week offset (0 = this week, -1 = last week, -2 = two weeks ago, etc.).
   */
  async setWeekOffset(n: number) {
    weekOffset = Math.min(0, n);
    await sessionsStore.refreshWeek();
  },

  /**
   * Refreshes today's sessions from the backend.
   *
   * Fetches all sessions for today only. Use this to update `todays` when
   * the week view is showing a past week (and thus not updating today's data).
   */
  async refreshToday() {
    const today = toIsoDate(new Date());
    todaysSessions = await listSessions(today, today);
  },

  /**
   * Refreshes the week view for the currently selected week.
   *
   * Fetches all sessions for Monday through Sunday of the selected week,
   * groups them by day, and builds the `weekSessions` array.
   *
   * If viewing the current week (offset = 0), also updates `todaysSessions`.
   *
   * @param offset - Optional week offset to set before refreshing.
   */
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

  /**
   * Refreshes the recent work orders list and all work orders (for favorites).
   *
   * Fetches the last 10 recently-used work orders and all work orders from the backend.
   */
  async refreshRecent() {
    const [recent, all] = await Promise.all([getRecentWorkOrders(10), listWorkOrders()]);
    recentWorkOrders = recent;
    allWorkOrders = all;
  },

  /**
   * Refreshes all data: week view, recent work orders, and today's sessions.
   *
   * Use this as the main refresh after creating/updating/deleting sessions or work orders.
   * If viewing a past week, also refreshes `todaysSessions` separately to keep it current.
   */
  async refreshAll() {
    await Promise.all([sessionsStore.refreshWeek(), sessionsStore.refreshRecent()]);
    // When on a past week, todaysSessions won't be updated by refreshWeek,
    // so refresh it separately to keep the $effect in +page.svelte working.
    if (weekOffset !== 0) {
      await sessionsStore.refreshToday();
    }
  }
};
