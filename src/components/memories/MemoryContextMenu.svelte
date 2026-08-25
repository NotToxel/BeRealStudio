<script lang="ts">
  import { contextMenuState, closeContextMenu, openFeedAt, openExportModal } from '$lib/memoriesStore';
  import { exportSingleMemory, revealInFolder, isTauri } from '$lib/tauri';
  import { save } from '@tauri-apps/plugin-dialog';
  import FolderOpen from 'lucide-svelte/icons/folder-open';
  import Download from 'lucide-svelte/icons/download';
  import Camera from 'lucide-svelte/icons/camera';
  import User from 'lucide-svelte/icons/circle-user';
  import Film from 'lucide-svelte/icons/film';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import Copy from 'lucide-svelte/icons/copy';
  import Check from 'lucide-svelte/icons/check';
  import Layers from 'lucide-svelte/icons/layers';
  import Eye from 'lucide-svelte/icons/eye';
  import SlidersHorizontal from 'lucide-svelte/icons/sliders-horizontal';

  let copiedText = '';
  let isExporting = false;

  $: menuState = $contextMenuState;
  $: memory = menuState.memory;

  async function handleOpenExplorer() {
    closeContextMenu();
    if (memory?.primaryPath) {
      await revealInFolder(memory.primaryPath);
    }
  }

  function handleOpenFeed() {
    closeContextMenu();
    if (memory) {
      openFeedAt(memory);
    }
  }

  function handleOpenExportDialog() {
    closeContextMenu();
    if (memory) {
      openExportModal(memory);
    }
  }

  async function handleExport(exportType: 'combined_pip' | 'combined_sidebyside' | 'primary_only' | 'secondary_only' | 'bts_only' | 'motion_photo') {
    closeContextMenu();
    if (!memory || !memory.primaryPath) return;

    try {
      isExporting = true;
      const isVideo = exportType === 'bts_only';
      const ext = isVideo ? 'mp4' : 'jpg';
      const defaultFilename = `${memory.takenAt.slice(0, 10)}_${exportType}.${ext}`;
      const filters = isVideo
        ? [{ name: 'MP4 Video', extensions: ['mp4'] }]
        : [{ name: 'JPEG Image', extensions: ['jpg', 'jpeg'] }];

      const savePath = await save({
        defaultPath: defaultFilename,
        filters,
      });

      if (!savePath) {
        isExporting = false;
        return;
      }

      await exportSingleMemory({
        memoryIndex: memory.index,
        primaryPath: memory.primaryPath,
        secondaryPath: memory.secondaryPath,
        btsPath: memory.btsPath,
        outputPath: savePath,
        exportType,
        format: 'Jpeg',
        quality: 92,
        embedExif: true,
        takenAt: memory.takenAt,
        latitude: memory.location ? memory.location.latitude : undefined,
        longitude: memory.location ? memory.location.longitude : undefined,
        caption: memory.caption,
      });
    } catch (err) {
      console.error('Failed to export memory from context menu:', err);
    } finally {
      isExporting = false;
    }
  }

  function handleCopy(text: string, label: string) {
    if (!text) return;
    navigator.clipboard.writeText(text);
    copiedText = label;
    setTimeout(() => {
      copiedText = '';
      closeContextMenu();
    }, 800);
  }
</script>

<svelte:window
  on:click={closeContextMenu}
  on:keydown={(e) => e.key === 'Escape' && closeContextMenu()}
/>

{#if menuState.isOpen && memory}
  <!-- Floating Context Menu Overlay -->
  <div
    class="custom-context-menu"
    style="left: {menuState.x}px; top: {menuState.y}px;"
    role="menu"
    tabindex="-1"
    on:click|stopPropagation
    on:keydown={(e) => e.key === 'Escape' && closeContextMenu()}
  >
    <!-- Menu Header with Date -->
    <div class="menu-header">
      <span class="menu-header-date">{memory.dateFormatted}</span>
      <span class="menu-header-time">{memory.timeFormatted}</span>
    </div>

    <div class="menu-divider"></div>

    <!-- Quick View -->
    <button type="button" class="menu-item" on:click={handleOpenFeed}>
      <Eye size={13} class="menu-item-icon text-sky-400" />
      <span>View in Feed</span>
    </button>

    <!-- Reveal in File Explorer -->
    <button type="button" class="menu-item" on:click={handleOpenExplorer}>
      <FolderOpen size={13} class="menu-item-icon text-amber-400" />
      <span>Reveal in Explorer</span>
    </button>

    <!-- Custom Export Dialog -->
    <button type="button" class="menu-item" on:click={handleOpenExportDialog}>
      <SlidersHorizontal size={13} class="menu-item-icon text-sky-400" />
      <span>Export Options...</span>
    </button>

    <div class="menu-divider"></div>

    <!-- Export Group -->
    <div class="menu-section-label">QUICK EXPORT</div>

    <button
      type="button"
      class="menu-item"
      disabled={isExporting || !memory.secondaryPath}
      on:click={() => handleExport('combined_pip')}
    >
      <Layers size={13} class="menu-item-icon text-yellow-400" />
      <span>Save Picture-in-Picture</span>
    </button>

    <button
      type="button"
      class="menu-item"
      disabled={isExporting || !memory.secondaryPath}
      on:click={() => handleExport('combined_sidebyside')}
    >
      <Download size={13} class="menu-item-icon text-emerald-400" />
      <span>Save Side-by-Side</span>
    </button>

    <button
      type="button"
      class="menu-item"
      disabled={isExporting || !memory.primaryPath}
      on:click={() => handleExport('primary_only')}
    >
      <Camera size={13} class="menu-item-icon text-sky-400" />
      <span>Save Main Camera Only</span>
    </button>

    {#if memory.secondaryPath}
      <button
        type="button"
        class="menu-item"
        disabled={isExporting}
        on:click={() => handleExport('secondary_only')}
      >
        <User size={13} class="menu-item-icon text-violet-400" />
        <span>Save Selfie Camera Only</span>
      </button>
    {/if}

    {#if memory.btsPath}
      <button
        type="button"
        class="menu-item"
        disabled={isExporting}
        on:click={() => handleExport('bts_only')}
      >
        <Film size={13} class="menu-item-icon text-amber-400" />
        <span>Save BTS Video (.mp4)</span>
      </button>

      <button
        type="button"
        class="menu-item"
        disabled={isExporting}
        on:click={() => handleExport('motion_photo')}
      >
        <Sparkles size={13} class="menu-item-icon text-emerald-400" />
        <span>Save Motion Photo (Live)</span>
      </button>
    {/if}

    <div class="menu-divider"></div>

    <!-- Copy Group -->
    <div class="menu-section-label">COPY INFO</div>

    {#if memory.caption}
      <button
        type="button"
        class="menu-item"
        on:click={() => handleCopy(memory?.caption || '', 'caption')}
      >
        {#if copiedText === 'caption'}
          <Check size={13} class="menu-item-icon text-emerald-400" />
          <span class="text-emerald-400">Caption Copied!</span>
        {:else}
          <Copy size={13} class="menu-item-icon" />
          <span>Copy Caption</span>
        {/if}
      </button>
    {/if}

    <button
      type="button"
      class="menu-item"
      on:click={() => handleCopy(`${memory?.dateFormatted} ${memory?.timeFormatted}`, 'date')}
    >
      {#if copiedText === 'date'}
        <Check size={13} class="menu-item-icon text-emerald-400" />
        <span class="text-emerald-400">Date Copied!</span>
      {:else}
        <Copy size={13} class="menu-item-icon" />
        <span>Copy Date &amp; Time</span>
      {/if}
    </button>

    {#if memory.location}
      <button
        type="button"
        class="menu-item"
        on:click={() => handleCopy(`${memory?.location?.latitude}, ${memory?.location?.longitude}`, 'coords')}
      >
        {#if copiedText === 'coords'}
          <Check size={13} class="menu-item-icon text-emerald-400" />
          <span class="text-emerald-400">Coordinates Copied!</span>
        {:else}
          <Copy size={13} class="menu-item-icon" />
          <span>Copy GPS Coordinates</span>
        {/if}
      </button>
    {/if}
  </div>
{/if}

<style>
  .custom-context-menu {
    position: fixed;
    width: 224px;
    background: rgba(14, 14, 20, 0.95);
    backdrop-filter: blur(24px);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.85);
    padding: 6px;
    z-index: 1000;
    display: flex;
    flex-direction: column;
    gap: 2px;
    animation: menuFadeIn 0.14s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes menuFadeIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .menu-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px 4px 8px;
    font-size: 11px;
    font-weight: 700;
    color: #ffffff;
  }

  .menu-header-time {
    color: var(--text-muted);
    font-size: 10.5px;
    font-weight: 600;
  }

  .menu-section-label {
    font-size: 9.5px;
    font-weight: 800;
    color: var(--text-muted);
    letter-spacing: 0.08em;
    padding: 4px 8px 2px 8px;
  }

  .menu-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.07);
    margin: 3px 0;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 6.5px 8px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    text-align: left;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .menu-item:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
  }

  .menu-item:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
</style>
