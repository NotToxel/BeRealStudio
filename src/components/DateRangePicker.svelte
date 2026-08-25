<script lang="ts">
  import type { MonthCount } from '$lib/types';
  import Calendar from 'lucide-svelte/icons/calendar';
  import BarChart3 from 'lucide-svelte/icons/chart-bar';

  export let histogram: MonthCount[] = [];
  export let minDate: string = '';
  export let maxDate: string = '';
  export let startDate: string = '';
  export let endDate: string = '';
  export let totalCount: number = 0;
  export let selectedCount: number = 0;

  $: maxBarCount = Math.max(...histogram.map((h) => h.count), 1);

  function isMonthInRange(monthStr: string): boolean {
    if (!startDate && !endDate) return true;
    const mStart = `${monthStr}-01`;
    const mEnd = `${monthStr}-31`;
    if (startDate && mEnd < startDate) return false;
    if (endDate && mStart > endDate) return false;
    return true;
  }

  function setPreset(preset: 'all' | '30d' | '6m' | '1y' | 'this_year') {
    if (!maxDate && histogram.length === 0) return;
    const reference = maxDate ? new Date(maxDate) : new Date();

    if (preset === 'all') {
      startDate = minDate || (histogram.length > 0 ? `${histogram[0].month}-01` : '');
      endDate = maxDate || (histogram.length > 0 ? `${histogram[histogram.length - 1].month}-28` : '');
      return;
    }

    const end = new Date(reference);
    const start = new Date(reference);

    if (preset === '30d') {
      start.setDate(end.getDate() - 30);
    } else if (preset === '6m') {
      start.setMonth(end.getMonth() - 6);
    } else if (preset === '1y') {
      start.setFullYear(end.getFullYear() - 1);
    } else if (preset === 'this_year') {
      start.setMonth(0, 1);
    }

    startDate = formatDate(start);
    endDate = formatDate(end);
  }

  function formatDate(d: Date): string {
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  }

  function handleBarClick(month: string) {
    startDate = `${month}-01`;
    const parts = month.split('-');
    const y = parseInt(parts[0], 10);
    const m = parseInt(parts[1], 10);
    const lastDay = new Date(y, m, 0).getDate();
    endDate = `${month}-${String(lastDay).padStart(2, '0')}`;
  }
</script>

<div class="range-picker card">
  <div class="header">
    <div class="title-group">
      <BarChart3 size={15} class="text-amber-400" />
      <span class="title-sm">Date Range &amp; Density</span>
      {#if totalCount > 0}
        <span class="badge badge-yellow">
          {selectedCount} of {totalCount} entries selected
        </span>
      {/if}
    </div>

    <!-- Presets -->
    <div class="presets">
      <button type="button" class="preset-btn" on:click={() => setPreset('all')}>All Time</button>
      <button type="button" class="preset-btn" on:click={() => setPreset('30d')}>30 Days</button>
      <button type="button" class="preset-btn" on:click={() => setPreset('6m')}>6 Months</button>
      <button type="button" class="preset-btn" on:click={() => setPreset('1y')}>1 Year</button>
    </div>
  </div>

  <!-- Density Histogram -->
  {#if histogram.length > 0}
    <div class="histogram-container">
      <div class="bars">
        {#each histogram as item}
          {@const inRange = isMonthInRange(item.month)}
          {@const heightPercent = Math.max((item.count / maxBarCount) * 100, 8)}
          <button
            type="button"
            class="bar-col"
            class:in-range={inRange}
            title="{item.month}: {item.count} BeReals"
            on:click={() => handleBarClick(item.month)}
          >
            <div class="bar" style="height: {heightPercent}%;"></div>
            <span class="bar-label">{item.month.slice(2)}</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Dual Date Pickers -->
  <div class="inputs-row">
    <div class="input-field">
      <label for="start-date">
        <Calendar size={12} class="text-secondary" />
        <span>Start Date</span>
      </label>
      <input
        id="start-date"
        type="date"
        class="input-text"
        bind:value={startDate}
        min={minDate}
        max={endDate || maxDate}
      />
    </div>

    <div class="input-field">
      <label for="end-date">
        <Calendar size={12} class="text-secondary" />
        <span>End Date</span>
      </label>
      <input
        id="end-date"
        type="date"
        class="input-text"
        bind:value={endDate}
        min={startDate || minDate}
        max={maxDate}
      />
    </div>
  </div>
</div>

<style>
  .range-picker {
    display: flex;
    flex-direction: column;
    gap: 16px;
    background: #111116;
    padding: 18px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .presets {
    display: flex;
    gap: 6px;
  }

  .preset-btn {
    background: #16161b;
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    padding: 4px 10px;
    font-size: 11.5px;
    font-weight: 500;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .preset-btn:hover {
    background: #202028;
    color: var(--text-main);
    border-color: var(--border-medium);
  }

  /* Histogram */
  .histogram-container {
    padding: 12px 0 4px 0;
    overflow-x: auto;
  }

  .bars {
    display: flex;
    align-items: flex-end;
    gap: 3px;
    height: 72px;
    min-width: 100%;
  }

  .bar-col {
    flex: 1;
    min-width: 14px;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    align-items: center;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0;
    position: relative;
  }

  .bar {
    width: 100%;
    background: #23232c;
    border-radius: 2px 2px 0 0;
    transition: background 0.15s ease, height 0.2s ease;
  }

  .bar-col.in-range .bar {
    background: linear-gradient(180deg, #ffe600 0%, #f59e0b 100%);
    box-shadow: 0 0 6px rgba(255, 230, 0, 0.3);
  }

  .bar-col:hover .bar {
    background: #38bdf8 !important;
    box-shadow: 0 0 8px rgba(56, 189, 248, 0.6) !important;
  }

  .bar-label {
    font-size: 8.5px;
    color: var(--text-muted);
    margin-top: 4px;
    font-family: var(--font-mono);
    white-space: nowrap;
    display: none;
  }

  .bar-col:nth-child(3n + 1) .bar-label {
    display: block;
  }

  /* Dual Inputs */
  .inputs-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .input-field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .input-field label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .input-text {
    padding: 7px 10px;
    font-size: 12.5px;
  }
</style>
