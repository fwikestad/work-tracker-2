<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';
  import { timer } from '$lib/stores/timer.svelte';
  import { sessionsStore } from '$lib/stores/sessions.svelte';
  import { recoverSession, discardOrphanSession } from '$lib/api/sessions';
  import { formatTime } from '$lib/utils/formatters';

  let recovering = $state(false);
  let discarding = $state(false);

  async function handleRecover() {
    if (!timer.orphan) return;
    recovering = true;
    try {
      await recoverSession(timer.orphan.sessionId);
      timer.setOrphan(null);
      await timer.refresh();
      await sessionsStore.refreshAll();
    } finally {
      recovering = false;
    }
  }

  async function handleDiscard() {
    if (!timer.orphan) return;
    discarding = true;
    try {
      await discardOrphanSession(timer.orphan.sessionId);
      timer.setOrphan(null);
    } finally {
      discarding = false;
    }
  }
</script>

{#if timer.orphan}
  <div class="overlay">
    <div class="dialog">
      <div class="icon">⚠️</div>
      <h2>Unfinished session found</h2>
      <p>
        You were tracking <strong>{timer.orphan.workOrderName}</strong> at
        <strong>{timer.orphan.customerName}</strong> — started at
        {formatTime(timer.orphan.startedAt)}.
      </p>
      <p>What do you want to do?</p>
      <div class="actions">
        <button class="btn-primary" onclick={handleRecover} disabled={recovering || discarding}>
          {recovering ? 'Closing...' : 'Close session now'}
        </button>
        <button class="btn-secondary" onclick={handleDiscard} disabled={recovering || discarding}>
          {discarding ? 'Discarding...' : 'Discard it'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 32px;
    max-width: 420px;
    width: 90%;
    text-align: center;
  }

  .icon {
    font-size: 48px;
    margin-bottom: 16px;
  }

  h2 {
    font-size: 20px;
    font-weight: 600;
    margin-bottom: 16px;
    color: var(--text);
  }

  p {
    color: var(--text-muted);
    margin-bottom: 12px;
    line-height: 1.6;
  }

  p strong {
    color: var(--text);
  }

  .actions {
    margin-top: 24px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .btn-primary,
  .btn-secondary {
    padding: 12px 24px;
    border: none;
    border-radius: var(--radius);
    font-family: inherit;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    min-height: 44px;
  }

  .btn-primary {
    background: var(--accent);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #3d9e6a;
  }

  .btn-secondary {
    background: transparent;
    color: var(--text-muted);
    border: 1px solid var(--border);
  }

  .btn-secondary:hover:not(:disabled) {
    color: var(--text);
    border-color: #333;
  }

  .btn-primary:disabled,
  .btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
