<script lang="ts">
  import { onMount } from 'svelte';
  import {
    currentView,
    activeFeature,
    toolkitConfig,
    recapperConfig,
    progressState,
    liveLogs,
    recordActivity,
  } from '$lib/stores';
  import { exportDebugLog, showInFolder } from '$lib/tauri';
  import CheckCircle from 'lucide-svelte/icons/circle-check';
  import FolderOpen from 'lucide-svelte/icons/folder-open';
  import ExternalLink from 'lucide-svelte/icons/external-link';
  import RotateCcw from 'lucide-svelte/icons/rotate-ccw';
  import Download from 'lucide-svelte/icons/download';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import Film from 'lucide-svelte/icons/film';
  import Camera from 'lucide-svelte/icons/camera';
  import History from 'lucide-svelte/icons/history';
  import LogConsole from '$components/LogConsole.svelte';

  onMount(() => {
    const isTk = $activeFeature === 'toolkit';
    const out = isTk ? $toolkitConfig.outputPath : $recapperConfig.outputPath;
    const inp = isTk ? $toolkitConfig.inputPath : $recapperConfig.inputFolder;
    recordActivity({
      type: isTk ? 'toolkit' : 'recapper',
      title: isTk ? 'Photo Processing Suite' : 'Recap Video Generator',
      outputPath: out,
      inputPath: inp,
      itemCount: $progressState.total || $progressState.current || 1,
      durationSecs: 0,
      status: 'success',
      details: isTk
        ? `Format: ${$toolkitConfig.convertFormat} • Combined: ${$toolkitConfig.createCombined ? 'Yes' : 'No'}`
        : `Speed: ${$recapperConfig.speedMode} • ${$recapperConfig.fps} FPS`,
    });
  });

  async function handleOpenOutput() {
    const out = $activeFeature === 'toolkit' ? $toolkitConfig.outputPath : $recapperConfig.outputPath;
    if (out) {
      try {
        await showInFolder(out);
      } catch (e) {
        console.error('Failed to open output path:', e);
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
    currentView.set('recapper-config');
  }
</script>

<div class="complete-view">
  <!-- Celebration Banner -->
  <div class="card celebration-card">
    <div class="badge-icon-wrap">
      <CheckCircle size={36} class="text-emerald-400" />
    </div>

    <div class="celebrate-text">
      <div class="badge-row">
        <span class="badge badge-success">Finished Successfully</span>
      </div>
      <h1 class="title-lg font-bold text-white">
        {$activeFeature === 'toolkit' ? 'All Memories Processed & Restored!' : 'Recap Video Generated!'}
      </h1>
      <p class="text-secondary subtitle">
        {$activeFeature === 'toolkit'
          ? `All photos converted, EXIF metadata injected, and dual-camera composites generated.`
          : `Music-synchronized vertical recap video rendered and exported.`}
      </p>
    </div>

    <div class="actions-row">
      <button type="button" class="btn btn-primary btn-sm" on:click={handleOpenOutput}>
        <FolderOpen size={14} />
        <span>Open Output in File Explorer</span>
      </button>

      {#if $activeFeature === 'toolkit'}
        <button type="button" class="btn btn-accent-violet btn-sm" on:click={handleOpenRecapper}>
          <Film size={14} />
          <span>Generate Recap Video Now &rarr;</span>
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
        <span>Export Activity Log</span>
      </button>
    </div>
  </div>

  <!-- Summary Metrics Grid -->
  <div class="metrics-grid">
    <div class="metric-card card">
      <div class="metric-head">
        <Camera size={16} class="text-amber-400" />
        <span class="text-secondary">Processed Count</span>
      </div>
      <strong class="metric-val text-amber-400">{$progressState.total || $progressState.current}</strong>
      <span class="text-muted text-xs">Total items completed</span>
    </div>

    <div class="metric-card card">
      <div class="metric-head">
        <Sparkles size={16} class="text-emerald-400" />
        <span class="text-secondary">Status</span>
      </div>
      <strong class="metric-val text-emerald-400">100%</strong>
      <span class="text-muted text-xs">Zero dropped frames</span>
    </div>

    <div class="metric-card card">
      <div class="metric-head">
        <FolderOpen size={16} class="text-sky-400" />
        <span class="text-secondary">Target Location</span>
      </div>
      <strong class="metric-val path-val text-sky-400">
        {$activeFeature === 'toolkit' ? $toolkitConfig.outputPath : $recapperConfig.outputPath}
      </strong>
      <span class="text-muted text-xs">Saved on local storage</span>
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
