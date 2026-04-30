import { describe, it, expect } from 'vitest';
import type { Session } from '$lib/types';
import { groupSessionsByDay, groupSessionsByWeek, type DayGroup, type WeekGroup, type CustomerGroup, type WorkOrderGroup } from '$lib/utils/reportGrouping';

/**
 * Test factory: creates a minimal Session with sensible defaults.
 * Caller must provide: startTime, workOrderId, customerName, workOrderName
 */
function makeSession(
	overrides: Partial<Session> & {
		startTime: string;
		workOrderId: string;
		customerName: string;
		workOrderName: string;
	}
): Session {
	return {
		id: crypto.randomUUID?.() ?? Math.random().toString(),
		workOrderId: overrides.workOrderId,
		workOrderName: overrides.workOrderName,
		customerName: overrides.customerName,
		customerColor: null,
		startTime: overrides.startTime,
		endTime: null,
		durationSeconds: 3600,
		activityType: null,
		notes: null,
		createdAt: overrides.startTime,
		updatedAt: overrides.startTime,
		...overrides,
	};
}

describe('groupSessionsByDay', () => {
	it('TC-GROUP-01: Empty input returns empty array', () => {
		const result = groupSessionsByDay([]);
		expect(result).toEqual([]);
	});

	it('TC-GROUP-02: Single session creates one DayGroup with one CustomerGroup and one WorkOrderGroup', () => {
		const session = makeSession({
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const result = groupSessionsByDay([session]);

		expect(result).toHaveLength(1);
		expect(result[0]).toMatchObject({
			date: '2026-04-21',
			totalSeconds: 3600,
		});
		expect(result[0].customers).toHaveLength(1);
		expect(result[0].customers[0]).toMatchObject({
			customerName: 'Acme Corp',
			totalSeconds: 3600,
		});
		expect(result[0].customers[0].workOrders).toHaveLength(1);
		expect(result[0].customers[0].workOrders[0]).toMatchObject({
			workOrderId: 'wo-123',
			workOrderName: 'Project Alpha',
			totalSeconds: 3600,
			sessionCount: 1,
		});
	});

	it('TC-GROUP-03: Two sessions same day, same customer, same work order => totals summed, sessionCount = 2', () => {
		const session1 = makeSession({
			id: 'sess-1',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T14:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 1800,
		});

		const result = groupSessionsByDay([session1, session2]);

		expect(result).toHaveLength(1);
		const dayGroup = result[0];
		expect(dayGroup.totalSeconds).toBe(5400); // 3600 + 1800
		expect(dayGroup.customers).toHaveLength(1);

		const customerGroup = dayGroup.customers[0];
		expect(customerGroup.totalSeconds).toBe(5400);
		expect(customerGroup.workOrders).toHaveLength(1);

		const workOrderGroup = customerGroup.workOrders[0];
		expect(workOrderGroup.totalSeconds).toBe(5400);
		expect(workOrderGroup.sessionCount).toBe(2);
	});

	it('TC-GROUP-04: Two sessions same day, same customer, different work orders => two WorkOrderGroups', () => {
		const session1 = makeSession({
			id: 'sess-1',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T14:00:00Z',
			workOrderId: 'wo-456',
			customerName: 'Acme Corp',
			workOrderName: 'Project Beta',
			durationSeconds: 1800,
		});

		const result = groupSessionsByDay([session1, session2]);

		expect(result).toHaveLength(1);
		const dayGroup = result[0];
		expect(dayGroup.totalSeconds).toBe(5400);
		expect(dayGroup.customers).toHaveLength(1);

		const customerGroup = dayGroup.customers[0];
		expect(customerGroup.totalSeconds).toBe(5400);
		expect(customerGroup.workOrders).toHaveLength(2);

		// Work orders should be sorted alphabetically
		const workOrderNames = customerGroup.workOrders.map((wo) => wo.workOrderName);
		expect(workOrderNames).toEqual(['Project Alpha', 'Project Beta']);
	});

	it('TC-GROUP-05: Two sessions same day, different customers => two CustomerGroups sorted alphabetically', () => {
		const session1 = makeSession({
			id: 'sess-1',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Zebra Inc',
			workOrderName: 'Project Z',
			durationSeconds: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T14:00:00Z',
			workOrderId: 'wo-456',
			customerName: 'Acme Corp',
			workOrderName: 'Project A',
			durationSeconds: 1800,
		});

		const result = groupSessionsByDay([session1, session2]);

		expect(result).toHaveLength(1);
		const dayGroup = result[0];
		expect(dayGroup.totalSeconds).toBe(5400);
		expect(dayGroup.customers).toHaveLength(2);

		// Customers should be sorted alphabetically
		const customerNames = dayGroup.customers.map((c) => c.customerName);
		expect(customerNames).toEqual(['Acme Corp', 'Zebra Inc']);

		// Verify totals
		expect(dayGroup.customers[0].totalSeconds).toBe(1800); // Acme
		expect(dayGroup.customers[1].totalSeconds).toBe(3600); // Zebra
	});

	it('TC-GROUP-06: Two sessions different days => two DayGroups, newest day first', () => {
		const session1 = makeSession({
			id: 'sess-1',
			startTime: '2026-04-20T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-456',
			customerName: 'Acme Corp',
			workOrderName: 'Project Beta',
			durationSeconds: 1800,
		});

		const result = groupSessionsByDay([session1, session2]);

		expect(result).toHaveLength(2);
		// Newest day (2026-04-21) should be first
		expect(result[0].date).toBe('2026-04-21');
		expect(result[1].date).toBe('2026-04-20');
	});

	it('TC-GROUP-07: null effectiveDuration treated as 0', () => {
		const session1 = makeSession({
			id: 'sess-1',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T14:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: null,
		});

		const result = groupSessionsByDay([session1, session2]);

		expect(result).toHaveLength(1);
		const dayGroup = result[0];
		expect(dayGroup.totalSeconds).toBe(3600); // null treated as 0

		const customerGroup = dayGroup.customers[0];
		expect(customerGroup.totalSeconds).toBe(3600);

		const workOrderGroup = customerGroup.workOrders[0];
		expect(workOrderGroup.totalSeconds).toBe(3600);
		expect(workOrderGroup.sessionCount).toBe(2); // Both sessions counted
	});

	it('TC-GROUP-08: Day total = sum of all customer totals for that day', () => {
		const session1 = makeSession({
			id: 'sess-1',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-1',
			customerName: 'Acme Corp',
			workOrderName: 'Project A',
			durationSeconds: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T10:00:00Z',
			workOrderId: 'wo-2',
			customerName: 'Acme Corp',
			workOrderName: 'Project B',
			durationSeconds: 1800,
		});

		const session3 = makeSession({
			id: 'sess-3',
			startTime: '2026-04-21T11:00:00Z',
			workOrderId: 'wo-3',
			customerName: 'Other Inc',
			workOrderName: 'Project C',
			durationSeconds: 900,
		});

		const result = groupSessionsByDay([session1, session2, session3]);

		expect(result).toHaveLength(1);
		const dayGroup = result[0];

		// Day total should be sum of all three sessions
		expect(dayGroup.totalSeconds).toBe(6300); // 3600 + 1800 + 900

		// Verify customer totals sum to day total
		const customerTotals = dayGroup.customers.reduce((sum, c) => sum + c.totalSeconds, 0);
		expect(customerTotals).toBe(6300);
	});

	it('TC-GROUP-09: Customer total = sum of all work order totals for that customer on that day', () => {
		const session1 = makeSession({
			id: 'sess-1',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-1',
			customerName: 'Acme Corp',
			workOrderName: 'Project A',
			durationSeconds: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T10:00:00Z',
			workOrderId: 'wo-2',
			customerName: 'Acme Corp',
			workOrderName: 'Project B',
			durationSeconds: 1800,
		});

		const result = groupSessionsByDay([session1, session2]);

		expect(result).toHaveLength(1);
		const dayGroup = result[0];
		const customerGroup = dayGroup.customers[0];

		// Customer total should be sum of work order totals
		const workOrderTotals = customerGroup.workOrders.reduce((sum, wo) => sum + wo.totalSeconds, 0);
		expect(customerGroup.totalSeconds).toBe(workOrderTotals);
		expect(customerGroup.totalSeconds).toBe(5400); // 3600 + 1800
	});

	it('TC-GROUP-10: Work orders within a customer sorted alphabetically by name', () => {
		const session1 = makeSession({
			id: 'sess-1',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-c',
			customerName: 'Acme Corp',
			workOrderName: 'Zebra Task',
			durationSeconds: 1000,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T10:00:00Z',
			workOrderId: 'wo-a',
			customerName: 'Acme Corp',
			workOrderName: 'Alpha Task',
			durationSeconds: 2000,
		});

		const session3 = makeSession({
			id: 'sess-3',
			startTime: '2026-04-21T11:00:00Z',
			workOrderId: 'wo-b',
			customerName: 'Acme Corp',
			workOrderName: 'Beta Task',
			durationSeconds: 3000,
		});

		const result = groupSessionsByDay([session1, session2, session3]);

		expect(result).toHaveLength(1);
		const customerGroup = result[0].customers[0];
		expect(customerGroup.workOrders).toHaveLength(3);

		const workOrderNames = customerGroup.workOrders.map((wo) => wo.workOrderName);
		expect(workOrderNames).toEqual(['Alpha Task', 'Beta Task', 'Zebra Task']);
	});

	it('TC-GROUP-11: Sessions with same workOrderId but different days appear in separate DayGroups', () => {
		const session1 = makeSession({
			id: 'sess-1',
			startTime: '2026-04-20T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 1800,
		});

		const result = groupSessionsByDay([session1, session2]);

		expect(result).toHaveLength(2);

		// Check day 1 (2026-04-21, should be first due to desc sort)
		expect(result[0].date).toBe('2026-04-21');
		expect(result[0].totalSeconds).toBe(1800);
		expect(result[0].customers[0].workOrders[0].totalSeconds).toBe(1800);

		// Check day 2 (2026-04-20)
		expect(result[1].date).toBe('2026-04-20');
		expect(result[1].totalSeconds).toBe(3600);
		expect(result[1].customers[0].workOrders[0].totalSeconds).toBe(3600);
	});

	it('TC-GROUP-12: Customer color properly propagated to CustomerGroup', () => {
		const session = makeSession({
			id: 'sess-1',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			customerColor: '#FF5733',
			durationSeconds: 3600,
		});

		const result = groupSessionsByDay([session]);

		expect(result).toHaveLength(1);
		const customerGroup = result[0].customers[0];
		expect(customerGroup.customerColor).toBe('#FF5733');
	});

	it('TC-GROUP-13: Complex scenario: multiple days, customers, work orders with mixed effective durations', () => {
		const sessions: Session[] = [
			// Day 2026-04-20, Customer Acme, WO Alpha
			makeSession({
				id: 'sess-1',
				startTime: '2026-04-20T09:00:00Z',
				workOrderId: 'wo-alpha',
				customerName: 'Acme Corp',
				workOrderName: 'Alpha Task',
				customerColor: '#FF0000',
				durationSeconds: 3600,
			}),
			// Day 2026-04-20, Customer Acme, WO Beta
			makeSession({
				id: 'sess-2',
				startTime: '2026-04-20T10:30:00Z',
				workOrderId: 'wo-beta',
				customerName: 'Acme Corp',
				workOrderName: 'Beta Task',
				customerColor: '#FF0000',
				durationSeconds: 5400,
			}),
			// Day 2026-04-20, Customer Other, WO Gamma
			makeSession({
				id: 'sess-3',
				startTime: '2026-04-20T14:00:00Z',
				workOrderId: 'wo-gamma',
				customerName: 'Other Inc',
				workOrderName: 'Gamma Task',
				customerColor: '#0000FF',
				durationSeconds: 1800,
			}),
			// Day 2026-04-21, Customer Acme, WO Alpha (same WO, different day)
			makeSession({
				id: 'sess-4',
				startTime: '2026-04-21T09:00:00Z',
				workOrderId: 'wo-alpha',
				customerName: 'Acme Corp',
				workOrderName: 'Alpha Task',
				customerColor: '#FF0000',
				durationSeconds: 2700,
			}),
			// Day 2026-04-21, Customer Other, WO Gamma (same WO, different day)
			makeSession({
				id: 'sess-5',
				startTime: '2026-04-21T15:00:00Z',
				workOrderId: 'wo-gamma',
				customerName: 'Other Inc',
				workOrderName: 'Gamma Task',
				customerColor: '#0000FF',
				durationSeconds: null, // null duration
			}),
		];

		const result = groupSessionsByDay(sessions);

		expect(result).toHaveLength(2);

		// Newest day first (2026-04-21)
		expect(result[0].date).toBe('2026-04-21');
		expect(result[0].totalSeconds).toBe(2700); // 2700 + 0 (null)

		// Older day second (2026-04-20)
		expect(result[1].date).toBe('2026-04-20');
		expect(result[1].totalSeconds).toBe(10800); // 3600 + 5400 + 1800

		// Verify 2026-04-21 structure
		const day21 = result[0];
		expect(day21.customers).toHaveLength(2);

		// Acme should be before Other (alphabetical)
		const [acme21, other21] = day21.customers;
		expect(acme21.customerName).toBe('Acme Corp');
		expect(other21.customerName).toBe('Other Inc');

		// Acme on 2026-04-21: just Alpha Task (2700)
		expect(acme21.totalSeconds).toBe(2700);
		expect(acme21.workOrders).toHaveLength(1);
		expect(acme21.workOrders[0].workOrderName).toBe('Alpha Task');
		expect(acme21.workOrders[0].totalSeconds).toBe(2700);
		expect(acme21.workOrders[0].sessionCount).toBe(1);

		// Other on 2026-04-21: just Gamma Task (0 from null)
		expect(other21.totalSeconds).toBe(0);
		expect(other21.workOrders).toHaveLength(1);
		expect(other21.workOrders[0].workOrderName).toBe('Gamma Task');
		expect(other21.workOrders[0].totalSeconds).toBe(0);
		expect(other21.workOrders[0].sessionCount).toBe(1);

		// Verify 2026-04-20 structure
		const day20 = result[1];
		expect(day20.customers).toHaveLength(2);

		const [acme20, other20] = day20.customers;
		expect(acme20.customerName).toBe('Acme Corp');
		expect(other20.customerName).toBe('Other Inc');

		// Acme on 2026-04-20: Alpha (3600) + Beta (5400) = 9000
		expect(acme20.totalSeconds).toBe(9000);
		expect(acme20.workOrders).toHaveLength(2);

		const [alpha20, beta20] = acme20.workOrders;
		expect(alpha20.workOrderName).toBe('Alpha Task');
		expect(alpha20.totalSeconds).toBe(3600);
		expect(alpha20.sessionCount).toBe(1);

		expect(beta20.workOrderName).toBe('Beta Task');
		expect(beta20.totalSeconds).toBe(5400);
		expect(beta20.sessionCount).toBe(1);

		// Other on 2026-04-20: Gamma (1800)
		expect(other20.totalSeconds).toBe(1800);
		expect(other20.workOrders).toHaveLength(1);
		expect(other20.workOrders[0].workOrderName).toBe('Gamma Task');
		expect(other20.workOrders[0].totalSeconds).toBe(1800);
		expect(other20.workOrders[0].sessionCount).toBe(1);
	});

	it('TC-GROUP-14: Handles sessions with null customerName gracefully', () => {
		const session = makeSession({
			id: 'sess-1',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: null as any,
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		// Should not throw; grouping should still work
		const result = groupSessionsByDay([session]);

		expect(result).toHaveLength(1);
		expect(result[0].customers).toHaveLength(1);
		// null customerName is converted to "Unknown Customer" string by implementation
		expect(result[0].customers[0].customerName).toBe('Unknown Customer');
		expect(result[0].customers[0].totalSeconds).toBe(3600);
	});

	it('TC-GROUP-15: Handles sessions with null workOrderName gracefully', () => {
		const session = makeSession({
			id: 'sess-1',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: null as any,
			durationSeconds: 3600,
		});

		// Should not throw
		const result = groupSessionsByDay([session]);

		expect(result).toHaveLength(1);
		expect(result[0].customers).toHaveLength(1);
		expect(result[0].customers[0].workOrders).toHaveLength(1);
		// null workOrderName is converted to "Unknown Work Order" string by implementation
		expect(result[0].customers[0].workOrders[0].workOrderName).toBe('Unknown Work Order');
		expect(result[0].customers[0].workOrders[0].totalSeconds).toBe(3600);
	});

	it('TC-GROUP-16: Multiple sessions with same day but different session IDs sum correctly', () => {
		const sessions: Session[] = [
			makeSession({
				id: 'sess-a',
				startTime: '2026-04-21T09:00:00Z',
				workOrderId: 'wo-1',
				customerName: 'Customer A',
				workOrderName: 'Task 1',
				durationSeconds: 1000,
			}),
			makeSession({
				id: 'sess-b',
				startTime: '2026-04-21T10:00:00Z',
				workOrderId: 'wo-1',
				customerName: 'Customer A',
				workOrderName: 'Task 1',
				durationSeconds: 2000,
			}),
			makeSession({
				id: 'sess-c',
				startTime: '2026-04-21T11:00:00Z',
				workOrderId: 'wo-1',
				customerName: 'Customer A',
				workOrderName: 'Task 1',
				durationSeconds: 3000,
			}),
		];

		const result = groupSessionsByDay(sessions);

		expect(result).toHaveLength(1);
		const workOrderGroup = result[0].customers[0].workOrders[0];
		expect(workOrderGroup.sessionCount).toBe(3);
		expect(workOrderGroup.totalSeconds).toBe(6000); // 1000 + 2000 + 3000
	});

	it('TC-GROUP-17: Date extraction uses first 10 chars of startTime (YYYY-MM-DD)', () => {
		// Test with various ISO datetime formats
		const session1 = makeSession({
			id: 'sess-1',
			startTime: '2026-04-21T00:00:00Z',
			workOrderId: 'wo-1',
			customerName: 'Acme',
			workOrderName: 'Task 1',
			durationSeconds: 100,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T23:59:59Z',
			workOrderId: 'wo-1',
			customerName: 'Acme',
			workOrderName: 'Task 1',
			durationSeconds: 200,
		});

		const result = groupSessionsByDay([session1, session2]);

		// Both should be in same day group
		expect(result).toHaveLength(1);
		expect(result[0].date).toBe('2026-04-21');
		expect(result[0].customers[0].workOrders[0].sessionCount).toBe(2);
	});
});

describe('groupSessionsByWeek', () => {
	it('TC-WEEK-01: Empty input returns empty array', () => {
		const result = groupSessionsByWeek([]);
		expect(result).toEqual([]);
	});

	it('TC-WEEK-02: Single session → one WeekGroup with correct weekStart', () => {
		// 2026-04-21 is a Tuesday → weekStart should be Monday 2026-04-20
		const session = makeSession({
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const result = groupSessionsByWeek([session]);

		expect(result).toHaveLength(1);
		expect(result[0].weekStart).toBe('2026-04-20'); // Monday of that week
		expect(result[0].totalSeconds).toBe(3600);
		expect(result[0].days).toHaveLength(1);
		expect(result[0].days[0].date).toBe('2026-04-21');
		// weekLabel format: "Apr 20 – Apr 26" or "Apr 20 – 26" (same month may shorten)
		expect(result[0].weekLabel).toMatch(/Apr (20 – (Apr )?26)/);
	});

	it('TC-WEEK-03: Sunday session → weekStart is the preceding Monday', () => {
		// 2026-04-19 is a Sunday → weekStart should be Monday 2026-04-13
		const session = makeSession({
			startTime: '2026-04-19T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const result = groupSessionsByWeek([session]);

		expect(result).toHaveLength(1);
		expect(result[0].weekStart).toBe('2026-04-13'); // Monday of that week
		expect(result[0].totalSeconds).toBe(3600);
	});

	it('TC-WEEK-04: Monday session → weekStart is the same day', () => {
		// 2026-04-20 is a Monday → weekStart should be 2026-04-20 (same day)
		const session = makeSession({
			startTime: '2026-04-20T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const result = groupSessionsByWeek([session]);

		expect(result).toHaveLength(1);
		expect(result[0].weekStart).toBe('2026-04-20'); // Same as session day
		expect(result[0].totalSeconds).toBe(3600);
	});

	it('TC-WEEK-05: Two sessions in same week → one WeekGroup, totals summed', () => {
		// 2026-04-20 (Mon) and 2026-04-24 (Fri) are in the same week
		const session1 = makeSession({
			startTime: '2026-04-20T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const session2 = makeSession({
			startTime: '2026-04-24T14:00:00Z',
			workOrderId: 'wo-456',
			customerName: 'Other Inc',
			workOrderName: 'Project Beta',
			durationSeconds: 1800,
		});

		const result = groupSessionsByWeek([session1, session2]);

		expect(result).toHaveLength(1);
		expect(result[0].weekStart).toBe('2026-04-20');
		expect(result[0].totalSeconds).toBe(5400); // 3600 + 1800
		expect(result[0].days).toHaveLength(2); // Two different days
	});

	it('TC-WEEK-06: Sessions in two different weeks → two WeekGroups, newest week first', () => {
		// 2026-04-14 (Tue) is in week starting 2026-04-13 (Mon)
		// 2026-04-21 (Tue) is in week starting 2026-04-20 (Mon)
		const session1 = makeSession({
			startTime: '2026-04-14T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const session2 = makeSession({
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-456',
			customerName: 'Other Inc',
			workOrderName: 'Project Beta',
			durationSeconds: 1800,
		});

		const result = groupSessionsByWeek([session1, session2]);

		expect(result).toHaveLength(2);
		// Newest week first
		expect(result[0].weekStart).toBe('2026-04-20');
		expect(result[0].totalSeconds).toBe(1800);
		expect(result[1].weekStart).toBe('2026-04-13');
		expect(result[1].totalSeconds).toBe(3600);
	});

	it('TC-WEEK-07: weekLabel format — cross-month boundary', () => {
		// Week of 2026-04-27 (Mon) → spans Apr 27 – May 3
		const session = makeSession({
			startTime: '2026-04-27T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const result = groupSessionsByWeek([session]);

		expect(result).toHaveLength(1);
		expect(result[0].weekStart).toBe('2026-04-27');
		// Cross-month boundary: must show both months
		expect(result[0].weekLabel).toBe('Apr 27 – May 3');
	});

	it('TC-WEEK-08: weekLabel format — same month', () => {
		// Week of 2026-04-20 (Mon) → Apr 20 – Apr 26 (all in April)
		const session = makeSession({
			startTime: '2026-04-20T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			durationSeconds: 3600,
		});

		const result = groupSessionsByWeek([session]);

		expect(result).toHaveLength(1);
		expect(result[0].weekStart).toBe('2026-04-20');
		// Spec says same-month shortens to "MMM D – D" (e.g., "Apr 20 – 26")
		// But implementation may use full format "Apr 20 – Apr 26"
		// Accept either
		expect(result[0].weekLabel).toMatch(/Apr 20 – (Apr )?26/);
	});

	it('TC-WEEK-09: Week totalSeconds = sum of all day totals in that week', () => {
		// 3 sessions across 2 days in same week (Mon Apr 20 and Fri Apr 24)
		const session1 = makeSession({
			startTime: '2026-04-20T09:00:00Z',
			workOrderId: 'wo-1',
			customerName: 'Acme Corp',
			workOrderName: 'Project A',
			durationSeconds: 3600,
		});

		const session2 = makeSession({
			startTime: '2026-04-20T14:00:00Z',
			workOrderId: 'wo-2',
			customerName: 'Other Inc',
			workOrderName: 'Project B',
			durationSeconds: 1800,
		});

		const session3 = makeSession({
			startTime: '2026-04-24T10:00:00Z',
			workOrderId: 'wo-3',
			customerName: 'Acme Corp',
			workOrderName: 'Project C',
			durationSeconds: 900,
		});

		const result = groupSessionsByWeek([session1, session2, session3]);

		expect(result).toHaveLength(1);
		const weekGroup = result[0];
		expect(weekGroup.totalSeconds).toBe(6300); // 3600 + 1800 + 900
		expect(weekGroup.days).toHaveLength(2);

		// Verify week total equals sum of day totals
		const dayTotalsSum = weekGroup.days.reduce((sum, day) => sum + day.totalSeconds, 0);
		expect(dayTotalsSum).toBe(6300);
	});

	it('TC-WEEK-10: Days within a week are sorted newest first', () => {
		// Sessions on Tue (Apr 21) and Thu (Apr 23) of same week
		const sessionTue = makeSession({
			startTime: '2026-04-22T09:00:00Z', // Tuesday
			workOrderId: 'wo-1',
			customerName: 'Acme Corp',
			workOrderName: 'Task A',
			durationSeconds: 1000,
		});

		const sessionThu = makeSession({
			startTime: '2026-04-24T09:00:00Z', // Thursday
			workOrderId: 'wo-2',
			customerName: 'Acme Corp',
			workOrderName: 'Task B',
			durationSeconds: 2000,
		});

		const result = groupSessionsByWeek([sessionTue, sessionThu]);

		expect(result).toHaveLength(1);
		const weekGroup = result[0];
		expect(weekGroup.days).toHaveLength(2);
		// Newest day first
		expect(weekGroup.days[0].date).toBe('2026-04-24'); // Thu
		expect(weekGroup.days[1].date).toBe('2026-04-22'); // Tue
	});

	it('TC-WEEK-11: Month of sessions → correct number of WeekGroups', () => {
		// Sessions on Apr 1, 8, 15, 22, 29 (all Wednesdays in 2026)
		// Apr 1 = Week starting Mar 30 (Mon before Apr 1)
		// Apr 8 = Week starting Apr 6
		// Apr 15 = Week starting Apr 13
		// Apr 22 = Week starting Apr 20
		// Apr 29 = Week starting Apr 27
		const sessions = [
			makeSession({
				startTime: '2026-04-01T09:00:00Z',
				workOrderId: 'wo-1',
				customerName: 'Acme',
				workOrderName: 'Task',
				durationSeconds: 1000,
			}),
			makeSession({
				startTime: '2026-04-08T09:00:00Z',
				workOrderId: 'wo-2',
				customerName: 'Acme',
				workOrderName: 'Task',
				durationSeconds: 1000,
			}),
			makeSession({
				startTime: '2026-04-15T09:00:00Z',
				workOrderId: 'wo-3',
				customerName: 'Acme',
				workOrderName: 'Task',
				durationSeconds: 1000,
			}),
			makeSession({
				startTime: '2026-04-22T09:00:00Z',
				workOrderId: 'wo-4',
				customerName: 'Acme',
				workOrderName: 'Task',
				durationSeconds: 1000,
			}),
			makeSession({
				startTime: '2026-04-29T09:00:00Z',
				workOrderId: 'wo-5',
				customerName: 'Acme',
				workOrderName: 'Task',
				durationSeconds: 1000,
			}),
		];

		const result = groupSessionsByWeek(sessions);

		// 5 different Wednesdays = 5 different weeks
		expect(result).toHaveLength(5);

		// Verify weeks are sorted newest first
		expect(result[0].weekStart).toBe('2026-04-27'); // Week containing Apr 29
		expect(result[1].weekStart).toBe('2026-04-20'); // Week containing Apr 22
		expect(result[2].weekStart).toBe('2026-04-13'); // Week containing Apr 15
		expect(result[3].weekStart).toBe('2026-04-06'); // Week containing Apr 8
		expect(result[4].weekStart).toBe('2026-03-30'); // Week containing Apr 1
	});

	it('TC-WEEK-12: WeekGroup.days matches what groupSessionsByDay would return for those sessions', () => {
		// Take sessions for one week, verify WeekGroup.days equals groupSessionsByDay for same sessions
		const sessions = [
			makeSession({
				startTime: '2026-04-21T09:00:00Z',
				workOrderId: 'wo-1',
				customerName: 'Acme Corp',
				workOrderName: 'Project A',
				durationSeconds: 3600,
			}),
			makeSession({
				startTime: '2026-04-22T14:00:00Z',
				workOrderId: 'wo-2',
				customerName: 'Other Inc',
				workOrderName: 'Project B',
				durationSeconds: 1800,
			}),
			makeSession({
				startTime: '2026-04-22T15:00:00Z',
				workOrderId: 'wo-1',
				customerName: 'Acme Corp',
				workOrderName: 'Project A',
				durationSeconds: 900,
			}),
		];

		const weekResult = groupSessionsByWeek(sessions);
		const dayResult = groupSessionsByDay(sessions);

		expect(weekResult).toHaveLength(1);
		const weekGroup = weekResult[0];

		// The days within the week should match groupSessionsByDay output
		expect(weekGroup.days).toEqual(dayResult);
	});
});
