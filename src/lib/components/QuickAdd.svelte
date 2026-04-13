<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';
  import { timer } from '$lib/stores/timer.svelte';
  import { sessionsStore } from '$lib/stores/sessions.svelte';
  import { listCustomers } from '$lib/api/customers';
  import { quickAdd } from '$lib/api/sessions';
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import type { Customer, ActiveSession } from '$lib/types';
  import { onMount } from 'svelte';

  let customers = $state<Customer[]>([]);
  let selectedCustomerId = $state('');
  let newCustomerName = $state('');
  let workOrderName = $state('');
  let submitting = $state(false);
  let error = $state('');
  let useNewCustomer = $state(false);
  let inputRef = $state<HTMLInputElement | undefined>(undefined);

  $effect(() => {
    if (uiStore.quickAdd) {
      listCustomers().then((c) => (customers = c));
      setTimeout(() => inputRef?.focus(), 100);
    }
  });

  $effect(() => {
    if (selectedCustomerId === '__new__') {
      useNewCustomer = true;
      selectedCustomerId = '';
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
        workOrderName: workOrderName.trim()
      });
      const active: ActiveSession = {
        sessionId: result.session.id,
        workOrderId: result.workOrder.id,
        workOrderName: result.workOrder.name,
        customerName: result.customer.name,
        customerColor: result.customer.color ?? null,
        startedAt: result.session.startTime,
        elapsedSeconds: 0,
        isPaused: false
      };
      timer.setActive(active);
      await sessionsStore.refreshAll();
      uiStore.closeQuickAdd();
      workOrderName = '';
      selectedCustomerId = '';
      newCustomerName = '';
      useNewCustomer = false;
    } catch (e: any) {
      console.error('Quick add failed:', e);
      error = e?.message || e?.toString() || 'Something went wrong';
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

  function handleBackdropClick(e: Event) {
    if (e.target === e.currentTarget) {
      uiStore.closeQuickAdd();
    }
  }
</script>

{#if uiStore.quickAdd}
  <div
    class="overlay"
    role="button"
    tabindex="0"
    onclick={handleBackdropClick}
    onkeydown={(e) => e.key === 'Enter' && handleBackdropClick(e)}
  >
    <div class="dialog">
      <div class="header">
        <h2>Quick Add</h2>
        <span class="hint">Esc to close</span>
      </div>

      <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
        <label>
          <span>Customer</span>
          {#if !useNewCustomer}
            <SearchableSelect
              bind:value={selectedCustomerId}
              options={[
                ...customers.map((c) => ({ value: c.id, label: c.name, color: c.color })),
                { value: '__new__', label: '+ New customer' }
              ]}
              placeholder="Select a customer"
            />
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

  input {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text);
    padding: 10px;
    font-family: inherit;
    font-size: 14px;
    min-height: 44px;
  }

  input:focus {
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
