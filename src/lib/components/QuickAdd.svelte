<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';
  import { timer } from '$lib/stores/timer.svelte';
  import { sessionsStore } from '$lib/stores/sessions.svelte';
  import { listCustomers } from '$lib/api/customers';
  import { quickAdd } from '$lib/api/sessions';
  import type { Customer } from '$lib/types';
  import { onMount } from 'svelte';

  let customers = $state<Customer[]>([]);
  let selectedCustomerId = $state('');
  let newCustomerName = $state('');
  let workOrderName = $state('');
  let workOrderCode = $state('');
  let submitting = $state(false);
  let error = $state('');
  let useNewCustomer = $state(false);
  let inputRef: HTMLInputElement;

  $effect(() => {
    if (uiStore.quickAdd) {
      listCustomers().then((c) => (customers = c));
      setTimeout(() => inputRef?.focus(), 100);
    }
  });

  async function handleSubmit() {
    if (!workOrderName.trim()) {
      error = 'Work order name is required';
      return;
    }
    if (!useNewCustomer && !selectedCustomerId) {
      error = 'Select a customer';
      return;
    }
    if (useNewCustomer && !newCustomerName.trim()) {
      error = 'Customer name is required';
      return;
    }

    submitting = true;
    error = '';
    try {
      const result = await quickAdd({
        customerId: useNewCustomer ? undefined : selectedCustomerId,
        customerName: useNewCustomer ? newCustomerName : undefined,
        workOrderName: workOrderName.trim(),
        workOrderCode: workOrderCode.trim() || undefined
      });
      timer.setActive({
        sessionId: result.session.id,
        workOrderId: result.workOrder.id,
        workOrderName: result.workOrder.name,
        customerName: result.customer.name,
        customerColor: result.customer.color,
        startedAt: result.session.startTime,
        elapsedSeconds: 0
      });
      await sessionsStore.refreshAll();
      uiStore.closeQuickAdd();
      workOrderName = '';
      workOrderCode = '';
      selectedCustomerId = '';
      newCustomerName = '';
      useNewCustomer = false;
    } catch (e: any) {
      error = e?.message ?? 'Something went wrong';
    } finally {
      submitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      uiStore.closeQuickAdd();
    }
  }
</script>

{#if uiStore.quickAdd}
  <div class="overlay" onclick={handleBackdropClick}>
    <div class="dialog">
      <div class="header">
        <h2>Quick Add</h2>
        <span class="hint">Esc to close</span>
      </div>

      <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
        <label>
          <span>Customer</span>
          {#if !useNewCustomer}
            <select bind:value={selectedCustomerId} required>
              <option value="">Select a customer</option>
              {#each customers as customer}
                <option value={customer.id}>{customer.name}</option>
              {/each}
              <option value="__new__">+ New customer</option>
            </select>
            {#if selectedCustomerId === '__new__'}
              {(useNewCustomer = true, (selectedCustomerId = ''))}
            {/if}
          {:else}
            <div class="new-customer-input">
              <input
                type="text"
                bind:value={newCustomerName}
                placeholder="Customer name"
                required
              />
              <button
                type="button"
                class="btn-link"
                onclick={() => {
                  useNewCustomer = false;
                  newCustomerName = '';
                }}
              >
                ← Select existing
              </button>
            </div>
          {/if}
        </label>

        <label>
          <span>Work order name *</span>
          <input
            type="text"
            bind:value={workOrderName}
            bind:this={inputRef}
            placeholder="e.g., API Development"
            required
          />
        </label>

        <label>
          <span>Work order code (optional)</span>
          <input type="text" bind:value={workOrderCode} placeholder="e.g., WO-2026-04" />
        </label>

        {#if error}
          <div class="error">{error}</div>
        {/if}

        <button type="submit" class="btn-primary" disabled={submitting}>
          {submitting ? 'Starting...' : 'Start tracking →'}
        </button>
      </form>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 999;
  }

  .dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 24px;
    max-width: 380px;
    width: 90%;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  h2 {
    font-size: 18px;
    font-weight: 600;
    color: var(--text);
  }

  .hint {
    font-size: 11px;
    color: var(--text-muted);
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  label span {
    font-size: 12px;
    color: var(--text-muted);
  }

  input,
  select {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text);
    padding: 10px;
    font-family: inherit;
    font-size: 14px;
    min-height: 44px;
  }

  input:focus,
  select:focus {
    outline: 1px solid var(--accent);
    border-color: var(--accent);
  }

  .new-customer-input {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .btn-link {
    background: none;
    border: none;
    color: var(--accent);
    font-size: 12px;
    cursor: pointer;
    text-align: left;
    padding: 0;
    font-family: inherit;
  }

  .btn-link:hover {
    text-decoration: underline;
  }

  .error {
    color: var(--danger);
    font-size: 12px;
    padding: 8px;
    background: rgba(224, 82, 82, 0.1);
    border-radius: var(--radius);
  }

  .btn-primary {
    background: var(--accent);
    color: white;
    border: none;
    border-radius: var(--radius);
    padding: 12px 24px;
    font-family: inherit;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    min-height: 44px;
  }

  .btn-primary:hover:not(:disabled) {
    background: #3d9e6a;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
