/**
 * SearchableSelect — Unit Tests
 *
 * Tests the core filtering logic for sticky options (value === '__new__').
 * Full component tests require @testing-library/svelte (deferred to a later phase).
 *
 * Covers:
 *   1. Sticky options always visible regardless of filter query
 *   2. Dynamic label shows '+ Create "X"' when filterQuery is "X"
 *   3. Enter auto-selects sticky when no regular matches
 *   4. newQuery is populated with the typed text when __new__ is selected
 */

import { describe, it, expect } from 'vitest';

// ---------------------------------------------------------------------------
// Mirror the SearchableSelect filter logic for unit testing
// ---------------------------------------------------------------------------

interface Option {
  value: string;
  label: string;
  color?: string | null;
}

const NEW_SENTINEL = '__new__';
const BASE_STICKY: Option = { value: NEW_SENTINEL, label: '+ New customer' };

function getRegularOptions(options: Option[]): Option[] {
  return options.filter((o) => o.value !== NEW_SENTINEL);
}

function getStickyOption(options: Option[]): Option | undefined {
  return options.find((o) => o.value === NEW_SENTINEL);
}

function filterRegular(options: Option[], query: string): Option[] {
  const regular = getRegularOptions(options);
  if (!query.trim()) return regular;
  const lower = query.toLowerCase();
  return regular.filter((o) => o.label.toLowerCase().includes(lower));
}

function buildVisibleOptions(options: Option[], filterQuery: string): Option[] {
  const regular = filterRegular(options, filterQuery);
  const sticky = getStickyOption(options);
  if (!sticky) return regular;
  const resolvedSticky: Option = filterQuery.trim()
    ? { ...sticky, label: `+ Create "${filterQuery.trim()}"` }
    : sticky;
  return [...regular, resolvedSticky];
}

function simulateEnter(
  options: Option[],
  filterQuery: string,
  highlightIndex: number
): { selectedValue: string; newQuery: string } {
  const regular = filterRegular(options, filterQuery);
  const sticky = getStickyOption(options);
  const visible = buildVisibleOptions(options, filterQuery);

  let selectedOpt: Option | undefined;
  if (regular.length === 0 && sticky) {
    selectedOpt = sticky;
  } else {
    selectedOpt = visible[highlightIndex];
  }

  const newQuery = selectedOpt?.value === NEW_SENTINEL ? filterQuery.trim() : '';
  return { selectedValue: selectedOpt?.value ?? '', newQuery };
}

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------

const SAMPLE_OPTIONS: Option[] = [
  { value: 'c1', label: 'Acme Corp' },
  { value: 'c2', label: 'GlobalTech' },
  { value: 'c3', label: 'Startup Inc' },
  BASE_STICKY
];

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('SearchableSelect — sticky option visibility', () => {
  it('SS-01: sticky option always included even when no regular matches', () => {
    const visible = buildVisibleOptions(SAMPLE_OPTIONS, 'zzznomatch');
    const hasSticky = visible.some((o) => o.value === NEW_SENTINEL);
    expect(hasSticky).toBe(true);
  });

  it('SS-02: sticky option always included when filter is empty', () => {
    const visible = buildVisibleOptions(SAMPLE_OPTIONS, '');
    const hasSticky = visible.some((o) => o.value === NEW_SENTINEL);
    expect(hasSticky).toBe(true);
  });

  it('SS-03: sticky option always included alongside matching regular options', () => {
    const visible = buildVisibleOptions(SAMPLE_OPTIONS, 'acme');
    expect(visible.some((o) => o.value === 'c1')).toBe(true);
    expect(visible.some((o) => o.value === NEW_SENTINEL)).toBe(true);
  });

  it('SS-04: regular options are still filtered normally', () => {
    const visible = buildVisibleOptions(SAMPLE_OPTIONS, 'acme');
    const regular = visible.filter((o) => o.value !== NEW_SENTINEL);
    expect(regular).toHaveLength(1);
    expect(regular[0].label).toBe('Acme Corp');
  });

  it('SS-05: sticky option appears last in the list', () => {
    const visible = buildVisibleOptions(SAMPLE_OPTIONS, 'acme');
    expect(visible[visible.length - 1].value).toBe(NEW_SENTINEL);
  });
});

describe('SearchableSelect — dynamic sticky label', () => {
  it('SS-06: sticky label shows default text when filter is empty', () => {
    const visible = buildVisibleOptions(SAMPLE_OPTIONS, '');
    const sticky = visible.find((o) => o.value === NEW_SENTINEL);
    expect(sticky?.label).toBe('+ New customer');
  });

  it('SS-07: sticky label shows "+ Create \\"X\\"" when filterQuery is "X"', () => {
    const visible = buildVisibleOptions(SAMPLE_OPTIONS, 'Acme Corp');
    const sticky = visible.find((o) => o.value === NEW_SENTINEL);
    expect(sticky?.label).toBe('+ Create "Acme Corp"');
  });

  it('SS-08: sticky label trims whitespace from filterQuery', () => {
    const visible = buildVisibleOptions(SAMPLE_OPTIONS, '  New Co  ');
    const sticky = visible.find((o) => o.value === NEW_SENTINEL);
    expect(sticky?.label).toBe('+ Create "New Co"');
  });

  it('SS-09: sticky label reverts to default when filter is cleared', () => {
    const withQuery = buildVisibleOptions(SAMPLE_OPTIONS, 'Acme');
    const withEmpty = buildVisibleOptions(SAMPLE_OPTIONS, '');
    const stickyAfter = withEmpty.find((o) => o.value === NEW_SENTINEL);
    expect(stickyAfter?.label).toBe('+ New customer');
    // Ensure the original options object is not mutated
    const originalSticky = SAMPLE_OPTIONS.find((o) => o.value === NEW_SENTINEL);
    expect(originalSticky?.label).toBe('+ New customer');
    void withQuery;
  });
});

describe('SearchableSelect — Enter key auto-selects sticky when no regular matches', () => {
  it('SS-10: Enter with no regular matches auto-selects __new__', () => {
    const result = simulateEnter(SAMPLE_OPTIONS, 'zzznomatch', 0);
    expect(result.selectedValue).toBe(NEW_SENTINEL);
  });

  it('SS-11: Enter with no regular matches populates newQuery with filter text', () => {
    const result = simulateEnter(SAMPLE_OPTIONS, 'Brand New Co', 0);
    expect(result.newQuery).toBe('Brand New Co');
  });

  it('SS-12: Enter with regular matches selects highlighted option (not sticky)', () => {
    const result = simulateEnter(SAMPLE_OPTIONS, 'acme', 0);
    expect(result.selectedValue).toBe('c1');
    expect(result.newQuery).toBe('');
  });

  it('SS-13: Enter on highlighted sticky option populates newQuery', () => {
    // 1 regular match + sticky at index 1 → user arrows down to sticky
    const result = simulateEnter(SAMPLE_OPTIONS, 'acme', 1);
    expect(result.selectedValue).toBe(NEW_SENTINEL);
    expect(result.newQuery).toBe('acme');
  });
});

describe('SearchableSelect — no sticky option present (plain select)', () => {
  const plainOptions: Option[] = [
    { value: 'a', label: 'Alpha' },
    { value: 'b', label: 'Beta' }
  ];

  it('SS-14: filter works normally with no sticky option', () => {
    const visible = buildVisibleOptions(plainOptions, 'alpha');
    expect(visible).toHaveLength(1);
    expect(visible[0].label).toBe('Alpha');
  });

  it('SS-15: no match returns empty array when no sticky present', () => {
    const visible = buildVisibleOptions(plainOptions, 'zzz');
    expect(visible).toHaveLength(0);
  });
});
