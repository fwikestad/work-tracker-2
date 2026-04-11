<script lang="ts">
  import { sessionsStore } from '$lib/stores/sessions.svelte';
  import { updateSession, deleteSession } from '$lib/api/sessions';
  import { formatTime, formatHuman } from '$lib/utils/formatters';
  import type { Session } from '$lib/types';

  let editingId = $state<string | null>(null);
  let editNotes = $state('');
  let editDuration = $state('');
  let editActivityType = $state('');
  let saving = $state(false);

  function startEdit(session: Session) {
    editingId = session.id;
    editNotes = session.notes ?? '';
    editActivityType = session.activityType ?? '';
    editDuration = session.effectiveDuration ? String(Math.round(session.effectiveDuration / 60)) : '';
  }

  function cancelEdit() {
    editingId = null;
    editNotes = '';
    editDuration = '';
    editActivityType = '';
  }

  async function saveEdit(sessionId: string) {
    saving = true;
    try {
      const durationMins = parseInt(editDuration);
      await updateSession(sessionId, {
        notes: editNotes || undefined,
        activityType: editActivityType || undefined,
        durationOverride: editDuration && !isNaN(durationMins) ? durationMins * 60 : undefined
      });
      await sessionsStore.refreshToday();
      editingId = null;
    } catch (e: any) {
      alert(e?.message ?? 'Failed to save');
    } finally {
      saving = false;
    }
  }

  async function handleDelete(sessionId: string) {
    if (!confirm('Delete this session?')) return;
    try {
      await deleteSession(sessionId);
      await sessionsStore.refreshToday();
    } catch (e: any) {
      alert(e?.message ?? 'Failed to delete');
    }
  }
</script>

<section class="sessions-section">
  <div class="header">
    <h3>Today's sessions</h3>
  </div>

  {#if sessionsStore.todays.length === 0}
    <div class="empty">
      <p>No sessions today</p>
      <p class="hint">Start tracking with Ctrl+K</p>
    </div>
  {:else}
    <div class="sessions-list">
      {#each sessionsStore.todays as session}
        {#if editingId === session.id}
          <div class="session editing" style="border-left-color: {session.customerColor ?? 'var(--border)'}">
            <div class="edit-form">
              <label>
                <span>Duration (minutes)</span>
                <input type="number" bind:value={editDuration} placeholder="Auto" />
              </label>
              <label>
                <span>Activity type</span>
                <select bind:value={editActivityType}>
                  <option value="">—</option>
                  <option value="meeting">Meeting</option>
                  <option value="development">Development</option>
                  <option value="design">Design</option>
                  <option value="review">Review</option>
                  <option value="admin">Admin</option>
                  <option value="other">Other</option>
                </select>
              </label>
              <label>
                <span>Notes</span>
                <textarea bind:value={editNotes} rows="3" placeholder="What did you work on?"></textarea>
              </label>
              <div class="actions">
                <button class="btn-sm btn-primary" onclick={() => saveEdit(session.id)} disabled={saving}>
                  {saving ? 'Saving...' : 'Save'}
                </button>
                <button class="btn-sm btn-secondary" onclick={cancelEdit} disabled={saving}>
                  Cancel
                </button>
              </div>
            </div>
          </div>
        {:else}
          <div
            class="session"
            role="button"
            tabindex="0"
            onclick={() => startEdit(session)}
            onkeydown={(e) => e.key === 'Enter' && startEdit(session)}
            style="border-left-color: {session.customerColor ?? 'var(--border)'}"
          >
            <div class="session-header">
              <div class="session-main">
                <span class="session-name">{session.workOrderName}</span>
                {#if session.activityType}
                  <span class="activity-badge">{session.activityType}</span>
                {/if}
              </div>
              <span class="session-duration">{formatHuman(session.effectiveDuration ?? 0)}</span>
              <button
                class="btn-delete"
                onclick={(e) => {
                  e.stopPropagation();
                  handleDelete(session.id);
                }}
              >
                ×
              </button>
            </div>
            <div class="session-meta">
              {#if session.customerColor}
                <span class="dot" style="background: {session.customerColor}"></span>
              {/if}
              <span>{session.customerName}</span>
              <span class="sep">•</span>
              <span>
                {formatTime(session.startTime)}
                {#if session.endTime}
                  – {formatTime(session.endTime)}
                {/if}
              </span>
            </div>
            {#if session.notes}
              <div class="session-notes">{session.notes}</div>
            {/if}
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</section>

<style>
  .sessions-section {
    background: var(--surface);
    padding: 16px;
  }

  .header {
    margin-bottom: 12px;
  }

  h3 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
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

  .sessions-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .session {
    background: var(--bg);
    border: 1px solid var(--border);
    border-left: 3px solid var(--border);
    border-radius: var(--radius);
    padding: 12px;
    cursor: pointer;
  }

  .session:not(.editing):hover {
    background: #1f1f1f;
    border-color: #333;
  }

  .session.editing {
    cursor: default;
  }

  .session-header {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin-bottom: 4px;
  }

  .session-main {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .session-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text);
  }

  .activity-badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 3px;
    background: var(--border);
    color: var(--text-muted);
    font-weight: 500;
  }

  .session-duration {
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
  }

  .btn-delete {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 20px;
    cursor: pointer;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
  }

  .btn-delete:hover {
    background: var(--danger);
    color: white;
  }

  .session-meta {
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

  .sep {
    color: var(--border);
  }

  .session-notes {
    margin-top: 8px;
    font-size: 13px;
    color: var(--text-muted);
    font-style: italic;
    line-height: 1.5;
  }

  .edit-form {
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

  input,
  select,
  textarea {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text);
    padding: 8px;
    font-family: inherit;
    font-size: 14px;
  }

  input:focus,
  select:focus,
  textarea:focus {
    outline: 1px solid var(--accent);
    border-color: var(--accent);
  }

  textarea {
    resize: vertical;
    min-height: 64px;
  }

  .actions {
    display: flex;
    gap: 8px;
  }

  .btn-sm {
    padding: 8px 16px;
    border: none;
    border-radius: var(--radius);
    font-family: inherit;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
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

  .btn-sm:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
