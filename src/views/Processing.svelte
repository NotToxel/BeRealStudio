<script lang="ts">
  import {
    currentView,
    activeFeature,
    progressState,
    liveLogs,
    isProcessing,
  } from '$lib/stores';
  import { cancelToolkit, cancelRecapper } from '$lib/tauri';
  import XCircle from 'lucide-svelte/icons/circle-x';
  import Loader2 from 'lucide-svelte/icons/loader-circle';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import Minimize2 from 'lucide-svelte/icons/minimize-2';
  import { onMount } from 'svelte';
  import ProgressBar from '$components/ProgressBar.svelte';
  import LogConsole from '$components/LogConsole.svelte';

  onMount(() => {
    if (!$isProcessing) {
      currentView.set($activeFeature === 'toolkit' ? 'toolkit-config' : 'recapper-config');
    }
  });

  async function handleCancel() {
    try {
      if ($activeFeature === 'toolkit') {
        await cancelToolkit();
      } else {
        await cancelRecapper();
      }
    } catch (e) {
      console.warn('Cancel request error:', e);
    } finally {
      isProcessing.set(false);
      currentView.set('home');
    }
  }

  function handleRunInBackground() {
    currentView.set($activeFeature === 'toolkit' ? 'toolkit-config' : 'recapper-config');
  }
</script>

<div class="processing-view">
  <div class="header card">
    <div class="spinner-group">
      <div class="loader-wrap">
        <Loader2 size={32} class="animate-spin text-amber-400" />
      </div>
      <div class="title-info">
        <div class="stage-row">
          <h1 class="title-md font-bold text-white">
            {$activeFeature === 'toolkit' ? 'Processing Photos & Restoring EXIF...' : 'Generating Recap Video...'}
          </h1>
          <span class="badge badge-yellow">{$progressState.stage}</span>
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

      <button type="button" class="btn btn-danger btn-sm" on:click={handleCancel}>
        <XCircle size={14} />
        <span>Cancel Operation</span>
      </button>
    </div>
  </div>

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
</style>
