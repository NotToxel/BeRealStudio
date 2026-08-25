<script lang="ts">
  import { onMount } from 'svelte';
  import { listSystemFonts, pickFile } from '$lib/tauri';
  import { BUILTIN_FONT_OPTIONS } from '$lib/fonts';
  import type { FontInfo } from '$lib/types';
  import FolderOpen from 'lucide-svelte/icons/folder-open';
  import X from 'lucide-svelte/icons/circle-x';

  export let selectedPath: string = 'inter';
  export let label: string = 'Font Family';

  let customFonts: FontInfo[] = [];
  let isCustomFile = false;
  let customFileName = '';

  $: {
    const isBuiltin = BUILTIN_FONT_OPTIONS.some((b) => b.id === selectedPath);
    const isSystem = customFonts.some((f) => f.path === selectedPath);
    isCustomFile = Boolean(selectedPath) && !isBuiltin && !isSystem && (selectedPath.includes('/') || selectedPath.includes('\\'));
    if (isCustomFile) {
      customFileName = selectedPath.split(/[/\\]/).pop() || selectedPath;
    }
  }

  async function handleBrowseCustomFont() {
    try {
      const file = await pickFile('Select Custom Font File', ['ttf', 'otf', 'woff2']);
      if (file) {
        selectedPath = file;
      }
    } catch (e) {
      console.warn('Failed to pick custom font', e);
    }
  }

  function handleResetCustom() {
    selectedPath = 'inter';
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

<div class="font-picker-wrapper">
  <div class="header">
    <label class="label" for="font-select">{label}</label>
  </div>

  <div class="controls-row">
    {#if isCustomFile}
      <div class="custom-pill">
        <span class="font-name font-mono">{customFileName}</span>
        <button type="button" class="btn-clear" title="Remove custom font" on:click={handleResetCustom}>
          <X size={13} />
        </button>
      </div>
    {:else}
      <select id="font-select" class="input-select font-dropdown" bind:value={selectedPath}>
        <optgroup label="Built-in Fonts">
          {#each BUILTIN_FONT_OPTIONS as font}
            <option value={font.id}>
              {font.name} ({font.category})
            </option>
          {/each}
        </optgroup>
        {#if customFonts.length > 0}
          <optgroup label="System Fonts">
            {#each customFonts as font}
              <option value={font.path}>
                {font.family} {font.style !== '0' ? `(${font.style})` : ''}
              </option>
            {/each}
          </optgroup>
        {/if}
      </select>
    {/if}

    <button
      type="button"
      class="btn btn-secondary btn-browse-font"
      title="Load your own .ttf or .otf file"
      on:click={handleBrowseCustomFont}
    >
      <FolderOpen size={14} class="text-amber-400" />
      <span>Custom Font...</span>
    </button>
  </div>
</div>

<style>
  .font-picker-wrapper {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
  }

  .header {
    display: flex;
    align-items: center;
  }

  .label {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-main);
  }

  .controls-row {
    display: flex;
    gap: 8px;
    align-items: center;
    width: 100%;
  }

  .font-dropdown {
    flex: 1;
    padding: 8px 12px;
    font-size: 13px;
  }

  .btn-browse-font {
    white-space: nowrap;
    padding: 8px 12px;
    font-size: 12.5px;
    flex-shrink: 0;
  }

  .custom-pill {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: #14141a;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 7px 12px;
  }

  .font-name {
    font-size: 12.5px;
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
</style>
