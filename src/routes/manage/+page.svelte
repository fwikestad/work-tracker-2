<script lang="ts">
  import CustomerList from '$lib/components/customers/CustomerList.svelte';
  import WorkOrderList from '$lib/components/workorders/WorkOrderList.svelte';

  let activeTab = $state<'customers' | 'workorders'>('customers');
</script>

<div class="manage-page">
  <header class="page-header">
    <nav class="tabs">
      <button
        class="tab-btn"
        class:active={activeTab === 'customers'}
        onclick={() => (activeTab = 'customers')}
      >
        Customers
      </button>
      <button
        class="tab-btn"
        class:active={activeTab === 'workorders'}
        onclick={() => (activeTab = 'workorders')}
      >
        Work Orders
      </button>
    </nav>
  </header>

  <div class="page-content">
    {#if activeTab === 'customers'}
      <CustomerList />
    {:else if activeTab === 'workorders'}
      <WorkOrderList />
    {/if}
  </div>

  <footer class="page-footer">
    <a href="/" class="back-link">← Back to tracking</a>
  </footer>
</div>

<style>
  .manage-page {
    max-width: 720px;
    margin: 0 auto;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg);
  }

  .page-header {
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    padding: 16px;
  }

  .tabs {
    display: flex;
    gap: 8px;
  }

  .tab-btn {
    padding: 8px 16px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-muted);
    cursor: pointer;
    font-size: 13px;
    font-family: inherit;
  }

  .tab-btn:hover {
    color: var(--text);
    border-color: #333;
  }

  .tab-btn.active {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  .page-content {
    flex: 1;
    background: var(--bg);
  }

  .page-footer {
    background: var(--surface);
    border-top: 1px solid var(--border);
    padding: 12px 16px;
    text-align: center;
  }

  .back-link {
    color: var(--accent);
    text-decoration: none;
    font-size: 14px;
  }

  .back-link:hover {
    text-decoration: underline;
  }
</style>
