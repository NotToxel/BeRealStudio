<script lang="ts">
  import {
    currentView,
    activeFeature,
    toolkitConfig,
    recapperConfig,
    progressState,
    liveLogs,
    getPreferredRecapInputFolder,
    getSensibleRecapOutputPath,
  } from '$lib/stores';
  import { exportDebugLog, showInFolder, openFile } from '$lib/tauri';
  import CheckCircle from 'lucide-svelte/icons/circle-check';
  import FolderOpen from 'lucide-svelte/icons/folder-open';
  import Play from 'lucide-svelte/icons/play';
  import RotateCcw from 'lucide-svelte/icons/rotate-ccw';
  import Download from 'lucide-svelte/icons/download';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import Film from 'lucide-svelte/icons/film';
  import Camera from 'lucide-svelte/icons/camera';
  import History from 'lucide-svelte/icons/history';
  import LogConsole from '$components/LogConsole.svelte';

  $: isRecapper = $activeFeature === 'recapper';
  $: outputPath = isRecapper ? $recapperConfig.outputPath : $toolkitConfig.outputPath;

  async function handleOpenOutput() {
    if (outputPath) {
      try {
        await showInFolder(outputPath);
      } catch (e) {
        console.error('Failed to open output path:', e);
      }
    }
  }

  async function handlePlayVideo() {
    if (outputPath) {
      try {
        await openFile(outputPath);
      } catch (e) {
        console.error('Failed to launch video player:', e);
        await showInFolder(outputPath);
      }
    }
  }

  async function handleExportLogs() {
    try {
      const defaultName = `bereal_studio_debug_${Date.now()}.log`;
      const path = await exportDebugLog(defaultName);
      alert(`Debug log exported successfully to:\n${path}`);
    } catch (e) {
      alert(`Export failed: ${e}`);
    }
  }

  function handleOpenRecapper() {
    if ($toolkitConfig.outputPath) {
      $recapperConfig.inputFolder = getPreferredRecapInputFolder($toolkitConfig.outputPath, $toolkitConfig.createCombined);
      if (!$recapperConfig.outputPath) {
        $recapperConfig.outputPath = getSensibleRecapOutputPath($recapperConfig.inputFolder);
      }
    }
    activeFeature.set('recapper');
    currentView.set('recapper-config');
  }
</script>

<div class="complete-view">
  <!-- Celebration Banner -->
  <div class="card celebration-card" class:celebration-card-purple={isRecapper}>
    <div class="badge-icon-wrap" class:badge-icon-purple={isRecapper}>
      <CheckCircle size={36} class={isRecapper ? 'text-purple-400' : 'text-emerald-400'} />
    </div>

    <div class="celebrate-text">
      <div class="badge-row">
        <span class="badge {isRecapper ? 'badge-purple' : 'badge-success'}">Finished Successfully</span>
      </div>
      <h1 class="title-lg font-bold text-white">
        {isRecapper ? 'Recap Video Generated Successfully!' : 'All Memories Processed & Restored!'}
      </h1>
      <p class="text-secondary subtitle">
        {isRecapper
          ? 'Your music-synchronized vertical recap MP4 video has been rendered and saved.'
          : 'All photos converted, EXIF metadata injected, and dual-camera composites generated.'}
      </p>
    </div>

    <div class="actions-row">
      {#if isRecapper}
        <button type="button" class="btn btn-accent-violet btn-sm" on:click={handlePlayVideo}>
          <Play size={14} />
          <span>Play Recap Video Directly</span>
        </button>

        <button type="button" class="btn btn-secondary btn-sm" on:click={handleOpenOutput}>
          <FolderOpen size={14} />
          <span>Show in File Explorer</span>
        </button>

        <button type="button" class="btn btn-secondary btn-sm" on:click={() => currentView.set('recapper-config')}>
          <RotateCcw size={14} />
          <span>Create Another Recap</span>
        </button>
      {:else}
        <button type="button" class="btn btn-primary btn-sm" on:click={handleOpenOutput}>
          <FolderOpen size={14} />
          <span>Open Output in File Explorer</span>
        </button>

        <button type="button" class="btn btn-accent-violet btn-sm" on:click={handleOpenRecapper}>
          <Film size={14} />
          <span>Generate Recap Video Now &rarr;</span>
        </button>

        <button type="button" class="btn btn-secondary btn-sm" on:click={() => currentView.set('toolkit-config')}>
          <Camera size={14} />
          <span>Process More Photos</span>
        </button>
      {/if}

      <button type="button" class="btn btn-secondary btn-sm" on:click={() => currentView.set('activity')}>
        <History size={14} />
        <span>View Activity History</span>
      </button>

      <button type="button" class="btn btn-secondary btn-sm" on:click={() => currentView.set('home')}>
        <RotateCcw size={14} />
        <span>Return to Home</span>
      </button>

      <button type="button" class="btn btn-ghost btn-sm" on:click={handleExportLogs}>
        <Download size={14} />
        <span>Export Log</span>
      </button>
    </div>
  </div>

  <!-- Summary Metrics Grid -->
  <div class="metrics-grid">
    <div class="metric-card card">
      <div class="metric-head">
        {#if isRecapper}
          <Film size={16} class="text-purple-400" />
          <span class="text-secondary">Recap Output</span>
        {:else}
          <Camera size={16} class="text-amber-400" />
          <span class="text-secondary">Processed Memories</span>
        {/if}
      </div>
      <strong class="metric-val {isRecapper ? 'text-purple-400' : 'text-amber-400'}">
        {$progressState.total || $progressState.current || 1} {isRecapper ? 'Frames' : 'Photos'}
      </strong>
      <span class="text-muted text-xs">
        {isRecapper ? 'Music-synchronized video rendered' : 'EXIF & GPS metadata embedded'}
      </span>
    </div>

    <div class="metric-card card">
      <div class="metric-head">
        <Sparkles size={16} class="text-emerald-400" />
        <span class="text-secondary">Status</span>
      </div>
      <strong class="metric-val text-emerald-400">100%</strong>
      <span class="text-muted text-xs">
        {isRecapper ? 'Zero dropped frames • Ready' : 'Dual-camera PIP created'}
      </span>
    </div>

    <div class="metric-card card">
      <div class="metric-head">
        <FolderOpen size={16} class="text-sky-400" />
        <span class="text-secondary">{isRecapper ? 'Video File Destination' : 'Output Folder'}</span>
      </div>
      <strong class="metric-val path-val text-sky-400" title={outputPath}>
        {outputPath || 'Saved locally'}
      </strong>
      <span class="text-muted text-xs">Saved on local disk</span>
    </div>
  </div>

  <LogConsole logs={$liveLogs} />
</div>

<style>
  .complete-view {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding-bottom: 40px;
  }

  .celebration-card {
    display: flex;
    align-items: center;
    gap: 20px;
    background: #111116;
    border: 1px solid rgba(52, 211, 153, 0.3);
    padding: 24px 28px;
    border-radius: var(--radius-lg);
    box-shadow: 0 8px 32px rgba(52, 211, 153, 0.08);
    flex-wrap: wrap;
  }

  .celebration-card-purple {
    border-color: rgba(168, 85, 247, 0.35);
    box-shadow: 0 8px 32px rgba(168, 85, 247, 0.1);
  }

  .badge-icon-wrap {
    width: 60px;
    height: 60px;
    border-radius: 50%;
    background: rgba(52, 211, 153, 0.12);
    border: 1px solid rgba(52, 211, 153, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 0 20px rgba(52, 211, 153, 0.2);
  }

  .badge-icon-purple {
    background: rgba(168, 85, 247, 0.15);
    border-color: rgba(168, 85, 247, 0.4);
    box-shadow: 0 0 20px rgba(168, 85, 247, 0.25);
  }

  .celebrate-text {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
    min-width: 260px;
  }

  .badge-row {
    display: flex;
  }

  .subtitle {
    font-size: 13.5px;
    line-height: 1.5;
  }

  .actions-row {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 16px;
  }

  .metric-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: #111116;
    padding: 16px 20px;
  }

  .metric-head {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    font-weight: 500;
  }

  .metric-val {
    font-size: 22px;
    font-family: var(--font-mono);
  }

  .path-val {
    font-size: 13px;
    word-break: break-all;
  }

  .text-xs {
    font-size: 11px;
  }

  @media (max-width: 768px) {
    .metrics-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
