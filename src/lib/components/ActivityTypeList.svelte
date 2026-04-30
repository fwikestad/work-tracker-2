<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { ActivityType } from '$lib/types';

  let activityTypes = $state<ActivityType[]>([]);
  let newName = $state('');
  let editingId = $state<string | null>(null);
  let editingName = $state('');
  let saving = $state(false);

  async function load() {
    activityTypes = await invoke<ActivityType[]>('list_activity_types');
  }

  async function addType() {
    if (!newName.trim()) return;
    saving = true;
    try {
      await invoke('create_activity_type', { params: { name: newName.trim() } });
      newName = '';
      await load();
    } catch (e: any) {
      alert(e?.message ?? 'Failed to create activity type');
    } finally {
      saving = false;
    }
  }

  function startEdit(type: ActivityType) {
    editingId = type.id;
    editingName = type.name;
  }

  async function saveEdit(id: string) {
    if (!editingName.trim()) return;
    saving = true;
    try {
      await invoke('update_activity_type', { id, params: { name: editingName.trim() } });
      editingId = null;
      await load();
    } catch (e: any) {
      alert(e?.message ?? 'Failed to update activity type');
    } finally {
      saving = false;
    }
  }

  async function deleteType(id: string, name: string) {
    const ok = confirm(
      `Delete activity type "${name}"?\n\nExisting sessions using this type will keep the label, but it won't appear in dropdowns.`
    );
    if (!ok) return;
    try {
      await invoke('delete_activity_type', { id });
      await load();
    } catch (e: any) {
      alert(e?.message ?? 'Failed to delete activity type');
    }
  }

  async function moveUp(type: ActivityType, index: number) {
    if (index === 0) return;
    const prev = activityTypes[index - 1];
    await invoke('update_activity_type', { id: type.id, params: { sortOrder: prev.sortOrder } });
    await invoke('update_activity_type', { id: prev.id, params: { sortOrder: type.sortOrder } });
    await load();
  }

  async function moveDown(type: ActivityType, index: number) {
    if (index === activityTypes.length - 1) return;
    const next = activityTypes[index + 1];
    await invoke('update_activity_type', { id: type.id, params: { sortOrder: next.sortOrder } });
    await invoke('update_activity_type', { id: next.id, params: { sortOrder: type.sortOrder } });
    await load();
  }

  $effect(() => {
    load();
  });
</script>

<div class="activity-type-list">
  <div class="header">
    <h2>Activity Types</h2>
  </div>

  <div class="add-form">
    <input
      type="text"
      bind:value={newName}
      placeholder="New activity type name"
      onkeydown={(e) => e.key === 'Enter' && addType()}
    />
    <button class="btn-primary" onclick={addType} disabled={saving || !newName.trim()}>
      Add
    </button>
  </div>

  {#if activityTypes.length === 0}
    <div class="empty">No activity types yet. Add one above.</div>
  {:else}
    <div class="items">
      {#each activityTypes as type, index}
        <div class="item">
          <div class="order-controls">
            <button
              class="btn-order"
              onclick={() => moveUp(type, index)}
              disabled={index === 0}
              title="Move up"
            >↑</button>
            <button
              class="btn-order"
              onclick={() => moveDown(type, index)}
              disabled={index === activityTypes.length - 1}
              title="Move down"
            >↓</button>
          </div>

          {#if editingId === type.id}
            <input
              class="edit-input"
              type="text"
              bind:value={editingName}
              onkeydown={(e) => {
                if (e.key === 'Enter') saveEdit(type.id);
                if (e.key === 'Escape') editingId = null;
              }}
            />
            <div class="item-actions">
              <button class="btn-sm btn-primary" onclick={() => saveEdit(type.id)} disabled={saving}>
                Save
              </button>
              <button class="btn-sm btn-secondary" onclick={() => (editingId = null)}>Cancel</button>
            </div>
          {:else}
            <span class="type-name">{type.name}</span>
            <div class="item-actions">
              <button class="btn-edit" onclick={() => startEdit(type)} title="Edit">✏️</button>
              <button class="btn-delete" onclick={() => deleteType(type.id, type.name)} title="Delete">🗑</button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .activity-type-list {
    padding: 16px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  h2 {
    font-size: 18px;
    font-weight: 600;
    color: var(--text);
  }

  .add-form {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }

  .add-form input {
    flex: 1;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text);
    padding: 8px;
    font-family: inherit;
    font-size: 14px;
  }

  .add-form input:focus {
    outline: 1px solid var(--accent);
    border-color: var(--accent);
  }

  .btn-primary {
    background: var(--accent);
    color: white;
    border: none;
    border-radius: var(--radius);
    padding: 8px 16px;
    font-family: inherit;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn-primary:hover:not(:disabled) {
    background: #3d9e6a;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .empty {
    text-align: center;
    padding: 32px;
    color: var(--text-muted);
    font-size: 14px;
  }

  .items {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .item {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 10px 12px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .order-controls {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .btn-order {
    background: none;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-muted);
    font-size: 11px;
    line-height: 1;
    padding: 2px 4px;
    cursor: pointer;
    width: 22px;
  }

  .btn-order:hover:not(:disabled) {
    color: var(--text);
    border-color: #333;
  }

  .btn-order:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .type-name {
    flex: 1;
    font-size: 14px;
    color: var(--text);
  }

  .edit-input {
    flex: 1;
    background: var(--bg);
    border: 1px solid var(--accent);
    border-radius: var(--radius);
    color: var(--text);
    padding: 6px 8px;
    font-family: inherit;
    font-size: 14px;
    outline: none;
  }

  .item-actions {
    display: flex;
    gap: 4px;
  }

  .btn-edit,
  .btn-delete {
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 4px 8px;
    font-size: 13px;
    cursor: pointer;
    color: var(--text-muted);
  }

  .btn-delete:hover {
    border-color: var(--danger);
    color: var(--danger);
  }

  .btn-edit:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .btn-sm {
    padding: 6px 12px;
    border: none;
    border-radius: var(--radius);
    font-family: inherit;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn-sm.btn-primary {
    background: var(--accent);
    color: white;
  }

  .btn-sm.btn-secondary {
    background: var(--border);
    color: var(--text);
  }

  .btn-sm.btn-secondary:hover {
    background: #333;
  }

  .btn-sm:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
