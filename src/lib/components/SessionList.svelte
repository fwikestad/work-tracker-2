<script lang="ts">
  import { sessionsStore } from '$lib/stores/sessions.svelte';
  import { timer } from '$lib/stores/timer.svelte';
  import { updateSession, deleteSession } from '$lib/api/sessions';
  import { formatTime, formatHuman, parseTimestamp } from '$lib/utils/formatters';
  import type { Session } from '$lib/types';

  type EditState = { 
    id: string; 
    startTime: string;
    endTime: string;
    notes: string; 
    duration: string; 
    activityType: string;
  };
  let editState = $state<EditState | null>(null);
  let saving = $state(false);
  let validationError = $state<string | null>(null);

  function isRunning(session: Session) {
    return session.id === timer.active?.sessionId;
  }

  // Convert ISO 8601 (UTC) to datetime-local format (YYYY-MM-DDTHH:mm) in local time
  function toDatetimeLocal(isoString: string | null | undefined): string {
    if (!isoString) return '';
    const dt = parseTimestamp(isoString);
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${dt.getFullYear()}-${pad(dt.getMonth() + 1)}-${pad(dt.getDate())}T${pad(dt.getHours())}:${pad(dt.getMinutes())}`;
  }

  // Convert datetime-local (local time, YYYY-MM-DDTHH:mm) to RFC3339 UTC
  function fromDatetimeLocal(localString: string): string {
    if (!localString) return '';
    // Parse components as local time (avoids engine-specific string parsing behavior)
    const [datePart, timePart] = localString.split('T');
    const [year, month, day] = datePart.split('-').map(Number);
    const [hours, minutes] = timePart.split(':').map(Number);
    return new Date(year, month - 1, day, hours, minutes, 0).toISOString();
  }

  function startEdit(session: Session) {
    editState = {
      id: session.id,
      startTime: toDatetimeLocal(session.startTime),
      endTime: toDatetimeLocal(session.endTime),
      notes: session.notes ?? '',
      activityType: session.activityType ?? '',
      duration: session.durationSeconds ? String(Math.round(session.durationSeconds / 60)) : ''
    };
    validationError = null;
  }

  function cancelEdit() {
    editState = null;
    validationError = null;
  }

  async function saveEdit(sessionId: string) {
    if (!editState) return;
    
    // Client-side validation
    validationError = null;
    if (editState.startTime && editState.endTime) {
      const start = new Date(fromDatetimeLocal(editState.startTime));
      const end = new Date(fromDatetimeLocal(editState.endTime));
      if (start >= end) {
        validationError = 'Start time must be before end time';
        return;
      }
    }
    
    saving = true;
    try {
      await updateSession(sessionId, {
        startTime: editState.startTime ? fromDatetimeLocal(editState.startTime) : undefined,
        endTime: editState.endTime ? fromDatetimeLocal(editState.endTime) : undefined,
        notes: editState.notes || undefined,
        activityType: editState.activityType || undefined
      });
      await sessionsStore.refreshWeek();
      editState = null;
      validationError = null;
    } catch (e: any) {
      validationError = e?.message ?? 'Failed to save';
    } finally {
      saving = false;
    }
  }

  async function handleDelete(sessionId: string) {
    if (!confirm('Delete this session?')) return;
    try {
      await deleteSession(sessionId);
      await sessionsStore.refreshWeek();
    } catch (e: any) {
      alert(e?.message ?? 'Failed to delete');
    }
  }

  const hasAnySessions = $derived(sessionsStore.weekSessions.some(d => d.sessions.length > 0));
</script>

<section class="sessions-section">
  <div class="week-nav">
    <button
      class="nav-arrow"
      onclick={() => sessionsStore.setWeekOffset(sessionsStore.weekOffset - 1)}
      aria-label="Previous week"
    >◀</button>
    <span class="week-label">{sessionsStore.selectedWeekLabel}</span>
    <button
      class="nav-arrow"
      onclick={() => sessionsStore.setWeekOffset(sessionsStore.weekOffset + 1)}
      disabled={sessionsStore.weekOffset === 0}
      aria-label="Next week"
    >▶</button>
  </div>

  {#if !hasAnySessions}
    <div class="empty">
      <p>No sessions this week</p>
      <p class="hint">Start tracking with Ctrl+K</p>
    </div>
  {:else}
    <div class="sessions-list">
      {#each sessionsStore.weekSessions as day}
        {#if day.sessions.length > 0}
          <div class="day-header" class:today={day.isToday}>{day.label}</div>
          {#each day.sessions as session}
            {#if editState?.id === session.id}
              <div class="session editing" style="border-left-color: {session.customerColor ?? 'var(--border)'}">
                <div class="edit-form">
                  {#if validationError}
                    <div class="error-banner">{validationError}</div>
                  {/if}
                  <label>
                    <span>Start time</span>
                    <input 
                      type="datetime-local" 
                      bind:value={editState.startTime} 
                      disabled={isRunning(session) || saving}
                      required
                    />
                  </label>
                  <label>
                    <span>End time</span>
                    <input 
                      type="datetime-local" 
                      bind:value={editState.endTime} 
                      disabled={isRunning(session) || saving}
                      required={!!editState.endTime}
                    />
                  </label>
                  {#if isRunning(session)}
                    <div class="hint-note">Stop the session before editing times</div>
                  {/if}
                  <label>
                    <span>Activity type</span>
                    <select bind:value={editState.activityType} disabled={saving}>
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
                    <textarea bind:value={editState.notes} rows="3" placeholder="What did you work on?" disabled={saving}></textarea>
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
                class:running={isRunning(session)}
                role="button"
                tabindex="0"
                onclick={() => startEdit(session)}
                onkeydown={(e) => e.key === 'Enter' && startEdit(session)}
                style="border-left-color: {session.customerColor ?? 'var(--border)'}"
              >
                <div class="session-header">
                  <div class="session-main">
                    {#if isRunning(session)}
                      <span class="state-dot running" title="Running">●</span>
                    {/if}
                    <span class="session-name">{session.workOrderName}</span>
                    {#if session.activityType}
                      <span class="activity-badge">{session.activityType}</span>
                    {/if}
                  </div>
                  <span class="session-duration">{formatHuman(session.durationSeconds ?? 0)}</span>
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

  .week-nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    gap: 8px;
  }

  .week-label {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    flex: 1;
    text-align: center;
  }

  .nav-arrow {
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-muted);
    font-size: 11px;
    cursor: pointer;
    min-width: 28px;
    min-height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    transition: color 0.1s, border-color 0.1s;
  }

  .nav-arrow:hover:not(:disabled) {
    color: var(--text);
    border-color: var(--text-muted);
  }

  .nav-arrow:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .day-header {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    color: var(--text-muted);
    padding: 8px 0 4px;
    margin-top: 4px;
  }

  .day-header:first-child {
    margin-top: 0;
  }

  .day-header.today {
    color: var(--accent);
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

  .session.running {
    border-top-color: var(--accent);
    border-right-color: var(--accent);
    border-bottom-color: var(--accent);
  }

  .state-dot {
    font-size: 10px;
    line-height: 1;
    flex-shrink: 0;
  }

  .state-dot.running {
    color: var(--accent);
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

  .error-banner {
    background: var(--danger);
    color: white;
    padding: 8px 12px;
    border-radius: var(--radius);
    font-size: 13px;
    font-weight: 500;
  }

  .hint-note {
    font-size: 12px;
    color: var(--text-muted);
    font-style: italic;
    padding: 4px 0;
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
    min-height: 44px;
  }

  input:focus,
  select:focus,
  textarea:focus {
    outline: 1px solid var(--accent);
    border-color: var(--accent);
  }

  input:disabled,
  select:disabled,
  textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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
