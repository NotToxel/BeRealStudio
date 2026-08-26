<script lang="ts">
  import type { ExplorerMemory } from '$lib/types';
  import { openExportModal } from '$lib/memoriesStore';
  import { exportSingleMemory, revealInFolder, isTauri } from '$lib/tauri';
  import { save } from '@tauri-apps/plugin-dialog';
  import MoreHorizontal from 'lucide-svelte/icons/ellipsis';
  import FolderOpen from 'lucide-svelte/icons/folder-open';
  import Download from 'lucide-svelte/icons/download';
  import Camera from 'lucide-svelte/icons/camera';
  import User from 'lucide-svelte/icons/circle-user';
  import Film from 'lucide-svelte/icons/film';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import Copy from 'lucide-svelte/icons/copy';
  import Check from 'lucide-svelte/icons/check';
  import Layers from 'lucide-svelte/icons/layers';
  import SlidersHorizontal from 'lucide-svelte/icons/sliders-horizontal';

  import Share2 from 'lucide-svelte/icons/share-2';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import FileText from 'lucide-svelte/icons/file-text';
  import Eye from 'lucide-svelte/icons/eye';

  export let memory: ExplorerMemory;

  let isOpen = false;
  let isExporting = false;
  let copiedText = '';

  function toggleMenu(e: MouseEvent) {
    e.stopPropagation();
    isOpen = !isOpen;
  }

  function closeMenu() {
    isOpen = false;
  }

  function handleOpenExportDialog() {
    closeMenu();
    openExportModal(memory);
  }

  async function handleOpenExplorer() {
    closeMenu();
    if (memory.primaryPath) {
      await revealInFolder(memory.primaryPath);
    }
  }

  function getFilenameFromPath(filePath?: string): string {
    if (!filePath) return `${memory.takenAt.slice(0, 10)}_bereal.jpg`;
    return filePath.split(/[/\\]/).pop() || `${memory.takenAt.slice(0, 10)}_bereal.jpg`;
  }

  async function handleQuickDownload() {
    closeMenu();
    await handleExport('combined_pip');
  }

  async function handleExport(exportType: 'combined_pip' | 'combined_sidebyside' | 'primary_only' | 'secondary_only' | 'bts_only' | 'motion_photo') {
    closeMenu();
    if (!memory.primaryPath) return;

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
        quality: 95,
        embedExif: true,
        takenAt: memory.takenAt,
        latitude: memory.location?.latitude,
        longitude: memory.location?.longitude,
        caption: memory.caption,
      });

      isExporting = false;
    } catch (err) {
      console.error('Failed to export single memory:', err);
      isExporting = false;
    }
  }

  async function copyToClipboard(text: string, label: string) {
    closeMenu();
    try {
      await navigator.clipboard.writeText(text);
      copiedText = label;
      setTimeout(() => {
        copiedText = '';
      }, 2000);
    } catch (err) {
      console.error('Copy failed:', err);
    }
  }
</script>

<svelte:window on:click={closeMenu} />

<div class="action-menu-wrap">
  <button
    type="button"
    class="menu-trigger-btn"
    class:active={isOpen}
    on:click={toggleMenu}
    title="Memory options"
    aria-label="Memory options"
  >
    <MoreHorizontal size={18} />
  </button>

  {#if isOpen}
    <div class="menu-popover" on:click|stopPropagation on:keydown|stopPropagation role="menu" tabindex="-1">
      <!-- Official BeReal Primary Actions -->
      <div class="menu-section">
        <button type="button" class="menu-item primary-action" on:click={handleOpenExportDialog}>
          <Share2 size={16} class="text-sky-400" />
          <span>Share BeReal.</span>
        </button>

        <button type="button" class="menu-item" on:click={handleQuickDownload}>
          <Download size={16} class="text-emerald-400" />
          <span>Download</span>
        </button>

        <button type="button" class="menu-item" on:click={() => copyToClipboard(getFilenameFromPath(memory.primaryPath), 'Filename')}>
          <Copy size={16} class="text-indigo-400" />
          <span>Copy Filename</span>
        </button>

        {#if memory.primaryPath}
          <button type="button" class="menu-item" on:click={handleOpenExplorer}>
            <FolderOpen size={16} class="text-amber-400" />
            <span>Show in Explorer</span>
          </button>
        {/if}
      </div>

      <div class="menu-divider"></div>

      <!-- Secondary Camera & Media Exports -->
      <div class="menu-section">
        <span class="menu-header">Perspectives &amp; Motion</span>

        {#if memory.secondaryPath}
          <button type="button" class="menu-item" on:click={() => handleExport('combined_sidebyside')}>
            <Layers size={15} class="text-purple-400" />
            <span>Save Side-by-Side</span>
          </button>

          <button type="button" class="menu-item" on:click={() => handleExport('secondary_only')}>
            <User size={15} class="text-cyan-400" />
            <span>Save Front Camera</span>
          </button>
        {/if}

        <button type="button" class="menu-item" on:click={() => handleExport('primary_only')}>
          <Camera size={15} class="text-emerald-400" />
          <span>Save Main Camera</span>
        </button>

        {#if memory.btsPath}
          <button type="button" class="menu-item" on:click={() => handleExport('bts_only')}>
            <Film size={15} class="text-amber-400" />
            <span>Save BTS Video (.mp4)</span>
          </button>

          <button type="button" class="menu-item" on:click={() => handleExport('motion_photo')}>
            <Sparkles size={15} class="text-emerald-400" />
            <span>Save Motion Photo (Live)</span>
          </button>
        {/if}
      </div>

      <div class="menu-divider"></div>

      <!-- Copy Details & Coordinates -->
      <div class="menu-section">
        <span class="menu-header">Copy Info</span>

        {#if memory.caption}
          <button type="button" class="menu-item" on:click={() => copyToClipboard(memory.caption || '', 'Caption')}>
            <FileText size={15} class="text-yellow-400" />
            <span>Copy Caption</span>
          </button>
        {/if}

        {#if memory.locationName || memory.location}
          <button
            type="button"
            class="menu-item"
            on:click={() =>
              copyToClipboard(
                memory.locationName || `${memory.location?.latitude.toFixed(6)}, ${memory.location?.longitude.toFixed(6)}`,
                'Location'
              )}
          >
            <MapPin size={15} class="text-rose-400" />
            <span>Copy Location</span>
          </button>
        {/if}
      </div>
    </div>
  {/if}

  {#if copiedText}
    <div class="toast-indicator">
      <Check size={12} />
      <span>{copiedText} copied!</span>
    </div>
  {/if}
</div>

<style>
  .action-menu-wrap {
    position: relative;
    display: inline-block;
  }

  .menu-trigger-btn {
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

  .menu-trigger-btn:hover,
  .menu-trigger-btn.active {
    background: rgba(255, 255, 255, 0.12);
    color: #ffffff;
  }

  .menu-popover {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    width: 220px;
    background: #14141c;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.8);
    padding: 6px;
    z-index: 100;
    animation: popoverIn 0.14s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes popoverIn {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .menu-section {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .menu-header {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    padding: 4px 8px;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 8px;
    border-radius: var(--radius-sm);
    background: transparent;
    border: none;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-main);
    text-align: left;
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
    width: 100%;
  }

  .menu-item:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
  }

  .menu-divider {
    height: 1px;
    background: var(--border-subtle);
    margin: 4px 0;
  }

  .toast-indicator {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    z-index: 250;
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 12px;
    background: #059669;
    color: #ffffff;
    border-radius: var(--radius-full);
    font-size: 11.5px;
    font-weight: 700;
    white-space: nowrap;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.7);
    animation: popoverIn 0.15s ease-out;
  }
</style>
