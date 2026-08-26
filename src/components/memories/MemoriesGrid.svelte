<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { filteredMemories, openFeedAt, openContextMenu } from '$lib/memoriesStore';
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

  let renderedMonthCount = 4;

  $: visibleMonthGroups = monthGroups.slice(0, renderedMonthCount);

  function handleMemoryClick(memory: ExplorerMemory) {
    openFeedAt(memory);
  }

  function updateActiveMonth() {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    const maxScroll = scrollHeight - clientHeight;
    if (maxScroll > 0) {
      scrollThumbTopPercent = Math.min(100, Math.max(0, (scrollTop / maxScroll) * 100));
    }

    // Find the month currently visible in viewport
    const containerTop = scrollContainer.getBoundingClientRect().top;
    for (let i = visibleMonthGroups.length - 1; i >= 0; i--) {
      const group = visibleMonthGroups[i];
      const el = document.getElementById(`month-group-${group.key}`);
      if (el) {
        const elTop = el.getBoundingClientRect().top;
        if (elTop - containerTop <= 160) {
          activeScrubMonth = group.title;
          activeScrubKey = group.key;
          break;
        }
      }
    }
    if (!activeScrubMonth && visibleMonthGroups.length > 0) {
      activeScrubMonth = visibleMonthGroups[0].title;
      activeScrubKey = visibleMonthGroups[0].key;
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
    }, 1400);

    // Progressive infinite scroll: render next batch when near bottom
    if (scrollContainer) {
      const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
      if (scrollHeight - (scrollTop + clientHeight) < 700) {
        if (renderedMonthCount < monthGroups.length) {
          renderedMonthCount = Math.min(monthGroups.length, renderedMonthCount + 3);
        }
      }
    }
  }

  async function scrollToMonth(key: string, title: string) {
    const targetIdx = monthGroups.findIndex((g) => g.key === key);
    if (targetIdx !== -1 && targetIdx >= renderedMonthCount) {
      renderedMonthCount = Math.min(monthGroups.length, targetIdx + 2);
      await tick();
    }

    activeScrubMonth = title;
    activeScrubKey = key;
    isScrolling = true;
    clearTimeout(scrollHideTimer);
    scrollHideTimer = setTimeout(() => {
      isScrolling = false;
    }, 2000);

    const el = document.getElementById(`month-group-${key}`);
    if (el && scrollContainer) {
      const containerTop = scrollContainer.getBoundingClientRect().top;
      const elTop = el.getBoundingClientRect().top;
      const targetScroll = scrollContainer.scrollTop + (elTop - containerTop) - 12;
      scrollContainer.scrollTo({ top: targetScroll, behavior: 'smooth' });
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
      }, 1400);
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

    const maxScroll = scrollContainer.scrollHeight - scrollContainer.clientHeight;
    scrollContainer.scrollTop = ratio * maxScroll;
    scrollThumbTopPercent = ratio * 100;
    updateActiveMonth();
  }

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
        {#each visibleMonthGroups as group (group.key)}
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
                  />

                  <!-- Hover Overlay Subtext info -->
                  <div class="card-caption-strip">
                    <span class="card-date-text">{memory.dateFormatted}</span>
                    {#if memory.locationName}
                      <div class="card-loc-pill" title={memory.locationName}>
                        <MapPin size={10} />
                        <span>{memory.city || memory.locationName}</span>
                      </div>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          </section>
        {/each}
      </div>
    </div>

    <!-- Fixed Vertical Timeline Scrubber with Month Markers & Floating Date Popup (Stays fixed on screen) -->
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

      <!-- Month Checkmarkers along fixed right track -->
      {#each monthGroups as group, i}
        {@const dotPosPercent = monthGroups.length > 1 ? (i / (monthGroups.length - 1)) * 92 + 4 : 50}
        {@const isGroupActive = group.key === activeScrubKey}
        <button
          type="button"
          class="timeline-month-dot"
          class:is-active={isGroupActive}
          style="top: {dotPosPercent}%;"
          on:click|stopPropagation={() => scrollToMonth(group.key, group.title)}
          title="Jump to {group.title} ({group.memories.length} BeReals)"
          aria-label="Jump to {group.title}"
        >
          <span class="dot-inner-core"></span>
        </button>
      {/each}

      <!-- Floating Scrub/Scroll Date Popup Pill -->
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
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  /* Hide default scrollbar so only the custom timeline rail is visible */
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
  }

  .grid-card-wrap:hover {
    transform: translateY(-4px) scale(1.02);
    box-shadow: 0 12px 28px rgba(0, 0, 0, 0.7);
    z-index: 5;
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

  .card-loc-pill {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 90px;
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
    background: rgba(56, 189, 248, 0.4);
    width: 4px;
  }

  .timeline-scrubber-thumb {
    position: absolute;
    width: 10px;
    height: 24px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.4);
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.8);
    transform: translateY(-50%);
    pointer-events: none;
    transition: background 0.15s ease, width 0.15s ease;
  }

  .timeline-scrubber-thumb.is-active,
  .timeline-scrubber-track:hover .timeline-scrubber-thumb {
    background: #38bdf8;
    box-shadow: 0 0 14px rgba(56, 189, 248, 0.6);
    width: 12px;
  }

  .timeline-month-dot {
    position: absolute;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #181824;
    border: 1.5px solid rgba(255, 255, 255, 0.6);
    padding: 0;
    cursor: pointer;
    transform: translateY(-50%);
    box-shadow: 0 0 6px rgba(0, 0, 0, 0.9);
    transition: transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1), background 0.15s ease, border-color 0.15s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dot-inner-core {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: #ffffff;
    transition: transform 0.15s ease, background 0.15s ease;
  }

  .timeline-month-dot:hover {
    transform: translateY(-50%) scale(1.6);
    border-color: #38bdf8;
    background: #09090e;
    z-index: 10;
  }

  .timeline-month-dot:hover .dot-inner-core {
    background: #38bdf8;
    transform: scale(1.3);
  }

  .timeline-month-dot.is-active {
    border-color: #38bdf8;
    background: #38bdf8;
    transform: translateY(-50%) scale(1.35);
    box-shadow: 0 0 10px rgba(56, 189, 248, 0.65);
  }

  .timeline-month-dot.is-active .dot-inner-core {
    background: #09090e;
  }

  /* Floating Popup Badge */
  .timeline-floating-badge {
    position: absolute;
    right: 28px;
    transform: translateY(-50%);
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 7px 16px;
    background: rgba(16, 16, 24, 0.96);
    backdrop-filter: blur(20px);
    border: 1.5px solid rgba(56, 189, 248, 0.4);
    border-radius: 999px;
    color: #ffffff;
    font-size: 13px;
    font-weight: 800;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.85), 0 0 12px rgba(56, 189, 248, 0.25);
    white-space: nowrap;
    pointer-events: none;
    z-index: 90;
    animation: badgePopIn 0.16s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes badgePopIn {
    from {
      opacity: 0;
      transform: translateY(-50%) scale(0.9);
    }
    to {
      opacity: 1;
      transform: translateY(-50%) scale(1);
    }
  }

  .badge-pointer-arrow {
    position: absolute;
    right: -6px;
    top: 50%;
    transform: translateY(-50%) rotate(45deg);
    width: 10px;
    height: 10px;
    background: rgba(16, 16, 24, 0.96);
    border-top: 1.5px solid rgba(56, 189, 248, 0.4);
    border-right: 1.5px solid rgba(56, 189, 248, 0.4);
  }

  .empty-memories-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    text-align: center;
    background: #0f0f15;
    border: 1px dashed var(--border-medium);
    border-radius: var(--radius-lg);
  }

  .empty-icon-wrap {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: #181822;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 12px;
  }

  .empty-title {
    font-size: 16px;
    font-weight: 700;
    color: #ffffff;
    margin-bottom: 4px;
  }

  .empty-desc {
    font-size: 13px;
    color: var(--text-secondary);
    max-width: 320px;
  }

  @media (max-width: 600px) {
    .memories-grid {
      grid-template-columns: repeat(2, 1fr);
      gap: 10px;
    }
  }
</style>
