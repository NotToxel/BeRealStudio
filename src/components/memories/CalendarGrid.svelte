<script lang="ts">
  import {
    explorerData,
    calendarCurrentMonth,
    memoriesByDate,
    rawMemoriesByDate,
    openFeedAt,
    openContextMenu,
    activeExplorerView,
    activeFeedMemory,
    explorerFilterCounts,
    activeFilterCount,
    resetFilters,
    showMemoryDebugBadges,
    memoryHeaderSettings,
  } from '$lib/memoriesStore';
  import DualCameraFrame from './DualCameraFrame.svelte';
  import ChevronLeft from 'lucide-svelte/icons/chevron-left';
  import ChevronRight from 'lucide-svelte/icons/chevron-right';
  import ChevronDown from 'lucide-svelte/icons/chevron-down';
  import CalendarIcon from 'lucide-svelte/icons/calendar';
  import Grid from 'lucide-svelte/icons/layout-grid';
  import Sparkles from 'lucide-svelte/icons/sparkles';

  const weekdays = ['MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT', 'SUN'];
  const monthNames = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

  let isPickerOpen = false;

  // Parse active month key "YYYY-MM"
  $: yearMonth = $calendarCurrentMonth || ($explorerData?.uniqueMonths && $explorerData.uniqueMonths.length > 0 ? $explorerData.uniqueMonths[$explorerData.uniqueMonths.length - 1] : '2024-08');
  $: currentYear = parseInt(yearMonth.slice(0, 4), 10) || new Date().getFullYear();
  $: currentMonthNum = parseInt(yearMonth.slice(5, 7), 10) || (new Date().getMonth() + 1); // 1-12

  // Selected year inside picker popover
  let pickerYear = 2024;
  $: if (currentYear) pickerYear = currentYear;

  // Month Title e.g. "August 2024"
  $: monthTitle = (() => {
    try {
      const d = new Date(currentYear, currentMonthNum - 1, 1);
      return d.toLocaleDateString('en-US', { month: 'long', year: 'numeric' });
    } catch {
      return yearMonth;
    }
  })();

  // Days in current month
  $: daysInMonth = new Date(currentYear, currentMonthNum, 0).getDate();

  // First day of month weekday offset (0 = Monday, 6 = Sunday)
  $: startDayOffset = (() => {
    const firstDay = new Date(currentYear, currentMonthNum - 1, 1).getDay();
    // JS getDay(): 0 = Sunday, 1 = Monday. We want Monday = 0
    return firstDay === 0 ? 6 : firstDay - 1;
  })();

  // Available months & years from dataset
  $: availableMonths = $explorerData?.uniqueMonths ?? [];
  $: uniqueYears = $explorerData?.uniqueYears ?? [currentYear];
  $: availableYears = [...new Set(availableMonths.map((m) => parseInt(m.slice(0, 4), 10)))].sort((a, b) => a - b);
  $: hasPrevYear = availableYears.some((y) => y < pickerYear);
  $: hasNextYear = availableYears.some((y) => y > pickerYear);
  $: currentMonthIdx = availableMonths.indexOf(yearMonth);
  $: hasPrevMonth = currentMonthIdx > 0;
  $: hasNextMonth = currentMonthIdx < availableMonths.length - 1;

  $: activeFilters = $activeFilterCount;
  $: isFiltering = activeFilters > 0;

  // All dates with matching memories across the entire archive sorted chronologically
  $: allMatchingDates = (() => {
    const dates: string[] = [];
    for (const [dateStr, posts] of $memoriesByDate.entries()) {
      if (posts.length > 0) {
        dates.push(dateStr);
      }
    }
    dates.sort();
    return dates;
  })();

  let focusedMatchingDate: string | null = null;
  let focusedDateTimer: any = null;

  function jumpToNextMatchingPost() {
    if (allMatchingDates.length === 0) return;
    const currentRef = focusedMatchingDate || `${currentYear}-${String(currentMonthNum).padStart(2, '0')}-01`;
    const nextDates = allMatchingDates.filter((d) => d > currentRef);
    const targetDate = nextDates.length > 0 ? nextDates[0] : allMatchingDates[0];
    jumpToSpecificDate(targetDate);
  }

  function jumpToPrevMatchingPost() {
    if (allMatchingDates.length === 0) return;
    const currentRef = focusedMatchingDate || `${currentYear}-${String(currentMonthNum).padStart(2, '0')}-31`;
    const prevDates = allMatchingDates.filter((d) => d < currentRef);
    const targetDate = prevDates.length > 0 ? prevDates[prevDates.length - 1] : allMatchingDates[allMatchingDates.length - 1];
    jumpToSpecificDate(targetDate);
  }

  function jumpToSpecificDate(targetDate: string) {
    const targetMonth = targetDate.slice(0, 7);
    if (targetMonth !== yearMonth) {
      calendarCurrentMonth.set(targetMonth);
    }
    focusedMatchingDate = targetDate;
    clearTimeout(focusedDateTimer);
    focusedDateTimer = setTimeout(() => {
      focusedMatchingDate = null;
    }, 3200);
  }

  // Month matching statistics
  $: monthStats = (() => {
    let totalInMonth = 0;
    let matchInMonth = 0;
    for (let day = 1; day <= daysInMonth; day++) {
      const dStr = `${currentYear}-${String(currentMonthNum).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
      const raw = $rawMemoriesByDate.get(dStr) || [];
      const match = $memoriesByDate.get(dStr) || [];
      if (raw.length > 0) totalInMonth += raw.length;
      if (match.length > 0) matchInMonth += match.length;
    }
    return { total: totalInMonth, matches: matchInMonth };
  })();

  function prevMonth() {
    if (hasPrevMonth) {
      calendarCurrentMonth.set(availableMonths[currentMonthIdx - 1]);
    }
  }

  function nextMonth() {
    if (hasNextMonth) {
      calendarCurrentMonth.set(availableMonths[currentMonthIdx + 1]);
    }
  }

  function selectMonthFromPicker(mIdx: number) {
    const targetMonthKey = `${pickerYear}-${String(mIdx + 1).padStart(2, '0')}`;
    if (!availableMonths.includes(targetMonthKey)) return;
    calendarCurrentMonth.set(targetMonthKey);
    isPickerOpen = false;
  }

  function jumpToLatestMonth() {
    if (availableMonths.length > 0) {
      calendarCurrentMonth.set(availableMonths[availableMonths.length - 1]);
    }
    isPickerOpen = false;
  }

  function handleDayClick(dayNum: number) {
    const dateStr = `${currentYear}-${String(currentMonthNum).padStart(2, '0')}-${String(dayNum).padStart(2, '0')}`;
    const raw = $rawMemoriesByDate.get(dateStr);
    const posts = $memoriesByDate.get(dateStr);
    if (posts && posts.length > 0) {
      openFeedAt(posts[0]);
    } else if (raw && raw.length > 0) {
      openFeedAt(raw[0]);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ($activeFeedMemory) return;
    const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
    if (tag === 'input' || tag === 'textarea' || tag === 'select') return;

    if (e.key === 'ArrowLeft') {
      if (hasPrevMonth) {
        e.preventDefault();
        prevMonth();
      }
    } else if (e.key === 'ArrowRight') {
      if (hasNextMonth) {
        e.preventDefault();
        nextMonth();
      }
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="calendar-view-container">
  <!-- Month Switcher Header with Centered Nav Cluster -->
  <div class="calendar-header-bar">
    <div class="calendar-nav-cluster">
      <button
        type="button"
        class="month-nav-btn prev-btn"
        disabled={!hasPrevMonth}
        on:click={prevMonth}
        title="Previous Month (or scroll up)"
        aria-label="Previous month"
      >
        <ChevronLeft size={16} />
      </button>

      <div class="month-title-group">
        <button
          type="button"
          class="month-picker-trigger-btn"
          class:is-active={isPickerOpen}
          on:click={() => (isPickerOpen = !isPickerOpen)}
          title="Click to open Month & Year picker"
          aria-expanded={isPickerOpen}
        >
          <CalendarIcon size={16} class="text-sky-400" />
          <span class="month-title-text">{monthTitle}</span>
          <ChevronDown size={14} class="picker-chevron {isPickerOpen ? 'rotate-180' : ''}" />
        </button>

        <!-- Rich Month / Year Popover -->
        {#if isPickerOpen}
          <div class="month-picker-popover">
            <div class="picker-year-row">
              <button
                type="button"
                class="picker-nav-btn"
                disabled={!hasPrevYear}
                on:click={() => hasPrevYear && pickerYear--}
                title={hasPrevYear ? "Previous Year" : "No memories in earlier years"}
              >
                <ChevronLeft size={14} />
              </button>
              <span class="picker-year-title">{pickerYear}</span>
              <button
                type="button"
                class="picker-nav-btn"
                disabled={!hasNextYear}
                on:click={() => hasNextYear && pickerYear++}
                title={hasNextYear ? "Next Year" : "No memories in later years"}
              >
                <ChevronRight size={14} />
              </button>
            </div>

            <div class="picker-months-grid">
              {#each monthNames as mName, mIdx}
                {@const mKey = `${pickerYear}-${String(mIdx + 1).padStart(2, '0')}`}
                {@const isCurrent = mKey === yearMonth}
                {@const hasData = availableMonths.includes(mKey)}
                {@const cnt = $explorerFilterCounts?.byMonth.get(mKey) || 0}

                <button
                  type="button"
                  class="picker-month-btn"
                  class:is-selected={isCurrent}
                  class:has-memories={hasData}
                  disabled={!hasData}
                  on:click={() => hasData && selectMonthFromPicker(mIdx)}
                  title={hasData ? `${mName} ${pickerYear}: ${cnt} BeReals` : `No memories in ${mName} ${pickerYear}`}
                >
                  <span class="month-name-text">{mName}</span>
                  {#if cnt > 0}
                    <span class="month-count-tag">{cnt}</span>
                  {:else if hasData}
                    <span class="month-data-dot"></span>
                  {/if}
                </button>
              {/each}
            </div>

            <div class="picker-footer-row">
              <button
                type="button"
                class="picker-today-btn"
                on:click={jumpToLatestMonth}
              >
                <Sparkles size={12} class="text-amber-400" />
                <span>Latest Month</span>
              </button>
            </div>
          </div>
        {/if}
      </div>

      <button
        type="button"
        class="month-nav-btn next-btn"
        disabled={!hasNextMonth}
        on:click={nextMonth}
        title="Next Month (Right arrow)"
        aria-label="Next month"
      >
        <ChevronRight size={16} />
      </button>
    </div>

    <!-- Right Side: Filter Quick Control Panel or Key Hint -->
    {#if isFiltering}
      <div class="calendar-filter-control-panel">
        <div class="filter-panel-meta">
          <span class="filter-sparkle">✦</span>
          <span class="filter-stats-text">
            <strong>{allMatchingDates.length}</strong> matching {allMatchingDates.length === 1 ? 'day' : 'days'}
          </span>
        </div>

        <div class="filter-nav-btn-group">
          <button
            type="button"
            class="filter-step-btn"
            disabled={allMatchingDates.length <= 1}
            on:click={jumpToPrevMatchingPost}
            title="Jump to Previous Matching BeReal"
            aria-label="Previous matching post"
          >
            <ChevronLeft size={12} />
            <span>Prev</span>
          </button>

          <button
            type="button"
            class="filter-step-btn next-step-btn"
            disabled={allMatchingDates.length <= 1}
            on:click={jumpToNextMatchingPost}
            title="Jump to Next Matching BeReal"
            aria-label="Next matching post"
          >
            <span>Next</span>
            <ChevronRight size={12} />
          </button>
        </div>

        <button
          type="button"
          class="filter-clear-pill-btn"
          on:click={resetFilters}
          title="Reset all filters"
        >
          Reset
        </button>
      </div>
    {:else}
      <!-- Arrow Keys Navigation Hint Badge -->
      <div class="scroll-hint-tag" title="Use Left / Right arrow keys to change months">
        <span>← / → to flip months</span>
      </div>
    {/if}
  </div>

  <!-- Weekday Headers -->
  <div class="weekdays-grid">
    {#each weekdays as day}
      <div class="weekday-header-cell">
        <span>{day}</span>
      </div>
    {/each}
  </div>

  <!-- Calendar Days Grid (Smooth transitions on filter toggles) -->
  {#key yearMonth}
    <div class="days-matrix-grid month-fade-in">
      <!-- Empty offset cells before 1st of month -->
      {#each Array(startDayOffset) as _, i}
        <div class="day-cell empty-offset-cell"></div>
      {/each}

      <!-- Actual month days -->
      {#each Array(daysInMonth) as _, i}
        {@const dayNum = i + 1}
        {@const dateStr = `${currentYear}-${String(currentMonthNum).padStart(2, '0')}-${String(dayNum).padStart(2, '0')}`}
        {@const rawPosts = $rawMemoriesByDate.get(dateStr) || []}
        {@const dayPosts = $memoriesByDate.get(dateStr) || []}
        {@const hasRawPost = rawPosts.length > 0}
        {@const hasMatchingPost = dayPosts.length > 0}
        {@const isFilteredOut = isFiltering && hasRawPost && !hasMatchingPost}
        {@const isFilterMatch = isFiltering && hasMatchingPost}
        {@const isFocusedMatch = focusedMatchingDate === dateStr}
        {@const primaryPost = hasMatchingPost ? dayPosts[0] : (hasRawPost ? rawPosts[0] : null)}
        {@const isOnTime = hasRawPost && primaryPost ? !primaryPost.isLate : false}
        {@const isLate = hasRawPost && primaryPost ? primaryPost.isLate : false}
        {@const lateTooltip = primaryPost ? (primaryPost.lateExact ? `${primaryPost.lateExact} (${primaryPost.timeFormatted})` : (primaryPost.lateDuration ? `${primaryPost.lateDuration} (${primaryPost.timeFormatted})` : 'Posted late')) : ''}

        <div
          class="day-cell"
          class:has-memory={hasRawPost}
          class:is-on-time={isOnTime}
          class:is-filtered-out={isFilteredOut}
          class:is-filter-match={isFilterMatch}
          class:is-focused-match={isFocusedMatch}
          role="button"
          tabindex={hasRawPost ? 0 : -1}
          on:click={() => hasRawPost && handleDayClick(dayNum)}
          on:contextmenu={(e) => hasRawPost && primaryPost && openContextMenu(e, primaryPost)}
          on:keydown={(e) => hasRawPost && (e.key === 'Enter' || e.key === ' ') && handleDayClick(dayNum)}
          title={isFilteredOut ? `Excluded by filters (${rawPosts.length} BeReal)` : (isFilterMatch ? `Matches filter (${dayPosts.length} BeReal)` : (isLate ? lateTooltip : (hasRawPost ? `On Time • ${primaryPost?.timeFormatted}` : '')))}
        >
          {#if hasRawPost && primaryPost}
            <div class="active-day-card">
              <DualCameraFrame
                primarySrc={primaryPost.primaryPath}
                secondarySrc={primaryPost.secondaryPath}
                btsSrc={primaryPost.btsPath}
                isVideo={primaryPost.isVideo}
                alt="BeReal {primaryPost.dateFormatted}"
                dayNumberOverlay={String(dayNum)}
                size="sm"
                interactive={false}
                isLate={primaryPost.isLate}
                lateDuration={primaryPost.lateDuration}
                lateExact={primaryPost.lateExact}
                takenAt={primaryPost.takenAt}
                rawJson={primaryPost.rawJson}
                debugInfo={primaryPost.debugInfo}
              />

              <!-- Late indicator badge if isLate is true -->
              {#if isLate && ($memoryHeaderSettings.showLatePillsInCalendar ?? true)}
                <div class="day-late-badge" title={lateTooltip}>
                  <span>{primaryPost.lateDuration || 'Late'}</span>
                </div>
              {/if}

              <!-- Top-right corner BeReal count badge -->
              {#if isFilterMatch && dayPosts.length > 1}
                <div class="day-count-badge is-matching-badge" title="{dayPosts.length} matching BeReals">
                  {dayPosts.length}
                </div>
              {:else if !isFiltering && rawPosts.length > 1}
                <div class="day-count-badge" title="{rawPosts.length} BeReals on this day">
                  {rawPosts.length}
                </div>
              {/if}

              <!-- Filtered Out Ghost Tag -->
              {#if isFilteredOut}
                <div class="filtered-out-indicator">
                  <span>Filtered</span>
                </div>
              {/if}
            </div>
          {:else}
            <div class="empty-day-box">
              <span class="empty-day-number">{dayNum}</span>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/key}

  <!-- Explore Memories Outline Box Button -->
  <div class="calendar-footer-actions">
    <button
      type="button"
      class="explore-memories-outline-btn"
      on:click={() => activeExplorerView.set('grid')}
      title="Switch to full memories photo grid"
    >
      <Grid size={15} />
      <span>Explore All Memories</span>
    </button>
  </div>
</div>

<style>
  .calendar-view-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
    width: 100%;
    background: #09090e;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: 18px;
  }

  .calendar-header-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border-subtle);
    position: relative;
  }

  .calendar-nav-cluster {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .scroll-hint-tag {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-muted);
    background: #14141d;
    padding: 4px 10px;
    border-radius: var(--radius-full);
    border: 1px solid var(--border-subtle);
    user-select: none;
  }

  .month-title-group {
    position: relative;
  }

  .month-picker-trigger-btn {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 6px 14px;
    background: #14141d;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .month-picker-trigger-btn:hover,
  .month-picker-trigger-btn.is-active {
    background: #1a1a26;
    border-color: rgba(56, 189, 248, 0.5);
    box-shadow: 0 4px 16px rgba(56, 189, 248, 0.15);
  }

  .month-title-text {
    font-size: 16px;
    font-weight: 800;
    color: #ffffff;
    letter-spacing: -0.01em;
  }

  :global(.picker-chevron) {
    color: var(--text-muted);
    transition: transform 0.2s ease;
  }

  :global(.rotate-180) {
    transform: rotate(180deg);
  }

  /* Popover Dialog Matrix */
  .month-picker-popover {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    width: 260px;
    background: #101017;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.85);
    padding: 14px;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 12px;
    animation: popoverScaleIn 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes popoverScaleIn {
    from {
      opacity: 0;
      transform: translateY(-6px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .picker-year-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .picker-year-title {
    font-size: 14px;
    font-weight: 800;
    color: #ffffff;
  }

  .picker-nav-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: var(--radius-sm);
    background: #181824;
    border: 1px solid var(--border-subtle);
    color: var(--text-main);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .picker-nav-btn:hover:not(:disabled) {
    background: #242436;
    border-color: var(--border-medium);
  }

  .picker-nav-btn:disabled {
    opacity: 0.25;
    cursor: not-allowed;
    background: transparent;
    border-color: transparent;
  }

  .picker-months-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }

  .picker-month-btn {
    position: relative;
    padding: 8px 4px;
    border-radius: var(--radius-sm);
    background: #151520;
    border: 1px solid transparent;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
  }

  .picker-month-btn:hover:not(:disabled) {
    background: #1f1f2e;
    color: #ffffff;
  }

  .picker-month-btn:disabled {
    opacity: 0.22;
    cursor: not-allowed;
    background: transparent;
    border-color: transparent;
    color: var(--text-muted);
  }

  .picker-month-btn.has-memories {
    color: #ffffff;
    font-weight: 700;
  }

  .picker-month-btn.is-selected {
    background: #38bdf8;
    color: #09090b;
    font-weight: 800;
    box-shadow: 0 2px 10px rgba(56, 189, 248, 0.4);
  }

  .month-data-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: #38bdf8;
  }

  .picker-month-btn.is-selected .month-data-dot {
    background: #09090b;
  }

  .picker-footer-row {
    display: flex;
    justify-content: center;
    padding-top: 4px;
    border-top: 1px solid var(--border-subtle);
  }

  .picker-today-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    border-radius: var(--radius-full);
    transition: all 0.15s ease;
  }

  .picker-today-btn:hover {
    color: #ffffff;
    background: #181824;
  }

  .month-nav-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: #181824;
    border: 1px solid var(--border-subtle);
    color: #ffffff;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .month-nav-btn:hover:not(:disabled) {
    background: #242436;
    border-color: var(--border-medium);
  }

  .month-nav-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .weekdays-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 8px;
    text-align: center;
  }

  .weekday-header-cell {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-muted);
    letter-spacing: 0.06em;
    padding: 4px 0;
  }

  .month-count-tag {
    font-size: 9.5px;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: var(--radius-full);
    background: rgba(255, 255, 255, 0.12);
    color: var(--text-secondary);
  }

  .picker-month-btn.is-selected .month-count-tag {
    background: rgba(0, 0, 0, 0.2);
    color: #000000;
  }

  .month-fade-in {
    animation: monthFadeIn 0.22s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  @keyframes monthFadeIn {
    from {
      opacity: 0;
      background: #000000;
    }
    to {
      opacity: 1;
      background: transparent;
    }
  }

  .days-matrix-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 8px;
    border-radius: 14px;
  }

  .day-cell {
    position: relative;
    width: 100%;
    aspect-ratio: 3 / 4;
    border-radius: 12px;
    overflow: hidden;
  }

  .day-cell.has-memory {
    cursor: pointer;
    border: none;
    outline: none;
    box-sizing: border-box;
    transition: transform 0.25s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.25s ease, border-color 0.25s ease, opacity 0.25s ease, filter 0.25s ease;
  }

  /* Crisp white outline for On-Time BeReals ONLY */
  .day-cell.has-memory.is-on-time {
    border: 2px solid #ffffff;
    box-shadow: 0 0 10px rgba(255, 255, 255, 0.35);
  }

  /* Late BeReals have NO border outline */
  .day-cell.has-memory:not(.is-on-time):not(.is-filter-match) {
    border: none;
    box-shadow: none;
  }

  /* Filtered-out ghost state: soft dimming preserving calendar shape & context */
  .day-cell.has-memory.is-filtered-out {
    opacity: 0.28;
    filter: grayscale(85%) contrast(85%);
    transform: scale(0.96);
    border-color: rgba(255, 255, 255, 0.04);
    box-shadow: none;
  }

  .day-cell.has-memory.is-filtered-out:hover {
    opacity: 0.75;
    filter: grayscale(25%);
    transform: scale(1.02);
    z-index: 6;
  }

  /* Filter-matching highlighted state */
  .day-cell.has-memory.is-filter-match {
    border: 2px solid #38bdf8;
    box-shadow: 0 0 16px rgba(56, 189, 248, 0.4);
    transform: scale(1.01);
  }

  .day-cell.has-memory.is-filter-match:hover {
    transform: scale(1.06);
    box-shadow: 0 8px 24px rgba(56, 189, 248, 0.55);
    z-index: 6;
  }

  .day-cell.has-memory:hover {
    transform: scale(1.05);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.8);
    z-index: 5;
  }

  .day-cell.has-memory:active {
    transform: scale(0.96);
    transition: transform 0.08s ease;
  }

  .active-day-card {
    position: relative;
    width: 100%;
    height: 100%;
  }

  /* Top Right White Pill Badge with Black Text */
  .day-count-badge {
    position: absolute;
    top: 6px;
    right: 6px;
    min-width: 19px;
    height: 19px;
    padding: 0 5px;
    border-radius: var(--radius-full);
    background: #ffffff;
    color: #000000;
    font-size: 11px;
    font-weight: 800;
    letter-spacing: -0.02em;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.7);
    z-index: 10;
  }

  .day-count-badge.is-matching-badge {
    background: #38bdf8;
    color: #000000;
    box-shadow: 0 2px 8px rgba(56, 189, 248, 0.6);
  }

  /* Bottom-right late badge on day cell */
  .day-late-badge {
    position: absolute;
    bottom: 5px;
    right: 5px;
    padding: 1px 5px;
    border-radius: var(--radius-full);
    background: rgba(0, 0, 0, 0.82);
    backdrop-filter: blur(6px);
    border: 1px solid rgba(255, 255, 255, 0.25);
    color: #f4f4f5;
    font-size: 8.5px;
    font-weight: 700;
    letter-spacing: -0.01em;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.6);
    z-index: 10;
    transition: transform 0.15s ease, background 0.15s ease, border-color 0.15s ease;
  }

  .day-late-badge:hover {
    background: rgba(0, 0, 0, 0.95);
    border-color: #38bdf8;
    transform: scale(1.08);
  }

  .filtered-out-indicator {
    position: absolute;
    bottom: 6px;
    left: 50%;
    transform: translateX(-50%);
    padding: 1px 6px;
    border-radius: var(--radius-full);
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(4px);
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: #a1a1aa;
    font-size: 8.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    pointer-events: none;
    z-index: 10;
  }

  /* Focused Match Pulse State when stepping with Prev/Next */
  .day-cell.has-memory.is-focused-match {
    border: 3px solid #ffe600;
    box-shadow: 0 0 24px rgba(255, 230, 0, 0.7);
    transform: scale(1.08);
    z-index: 20;
    animation: matchPulse 0.4s ease-out;
  }

  @keyframes matchPulse {
    0% { transform: scale(1); box-shadow: 0 0 0 rgba(255, 230, 0, 0); }
    50% { transform: scale(1.12); box-shadow: 0 0 30px rgba(255, 230, 0, 0.9); }
    100% { transform: scale(1.08); box-shadow: 0 0 24px rgba(255, 230, 0, 0.7); }
  }

  /* Calendar Filter Quick Jump Control Panel in header */
  .calendar-filter-control-panel {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px 4px 12px;
    background: rgba(18, 18, 26, 0.95);
    border: 1px solid rgba(56, 189, 248, 0.4);
    border-radius: var(--radius-full);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.6);
    animation: fadeIn 0.18s ease-out;
  }

  .filter-panel-meta {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .filter-stats-text {
    font-size: 11.5px;
    color: #38bdf8;
    white-space: nowrap;
  }

  .filter-stats-text strong {
    color: #ffffff;
  }

  .filter-nav-btn-group {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .filter-step-btn {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    background: #181826;
    border: 1px solid var(--border-subtle);
    color: #ffffff;
    font-size: 11px;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .filter-step-btn:hover:not(:disabled) {
    background: #38bdf8;
    color: #09090b;
    border-color: #38bdf8;
    box-shadow: 0 2px 8px rgba(56, 189, 248, 0.35);
  }

  .filter-step-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .filter-clear-pill-btn {
    padding: 3px 8px;
    border-radius: var(--radius-full);
    background: rgba(244, 63, 94, 0.16);
    border: 1px solid rgba(244, 63, 94, 0.4);
    color: #fda4af;
    font-size: 10.5px;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .filter-clear-pill-btn:hover {
    background: #f43f5e;
    color: #ffffff;
    border-color: #f43f5e;
  }

  .empty-day-box {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #111118;
    border: 1px solid rgba(255, 255, 255, 0.03);
    border-radius: 12px;
  }

  .empty-day-number {
    font-size: 14px;
    font-weight: 600;
    color: #4b4b5a;
  }

  .empty-offset-cell {
    background: transparent;
    border: none;
  }

  /* Explore Memories Outline Button */
  .calendar-footer-actions {
    display: flex;
    justify-content: center;
    padding-top: 14px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    margin-top: 6px;
  }

  .explore-memories-outline-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 10px 24px;
    background: rgba(56, 189, 248, 0.08);
    border: 1.5px solid rgba(56, 189, 248, 0.4);
    border-radius: var(--radius-full);
    color: #38bdf8;
    font-size: 13px;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
  }

  .explore-memories-outline-btn:hover {
    background: #38bdf8;
    color: #09090b;
    border-color: #38bdf8;
    transform: translateY(-1px);
    box-shadow: 0 4px 16px rgba(56, 189, 248, 0.35);
  }

  @media (max-width: 600px) {
    .days-matrix-grid,
    .weekdays-grid {
      gap: 4px;
    }
  }
</style>
