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

  const CARD_HEIGHT = 700; // Height per card slot (Header + 3:4 Frame + Caption + Padding)

  let scrollContainer: HTMLElement | null = null;
  let scrollTop = 0;
  let activeIndex = 0;
  let lastJumpedId: string | null = null;
  let isJumping = false;

  $: totalVirtualHeight = $filteredMemories.length * CARD_HEIGHT;

  // Windowed virtual range: Render 1 card before and 3 cards ahead (max 5 in DOM)
  $: windowStartIndex = Math.max(0, Math.floor(scrollTop / CARD_HEIGHT) - 1);
  $: windowEndIndex = Math.min($filteredMemories.length, windowStartIndex + 5);
  $: visibleSlice = $filteredMemories.slice(windowStartIndex, windowEndIndex);

  $: currentMemory = $filteredMemories[activeIndex] || $activeFeedMemory;

  // Jump to active memory ONLY when the active memory ID changes (guarded to prevent recursive reactive loops)
  $: if ($activeFeedMemory && scrollContainer && $activeFeedMemory.id !== lastJumpedId) {
    lastJumpedId = $activeFeedMemory.id;
    jumpToActiveMemory();
  }

  async function jumpToActiveMemory() {
    if (!$activeFeedMemory || !scrollContainer) return;
    const targetIdx = $filteredMemories.findIndex((m) => m.id === $activeFeedMemory?.id);
    if (targetIdx !== -1) {
      activeIndex = targetIdx;
      isJumping = true;
      await tick();
      if (!scrollContainer) return;
      const vh = scrollContainer.clientHeight || 750;
      const centerOffset = Math.max(0, (vh - CARD_HEIGHT) / 2);
      const targetScroll = Math.max(0, targetIdx * CARD_HEIGHT - centerOffset);
      scrollTop = targetScroll;
      scrollContainer.scrollTop = targetScroll;
      setTimeout(() => {
        isJumping = false;
      }, 50);
    }
  }

  function handleScroll(e: Event) {
    const el = e.currentTarget as HTMLElement;
    if (!el) return;
    scrollTop = el.scrollTop;
    if (!isJumping) {
      const newIdx = Math.min(
        $filteredMemories.length - 1,
        Math.max(0, Math.round((scrollTop + CARD_HEIGHT * 0.4) / CARD_HEIGHT))
      );
      if (newIdx !== activeIndex) {
        activeIndex = newIdx;
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!$activeFeedMemory) return;
    if (e.key === 'Escape') {
      closeFeed();
    } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft' || e.key === 'k') {
      e.preventDefault();
      if (scrollContainer && activeIndex > 0) {
        scrollContainer.scrollBy({ top: -CARD_HEIGHT, behavior: 'smooth' });
      }
    } else if (e.key === 'ArrowDown' || e.key === 'ArrowRight' || e.key === 'j' || e.key === ' ') {
      e.preventDefault();
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
    transition:fade={{ duration: 200 }}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click={(e) => e.target === e.currentTarget && handleClose()}
    on:keydown={(e) => e.key === 'Escape' && handleClose()}
  >
    <div
      class="feed-modal-shell"
      transition:scale={{ start: 0.94, duration: 220, opacity: 0, easing: cubicOut }}
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

      <!-- High-Performance Virtual Windowed Infinite Scroll Viewport -->
      <div
        bind:this={scrollContainer}
        class="feed-virtual-viewport custom-thick-scrollbar"
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

              <!-- Divider between items in infinite scroll -->
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
</style>
