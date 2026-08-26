<script lang="ts">
  import { onMount } from 'svelte';
  import {
    explorerData,
    isLoadingMemories,
    memoriesLoadProgress,
    memoriesLoadError,
    activeExplorerView,
    activeFeedMemory,
    loadMemories,
  } from '$lib/memoriesStore';
  import { lastScannedArchivePath, currentArchive } from '$lib/stores';
  import MemoriesGrid from '$components/memories/MemoriesGrid.svelte';
  import CalendarGrid from '$components/memories/CalendarGrid.svelte';
  import MemoryFeedModal from '$components/memories/MemoryFeedModal.svelte';
  import MemoryFilterBar from '$components/memories/MemoryFilterBar.svelte';
  import PerspectiveSwitcher from '$components/memories/PerspectiveSwitcher.svelte';
  import FilePicker from '$components/FilePicker.svelte';

  import Images from 'lucide-svelte/icons/images';
  import Calendar from 'lucide-svelte/icons/calendar';
  import FolderOpen from 'lucide-svelte/icons/folder-open';
  import AlertTriangle from 'lucide-svelte/icons/triangle-alert';
  import RefreshCw from 'lucide-svelte/icons/refresh-cw';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import Loader2 from 'lucide-svelte/icons/loader-circle';
  import { isDemoExplicitlyRequested } from '$lib/devMode';

  let selectedPathInput = $lastScannedArchivePath || '';

  onMount(() => {
    if (isDemoExplicitlyRequested()) {
      return;
    }
    // Auto-load if we have an archive path and haven't loaded yet
    if ($lastScannedArchivePath && !$explorerData && !$isLoadingMemories) {
      loadMemories($lastScannedArchivePath);
    }
  });

  async function handleLoadArchive(path: string) {
    if (!path) return;
    selectedPathInput = path;
    lastScannedArchivePath.set(path);
    await loadMemories(path);
  }
</script>

<div class="memories-view-container">
  <!-- Sticky Top Header & Filter Controls (Remains visible and accessible on scroll) -->
  <div class="memories-sticky-header">
    <div class="view-header-bar">
      <div class="header-left">
        <div class="view-title-pill">
          <Sparkles size={16} class="text-sky-400" />
          <span class="title-text">Memories Explorer</span>
        </div>

        <!-- Segmented View Tabs (Memories | Calendar) -->
        <div class="segmented-view-picker">
          <button
            type="button"
            class="segment-btn"
            class:active={$activeExplorerView === 'grid'}
            on:click={() => activeExplorerView.set('grid')}
          >
            <Images size={14} />
            <span>Memories</span>
          </button>

          <button
            type="button"
            class="segment-btn"
            class:active={$activeExplorerView === 'calendar'}
            on:click={() => activeExplorerView.set('calendar')}
          >
            <Calendar size={14} />
            <span>Calendar</span>
          </button>
        </div>
      </div>

      <div class="header-right">
        {#if $explorerData}
          <button
            type="button"
            class="archive-info-badge"
            on:click={() => handleLoadArchive($lastScannedArchivePath)}
            title="Reload Archive ({$lastScannedArchivePath})"
          >
            <FolderOpen size={12} class="text-muted" />
            <span class="archive-name-text">{$lastScannedArchivePath.split('\\').pop()?.split('/').pop() || 'Archive'}</span>
            <RefreshCw size={11} class="reload-icon" />
          </button>
        {/if}
      </div>
    </div>

    <!-- Search & Filters Bar (Sticky with header) -->
    {#if $explorerData && !$isLoadingMemories}
      <MemoryFilterBar />
    {/if}
  </div>

  <!-- Loading State with Progress Bar -->
  {#if $isLoadingMemories}
    <div class="loading-state-card card">
      <div class="loading-content-box">
        <div class="loading-spinner-row">
          <Loader2 size={32} class="animate-spin text-sky-400" />
          <div class="loading-text-group">
            <h3 class="loading-title">Loading Your BeReal Memories...</h3>
            <p class="loading-subtitle">{$memoriesLoadProgress.stage}</p>
          </div>
          <span class="loading-pct-badge font-mono">{$memoriesLoadProgress.percentage}%</span>
        </div>

        <div class="loading-track">
          <div
            class="loading-bar-fill"
            style="width: {$memoriesLoadProgress.percentage}%;"
          ></div>
        </div>
      </div>
    </div>

  <!-- Error / Missing Archive Card -->
  {:else if $memoriesLoadError || !$explorerData}
    <div class="connect-archive-card card">
      <div class="connect-head">
        <div class="connect-icon-wrap">
          <Images size={28} class="text-sky-400" />
        </div>
        <div class="connect-text">
          <h2 class="title-md">Connect Your BeReal Archive</h2>
          <p class="text-secondary text-sm">
            Select your unextracted BeReal export <code>.zip</code> or photos folder to explore your memories grid and calendar.
          </p>
        </div>
      </div>

      {#if $memoriesLoadError}
        <div class="error-banner">
          <AlertTriangle size={16} class="text-rose-400" />
          <span>{$memoriesLoadError}</span>
        </div>
      {/if}

      <div class="picker-section">
        <FilePicker
          label="Select BeReal ZIP or Folder"
          placeholder="Choose ZIP file or extracted folder..."
          isDirectory={false}
          allowBoth={true}
          fileExtensions={['zip']}
          bind:value={selectedPathInput}
        />
        {#if selectedPathInput && selectedPathInput !== $lastScannedArchivePath}
          <button
            type="button"
            class="btn btn-accent-blue btn-md mt-3"
            on:click={() => handleLoadArchive(selectedPathInput)}
          >
            Load Selected Archive
          </button>
        {/if}
      </div>
    </div>

  <!-- Main Explorer View (Both Grid and Calendar retained in DOM for 0ms instant tab switching) -->
  {:else}
    <div class="explorer-content-layout">
      <!-- Active Grid or Calendar View -->
      <div class="active-view-frame">
        <div class="explorer-view-stage" class:is-active={$activeExplorerView === 'grid'} aria-hidden={$activeExplorerView !== 'grid'}>
          <MemoriesGrid />
        </div>
        <div class="explorer-view-stage" class:is-active={$activeExplorerView === 'calendar'} aria-hidden={$activeExplorerView !== 'calendar'}>
          <CalendarGrid />
        </div>
      </div>

      <!-- Floating Bottom-Left Perspective Toggle Pill -->
      <PerspectiveSwitcher variant="floating-window" />
    </div>
  {/if}

  <!-- Fullscreen / Modal Feed View (when memory clicked) -->
  <MemoryFeedModal />
</div>

<style>
  .memories-view-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    max-width: 1400px;
    margin: 0 auto;
    overflow: hidden;
    animation: viewFade 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .explorer-content-layout {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  .active-view-frame {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    position: relative;
    overflow: hidden;
  }

  .explorer-view-stage {
    flex: 1;
    min-height: 0;
    display: none;
    flex-direction: column;
    position: relative;
    height: 100%;
    overflow: hidden;
  }

  .explorer-view-stage.is-active {
    display: flex;
  }

  @keyframes viewFade {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .memories-sticky-header {
    position: sticky;
    top: -18px;
    z-index: 60;
    background: rgba(9, 9, 12, 0.94);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    padding: 10px 0 12px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.07);
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 4px;
  }

  .view-header-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }

  .view-title-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .title-text {
    font-size: 20px;
    font-weight: 800;
    color: #ffffff;
    letter-spacing: -0.02em;
  }

  /* Segmented Pill Tabs (Memories | Calendar) */
  .segmented-view-picker {
    display: inline-flex;
    align-items: center;
    background: #101016;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-full);
    padding: 3px;
    gap: 2px;
  }

  .segment-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    background: transparent;
    border: none;
    border-radius: var(--radius-full);
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .segment-btn:hover {
    color: #ffffff;
  }

  .segment-btn.active {
    background: #ffffff;
    color: #000000;
    font-weight: 700;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .archive-info-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    background: #14141d;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    font-size: 11.5px;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .archive-info-badge:hover {
    color: #ffffff;
    border-color: var(--border-medium);
    background: #1a1a24;
  }

  :global(.reload-icon) {
    color: var(--text-muted);
  }

  .explorer-content-layout {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 16px;
    width: 100%;
  }

  .active-view-frame {
    width: 100%;
  }

  .explorer-view-stage {
    width: 100%;
    display: none;
  }

  .explorer-view-stage.is-active {
    display: block;
  }

  /* Connect Archive Card */
  .connect-archive-card {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 32px;
    background: #111116;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
  }

  .connect-head {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .connect-icon-wrap {
    width: 52px;
    height: 52px;
    border-radius: var(--radius-md);
    background: rgba(56, 189, 248, 0.1);
    border: 1px solid rgba(56, 189, 248, 0.25);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .connect-text {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    background: rgba(244, 63, 94, 0.12);
    border: 1px solid rgba(244, 63, 94, 0.3);
    border-radius: var(--radius-md);
    color: #fda4af;
    font-size: 12px;
  }

  .loading-state-card {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 80px 20px;
    background: #111116;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
  }

  .loading-content-box {
    display: flex;
    flex-direction: column;
    gap: 18px;
    width: 100%;
    max-width: 520px;
  }

  .loading-spinner-row {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .loading-text-group {
    display: flex;
    flex-direction: column;
    gap: 3px;
    flex: 1;
  }

  .loading-title {
    font-size: 16px;
    font-weight: 700;
    color: #ffffff;
    margin: 0;
  }

  .loading-subtitle {
    font-size: 12.5px;
    color: var(--text-secondary);
    margin: 0;
  }

  .loading-pct-badge {
    font-size: 13px;
    font-weight: 800;
    color: #38bdf8;
    background: rgba(56, 189, 248, 0.12);
    border: 1px solid rgba(56, 189, 248, 0.3);
    padding: 4px 10px;
    border-radius: var(--radius-full);
  }

  .loading-track {
    width: 100%;
    height: 7px;
    background: #09090e;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }

  .loading-bar-fill {
    height: 100%;
    background: linear-gradient(90deg, #0284c7, #38bdf8);
    border-radius: 999px;
    box-shadow: 0 0 14px rgba(56, 189, 248, 0.6);
    transition: width 0.22s ease-out;
  }
</style>
