<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount, onDestroy } from 'svelte';

  const appWindow = getCurrentWindow();
  let isMaximized = false;
  let unlistenResize: (() => void) | undefined;

  async function checkMaximized() {
    try {
      isMaximized = await appWindow.isMaximized();
    } catch {
      // Fallback
    }
  }

  async function minimize() {
    try {
      await appWindow.minimize();
    } catch (e) {
      console.error('Minimize error:', e);
    }
  }

  async function toggleMaximize() {
    try {
      const max = await appWindow.isMaximized();
      if (max) {
        await appWindow.unmaximize();
        isMaximized = false;
      } else {
        await appWindow.maximize();
        isMaximized = true;
      }
    } catch (e) {
      console.error('Toggle maximize error:', e);
      try {
        await appWindow.toggleMaximize();
        isMaximized = await appWindow.isMaximized();
      } catch (err2) {
        console.error('Fallback toggle maximize error:', err2);
      }
    }
  }

  async function toggleFullscreen() {
    try {
      const fs = await appWindow.isFullscreen();
      await appWindow.setFullscreen(!fs);
    } catch (e) {
      console.error('Fullscreen error:', e);
    }
  }

  async function close() {
    try {
      await appWindow.close();
    } catch (e) {
      console.error('Close error:', e);
    }
  }

  function handleMouseDown(e: MouseEvent) {
    // Only drag on primary left click and if not clicking a button
    if (e.button === 0 && !(e.target as HTMLElement)?.closest('button')) {
      try {
        appWindow.startDragging();
      } catch (err) {
        console.error('Drag error:', err);
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'F11') {
      e.preventDefault();
      toggleFullscreen();
    }
  }

  onMount(async () => {
    await checkMaximized();
    try {
      unlistenResize = await appWindow.onResized(() => {
        checkMaximized();
      });
    } catch {
      // Ignore if not supported in dev
    }
  });

  onDestroy(() => {
    if (unlistenResize) unlistenResize();
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- The custom titlebar with direct startDragging + data-tauri-drag-region + double click to maximize -->
<div
  class="custom-titlebar"
  role="toolbar"
  tabindex="-1"
  data-tauri-drag-region
  on:mousedown={handleMouseDown}
  on:dblclick={toggleMaximize}
>
  <!-- Left Brand Group -->
  <div class="brand-zone" data-tauri-drag-region>
    <svg class="brand-glyph" width="16" height="16" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect x="2" y="8" width="18" height="16" rx="4" fill="rgba(255,230,0,0.18)" stroke="rgba(255,230,0,0.7)" stroke-width="1.5"/>
      <rect x="12" y="4" width="18" height="16" rx="4" fill="rgba(139,92,246,0.18)" stroke="rgba(139,92,246,0.7)" stroke-width="1.5"/>
      <circle cx="11" cy="16" r="3.5" fill="rgba(255,230,0,0.3)" stroke="rgba(255,230,0,0.9)" stroke-width="1"/>
      <circle cx="21" cy="12" r="3.5" fill="rgba(139,92,246,0.3)" stroke="rgba(139,92,246,0.9)" stroke-width="1"/>
    </svg>
    <span class="app-title-text" data-tauri-drag-region>BeReal Studio</span>
  </div>

  <!-- Draggable Center Filler -->
  <div class="drag-spacer" data-tauri-drag-region></div>

  <!-- Right Window Action Controls -->
  <div class="window-actions">
    <button
      type="button"
      class="win-btn win-min"
      on:click|stopPropagation={minimize}
      title="Minimize"
      aria-label="Minimize"
    >
      <svg width="10" height="1" viewBox="0 0 10 1" fill="currentColor">
        <rect width="10" height="1"/>
      </svg>
    </button>

    <button
      type="button"
      class="win-btn win-max"
      on:click|stopPropagation={toggleMaximize}
      title={isMaximized ? 'Restore' : 'Maximize'}
      aria-label={isMaximized ? 'Restore' : 'Maximize'}
    >
      {#if isMaximized}
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1">
          <rect x="2.5" y="0.5" width="7" height="7"/>
          <polyline points="0.5,2.5 0.5,9.5 7.5,9.5"/>
        </svg>
      {:else}
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1">
          <rect x="0.5" y="0.5" width="9" height="9"/>
        </svg>
      {/if}
    </button>

    <button
      type="button"
      class="win-btn win-close"
      on:click|stopPropagation={close}
      title="Close"
      aria-label="Close"
    >
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2">
        <line x1="0" y1="0" x2="10" y2="10"/>
        <line x1="10" y1="0" x2="0" y2="10"/>
      </svg>
    </button>
  </div>
</div>

<style>
  .custom-titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 34px;
    background: #08080b;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    user-select: none;
    -webkit-user-select: none;
    flex-shrink: 0;
    z-index: 1000;
    cursor: default;
  }

  .brand-zone {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-left: 12px;
    cursor: default;
  }

  .brand-glyph {
    flex-shrink: 0;
  }

  .app-title-text {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .drag-spacer {
    flex: 1;
    height: 100%;
    cursor: default;
  }

  .window-actions {
    display: flex;
    align-items: center;
    height: 100%;
    flex-shrink: 0;
  }

  .win-btn {
    width: 44px;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: #94a3b8;
    cursor: pointer;
    transition: background 100ms ease, color 100ms ease;
    outline: none;
    border-radius: 0;
  }

  .win-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
  }

  .win-close:hover {
    background: #e11d48 !important;
    color: #ffffff !important;
  }

  .win-btn:active {
    opacity: 0.8;
  }
</style>
