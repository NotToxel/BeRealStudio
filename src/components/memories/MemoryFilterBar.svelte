<script lang="ts">
  import { explorerFilter, explorerData, filteredMemories, resetFilters } from '$lib/memoriesStore';
  import Search from 'lucide-svelte/icons/search';
  import X from 'lucide-svelte/icons/circle-x';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import Film from 'lucide-svelte/icons/film';
  import MessageSquare from 'lucide-svelte/icons/message-square';
  import Repeat from 'lucide-svelte/icons/repeat';
  import Filter from 'lucide-svelte/icons/filter';
  import RotateCcw from 'lucide-svelte/icons/rotate-ccw';

  let showAdvanced = false;

  $: isFiltered =
    $explorerFilter.searchQuery !== '' ||
    $explorerFilter.selectedYear !== 'all' ||
    $explorerFilter.selectedMonth !== 'all' ||
    $explorerFilter.selectedCity !== 'all' ||
    $explorerFilter.selectedCountry !== 'all' ||
    $explorerFilter.hasLocationOnly ||
    $explorerFilter.hasBtsOnly ||
    $explorerFilter.hasCaptionOnly ||
    $explorerFilter.retakesOnly;

  $: totalMemories = $explorerData?.totalCount ?? 0;
  $: countShown = $filteredMemories.length;
</script>

<div class="filter-bar-container">
  <div class="main-filter-row">
    <!-- Search Input -->
    <div class="search-input-wrap">
      <Search size={15} class="search-icon text-muted" />
      <input
        type="text"
        class="search-input"
        placeholder="Search memories by caption, location, or date..."
        bind:value={$explorerFilter.searchQuery}
      />
      {#if $explorerFilter.searchQuery}
        <button
          type="button"
          class="clear-search-btn"
          on:click={() => ($explorerFilter.searchQuery = '')}
          title="Clear search"
          aria-label="Clear search input"
        >
          <X size={13} />
        </button>
      {/if}
    </div>

    <!-- Toggle Quick Filter Chips -->
    <div class="quick-chips-row">
      <button
        type="button"
        class="filter-chip chip-location"
        class:active={$explorerFilter.hasLocationOnly}
        on:click={() => ($explorerFilter.hasLocationOnly = !$explorerFilter.hasLocationOnly)}
      >
        <MapPin size={12} />
        <span>Location</span>
      </button>

      <button
        type="button"
        class="filter-chip chip-bts"
        class:active={$explorerFilter.hasBtsOnly}
        on:click={() => ($explorerFilter.hasBtsOnly = !$explorerFilter.hasBtsOnly)}
      >
        <Film size={12} />
        <span>BTS Clips</span>
      </button>

      <button
        type="button"
        class="filter-chip chip-captions"
        class:active={$explorerFilter.hasCaptionOnly}
        on:click={() => ($explorerFilter.hasCaptionOnly = !$explorerFilter.hasCaptionOnly)}
      >
        <MessageSquare size={12} />
        <span>Captions</span>
      </button>

      <button
        type="button"
        class="filter-chip chip-retakes"
        class:active={$explorerFilter.retakesOnly}
        on:click={() => ($explorerFilter.retakesOnly = !$explorerFilter.retakesOnly)}
      >
        <Repeat size={12} />
        <span>Retakes</span>
      </button>

      <button
        type="button"
        class="filter-chip advanced-toggle"
        class:active={showAdvanced}
        on:click={() => (showAdvanced = !showAdvanced)}
        title="More filters"
      >
        <Filter size={12} />
        <span>Filters</span>
      </button>

      {#if isFiltered}
        <button
          type="button"
          class="filter-chip reset-chip"
          on:click={resetFilters}
          title="Reset all filters"
        >
          <RotateCcw size={11} />
          <span>Reset</span>
        </button>
      {/if}
    </div>

    <!-- Count Pill -->
    <div class="count-pill" class:is-active-filter={isFiltered}>
      {#if isFiltered}
        <span class="count-badge-tag">Filtered</span>
      {/if}
      <span class="count-bold">{countShown}</span>
      <span class="count-total">/ {totalMemories}</span>
    </div>
  </div>

  <!-- Advanced Dropdowns Drawer (City, Country, Year, Month) -->
  {#if showAdvanced && $explorerData}
    <div class="advanced-filter-drawer">
      {#if $explorerData.uniqueYears.length > 0}
        <div class="filter-dropdown-group">
          <label class="dropdown-label" for="filter-year-select">Year</label>
          <select
            id="filter-year-select"
            class="filter-select"
            bind:value={$explorerFilter.selectedYear}
          >
            <option value="all">All Years ({$explorerData.uniqueYears.length})</option>
            {#each $explorerData.uniqueYears as yr}
              <option value={yr}>{yr}</option>
            {/each}
          </select>
        </div>
      {/if}

      {#if $explorerData.uniqueMonths.length > 0}
        <div class="filter-dropdown-group">
          <label class="dropdown-label" for="filter-month-select">Month</label>
          <select
            id="filter-month-select"
            class="filter-select"
            bind:value={$explorerFilter.selectedMonth}
          >
            <option value="all">All Months</option>
            {#each $explorerData.uniqueMonths as mo}
              <option value={mo}>{mo}</option>
            {/each}
          </select>
        </div>
      {/if}

      {#if $explorerData.uniqueCities.length > 0}
        <div class="filter-dropdown-group">
          <label class="dropdown-label" for="filter-city-select">City</label>
          <select
            id="filter-city-select"
            class="filter-select"
            bind:value={$explorerFilter.selectedCity}
          >
            <option value="all">All Cities ({$explorerData.uniqueCities.length})</option>
            {#each $explorerData.uniqueCities as city}
              <option value={city}>{city}</option>
            {/each}
          </select>
        </div>
      {/if}

      {#if $explorerData.uniqueCountries.length > 0}
        <div class="filter-dropdown-group">
          <label class="dropdown-label" for="filter-country-select">Country</label>
          <select
            id="filter-country-select"
            class="filter-select"
            bind:value={$explorerFilter.selectedCountry}
          >
            <option value="all">All Countries ({$explorerData.uniqueCountries.length})</option>
            {#each $explorerData.uniqueCountries as country}
              <option value={country}>{country}</option>
            {/each}
          </select>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .filter-bar-container {
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: #111117;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: 10px 14px;
  }

  .main-filter-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .search-input-wrap {
    position: relative;
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 240px;
  }

  :global(.search-icon) {
    position: absolute;
    left: 12px;
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    height: 36px;
    padding: 0 34px 0 34px;
    background: #09090d;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-full);
    color: #ffffff;
    font-size: 12.5px;
    outline: none;
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
  }

  .search-input:focus {
    border-color: #ffe600;
    box-shadow: 0 0 0 2px rgba(255, 230, 0, 0.15);
  }

  .clear-search-btn {
    position: absolute;
    right: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    color: #ffffff;
    cursor: pointer;
  }

  .quick-chips-row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .filter-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    background: #181822;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    color: var(--text-secondary);
    font-size: 11.5px;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .filter-chip:hover {
    color: #ffffff;
    border-color: var(--border-medium);
    background: #20202d;
  }

  .filter-chip.active {
    font-weight: 700;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
  }

  .filter-chip.chip-location.active {
    background: rgba(16, 185, 129, 0.16);
    border-color: #10b981;
    color: #34d399;
    box-shadow: 0 2px 10px rgba(16, 185, 129, 0.25);
  }

  .filter-chip.chip-bts.active {
    background: rgba(245, 158, 11, 0.16);
    border-color: #f59e0b;
    color: #fbbf24;
    box-shadow: 0 2px 10px rgba(245, 158, 11, 0.25);
  }

  .filter-chip.chip-captions.active {
    background: rgba(56, 189, 248, 0.16);
    border-color: #38bdf8;
    color: #38bdf8;
    box-shadow: 0 2px 10px rgba(56, 189, 248, 0.25);
  }

  .filter-chip.chip-retakes.active {
    background: rgba(168, 85, 247, 0.16);
    border-color: #a855f7;
    color: #c084fc;
    box-shadow: 0 2px 10px rgba(168, 85, 247, 0.25);
  }

  .filter-chip.advanced-toggle.active {
    background: rgba(139, 92, 246, 0.18);
    border-color: #8b5cf6;
    color: #a78bfa;
    box-shadow: 0 2px 10px rgba(139, 92, 246, 0.25);
  }

  .reset-chip {
    background: rgba(244, 63, 94, 0.12);
    border-color: rgba(244, 63, 94, 0.4);
    color: #fda4af;
  }

  .reset-chip:hover {
    background: #f43f5e;
    color: #ffffff;
    border-color: #f43f5e;
    box-shadow: 0 2px 10px rgba(244, 63, 94, 0.4);
  }

  .count-pill {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 5px 12px;
    background: #181824;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    font-size: 11.5px;
    margin-left: auto;
    transition: all 0.2s ease;
  }

  .count-pill.is-active-filter {
    background: rgba(56, 189, 248, 0.1);
    border-color: rgba(56, 189, 248, 0.5);
    color: #38bdf8;
    box-shadow: 0 2px 10px rgba(56, 189, 248, 0.2);
  }

  .count-bold {
    color: #ffffff;
    font-weight: 700;
  }

  .advanced-filter-drawer {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 10px;
    padding-top: 10px;
    border-top: 1px solid var(--border-subtle);
    animation: drawerFade 0.15s ease-out;
  }

  @keyframes drawerFade {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .filter-dropdown-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .dropdown-label {
    font-size: 10.5px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .filter-select {
    height: 32px;
    padding: 0 8px;
    background: #09090d;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-sm);
    color: #ffffff;
    font-size: 12px;
    outline: none;
    cursor: pointer;
  }

  .filter-select:focus {
    border-color: #ffe600;
  }
</style>
