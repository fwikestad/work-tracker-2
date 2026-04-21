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

/** Groups sessions into Day → Customer → WorkOrder structure, sorted newest day first. */
export function groupSessionsByDay(sessions: Session[]): DayGroup[] {
  const dayMap = new Map<string, Map<string, { color: string | null; wos: Map<string, WorkOrderGroup> }>>();

  for (const session of sessions) {
    const date = session.startTime.slice(0, 10);
    const customerName = session.customerName ?? 'Unknown Customer';
    const woId = session.workOrderId;
    const woName = session.workOrderName ?? 'Unknown Work Order';
    const seconds = session.effectiveDuration ?? 0;

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
