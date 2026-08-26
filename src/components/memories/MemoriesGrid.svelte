<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { filteredMemories, openFeedAt, openContextMenu, memoryHeaderSettings } from '$lib/memoriesStore';
  import type { ExplorerMemory } from '$lib/types';
  import DualCameraFrame from './DualCameraFrame.svelte';
  import Images from 'lucide-svelte/icons/images';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import Calendar from 'lucide-svelte/icons/calendar';

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
    const containerTop = scrollContainer.getBoundingClientRect().top + scrollContainer.scrollTop;

    const positions: { key: string; title: string; topPercent: number; offsetTop: number }[] = [];

    for (let i = 0; i < monthGroups.length; i++) {
      const group = monthGroups[i];
      const el = document.getElementById(`month-group-${group.key}`);
      if (el) {
        const elRect = el.getBoundingClientRect();
        const sectionTop = elRect.top + scrollContainer.scrollTop - containerTop;
        const topPercent = Math.min(96, Math.max(4, (sectionTop / maxScroll) * 92 + 4));
        positions.push({
          key: group.key,
          title: group.title,
          topPercent,
          offsetTop: sectionTop,
        });
      } else {
        // Fallback linear distribution if element not yet queried
        const topPercent = monthGroups.length > 1 ? (i / (monthGroups.length - 1)) * 92 + 4 : 50;
        positions.push({
          key: group.key,
          title: group.title,
          topPercent,
          offsetTop: (topPercent / 100) * maxScroll,
        });
      }
    }

    monthPositions = positions;
    updateActiveMonth();
  }

  function updateActiveMonth() {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    const maxScroll = Math.max(1, scrollHeight - clientHeight);
    scrollThumbTopPercent = Math.min(100, Math.max(0, (scrollTop / maxScroll) * 100));

    // Find the month currently visible in viewport with 80px offset
    const currentScrollPos = scrollTop + 80;
    let found = false;

    for (let i = monthPositions.length - 1; i >= 0; i--) {
      const pos = monthPositions[i];
      if (currentScrollPos >= pos.offsetTop) {
        activeScrubMonth = pos.title;
        activeScrubKey = pos.key;
        found = true;
        break;
      }
    }

    if (!found && monthPositions.length > 0) {
      activeScrubMonth = monthPositions[0].title;
      activeScrubKey = monthPositions[0].key;
    }
  }

  function handleScroll() {
    updateActiveMonth();
    isScrolling = true;
    clearTimeout(scrollHideTimer);
    scrollHideTimer = setTimeout(() => {
      if (!isDraggingScrubber) {
        isScrolling = false;
      }
    }, 1200);
  }

  function scrollToMonth(key: string, title: string) {
    activeScrubMonth = title;
    activeScrubKey = key;
    isScrolling = true;
    clearTimeout(scrollHideTimer);
    scrollHideTimer = setTimeout(() => {
      isScrolling = false;
    }, 1800);

    const el = document.getElementById(`month-group-${key}`);
    if (el && scrollContainer) {
      const containerRect = scrollContainer.getBoundingClientRect();
      const elRect = el.getBoundingClientRect();
      const targetScroll = scrollContainer.scrollTop + (elRect.top - containerRect.top) - 10;
      scrollContainer.scrollTo({ top: Math.max(0, targetScroll), behavior: 'smooth' });
    }
  }

  function startScrubberDrag(e: MouseEvent | TouchEvent) {
    isDraggingScrubber = true;
    isScrolling = true;
    handleScrubberMove(e);

    function onMove(ev: MouseEvent | TouchEvent) {
      handleScrubberMove(ev);
    }

    function onEnd() {
      isDraggingScrubber = false;
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onEnd);
      window.removeEventListener('touchmove', onMove);
      window.removeEventListener('touchend', onEnd);
      clearTimeout(scrollHideTimer);
      scrollHideTimer = setTimeout(() => {
        isScrolling = false;
      }, 1200);
    }

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onEnd);
    window.addEventListener('touchmove', onMove);
    window.addEventListener('touchend', onEnd);
  }

  function handleScrubberMove(e: MouseEvent | TouchEvent) {
    if (!scrubberTrackEl || !scrollContainer) return;
    const rect = scrubberTrackEl.getBoundingClientRect();
    const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
    const offsetY = Math.max(0, Math.min(rect.height, clientY - rect.top));
    const ratio = offsetY / rect.height;

    const maxScroll = Math.max(1, scrollContainer.scrollHeight - scrollContainer.clientHeight);
    scrollContainer.scrollTop = ratio * maxScroll;
    scrollThumbTopPercent = ratio * 100;
    updateActiveMonth();
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
                <div
                  class="grid-card-wrap"
                  role="button"
                  tabindex="0"
                  on:click={() => handleMemoryClick(memory)}
                  on:contextmenu={(e) => openContextMenu(e, memory)}
                  on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleMemoryClick(memory)}
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

                  <!-- Hover Overlay Subtext info -->
                  <div class="card-caption-strip">
                    <span class="card-date-text">{memory.dateFormatted}</span>
                    <div class="card-meta-right">
                      {#if memory.isLate && ($memoryHeaderSettings.showLatePillsInGrid ?? true)}
                        <span
                          class="card-late-pill"
                          title={memory.lateExact ? `${memory.lateExact} (Posted ${memory.timeFormatted})` : (memory.lateDuration ? `${memory.lateDuration} (Posted ${memory.timeFormatted})` : 'Posted late')}
                        >
                          {memory.lateDuration || 'Late'}
                        </span>
                      {/if}
                      {#if memory.locationName}
                        <div class="card-loc-pill" title={memory.locationName}>
                          <MapPin size={10} />
                          <span>{memory.city || memory.locationName}</span>
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
    <div
      bind:this={scrubberTrackEl}
      class="timeline-scrubber-track"
      role="slider"
      tabindex="0"
      aria-label="Timeline Month Scrubber"
      aria-valuenow={Math.round(scrollThumbTopPercent)}
      on:mousedown={startScrubberDrag}
      on:touchstart|passive={startScrubberDrag}
      title="Drag scrubber or click markers to jump through timeline"
    >
      <div class="timeline-line-rail"></div>

      <!-- Scrubber Thumb Pill -->
      <div
        class="timeline-scrubber-thumb"
        class:is-active={isDraggingScrubber}
        style="top: {scrollThumbTopPercent}%;"
      ></div>

      <!-- Month Checkmarkers along track placed at exact section percentage -->
      {#each monthPositions as pos}
        {@const isGroupActive = pos.key === activeScrubKey}
        <button
          type="button"
          class="timeline-month-dot"
          class:is-active={isGroupActive}
          style="top: {pos.topPercent}%;"
          on:click|stopPropagation={() => scrollToMonth(pos.key, pos.title)}
          title="Jump to {pos.title}"
          aria-label="Jump to {pos.title}"
        >
          <span class="dot-inner-core"></span>
        </button>
      {/each}

      <!-- Floating Scrub/Scroll Date Popup Pill (Aligned with Thumb) -->
      {#if isScrolling || isDraggingScrubber}
        <div
          class="timeline-floating-badge"
          style="top: {scrollThumbTopPercent}%;"
        >
          <Calendar size={13} class="text-sky-400" />
          <span class="floating-badge-text">{activeScrubMonth || 'Timeline'}</span>
          <div class="badge-pointer-arrow"></div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .memories-grid-wrapper {
    position: relative;
    width: 100%;
    min-height: 480px;
  }

  .memories-scroll-viewport {
    width: 100%;
    max-height: calc(100vh - 170px);
    overflow-y: auto;
    overflow-x: hidden;
    padding-right: 36px;
    position: relative;
    scroll-behavior: smooth;
  }

  .memories-groups-container {
    display: flex;
    flex-direction: column;
    gap: 32px;
    width: 100%;
  }

  .month-section {
    display: flex;
    flex-direction: column;
    gap: 14px;
    width: 100%;
    content-visibility: auto;
    contain-intrinsic-size: auto 380px;
    contain: layout style;
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
    contain: layout style paint;
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

  .card-caption-strip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    padding: 6px 4px 2px 4px;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .card-date-text {
    font-weight: 600;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-meta-right {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
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
    max-width: 80px;
  }

  /* Fixed Vertical Timeline Fast Scrubber Track */
  .timeline-scrubber-track {
    position: absolute;
    right: 6px;
    top: 20px;
    bottom: 24px;
    width: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: ns-resize;
    user-select: none;
    z-index: 80;
  }

  .timeline-line-rail {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 3px;
    background: rgba(255, 255, 255, 0.14);
    border-radius: 999px;
    transition: background 0.15s ease, width 0.15s ease;
  }

  .timeline-scrubber-track:hover .timeline-line-rail {
    background: rgba(255, 255, 255, 0.28);
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
    z-index: 85;
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
    z-index: 82;
    transition: transform 0.15s ease;
  }

  .timeline-month-dot:hover {
    transform: translate(-50%, -50%) scale(1.35);
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
