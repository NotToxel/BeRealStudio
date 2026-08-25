<script lang="ts">
  import FolderOpen from 'lucide-svelte/icons/folder-open';
  import FileCode from 'lucide-svelte/icons/file-code';
  import AlertCircle from 'lucide-svelte/icons/alert-circle';
  import { pickFolder, pickFile } from '$lib/tauri';

  export let label: string;
  export let value: string = '';
  export let placeholder: string = 'Select a path...';
  export let isDirectory: boolean = true;
  export let allowBoth: boolean = false;
  export let fileExtensions: string[] = ['zip'];
  export let dialogTitle: string = 'Choose Path';
  export let required: boolean = false;
  export let disabled: boolean = false;
  export let isMissing: boolean = false;
  export let id: string = '';

  async function handleBrowse(folderMode: boolean = isDirectory) {
    if (disabled) return;
    isMissing = false;
    try {
      const selected = folderMode
        ? await pickFolder(dialogTitle)
        : await pickFile(dialogTitle, fileExtensions);
      if (selected) {
        value = selected;
      }
    } catch (e) {
      console.error('Failed to open file dialog', e);
    }
  }

  function handleFocus() {
    isMissing = false;
  }
</script>

<div {id} class="picker-group" class:has-missing-error={isMissing}>
  <div class="header">
    <label class="label" for={id ? `${id}-input` : 'path-input'}>{label}</label>
    {#if isMissing}
      <span class="missing-alert-tag">
        <AlertCircle size={11} /> Please select this {allowBoth ? 'file or folder' : isDirectory ? 'folder' : 'file'}
      </span>
    {:else if required && !value}
      <span class="required-tag">Required</span>
    {/if}
  </div>

  <div class="input-row">
    <input
      id={id ? `${id}-input` : 'path-input'}
      type="text"
      class="input-text"
      class:input-missing={isMissing}
      bind:value
      {placeholder}
      {disabled}
      on:focus={handleFocus}
      on:input={() => (isMissing = false)}
      spellcheck="false"
    />
    {#if allowBoth}
      <button
        type="button"
        class="btn btn-secondary btn-sm-pick"
        class:btn-missing={isMissing}
        on:click={() => handleBrowse(false)}
        {disabled}
        title="Select BeReal .ZIP file"
      >
        <FileCode size={14} class="text-amber-400" />
        <span>ZIP</span>
      </button>
      <button
        type="button"
        class="btn btn-secondary btn-sm-pick"
        class:btn-missing={isMissing}
        on:click={() => handleBrowse(true)}
        {disabled}
        title="Select unzipped BeReal export folder"
      >
        <FolderOpen size={14} class="text-sky-400" />
        <span>Folder</span>
      </button>
    {:else}
      <button
        type="button"
        class="btn btn-secondary"
        class:btn-missing={isMissing}
        on:click={() => handleBrowse(isDirectory)}
        {disabled}
      >
        {#if isDirectory}
          <FolderOpen size={14} class="text-amber-400" />
        {:else}
          <FileCode size={14} class="text-sky-400" />
        {/if}
        <span>Browse</span>
      </button>
    {/if}
  </div>
</div>

<style>
  .picker-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
    transition: transform 0.2s ease;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .label {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-main);
  }

  .required-tag {
    font-size: 11px;
    color: var(--status-warning);
    font-weight: 600;
  }

  .missing-alert-tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11.5px;
    color: #f87171;
    font-weight: 600;
    animation: fadeIn 0.2s ease-in;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-2px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .input-row {
    display: flex;
    gap: 8px;
    width: 100%;
  }

  .input-text {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 12.5px;
    transition: all var(--transition-fast);
  }

  @keyframes shake {
    0%, 100% { transform: translateX(0); }
    20%, 60% { transform: translateX(-5px); }
    40%, 80% { transform: translateX(5px); }
  }

  .has-missing-error {
    animation: shake 0.35s ease-in-out;
  }

  .input-missing {
    border-color: rgba(239, 68, 68, 0.75) !important;
    background: rgba(239, 68, 68, 0.05) !important;
    box-shadow: 0 0 12px rgba(239, 68, 68, 0.25) !important;
  }

  .btn-missing {
    border-color: rgba(239, 68, 68, 0.6) !important;
  }

  .btn-sm-pick {
    padding: 0 10px;
    font-size: 12px;
    gap: 4px;
    white-space: nowrap;
  }
</style>
