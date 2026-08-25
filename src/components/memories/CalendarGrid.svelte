<script lang="ts">
  import {
    explorerData,
    calendarCurrentMonth,
    memoriesByDate,
    openFeedAt,
    activeExplorerView,
  } from '$lib/memoriesStore';
  import DualCameraFrame from './DualCameraFrame.svelte';
  import ChevronLeft from 'lucide-svelte/icons/chevron-left';
  import ChevronRight from 'lucide-svelte/icons/chevron-right';
  import CalendarIcon from 'lucide-svelte/icons/calendar';
  import Grid from 'lucide-svelte/icons/layout-grid';

  const weekdays = ['MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT', 'SUN'];

  // Parse active month key "YYYY-MM"
  $: yearMonth = $calendarCurrentMonth || ($explorerData?.uniqueMonths[0] ?? '2024-08');
  $: currentYear = parseInt(yearMonth.slice(0, 4), 10);
  $: currentMonthNum = parseInt(yearMonth.slice(5, 7), 10); // 1-12

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

  // Available months for navigation
  $: availableMonths = $explorerData?.uniqueMonths ?? [];
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

  function handleDayClick(dayNum: number) {
    const dateStr = `${currentYear}-${String(currentMonthNum).padStart(2, '0')}-${String(dayNum).padStart(2, '0')}`;
    const posts = $memoriesByDate.get(dateStr);
    if (posts && posts.length > 0) {
      openFeedAt(posts[0]);
    }
  }
</script>

<div class="calendar-view-container">
  <!-- Month Switcher Header -->
  <div class="calendar-header-bar">
    <div class="month-title-group">
      <h2 class="month-title-text">{monthTitle}</h2>
    </div>

    <div class="month-nav-actions">
      <button
        type="button"
        class="month-nav-btn"
        disabled={!hasPrevMonth}
        on:click={prevMonth}
        title="Previous Month"
        aria-label="Previous month"
      >
        <ChevronLeft size={16} />
      </button>

      {#if availableMonths.length > 1}
        <select
          class="month-select-dropdown"
          bind:value={$calendarCurrentMonth}
        >
          {#each availableMonths as mo}
            <option value={mo}>{mo}</option>
          {/each}
        </select>
      {/if}

      <button
        type="button"
        class="month-nav-btn"
        disabled={!hasNextMonth}
        on:click={nextMonth}
        title="Next Month"
        aria-label="Next month"
      >
        <ChevronRight size={16} />
      </button>
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

      <div
        class="day-cell"
        class:has-memory={hasPost}
        role="button"
        tabindex={hasPost ? 0 : -1}
        on:click={() => hasPost && handleDayClick(dayNum)}
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
              badgeText={dayPosts.length > 1 ? `(${dayPosts.length})` : primaryPost.retakeCounter > 0 ? `(${primaryPost.retakeCounter})` : ''}
              size="sm"
              interactive={false}
            />
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
  }

  .month-title-text {
    font-size: 18px;
    font-weight: 800;
    color: #ffffff;
    letter-spacing: -0.01em;
  }

  .month-nav-actions {
    display: flex;
    align-items: center;
    gap: 8px;
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

  .month-select-dropdown {
    height: 32px;
    padding: 0 10px;
    background: #181824;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: #ffffff;
    font-size: 12px;
    font-weight: 600;
    outline: none;
    cursor: pointer;
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
    transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.2s ease;
  }

  .day-cell.has-memory:hover {
    transform: scale(1.05);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.8);
    z-index: 5;
  }

  .active-day-card {
    width: 100%;
    height: 100%;
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
