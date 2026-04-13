# Skill: Vitest Pure Function Extraction for Component Logic

**Category**: Testing  
**Author**: Wedge (Tester)  
**Created**: 2026-04-12  
**Applies to**: Svelte 5 + Vitest projects where component logic is not directly exportable

---

## Problem

Svelte 5 components using `$state`, `$derived`, and `$effect` runes **cannot be imported directly in Vitest**. The runes require a Svelte component context that doesn't exist in a plain Node/jsdom test environment.

This means filter logic, sort logic, and other business rules buried inside `.svelte` files are untestable without either:
1. Setting up `@testing-library/svelte` (heavier infrastructure)
2. Moving to Phase 2+ where component testing is formally set up

---

## Solution: Replicate Pure Logic in Test File

For **stateless filter/sort/transform logic** embedded in a Svelte component, replicate the algorithm as a pure function directly in the test file. Test the pure function. Add a comment noting the original source location.

### Example (from SearchSwitch.svelte)

**In the component** (not exported):
```typescript
// SearchSwitch.svelte — inline filter, not exported
searchResults = all.filter(
  (wo) =>
    wo.name.toLowerCase().includes(lowerQuery) ||
    wo.customerName?.toLowerCase().includes(lowerQuery)
);
```

**In the test file** (replicated):
```typescript
// src/lib/components/SearchSwitch.test.ts
// Replicated from SearchSwitch.svelte — must stay in sync if component changes.
function filterWorkOrders(all: WorkOrder[], query: string): WorkOrder[] {
  const lowerQuery = query.toLowerCase();
  return all.filter(
    (wo) =>
      wo.name.toLowerCase().includes(lowerQuery) ||
      wo.customerName?.toLowerCase().includes(lowerQuery)
  );
}
```

---

## When to Use This Pattern

✅ **Good candidates** (pure, stateless, deterministic):
- Filter functions: `items.filter(predicate)`
- Sort functions: `items.sort(comparator)`
- Transform/map functions: `items.map(transform)`
- Validation functions: `validate(value) => boolean | string`
- Format functions: `formatDuration(seconds) => string`

❌ **Not suitable** (requires component context):
- Functions that read `$state` variables
- Functions that call `$derived` values
- Event handlers that trigger side effects
- Functions that call external APIs (`invoke()`, `fetch()`)

---

## Performance Testing Pattern

Use `performance.now()` for inline timing assertions. Works reliably in Vitest's jsdom environment.

```typescript
it('filter 1000 items in <50ms', () => {
  const large = Array.from({ length: 1000 }, (_, i) => makeItem(i));
  
  const start = performance.now();
  const results = filterWorkOrders(large, 'query');
  const elapsed = performance.now() - start;
  
  expect(results.length).toBeGreaterThan(0);
  expect(elapsed).toBeLessThan(50); // 50ms budget
});
```

**Practical note**: In-process pure JS filters on 1,000 items typically complete in 0.1–2ms. The 50ms assertion is conservative and won't flake on slow CI machines.

---

## Maintenance Note

The replication creates a sync obligation. If the component's filter logic changes, the test file's pure function must be updated.

Mitigate this by:
1. Adding a prominent comment: `// Replicated from SearchSwitch.svelte — must stay in sync if component changes.`
2. Moving to exported utility functions when the component grows: `src/lib/utils/filterWorkOrders.ts`
3. Adding the utility to CI type-checking to catch divergence

---

## Migration Path to Full Component Testing

Once `@testing-library/svelte` is set up:

1. Move pure functions to `src/lib/utils/` (if not already there)
2. Update import in component and test file
3. Add component-level tests that test behaviour through the rendered DOM
4. Keep pure function tests for performance regression coverage

---

## Related Files

- `src/lib/components/SearchSwitch.test.ts` — Example application of this skill
- `src/lib/components/SearchSwitch.svelte` — Source of replicated logic
- `src/lib/stores/timer.test.ts` — Counter-example (cannot use this pattern due to `$effect` side effects)
