<script lang="ts">
  import { tick, onMount, onDestroy } from 'svelte';
  import { fade } from 'svelte/transition';
  import {
    activeFeedMemory,
    closeFeed,
    filteredMemories,
    explorerData,
    openExportModal,
    exportPreferences,
    memoryHeaderSettings,
    formatMemoryLocation,
    formatMemoryTimeTag,
  } from '$lib/memoriesStore';
  import { exportSingleMemory } from '$lib/tauri';
  import { save } from '@tauri-apps/plugin-dialog';
  import type { ExplorerMemory } from '$lib/types';
  import DualCameraFrame from './DualCameraFrame.svelte';
  import MemoryActionMenu from './MemoryActionMenu.svelte';
  import ArrowLeft from 'lucide-svelte/icons/arrow-left';
  import Download from 'lucide-svelte/icons/download';
  import ChevronUp from 'lucide-svelte/icons/chevron-up';
  import ChevronDown from 'lucide-svelte/icons/chevron-down';

  let scrollContainer: HTMLElement | null = null;
  let activeIndex = 0;
  let hasScrolledToInitial = false;
  let initialTargetId = '';
  let scrollTimeout: any = null;

  $: currentMemory = $filteredMemories[activeIndex] || $activeFeedMemory;

  // When activeFeedMemory opens, record target ID and trigger scroll after DOM mounts
  $: if ($activeFeedMemory && $activeFeedMemory.id !== initialTargetId) {
    initialTargetId = $activeFeedMemory.id;
    hasScrolledToInitial = false;
    const foundIdx = $filteredMemories.findIndex((m) => m.id === $activeFeedMemory?.id);
    if (foundIdx !== -1) {
      activeIndex = foundIdx;
    }
    scrollToInitialMemory();
  }

  async function scrollToInitialMemory() {
    await tick();
    if (!initialTargetId || !scrollContainer) return;
    const el = document.getElementById(`feed-card-${initialTargetId}`);
    if (el) {
      el.scrollIntoView({ block: 'center', behavior: 'instant' as ScrollBehavior });
      hasScrolledToInitial = true;
    }
  }

  function handleScroll() {
    if (!scrollContainer || !hasScrolledToInitial) return;
    clearTimeout(scrollTimeout);
    scrollTimeout = setTimeout(() => {
      updateActiveIndexFromScroll();
    }, 40);
  }

  function updateActiveIndexFromScroll() {
    if (!scrollContainer) return;
    const containerRect = scrollContainer.getBoundingClientRect();
    const centerY = containerRect.top + containerRect.height / 2;

    let closestIdx = activeIndex;
    let minDistance = Infinity;

    for (let i = 0; i < $filteredMemories.length; i++) {
      const mem = $filteredMemories[i];
      const el = document.getElementById(`feed-card-${mem.id}`);
      if (el) {
        const rect = el.getBoundingClientRect();
        const cardCenterY = rect.top + rect.height / 2;
        const dist = Math.abs(centerY - cardCenterY);
        if (dist < minDistance) {
          minDistance = dist;
          closestIdx = i;
        }
      }
    }

    if (closestIdx !== activeIndex) {
      activeIndex = closestIdx;
    }
  }

  function scrollToIndex(idx: number, smooth: boolean = true) {
    if (idx < 0 || idx >= $filteredMemories.length || !scrollContainer) return;
    const mem = $filteredMemories[idx];
    const el = document.getElementById(`feed-card-${mem.id}`);
    if (el) {
      el.scrollIntoView({ block: 'center', behavior: smooth ? 'smooth' : 'instant' as ScrollBehavior });
      activeIndex = idx;
    }
  }

  function handlePrev() {
    if (activeIndex > 0) {
      scrollToIndex(activeIndex - 1);
    }
  }

  function handleNext() {
    if (activeIndex < $filteredMemories.length - 1) {
      scrollToIndex(activeIndex + 1);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!$activeFeedMemory) return;
    if (e.key === 'Escape') {
      closeFeed();
    } else if (e.key === 'ArrowUp' || e.key === 'k') {
      e.preventDefault();
      handlePrev();
    } else if (e.key === 'ArrowDown' || e.key === 'j' || e.key === ' ') {
      e.preventDefault();
      handleNext();
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

  onDestroy(() => {
    clearTimeout(scrollTimeout);
  });

  $: userName = $explorerData?.userName || 'toxel';
  $: profilePic = $explorerData?.profilePictureDataUrl || '';
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $activeFeedMemory && currentMemory}
  <div
    class="feed-modal-backdrop"
    transition:fade={{ duration: 160 }}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click={(e) => e.target === e.currentTarget && closeFeed()}
    on:keydown={(e) => e.key === 'Escape' && closeFeed()}
  >
    <div
      class="feed-modal-shell"
      role="document"
    >
      <!-- Sticky Top Header -->
      <div class="feed-top-bar">
        <button
          type="button"
          class="back-nav-btn"
          on:click={closeFeed}
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

      <!-- Natural Continuous Infinite Scroll Feed Viewport -->
      <div
        bind:this={scrollContainer}
        class="feed-scroll-viewport custom-thick-scrollbar"
        on:scroll={handleScroll}
      >
        <div class="feed-cards-container">
          {#each $filteredMemories as memory (memory.id)}
            {@const locText = formatMemoryLocation(memory, $memoryHeaderSettings)}
            {@const timeText = formatMemoryTimeTag(memory, $memoryHeaderSettings)}

            <article id="feed-card-{memory.id}" class="feed-post-card">
              <!-- Post Header -->
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
                        <span class="time-text">{timeText}</span>
                      {/if}
                    </div>
                  {/if}
                </div>
              </div>

              <!-- Caption -->
              {#if memory.caption}
                <div class="post-header-caption-wrap">
                  <p class="post-header-caption-text">{memory.caption}</p>
                </div>
              {/if}

              <!-- Dual Camera Frame (Full size) -->
              <div class="dual-frame-wrapper">
                <DualCameraFrame
                  primarySrc={memory.primaryPath}
                  secondarySrc={memory.secondaryPath}
                  btsSrc={memory.btsPath}
                  isVideo={memory.isVideo}
                  alt="BeReal {memory.dateFormatted}"
                  size="lg"
                  interactive={true}
                />
              </div>

              <!-- Divider line between items in the infinite feed -->
              <div class="feed-card-divider"></div>
            </article>
          {/each}
        </div>
      </div>

      <!-- Quick Prev / Next Floating Navigation Arrows -->
      <div class="feed-floating-nav">
        <button
          type="button"
          class="nav-circle-btn"
          disabled={activeIndex === 0}
          on:click={handlePrev}
          title="Previous Memory (↑)"
        >
          <ChevronUp size={16} />
        </button>
        <button
          type="button"
          class="nav-circle-btn"
          disabled={activeIndex >= $filteredMemories.length - 1}
          on:click={handleNext}
          title="Next Memory (↓)"
        >
          <ChevronDown size={16} />
        </button>
      </div>
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
    padding: 12px 18px;
    background: rgba(0, 0, 0, 0.92);
    backdrop-filter: blur(14px);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    z-index: 40;
    flex-shrink: 0;
  }

  .feed-scroll-viewport {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    scroll-behavior: smooth;
    padding: 18px 16px 40px 16px;
    scroll-snap-type: y proximity;
  }

  .feed-cards-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 36px;
    width: 100%;
  }

  .feed-post-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    width: 100%;
    max-width: 480px;
    scroll-snap-align: center;
  }

  .feed-card-divider {
    width: 100%;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.12), transparent);
    margin-top: 24px;
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
    padding: 6px 12px;
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

  .quick-download-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: #14141e;
    border: 1px solid var(--border-medium);
    color: #ffffff;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .quick-download-btn:hover {
    background: #252536;
    border-color: rgba(255, 255, 255, 0.3);
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

  .subtitle-bullet {
    opacity: 0.5;
  }

  .post-header-caption-wrap {
    width: 100%;
    padding: 0 4px;
  }

  .post-header-caption-text {
    font-size: 13.5px;
    line-height: 1.4;
    color: #ffffff;
    margin: 0;
    word-break: break-word;
  }

  .dual-frame-wrapper {
    width: 100%;
    display: flex;
    justify-content: center;
  }

  .feed-floating-nav {
    position: absolute;
    right: 18px;
    bottom: 24px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 50;
  }

  .nav-circle-btn {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    background: rgba(22, 22, 32, 0.88);
    backdrop-filter: blur(10px);
    border: 1px solid var(--border-medium);
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all var(--transition-fast);
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.6);
  }

  .nav-circle-btn:hover:not(:disabled) {
    background: #2b2b3e;
    border-color: rgba(255, 255, 255, 0.3);
    transform: scale(1.08);
  }

  .nav-circle-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
</style>
