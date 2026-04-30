import type { Session } from '../types';

export interface WorkOrderGroup {
  workOrderId: string;
  workOrderName: string;
  totalSeconds: number;
  sessionCount: number;
}

export interface CustomerGroup {
  customerName: string;
  customerColor: string | null;
  totalSeconds: number;
  workOrders: WorkOrderGroup[];
}

export interface DayGroup {
  date: string; // "YYYY-MM-DD"
  totalSeconds: number;
  customers: CustomerGroup[];
}

export interface WeekGroup {
  weekStart: string;    // "YYYY-MM-DD" — the Monday that starts this week
  weekLabel: string;    // Human-readable, e.g. "Apr 14 – Apr 20"
  totalSeconds: number;
  days: DayGroup[];     // sorted newest first within the week
}

/** Groups sessions into Day → Customer → WorkOrder structure, sorted newest day first. */
export function groupSessionsByDay(sessions: Session[]): DayGroup[] {
  const dayMap = new Map<string, Map<string, { color: string | null; wos: Map<string, WorkOrderGroup> }>>();

  for (const session of sessions) {
    const date = session.startTime.slice(0, 10);
    const customerName = session.customerName ?? 'Unknown Customer';
    const woId = session.workOrderId;
    const woName = session.workOrderName ?? 'Unknown Work Order';
    const seconds = session.durationSeconds ?? 0;

    if (!dayMap.has(date)) {
      dayMap.set(date, new Map());
    }
    const customerMap = dayMap.get(date)!;

    if (!customerMap.has(customerName)) {
      customerMap.set(customerName, { color: session.customerColor, wos: new Map() });
    }
    const customerEntry = customerMap.get(customerName)!;

    if (!customerEntry.wos.has(woId)) {
      customerEntry.wos.set(woId, { workOrderId: woId, workOrderName: woName, totalSeconds: 0, sessionCount: 0 });
    }
    const woEntry = customerEntry.wos.get(woId)!;
    woEntry.totalSeconds += seconds;
    woEntry.sessionCount += 1;
  }

  const dayGroups: DayGroup[] = [];

  for (const [date, customerMap] of dayMap) {
    const customers: CustomerGroup[] = [];

    for (const [customerName, customerEntry] of customerMap) {
      const workOrders = Array.from(customerEntry.wos.values()).sort((a, b) =>
        a.workOrderName.localeCompare(b.workOrderName)
      );
      const totalSeconds = workOrders.reduce((sum, wo) => sum + wo.totalSeconds, 0);
      customers.push({ customerName, customerColor: customerEntry.color, totalSeconds, workOrders });
    }

    customers.sort((a, b) => a.customerName.localeCompare(b.customerName));
    const totalSeconds = customers.reduce((sum, c) => sum + c.totalSeconds, 0);
    dayGroups.push({ date, totalSeconds, customers });
  }

  // Newest day first
  dayGroups.sort((a, b) => b.date.localeCompare(a.date));

  return dayGroups;
}

/** Returns the Monday (YYYY-MM-DD) of the week that contains the given date. */
function getMondayOf(dateStr: string): string {
  const d = new Date(dateStr + 'T00:00:00Z');
  const day = d.getUTCDay(); // 0=Sun, 1=Mon...6=Sat
  const diff = (day === 0) ? -6 : 1 - day; // days back to Monday
  d.setUTCDate(d.getUTCDate() + diff);
  return d.toISOString().slice(0, 10);
}

/** Formats a week label like "Apr 14 – Apr 20" or "Apr 14 – 20" if same month. */
function formatWeekLabel(mondayStr: string): string {
  const monday = new Date(mondayStr + 'T00:00:00Z');
  const sunday = new Date(monday);
  sunday.setUTCDate(sunday.getUTCDate() + 6);

  const monthNames = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  const monMonth = monthNames[monday.getUTCMonth()];
  const sunMonth = monthNames[sunday.getUTCMonth()];
  const monDay = monday.getUTCDate();
  const sunDay = sunday.getUTCDate();

  if (monMonth === sunMonth) {
    return `${monMonth} ${monDay} – ${sunDay}`;
  } else {
    return `${monMonth} ${monDay} – ${sunMonth} ${sunDay}`;
  }
}

/** Groups sessions into Week → Day → Customer → WorkOrder structure, sorted newest week first. */
export function groupSessionsByWeek(sessions: Session[]): WeekGroup[] {
  const dayGroups = groupSessionsByDay(sessions);
  const weekMap = new Map<string, DayGroup[]>();

  for (const dayGroup of dayGroups) {
    const weekStart = getMondayOf(dayGroup.date);
    if (!weekMap.has(weekStart)) {
      weekMap.set(weekStart, []);
    }
    weekMap.get(weekStart)!.push(dayGroup);
  }

  const weekGroups: WeekGroup[] = [];

  for (const [weekStart, days] of weekMap) {
    // Days are already sorted newest first within the week (from groupSessionsByDay)
    const totalSeconds = days.reduce((sum, d) => sum + d.totalSeconds, 0);
    const weekLabel = formatWeekLabel(weekStart);
    weekGroups.push({ weekStart, weekLabel, totalSeconds, days });
  }

  // Sort weeks newest first
  weekGroups.sort((a, b) => b.weekStart.localeCompare(a.weekStart));

  return weekGroups;
}
