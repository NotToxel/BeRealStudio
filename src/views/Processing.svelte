<script lang="ts">
  import {
    currentView,
    activeFeature,
    progressState,
    liveLogs,
    isProcessing,
    toolkitConfig,
    recapperConfig,
    activeJobs,
  } from '$lib/stores';
  import { cancelToolkit, cancelRecapper, cleanupCancelledOutput, showInFolder } from '$lib/tauri';
  import Modal from '$components/Modal.svelte';
  import XCircle from 'lucide-svelte/icons/circle-x';
  import Loader2 from 'lucide-svelte/icons/loader-circle';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import Minimize2 from 'lucide-svelte/icons/minimize-2';
  import FolderOpen from 'lucide-svelte/icons/folder-open';
  import ExternalLink from 'lucide-svelte/icons/external-link';
  import AlertTriangle from 'lucide-svelte/icons/triangle-alert';
  import Trash2 from 'lucide-svelte/icons/trash-2';
  import Check from 'lucide-svelte/icons/check';
  import { onMount } from 'svelte';
  import ProgressBar from '$components/ProgressBar.svelte';
  import LogConsole from '$components/LogConsole.svelte';

  $: isRecapper = $activeFeature === 'recapper';
  $: currentOutputPath = isRecapper ? $recapperConfig.outputPath : $toolkitConfig.outputPath;

  let showCancelModal = false;

  onMount(() => {
    if (!$isProcessing) {
      currentView.set(isRecapper ? 'recapper-config' : 'toolkit-config');
    }
  });

  function handleCancelClick() {
    if (isRecapper) {
      handleCancelRecapper();
    } else {
      showCancelModal = true;
    }
  }

  async function handleCancelRecapper() {
    try {
      await cancelRecapper();
      if (currentOutputPath) {
        await cleanupCancelledOutput(currentOutputPath);
      }
    } catch (e) {
      console.warn('Cancel recapper error:', e);
    } finally {
      isProcessing.set(false);
      currentView.set('recapper-config');
    }
  }

  async function handleConfirmCancel(deleteFiles: boolean) {
    showCancelModal = false;
    try {
      await cancelToolkit();
      if (deleteFiles && currentOutputPath) {
        await cleanupCancelledOutput(currentOutputPath);
      }
    } catch (e) {
      console.warn('Cancel toolkit error:', e);
    } finally {
      isProcessing.set(false);
      currentView.set('toolkit-config');
    }
  }

  function handleRunInBackground() {
    currentView.set(isRecapper ? 'recapper-config' : 'toolkit-config');
  }

  async function handleOpenOutputFolder() {
    if (currentOutputPath) {
      try {
        await showInFolder(currentOutputPath);
      } catch (e) {
        console.warn('Failed to open destination folder:', e);
      }
    }
  }
</script>

<div class="processing-view" class:theme-purple={isRecapper}>
  <div class="header card">
    <div class="spinner-group">
      <div class="loader-wrap" class:loader-wrap-purple={isRecapper}>
        <Loader2 size={32} class="animate-spin {isRecapper ? 'text-purple-400' : 'text-amber-400'}" />
      </div>
      <div class="title-info">
        <div class="stage-row">
          <h1 class="title-md font-bold text-white">
            {$activeFeature === 'toolkit' ? 'Processing Photos & Restoring EXIF...' : 'Generating Recap Video...'}
          </h1>
          <span class="badge {isRecapper ? 'badge-purple' : 'badge-yellow'}">{$progressState.stage}</span>
        </div>
        <p class="text-secondary text-sub">
          {$progressState.current} of {$progressState.total} items &bull; <strong class="text-white font-mono">{$progressState.percentage.toFixed(2)}%</strong> completed
        </p>
      </div>
    </div>

    <div class="actions-group">
      <button type="button" class="btn btn-secondary btn-sm" on:click={handleRunInBackground} title="Continue working while processing runs in background">
        <Minimize2 size={14} />
        <span>Hide to Background</span>
      </button>

      <button type="button" class="btn btn-danger btn-sm" on:click={handleCancelClick}>
        <XCircle size={14} />
        <span>Cancel Operation</span>
      </button>
    </div>
  </div>

  <Modal bind:open={showCancelModal} title="Cancel Photo Processing?" maxWidth="460px">
    <svelte:fragment slot="title">
      <div class="modal-head-row">
        <AlertTriangle size={20} class="text-amber-400" />
        <h3 class="title-sm font-bold text-white">Cancel Photo Processing?</h3>
      </div>
    </svelte:fragment>

    <div class="modal-body-content">
      <p class="text-secondary text-sm">
        Processing will be stopped immediately. What would you like to do with the files that have already been generated in the output folder?
      </p>
      <div class="dest-preview-box font-mono text-xs text-muted">
        {currentOutputPath}
      </div>
    </div>

    <svelte:fragment slot="footer">
      <div class="cancel-modal-actions">
        <button type="button" class="btn btn-secondary btn-sm" on:click={() => (showCancelModal = false)}>
          Continue Processing
        </button>
        <button type="button" class="btn btn-secondary btn-sm text-amber-400" on:click={() => handleConfirmCancel(false)}>
          <Check size={13} />
          <span>Keep Files &amp; Cancel</span>
        </button>
        <button type="button" class="btn btn-danger btn-sm" on:click={() => handleConfirmCancel(true)}>
          <Trash2 size={13} />
          <span>Delete Files &amp; Cancel</span>
        </button>
      </div>
    </svelte:fragment>
  </Modal>

  {#if currentOutputPath}
    <div class="destination-bar card">
      <div class="dest-info">
        <FolderOpen size={16} class={isRecapper ? 'text-purple-400' : 'text-amber-400'} />
        <div class="dest-text-group">
          <span class="dest-label">Output Destination:</span>
          <span class="dest-path font-mono text-muted">{currentOutputPath}</span>
        </div>
      </div>
      <button
        type="button"
        class="btn btn-secondary btn-xs"
        on:click={handleOpenOutputFolder}
        title="Open destination folder in system file explorer"
      >
        <ExternalLink size={12} />
        <span>Open Destination Folder</span>
      </button>
    </div>
  {/if}

  <ProgressBar
    percentage={$progressState.percentage}
    stage={$progressState.stage}
    current={$progressState.current}
    total={$progressState.total}
  />

  <LogConsole logs={$liveLogs} />
</div>

<style>
  .processing-view {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-bottom: 30px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #111116;
    border: 1px solid var(--border-subtle);
    padding: 20px 24px;
    border-radius: var(--radius-lg);
  }

  .spinner-group {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .loader-wrap {
    width: 44px;
    height: 44px;
    background: rgba(255, 230, 0, 0.1);
    border: 1px solid rgba(255, 230, 0, 0.25);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .loader-wrap-purple {
    background: rgba(168, 85, 247, 0.12);
    border-color: rgba(168, 85, 247, 0.35);
  }

  .title-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .stage-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .text-sub {
    font-size: 12.5px;
    font-family: var(--font-mono);
  }

  .actions-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .destination-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    background: #0e0e13;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .dest-info {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .dest-text-group {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex-wrap: wrap;
  }

  .dest-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-main);
  }

  .dest-path {
    font-size: 11.5px;
    word-break: break-all;
  }

  .modal-head-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .modal-body-content {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 6px 0;
  }

  .dest-preview-box {
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    word-break: break-all;
  }

  .cancel-modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    flex-wrap: wrap;
  }
</style>
