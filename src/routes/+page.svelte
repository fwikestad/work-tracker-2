<script lang="ts">
  import Timer from '$lib/components/Timer.svelte';
  import SearchSwitch from '$lib/components/SearchSwitch.svelte';
  import DailySummary from '$lib/components/DailySummary.svelte';
  import SessionList from '$lib/components/SessionList.svelte';
  import { onMount } from 'svelte';
  import { timer } from '$lib/stores/timer.svelte';
  import { sessionsStore } from '$lib/stores/sessions.svelte';

  let summaryRef: DailySummary;

  onMount(() => {
    summaryRef?.refresh();
  });

  $effect(() => {
    // Refresh summary whenever timer or sessions change
    if (timer.active || sessionsStore.todays.length) {
      summaryRef?.refresh();
    }
  });
</script>

<div class="app">
  <nav class="nav">
    <a href="/" class="nav-btn">Track</a>
    <a href="/manage" class="nav-btn">Manage</a>
  </nav>

  <div class="main-view">
    <Timer />
    <SearchSwitch />
    <DailySummary bind:this={summaryRef} />
    <SessionList />
  </div>

  <footer class="shortcuts">
    <span>Ctrl+N Quick add</span>
    <span>Ctrl+K Search</span>
    <span>Ctrl+S Stop</span>
    <span>P Pause</span>
    <span>R Resume</span>
    <span>Esc Cancel</span>
  </footer>
</div>

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
