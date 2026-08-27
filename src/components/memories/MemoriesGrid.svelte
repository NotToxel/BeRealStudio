<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { filteredMemories, openFeedAt, openContextMenu, memoryHeaderSettings, isFirstBeRealOfDay } from '$lib/memoriesStore';
  import type { ExplorerMemory } from '$lib/types';
  import DualCameraFrame from './DualCameraFrame.svelte';
  import Images from 'lucide-svelte/icons/images';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import Calendar from 'lucide-svelte/icons/calendar';
  import MessageSquare from 'lucide-svelte/icons/message-square';
  import Clock from 'lucide-svelte/icons/clock';
  import Sparkles from 'lucide-svelte/icons/sparkles';

  let scrollContainer: HTMLElement | null = null;
  let scrubberTrackEl: HTMLElement | null = null;
  let activeScrubMonth = '';
  let activeScrubKey = '';
  let isScrolling = false;
  let isDraggingScrubber = false;
  let scrollHideTimer: any = null;
  let scrollThumbTopPercent = 0;
  let monthPositions: { key: string; title: string; topPercent: number; offsetTop: number }[] = [];

  interface MonthGroup {
    key: string;
    title: string;
    memories: ExplorerMemory[];
  }

  $: monthGroups = (() => {
    const groups: MonthGroup[] = [];
    const map = new Map<string, ExplorerMemory[]>();

    for (const mem of $filteredMemories) {
      const ym = mem.monthKey || 'Unknown';
      if (!map.has(ym)) {
        map.set(ym, []);
      }
      map.get(ym)!.push(mem);
    }

    for (const [ym, items] of map.entries()) {
      let title = ym;
      if (ym !== 'Unknown') {
        try {
          const [y, m] = ym.split('-').map(Number);
          const d = new Date(y, m - 1, 1);
          title = d.toLocaleDateString('en-US', { month: 'long', year: 'numeric' });
        } catch {
          title = ym;
        }
      }
      groups.push({ key: ym, title, memories: items });
    }
    return groups;
  })();

  let calcPositionsTimer: ReturnType<typeof setTimeout> | null = null;

  function scheduleCalculateMonthPositions() {
    if (calcPositionsTimer) clearTimeout(calcPositionsTimer);
    calcPositionsTimer = setTimeout(() => {
      if (typeof requestAnimationFrame !== 'undefined') {
        requestAnimationFrame(() => {
          calculateMonthPositions();
        });
      } else {
        calculateMonthPositions();
      }
    }, 60);
  }

  $: if (monthGroups.length > 0) {
    scheduleCalculateMonthPositions();
  }

  function handleMemoryClick(memory: ExplorerMemory) {
    openFeedAt(memory);
  }

  function calculateMonthPositions() {
    if (!scrollContainer) return;
    const { scrollHeight, clientHeight } = scrollContainer;
    const maxScroll = Math.max(1, scrollHeight - clientHeight);

    const positions: { key: string; title: string; topPercent: number; offsetTop: number }[] = [];

    for (let i = 0; i < monthGroups.length; i++) {
      const group = monthGroups[i];
      const el = document.getElementById(`month-group-${group.key}`);
      const topPercent = monthGroups.length > 1 ? (i / (monthGroups.length - 1)) * 92 + 4 : 50;
      const sectionTop = el ? el.offsetTop : (i / Math.max(1, monthGroups.length - 1)) * maxScroll;

      positions.push({
        key: group.key,
        title: group.title,
        topPercent,
        offsetTop: sectionTop,
      });
    }

    monthPositions = positions;
    updateActiveMonth();
  }

  let scrollRafPending = false;

  function updateActiveMonth() {
    if (!scrollContainer || monthGroups.length === 0) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    const maxScroll = Math.max(1, scrollHeight - clientHeight);

    // If at or near bottom (within 24px of maxScroll), lock to the last month
    if (scrollTop >= maxScroll - 24) {
      const last = monthGroups[monthGroups.length - 1];
      activeScrubMonth = last.title;
      activeScrubKey = last.key;
      scrollThumbTopPercent = 96;
      return;
    }

    // If at or near top (within 24px of top), lock to the first month
    if (scrollTop <= 24) {
      const first = monthGroups[0];
      activeScrubMonth = first.title;
      activeScrubKey = first.key;
      scrollThumbTopPercent = 4;
      return;
    }

    let activeIdx = 0;
    let nextFrac = 0;
    const vRect = scrollContainer.getBoundingClientRect();

    for (let i = 0; i < monthGroups.length; i++) {
      const el = document.getElementById(`month-group-${monthGroups[i].key}`);
      if (!el) continue;
      const elRect = el.getBoundingClientRect();
      const realTop = elRect.top - vRect.top;
      if (realTop <= 50) {
        activeIdx = i;
      } else {
        break;
      }
    }

    if (activeIdx < monthGroups.length - 1) {
      const curEl = document.getElementById(`month-group-${monthGroups[activeIdx].key}`);
      const nextEl = document.getElementById(`month-group-${monthGroups[activeIdx + 1].key}`);
      if (curEl && nextEl) {
        const curTop = curEl.getBoundingClientRect().top - vRect.top;
        const nextTop = nextEl.getBoundingClientRect().top - vRect.top;
        const span = Math.max(1, nextTop - curTop);
        nextFrac = Math.min(1, Math.max(0, -curTop / span));
      }
    }

    const currentGroup = monthGroups[activeIdx];
    if (currentGroup) {
      activeScrubMonth = currentGroup.title;
      activeScrubKey = currentGroup.key;
    }

    if (monthGroups.length > 1) {
      const stepPct = 92 / (monthGroups.length - 1);
      scrollThumbTopPercent = 4 + (activeIdx + nextFrac) * stepPct;
    } else {
      scrollThumbTopPercent = 50;
    }
  }

  function handleScroll() {
    if (isDraggingScrubber) return;
    if (!scrollRafPending) {
      scrollRafPending = true;
      requestAnimationFrame(() => {
        updateActiveMonth();
        scrollRafPending = false;
      });
    }
    isScrolling = true;
    clearTimeout(scrollHideTimer);
    scrollHideTimer = setTimeout(() => {
      if (!isDraggingScrubber) {
        isScrolling = false;
      }
    }, 800);
  }

  function scrollToMonth(key: string, title: string) {
    activeScrubMonth = title;
    activeScrubKey = key;
    isScrolling = true;

    clearTimeout(scrollHideTimer);
    scrollHideTimer = setTimeout(() => {
      if (!isDraggingScrubber) {
        isScrolling = false;
      }
    }, 1200);

    const el = document.getElementById(`month-group-${key}`);
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }

  function jumpToNearestMonthFromPct(pct: number, smooth: boolean = false) {
    if (monthGroups.length === 0 || !scrollContainer) return;
    if (monthGroups.length === 1) {
      scrollToMonth(monthGroups[0].key, monthGroups[0].title);
      return;
    }
    const stepPct = 92 / (monthGroups.length - 1);
    const nearestIdx = Math.max(0, Math.min(monthGroups.length - 1, Math.round((pct - 4) / stepPct)));
    const targetGroup = monthGroups[nearestIdx];
    if (!targetGroup) return;

    activeScrubMonth = targetGroup.title;
    activeScrubKey = targetGroup.key;
    scrollThumbTopPercent = 4 + (nearestIdx / (monthGroups.length - 1)) * 92;

    const el = document.getElementById(`month-group-${targetGroup.key}`);
    if (el) {
      if (smooth) {
        el.scrollIntoView({ behavior: 'smooth', block: 'start' });
      } else {
        el.scrollIntoView({ behavior: 'auto', block: 'start' });
      }
    }
  }

  let dragStartY = 0;
  let hasDragged = false;
  let activeDotTarget: { key: string; title: string } | null = null;

  function handleTrackPointerDown(e: PointerEvent) {
    if (e.button !== 0 || !scrubberTrackEl) return;
    dragStartY = e.clientY;
    hasDragged = false;

    const dotBtn = (e.target as HTMLElement)?.closest('.timeline-month-dot') as HTMLElement | null;
    if (dotBtn && dotBtn.dataset.monthKey) {
      activeDotTarget = {
        key: dotBtn.dataset.monthKey,
        title: dotBtn.dataset.monthTitle || '',
      };
    } else {
      activeDotTarget = null;
    }

    isDraggingScrubber = true;
    isScrolling = true;
    try {
      scrubberTrackEl.setPointerCapture(e.pointerId);
    } catch {}

    if (!activeDotTarget) {
      handleScrubberMove(e);
    }
  }

  function handleTrackPointerMove(e: PointerEvent) {
    if (!isDraggingScrubber) return;
    if (Math.abs(e.clientY - dragStartY) > 3) {
      hasDragged = true;
      activeDotTarget = null;
    }
    if (hasDragged) {
      handleScrubberMove(e);
    }
  }

  function handleTrackPointerUp(e: PointerEvent) {
    if (!isDraggingScrubber) return;
    try {
      scrubberTrackEl?.releasePointerCapture(e.pointerId);
    } catch {}

    if (!hasDragged && activeDotTarget) {
      scrollToMonth(activeDotTarget.key, activeDotTarget.title);
    }

    isDraggingScrubber = false;
    activeDotTarget = null;
    clearTimeout(scrollHideTimer);
    scrollHideTimer = setTimeout(() => {
      isScrolling = false;
    }, 1000);
  }

  function handleTrackWheel(e: WheelEvent) {
    if (!scrollContainer) return;
    scrollContainer.scrollTop += e.deltaY;
  }

  function handleScrubberMove(e: PointerEvent | MouseEvent | TouchEvent) {
    if (!scrubberTrackEl || !scrollContainer || monthGroups.length === 0) return;
    const rect = scrubberTrackEl.getBoundingClientRect();
    const clientY = 'touches' in e ? e.touches[0].clientY : (e as MouseEvent).clientY;
    const offsetY = Math.max(0, Math.min(rect.height, clientY - rect.top));
    const ratio = offsetY / rect.height; // 0..1
    const pct = ratio * 92 + 4; // 4%..96%

    // Snap to nearest discrete month
    jumpToNearestMonthFromPct(pct, false);
  }

  onMount(() => {
    calculateMonthPositions();
    const resizeObserver = new ResizeObserver(() => {
      calculateMonthPositions();
    });
    if (scrollContainer) {
      resizeObserver.observe(scrollContainer);
    }
    return () => {
      resizeObserver.disconnect();
    };
  });

  onDestroy(() => {
    clearTimeout(scrollHideTimer);
  });
</script>

<div class="memories-grid-wrapper">
  {#if $filteredMemories.length === 0}
    <div class="empty-memories-state">
      <div class="empty-icon-wrap">
        <Images size={28} class="text-muted" />
      </div>
      <h3 class="empty-title">No Memories Match Your Filter</h3>
      <p class="empty-desc">Try clearing your search query or adjusting your filters above.</p>
    </div>
  {:else}
    <div
      bind:this={scrollContainer}
      class="memories-scroll-viewport custom-thick-scrollbar"
      on:scroll={handleScroll}
    >
      <div class="memories-groups-container">
        {#each monthGroups as group (group.key)}
          <section id="month-group-{group.key}" class="month-section">
            <!-- BeReal-Style Month Spacer Header -->
            <div class="month-section-header">
              <span class="month-header-title">{group.title}</span>
              <div class="month-header-line"></div>
              <span class="month-count-pill">{group.memories.length}</span>
            </div>

            <!-- Grid of BeReals for this month -->
            <div class="memories-grid">
              {#each group.memories as memory (memory.id)}
                {@const isFirst = isFirstBeRealOfDay(memory)}
                {@const isLateFirst = memory.isLate && isFirst}
                <div
                  class="grid-card-wrap"
                  role="button"
                  tabindex="0"
                  on:click={() => handleMemoryClick(memory)}
                  on:contextmenu={(e) => openContextMenu(e, memory)}
                  on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleMemoryClick(memory)}
                  aria-label="BeReal from {memory.dateFormatted} at {memory.timeFormatted || 'unknown time'}"
                >
                  <DualCameraFrame
                    primarySrc={memory.primaryPath}
                    secondarySrc={memory.secondaryPath}
                    btsSrc={memory.btsPath}
                    isVideo={memory.isVideo}
                    alt="BeReal {memory.dateFormatted}"
                    dayNumberOverlay={memory.dayNumber}
                    badgeText={memory.btsPath ? 'BTS' : ''}
                    size="md"
                    interactive={false}
                    allowPreviewSwap={true}
                    isLate={memory.isLate}
                    lateDuration={memory.lateDuration}
                    lateExact={memory.lateExact}
                    takenAt={memory.takenAt}
                    rawJson={memory.rawJson}
                    debugInfo={memory.debugInfo}
                  />

                  <!-- Simple Caption Floating Overlay on Photo Hover (Compact) -->
                  {#if memory.caption}
                    <div class="card-caption-hover-bubble" role="tooltip">
                      <span class="caption-bubble-text">{memory.caption}</span>
                    </div>
                  {/if}

                  <!-- Hover Overlay Subtext info -->
                  <div class="card-caption-strip">
                    <div class="card-date-wrap">
                      <span class="card-date-text">{memory.dateFormatted}</span>
                      {#if memory.caption}
                        <span class="card-caption-dot" title="Has caption" aria-label="Has caption"></span>
                      {/if}
                      
                      <!-- Rich Date & Time Tooltip Popover -->
                      <div class="date-rich-tooltip" role="tooltip">
                        <div class="tooltip-row">
                          <Clock size={11} class="text-sky-400" />
                          <span class="tooltip-time-val">{memory.timeFormatted || 'Unknown time'}</span>
                        </div>
                        <div class="tooltip-subtag" class:is-late={isLateFirst} class:is-ontime={!isLateFirst}>
                          {isLateFirst ? (memory.lateExact || memory.lateDuration || 'Late') : (!isFirst ? 'Bonus BeReal' : 'On Time')}
                        </div>
                        <div class="tooltip-tip-arrow"></div>
                      </div>
                    </div>

                    <div class="card-meta-right">
                      {#if isLateFirst && ($memoryHeaderSettings.showLatePillsInGrid ?? true)}
                        <div class="card-late-wrap">
                          <span class="card-late-pill">
                            {memory.lateDuration || 'Late'}
                          </span>
                          <!-- Rich Late Tooltip Popover -->
                          <div class="late-rich-tooltip" role="tooltip">
                            <div class="late-tooltip-title">Posted Late</div>
                            <div class="late-tooltip-desc">{memory.lateExact || memory.lateDuration || 'Posted after BeReal alert'}</div>
                            <div class="tooltip-tip-arrow"></div>
                          </div>
                        </div>
                      {/if}

                      {#if memory.locationName || memory.location}
                        <div class="card-loc-wrap">
                          <div class="card-loc-pill" aria-label={memory.locationName || 'Location'}>
                            <MapPin size={10} class="flex-shrink-0" />
                            <span class="card-loc-text">{memory.city || memory.locationName}</span>
                          </div>
                          <!-- Rich Location Tooltip Popover -->
                          <div class="loc-rich-tooltip" role="tooltip">
                            <div class="loc-tooltip-title">{memory.locationName || 'Location'}</div>
                            {#if memory.location}
                              <div class="loc-tooltip-coords font-mono">
                                {memory.location.latitude.toFixed(4)}°, {memory.location.longitude.toFixed(4)}°
                              </div>
                            {/if}
                            <div class="tooltip-tip-arrow"></div>
                          </div>
                        </div>
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </section>
        {/each}
      </div>
    </div>

    <!-- Fixed Vertical Timeline Scrubber with Accurately Aligned Month Markers -->
    {#if monthGroups.length >= 2}
      <div
        bind:this={scrubberTrackEl}
        class="timeline-scrubber-track"
        class:is-dragging={isDraggingScrubber}
        class:is-scrolling={isScrolling}
        role="slider"
        tabindex="0"
        aria-label="Timeline Month Scrubber"
        aria-valuenow={Math.round(scrollThumbTopPercent)}
        on:pointerdown={handleTrackPointerDown}
        on:pointermove={handleTrackPointerMove}
        on:pointerup={handleTrackPointerUp}
        on:pointercancel={handleTrackPointerUp}
        on:lostpointercapture={handleTrackPointerUp}
        on:wheel|passive={handleTrackWheel}
        title="Drag scrubber or click markers to jump through timeline"
      >
        <div class="timeline-line-rail"></div>

        <!-- Dynamic Drag Thumb Indicator -->
        <div
          class="timeline-scrubber-thumb"
          class:is-active={isScrolling || isDraggingScrubber}
          style="top: {Math.max(3, Math.min(97, scrollThumbTopPercent))}%;"
        >
          <div class="thumb-inner-capsule"></div>
        </div>

        <!-- Aligned Month Dot Markers along Rail (Evenly Distributed 4%..96%) -->
        {#each monthPositions as pos}
          {@const grp = monthGroups.find((g) => g.key === pos.key)}
          {@const postCount = grp?.memories.length || 0}
          <button
            type="button"
            class="timeline-month-dot"
            class:is-active={activeScrubKey === pos.key}
            style="top: {pos.topPercent}%;"
            data-month-key={pos.key}
            data-month-title={pos.title}
            aria-label="Jump to {pos.title}"
          >
            <span class="dot-inner-core"></span>

            <!-- Rich Sleek Hover Tooltip -->
            <div class="month-hover-tooltip" role="tooltip">
              <span class="month-tooltip-title">{pos.title}</span>
              {#if postCount > 0}
                <span class="month-tooltip-badge">{postCount} {postCount === 1 ? 'post' : 'posts'}</span>
              {/if}
              <div class="tooltip-arrow"></div>
            </div>
          </button>
        {/each}

        <!-- Interactive Floating Tooltip Pill (Only shown while scrolling/dragging when not hovering specific dots) -->
        {#if (isScrolling || isDraggingScrubber) && activeScrubMonth}
          <div
            class="timeline-floating-badge"
            style="top: {Math.max(5, Math.min(95, scrollThumbTopPercent))}%;"
          >
            <span class="floating-badge-text">{activeScrubMonth || 'Timeline'}</span>
            <div class="badge-pointer-arrow"></div>
          </div>
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .memories-grid-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .memories-scroll-viewport {
    flex: 1;
    min-height: 0;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    padding-right: 36px;
    padding-bottom: 24px;
    position: relative;
    scrollbar-width: none;
    -ms-overflow-style: none;
    scroll-behavior: auto;
    overscroll-behavior-y: contain;
    -webkit-overflow-scrolling: touch;
  }

  .memories-scroll-viewport::-webkit-scrollbar {
    display: none;
    width: 0;
    height: 0;
  }

  .memories-groups-container {
    display: flex;
    flex-direction: column;
    gap: 32px;
    width: 100%;
    padding-bottom: 60px;
  }

  .month-section {
    display: flex;
    flex-direction: column;
    gap: 14px;
    width: 100%;
  }

  /* BeReal Style Month Spacers / Divider */
  .month-section-header {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 6px 0 2px 0;
  }

  .month-header-title {
    font-size: 15px;
    font-weight: 800;
    color: #f4f4f5;
    letter-spacing: -0.01em;
    white-space: nowrap;
  }

  .month-header-line {
    flex: 1;
    height: 1px;
    background: linear-gradient(90deg, rgba(255, 255, 255, 0.15) 0%, rgba(255, 255, 255, 0.03) 100%);
  }

  .month-count-pill {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-muted);
    background: rgba(255, 255, 255, 0.06);
    padding: 2px 8px;
    border-radius: var(--radius-full);
  }

  .memories-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 14px;
    width: 100%;
  }

  .grid-card-wrap {
    position: relative;
    cursor: pointer;
    border-radius: 18px;
    transition: transform 0.22s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.22s ease;
    outline: none;
    content-visibility: auto;
    contain-intrinsic-size: auto 240px;
  }

  .grid-card-wrap:hover {
    transform: translateY(-4px) scale(1.02);
    box-shadow: 0 12px 28px rgba(0, 0, 0, 0.7);
    z-index: 5;
  }

  .grid-card-wrap:active {
    transform: scale(0.96);
    transition: transform 0.08s ease;
  }

  .grid-card-wrap:focus-visible {
    box-shadow: 0 0 0 3px #38bdf8;
  }

  /* Sleek & High-Performance Caption Floating Overlay on Photo Hover */
  .card-caption-hover-bubble {
    position: absolute;
    bottom: 34px;
    left: 8px;
    right: 8px;
    background: rgba(10, 10, 16, 0.94);
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-radius: 999px;
    padding: 4px 10px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.8);
    opacity: 0;
    visibility: hidden;
    transform: translateY(4px);
    pointer-events: none;
    transition: opacity 0.16s ease, transform 0.16s cubic-bezier(0.16, 1, 0.3, 1), visibility 0.16s ease;
    z-index: 15;
    text-align: center;
  }

  .grid-card-wrap:hover .card-caption-hover-bubble,
  .grid-card-wrap:focus-visible .card-caption-hover-bubble {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
  }

  .caption-bubble-text {
    font-size: 10.5px;
    font-weight: 500;
    color: #f1f5f9;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .card-caption-strip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 4px;
    padding: 5px 2px 2px 2px;
    font-size: 11px;
    color: var(--text-secondary);
    min-width: 0;
  }

  /* Rich Date & Time Tooltip Popover */
  .card-date-wrap {
    position: relative;
    display: inline-flex;
    align-items: center;
    cursor: default;
    min-width: 0;
    flex: 1 1 auto;
    overflow: hidden;
  }

  .card-caption-dot {
    display: inline-block;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: #38bdf8;
    margin-left: 4px;
    flex-shrink: 0;
    box-shadow: 0 0 3px rgba(56, 189, 248, 0.6);
    opacity: 0.85;
  }

  .date-rich-tooltip,
  .late-rich-tooltip,
  .loc-rich-tooltip {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%) translateY(4px);
    background: rgba(13, 13, 20, 0.96);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 10px;
    padding: 6px 10px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    white-space: nowrap;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.85);
    opacity: 0;
    visibility: hidden;
    pointer-events: none;
    transition: opacity 0.16s ease, transform 0.16s cubic-bezier(0.16, 1, 0.3, 1), visibility 0.16s ease;
    z-index: 50;
  }

  .card-date-wrap:hover .date-rich-tooltip,
  .card-late-wrap:hover .late-rich-tooltip,
  .card-loc-wrap:hover .loc-rich-tooltip {
    opacity: 1;
    visibility: visible;
    transform: translateX(-50%) translateY(0);
  }

  /* Align right-hand meta tooltips to the right edge to avoid screen overflow */
  .card-meta-right .loc-rich-tooltip,
  .card-meta-right .late-rich-tooltip {
    left: auto;
    right: 0;
    transform: translateY(4px);
  }

  .card-meta-right .card-late-wrap:hover .late-rich-tooltip,
  .card-meta-right .card-loc-wrap:hover .loc-rich-tooltip {
    transform: translateY(0);
  }

  .card-meta-right .tooltip-tip-arrow {
    left: auto;
    right: 12px;
    transform: rotate(45deg);
  }

  .tooltip-tip-arrow {
    position: absolute;
    bottom: -4px;
    left: 50%;
    transform: translateX(-50%) rotate(45deg);
    width: 8px;
    height: 8px;
    background: #0d0d14;
    border-bottom: 1px solid rgba(255, 255, 255, 0.15);
    border-right: 1px solid rgba(255, 255, 255, 0.15);
  }

  .tooltip-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .tooltip-time-val {
    font-size: 11.5px;
    font-weight: 700;
    color: #ffffff;
    font-family: var(--font-mono);
  }

  .tooltip-subtag {
    font-size: 10px;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: 999px;
  }

  .tooltip-subtag.is-ontime {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.3);
  }

  .tooltip-subtag.is-late {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .late-tooltip-title {
    font-size: 11px;
    font-weight: 700;
    color: #f87171;
  }

  .late-tooltip-desc {
    font-size: 10px;
    color: var(--text-secondary);
  }

  .loc-tooltip-title {
    font-size: 11px;
    font-weight: 700;
    color: #ffffff;
  }

  .loc-tooltip-coords {
    font-size: 10px;
    font-family: var(--font-mono);
    color: #38bdf8;
  }

  .card-late-wrap {
    position: relative;
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
  }

  .card-loc-wrap {
    position: relative;
    display: inline-flex;
    align-items: center;
    min-width: 0;
    max-width: 100%;
    flex-shrink: 1;
  }

  .card-date-text {
    font-weight: 600;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
    font-size: 10.5px;
  }

  .card-meta-right {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
    flex-shrink: 0;
    max-width: 55%;
  }

  .card-late-pill {
    display: inline-flex;
    align-items: center;
    padding: 1px 5px;
    background: rgba(239, 68, 68, 0.12);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #f87171;
    font-size: 9.5px;
    font-weight: 700;
    border-radius: var(--radius-full);
    white-space: nowrap;
    cursor: help;
    transition: all 0.15s ease;
  }

  .card-late-pill:hover {
    background: rgba(239, 68, 68, 0.22);
    border-color: #ef4444;
    color: #ffffff;
    transform: scale(1.05);
  }

  .card-loc-pill {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 72px;
    cursor: help;
  }

  .card-loc-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  /* Fixed Vertical Timeline Fast Scrubber Track */
  .timeline-scrubber-track {
    position: absolute;
    right: 6px;
    top: 14px;
    bottom: 24px;
    width: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: ns-resize;
    user-select: none;
    z-index: 30;
  }

  /* Suppress floating thumb badge if user is hovering over any month dot */
  .timeline-scrubber-track:has(.timeline-month-dot:hover) .timeline-floating-badge {
    display: none !important;
    opacity: 0 !important;
  }

  /* Suppress individual dot tooltips and pointer events while user is actively dragging, scrolling, or clicking */
  .timeline-scrubber-track.is-dragging .timeline-month-dot {
    pointer-events: none !important;
  }

  .timeline-scrubber-track.is-dragging .month-hover-tooltip,
  .timeline-scrubber-track.is-scrolling .month-hover-tooltip,
  .timeline-scrubber-track:active .month-hover-tooltip {
    display: none !important;
    opacity: 0 !important;
    pointer-events: none !important;
  }

  .timeline-line-rail {
    position: absolute;
    top: 8px;
    bottom: 8px;
    width: 3px;
    background: rgba(255, 255, 255, 0.16);
    border-radius: 999px;
    transition: background 0.15s ease, width 0.15s ease;
  }

  .timeline-line-rail::before,
  .timeline-line-rail::after {
    content: '';
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.35);
    border: 1px solid rgba(255, 255, 255, 0.6);
    box-shadow: 0 0 6px rgba(0, 0, 0, 0.8);
  }

  .timeline-line-rail::before {
    top: -4px;
  }

  .timeline-line-rail::after {
    bottom: -4px;
  }

  .timeline-scrubber-track:hover .timeline-line-rail {
    background: rgba(255, 255, 255, 0.32);
    width: 4px;
  }

  .timeline-scrubber-thumb {
    position: absolute;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 14px;
    height: 24px;
    background: #f4f4f5;
    border-radius: 12px;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.6), 0 0 0 2px rgba(255, 255, 255, 0.3);
    pointer-events: none;
    transition: transform 0.1s ease, background 0.15s ease;
    z-index: 35;
  }

  .timeline-scrubber-thumb.is-active,
  .timeline-scrubber-track:hover .timeline-scrubber-thumb {
    background: #38bdf8;
    box-shadow: 0 4px 16px rgba(56, 189, 248, 0.5), 0 0 0 2px #38bdf8;
    transform: translate(-50%, -50%) scale(1.15);
  }

  .timeline-month-dot {
    position: absolute;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: transparent;
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
    z-index: 32;
    transition: transform 0.15s ease;
  }

  .timeline-month-dot:hover {
    transform: translate(-50%, -50%) scale(1.35);
    z-index: 45;
  }

  /* Sleek Hover Tooltip Popover for Month Dots */
  .month-hover-tooltip {
    position: absolute;
    right: calc(100% + 12px);
    top: 50%;
    transform: translateY(-50%) translateX(6px) scale(0.95);
    background: rgba(14, 14, 22, 0.96);
    border: 1px solid rgba(56, 189, 248, 0.35);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.8), 0 0 12px rgba(56, 189, 248, 0.15);
    border-radius: var(--radius-full);
    padding: 5px 12px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    white-space: nowrap;
    opacity: 0;
    pointer-events: none;
    transition: all 0.16s cubic-bezier(0.16, 1, 0.3, 1);
    z-index: 100;
  }

  .timeline-month-dot:hover .month-hover-tooltip {
    opacity: 1;
    transform: translateY(-50%) translateX(0) scale(1);
  }

  .month-tooltip-title {
    font-size: 12px;
    font-weight: 700;
    color: #ffffff;
    letter-spacing: -0.01em;
  }

  .month-tooltip-badge {
    font-size: 10px;
    font-weight: 700;
    color: #38bdf8;
    background: rgba(56, 189, 248, 0.14);
    padding: 1px 6px;
    border-radius: var(--radius-full);
    border: 1px solid rgba(56, 189, 248, 0.25);
  }

  .tooltip-arrow {
    position: absolute;
    right: -4px;
    top: 50%;
    transform: translateY(-50%) rotate(45deg);
    width: 7px;
    height: 7px;
    background: #0e0e16;
    border-top: 1px solid rgba(56, 189, 248, 0.35);
    border-right: 1px solid rgba(56, 189, 248, 0.35);
  }

  .dot-inner-core {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.4);
    border: 1px solid rgba(0, 0, 0, 0.5);
    transition: all 0.18s ease;
  }

  .timeline-month-dot.is-active .dot-inner-core {
    width: 9px;
    height: 9px;
    background: #38bdf8;
    box-shadow: 0 0 8px #38bdf8;
    border-color: #ffffff;
  }

  .timeline-month-dot:hover .dot-inner-core {
    background: #ffffff;
    transform: scale(1.2);
  }

  /* Floating Scrub/Scroll Date Popup Pill (Shows directly adjacent to thumb) */
  .timeline-floating-badge {
    position: absolute;
    right: 28px;
    transform: translateY(-50%);
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: rgba(18, 18, 28, 0.95);
    backdrop-filter: blur(16px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 12px;
    padding: 5px 12px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.7);
    pointer-events: none;
    white-space: nowrap;
    z-index: 90;
    animation: popupIn 0.15s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes popupIn {
    from {
      opacity: 0;
      transform: translateY(-50%) translateX(6px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(-50%) translateX(0) scale(1);
    }
  }

  .floating-badge-text {
    font-size: 12px;
    font-weight: 700;
    color: #ffffff;
    letter-spacing: -0.01em;
  }

  .badge-pointer-arrow {
    position: absolute;
    right: -5px;
    top: 50%;
    transform: translateY(-50%) rotate(45deg);
    width: 8px;
    height: 8px;
    background: rgba(18, 18, 28, 0.95);
    border-top: 1px solid rgba(255, 255, 255, 0.2);
    border-right: 1px solid rgba(255, 255, 255, 0.2);
  }

  .empty-memories-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 70px 20px;
    text-align: center;
    background: #0d0d13;
    border: 1px dashed var(--border-medium);
    border-radius: var(--radius-xl);
  }

  .empty-icon-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 60px;
    height: 60px;
    border-radius: 50%;
    background: #15151f;
    margin-bottom: 16px;
  }

  .empty-title {
    font-size: 16px;
    font-weight: 700;
    color: var(--text-main);
    margin-bottom: 6px;
  }

  .empty-desc {
    font-size: 13px;
    color: var(--text-secondary);
    max-width: 360px;
  }
</style>
