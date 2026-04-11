<script lang="ts">
  import {
    listWorkOrders,
    createWorkOrder,
    updateWorkOrder,
    archiveWorkOrder,
    toggleFavorite
  } from '$lib/api/workOrders';
  import { listCustomers } from '$lib/api/customers';
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import type { WorkOrder, Customer } from '$lib/types';

  let workOrders = $state<WorkOrder[]>([]);
  let customers = $state<Customer[]>([]);
  let loading = $state(false);
  let loadError = $state<string | null>(null);
  let editingId = $state<string | null>(null);
  let editName = $state('');
  let editDescription = $state('');
  let editStatus = $state<'active' | 'paused' | 'closed'>('active');
  let showArchived = $state(false);
  let filterCustomerId = $state('');
  let showAddForm = $state(false);
  let newCustomerId = $state('');
  let newName = $state('');
  let newDescription = $state('');
  let saving = $state(false);

  async function loadData() {
    loading = true;
    loadError = null;
    try {
      [workOrders, customers] = await Promise.all([
        listWorkOrders(filterCustomerId || undefined),
        listCustomers()
      ]);
    } catch (e: any) {
      console.error('Failed to load work orders/customers:', e);
      loadError = e?.message ?? 'Failed to load data';
    } finally {
      loading = false;
    }
  }

  async function handleCreate() {
    if (!newName.trim() || !newCustomerId) return;
    saving = true;
    try {
      await createWorkOrder({
        customerId: newCustomerId,
        name: newName.trim(),
        description: newDescription.trim() || undefined
      });
      await loadData();
      newName = '';
      newDescription = '';
      newCustomerId = '';
      showAddForm = false;
    } catch (e: any) {
      alert(e?.message ?? 'Failed to create work order');
    } finally {
      saving = false;
    }
  }

  function startEdit(wo: WorkOrder) {
    editingId = wo.id;
    editName = wo.name;
    editDescription = wo.description ?? '';
    editStatus = wo.status;
  }

  async function saveEdit(woId: string) {
    saving = true;
    try {
      await updateWorkOrder(woId, {
        name: editName.trim() || undefined,
        description: editDescription.trim() || undefined,
        status: editStatus
      });
      await loadData();
      editingId = null;
    } catch (e: any) {
      alert(e?.message ?? 'Failed to update');
    } finally {
      saving = false;
    }
  }

  async function handleArchive(woId: string) {
    if (!confirm('Archive this work order?')) return;
    try {
      await archiveWorkOrder(woId);
      await loadData();
    } catch (e: any) {
      alert(e?.message ?? 'Failed to archive');
    }
  }

  async function handleToggleFavorite(e: Event, woId: string) {
    e.stopPropagation();
    try {
      await toggleFavorite(woId);
      await loadData();
    } catch (e: any) {
      alert(e?.message ?? 'Failed to toggle favorite');
    }
  }

  $effect(() => {
    loadData();
  });
</script>

<div class="workorder-list">
  <div class="header">
    <h2>Work Orders</h2>
    <button class="btn-primary" onclick={() => (showAddForm = !showAddForm)}>
      {showAddForm ? 'Cancel' : '+ Add work order'}
    </button>
  </div>

  {#if showAddForm}
    <div class="add-form">
      <label>
        <span>Customer *</span>
        {#if customers.length === 0}
          <div class="no-customers">No customers yet. Create a customer first on the Customers tab.</div>
        {:else}
          <SearchableSelect
            bind:value={newCustomerId}
            options={customers.map((c) => ({ value: c.id, label: c.name, color: c.color }))}
            placeholder="Select customer"
          />
        {/if}
      </label>
      <label>
        <span>Name *</span>
        <input type="text" bind:value={newName} placeholder="Work order name" />
      </label>
      <label>
        <span>Description</span>
        <textarea bind:value={newDescription} rows="3" placeholder="Optional description"></textarea>
      </label>
      <button class="btn-primary" onclick={handleCreate} disabled={saving}>
        {saving ? 'Creating...' : 'Create'}
      </button>
    </div>
  {/if}

  <div class="controls">
    <label>
      <span>Filter by customer</span>
      <SearchableSelect
        bind:value={filterCustomerId}
        options={[
          { value: '', label: 'All customers' },
          ...customers.map((c) => ({ value: c.id, label: c.name, color: c.color }))
        ]}
        placeholder="All customers"
      />
    </label>
    <label class="checkbox">
      <input type="checkbox" bind:checked={showArchived} />
      Show archived
    </label>
  </div>

  {#if loading}
    <div class="loading">Loading...</div>
  {:else if loadError}
    <div class="load-error">⚠ {loadError}</div>
  {:else if workOrders.length === 0}
    <div class="empty">No work orders</div>
  {:else}
    <div class="items">
      {#each workOrders as wo}
        {#if editingId === wo.id}
          <div class="item editing">
            <label>
              <span>Name</span>
              <input type="text" bind:value={editName} />
            </label>
            <label>
              <span>Description</span>
              <textarea bind:value={editDescription} rows="3"></textarea>
            </label>
            <label>
              <span>Status</span>
              <select bind:value={editStatus}>
                <option value="active">Active</option>
                <option value="paused">Paused</option>
                <option value="closed">Closed</option>
              </select>
            </label>
            <div class="actions">
              <button class="btn-sm btn-primary" onclick={() => saveEdit(wo.id)} disabled={saving}>
                Save
              </button>
              <button class="btn-sm btn-secondary" onclick={() => (editingId = null)}>Cancel</button>
            </div>
          </div>
        {:else}
          <div class="item">
            <div class="item-info" onclick={() => startEdit(wo)}>
              <div class="item-main-info">
                <span
                  class="star-btn"
                  role="button"
                  tabindex="0"
                  onclick={(e) => handleToggleFavorite(e, wo.id)}
                  onkeydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      handleToggleFavorite(e, wo.id);
                    }
                  }}
                  title={wo.isFavorite ? 'Remove from favorites' : 'Add to favorites'}
                >
                  {wo.isFavorite ? '⭐' : '☆'}
                </span>
                <div>
                  <div class="item-name">{wo.name}</div>
                  <div class="item-meta">
                    {#if wo.customerColor}
                      <span class="dot" style="background: {wo.customerColor}"></span>
                    {/if}
                    <span>{wo.customerName}</span>
                  </div>
                </div>
              </div>
              <span class="badge badge-{wo.status}">{wo.status}</span>
            </div>
            <button class="btn-archive" onclick={() => handleArchive(wo.id)}>Archive</button>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .workorder-list {
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

  .add-form {
    background: var(--surface);
    padding: 16px;
    border-radius: var(--radius);
    margin-bottom: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .controls {
    margin-bottom: 16px;
    display: flex;
    gap: 16px;
    align-items: flex-end;
  }

  .checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: var(--text);
    cursor: pointer;
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
    background: var(--bg);
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
  }

  .loading,
  .empty,
  .load-error {
    text-align: center;
    padding: 32px;
    color: var(--text-muted);
  }

  .load-error {
    color: var(--danger);
  }

  .no-customers {
    font-size: 13px;
    color: var(--text-muted);
    padding: 10px 0;
    font-style: italic;
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
    padding: 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .item.editing {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }

  .item-info {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    cursor: pointer;
    flex: 1;
  }

  .item-main-info {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .star-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 14px;
    cursor: pointer;
    padding: 0;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .star-btn:hover {
    color: #fbbf24;
  }

  .item-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text);
    margin-bottom: 4px;
  }

  .item-meta {
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

  .badge {
    font-size: 11px;
    padding: 4px 8px;
    border-radius: 3px;
    font-weight: 600;
  }

  .badge-active {
    background: var(--accent);
    color: white;
  }

  .badge-paused {
    background: #aaaa7a;
    color: white;
  }

  .badge-closed {
    background: var(--border);
    color: var(--text-muted);
  }

  .btn-archive {
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 6px 12px;
    font-family: inherit;
    font-size: 12px;
    color: var(--text-muted);
    cursor: pointer;
  }

  .btn-archive:hover {
    border-color: var(--danger);
    color: var(--danger);
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
</style>
