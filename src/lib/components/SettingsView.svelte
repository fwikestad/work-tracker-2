<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  let roundToHalfHour = $state(false);
  let saving = $state(false);
  let saveError = $state('');

  onMount(async () => {
    try {
      const value = await invoke<string>('get_setting', { key: 'round_to_half_hour' });
      roundToHalfHour = value === 'true';
    } catch (e: any) {
      // Setting not yet stored — default off is correct
      console.error('get_setting round_to_half_hour:', e);
    }
  });

  async function handleToggle() {
    const next = !roundToHalfHour;
    saving = true;
    saveError = '';
    try {
      await invoke('set_setting', { key: 'round_to_half_hour', value: next ? 'true' : 'false' });
      roundToHalfHour = next;
    } catch (e: any) {
      console.error('set_setting round_to_half_hour:', e);
      saveError = e?.message ?? 'Failed to save setting';
    } finally {
      saving = false;
    }
  }
</script>

<div class="settings-view">
  <h2 class="section-title">Settings</h2>

  <div class="settings-group">
    <h3 class="group-title">Export</h3>

    <!-- Toggle row -->
    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">Round to started half-hour</span>
        <span class="setting-desc">Time exports use the nearest started 30-minute mark (e.g. 9:17 → 9:00)</span>
      </div>

      <!-- Accessible toggle: button acts as a switch -->
      <button
        class="toggle"
        class:on={roundToHalfHour}
        role="switch"
        aria-checked={roundToHalfHour}
        aria-label="Round to started half-hour"
        disabled={saving}
        onclick={handleToggle}
      >
        <span class="toggle-thumb"></span>
      </button>
    </div>

    {#if saveError}
      <p class="error">{saveError}</p>
    {/if}
  </div>
</div>

<style>
  .settings-view {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .section-title {
    font-size: 16px;
    font-weight: 700;
    color: var(--text);
    margin: 0;
  }

  .settings-group {
    background: var(--surface);
    border-radius: var(--radius);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .group-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    padding: 10px 16px 8px;
    margin: 0;
    border-bottom: 1px solid var(--border);
  }

  .setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 16px;
    gap: 16px;
    min-height: 64px;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
  }

  .setting-label {
    font-size: 14px;
    color: var(--text);
    font-weight: 500;
  }

  .setting-desc {
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.4;
  }

  /* Toggle switch */
  .toggle {
    position: relative;
    width: 48px;
    height: 28px;
    min-width: 48px;
    min-height: 44px; /* touch target */
    background: var(--border);
    border: none;
    border-radius: 14px;
    cursor: pointer;
    transition: background 0.15s ease;
    padding: 0;
    /* Vertically center the visible track within the tall touch target */
    display: flex;
    align-items: center;
    justify-content: flex-start;
  }

  .toggle:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .toggle.on {
    background: var(--accent);
  }

  .toggle:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-thumb {
    position: absolute;
    left: 3px;
    top: 50%;
    transform: translateY(-50%);
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: white;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    transition: left 0.15s ease;
    pointer-events: none;
  }

  .toggle.on .toggle-thumb {
    left: 23px;
  }

  .error {
    font-size: 12px;
    color: #ef4444;
    padding: 0 16px 12px;
    margin: 0;
  }
</style>
