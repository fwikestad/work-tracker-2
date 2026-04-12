<script lang="ts">
  import { getDailySummary } from '$lib/api/reports';
  import { today, formatHuman } from '$lib/utils/formatters';
  import type { DailySummary } from '$lib/types';

  let summary = $state<DailySummary | null>(null);
  let loading = $state(false);

  export async function refresh() {
    loading = true;
    try {
      summary = await getDailySummary(today());
    } catch (e) {
      console.error('Failed to refresh daily summary:', e);
      // Optionally set an error state to show in UI
    } finally {
      loading = false;
    }
  }
</script>

<section class="summary-section">
  <div class="header">
    <h3>Daily summary</h3>
  </div>

  {#if loading}
    <div class="loading">Loading...</div>
  {:else if summary}
    <div class="total">
      <span class="total-label">Total today</span>
      <span class="total-value">{formatHuman(summary.totalSeconds)}</span>
    </div>

    {#if summary.entries.length > 0}
      <div class="breakdown">
        {#each summary.entries as entry}
          <div class="entry">
            <div class="entry-info">
              {#if entry.customerColor}
                <span class="dot" style="background: {entry.customerColor}"></span>
              {/if}
              <span class="entry-name">{entry.customerName}</span>
            </div>
            <span class="entry-duration">{formatHuman(entry.totalSeconds)}</span>
          </div>
        {/each}
      </div>
    {/if}
  {:else}
    <div class="empty">No data</div>
  {/if}
</section>

<style>
  .summary-section {
    background: var(--surface);
    padding: 16px;
  }

  .header {
    margin-bottom: 12px;
  }

  h3 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
  }

  .loading,
  .empty {
    text-align: center;
    padding: 24px 16px;
    color: var(--text-muted);
    font-size: 14px;
  }

  .total {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    background: var(--bg);
    border-radius: var(--radius);
    margin-bottom: 12px;
  }

  .total-label {
    font-size: 14px;
    color: var(--text-muted);
  }

  .total-value {
    font-size: 18px;
    font-weight: 700;
    color: var(--text);
  }

  .breakdown {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .entry {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: var(--bg);
    border-radius: var(--radius);
  }

  .entry-info {
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

  .entry-name {
    font-size: 13px;
    color: var(--text);
  }

  .entry-duration {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-muted);
  }
</style>
