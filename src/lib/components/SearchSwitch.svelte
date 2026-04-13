<script lang="ts">
  import { sessionsStore } from '$lib/stores/sessions.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { timer } from '$lib/stores/timer.svelte';
  import { listWorkOrders, toggleFavorite } from '$lib/api/workOrders';
  import { startSession } from '$lib/api/sessions';
  import type { WorkOrder } from '$lib/types';

  let query = $state('');
  let searchResults = $state<WorkOrder[]>([]);
  let searching = $state(false);
  let selectedIndex = $state(0);
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;
  let searchGen = 0;
  let inputElement: HTMLInputElement | undefined;

  // Expose focus method for parent component
  export function focus() {
    inputElement?.focus();
  }

  // Grouped items for idle view (no query)
  let favs = $derived(sessionsStore.allFavorites);
  let recentGroup = $derived(sessionsStore.recent.filter((wo) => !wo.isFavorite));

  // Flat sorted list for keyboard navigation
  let displayItems = $derived(
    query.trim()
      ? [...searchResults].sort((a, b) => {
          if (a.isFavorite && !b.isFavorite) return -1;
          if (!a.isFavorite && b.isFavorite) return 1;
          return 0;
        })
      : [...favs, ...recentGroup]
  );

  let hasIdleItems = $derived(favs.length > 0 || recentGroup.length > 0);

  async function search(q: string) {
    if (!q.trim()) {
      searchResults = [];
      return;
    }
    const gen = ++searchGen;
    searching = true;
    try {
      const all = await listWorkOrders();
      if (gen !== searchGen) return;
      const lowerQuery = q.toLowerCase();
      searchResults = all.filter(
        (wo) =>
          wo.name.toLowerCase().includes(lowerQuery) ||
          wo.customerName?.toLowerCase().includes(lowerQuery)
      );
    } finally {
      if (gen === searchGen) {
        searching = false;
      }
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
      // Log full error for debugging
      console.error('Switch failed:', e);
      // Show actual error message from backend
      const errorMsg = e?.message || e?.toString() || 'Unknown error occurred';
      alert(`Failed to switch: ${errorMsg}`);
    }
  }

  async function handleToggleFavorite(e: Event, workOrderId: string) {
    e.stopPropagation();
    try {
      await toggleFavorite(workOrderId);
      await sessionsStore.refreshRecent();
      if (query.trim()) {
        await search(query);
      }
    } catch (e: any) {
      console.error('Toggle favorite failed:', e);
      const errorMsg = e?.message || e?.toString() || 'Unknown error occurred';
      alert(`Failed to toggle favorite: ${errorMsg}`);
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
  <input
    bind:this={inputElement}
    type="text"
    class="search-input"
    placeholder="Search work orders... (Ctrl+K)"
    value={query}
    oninput={handleInput}
    onkeydown={handleKeydown}
  />

  {#if query.trim()}
    <!-- Search results: flat list, favorites sorted first -->
    {#if displayItems.length === 0}
      <div class="empty">
        <p>No work orders found</p>
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
              <span
                class="star-btn"
                role="button"
                tabindex="0"
                onclick={(e) => handleToggleFavorite(e, item.id)}
                onkeydown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    handleToggleFavorite(e, item.id);
                  }
                }}
                title={item.isFavorite ? 'Remove from favorites' : 'Add to favorites'}
              >
                {item.isFavorite ? '⭐' : '☆'}
              </span>
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
  {:else}
    <!-- Idle grouped view -->
    {#if !hasIdleItems}
      <div class="empty">
        <p>No work orders yet</p>
        <p class="hint">Press Ctrl+N to add one</p>
      </div>
    {:else}
      <div class="results">
        {#if favs.length > 0}
          <div class="group-header">⭐ Favorites</div>
          {#each favs as item, i}
            {@const isActive = timer.active?.workOrderId === item.id}
            <button
              class="result-item"
              class:selected={selectedIndex === i}
              class:active={isActive}
              onclick={() => switchTo(item.id)}
            >
              <div class="item-main">
                <span
                  class="star-btn"
                  role="button"
                  tabindex="0"
                  onclick={(e) => handleToggleFavorite(e, item.id)}
                  onkeydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      handleToggleFavorite(e, item.id);
                    }
                  }}
                  title="Remove from favorites"
                >⭐</span>
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
        {/if}

        {#if recentGroup.length > 0}
          <div class="group-header">🕐 Recent</div>
          {#each recentGroup as item, j}
            {@const isActive = timer.active?.workOrderId === item.id}
            {@const idx = favs.length + j}
            <button
              class="result-item"
              class:selected={selectedIndex === idx}
              class:active={isActive}
              onclick={() => switchTo(item.id)}
            >
              <div class="item-main">
                <span
                  class="star-btn"
                  role="button"
                  tabindex="0"
                  onclick={(e) => handleToggleFavorite(e, item.id)}
                  onkeydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      handleToggleFavorite(e, item.id);
                    }
                  }}
                  title="Add to favorites"
                >☆</span>
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
        {/if}
      </div>
    {/if}
  {/if}
</section>

<style>
  .search-section {
    background: var(--surface);
    padding: 16px;
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
    box-sizing: border-box;
  }

  .search-input:focus {
    outline: 1px solid var(--accent);
    border-color: var(--accent);
  }

  .group-header {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    padding: 8px 4px 4px;
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

  .star-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 14px;
    cursor: pointer;
    padding: 0;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    flex-shrink: 0;
  }

  .star-btn:hover {
    color: #fbbf24;
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
    margin-left: 28px;
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
