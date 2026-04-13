/**
 * SearchSwitch — Unit & Performance Tests
 *
 * The SearchSwitch Svelte component contains filter/sort logic inline and
 * depends on Tauri APIs + Svelte 5 runes, so full component testing requires
 * @testing-library/svelte (deferred to a later phase).
 *
 * This file tests:
 *   1. Pure filter function behaviour (replicated from SearchSwitch.svelte)
 *   2. Phase 2 sorting spec — favorites before recents (spec for Leia)
 *   3. Performance: filter 1 000 work orders within 50 ms target
 *
 * Component-level manual test cases are documented in docs/test-plan.md
 * under "## Phase 2 Test Cases" (TC-P2-011 through TC-P2-020).
 */

import { describe, it, expect } from 'vitest';
import type { WorkOrder } from '$lib/types';

// ---------------------------------------------------------------------------
// Helpers — mirror the filter logic from SearchSwitch.svelte
// ---------------------------------------------------------------------------

/** Replicated from SearchSwitch.svelte — must stay in sync if component changes. */
function filterWorkOrders(all: WorkOrder[], query: string): WorkOrder[] {
  const lowerQuery = query.toLowerCase();
  return all.filter(
    (wo) =>
      wo.name.toLowerCase().includes(lowerQuery) ||
      wo.customerName?.toLowerCase().includes(lowerQuery)
  );
}

/**
 * Phase 2 sorting spec — favourites first, then recents, then rest.
 * SearchSwitch currently shows sessionsStore.recent as-is (no client-side
 * sort). This function describes the DESIRED behaviour Leia should implement.
 *
 * Sort order:
 *   1. isFavorite === true, ordered by most-recently-used (id desc as proxy)
 *   2. isFavorite === false, ordered by most-recently-used (same proxy)
 */
function sortFavoritesFirst(items: WorkOrder[]): WorkOrder[] {
  return [...items].sort((a, b) => {
    if (a.isFavorite && !b.isFavorite) return -1;
    if (!a.isFavorite && b.isFavorite) return 1;
    return 0; // preserve original (recency) order within each group
  });
}

// ---------------------------------------------------------------------------
// Factory helpers
// ---------------------------------------------------------------------------

function makeWorkOrder(
  id: string,
  name: string,
  customerName: string,
  isFavorite = false
): WorkOrder {
  return {
    id,
    customerId: `cust-${id}`,
    customerName,
    customerColor: null,
    name,
    code: null,
    description: null,
    status: 'active',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    archivedAt: null,
    isFavorite,
  };
}

// ---------------------------------------------------------------------------
// Filter behaviour tests
// ---------------------------------------------------------------------------

describe('SearchSwitch — filter logic', () => {
  const workOrders = [
    makeWorkOrder('1', 'API Development', 'GlobalTech'),
    makeWorkOrder('2', 'Website Redesign', 'Acme Corp'),
    makeWorkOrder('3', 'Mobile App', 'GlobalTech'),
    makeWorkOrder('4', 'Admin Tasks', 'Acme Corp'),
    makeWorkOrder('5', 'Database Migration', 'Startup Inc'),
  ];

  it('TC-P2-SEARCH-01: empty query returns all items unmodified', () => {
    const results = filterWorkOrders(workOrders, '');
    // Caller (SearchSwitch) uses sessionsStore.recent on empty query —
    // the filter itself returns everything when called with empty string.
    expect(results).toHaveLength(workOrders.length);
  });

  it('TC-P2-SEARCH-02: query matches work order name (case-insensitive)', () => {
    const results = filterWorkOrders(workOrders, 'api');
    expect(results).toHaveLength(1);
    expect(results[0].name).toBe('API Development');
  });

  it('TC-P2-SEARCH-03: query matches customer name (case-insensitive)', () => {
    const results = filterWorkOrders(workOrders, 'globaltech');
    expect(results).toHaveLength(2);
    expect(results.map((r) => r.name)).toEqual(
      expect.arrayContaining(['API Development', 'Mobile App'])
    );
  });

  it('TC-P2-SEARCH-04: query matches partial customer name', () => {
    const results = filterWorkOrders(workOrders, 'acme');
    expect(results).toHaveLength(2);
    expect(results.map((r) => r.name)).toEqual(
      expect.arrayContaining(['Website Redesign', 'Admin Tasks'])
    );
  });

  it('TC-P2-SEARCH-05: no match returns empty array', () => {
    const results = filterWorkOrders(workOrders, 'zzznomatch');
    expect(results).toHaveLength(0);
  });

  it('TC-P2-SEARCH-06: query is matched against both name and customerName simultaneously', () => {
    // "a" is present in both names and customer names — should return multiple
    const results = filterWorkOrders(workOrders, 'database');
    expect(results).toHaveLength(1);
    expect(results[0].name).toBe('Database Migration');
  });

  it('TC-P2-SEARCH-07: null customerName does not throw', () => {
    const woWithoutCustomer = makeWorkOrder('6', 'Orphan Task', '');
    woWithoutCustomer.customerName = null;
    expect(() => filterWorkOrders([woWithoutCustomer], 'orphan')).not.toThrow();
    const results = filterWorkOrders([woWithoutCustomer], 'orphan');
    expect(results).toHaveLength(1);
  });
});

// ---------------------------------------------------------------------------
// Phase 2 — Favorites-first sort order (spec tests)
// These describe DESIRED behaviour that Leia must implement in SearchSwitch.
// They test the `sortFavoritesFirst` spec function above — if/when the
// component exposes the same logic, these tests should run against that.
// ---------------------------------------------------------------------------

describe('SearchSwitch — Phase 2 favorites sort (spec)', () => {
  it('TC-P2-FAV-01: with no query — favorites appear before recents', () => {
    const items = [
      makeWorkOrder('1', 'Regular A', 'CustA', false),
      makeWorkOrder('2', 'Favorite B', 'CustB', true),
      makeWorkOrder('3', 'Regular C', 'CustC', false),
    ];
    const sorted = sortFavoritesFirst(items);
    expect(sorted[0].name).toBe('Favorite B');
    expect(sorted[0].isFavorite).toBe(true);
    // Non-favorites follow
    expect(sorted[1].isFavorite).toBe(false);
    expect(sorted[2].isFavorite).toBe(false);
  });

  it('TC-P2-FAV-02: with no query — recents appear before non-recent work orders (order preserved)', () => {
    // Items arrive ordered by recency (most-recent first) from sessionsStore.recent.
    // sortFavoritesFirst must preserve that relative order within each group.
    const items = [
      makeWorkOrder('recent-1', 'Recent First', 'Cust', false),
      makeWorkOrder('recent-2', 'Recent Second', 'Cust', false),
      makeWorkOrder('old-1', 'Old One', 'Cust', false),
    ];
    const sorted = sortFavoritesFirst(items);
    // All non-favorites; original order preserved
    expect(sorted[0].id).toBe('recent-1');
    expect(sorted[1].id).toBe('recent-2');
    expect(sorted[2].id).toBe('old-1');
  });

  it('TC-P2-FAV-03: toggling favorite moves starred item to top of list', () => {
    const items = [
      makeWorkOrder('1', 'Item A', 'Cust', false),
      makeWorkOrder('2', 'Item B', 'Cust', false),
      makeWorkOrder('3', 'Item C', 'Cust', false),
    ];
    // Simulate favoriting item C
    items[2] = { ...items[2], isFavorite: true };
    const sorted = sortFavoritesFirst(items);
    expect(sorted[0].id).toBe('3');
    expect(sorted[0].isFavorite).toBe(true);
  });

  it('TC-P2-FAV-04: multiple favorites — all appear before non-favorites', () => {
    const items = [
      makeWorkOrder('1', 'Normal A', 'Cust', false),
      makeWorkOrder('2', 'Fav X', 'Cust', true),
      makeWorkOrder('3', 'Normal B', 'Cust', false),
      makeWorkOrder('4', 'Fav Y', 'Cust', true),
    ];
    const sorted = sortFavoritesFirst(items);
    const favGroup = sorted.filter((i) => i.isFavorite);
    const normalGroup = sorted.filter((i) => !i.isFavorite);
    // All favorites precede all non-favorites
    expect(sorted.indexOf(favGroup[favGroup.length - 1])).toBeLessThan(
      sorted.indexOf(normalGroup[0])
    );
  });

  it('TC-P2-FAV-05: search query — results sorted by name match relevance (favorites ranked higher)', () => {
    // When there IS a query, favorites in the result should still rank first.
    // This tests the combined filter+sort pipeline.
    const all = [
      makeWorkOrder('1', 'API Work', 'GlobalTech', false),
      makeWorkOrder('2', 'API Favorite', 'GlobalTech', true),
      makeWorkOrder('3', 'API Admin', 'GlobalTech', false),
    ];
    const filtered = filterWorkOrders(all, 'api');
    const sorted = sortFavoritesFirst(filtered);
    expect(sorted[0].name).toBe('API Favorite');
    expect(sorted[0].isFavorite).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Performance tests — SearchSwitch filter must respond in <50ms
// ---------------------------------------------------------------------------

describe('SearchSwitch — performance targets', () => {
  /**
   * TC-P2-PERF-01
   * Filtering 1 000 work orders must complete in under 50ms.
   * This validates the <50ms search target from the performance framework.
   */
  it('TC-P2-PERF-01: filter 1000 work orders in <50ms', () => {
    const large: WorkOrder[] = Array.from({ length: 1000 }, (_, i) =>
      makeWorkOrder(
        `id-${i}`,
        `Work Order ${i}`,
        i % 2 === 0 ? 'Acme Corp' : 'GlobalTech'
      )
    );

    const start = performance.now();
    const results = filterWorkOrders(large, 'acme');
    const elapsed = performance.now() - start;

    expect(results.length).toBe(500); // half have "Acme Corp"
    expect(elapsed).toBeLessThan(50);
  });

  /**
   * TC-P2-PERF-02
   * Sorting 1 000 work orders (favorites-first) must complete in under 50ms.
   * Validates that the sort step does not introduce observable latency.
   */
  it('TC-P2-PERF-02: sort 1000 work orders favorites-first in <50ms', () => {
    const large: WorkOrder[] = Array.from({ length: 1000 }, (_, i) =>
      makeWorkOrder(`id-${i}`, `Work Order ${i}`, 'Cust', i % 10 === 0)
    );

    const start = performance.now();
    const sorted = sortFavoritesFirst(large);
    const elapsed = performance.now() - start;

    expect(sorted[0].isFavorite).toBe(true);
    expect(elapsed).toBeLessThan(50);
  });

  /**
   * TC-P2-PERF-03
   * Combined filter + sort pipeline must complete in under 50ms end-to-end.
   * Validates that chaining the two operations stays within the target.
   */
  it('TC-P2-PERF-03: filter + sort pipeline on 1000 items in <50ms', () => {
    const large: WorkOrder[] = Array.from({ length: 1000 }, (_, i) =>
      makeWorkOrder(`id-${i}`, `Order ${i}`, i % 3 === 0 ? 'Acme' : 'Other', i % 15 === 0)
    );

    const start = performance.now();
    const filtered = filterWorkOrders(large, 'acme');
    const sorted = sortFavoritesFirst(filtered);
    const elapsed = performance.now() - start;

    expect(sorted.length).toBeGreaterThan(0);
    expect(elapsed).toBeLessThan(50);
  });
});
