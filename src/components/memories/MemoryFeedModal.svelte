<script lang="ts">
  import { tick } from 'svelte';
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

  const CARD_HEIGHT = 700; // Total height per card slot (Header + 3:4 Frame + Caption + Spacing)

  let scrollContainer: HTMLElement | null = null;
  let scrollTop = 0;
  let activeIndex = 0;

  $: totalVirtualHeight = $filteredMemories.length * CARD_HEIGHT;

  // Windowed range: Render 1 card before and 3 cards ahead (max 5 in DOM)
  $: windowStartIndex = Math.max(0, Math.floor(scrollTop / CARD_HEIGHT) - 1);
  $: windowEndIndex = Math.min($filteredMemories.length, windowStartIndex + 4);
  $: visibleSlice = $filteredMemories.slice(windowStartIndex, windowEndIndex);

  $: currentMemory = $filteredMemories[activeIndex] || $activeFeedMemory;

  // When activeFeedMemory is set, immediately position virtual scroll container at target index centered
  $: if ($activeFeedMemory && scrollContainer) {
    jumpToActiveMemory();
  }

  async function jumpToActiveMemory() {
    const targetIdx = $filteredMemories.findIndex((m) => m.id === $activeFeedMemory?.id);
    if (targetIdx !== -1 && scrollContainer) {
      activeIndex = targetIdx;
      await tick();
      const vh = scrollContainer.clientHeight || 750;
      const centerOffset = Math.max(0, (vh - CARD_HEIGHT) / 2);
      const targetScroll = Math.max(0, (targetIdx * CARD_HEIGHT) - centerOffset);
      scrollTop = targetScroll;
      scrollContainer.scrollTop = targetScroll;
    }
  }

  function handleScroll(e: Event) {
    const el = e.currentTarget as HTMLElement;
    if (!el) return;
    scrollTop = el.scrollTop;
    // Pure mathematical index (zero layout queries)
    const newIdx = Math.min($filteredMemories.length - 1, Math.max(0, Math.round(scrollTop / CARD_HEIGHT)));
    if (newIdx !== activeIndex) {
      activeIndex = newIdx;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!$activeFeedMemory) return;
    if (e.key === 'Escape') {
      closeFeed();
    } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
      if (scrollContainer && activeIndex > 0) {
        scrollContainer.scrollBy({ top: -CARD_HEIGHT, behavior: 'smooth' });
      }
    } else if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
      if (scrollContainer && activeIndex < $filteredMemories.length - 1) {
        scrollContainer.scrollBy({ top: CARD_HEIGHT, behavior: 'smooth' });
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

  $: userName = $explorerData?.userName || 'toxel';
  $: profilePic = $explorerData?.profilePictureDataUrl || '';
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $activeFeedMemory && currentMemory}
  <div
    class="feed-modal-backdrop"
    transition:fade={{ duration: 220 }}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click={(e) => e.target === e.currentTarget && closeFeed()}
    on:keydown={(e) => e.key === 'Escape' && closeFeed()}
  >
    <div
      class="feed-modal-shell"
      transition:scale={{ start: 0.93, duration: 250, opacity: 0, easing: cubicOut }}
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

      <!-- High-Performance Virtual Scroll Viewport -->
      <div
        bind:this={scrollContainer}
        class="feed-virtual-viewport"
        on:scroll={handleScroll}
      >
        <!-- Virtual Spacer representing total scroll height -->
        <div class="virtual-canvas" style="height: {totalVirtualHeight}px;">
          {#each visibleSlice as memory, i (memory.id)}
            {@const globalIdx = windowStartIndex + i}
            {@const locText = formatMemoryLocation(memory, $memoryHeaderSettings)}
            {@const timeText = formatMemoryTimeTag(memory, $memoryHeaderSettings)}

            <article
              class="virtual-feed-card"
              style="top: {globalIdx * CARD_HEIGHT}px; height: {CARD_HEIGHT}px;"
            >
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

              <!-- BeReal Dual Camera Frame (Full size, with placeholder fallback while loading) -->
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

              <!-- Divider between items -->
              <div class="virtual-card-divider"></div>
            </article>
          {/each}
        </div>
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
    padding: 10px 16px;
    background: rgba(0, 0, 0, 0.88);
    backdrop-filter: blur(14px);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    z-index: 40;
    flex-shrink: 0;
  }

  .feed-virtual-viewport {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    scroll-behavior: auto;
    position: relative;
  }

  .virtual-canvas {
    position: relative;
    width: 100%;
  }

  .virtual-feed-card {
    position: absolute;
    left: 0;
    right: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px 16px 20px 16px;
    gap: 10px;
    box-sizing: border-box;
  }

  .virtual-card-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.08);
    margin-top: 14px;
    width: 100%;
    max-width: 480px;
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

  .post-header-row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    max-width: 480px;
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
    color: #ffffff;
    font-weight: 700;
    font-size: 14px;
    background: linear-gradient(135deg, #38bdf8 0%, #a855f7 100%);
  }

  .user-meta-column {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }

  .user-name-row {
    display: flex;
    align-items: baseline;
    gap: 5px;
  }

  .user-name {
    font-size: 15px;
    font-weight: 700;
    color: #ffffff;
    letter-spacing: -0.01em;
  }

  .user-subtitle-row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 500;
    color: #a1a1aa;
    line-height: 1.3;
  }

  .location-text {
    color: #d4d4d8;
    font-weight: 500;
  }

  .subtitle-bullet {
    color: #71717a;
    font-size: 10px;
  }

  .time-text {
    color: #a1a1aa;
    font-weight: 400;
  }

  .quick-download-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-full);
    background: #181824;
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .quick-download-btn:hover {
    background: rgba(56, 189, 248, 0.15);
    border-color: #38bdf8;
    color: #38bdf8;
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(56, 189, 248, 0.25);
  }

  .post-header-caption-wrap {
    width: 100%;
    max-width: 480px;
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
    max-width: 480px;
    display: flex;
    justify-content: center;
  }

  @media (max-width: 600px) {
    .feed-modal-shell {
      height: 100vh;
      max-height: 100vh;
      border-radius: 0;
      border: none;
    }
  }
</style>
