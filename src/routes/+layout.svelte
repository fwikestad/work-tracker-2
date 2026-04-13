<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { timer } from '$lib/stores/timer.svelte';
  import { sessionsStore } from '$lib/stores/sessions.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { checkForOrphanSession, stopSession } from '$lib/api/sessions';
  import { registerShortcuts } from '$lib/utils/shortcuts';
  import { listen } from '@tauri-apps/api/event';
  import QuickAdd from '$lib/components/QuickAdd.svelte';
  import RecoveryDialog from '$lib/components/RecoveryDialog.svelte';

  onMount(async () => {
    // Initialize app state
    await Promise.all([timer.refresh(), sessionsStore.refreshAll()]);

    // Check for orphan session (crash recovery)
    const orphan = await checkForOrphanSession();
    if (orphan) timer.setOrphan(orphan);

    // Register global keyboard shortcuts
    const unregKb = registerShortcuts({
      onQuickAdd: () => uiStore.openQuickAdd(),
      onSearch: () => uiStore.openSearch(),
      onStop: async () => {
        if (timer.isTracking) {
          await stopSession();
          await timer.refresh();
          await sessionsStore.refreshAll();
        }
      },
      onEscape: () => {
        uiStore.closeQuickAdd();
        uiStore.closeSearch();
      },
      onPause: async () => {
        if (timer.isTracking && !timer.isPaused) {
          await timer.pause();
        }
      },
      onResume: async () => {
        if (timer.isTracking && timer.isPaused) {
          await timer.resume();
        }
      }
    });

    // Listen for global shortcut event from Rust (Ctrl+Shift+S from any app)
    const unlistenSearch = await listen('focus-search', () => {
      uiStore.openSearch();
    });

    return () => {
      unregKb();
      unlistenSearch();
    };
  });
</script>

{#if timer.orphan}
  <RecoveryDialog />
{/if}

{#if uiStore.quickAdd}
  <QuickAdd />
{/if}

<slot />
