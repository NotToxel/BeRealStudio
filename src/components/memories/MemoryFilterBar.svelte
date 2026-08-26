<script lang="ts">
  import {
    explorerFilter,
    explorerData,
    filteredMemories,
    explorerFilterCounts,
    citiesByCountry,
    resetFilters,
    toggleVideoOnlyFilter,
    toggleBtsOnlyFilter,
  } from '$lib/memoriesStore';
  import CountryFlag from '../common/CountryFlag.svelte';
  import Search from 'lucide-svelte/icons/search';
  import X from 'lucide-svelte/icons/circle-x';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import Film from 'lucide-svelte/icons/film';
  import Clapperboard from 'lucide-svelte/icons/clapperboard';
  import MessageSquare from 'lucide-svelte/icons/message-square';
  import Filter from 'lucide-svelte/icons/filter';
  import RotateCcw from 'lucide-svelte/icons/rotate-ccw';
  import ChevronDown from 'lucide-svelte/icons/chevron-down';
  import Check from 'lucide-svelte/icons/check';
  import Globe from 'lucide-svelte/icons/globe';
  import Building from 'lucide-svelte/icons/building';
  import Calendar from 'lucide-svelte/icons/calendar';

  let showAdvanced = false;
  let activeOpenDropdown: 'country' | 'city' | 'suburb' | 'year' | 'month' | null = null;

  function toggleDropdown(name: 'country' | 'city' | 'suburb' | 'year' | 'month', e?: MouseEvent) {
    e?.stopPropagation();
    activeOpenDropdown = activeOpenDropdown === name ? null : name;
  }

  function closeDropdowns() {
    activeOpenDropdown = null;
  }

  function selectCountry(val: string) {
    $explorerFilter.selectedCountry = val;
    closeDropdowns();
  }

  function selectCity(val: string) {
    $explorerFilter.selectedCity = val;
    closeDropdowns();
  }

  function selectSuburb(val: string) {
    $explorerFilter.selectedSuburb = val;
    closeDropdowns();
  }

  function selectYear(val: number | 'all') {
    $explorerFilter.selectedYear = val;
    closeDropdowns();
  }

  function selectMonth(val: string) {
    $explorerFilter.selectedMonth = val;
    closeDropdowns();
  }

  $: isFiltered =
    $explorerFilter.searchQuery !== '' ||
    $explorerFilter.selectedYear !== 'all' ||
    $explorerFilter.selectedMonth !== 'all' ||
    $explorerFilter.selectedCountry !== 'all' ||
    $explorerFilter.selectedCity !== 'all' ||
    $explorerFilter.selectedSuburb !== 'all' ||
    $explorerFilter.hasLocationOnly ||
    $explorerFilter.hasBtsOnly ||
    $explorerFilter.hasCaptionOnly ||
    $explorerFilter.hasVideoOnly;

  $: totalMemories = $explorerData?.totalCount ?? 0;
  $: countShown = $filteredMemories.length;
  $: counts = $explorerFilterCounts;
</script>

<svelte:window on:click={closeDropdowns} />

<div class="filter-bar-container">
  <div class="main-filter-row">
    <!-- Search Input -->
    <div class="search-input-wrap">
      <Search size={15} class="search-icon text-muted" />
      <input
        type="text"
        class="search-input"
        placeholder="Search memories by caption, suburb, city, or date..."
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

    <!-- Toggle Quick Filter Chips (with Live Dynamic Compound Counts) -->
    <div class="quick-chips-row">
      <button
        type="button"
        class="filter-chip chip-location"
        class:active={$explorerFilter.hasLocationOnly}
        on:click={() => ($explorerFilter.hasLocationOnly = !$explorerFilter.hasLocationOnly)}
        title="Show memories with GPS location tag ({counts.locationCount})"
      >
        <MapPin size={12} />
        <span>Location</span>
        {#if counts.locationCount > 0}
          <span class="chip-count-badge">{counts.locationCount}</span>
        {/if}
      </button>

      <button
        type="button"
        class="filter-chip chip-video"
        class:active={$explorerFilter.hasVideoOnly}
        on:click={toggleVideoOnlyFilter}
        title="Show memories with video ({counts.videoCount})"
      >
        <Clapperboard size={12} />
        <span>Videos</span>
        {#if counts.videoCount > 0}
          <span class="chip-count-badge">{counts.videoCount}</span>
        {/if}
      </button>

      <button
        type="button"
        class="filter-chip chip-bts"
        class:active={$explorerFilter.hasBtsOnly}
        on:click={toggleBtsOnlyFilter}
        title="Show memories with Behind-the-Scenes live clip ({counts.btsCount})"
      >
        <Film size={12} />
        <span>BTS Clips</span>
        {#if counts.btsCount > 0}
          <span class="chip-count-badge">{counts.btsCount}</span>
        {/if}
      </button>

      <button
        type="button"
        class="filter-chip chip-captions"
        class:active={$explorerFilter.hasCaptionOnly}
        on:click={() => ($explorerFilter.hasCaptionOnly = !$explorerFilter.hasCaptionOnly)}
        title="Show memories with caption text ({counts.captionCount})"
      >
        <MessageSquare size={12} />
        <span>Captions</span>
        {#if counts.captionCount > 0}
          <span class="chip-count-badge">{counts.captionCount}</span>
        {/if}
      </button>

      <div class="chips-divider"></div>

      <button
        type="button"
        class="filter-chip advanced-toggle"
        class:active={showAdvanced}
        on:click={() => (showAdvanced = !showAdvanced)}
        title="More filters (Country, City, Year, Month)"
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

    <!-- Count Pill with Smooth Micro-Glow Indicator -->
    <div class="count-pill" class:is-active-filter={isFiltered}>
      {#if isFiltered}
        <span class="count-badge-tag">Filtered</span>
      {/if}
      <span class="count-bold">{countShown}</span>
      <span class="count-total">/ {totalMemories}</span>
    </div>
  </div>

  <!-- Advanced Custom Dropdowns (Country with flags, Grouped Cities, Suburbs, Years, Months) -->
  {#if showAdvanced && $explorerData}
    <div class="advanced-filter-drawer" role="presentation" on:click|stopPropagation>
      <!-- Country Dropdown -->
      {#if $explorerData.uniqueCountries.length > 0}
        <div class="custom-select-group">
          <span class="custom-select-label">Country</span>
          <button
            type="button"
            class="custom-select-trigger"
            class:is-selected={$explorerFilter.selectedCountry !== 'all'}
            on:click={(e) => toggleDropdown('country', e)}
          >
            <div class="trigger-left-content">
              {#if $explorerFilter.selectedCountry !== 'all'}
                <CountryFlag country={$explorerFilter.selectedCountry} size="sm" />
                <span class="trigger-label-text">{$explorerFilter.selectedCountry}</span>
              {:else}
                <Globe size={13} class="text-secondary" />
                <span class="trigger-label-text text-secondary">All Countries</span>
              {/if}
            </div>
            <ChevronDown size={13} class="chevron-arrow" />
          </button>

          {#if activeOpenDropdown === 'country'}
            <div class="custom-popover-menu">
              <button
                type="button"
                class="popover-item"
                class:active={$explorerFilter.selectedCountry === 'all'}
                on:click={() => selectCountry('all')}
              >
                <div class="popover-item-left">
                  <Globe size={14} class="text-secondary" />
                  <span>All Countries</span>
                </div>
                <span class="item-count-badge">{$explorerData.uniqueCountries.length}</span>
              </button>

              <div class="popover-divider"></div>

              {#each $explorerData.uniqueCountries.filter((c) => (counts.byCountry.get(c) || 0) > 0) as country}
                {@const cnt = counts.byCountry.get(country) || 0}
                <button
                  type="button"
                  class="popover-item"
                  class:active={$explorerFilter.selectedCountry === country}
                  on:click={() => selectCountry(country)}
                >
                  <div class="popover-item-left">
                    <CountryFlag {country} size="sm" />
                    <span class="popover-item-title">{country}</span>
                  </div>
                  <span class="item-count-badge">{cnt}</span>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- City Dropdown (Grouped by Country with Flags) -->
      {#if $citiesByCountry.length > 0}
        <div class="custom-select-group">
          <span class="custom-select-label">City</span>
          <button
            type="button"
            class="custom-select-trigger"
            class:is-selected={$explorerFilter.selectedCity !== 'all'}
            on:click={(e) => toggleDropdown('city', e)}
          >
            <div class="trigger-left-content">
              {#if $explorerFilter.selectedCity !== 'all'}
                <Building size={13} class="text-sky-400" />
                <span class="trigger-label-text">{$explorerFilter.selectedCity}</span>
              {:else}
                <Building size={13} class="text-secondary" />
                <span class="trigger-label-text text-secondary">All Cities</span>
              {/if}
            </div>
            <ChevronDown size={13} class="chevron-arrow" />
          </button>

          {#if activeOpenDropdown === 'city'}
            <div class="custom-popover-menu max-height-scroll">
              <button
                type="button"
                class="popover-item"
                class:active={$explorerFilter.selectedCity === 'all'}
                on:click={() => selectCity('all')}
              >
                <div class="popover-item-left">
                  <Building size={14} class="text-secondary" />
                  <span>All Cities</span>
                </div>
                <span class="item-count-badge">{$explorerData.uniqueCities.length}</span>
              </button>

              {#each $citiesByCountry as group}
                {#if group.totalPosts > 0}
                  <div class="popover-group-header">
                    <CountryFlag country={group.country} size="sm" />
                    <span class="group-country-name">{group.country}</span>
                    <span class="group-total-badge">{group.totalPosts}</span>
                  </div>

                  {#each group.cities.filter((c) => c.count > 0) as cityItem}
                    <button
                      type="button"
                      class="popover-item sub-item"
                      class:active={$explorerFilter.selectedCity === cityItem.name}
                      on:click={() => selectCity(cityItem.name)}
                    >
                      <div class="popover-item-left">
                        <span class="popover-item-title">{cityItem.name}</span>
                      </div>
                      <span class="item-count-badge">{cityItem.count}</span>
                    </button>
                  {/each}
                {/if}
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Suburb / Area Dropdown -->
      {#if $explorerData.uniqueSuburbs && $explorerData.uniqueSuburbs.length > 0}
        <div class="custom-select-group">
          <span class="custom-select-label">Area / Suburb</span>
          <button
            type="button"
            class="custom-select-trigger"
            class:is-selected={$explorerFilter.selectedSuburb !== 'all'}
            on:click={(e) => toggleDropdown('suburb', e)}
          >
            <div class="trigger-left-content">
              {#if $explorerFilter.selectedSuburb !== 'all'}
                <MapPin size={13} class="text-emerald-400" />
                <span class="trigger-label-text">{$explorerFilter.selectedSuburb}</span>
              {:else}
                <MapPin size={13} class="text-secondary" />
                <span class="trigger-label-text text-secondary">All Areas</span>
              {/if}
            </div>
            <ChevronDown size={13} class="chevron-arrow" />
          </button>

          {#if activeOpenDropdown === 'suburb'}
            <div class="custom-popover-menu max-height-scroll">
              <button
                type="button"
                class="popover-item"
                class:active={$explorerFilter.selectedSuburb === 'all'}
                on:click={() => selectSuburb('all')}
              >
                <span>All Areas</span>
                <span class="item-count-badge">{$explorerData.uniqueSuburbs.length}</span>
              </button>

              <div class="popover-divider"></div>

              {#each $explorerData.uniqueSuburbs.filter((s) => (counts.bySuburb.get(s) || 0) > 0) as suburb}
                {@const cnt = counts.bySuburb.get(suburb) || 0}
                <button
                  type="button"
                  class="popover-item"
                  class:active={$explorerFilter.selectedSuburb === suburb}
                  on:click={() => selectSuburb(suburb)}
                >
                  <span class="popover-item-title">{suburb}</span>
                  <span class="item-count-badge">{cnt}</span>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Year Dropdown -->
      {#if $explorerData.uniqueYears.length > 0}
        <div class="custom-select-group">
          <span class="custom-select-label">Year</span>
          <button
            type="button"
            class="custom-select-trigger"
            class:is-selected={$explorerFilter.selectedYear !== 'all'}
            on:click={(e) => toggleDropdown('year', e)}
          >
            <div class="trigger-left-content">
              <Calendar size={13} class={$explorerFilter.selectedYear !== 'all' ? 'text-amber-400' : 'text-secondary'} />
              <span class="trigger-label-text">
                {$explorerFilter.selectedYear !== 'all' ? $explorerFilter.selectedYear : 'All Years'}
              </span>
            </div>
            <ChevronDown size={13} class="chevron-arrow" />
          </button>

          {#if activeOpenDropdown === 'year'}
            <div class="custom-popover-menu">
              <button
                type="button"
                class="popover-item"
                class:active={$explorerFilter.selectedYear === 'all'}
                on:click={() => selectYear('all')}
              >
                <span>All Years</span>
              </button>

              <div class="popover-divider"></div>

              {#each $explorerData.uniqueYears.filter((y) => (counts.byYear.get(y) || 0) > 0) as yr}
                {@const cnt = counts.byYear.get(yr) || 0}
                <button
                  type="button"
                  class="popover-item"
                  class:active={$explorerFilter.selectedYear === yr}
                  on:click={() => selectYear(yr)}
                >
                  <span class="popover-item-title">{yr}</span>
                  <span class="item-count-badge">{cnt}</span>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Month Dropdown -->
      {#if $explorerData.uniqueMonths.length > 0}
        <div class="custom-select-group">
          <span class="custom-select-label">Month</span>
          <button
            type="button"
            class="custom-select-trigger"
            class:is-selected={$explorerFilter.selectedMonth !== 'all'}
            on:click={(e) => toggleDropdown('month', e)}
          >
            <div class="trigger-left-content">
              <Calendar size={13} class={$explorerFilter.selectedMonth !== 'all' ? 'text-amber-400' : 'text-secondary'} />
              <span class="trigger-label-text">
                {#if $explorerFilter.selectedMonth !== 'all'}
                  {@const [y, m] = $explorerFilter.selectedMonth.split('-')}
                  {@const mName = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'][parseInt(m) - 1]}
                  {mName} {y}
                {:else}
                  All Months
                {/if}
              </span>
            </div>
            <ChevronDown size={13} class="chevron-arrow" />
          </button>

          {#if activeOpenDropdown === 'month'}
            <div class="custom-popover-menu max-height-scroll">
              <button
                type="button"
                class="popover-item"
                class:active={$explorerFilter.selectedMonth === 'all'}
                on:click={() => selectMonth('all')}
              >
                <span>All Months</span>
              </button>

              <div class="popover-divider"></div>

              {#each $explorerData.uniqueMonths.filter((mo) => (counts.byMonth.get(mo) || 0) > 0) as mo}
                {@const cnt = counts.byMonth.get(mo) || 0}
                {@const y = parseInt(mo.slice(0, 4))}
                {@const m = parseInt(mo.slice(5, 7))}
                {@const mName = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'][m - 1] || mo}
                <button
                  type="button"
                  class="popover-item"
                  class:active={$explorerFilter.selectedMonth === mo}
                  on:click={() => selectMonth(mo)}
                >
                  <span class="popover-item-title">{mName} {y}</span>
                  <span class="item-count-badge">{cnt}</span>
                </button>
              {/each}
            </div>
          {/if}
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

  .filter-chip.chip-video.active {
    background: rgba(236, 72, 153, 0.16);
    border-color: #ec4899;
    color: #f472b6;
    box-shadow: 0 2px 10px rgba(236, 72, 153, 0.25);
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

  .filter-chip.advanced-toggle.active {
    background: rgba(139, 92, 246, 0.18);
    border-color: #8b5cf6;
    color: #a78bfa;
    box-shadow: 0 2px 10px rgba(139, 92, 246, 0.25);
  }

  .chips-divider {
    width: 1px;
    height: 20px;
    background: var(--border-subtle);
    margin: 0 2px;
  }

  .chip-count-badge {
    font-size: 9.5px;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: var(--radius-full);
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .filter-chip.active .chip-count-badge {
    background: rgba(255, 255, 255, 0.2);
    color: inherit;
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
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 12px;
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

  .custom-select-group {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .custom-select-label {
    font-size: 10.5px;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .custom-select-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    height: 34px;
    padding: 0 10px;
    background: #09090e;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-sm);
    color: #ffffff;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s ease;
    width: 100%;
    text-align: left;
  }

  .custom-select-trigger:hover {
    background: #14141c;
    border-color: var(--border-strong);
  }

  .custom-select-trigger.is-selected {
    background: rgba(56, 189, 248, 0.08);
    border-color: #38bdf8;
    color: #38bdf8;
    font-weight: 600;
    box-shadow: 0 2px 8px rgba(56, 189, 248, 0.18);
  }

  .trigger-left-content {
    display: flex;
    align-items: center;
    gap: 6px;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    flex: 1;
  }

  .trigger-label-text {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  :global(.chevron-arrow) {
    color: var(--text-muted);
    flex-shrink: 0;
    transition: transform 0.15s ease;
  }

  /* Custom Popover Glass Menu */
  .custom-popover-menu {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    min-width: 200px;
    max-width: 280px;
    background: #12121a;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: var(--radius-md);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.85);
    padding: 6px;
    z-index: 120;
    display: flex;
    flex-direction: column;
    gap: 2px;
    animation: popoverFadeIn 0.14s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .custom-popover-menu.max-height-scroll {
    max-height: 260px;
    overflow-y: auto;
  }

  @keyframes popoverFadeIn {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .popover-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 10px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: #f4f4f5;
    font-size: 12px;
    cursor: pointer;
    text-align: left;
    transition: all 0.12s ease;
    width: 100%;
  }

  .popover-item:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
  }

  .popover-item.active {
    background: rgba(56, 189, 248, 0.15);
    color: #38bdf8;
    font-weight: 700;
  }

  .popover-item:disabled {
    opacity: 0.28;
    cursor: not-allowed;
    pointer-events: none;
    filter: grayscale(80%);
  }

  .popover-item.sub-item {
    padding-left: 18px;
  }

  .popover-item-left {
    display: flex;
    align-items: center;
    gap: 6px;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .popover-item-title {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .item-count-badge {
    font-size: 10.5px;
    font-weight: 600;
    color: var(--text-muted);
    background: rgba(255, 255, 255, 0.06);
    padding: 1px 6px;
    border-radius: var(--radius-full);
  }

  .popover-group-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 10px 4px 8px;
    font-size: 11px;
    font-weight: 700;
    color: #a1a1aa;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    margin-top: 4px;
  }

  .group-country-name {
    flex: 1;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .group-total-badge {
    font-size: 10px;
    color: var(--text-muted);
  }

  .popover-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.06);
    margin: 4px 0;
  }
</style>
