<script lang="ts">
  import {
    listCustomers,
    createCustomer,
    updateCustomer,
    archiveCustomer
  } from '$lib/api/customers';
  import type { Customer, CreateCustomerParams, UpdateCustomerParams } from '$lib/types';

  let customers = $state<Customer[]>([]);
  let loading = $state(false);
  let editingId = $state<string | null>(null);
  let editName = $state('');
  let editCode = $state('');
  let editColor = $state('');
  let showArchived = $state(false);
  let showAddForm = $state(false);
  let newName = $state('');
  let newCode = $state('');
  let newColor = $state('#7c7aaa');
  let saving = $state(false);

  async function loadCustomers() {
    loading = true;
    try {
      customers = await listCustomers(showArchived);
    } finally {
      loading = false;
    }
  }

  async function handleCreate() {
    if (!newName.trim()) return;
    saving = true;
    try {
      await createCustomer({
        name: newName.trim(),
        code: newCode.trim() || undefined,
        color: newColor || undefined
      });
      await loadCustomers();
      newName = '';
      newCode = '';
      newColor = '#7c7aaa';
      showAddForm = false;
    } catch (e: any) {
      alert(e?.message ?? 'Failed to create customer');
    } finally {
      saving = false;
    }
  }

  function startEdit(customer: Customer) {
    editingId = customer.id;
    editName = customer.name;
    editCode = customer.code ?? '';
    editColor = customer.color ?? '#7c7aaa';
  }

  async function saveEdit(customerId: string) {
    saving = true;
    try {
      await updateCustomer(customerId, {
        name: editName.trim() || undefined,
        code: editCode.trim() || undefined,
        color: editColor || undefined
      });
      await loadCustomers();
      editingId = null;
    } catch (e: any) {
      alert(e?.message ?? 'Failed to update');
    } finally {
      saving = false;
    }
  }

  async function handleArchive(customerId: string) {
    if (!confirm('Archive this customer?')) return;
    try {
      await archiveCustomer(customerId);
      await loadCustomers();
    } catch (e: any) {
      alert(e?.message ?? 'Failed to archive');
    }
  }

  $effect(() => {
    loadCustomers();
  });
</script>

<div class="customer-list">
  <div class="header">
    <h2>Customers</h2>
    <button class="btn-primary" onclick={() => (showAddForm = !showAddForm)}>
      {showAddForm ? 'Cancel' : '+ Add customer'}
    </button>
  </div>

  {#if showAddForm}
    <div class="add-form">
      <label>
        <span>Name *</span>
        <input type="text" bind:value={newName} placeholder="Customer name" />
      </label>
      <label>
        <span>Code</span>
        <input type="text" bind:value={newCode} placeholder="Optional code" />
      </label>
      <label>
        <span>Color</span>
        <input type="color" bind:value={newColor} />
      </label>
      <button class="btn-primary" onclick={handleCreate} disabled={saving}>
        {saving ? 'Creating...' : 'Create'}
      </button>
    </div>
  {/if}

  <div class="controls">
    <label class="checkbox">
      <input type="checkbox" bind:checked={showArchived} />
      Show archived
    </label>
  </div>

  {#if loading}
    <div class="loading">Loading...</div>
  {:else if customers.length === 0}
    <div class="empty">No customers yet</div>
  {:else}
    <div class="items">
      {#each customers as customer}
        {#if editingId === customer.id}
          <div class="item editing">
            <label>
              <span>Name</span>
              <input type="text" bind:value={editName} />
            </label>
            <label>
              <span>Code</span>
              <input type="text" bind:value={editCode} />
            </label>
            <label>
              <span>Color</span>
              <input type="color" bind:value={editColor} />
            </label>
            <div class="actions">
              <button class="btn-sm btn-primary" onclick={() => saveEdit(customer.id)} disabled={saving}>
                Save
              </button>
              <button class="btn-sm btn-secondary" onclick={() => (editingId = null)}>
                Cancel
              </button>
            </div>
          </div>
        {:else}
          <div class="item">
            <div class="item-info" onclick={() => startEdit(customer)}>
              {#if customer.color}
                <span class="dot" style="background: {customer.color}"></span>
              {/if}
              <div>
                <div class="item-name">{customer.name}</div>
                {#if customer.code}
                  <div class="item-code">{customer.code}</div>
                {/if}
              </div>
            </div>
            <button class="btn-archive" onclick={() => handleArchive(customer.id)}>Archive</button>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .customer-list {
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

  input[type='text'],
  input[type='color'] {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text);
    padding: 8px;
    font-family: inherit;
    font-size: 14px;
  }

  input[type='color'] {
    width: 60px;
    height: 40px;
    cursor: pointer;
  }

  input:focus {
    outline: 1px solid var(--accent);
    border-color: var(--accent);
  }

  .loading,
  .empty {
    text-align: center;
    padding: 32px;
    color: var(--text-muted);
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
    gap: 10px;
    cursor: pointer;
    flex: 1;
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .item-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text);
  }

  .item-code {
    font-size: 12px;
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

  .btn-primary {
    background: var(--accent);
    color: white;
  }

  .btn-secondary {
    background: var(--border);
    color: var(--text);
  }

  .btn-secondary:hover {
    background: #333;
  }
</style>
