<script lang="ts">
  import { timer } from '$lib/stores/timer.svelte';
  import { widgetStore } from '$lib/stores/widget.svelte';
  import { sessionsStore } from '$lib/stores/sessions.svelte';
  import { toggleWidgetMode, resizeWidget } from '$lib/api/window';
  import { startSession } from '$lib/api/sessions';

  const WIDGET_W = 320;
  const WIDGET_BASE_H = 100;
  const ITEM_H = 40;
  const DROPDOWN_MAX_ITEMS = 6;

  async function exitWidgetMode() {
    await toggleWidgetMode(false);
    widgetStore.setWidgetMode(false);
  }

  const stateBadge = $derived(
    !timer.isTracking
      ? { icon: '⊘', label: 'Stopped', cls: 'stopped' }
      : { icon: '🟢', label: 'Running', cls: 'running' }
  );

  let dropdownOpen = $state(false);
  let highlightIndex = $state(0);

  function openDropdown() {
    sessionsStore.refreshRecent();
    dropdownOpen = true;
    highlightIndex = 0;
    const itemCount = Math.min(sessionsStore.recent.length || 1, DROPDOWN_MAX_ITEMS);
    resizeWidget(WIDGET_W, WIDGET_BASE_H + itemCount * ITEM_H + 8);
  }

  function closeDropdown() {
    dropdownOpen = false;
    highlightIndex = 0;
    resizeWidget(WIDGET_W, WIDGET_BASE_H);
  }

  function toggleDropdown() {
    if (dropdownOpen) closeDropdown();
    else openDropdown();
  }

  async function switchTo(workOrderId: string) {
    closeDropdown();
    if (timer.active?.workOrderId === workOrderId) return;
    await startSession(workOrderId);
    await timer.refresh();
    await sessionsStore.refreshRecent();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!dropdownOpen) return;
    const items = sessionsStore.recent.slice(0, DROPDOWN_MAX_ITEMS);
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
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="widget">
  <div class="header">
    <span class="badge {stateBadge.cls}" aria-label={stateBadge.label}>
      {stateBadge.icon}
    </span>
    {#if timer.active}
      <span class="customer-name" title={timer.active.customerName}>
        {timer.active.customerName}
      </span>
    {:else}
      <span class="customer-name muted">Not tracking</span>
    {/if}
    <button class="exit-btn" onclick={exitWidgetMode} title="Exit widget mode (Ctrl+Alt+W)">
      ✕
    </button>
  </div>

  <div class="context-switcher">
    <button
      class="context-btn"
      onclick={toggleDropdown}
      aria-expanded={dropdownOpen}
      aria-haspopup="listbox"
      title={timer.active ? 'Switch work order' : 'Start tracking'}
    >
      {#if timer.active}
        <span class="work-order-name" title={timer.active.workOrderName}>
          {timer.active.workOrderName}
        </span>
      {:else}
        <span class="work-order-name placeholder">Start tracking…</span>
      {/if}
      <span class="chevron" class:open={dropdownOpen}>▾</span>
    </button>

    {#if dropdownOpen}
      <div class="backdrop" role="presentation" onmousedown={closeDropdown}></div>
      <div class="dropdown" role="listbox" aria-label="Recent work orders">
        {#each sessionsStore.recent.slice(0, DROPDOWN_MAX_ITEMS) as wo, i}
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
                <span class="dot" style="background: {wo.customerColor}"></span>
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
</div>

<style>
  .widget {
    width: 100%;
    height: auto;
    background: var(--surface);
    display: flex;
    flex-direction: column;
    padding: 10px 12px 8px;
    box-sizing: border-box;
    overflow: visible;
    gap: 4px;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 6px;
    min-height: 28px;
  }

  .badge {
    font-size: 14px;
    flex-shrink: 0;
    line-height: 1;
  }

  .customer-name {
    flex: 1;
    min-width: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .customer-name.muted {
    color: var(--text-muted);
    font-weight: 400;
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
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .exit-btn:hover {
    background: var(--border);
    color: var(--text);
  }

  .context-switcher {
    position: relative;
  }

  .context-btn {
    width: 100%;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 6px 10px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 4px;
    text-align: left;
    min-height: 36px;
  }

  .context-btn:hover {
    background: var(--bg);
  }

  .context-btn:focus-visible {
    outline: 1px solid var(--accent);
  }

  .work-order-name {
    font-size: 13px;
    font-weight: 400;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .work-order-name.placeholder {
    font-style: italic;
    opacity: 0.6;
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
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
    display: inline-block;
  }

  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 49;
    cursor: default;
  }

  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    overflow-y: auto;
    z-index: 50;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    margin-top: 2px;
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
    min-height: 40px;
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

  .empty {
    padding: 10px 12px;
    font-size: 12px;
    color: var(--text-muted);
    text-align: center;
  }
</style>
