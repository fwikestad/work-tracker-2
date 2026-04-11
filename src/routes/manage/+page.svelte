<script lang="ts">
  import CustomerList from '$lib/components/customers/CustomerList.svelte';
  import WorkOrderList from '$lib/components/workorders/WorkOrderList.svelte';
  import ReportView from '$lib/components/ReportView.svelte';
  import { exportCsv } from '$lib/api/reports';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';

  let activeTab = $state<'customers' | 'workorders' | 'reports'>('customers');
  let startDate = $state('');
  let endDate = $state('');
  let exporting = $state(false);

  // Set default dates to this month
  const now = new Date();
  const firstDay = new Date(now.getFullYear(), now.getMonth(), 1).toISOString().split('T')[0];
  const lastDay = new Date(now.getFullYear(), now.getMonth() + 1, 0).toISOString().split('T')[0];
  startDate = firstDay;
  endDate = lastDay;

  async function handleExport() {
    if (!startDate || !endDate) {
      alert('Please select date range');
      return;
    }
    exporting = true;
    try {
      const csv = await exportCsv(startDate, endDate);
      const path = await save({
        filters: [{ name: 'CSV', extensions: ['csv'] }],
        defaultPath: `work-tracker-${startDate}-${endDate}.csv`
      });
      if (path) {
        await writeTextFile(path, csv);
        alert('Export successful!');
      }
    } catch (e: any) {
      alert(e?.message ?? 'Export failed');
    } finally {
      exporting = false;
    }
  }
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
      <button
        class="tab-btn"
        class:active={activeTab === 'reports'}
        onclick={() => (activeTab = 'reports')}
      >
        Reports
      </button>
    </nav>

    {#if activeTab !== 'reports'}
      <div class="export-section">
        <h3>Export CSV</h3>
        <div class="export-form">
          <input type="date" bind:value={startDate} />
          <span>to</span>
          <input type="date" bind:value={endDate} />
          <button class="btn-export" onclick={handleExport} disabled={exporting}>
            {exporting ? 'Exporting...' : 'Export'}
          </button>
        </div>
      </div>
    {/if}
  </header>

  <div class="page-content">
    {#if activeTab === 'customers'}
      <CustomerList />
    {:else if activeTab === 'workorders'}
      <WorkOrderList />
    {:else if activeTab === 'reports'}
      <ReportView />
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
    margin-bottom: 16px;
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

  .export-section {
    padding: 12px;
    background: var(--bg);
    border-radius: var(--radius);
  }

  h3 {
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
    margin-bottom: 8px;
  }

  .export-form {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }

  .export-form span {
    color: var(--text-muted);
    font-size: 13px;
  }

  input[type='date'] {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text);
    padding: 6px 10px;
    font-family: inherit;
    font-size: 13px;
  }

  input[type='date']:focus {
    outline: 1px solid var(--accent);
    border-color: var(--accent);
  }

  .btn-export {
    background: var(--accent);
    color: white;
    border: none;
    border-radius: var(--radius);
    padding: 6px 16px;
    font-family: inherit;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn-export:hover:not(:disabled) {
    background: #3d9e6a;
  }

  .btn-export:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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
