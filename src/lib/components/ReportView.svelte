<script lang="ts">
  import { getReport } from '$lib/api/reports';
  import { formatHuman, formatDay } from '$lib/utils/formatters';
  import { exportCsv } from '$lib/api/reports';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import type { ReportData } from '$lib/types';
  import { groupSessionsByDay } from '$lib/utils/reportGrouping';
  import { onMount } from 'svelte';

  let reportData = $state<ReportData | null>(null);
  let loading = $state(false);
  let exporting = $state(false);
  let error = $state('');
  let exportSuccess = $state(false);
  let rangeType = $state<'week' | 'month' | 'custom'>('week');
  let startDate = $state('');
  let endDate = $state('');
  let expandedDays = $state<Set<string>>(new Set());
  let expandedCustomers = $state<Set<string>>(new Set());

  // Initialize with "this week" once mounted (client-only; avoids SSR invoke failure)
  onMount(() => updateDateRange('week'));

  function updateDateRange(type: 'week' | 'month' | 'custom') {
    rangeType = type;
    const now = new Date();

    if (type === 'week') {
      // Monday of current week
      const day = now.getDay();
      const diff = now.getDate() - day + (day === 0 ? -6 : 1);
      const monday = new Date(now.setDate(diff));
      startDate = monday.toISOString().split('T')[0];
      endDate = new Date().toISOString().split('T')[0];
      loadReport();
    } else if (type === 'month') {
      // 1st of current month
      startDate = new Date(now.getFullYear(), now.getMonth(), 1).toISOString().split('T')[0];
      endDate = new Date().toISOString().split('T')[0];
      loadReport();
    }
    // For custom, wait for user to set dates
  }

  async function loadReport() {
    if (!startDate || !endDate) return;
    loading = true;
    error = '';
    try {
      reportData = await getReport(startDate, endDate);
    } catch (e: any) {
      error = e?.message ?? 'Failed to load report';
    } finally {
      loading = false;
    }
  }

  async function handleExport() {
    if (!startDate || !endDate) {
      error = 'Please select date range';
      return;
    }
    exporting = true;
    error = '';
    exportSuccess = false;
    try {
      const csv = await exportCsv(startDate, endDate);
      const path = await save({
        filters: [{ name: 'CSV', extensions: ['csv'] }],
        defaultPath: `work-tracker-${startDate}-${endDate}.csv`
      });
      if (path) {
        await writeTextFile(path, csv);
        exportSuccess = true;
        setTimeout(() => (exportSuccess = false), 3000);
      }
    } catch (e: any) {
      error = e?.message ?? 'Export failed';
    } finally {
      exporting = false;
    }
  }

  function toggleDay(date: string) {
    if (expandedDays.has(date)) {
      expandedDays.delete(date);
    } else {
      expandedDays.add(date);
    }
    expandedDays = new Set(expandedDays);
  }

  function toggleCustomer(date: string, customerName: string) {
    const key = `${date}::${customerName}`;
    if (expandedCustomers.has(key)) {
      expandedCustomers.delete(key);
    } else {
      expandedCustomers.add(key);
    }
    expandedCustomers = new Set(expandedCustomers);
  }

  function isCustomerExpanded(date: string, customerName: string): boolean {
    return expandedCustomers.has(`${date}::${customerName}`);
  }

  const dayGroups = $derived.by(() => {
    if (!reportData) return [];
    return groupSessionsByDay(reportData.sessions ?? []);
  });

  // When report data loads, expand all day groups by default
  $effect(() => {
    if (reportData) {
      const groups = groupSessionsByDay(reportData.sessions ?? []);
      expandedDays = new Set(groups.map((g) => g.date));
      expandedCustomers = new Set();
    }
  });

  $effect(() => {
    if (rangeType === 'custom' && startDate && endDate) {
      loadReport();
    }
  });
</script>

<div class="report-view">
  <div class="controls">
    <div class="range-buttons">
      <button
        class="range-btn"
        class:active={rangeType === 'week'}
        onclick={() => updateDateRange('week')}
      >
        This week
      </button>
      <button
        class="range-btn"
        class:active={rangeType === 'month'}
        onclick={() => updateDateRange('month')}
      >
        This month
      </button>
      <button
        class="range-btn"
        class:active={rangeType === 'custom'}
        onclick={() => (rangeType = 'custom')}
      >
        Custom
      </button>
    </div>

    {#if rangeType === 'custom'}
      <div class="date-inputs">
        <input type="date" bind:value={startDate} />
        <span>to</span>
        <input type="date" bind:value={endDate} />
        <button class="btn-load" onclick={loadReport} disabled={loading}>
          {loading ? 'Loading...' : 'Load'}
        </button>
      </div>
    {/if}

    <button class="btn-export" onclick={handleExport} disabled={exporting || !reportData}>
      {exporting ? 'Exporting...' : exportSuccess ? '✓ Exported!' : 'Export CSV'}
    </button>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if loading}
    <div class="loading">Loading report...</div>
  {:else if reportData}
    <div class="total-section">
      <span class="total-label">Total hours</span>
      <span class="total-value">{formatHuman(reportData.totalSeconds)}</span>
    </div>

    {#if dayGroups.length === 0}
      <div class="empty">No sessions in this period</div>
    {:else}
      <div class="breakdown">
        {#each dayGroups as dayGroup}
          <div class="day-group">
            <button class="day-header" onclick={() => toggleDay(dayGroup.date)}>
              <div class="day-info">
                <span class="expand-icon">{expandedDays.has(dayGroup.date) ? '▼' : '▶'}</span>
                <span class="day-label">{formatDay(dayGroup.date)}</span>
              </div>
              <span class="day-total">{formatHuman(dayGroup.totalSeconds)}</span>
            </button>

            {#if expandedDays.has(dayGroup.date)}
              <div class="day-customers">
                {#each dayGroup.customers as customer}
                  <div class="customer-group">
                    <button
                      class="customer-header"
                      onclick={() => toggleCustomer(dayGroup.date, customer.customerName)}
                    >
                      <div class="customer-info">
                        {#if customer.customerColor}
                          <span class="dot" style="background: {customer.customerColor}"></span>
                        {/if}
                        <span class="customer-name">{customer.customerName}</span>
                        <span class="expand-icon"
                          >{isCustomerExpanded(dayGroup.date, customer.customerName) ? '▼' : '▶'}</span
                        >
                      </div>
                      <span class="customer-total">{formatHuman(customer.totalSeconds)}</span>
                    </button>

                    {#if isCustomerExpanded(dayGroup.date, customer.customerName)}
                      <div class="work-orders">
                        {#each customer.workOrders as wo}
                          <div class="work-order-entry">
                            <div class="work-order-info">
                              <span class="work-order-name">{wo.workOrderName}</span>
                              <span class="session-count"
                                >{wo.sessionCount} session{wo.sessionCount !== 1 ? 's' : ''}</span
                              >
                            </div>
                            <span class="work-order-total">{formatHuman(wo.totalSeconds)}</span>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  {:else}
    <div class="empty">Select a date range to view report</div>
  {/if}
</div>

<style>
  .report-view {
    padding: 16px;
  }

  .controls {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 16px;
    padding: 12px;
    background: var(--surface);
    border-radius: var(--radius);
  }

  .range-buttons {
    display: flex;
    gap: 8px;
  }

  .range-btn {
    padding: 8px 16px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-muted);
    cursor: pointer;
    font-size: 13px;
    font-family: inherit;
  }

  .range-btn:hover {
    color: var(--text);
    border-color: #333;
  }

  .range-btn.active {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  .date-inputs {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }

  .date-inputs span {
    color: var(--text-muted);
    font-size: 13px;
  }

  input[type='date'] {
    background: var(--bg);
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

  .btn-load,
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

  .btn-load:hover:not(:disabled),
  .btn-export:hover:not(:disabled) {
    background: #3d9e6a;
  }

  .btn-load:disabled,
  .btn-export:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error-message {
    color: #ef4444;
    font-size: 12px;
    padding: 8px 12px;
    background: rgba(239, 68, 68, 0.1);
    border-radius: var(--radius);
    margin-bottom: 12px;
  }

  .loading,
  .empty {
    text-align: center;
    padding: 32px 16px;
    color: var(--text-muted);
    font-size: 14px;
  }

  .total-section {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: var(--surface);
    border-radius: var(--radius);
    margin-bottom: 16px;
  }

  .total-label {
    font-size: 16px;
    color: var(--text-muted);
  }

  .total-value {
    font-size: 24px;
    font-weight: 700;
    color: var(--text);
  }

  .breakdown {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  /* Day-level container */
  .day-group {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }

  .day-header {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 16px;
    background: transparent;
    border: none;
    cursor: pointer;
    font-family: inherit;
    color: var(--text);
  }

  .day-header:hover {
    background: var(--bg);
  }

  .day-info {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .day-label {
    font-size: 15px;
    font-weight: 700;
  }

  .day-total {
    font-size: 15px;
    font-weight: 700;
  }

  /* Customer rows indented under the day */
  .day-customers {
    border-top: 1px solid var(--border);
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .customer-group {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
    margin-left: 14px;
  }

  .customer-header {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    background: transparent;
    border: none;
    cursor: pointer;
    font-family: inherit;
    color: var(--text);
  }

  .customer-header:hover {
    background: var(--surface);
  }

  .customer-info {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    display: inline-block;
    flex-shrink: 0;
  }

  .customer-name {
    font-size: 14px;
    font-weight: 600;
  }

  .expand-icon {
    font-size: 10px;
    color: var(--text-muted);
  }

  .customer-total {
    font-size: 14px;
    font-weight: 700;
  }

  .work-orders {
    border-top: 1px solid var(--border);
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .work-order-entry {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: var(--surface);
    border-radius: var(--radius);
  }

  .work-order-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .work-order-name {
    font-size: 13px;
    color: var(--text);
  }

  .session-count {
    font-size: 11px;
    color: var(--text-muted);
  }

  .work-order-total {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }
</style>
