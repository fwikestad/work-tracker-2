<script lang="ts">
  import { timer } from '$lib/stores/timer.svelte';
  import { sessionsStore } from '$lib/stores/sessions.svelte';
  import { stopSession } from '$lib/api/sessions';
  import { formatDuration } from '$lib/utils/formatters';

  let stopping = $state(false);
  let showNotes = $state(false);
  let notes = $state('');
  let activityType = $state('');

  async function handleStop() {
    stopping = true;
    try {
      await stopSession(notes || undefined, activityType || undefined);
      timer.setActive(null);
      await sessionsStore.refreshAll();
      notes = '';
      activityType = '';
      showNotes = false;
    } finally {
      stopping = false;
    }
  }

  function toggleNotes() {
    showNotes = !showNotes;
  }

  async function handlePause() {
    await timer.pause();
  }

  async function handleResume() {
    await timer.resume();
  }
</script>

<section class="timer-section" class:active={timer.isTracking} class:paused={timer.isPaused}>
  {#if timer.isTracking && timer.active}
    <div class="timer-display">
      <div class="status-indicator">
        {#if timer.isPaused}
          <span class="indicator paused">●</span>
          <span class="badge paused">Paused</span>
        {:else}
          <span class="indicator running">●</span>
          <span class="badge running">Running</span>
        {/if}
      </div>
      <div class="elapsed">{formatDuration(timer.elapsed)}</div>
      <div class="work-order">{timer.active.workOrderName}</div>
      <div class="customer">
        {#if timer.active.customerColor}
          <span class="dot" style="background: {timer.active.customerColor}"></span>
        {/if}
        {timer.active.customerName}
      </div>
    </div>

    {#if showNotes}
      <div class="notes-form">
        <label>
          <span>Notes (optional)</span>
          <textarea bind:value={notes} rows="3" placeholder="What did you work on?" />
        </label>
        <label>
          <span>Activity type</span>
          <select bind:value={activityType}>
            <option value="">—</option>
            <option value="meeting">Meeting</option>
            <option value="development">Development</option>
            <option value="design">Design</option>
            <option value="review">Review</option>
            <option value="admin">Admin</option>
            <option value="other">Other</option>
          </select>
        </label>
      </div>
    {/if}

    <div class="actions">
      {#if !showNotes}
        <button class="btn-secondary" onclick={toggleNotes} disabled={stopping}>
          Add details
        </button>
      {/if}
      {#if timer.isPaused}
        <button class="btn-primary" onclick={handleResume} disabled={stopping}>
          ▶ Resume
        </button>
      {:else}
        <button class="btn-secondary" onclick={handlePause} disabled={stopping}>
          ⏸ Pause
        </button>
      {/if}
      <button class="btn-danger" onclick={handleStop} disabled={stopping}>
        {stopping ? 'Stopping...' : 'Stop tracking'}
      </button>
    </div>
  {:else}
    <div class="not-tracking">
      <div class="message">Not tracking</div>
      <div class="hint">Press Ctrl+N or Ctrl+K to start</div>
    </div>
  {/if}
</section>

<style>
  .timer-section {
    background: var(--surface);
    padding: 24px;
    border-left: 3px solid transparent;
  }

  .timer-section.active {
    border-left-color: var(--accent);
  }

  .timer-section.paused {
    border-left-color: #f59e0b;
  }

  .timer-display {
    text-align: left;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .indicator {
    font-size: 14px;
    line-height: 1;
  }

  .indicator.running {
    color: var(--accent);
  }

  .indicator.paused {
    color: #f59e0b;
  }

  .badge {
    font-size: 11px;
    padding: 3px 8px;
    border-radius: 3px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .badge.running {
    background: var(--accent);
    color: white;
  }

  .badge.paused {
    background: #f59e0b;
    color: white;
  }

  .elapsed {
    font-size: 36px;
    font-weight: 700;
    font-family: 'Consolas', 'Monaco', monospace;
    color: var(--text);
    letter-spacing: -1px;
    margin-bottom: 8px;
  }

  .work-order {
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
    margin-bottom: 4px;
  }

  .customer {
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

  .not-tracking {
    text-align: center;
    padding: 24px 0;
  }

  .message {
    font-size: 18px;
    color: var(--text-muted);
    margin-bottom: 8px;
  }

  .hint {
    font-size: 12px;
    color: var(--text-muted);
  }

  .notes-form {
    margin-top: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  label span {
    font-size: 12px;
    color: var(--text-muted);
  }

  textarea,
  select {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text);
    padding: 8px;
    font-family: inherit;
    font-size: 14px;
    resize: vertical;
  }

  textarea:focus,
  select:focus {
    outline: 1px solid var(--accent);
    border-color: var(--accent);
  }

  .actions {
    margin-top: 16px;
    display: flex;
    gap: 8px;
  }

  .btn-primary,
  .btn-secondary,
  .btn-danger {
    flex: 1;
    padding: 10px 16px;
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
    background: var(--border);
    color: var(--text);
  }

  .btn-secondary:hover:not(:disabled) {
    background: #333;
  }

  .btn-danger {
    background: var(--danger);
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    background: #c73e3e;
  }

  .btn-primary:disabled,
  .btn-secondary:disabled,
  .btn-danger:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
