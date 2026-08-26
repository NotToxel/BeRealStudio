<script lang="ts">
  import { fade } from 'svelte/transition';
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
  import Clock from 'lucide-svelte/icons/clock';
  import Share2 from 'lucide-svelte/icons/share-2';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import FileText from 'lucide-svelte/icons/file-text';
  import Eye from 'lucide-svelte/icons/eye';

  export let memory: ExplorerMemory;

  let isOpen = false;
  let isExporting = false;
  let copiedInfo: { label: string; text: string } | null = null;

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

  async function handleReveal(filePath?: string) {
    closeMenu();
    if (filePath) {
      await revealInFolder(filePath);
    }
  }

  function getFilenameFromPath(filePath?: string): string {
    if (!filePath) return `${memory.takenAt.slice(0, 10)}_bereal.jpg`;
    return filePath.split(/[/\\]/).pop() || `${memory.takenAt.slice(0, 10)}_bereal.jpg`;
  }

  function getFormattedOutputFilename(mem: ExplorerMemory, exportType: string = 'combined_pip', ext: string = 'jpg'): string {
    let timeTag = mem.takenAt
      ? mem.takenAt.replace(/[:]/g, '-').replace(/\.\d+Z?$/, '').replace(/Z$/, '')
      : `${mem.year}-${String(mem.month).padStart(2, '0')}-${String(mem.day).padStart(2, '0')}`;

    if (!timeTag.includes('T')) {
      timeTag = `${timeTag}T${mem.timeFormatted ? mem.timeFormatted.replace(':', '-') : '12-00-00'}`;
    }

    const suffix = exportType === 'combined_pip' || exportType === 'combined'
      ? '_combined'
      : exportType === 'combined_sidebyside'
      ? '_combined_sidebyside'
      : exportType === 'primary_only'
      ? '_primary'
      : exportType === 'secondary_only'
      ? '_secondary'
      : exportType === 'bts_only'
      ? '_bts'
      : exportType === 'motion_photo'
      ? '_motion'
      : `_${exportType}`;

    const extension = exportType === 'bts_only' ? 'mp4' : ext;
    return `${timeTag}${suffix}.${extension}`;
  }

  async function handleQuickDownload() {
    closeMenu();
    await handleExport('combined_pip');
  }

  async function handleExport(exportType: 'combined_pip' | 'combined_sidebyside' | 'primary_only' | 'secondary_only' | 'bts_only' | 'motion_photo' | 'apple_live_photo') {
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

  async function performWriteToClipboard(text: string): Promise<boolean> {
    try {
      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(text);
        return true;
      }
    } catch (e) {
      console.warn('Standard clipboard write failed, trying fallback:', e);
    }

    try {
      const textarea = document.createElement('textarea');
      textarea.value = text;
      textarea.style.position = 'fixed';
      textarea.style.left = '-999999px';
      textarea.style.top = '-999999px';
      document.body.appendChild(textarea);
      textarea.focus();
      textarea.select();
      const success = document.execCommand('copy');
      document.body.removeChild(textarea);
      return success;
    } catch (err) {
      console.error('Fallback clipboard copy failed:', err);
      return false;
    }
  }

  async function copyToClipboard(text: string, label: string) {
    closeMenu();
    if (!text) return;
    const success = await performWriteToClipboard(text);
    if (success) {
      copiedInfo = { label, text };
      setTimeout(() => {
        copiedInfo = null;
      }, 2500);
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

        <!-- Reveal in Folder (Inline Camera & Video File Selector) -->
        {#if memory.primaryPath || memory.secondaryPath || memory.btsPath}
          <div class="inline-reveal-row">
            <span class="inline-reveal-label">
              <FolderOpen size={13} class="text-amber-400" />
              <span>Reveal in Folder:</span>
            </span>
            <div class="inline-reveal-buttons">
              {#if memory.primaryPath}
                <button
                  type="button"
                  class="inline-pill-btn"
                  on:click={() => handleReveal(memory.primaryPath)}
                  title="Reveal Main Camera photo in file explorer"
                >
                  <Camera size={11} class="text-teal-400" />
                  <span>Main</span>
                </button>
              {/if}

              {#if memory.secondaryPath}
                <button
                  type="button"
                  class="inline-pill-btn"
                  on:click={() => handleReveal(memory.secondaryPath)}
                  title="Reveal Selfie Camera photo in file explorer"
                >
                  <User size={11} class="text-rose-400" />
                  <span>Selfie</span>
                </button>
              {/if}

              {#if memory.btsPath}
                <button
                  type="button"
                  class="inline-pill-btn"
                  on:click={() => handleReveal(memory.btsPath)}
                  title="Reveal BTS Video in file explorer"
                >
                  <Film size={11} class="text-amber-400" />
                  <span>BTS</span>
                </button>
              {/if}
            </div>
          </div>
        {/if}
      </div>

      <div class="menu-divider"></div>

      <!-- Quick Export Variations -->
      <div class="menu-section">
        <span class="menu-header">Quick Save</span>

        <button type="button" class="menu-item" on:click={() => handleExport('combined_pip')}>
          <Layers size={14} class="text-blue-400" />
          <span>Save Picture-in-Picture</span>
        </button>

        {#if memory.secondaryPath}
          <button type="button" class="menu-item" on:click={() => handleExport('combined_sidebyside')}>
            <SlidersHorizontal size={14} class="text-purple-400" />
            <span>Save Side-by-Side</span>
          </button>
        {/if}

        <button type="button" class="menu-item" on:click={() => handleExport('primary_only')}>
          <Camera size={14} class="text-teal-400" />
          <span>Save Main Camera</span>
        </button>

        {#if memory.secondaryPath}
          <button type="button" class="menu-item" on:click={() => handleExport('secondary_only')}>
            <User size={14} class="text-rose-400" />
            <span>Save Selfie Camera</span>
          </button>
        {/if}

        {#if memory.btsPath}
          <button type="button" class="menu-item" on:click={() => handleExport('bts_only')}>
            <Film size={14} class="text-amber-400" />
            <span>Save BTS Video (.mp4)</span>
          </button>
          <button type="button" class="menu-item" on:click={() => handleExport('motion_photo')}>
            <Sparkles size={14} class="text-emerald-400" />
            <span>Save Motion Photo (Android)</span>
          </button>
          <button type="button" class="menu-item" on:click={() => handleExport('apple_live_photo')}>
            <Sparkles size={14} class="text-sky-400" />
            <span>Save Apple Live Photo (iOS)</span>
          </button>
        {/if}
      </div>

      <div class="menu-divider"></div>

      <!-- Compact Copy Info Grid -->
      <div class="menu-section">
        <span class="menu-header">Copy</span>

        <div class="copy-compact-grid">
          <button
            type="button"
            class="copy-grid-btn"
            on:click={() => copyToClipboard(`${memory.dateFormatted} • ${memory.timeFormatted}`, 'Date & Time')}
            title="Copy formatted date and time"
          >
            <Clock size={12} class="text-indigo-400" />
            <span>Date</span>
          </button>

          {#if memory.primaryPath}
            <button
              type="button"
              class="copy-grid-btn"
              on:click={() => copyToClipboard(memory.primaryPath || '', 'File Path')}
              title="Copy absolute file path"
            >
              <FolderOpen size={12} class="text-slate-400" />
              <span>Path</span>
            </button>
          {/if}

          {#if memory.locationName || memory.location}
            <button
              type="button"
              class="copy-grid-btn"
              on:click={() =>
                copyToClipboard(
                  memory.locationName || `${memory.location?.latitude.toFixed(6)}, ${memory.location?.longitude.toFixed(6)}`,
                  'Location'
                )}
              title="Copy location / GPS coordinates"
            >
              <MapPin size={12} class="text-rose-400" />
              <span>Location</span>
            </button>
          {/if}

          {#if memory.caption}
            <button
              type="button"
              class="copy-grid-btn"
              on:click={() => copyToClipboard(memory.caption || '', 'Caption')}
              title="Copy caption text"
            >
              <FileText size={12} class="text-yellow-400" />
              <span>Caption</span>
            </button>
          {/if}

          <button
            type="button"
            class="copy-grid-btn"
            on:click={() => copyToClipboard(getFormattedOutputFilename(memory), 'Filename')}
            title="Copy standard output filename"
          >
            <FileText size={12} class="text-emerald-400" />
            <span>Filename</span>
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Live Toast Indicator with Copy Preview -->
  {#if copiedInfo}
    <div class="toast-preview-card" transition:fade={{ duration: 150 }}>
      <div class="toast-preview-header">
        <Check size={13} class="text-emerald-400" />
        <span class="toast-preview-title">Copied {copiedInfo.label}</span>
      </div>
      <div class="toast-preview-snippet" title={copiedInfo.text}>
        <code>{copiedInfo.text.length > 36 ? copiedInfo.text.slice(0, 36) + '...' : copiedInfo.text}</code>
      </div>
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

  /* Compact Copy Grid (2-column pill buttons) */
  .copy-compact-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
    padding: 2px 4px 4px 4px;
  }

  .copy-grid-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 8px;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.07);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s ease;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .copy-grid-btn:hover {
    background: rgba(255, 255, 255, 0.09);
    border-color: rgba(255, 255, 255, 0.2);
    color: #ffffff;
    transform: translateY(-0.5px);
  }

  /* Inline Reveal Selection Row */
  .inline-reveal-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px 8px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: var(--radius-sm);
    margin: 2px 0;
  }

  .inline-reveal-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .inline-reveal-buttons {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-wrap: wrap;
  }

  .inline-pill-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: #1a1a26;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-sm);
    color: #ffffff;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .inline-pill-btn:hover {
    background: #28283a;
    border-color: rgba(255, 255, 255, 0.35);
    transform: translateY(-1px);
  }

  .toast-preview-card {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    z-index: 250;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 12px;
    background: rgba(12, 12, 18, 0.96);
    backdrop-filter: blur(16px);
    border: 1px solid rgba(16, 185, 129, 0.5);
    border-radius: var(--radius-md);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.85);
    white-space: nowrap;
    min-width: 170px;
    max-width: 280px;
    animation: popoverIn 0.15s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .toast-preview-header {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11.5px;
    font-weight: 700;
    color: #34d399;
  }

  .toast-preview-snippet {
    font-size: 10.5px;
    font-family: var(--font-mono);
    color: #cbd5e1;
    background: rgba(0, 0, 0, 0.45);
    padding: 3px 6px;
    border-radius: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
