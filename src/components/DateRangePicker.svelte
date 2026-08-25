<script lang="ts">
  import type { MonthCount } from '$lib/types';
  import Calendar from 'lucide-svelte/icons/calendar';
  import BarChart3 from 'lucide-svelte/icons/chart-bar';
  import RotateCcw from 'lucide-svelte/icons/rotate-ccw';

  export let histogram: MonthCount[] = [];
  export let minDate: string = '';
  export let maxDate: string = '';
  export let startDate: string = '';
  export let endDate: string = '';
  export let totalCount: number = 0;
  export let selectedCount: number = 0;

  $: maxBarCount = Math.max(...histogram.map((h) => h.count), 1);

  // Month date range helper
  function isMonthInRange(monthStr: string): boolean {
    if (!startDate && !endDate) return true;
    const mStart = `${monthStr}-01`;
    // Month end estimate (using day 31 is fine for string ISO comparison)
    const mEnd = `${monthStr}-31`;
    if (startDate && mEnd < startDate) return false;
    if (endDate && mStart > endDate) return false;
    return true;
  }

  // Reactively calculate selected entries count from histogram
  $: {
    if (!startDate && !endDate) {
      selectedCount = totalCount;
    } else if (histogram.length > 0) {
      const count = histogram
        .filter((h) => isMonthInRange(h.month))
        .reduce((sum, h) => sum + h.count, 0);
      selectedCount = count;
    } else {
      selectedCount = totalCount;
    }
  }

  let activePreset: 'all' | '30d' | '6m' | '1y' | 'custom' = 'all';

  function setPreset(preset: 'all' | '30d' | '6m' | '1y') {
    activePreset = preset;
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
    activePreset = 'custom';
    startDate = `${month}-01`;
    const parts = month.split('-');
    const y = parseInt(parts[0], 10);
    const m = parseInt(parts[1], 10);
    const lastDay = new Date(y, m, 0).getDate();
    endDate = `${month}-${String(lastDay).padStart(2, '0')}`;
  }

  function resetRange() {
    setPreset('all');
  }

  // Format month for label "Jan 23"
  const MONTH_NAMES = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  function formatMonthLabel(mStr: string): string {
    const parts = mStr.split('-');
    if (parts.length < 2) return mStr;
    const mIdx = parseInt(parts[1], 10) - 1;
    const yr = parts[0].slice(2);
    return `${MONTH_NAMES[mIdx]} '${yr}`;
  }

  $: isFiltered = Boolean((startDate && startDate !== minDate) || (endDate && endDate !== maxDate));
</script>

<div class="range-picker card">
  <div class="header">
    <div class="title-group">
      <BarChart3 size={16} class="text-amber-400" />
      <span class="title-sm font-semibold">Date Range &amp; Timeline</span>
      {#if totalCount > 0}
        <span class="badge {isFiltered ? 'badge-yellow' : 'badge-neutral'} font-mono">
          {selectedCount} / {totalCount} memories
        </span>
      {/if}
    </div>

    <!-- Presets -->
    <div class="presets">
      <button
        type="button"
        class="preset-btn"
        class:active={!isFiltered || activePreset === 'all'}
        on:click={() => setPreset('all')}
      >
        All Time
      </button>
      <button
        type="button"
        class="preset-btn"
        class:active={activePreset === '30d'}
        on:click={() => setPreset('30d')}
      >
        30 Days
      </button>
      <button
        type="button"
        class="preset-btn"
        class:active={activePreset === '6m'}
        on:click={() => setPreset('6m')}
      >
        6 Months
      </button>
      <button
        type="button"
        class="preset-btn"
        class:active={activePreset === '1y'}
        on:click={() => setPreset('1y')}
      >
        1 Year
      </button>
      {#if isFiltered}
        <button
          type="button"
          class="reset-btn"
          on:click={resetRange}
          title="Reset to full date range"
        >
          <RotateCcw size={12} />
          <span>Reset</span>
        </button>
      {/if}
    </div>
  </div>

  <!-- Density Histogram -->
  {#if histogram.length > 0}
    <div class="histogram-wrapper">
      <div class="histogram-container">
        <div class="bars">
          {#each histogram as item, idx}
            {@const inRange = isMonthInRange(item.month)}
            {@const heightPercent = Math.max((item.count / maxBarCount) * 100, 10)}
            {@const isFirstOfYear = item.month.endsWith('-01')}
            <button
              type="button"
              class="bar-col"
              class:in-range={inRange}
              class:first-of-year={isFirstOfYear}
              title="{formatMonthLabel(item.month)}: {item.count} BeReals"
              on:click={() => handleBarClick(item.month)}
            >
              <div class="bar" style="height: {heightPercent}%;"></div>
              {#if idx === 0 || idx === histogram.length - 1 || isFirstOfYear || idx % 4 === 0}
                <span class="bar-label">{formatMonthLabel(item.month)}</span>
              {/if}
            </button>
          {/each}
        </div>
      </div>
    </div>
  {/if}

  <!-- Dual Date Pickers -->
  <div class="inputs-row">
    <div class="input-field">
      <label for="start-date">
        <Calendar size={13} class="text-secondary" />
        <span>From Date</span>
      </label>
      <input
        id="start-date"
        type="date"
        class="input-text font-mono"
        bind:value={startDate}
        on:change={() => (activePreset = 'custom')}
        min={minDate}
        max={endDate || maxDate}
      />
    </div>

    <div class="input-field">
      <label for="end-date">
        <Calendar size={13} class="text-secondary" />
        <span>To Date</span>
      </label>
      <input
        id="end-date"
        type="date"
        class="input-text font-mono"
        bind:value={endDate}
        on:change={() => (activePreset = 'custom')}
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
    gap: 14px;
    background: #111116;
    padding: 16px;
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
    gap: 8px;
  }

  .presets {
    display: flex;
    align-items: center;
    gap: 5px;
    background: #0d0d12;
    padding: 3px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
  }

  .preset-btn {
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-muted);
    padding: 3px 9px;
    font-size: 11.5px;
    font-weight: 500;
    border-radius: calc(var(--radius-sm) - 2px);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .preset-btn:hover {
    color: var(--text-main);
    background: #191922;
  }

  .preset-btn.active {
    background: rgba(255, 230, 0, 0.16);
    color: #ffe600;
    border-color: rgba(255, 230, 0, 0.3);
    font-weight: 600;
  }

  .reset-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: rgba(239, 68, 68, 0.12);
    border: 1px solid rgba(239, 68, 68, 0.25);
    color: #f87171;
    padding: 3px 8px;
    font-size: 11px;
    border-radius: calc(var(--radius-sm) - 2px);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .reset-btn:hover {
    background: rgba(239, 68, 68, 0.22);
    color: #fca5a5;
  }

  /* Histogram */
  .histogram-wrapper {
    background: #0a0a0e;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 12px 10px 6px 10px;
  }

  .histogram-container {
    overflow-x: auto;
    scrollbar-width: thin;
  }

  .bars {
    display: flex;
    align-items: flex-end;
    gap: 3px;
    height: 76px;
    min-width: 100%;
    padding-bottom: 2px;
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
    background: #1e1e26;
    border-radius: 2px 2px 0 0;
    transition: background 0.15s ease, height 0.2s ease, opacity 0.15s ease;
    opacity: 0.35;
  }

  .bar-col.in-range .bar {
    background: linear-gradient(180deg, #ffe600 0%, #d97706 100%);
    box-shadow: 0 0 6px rgba(255, 230, 0, 0.25);
    opacity: 1;
  }

  .bar-col:hover .bar {
    background: #38bdf8 !important;
    box-shadow: 0 0 8px rgba(56, 189, 248, 0.6) !important;
    opacity: 1 !important;
  }

  .bar-label {
    font-size: 8px;
    color: var(--text-muted);
    margin-top: 4px;
    font-family: var(--font-mono);
    white-space: nowrap;
    user-select: none;
  }

  .bar-col.first-of-year .bar-label {
    color: var(--text-secondary);
    font-weight: 600;
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
    font-size: 11.5px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .input-text {
    padding: 6px 10px;
    font-size: 12.5px;
    background: #0d0d12;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-main);
  }

  .badge-neutral {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-secondary);
  }
</style>
