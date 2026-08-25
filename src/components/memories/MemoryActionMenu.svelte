<script lang="ts">
  import type { ExplorerMemory } from '$lib/types';
  import { exportSingleMemory, openPath, isTauri } from '$lib/tauri';
  import { save } from '@tauri-apps/plugin-dialog';
  import MoreHorizontal from 'lucide-svelte/icons/ellipsis';
  import FolderOpen from 'lucide-svelte/icons/folder-open';
  import Download from 'lucide-svelte/icons/download';
  import Camera from 'lucide-svelte/icons/camera';
  import User from 'lucide-svelte/icons/circle-user';
  import Film from 'lucide-svelte/icons/film';
  import Copy from 'lucide-svelte/icons/copy';
  import Check from 'lucide-svelte/icons/check';
  import Layers from 'lucide-svelte/icons/layers';

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

  async function handleOpenExplorer() {
    closeMenu();
    if (memory.primaryPath) {
      await openPath(memory.primaryPath);
    }
  }

  async function handleExport(exportType: 'combined_pip' | 'combined_sidebyside' | 'primary_only' | 'secondary_only') {
    closeMenu();
    if (!memory.primaryPath) return;

    try {
      isExporting = true;
      const defaultFilename = `${memory.takenAt.slice(0, 10)}_${exportType}.jpg`;
      const savePath = await save({
        defaultPath: defaultFilename,
        filters: [{ name: 'JPEG Image', extensions: ['jpg', 'jpeg'] }],
      });

      if (!savePath) {
        isExporting = false;
        return;
      }

      await exportSingleMemory({
        memoryIndex: memory.index,
        primaryPath: memory.primaryPath,
        secondaryPath: memory.secondaryPath,
        outputPath: savePath,
        exportType,
        format: 'Jpeg',
        quality: 92,
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
    title="Memory actions & export options"
    aria-label="Memory options"
  >
    <MoreHorizontal size={18} />
  </button>

  {#if isOpen}
    <div class="menu-popover">
      <div class="menu-section">
        <span class="menu-header">Open & Export</span>

        {#if memory.primaryPath}
          <button type="button" class="menu-item" on:click={handleOpenExplorer}>
            <FolderOpen size={14} class="text-sky-400" />
            <span>Reveal in File Explorer</span>
          </button>
        {/if}

        <button type="button" class="menu-item" on:click={() => handleExport('combined_pip')}>
          <Layers size={14} class="text-amber-400" />
          <span>Save Combined Photo (PIP)</span>
        </button>

        {#if memory.secondaryPath}
          <button type="button" class="menu-item" on:click={() => handleExport('combined_sidebyside')}>
            <Download size={14} class="text-purple-400" />
            <span>Save Side-by-Side Photo</span>
          </button>

          <button type="button" class="menu-item" on:click={() => handleExport('secondary_only')}>
            <User size={14} class="text-cyan-400" />
            <span>Save Front / Selfie Camera</span>
          </button>
        {/if}

        <button type="button" class="menu-item" on:click={() => handleExport('primary_only')}>
          <Camera size={14} class="text-emerald-400" />
          <span>Save Main Camera</span>
        </button>
      </div>

      <div class="menu-divider"></div>

      <div class="menu-section">
        <span class="menu-header">Copy Details</span>

        {#if memory.caption}
          <button type="button" class="menu-item" on:click={() => copyToClipboard(memory.caption || '', 'Caption')}>
            <Copy size={14} />
            <span>Copy Caption</span>
          </button>
        {/if}

        <button type="button" class="menu-item" on:click={() => copyToClipboard(memory.takenAt, 'Date')}>
          <Copy size={14} />
          <span>Copy Timestamp ({memory.dateFormatted})</span>
        </button>

        {#if memory.location}
          <button
            type="button"
            class="menu-item"
            on:click={() =>
              copyToClipboard(
                `${memory.location?.latitude.toFixed(6)}, ${memory.location?.longitude.toFixed(6)}`,
                'GPS'
              )}
          >
            <Copy size={14} />
            <span>Copy GPS Coordinates</span>
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
    bottom: calc(100% + 8px);
    right: 0;
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    background: #059669;
    color: #ffffff;
    border-radius: var(--radius-full);
    font-size: 11px;
    font-weight: 600;
    white-space: nowrap;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    animation: popoverIn 0.15s ease-out;
  }
</style>
