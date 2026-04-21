import { describe, it, expect } from 'vitest';
import type { Session } from '$lib/types';
import { groupSessionsByDay, type DayGroup, type CustomerGroup, type WorkOrderGroup } from '$lib/utils/reportGrouping';

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
		durationOverride: null,
		effectiveDuration: 3600,
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
			effectiveDuration: 3600,
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
			effectiveDuration: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T14:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			effectiveDuration: 1800,
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
			effectiveDuration: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T14:00:00Z',
			workOrderId: 'wo-456',
			customerName: 'Acme Corp',
			workOrderName: 'Project Beta',
			effectiveDuration: 1800,
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
			effectiveDuration: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T14:00:00Z',
			workOrderId: 'wo-456',
			customerName: 'Acme Corp',
			workOrderName: 'Project A',
			effectiveDuration: 1800,
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
			effectiveDuration: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-456',
			customerName: 'Acme Corp',
			workOrderName: 'Project Beta',
			effectiveDuration: 1800,
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
			effectiveDuration: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T14:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			effectiveDuration: null,
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
			effectiveDuration: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T10:00:00Z',
			workOrderId: 'wo-2',
			customerName: 'Acme Corp',
			workOrderName: 'Project B',
			effectiveDuration: 1800,
		});

		const session3 = makeSession({
			id: 'sess-3',
			startTime: '2026-04-21T11:00:00Z',
			workOrderId: 'wo-3',
			customerName: 'Other Inc',
			workOrderName: 'Project C',
			effectiveDuration: 900,
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
			effectiveDuration: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T10:00:00Z',
			workOrderId: 'wo-2',
			customerName: 'Acme Corp',
			workOrderName: 'Project B',
			effectiveDuration: 1800,
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
			effectiveDuration: 1000,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T10:00:00Z',
			workOrderId: 'wo-a',
			customerName: 'Acme Corp',
			workOrderName: 'Alpha Task',
			effectiveDuration: 2000,
		});

		const session3 = makeSession({
			id: 'sess-3',
			startTime: '2026-04-21T11:00:00Z',
			workOrderId: 'wo-b',
			customerName: 'Acme Corp',
			workOrderName: 'Beta Task',
			effectiveDuration: 3000,
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
			effectiveDuration: 3600,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: 'Project Alpha',
			effectiveDuration: 1800,
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
			effectiveDuration: 3600,
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
				effectiveDuration: 3600,
			}),
			// Day 2026-04-20, Customer Acme, WO Beta
			makeSession({
				id: 'sess-2',
				startTime: '2026-04-20T10:30:00Z',
				workOrderId: 'wo-beta',
				customerName: 'Acme Corp',
				workOrderName: 'Beta Task',
				customerColor: '#FF0000',
				effectiveDuration: 5400,
			}),
			// Day 2026-04-20, Customer Other, WO Gamma
			makeSession({
				id: 'sess-3',
				startTime: '2026-04-20T14:00:00Z',
				workOrderId: 'wo-gamma',
				customerName: 'Other Inc',
				workOrderName: 'Gamma Task',
				customerColor: '#0000FF',
				effectiveDuration: 1800,
			}),
			// Day 2026-04-21, Customer Acme, WO Alpha (same WO, different day)
			makeSession({
				id: 'sess-4',
				startTime: '2026-04-21T09:00:00Z',
				workOrderId: 'wo-alpha',
				customerName: 'Acme Corp',
				workOrderName: 'Alpha Task',
				customerColor: '#FF0000',
				effectiveDuration: 2700,
			}),
			// Day 2026-04-21, Customer Other, WO Gamma (same WO, different day)
			makeSession({
				id: 'sess-5',
				startTime: '2026-04-21T15:00:00Z',
				workOrderId: 'wo-gamma',
				customerName: 'Other Inc',
				workOrderName: 'Gamma Task',
				customerColor: '#0000FF',
				effectiveDuration: null, // null duration
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
			effectiveDuration: 3600,
		});

		// Should not throw; grouping should still work
		const result = groupSessionsByDay([session]);

		expect(result).toHaveLength(1);
		expect(result[0].customers).toHaveLength(1);
		// null customerName should be kept as-is or grouped under "null" key
		expect(result[0].customers[0].customerName).toBeNull();
	});

	it('TC-GROUP-15: Handles sessions with null workOrderName gracefully', () => {
		const session = makeSession({
			id: 'sess-1',
			startTime: '2026-04-21T09:00:00Z',
			workOrderId: 'wo-123',
			customerName: 'Acme Corp',
			workOrderName: null as any,
			effectiveDuration: 3600,
		});

		// Should not throw
		const result = groupSessionsByDay([session]);

		expect(result).toHaveLength(1);
		expect(result[0].customers).toHaveLength(1);
		expect(result[0].customers[0].workOrders).toHaveLength(1);
		expect(result[0].customers[0].workOrders[0].workOrderName).toBeNull();
	});

	it('TC-GROUP-16: Multiple sessions with same day but different session IDs sum correctly', () => {
		const sessions: Session[] = [
			makeSession({
				id: 'sess-a',
				startTime: '2026-04-21T09:00:00Z',
				workOrderId: 'wo-1',
				customerName: 'Customer A',
				workOrderName: 'Task 1',
				effectiveDuration: 1000,
			}),
			makeSession({
				id: 'sess-b',
				startTime: '2026-04-21T10:00:00Z',
				workOrderId: 'wo-1',
				customerName: 'Customer A',
				workOrderName: 'Task 1',
				effectiveDuration: 2000,
			}),
			makeSession({
				id: 'sess-c',
				startTime: '2026-04-21T11:00:00Z',
				workOrderId: 'wo-1',
				customerName: 'Customer A',
				workOrderName: 'Task 1',
				effectiveDuration: 3000,
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
			effectiveDuration: 100,
		});

		const session2 = makeSession({
			id: 'sess-2',
			startTime: '2026-04-21T23:59:59Z',
			workOrderId: 'wo-1',
			customerName: 'Acme',
			workOrderName: 'Task 1',
			effectiveDuration: 200,
		});

		const result = groupSessionsByDay([session1, session2]);

		// Both should be in same day group
		expect(result).toHaveLength(1);
		expect(result[0].date).toBe('2026-04-21');
		expect(result[0].customers[0].workOrders[0].sessionCount).toBe(2);
	});
});
