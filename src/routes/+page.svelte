<script lang="ts">
  import Timer from '$lib/components/Timer.svelte';
  import SearchSwitch from '$lib/components/SearchSwitch.svelte';
  import DailySummary from '$lib/components/DailySummary.svelte';
  import SessionList from '$lib/components/SessionList.svelte';
  import ReportView from '$lib/components/ReportView.svelte';
  import WidgetOverlay from '$lib/components/WidgetOverlay.svelte';
  import { onMount, tick } from 'svelte';
  import { timer } from '$lib/stores/timer.svelte';
  import { sessionsStore } from '$lib/stores/sessions.svelte';
  import { widgetStore } from '$lib/stores/widget.svelte';
  import { toggleWidgetMode } from '$lib/api/window';
  import { listen } from '@tauri-apps/api/event';

  let activeView = $state<'track' | 'reports'>('track');
  let summaryRef = $state<DailySummary | null>(null);
  let searchSwitchRef = $state<SearchSwitch | null>(null);
  let togglingWidget = $state(false);

  async function handleWidgetToggle() {
    if (togglingWidget) return;
    togglingWidget = true;
    try {
      const next = !widgetStore.isWidgetMode;
      await toggleWidgetMode(next);
      widgetStore.setWidgetMode(next);
    } finally {
      togglingWidget = false;
    }
  }

  onMount(() => {
    summaryRef?.refresh();
    
    const unlistenReports = listen('open-reports', () => {
      activeView = 'reports';
    });

    const unlistenSwitch = listen('open-search-switch', async () => {
      activeView = 'track';
      await tick();
      searchSwitchRef?.focus();
    });

    // Tray actions (pause/resume, direct work order switch) update DB in Rust
    // but don't go through normal invoke flow — refresh UI state here so the
    // timer, session list, and tray icon all reflect the new state.
    const unlistenTrayAction = listen('tray-action', async () => {
      await timer.refresh();
      await sessionsStore.refreshAll();
    });

    // Global shortcut Ctrl+Alt+W from Rust toggles widget mode
    const unlistenWidget = listen('toggle-widget-mode', async (event) => {
      const enable = event.payload as boolean;
      widgetStore.setWidgetMode(enable);
    });
    
    return () => {
      unlistenReports.then(fn => fn());
      unlistenSwitch.then(fn => fn());
      unlistenTrayAction.then(fn => fn());
      unlistenWidget.then(fn => fn());
    };
  });

  $effect(() => {
    // Refresh summary whenever timer or sessions change
    if (timer.active || sessionsStore.todays.length) {
      summaryRef?.refresh();
    }
  });
</script>

{#if widgetStore.isWidgetMode}
  <WidgetOverlay />
{:else}
<div class="app">
  <nav class="nav">
    <button
      class="nav-btn"
      class:active={activeView === 'track'}
      onclick={() => (activeView = 'track')}
    >
      Track
    </button>
    <button
      class="nav-btn"
      class:active={activeView === 'reports'}
      onclick={() => (activeView = 'reports')}
    >
      Reports
    </button>
    <a href="/manage" class="nav-btn nav-link">Manage</a>
    <button
      class="nav-btn widget-toggle"
      class:widget-active={widgetStore.isWidgetMode}
      onclick={handleWidgetToggle}
      disabled={togglingWidget}
      title="Stay on top (Ctrl+Alt+W)"
      aria-label="Toggle always-on-top widget mode"
      aria-pressed={widgetStore.isWidgetMode}
    >
      📌
    </button>
  </nav>

  <div class="main-view">
    {#if activeView === 'track'}
      <Timer />
      <SearchSwitch bind:this={searchSwitchRef} />
      <DailySummary bind:this={summaryRef} />
      <SessionList />
    {:else if activeView === 'reports'}
      <ReportView />
    {/if}
  </div>

  {#if activeView === 'track'}
    <footer class="shortcuts">
      <span>Ctrl+N Quick add</span>
      <span>Ctrl+K Search</span>
      <span>Ctrl+S Stop</span>
      <span>P Pause</span>
      <span>R Resume</span>
      <span>Esc Cancel</span>
      <span>Ctrl+Alt+W Widget</span>
    </footer>
  {/if}
</div>
{/if}

<style>
  .app {
    max-width: 480px;
    margin: 0 auto;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg);
  }

  .nav {
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    display: flex;
    padding: 0 12px;
    position: sticky;
    top: 0;
    z-index: 100;
  }

  .nav-btn {
    padding: 10px 16px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 13px;
    font-family: inherit;
    min-height: 44px;
    text-decoration: none;
    display: block;
  }

  .nav-btn:hover {
    color: var(--text);
  }

  .nav-btn.active {
    color: var(--text);
    border-bottom-color: var(--accent);
  }

  .nav-link {
    margin-left: auto;
  }

  .widget-toggle {
    font-size: 16px;
    padding: 0 10px;
    opacity: 0.45;
    transition: opacity 0.15s;
  }

  .widget-toggle:hover:not(:disabled) {
    opacity: 1;
  }

  .widget-toggle.widget-active {
    opacity: 1;
    border-bottom-color: var(--accent);
  }

  .widget-toggle:disabled {
    cursor: not-allowed;
  }

  .main-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--border);
  }

  .shortcuts {
    background: var(--surface);
    border-top: 1px solid var(--border);
    padding: 8px 12px;
    display: flex;
    gap: 16px;
    justify-content: center;
    flex-wrap: wrap;
  }

  .shortcuts span {
    font-size: 11px;
    color: var(--text-muted);
  }
</style>
