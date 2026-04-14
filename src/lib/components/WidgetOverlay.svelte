<script lang="ts">
  import { timer } from '$lib/stores/timer.svelte';
  import { widgetStore } from '$lib/stores/widget.svelte';
  import { sessionsStore } from '$lib/stores/sessions.svelte';
  import { toggleWidgetMode } from '$lib/api/window';
  import { startSession } from '$lib/api/sessions';
  import { formatDuration } from '$lib/utils/formatters';

  async function exitWidgetMode() {
    await toggleWidgetMode(false);
    widgetStore.setWidgetMode(false);
  }

  const stateBadge = $derived(
    !timer.isTracking
      ? { icon: '⊘', label: 'Stopped', cls: 'stopped' }
      : timer.isPaused
        ? { icon: '🟡', label: 'Paused', cls: 'paused' }
        : { icon: '🟢', label: 'Running', cls: 'running' }
  );

  // Context-switch dropdown
  let dropdownOpen = $state(false);
  let highlightIndex = $state(0);
  let switcherRef = $state<HTMLDivElement | undefined>(undefined);

  function openDropdown() {
    sessionsStore.refreshRecent();
    dropdownOpen = true;
    highlightIndex = 0;
  }

  function closeDropdown() {
    dropdownOpen = false;
    highlightIndex = 0;
  }

  async function switchTo(workOrderId: string) {
    closeDropdown();
    if (timer.active?.workOrderId === workOrderId) return;
    await startSession(workOrderId);
    await timer.refresh();
    await sessionsStore.refreshRecent();
  }

  function handleClickOutside(e: MouseEvent) {
    if (switcherRef && !switcherRef.contains(e.target as Node)) {
      closeDropdown();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!dropdownOpen) return;
    const items = sessionsStore.recent.slice(0, 6);
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlightIndex = Math.min(highlightIndex + 1, items.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlightIndex = Math.max(highlightIndex - 1, 0);
    } else if (e.key === 'Enter' && items[highlightIndex]) {
      e.preventDefault();
      switchTo(items[highlightIndex].id);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      closeDropdown();
    }
  }

  $effect(() => {
    if (dropdownOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="widget">
  <div class="header">
    <span class="badge {stateBadge.cls}" aria-label={stateBadge.label}>
      {stateBadge.icon} {stateBadge.label}
    </span>
    <button class="exit-btn" onclick={exitWidgetMode} title="Exit widget mode (Ctrl+Alt+W)">
      ✕
    </button>
  </div>

  <div class="elapsed" class:dim={!timer.isTracking}>
    {formatDuration(timer.elapsed)}
  </div>

  {#if timer.active}
    <div class="context-switcher" bind:this={switcherRef}>
      <button
        class="context-btn"
        onclick={openDropdown}
        aria-expanded={dropdownOpen}
        aria-haspopup="listbox"
        title="Switch work order"
      >
        <span class="context-text">
          <span class="work-order" title={timer.active.workOrderName}>
            {timer.active.workOrderName}
          </span>
          <span class="customer" title={timer.active.customerName}>
            {#if timer.active.customerColor}
              <span class="dot" style="background: {timer.active.customerColor}"></span>
            {/if}
            {timer.active.customerName}
          </span>
        </span>
        <span class="chevron" class:open={dropdownOpen}>▾</span>
      </button>

      {#if dropdownOpen}
        <div class="dropdown" role="listbox" aria-label="Recent work orders">
          {#each sessionsStore.recent.slice(0, 6) as wo, i}
            <button
              class="dropdown-item"
              class:active={wo.id === timer.active?.workOrderId}
              class:highlighted={i === highlightIndex}
              role="option"
              aria-selected={wo.id === timer.active?.workOrderId}
              onclick={() => switchTo(wo.id)}
            >
              <span class="item-work-order">{wo.name}</span>
              <span class="item-customer">
                {#if wo.customerColor}
                  <span class="dot small" style="background: {wo.customerColor}"></span>
                {/if}
                {wo.customerName}
              </span>
            </button>
          {/each}
          {#if sessionsStore.recent.length === 0}
            <div class="empty">No recent work orders</div>
          {/if}
        </div>
      {/if}
    </div>
  {:else}
    <div class="not-tracking">Not tracking</div>
  {/if}
</div>

<style>
  .widget {
    width: 100%;
    height: 100vh;
    background: var(--surface);
    display: flex;
    flex-direction: column;
    padding: 10px 12px 8px;
    box-sizing: border-box;
    overflow: hidden;
    gap: 2px;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 26px;
  }

  .badge {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 3px;
    text-transform: uppercase;
    letter-spacing: 0.4px;
  }

  .badge.running {
    background: var(--accent);
    color: white;
  }

  .badge.paused {
    background: #f59e0b;
    color: white;
  }

  .badge.stopped {
    background: var(--border);
    color: var(--text-muted);
  }

  .exit-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 13px;
    padding: 4px 6px;
    border-radius: 3px;
    line-height: 1;
    min-height: 26px;
    min-width: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .exit-btn:hover {
    background: var(--border);
    color: var(--text);
  }

  .elapsed {
    font-size: 34px;
    font-weight: 700;
    font-family: 'Consolas', 'Monaco', monospace;
    color: var(--text);
    letter-spacing: -1px;
    line-height: 1.1;
    white-space: nowrap;
  }

  .elapsed.dim {
    color: var(--text-muted);
  }

  /* Context switcher trigger */
  .context-switcher {
    position: relative;
  }

  .context-btn {
    width: 100%;
    background: transparent;
    border: none;
    padding: 2px 4px 2px 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    border-radius: 3px;
    text-align: left;
    min-height: 44px;
  }

  .context-btn:hover {
    background: var(--border);
  }

  .context-btn:focus-visible {
    outline: 1px solid var(--accent);
  }

  .context-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .work-order {
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .customer {
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .chevron {
    font-size: 12px;
    color: var(--text-muted);
    flex-shrink: 0;
    transition: transform 0.15s ease;
    line-height: 1;
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
    display: inline-block;
  }

  /* Dropdown overlays above the widget content (fixed to bottom of viewport) */
  .dropdown {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    background: var(--surface);
    border: 1px solid var(--border);
    border-bottom: none;
    border-radius: 4px 4px 0 0;
    overflow-y: auto;
    z-index: 50;
    box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.4);
  }

  .dropdown-item {
    width: 100%;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--text);
    padding: 6px 12px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 1px;
    text-align: left;
    min-height: 32px;
  }

  .dropdown-item:last-child {
    border-bottom: none;
  }

  .dropdown-item:hover,
  .dropdown-item.highlighted {
    background: var(--bg);
  }

  .dropdown-item.active {
    background: color-mix(in srgb, var(--accent) 15%, transparent);
  }

  .dropdown-item.active .item-work-order {
    color: var(--accent);
  }

  .item-work-order {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-customer {
    font-size: 10px;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dot.small {
    width: 6px;
    height: 6px;
  }

  .empty {
    padding: 10px 12px;
    font-size: 12px;
    color: var(--text-muted);
    text-align: center;
  }

  .not-tracking {
    font-size: 13px;
    color: var(--text-muted);
  }
</style>
