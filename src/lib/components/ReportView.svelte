<script lang="ts">
  import { getReport } from '$lib/api/reports';
  import { formatHuman } from '$lib/utils/formatters';
  import { exportCsv } from '$lib/api/reports';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import type { ReportData, ReportEntry } from '$lib/types';
  import { onMount } from 'svelte';

  let reportData = $state<ReportData | null>(null);
  let loading = $state(false);
  let exporting = $state(false);
  let rangeType = $state<'week' | 'month' | 'custom'>('week');
  let startDate = $state('');
  let endDate = $state('');
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
    try {
      reportData = await getReport(startDate, endDate);
    } catch (e: any) {
      alert(e?.message ?? 'Failed to load report');
    } finally {
      loading = false;
    }
  }

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

  function toggleCustomer(customerId: string) {
    if (expandedCustomers.has(customerId)) {
      expandedCustomers.delete(customerId);
    } else {
      expandedCustomers.add(customerId);
    }
    expandedCustomers = new Set(expandedCustomers);
  }

  // Group entries by customer
  const groupedEntries = $derived.by(() => {
    if (!reportData) return new Map<string, { customerName: string; customerColor: string | null; entries: ReportEntry[]; totalSeconds: number }>();
    
    const map = new Map<string, { customerName: string; customerColor: string | null; entries: ReportEntry[]; totalSeconds: number }>();
    
    for (const entry of reportData.entries) {
      if (!map.has(entry.customerId)) {
        map.set(entry.customerId, {
          customerName: entry.customerName,
          customerColor: entry.customerColor,
          entries: [],
          totalSeconds: 0
        });
      }
      const group = map.get(entry.customerId)!;
      group.entries.push(entry);
      group.totalSeconds += entry.totalSeconds;
    }
    
    return map;
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
      {exporting ? 'Exporting...' : 'Export CSV'}
    </button>
  </div>

  {#if loading}
    <div class="loading">Loading report...</div>
  {:else if reportData}
    <div class="total-section">
      <span class="total-label">Total hours</span>
      <span class="total-value">{formatHuman(reportData.totalSeconds)}</span>
    </div>

    {#if groupedEntries.size === 0}
      <div class="empty">No sessions in this period</div>
    {:else}
      <div class="breakdown">
        {#each [...groupedEntries.entries()] as [customerId, group]}
          <div class="customer-group">
            <button class="customer-header" onclick={() => toggleCustomer(customerId)}>
              <div class="customer-info">
                {#if group.customerColor}
                  <span class="dot" style="background: {group.customerColor}"></span>
                {/if}
                <span class="customer-name">{group.customerName}</span>
                <span class="expand-icon">{expandedCustomers.has(customerId) ? '▼' : '▶'}</span>
              </div>
              <span class="customer-total">{formatHuman(group.totalSeconds)}</span>
            </button>

            {#if expandedCustomers.has(customerId)}
              <div class="work-orders">
                {#each group.entries as entry}
                  <div class="work-order-entry">
                    <div class="work-order-info">
                      <span class="work-order-name">{entry.workOrderName}</span>
                      <span class="session-count">{entry.sessionCount} session{entry.sessionCount !== 1 ? 's' : ''}</span>
                    </div>
                    <span class="work-order-total">{formatHuman(entry.totalSeconds)}</span>
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

  .customer-group {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }

  .customer-header {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    background: transparent;
    border: none;
    cursor: pointer;
    font-family: inherit;
    color: var(--text);
  }

  .customer-header:hover {
    background: var(--bg);
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
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .work-order-entry {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: var(--bg);
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
