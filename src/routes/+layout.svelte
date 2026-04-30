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

  onMount(() => {
    // Register shortcuts synchronously at the top of onMount — before any await
    // so a failing init call cannot prevent shortcuts from being registered.
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
      }
    });

    let unlistenSearch = () => {};
    let disposed = false;

    // Async init runs in a fire-and-forget IIFE so onMount can return
    // a synchronous cleanup function (Svelte 5 ignores async cleanup).
    (async () => {
      try {
        await Promise.all([timer.refresh(), sessionsStore.refreshAll()]);

        const orphan = await checkForOrphanSession();
        if (orphan) timer.setOrphan(orphan);

        // Listen for global shortcut event from Rust (Ctrl+Shift+S from any app)
        const off = await listen('focus-search', () => uiStore.openSearch());
        if (disposed) off();
        else unlistenSearch = off;
      } catch (e) {
        console.error('[layout] init error:', e);
      }
    })();

    return () => {
      disposed = true;
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
