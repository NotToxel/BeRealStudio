<script lang="ts">
  import {
    explorerData,
    calendarCurrentMonth,
    memoriesByDate,
    openFeedAt,
    openContextMenu,
    activeExplorerView,
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
  $: yearMonth = $calendarCurrentMonth || ($explorerData?.uniqueMonths[0] ?? '2024-08');
  $: currentYear = parseInt(yearMonth.slice(0, 4), 10);
  $: currentMonthNum = parseInt(yearMonth.slice(5, 7), 10); // 1-12

  // Selected year inside picker popover
  let pickerYear = currentYear;
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
  $: currentMonthIdx = availableMonths.indexOf(yearMonth);
  $: hasPrevMonth = currentMonthIdx > 0;
  $: hasNextMonth = currentMonthIdx < availableMonths.length - 1;

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
    const posts = $memoriesByDate.get(dateStr);
    if (posts && posts.length > 0) {
      openFeedAt(posts[0]);
    }
  }

  let scrollDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  function handleWheel(e: WheelEvent) {
    if (isPickerOpen) return;
    if (Math.abs(e.deltaY) > 20) {
      if (scrollDebounceTimer) return;
      if (e.deltaY > 0) {
        if (hasNextMonth) nextMonth();
      } else {
        if (hasPrevMonth) prevMonth();
      }
      scrollDebounceTimer = setTimeout(() => {
        scrollDebounceTimer = null;
      }, 240);
    }
  }
</script>

<div class="calendar-view-container" on:wheel={handleWheel}>
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
                on:click={() => pickerYear--}
                title="Previous Year"
              >
                <ChevronLeft size={14} />
              </button>
              <span class="picker-year-title">{pickerYear}</span>
              <button
                type="button"
                class="picker-nav-btn"
                on:click={() => pickerYear++}
                title="Next Year"
              >
                <ChevronRight size={14} />
              </button>
            </div>

            <div class="picker-months-grid">
              {#each monthNames as mName, mIdx}
                {@const mKey = `${pickerYear}-${String(mIdx + 1).padStart(2, '0')}`}
                {@const isCurrent = mKey === yearMonth}
                {@const hasData = availableMonths.includes(mKey)}

                <button
                  type="button"
                  class="picker-month-btn"
                  class:is-selected={isCurrent}
                  class:has-memories={hasData}
                  on:click={() => selectMonthFromPicker(mIdx)}
                >
                  <span>{mName}</span>
                  {#if hasData}
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
        title="Next Month (or scroll down)"
        aria-label="Next month"
      >
        <ChevronRight size={16} />
      </button>
    </div>

    <!-- Scroll Hint Badge -->
    <div class="scroll-hint-tag" title="Use mouse wheel or trackpad to flip between months">
      <span>Scroll to flip months</span>
    </div>
  </div>

  <!-- Weekday Headers -->
  <div class="weekdays-grid">
    {#each weekdays as day}
      <div class="weekday-header-cell">
        <span>{day}</span>
      </div>
    {/each}
  </div>

  <!-- Calendar Days Grid -->
  <div class="days-matrix-grid">
    <!-- Empty offset cells before 1st of month -->
    {#each Array(startDayOffset) as _, i}
      <div class="day-cell empty-offset-cell"></div>
    {/each}

    <!-- Actual month days -->
    {#each Array(daysInMonth) as _, i}
      {@const dayNum = i + 1}
      {@const dateStr = `${currentYear}-${String(currentMonthNum).padStart(2, '0')}-${String(dayNum).padStart(2, '0')}`}
      {@const dayPosts = $memoriesByDate.get(dateStr) || []}
      {@const hasPost = dayPosts.length > 0}
      {@const primaryPost = hasPost ? dayPosts[0] : null}
      {@const isOnTime = hasPost && primaryPost ? !primaryPost.isLate : false}

      <div
        class="day-cell"
        class:has-memory={hasPost}
        class:is-on-time={isOnTime}
        role="button"
        tabindex={hasPost ? 0 : -1}
        on:click={() => hasPost && handleDayClick(dayNum)}
        on:contextmenu={(e) => hasPost && primaryPost && openContextMenu(e, primaryPost)}
        on:keydown={(e) => hasPost && (e.key === 'Enter' || e.key === ' ') && handleDayClick(dayNum)}
      >
        {#if hasPost && primaryPost}
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
            />

            <!-- Top-right corner BeReal count badge (white background, black text) -->
            {#if dayPosts.length > 1}
              <div class="day-count-badge" title="{dayPosts.length} BeReals on this day">
                {dayPosts.length}
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

  .picker-nav-btn:hover {
    background: #242436;
    border-color: var(--border-medium);
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

  .picker-month-btn:hover {
    background: #1f1f2e;
    color: #ffffff;
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

  .days-matrix-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 8px;
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
    border: 1.5px solid rgba(255, 255, 255, 0.08);
    transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.2s ease, border-color 0.2s ease;
  }

  /* Medium-thick white outline for On-Time BeReals */
  .day-cell.has-memory.is-on-time {
    border: 2.5px solid #ffffff;
    box-shadow: 0 0 12px rgba(255, 255, 255, 0.25);
  }

  .day-cell.has-memory:hover {
    transform: scale(1.05);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.8);
    z-index: 5;
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
