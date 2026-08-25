<script lang="ts">
  import { onMount } from 'svelte';
  import { listSystemFonts, pickFile } from '$lib/tauri';
  import { BUILTIN_FONT_OPTIONS } from '$lib/fonts';
  import type { FontInfo } from '$lib/types';
  import FolderOpen from 'lucide-svelte/icons/folder-open';
  import X from 'lucide-svelte/icons/circle-x';
  import ChevronDown from 'lucide-svelte/icons/chevron-down';
  import Check from 'lucide-svelte/icons/check';
  import Search from 'lucide-svelte/icons/search';
  import Type from 'lucide-svelte/icons/type';

  export let selectedPath: string = 'inter';
  export let label: string = 'Font Family';

  let customFonts: FontInfo[] = [];
  let isCustomFile = false;
  let customFileName = '';
  let isOpen = false;
  let searchQuery = '';
  let pickerRef: HTMLElement;

  $: {
    const isBuiltin = BUILTIN_FONT_OPTIONS.some((b) => b.id === selectedPath);
    const isSystem = customFonts.some((f) => f.path === selectedPath);
    isCustomFile = Boolean(selectedPath) && !isBuiltin && !isSystem && (selectedPath.includes('/') || selectedPath.includes('\\'));
    if (isCustomFile) {
      customFileName = selectedPath.split(/[/\\]/).pop() || selectedPath;
    }
  }

  $: currentBuiltin = BUILTIN_FONT_OPTIONS.find((b) => b.id === selectedPath);
  $: currentSystem = customFonts.find((f) => f.path === selectedPath);

  $: filteredBuiltins = BUILTIN_FONT_OPTIONS.filter((f) =>
    f.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    f.category.toLowerCase().includes(searchQuery.toLowerCase())
  );

  $: filteredSystemFonts = customFonts.filter((f) =>
    f.family.toLowerCase().includes(searchQuery.toLowerCase())
  );

  async function handleBrowseCustomFont() {
    try {
      const file = await pickFile('Select Custom Font File', ['ttf', 'otf', 'woff2']);
      if (file) {
        selectedPath = file;
        isOpen = false;
      }
    } catch (e) {
      console.warn('Failed to pick custom font', e);
    }
  }

  function handleResetCustom() {
    selectedPath = 'inter';
  }

  function selectFont(idOrPath: string) {
    selectedPath = idOrPath;
    isOpen = false;
  }

  function handleOutsideClick(e: MouseEvent) {
    if (isOpen && pickerRef && !pickerRef.contains(e.target as Node)) {
      isOpen = false;
    }
  }

  onMount(async () => {
    try {
      const all = await listSystemFonts();
      customFonts = all.filter((f) => !BUILTIN_FONT_OPTIONS.some((b) => b.id === f.path));
    } catch {
      // ignore
    }
    if (!selectedPath) {
      selectedPath = 'inter';
    }
  });
</script>

<svelte:window on:click={handleOutsideClick} />

<div class="font-picker-wrapper" bind:this={pickerRef}>
  <div class="header">
    <label class="label" for="font-trigger">{label}</label>
  </div>

  <div class="controls-row">
    {#if isCustomFile}
      <div class="custom-pill">
        <div class="custom-title-row">
          <Type size={14} class="text-amber-400" />
          <span class="font-name font-mono">{customFileName}</span>
        </div>
        <button type="button" class="btn-clear" title="Remove Custom Font" on:click={handleResetCustom}>
          <X size={13} />
        </button>
      </div>
    {:else}
      <!-- Custom Trigger Box -->
      <button
        id="font-trigger"
        type="button"
        class="custom-font-trigger"
        class:is-open={isOpen}
        on:click|stopPropagation={() => (isOpen = !isOpen)}
      >
        <div class="trigger-left">
          <span class="font-preview-tag {currentBuiltin?.cssClass || ''}">
            {currentBuiltin ? currentBuiltin.name : currentSystem ? currentSystem.family : 'Select Font'}
          </span>
          {#if currentBuiltin}
            <span class="category-badge">{currentBuiltin.category}</span>
          {/if}
        </div>
        <ChevronDown size={14} class="text-muted transition-transform {isOpen ? 'rotate-180' : ''}" />
      </button>
    {/if}

    <button
      type="button"
      class="btn btn-secondary btn-browse-font"
      title="Load Custom .TTF or .OTF Font File"
      on:click={handleBrowseCustomFont}
    >
      <FolderOpen size={14} class="text-amber-400" />
      <span>Load File...</span>
    </button>
  </div>

  <!-- Custom Popover Dropdown Menu -->
  {#if isOpen}
    <div
      class="custom-font-menu card"
      role="dialog"
      aria-label="Font Selection Menu"
      tabindex="-1"
      on:click|stopPropagation
      on:keydown={(e) => e.key === 'Escape' && (isOpen = false)}
    >
      <div class="menu-search-box">
        <Search size={13} class="text-muted" />
        <input
          type="text"
          class="menu-search-input"
          placeholder="Search fonts..."
          bind:value={searchQuery}
        />
        {#if searchQuery}
          <button type="button" class="clear-search-btn" on:click={() => (searchQuery = '')}>
            <X size={11} />
          </button>
        {/if}
      </div>

      <div class="menu-scroll-area">
        <!-- Curated Built-in Fonts -->
        <div class="font-group">
          <span class="group-title">Curated Built-in Fonts</span>
          <div class="font-options-list">
            {#each filteredBuiltins as font}
              <button
                type="button"
                class="font-option-item"
                class:active={selectedPath === font.id}
                on:click={() => selectFont(font.id)}
              >
                <div class="font-item-left">
                  <span class="font-item-name {font.cssClass}">{font.name}</span>
                </div>

                <div class="font-item-right">
                  <span class="badge badge-subtle">{font.category}</span>
                  {#if selectedPath === font.id}
                    <Check size={13} class="text-amber-400" />
                  {/if}
                </div>
              </button>
            {/each}
          </div>
        </div>

        <!-- System Fonts (if available) -->
        {#if filteredSystemFonts.length > 0}
          <div class="font-group">
            <span class="group-title">System Fonts ({filteredSystemFonts.length})</span>
            <div class="font-options-list">
              {#each filteredSystemFonts.slice(0, 40) as font}
                <button
                  type="button"
                  class="font-option-item"
                  class:active={selectedPath === font.path}
                  on:click={() => selectFont(font.path)}
                >
                  <div class="font-item-left">
                    <span class="font-item-name" style="font-family: '{font.family}', sans-serif;">{font.family}</span>
                  </div>
                  {#if selectedPath === font.path}
                    <Check size={13} class="text-amber-400" />
                  {/if}
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .font-picker-wrapper {
    display: flex;
    flex-direction: column;
    gap: 5px;
    width: 100%;
    position: relative;
  }

  .header {
    display: flex;
    align-items: center;
  }

  .label {
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-main);
  }

  .controls-row {
    display: flex;
    gap: 8px;
    align-items: center;
    width: 100%;
  }

  .custom-font-trigger {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: #0e0e13;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 5px 10px;
    cursor: pointer;
    min-height: 32px;
    transition: all var(--transition-fast);
  }

  .custom-font-trigger:hover {
    border-color: rgba(255, 255, 255, 0.25);
    background: #14141c;
  }

  .custom-font-trigger.is-open {
    border-color: #8b5cf6;
    box-shadow: 0 0 10px rgba(139, 92, 246, 0.2);
  }

  .trigger-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .font-preview-tag {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .category-badge {
    font-size: 9.5px;
    font-weight: 600;
    padding: 1px 5px;
    border-radius: 4px;
    background: #1e1e28;
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
  }

  .btn-browse-font {
    white-space: nowrap;
    padding: 5px 10px;
    font-size: 12px;
    min-height: 32px;
    flex-shrink: 0;
  }

  .custom-pill {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: #14141a;
    border: 1px solid rgba(245, 158, 11, 0.4);
    border-radius: var(--radius-md);
    padding: 5px 10px;
    min-height: 32px;
  }

  .custom-title-row {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .font-name {
    font-size: 12px;
    color: var(--text-main);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .btn-clear {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    padding: 2px;
    border-radius: var(--radius-sm);
  }

  .btn-clear:hover {
    color: var(--status-error);
  }

  /* Popover Menu */
  .custom-font-menu {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    z-index: 200;
    background: #121218;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.7);
    display: flex;
    flex-direction: column;
    max-height: 260px;
    overflow: hidden;
  }

  .menu-search-box {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 10px;
    border-bottom: 1px solid var(--border-subtle);
    background: #0d0d12;
  }

  .menu-search-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    font-size: 12px;
    color: var(--text-main);
  }

  .clear-search-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
  }

  .menu-scroll-area {
    overflow-y: auto;
    padding: 5px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .font-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .group-title {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    padding: 3px 6px;
  }

  .font-options-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .font-option-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 5px 8px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: left;
    transition: all var(--transition-fast);
  }

  .font-option-item:hover {
    background: #1c1c26;
  }

  .font-option-item.active {
    background: rgba(139, 92, 246, 0.15);
    border-color: rgba(139, 92, 246, 0.35);
  }

  .font-item-left {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .font-item-name {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-main);
  }

  .font-option-item.active .font-item-name {
    color: #c084fc;
  }

  .font-item-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .badge-subtle {
    background: #181822;
    color: var(--text-secondary);
    font-size: 9.5px;
    padding: 1px 5px;
    border-radius: 4px;
  }
</style>
