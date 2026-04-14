<script lang="ts">
  import { timer } from '$lib/stores/timer.svelte';
  import { widgetStore } from '$lib/stores/widget.svelte';
  import { toggleWidgetMode } from '$lib/api/window';
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
</script>

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
    <div class="work-order" title={timer.active.workOrderName}>
      {timer.active.workOrderName}
    </div>
    <div class="customer" title={timer.active.customerName}>
      {#if timer.active.customerColor}
        <span class="dot" style="background: {timer.active.customerColor}"></span>
      {/if}
      {timer.active.customerName}
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

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
    display: inline-block;
  }

  .not-tracking {
    font-size: 13px;
    color: var(--text-muted);
  }
</style>
