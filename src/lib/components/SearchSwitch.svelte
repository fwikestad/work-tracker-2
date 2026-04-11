<script lang="ts">
  import { sessionsStore } from '$lib/stores/sessions.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { timer } from '$lib/stores/timer.svelte';
  import { listWorkOrders } from '$lib/api/workOrders';
  import { startSession } from '$lib/api/sessions';
  import type { WorkOrder } from '$lib/types';

  let query = $state('');
  let searchResults = $state<WorkOrder[]>([]);
  let searching = $state(false);
  let selectedIndex = $state(0);
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;

  let displayItems = $derived(query.trim() ? searchResults : sessionsStore.recent);

  async function search(q: string) {
    if (!q.trim()) {
      searchResults = [];
      return;
    }
    searching = true;
    try {
      const all = await listWorkOrders();
      const lowerQuery = q.toLowerCase();
      searchResults = all.filter(
        (wo) =>
          wo.name.toLowerCase().includes(lowerQuery) ||
          wo.customerName?.toLowerCase().includes(lowerQuery) ||
          wo.code?.toLowerCase().includes(lowerQuery)
      );
    } finally {
      searching = false;
    }
  }

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    query = target.value;
    selectedIndex = 0;

    if (searchTimeout) clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => search(query), 150);
  }

  async function switchTo(workOrderId: string) {
    try {
      await startSession(workOrderId);
      await timer.refresh();
      await sessionsStore.refreshAll();
      query = '';
      searchResults = [];
    } catch (e: any) {
      alert(e?.message ?? 'Failed to switch');
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, displayItems.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === 'Enter' && displayItems[selectedIndex]) {
      e.preventDefault();
      switchTo(displayItems[selectedIndex].id);
    }
  }
</script>

<section class="search-section">
  <div class="search-header">
    <h3>{query.trim() ? 'Search results' : 'Recent'}</h3>
  </div>

  <input
    type="text"
    class="search-input"
    placeholder="Search work orders... (Ctrl+K)"
    value={query}
    oninput={handleInput}
    onkeydown={handleKeydown}
  />

  {#if displayItems.length === 0}
    <div class="empty">
      {#if query.trim()}
        <p>No work orders found</p>
      {:else}
        <p>No recent work orders</p>
        <p class="hint">Press Ctrl+N to add one</p>
      {/if}
    </div>
  {:else}
    <div class="results">
      {#each displayItems as item, i}
        {@const isActive = timer.active?.workOrderId === item.id}
        <button
          class="result-item"
          class:selected={i === selectedIndex}
          class:active={isActive}
          onclick={() => switchTo(item.id)}
        >
          <div class="item-main">
            <span class="item-name">{item.name}</span>
            {#if isActive}
              <span class="badge">Active</span>
            {/if}
          </div>
          <div class="item-customer">
            {#if item.customerColor}
              <span class="dot" style="background: {item.customerColor}"></span>
            {/if}
            {item.customerName}
          </div>
        </button>
      {/each}
    </div>
  {/if}
</section>

<style>
  .search-section {
    background: var(--surface);
    padding: 16px;
  }

  .search-header {
    margin-bottom: 12px;
  }

  h3 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
  }

  .search-input {
    width: 100%;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text);
    padding: 10px;
    font-family: inherit;
    font-size: 14px;
    margin-bottom: 12px;
    min-height: 44px;
  }

  .search-input:focus {
    outline: 1px solid var(--accent);
    border-color: var(--accent);
  }

  .results {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 320px;
    overflow-y: auto;
  }

  .result-item {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 10px 12px;
    text-align: left;
    cursor: pointer;
    font-family: inherit;
    color: var(--text);
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-height: 44px;
  }

  .result-item:hover,
  .result-item.selected {
    background: #1f1f1f;
    border-color: #333;
  }

  .result-item.active {
    border-color: var(--accent);
  }

  .item-main {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .item-name {
    font-size: 14px;
    font-weight: 500;
    flex: 1;
  }

  .badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 3px;
    background: var(--accent);
    color: white;
    font-weight: 600;
  }

  .item-customer {
    font-size: 12px;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    display: inline-block;
  }

  .empty {
    text-align: center;
    padding: 32px 16px;
  }

  .empty p {
    color: var(--text-muted);
    font-size: 14px;
  }

  .empty .hint {
    font-size: 12px;
    margin-top: 4px;
  }
</style>
