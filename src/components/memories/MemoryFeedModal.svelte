<script lang="ts">
  import { tick, onMount, onDestroy } from 'svelte';
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
  import ChevronUp from 'lucide-svelte/icons/chevron-up';
  import ChevronDown from 'lucide-svelte/icons/chevron-down';
  import ChevronLeft from 'lucide-svelte/icons/chevron-left';
  import ChevronRight from 'lucide-svelte/icons/chevron-right';

  let activeIndex = 0;
  let lastMemoryId = '';

  // Sync activeIndex whenever activeFeedMemory changes
  $: if ($activeFeedMemory && $activeFeedMemory.id !== lastMemoryId) {
    lastMemoryId = $activeFeedMemory.id;
    const foundIdx = $filteredMemories.findIndex((m) => m.id === $activeFeedMemory?.id);
    if (foundIdx !== -1) {
      activeIndex = foundIdx;
    }
  }

  $: currentMemory = $filteredMemories[activeIndex] || $activeFeedMemory;

  function goToIndex(idx: number) {
    if (idx < 0 || idx >= $filteredMemories.length) return;
    activeIndex = idx;
    const target = $filteredMemories[idx];
    if (target) {
      lastMemoryId = target.id;
      activeFeedMemory.set(target);
    }
  }

  function handlePrev() {
    if (activeIndex > 0) {
      goToIndex(activeIndex - 1);
    }
  }

  function handleNext() {
    if (activeIndex < $filteredMemories.length - 1) {
      goToIndex(activeIndex + 1);
    }
  }

  let wheelCooldown = false;
  function handleWheel(e: WheelEvent) {
    if (wheelCooldown) return;
    if (Math.abs(e.deltaY) > 28) {
      wheelCooldown = true;
      if (e.deltaY > 0) {
        handleNext();
      } else {
        handlePrev();
      }
      setTimeout(() => {
        wheelCooldown = false;
      }, 260);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!$activeFeedMemory) return;
    if (e.key === 'Escape') {
      closeFeed();
    } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft' || e.key === 'k') {
      e.preventDefault();
      handlePrev();
    } else if (e.key === 'ArrowDown' || e.key === 'ArrowRight' || e.key === 'j' || e.key === ' ') {
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

  $: userName = $explorerData?.userName || 'toxel';
  $: profilePic = $explorerData?.profilePictureDataUrl || '';
  $: locText = currentMemory ? formatMemoryLocation(currentMemory, $memoryHeaderSettings) : '';
  $: timeText = currentMemory ? formatMemoryTimeTag(currentMemory, $memoryHeaderSettings) : '';
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $activeFeedMemory && currentMemory}
  <div
    class="feed-modal-backdrop"
    transition:fade={{ duration: 180 }}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click={(e) => e.target === e.currentTarget && closeFeed()}
    on:keydown={(e) => e.key === 'Escape' && closeFeed()}
    on:wheel={handleWheel}
  >
    <!-- Left / Prev Desktop Floating Button -->
    {#if activeIndex > 0}
      <button
        type="button"
        class="floating-nav-btn nav-left"
        on:click={handlePrev}
        title="Previous Memory (← or ↑)"
        aria-label="Previous Memory"
      >
        <ChevronLeft size={24} />
      </button>
    {/if}

    <!-- Right / Next Desktop Floating Button -->
    {#if activeIndex < $filteredMemories.length - 1}
      <button
        type="button"
        class="floating-nav-btn nav-right"
        on:click={handleNext}
        title="Next Memory (→ or ↓)"
        aria-label="Next Memory"
      >
        <ChevronRight size={24} />
      </button>
    {/if}

    <div
      class="feed-modal-shell"
      transition:scale={{ start: 0.95, duration: 200, opacity: 0, easing: cubicOut }}
      role="document"
    >
      <!-- Top Bar -->
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

      <!-- Feed Content Stage -->
      <div class="feed-stage-viewport">
        <article class="feed-card">
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
          {#if currentMemory.caption}
            <div class="post-header-caption-wrap">
              <p class="post-header-caption-text">{currentMemory.caption}</p>
            </div>
          {/if}

          <!-- Dual Camera Frame -->
          <div class="dual-frame-wrapper">
            {#key currentMemory.id}
              <DualCameraFrame
                primarySrc={currentMemory.primaryPath}
                secondarySrc={currentMemory.secondaryPath}
                btsSrc={currentMemory.btsPath}
                isVideo={currentMemory.isVideo}
                alt="BeReal {currentMemory.dateFormatted}"
                size="lg"
                interactive={true}
              />
            {/key}
          </div>
        </article>
      </div>

      <!-- Bottom Quick-Bar & Stepper -->
      <div class="feed-bottom-bar">
        <button
          type="button"
          class="bottom-step-btn"
          disabled={activeIndex === 0}
          on:click={handlePrev}
          title="Previous Memory"
        >
          <ChevronUp size={16} />
          <span>Previous</span>
        </button>

        <span class="stepper-count-pill">{activeIndex + 1} / {$filteredMemories.length}</span>

        <button
          type="button"
          class="bottom-step-btn"
          disabled={activeIndex >= $filteredMemories.length - 1}
          on:click={handleNext}
          title="Next Memory"
        >
          <span>Next</span>
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

  .floating-nav-btn {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: rgba(26, 26, 36, 0.85);
    border: 1px solid var(--border-medium);
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 510;
    transition: all var(--transition-fast);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
  }

  .floating-nav-btn:hover {
    background: #252536;
    border-color: rgba(255, 255, 255, 0.3);
    transform: translateY(-50%) scale(1.08);
  }

  .nav-left {
    left: 24px;
  }

  .nav-right {
    right: 24px;
  }

  @media (max-width: 768px) {
    .floating-nav-btn {
      display: none;
    }
  }

  .feed-modal-shell {
    position: relative;
    width: 100%;
    max-width: 530px;
    height: 94vh;
    max-height: 900px;
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
    position: relative;
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

  .feed-stage-viewport {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 12px 16px;
  }

  .feed-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    width: 100%;
    max-width: 480px;
    animation: cardFadeIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes cardFadeIn {
    from {
      opacity: 0;
      transform: scale(0.98);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
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

  .feed-bottom-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 18px;
    background: rgba(0, 0, 0, 0.92);
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    flex-shrink: 0;
  }

  .bottom-step-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: var(--radius-full);
    background: #14141e;
    border: 1px solid var(--border-medium);
    color: #ffffff;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .bottom-step-btn:hover:not(:disabled) {
    background: #252536;
    border-color: rgba(255, 255, 255, 0.3);
  }

  .bottom-step-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .stepper-count-pill {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
</style>
