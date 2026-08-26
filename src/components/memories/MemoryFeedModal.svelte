<script lang="ts">
  import { tick, onDestroy } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import {
    activeFeedMemory,
    closeFeed,
    filteredMemories,
    explorerData,
    openExportModal,
    exportPreferences,
    memoryHeaderSettings,
    showMemoryDebugBadges,
    formatMemoryLocation,
    formatMemoryTimeTag,
  } from '$lib/memoriesStore';
  import { exportSingleMemory } from '$lib/tauri';
  import { save } from '@tauri-apps/plugin-dialog';
  import DualCameraFrame from './DualCameraFrame.svelte';
  import MemoryActionMenu from './MemoryActionMenu.svelte';
  import PerspectiveSwitcher from './PerspectiveSwitcher.svelte';
  import ArrowLeft from 'lucide-svelte/icons/arrow-left';
  import Download from 'lucide-svelte/icons/download';

  const CARD_HEIGHT = 750; // Reference height for virtual spacers

  let scrollContainer: HTMLElement | null = null;
  let activeIndex = 0;
  let scrollTop = 0;
  let lastJumpedId: string | null = null;
  let isJumping = false;

  // Windowed virtual rendering: Render 2 items before and 2 items ahead (max 5 cards in DOM)
  $: windowStart = Math.max(0, activeIndex - 2);
  $: windowEnd = Math.min($filteredMemories.length, activeIndex + 3);
  $: visibleSlice = $filteredMemories.slice(windowStart, windowEnd);

  $: topSpacerHeight = windowStart * CARD_HEIGHT;
  $: bottomSpacerHeight = Math.max(0, ($filteredMemories.length - windowEnd) * CARD_HEIGHT);

  $: currentMemory = $filteredMemories[activeIndex] || $activeFeedMemory;

  // Trigger jump only once per memory opening to avoid recursive loops
  $: if ($activeFeedMemory && scrollContainer && $activeFeedMemory.id !== lastJumpedId) {
    lastJumpedId = $activeFeedMemory.id;
    const targetIdx = $filteredMemories.findIndex((m) => m.id === $activeFeedMemory?.id);
    if (targetIdx !== -1) {
      activeIndex = targetIdx;
      jumpToActiveMemory(targetIdx);
    }
  }

  async function jumpToActiveMemory(targetIdx: number) {
    isJumping = true;
    await tick();
    if (!scrollContainer) return;

    // Approximate initial scroll position
    const vh = scrollContainer.clientHeight || 750;
    const centerOffset = Math.max(0, (vh - CARD_HEIGHT) / 2);
    const targetScroll = Math.max(0, targetIdx * CARD_HEIGHT - centerOffset);
    scrollContainer.scrollTop = targetScroll;
    scrollTop = targetScroll;

    // Ensure DOM is fully mounted, then compute exact pixel-perfect vertical centering
    await tick();
    const targetMem = $filteredMemories[targetIdx];
    if (targetMem && scrollContainer) {
      const cardEl = document.getElementById(`feed-card-${targetMem.id}`);
      if (cardEl) {
        const containerRect = scrollContainer.getBoundingClientRect();
        const cardRect = cardEl.getBoundingClientRect();
        const currentCardTopRelativeToContainer = cardRect.top - containerRect.top + scrollContainer.scrollTop;
        const perfectCenterScroll = Math.max(0, currentCardTopRelativeToContainer - (containerRect.height - cardRect.height) / 2);
        scrollContainer.scrollTop = perfectCenterScroll;
        scrollTop = perfectCenterScroll;
      }
    }

    setTimeout(() => {
      isJumping = false;
    }, 80);
  }

  function handleScroll(e: Event) {
    const el = e.currentTarget as HTMLElement;
    if (!el) return;
    scrollTop = el.scrollTop;
    if (!isJumping) {
      const newIdx = Math.min(
        $filteredMemories.length - 1,
        Math.max(0, Math.round((scrollTop + CARD_HEIGHT * 0.35) / CARD_HEIGHT))
      );
      if (newIdx !== activeIndex) {
        activeIndex = newIdx;
      }
    }
  }

  function scrollToIndex(idx: number) {
    if (idx < 0 || idx >= $filteredMemories.length || !scrollContainer) return;
    activeIndex = idx;
    const targetMem = $filteredMemories[idx];
    if (targetMem) {
      const cardEl = document.getElementById(`feed-card-${targetMem.id}`);
      if (cardEl && scrollContainer) {
        const containerRect = scrollContainer.getBoundingClientRect();
        const cardRect = cardEl.getBoundingClientRect();
        const currentCardTopRelativeToContainer = cardRect.top - containerRect.top + scrollContainer.scrollTop;
        const perfectCenterScroll = Math.max(0, currentCardTopRelativeToContainer - (containerRect.height - cardRect.height) / 2);
        scrollContainer.scrollTo({ top: perfectCenterScroll, behavior: 'smooth' });
        return;
      }
    }
    const vh = scrollContainer.clientHeight || 750;
    const centerOffset = Math.max(0, (vh - CARD_HEIGHT) / 2);
    const targetScroll = Math.max(0, idx * CARD_HEIGHT - centerOffset);
    scrollContainer.scrollTo({ top: targetScroll, behavior: 'smooth' });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!$activeFeedMemory) return;
    if (e.key === 'Escape') {
      handleClose();
    } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft' || e.key === 'k') {
      e.preventDefault();
      if (activeIndex > 0) {
        scrollToIndex(activeIndex - 1);
      }
    } else if (e.key === 'ArrowDown' || e.key === 'ArrowRight' || e.key === 'j' || e.key === ' ') {
      e.preventDefault();
      if (activeIndex < $filteredMemories.length - 1) {
        scrollToIndex(activeIndex + 1);
      }
    }
  }

  async function handleQuickDownload(mem: ExplorerMemory) {
    if (!mem.primaryPath) return;
    const prefs = $exportPreferences;
    if (!prefs.isDefaultSet) {
      openExportModal(mem);
      return;
    }

    try {
      const isVideo = prefs.exportType === 'bts_only';
      const ext = isVideo ? 'mp4' : prefs.format.toLowerCase() === 'png' ? 'png' : prefs.format.toLowerCase() === 'webp' ? 'webp' : 'jpg';
      const defaultFilename = `${mem.takenAt.slice(0, 10)}_${prefs.exportType}.${ext}`;

      const savePath = await save({
        defaultPath: defaultFilename,
        filters: isVideo ? [{ name: 'MP4 Video', extensions: ['mp4'] }] : [{ name: 'Image', extensions: [ext] }],
      });

      if (!savePath) return;

      await exportSingleMemory({
        memoryIndex: mem.index,
        primaryPath: mem.primaryPath,
        secondaryPath: mem.secondaryPath,
        btsPath: mem.btsPath,
        outputPath: savePath,
        exportType: prefs.exportType,
        format: prefs.format,
        quality: prefs.quality || 92,
        embedExif: prefs.embedExif,
        takenAt: mem.takenAt,
        latitude: prefs.embedGps && mem.location ? mem.location.latitude : undefined,
        longitude: prefs.embedGps && mem.location ? mem.location.longitude : undefined,
        caption: mem.caption,
      });
    } catch (e) {
      console.error('Quick download failed:', e);
      openExportModal(mem);
    }
  }

  function handleClose() {
    lastJumpedId = null;
    closeFeed();
  }

  $: userName = $explorerData?.userName || 'toxel';
  $: profilePic = $explorerData?.profilePictureDataUrl || '';
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $activeFeedMemory && currentMemory}
  <div
    class="feed-modal-backdrop"
    transition:fade={{ duration: 240, easing: cubicOut }}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click={(e) => e.target === e.currentTarget && handleClose()}
    on:keydown={(e) => e.key === 'Escape' && handleClose()}
  >
    <div
      class="feed-modal-shell"
      in:scale={{ start: 0.88, duration: 280, opacity: 0, easing: cubicOut }}
      out:scale={{ start: 0.94, duration: 200, opacity: 0, easing: cubicOut }}
      role="document"
    >
      <!-- Sticky Top Header -->
      <div class="feed-top-bar">
        <button
          type="button"
          class="back-nav-btn"
          on:click={handleClose}
          title="Back to Grid / Calendar (Esc)"
          aria-label="Close feed"
        >
          <ArrowLeft size={16} />
          <span>Memories</span>
        </button>

        <div class="top-date-indicator">
          <span class="top-date-text">{currentMemory.dateFormatted}</span>
          <span class="top-index-text">{activeIndex + 1} of {$filteredMemories.length}</span>
        </div>

        <div class="top-actions">
          <button
            type="button"
            class="quick-download-btn"
            on:click={() => handleQuickDownload(currentMemory)}
            title="Export / Download this memory"
            aria-label="Download memory"
          >
            <Download size={14} />
          </button>
          <MemoryActionMenu memory={currentMemory} />
        </div>
      </div>

      <!-- High-Performance Windowed Infinite Scroll Viewport -->
      <div
        bind:this={scrollContainer}
        class="feed-scroll-viewport custom-thick-scrollbar"
        on:scroll={handleScroll}
      >
        <!-- Top Virtual Spacer (preserves scroll position without rendering off-screen DOM nodes) -->
        {#if topSpacerHeight > 0}
          <div class="virtual-spacer" style="height: {topSpacerHeight}px;"></div>
        {/if}

        <div class="feed-cards-container">
          {#each visibleSlice as memory (memory.id)}
            {@const locText = formatMemoryLocation(memory, $memoryHeaderSettings)}
            {@const timeText = formatMemoryTimeTag(memory, $memoryHeaderSettings)}

            <article id="feed-card-{memory.id}" class="feed-post-card">
              <!-- Post Header Row with Avatar, Name, Location/Time, Download & Action Menu -->
              <div class="post-header-row">
                <div class="user-avatar-wrap">
                  {#if profilePic}
                    <img src={profilePic} alt={userName} class="user-avatar-img" />
                  {:else}
                    <div class="user-avatar-placeholder">
                      <span>{userName.charAt(0).toUpperCase()}</span>
                    </div>
                  {/if}
                </div>

                <div class="user-meta-column">
                  <div class="user-name-row">
                    <span class="user-name">{userName}</span>
                  </div>

                  {#if locText || timeText}
                    <div class="user-subtitle-row">
                      {#if locText}
                        <span class="location-text">{locText}</span>
                      {/if}
                      {#if locText && timeText}
                        <span class="subtitle-bullet">•</span>
                      {/if}
                      {#if timeText}
                        {#if memory.isLate}
                          <span
                            class="time-text is-late-time"
                            title={memory.lateExact ? `${memory.lateExact} (${memory.timeFormatted})` : (memory.lateDuration ? `${memory.lateDuration} (${memory.timeFormatted})` : 'Posted late')}
                          >
                            {timeText} <span class="late-tag-accent">({memory.lateDuration || 'Late'})</span>
                          </span>
                        {:else}
                          <span class="time-text">{timeText}</span>
                        {/if}
                      {/if}
                    </div>
                  {/if}
                </div>

                <!-- Action Buttons directly on each memory post -->
                <div class="post-header-actions">
                  <button
                    type="button"
                    class="quick-download-btn"
                    on:click|stopPropagation={() => handleQuickDownload(memory)}
                    title="Export / Download this memory"
                    aria-label="Download memory"
                  >
                    <Download size={14} />
                  </button>
                  <MemoryActionMenu {memory} />
                </div>
              </div>

              <!-- Caption -->
              {#if memory.caption}
                <div class="post-header-caption-wrap">
                  <p class="post-header-caption-text">{memory.caption}</p>
                </div>
              {/if}

              <!-- BeReal Dual Camera Frame (Full size) -->
              <div class="dual-frame-wrapper">
                <DualCameraFrame
                  primarySrc={memory.primaryPath}
                  secondarySrc={memory.secondaryPath}
                  btsSrc={memory.btsPath}
                  isVideo={memory.isVideo}
                  alt="BeReal {memory.dateFormatted}"
                  size="lg"
                  interactive={true}
                  isLate={memory.isLate}
                  lateDuration={memory.lateDuration}
                  lateExact={memory.lateExact}
                  takenAt={memory.takenAt}
                  rawJson={memory.rawJson}
                  debugInfo={memory.debugInfo}
                />
              </div>

              <!-- Dev Debug JSON & Extraction Inspector (Hidden behind $showMemoryDebugBadges flag) -->
              {#if $showMemoryDebugBadges && (memory.rawJson || memory.debugInfo)}
                <details class="feed-dev-debug-accordion">
                  <summary class="debug-accordion-summary">
                    <span class="debug-tag {memory.isLate ? 'is-late' : 'is-ontime'}">
                      {memory.isLate ? `⚠️ LATE (${memory.lateDuration || 'Late'})` : '✓ ON TIME'}
                    </span>
                    <span class="debug-summary-title">🐞 Dev Debug JSON &amp; Extraction</span>
                  </summary>
                  <div class="debug-accordion-body">
                    <div class="debug-fields-grid">
                      <div><span class="field-k">isLate:</span> <span class="field-v">{String(memory.isLate)}</span></div>
                      <div><span class="field-k">lateDuration:</span> <span class="field-v">{memory.lateDuration || 'None'}</span></div>
                      <div><span class="field-k">lateInSeconds:</span> <span class="field-v">{memory.lateInSeconds ?? 'None'}</span></div>
                      <div><span class="field-k">takenAt:</span> <span class="field-v">{memory.takenAt}</span></div>
                      <div><span class="field-k">location:</span> <span class="field-v">{memory.location ? `${memory.location.latitude.toFixed(4)}, ${memory.location.longitude.toFixed(4)}` : 'None'}</span></div>
                      {#if memory.debugInfo}
                        <div class="full-w"><span class="field-k">debugInfo:</span> <span class="field-v">{memory.debugInfo}</span></div>
                      {/if}
                    </div>
                    {#if memory.rawJson}
                      <div class="debug-raw-json-wrap">
                        <span class="raw-json-heading">Raw Archive JSON:</span>
                        <pre class="raw-json-pre"><code>{(() => { try { return JSON.stringify(JSON.parse(memory.rawJson), null, 2); } catch { return memory.rawJson; } })()}</code></pre>
                      </div>
                    {/if}
                  </div>
                </details>
              {/if}

              <!-- Clean Spacing & Divider between items in infinite scroll -->
              <div class="feed-card-divider"></div>
            </article>
          {/each}
        </div>

        <!-- Bottom Virtual Spacer -->
        {#if bottomSpacerHeight > 0}
          <div class="virtual-spacer" style="height: {bottomSpacerHeight}px;"></div>
        {/if}
      </div>

      <!-- Always Visible Pinned Bottom-Left Perspective Toggle (Cannot be scrolled off) -->
      <PerspectiveSwitcher variant="floating-modal" />
    </div>
  </div>
{/if}

<style>
  .feed-modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.92);
    backdrop-filter: blur(24px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
  }

  .feed-modal-shell {
    position: relative;
    width: 100%;
    max-width: 530px;
    height: 96vh;
    display: flex;
    flex-direction: column;
    background: #000000;
    border: 1px solid var(--border-medium);
    border-radius: 28px;
    overflow: hidden;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.95);
  }

  @media (min-width: 1200px) {
    .feed-modal-shell {
      max-width: 550px;
    }
  }

  .feed-top-bar {
    position: sticky;
    top: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    background: rgba(0, 0, 0, 0.88);
    backdrop-filter: blur(14px);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    z-index: 40;
    flex-shrink: 0;
  }

  .feed-scroll-viewport {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    scroll-behavior: auto;
    padding: 20px 16px 48px 16px;
  }

  .virtual-spacer {
    width: 100%;
    flex-shrink: 0;
  }

  .feed-cards-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 32px;
    width: 100%;
  }

  .feed-post-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    width: 100%;
    max-width: 480px;
    box-sizing: border-box;
    animation: feedCardReveal 0.32s cubic-bezier(0.16, 1, 0.3, 1) backwards;
    will-change: transform, opacity;
  }

  @keyframes feedCardReveal {
    0% {
      opacity: 0.35;
      transform: scale(0.96) translateY(8px);
    }
    100% {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .feed-card-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.08);
    margin-top: 20px;
    width: 100%;
  }

  .back-nav-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: none;
    color: #ffffff;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    padding: 6px 10px;
    border-radius: var(--radius-full);
    transition: background var(--transition-fast);
  }

  .back-nav-btn:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .top-date-indicator {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .top-date-text {
    font-size: 13px;
    font-weight: 700;
    color: #ffffff;
  }

  .top-index-text {
    font-size: 10.5px;
    color: var(--text-muted);
  }

  .top-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .post-header-actions {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .quick-download-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .quick-download-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #ffffff;
  }

  .post-header-row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
  }

  .user-avatar-wrap {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    overflow: hidden;
    background: #181824;
    border: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .user-avatar-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .user-avatar-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #2a2a3c, #161622);
    color: #ffffff;
    font-weight: 700;
    font-size: 14px;
  }

  .user-meta-column {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .user-name {
    font-size: 13.5px;
    font-weight: 700;
    color: #ffffff;
    letter-spacing: -0.01em;
  }

  .user-subtitle-row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11.5px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .is-late-time {
    cursor: help;
    transition: color 0.15s ease;
  }

  .is-late-time:hover {
    color: #ffffff;
  }

  .late-tag-accent {
    color: #f87171;
    font-weight: 600;
  }

  .subtitle-bullet {
    opacity: 0.5;
  }

  .post-header-caption-wrap {
    width: 100%;
    padding: 0 4px;
  }

  .post-header-caption-text {
    font-size: 14px;
    font-weight: 500;
    color: #f4f4f5;
    line-height: 1.4;
    word-break: break-word;
  }

  .dual-frame-wrapper {
    width: 100%;
    display: flex;
    justify-content: center;
  }

  /* Dev Debug Accordion Card on Feed */
  .feed-dev-debug-accordion {
    width: 100%;
    margin-top: 8px;
    background: rgba(18, 18, 26, 0.95);
    border: 1px dashed rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-md);
    overflow: hidden;
    font-size: 11.5px;
  }

  .debug-accordion-summary {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 12px;
    cursor: pointer;
    background: rgba(255, 255, 255, 0.04);
    user-select: none;
    transition: background 0.15s ease;
  }

  .debug-accordion-summary:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .debug-tag {
    padding: 2px 7px;
    border-radius: var(--radius-full);
    font-size: 9.5px;
    font-weight: 800;
  }

  .debug-tag.is-late {
    background: rgba(239, 68, 68, 0.25);
    border: 1px solid #ef4444;
    color: #fca5a5;
  }

  .debug-tag.is-ontime {
    background: rgba(16, 185, 129, 0.25);
    border: 1px solid #10b981;
    color: #6ee7b7;
  }

  .debug-summary-title {
    color: var(--text-secondary);
    font-weight: 600;
  }

  .debug-accordion-body {
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }

  .debug-fields-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px 12px;
    font-family: var(--font-mono);
    font-size: 10.5px;
  }

  .debug-fields-grid .full-w {
    grid-column: 1 / -1;
  }

  .field-k {
    color: var(--text-muted);
    font-weight: 600;
  }

  .field-v {
    color: #38bdf8;
  }

  .debug-raw-json-wrap {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 4px;
  }

  .raw-json-heading {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .raw-json-pre {
    background: #09090d;
    padding: 8px 10px;
    border-radius: 6px;
    border: 1px solid var(--border-subtle);
    max-height: 180px;
    overflow-y: auto;
    font-family: var(--font-mono);
    font-size: 10px;
    color: #cbd5e1;
    white-space: pre-wrap;
    word-break: break-all;
  }
</style>
