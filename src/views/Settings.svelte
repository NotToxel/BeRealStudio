<script lang="ts">
  import { onMount } from 'svelte';
  import {
    toolkitConfig,
    recapperConfig,
    liveLogs,
    ffmpegInfo,
    defaultToolkitConfig,
    defaultRecapperConfig,
  } from '$lib/stores';
  import {
    detectFfmpeg,
    resetSettings,
    clearDebugLogs,
  } from '$lib/tauri';
  import Cpu from 'lucide-svelte/icons/cpu';
  import RotateCcw from 'lucide-svelte/icons/rotate-ccw';
  import Trash2 from 'lucide-svelte/icons/trash-2';
  import CheckCircle from 'lucide-svelte/icons/circle-check';
  import AlertTriangle from 'lucide-svelte/icons/triangle-alert';
  import RefreshCw from 'lucide-svelte/icons/refresh-cw';
  import ShieldCheck from 'lucide-svelte/icons/shield-check';

  let resetStatus = '';

  onMount(async () => {
    if (!$ffmpegInfo.checked) {
      await detectFfmpeg();
    }
  });

  async function handleResetDefaults() {
    try {
      await resetSettings();
      toolkitConfig.set({ ...defaultToolkitConfig });
      recapperConfig.set({ ...defaultRecapperConfig });
      resetStatus = 'All settings have been restored to factory defaults.';
      setTimeout(() => (resetStatus = ''), 3500);
    } catch (e) {
      resetStatus = `Reset failed: ${e}`;
    }
  }

  async function handleClearLogs() {
    try {
      await clearDebugLogs();
      liveLogs.set([]);
      resetStatus = 'Debug and activity logs cleared.';
      setTimeout(() => (resetStatus = ''), 3500);
    } catch (e) {
      resetStatus = `Clear failed: ${e}`;
    }
  }
</script>

<div class="settings-view">
  <div class="view-header">
    <h1 class="title-lg">Application Settings</h1>
    <p class="subtitle text-secondary">
      System dependencies, local storage management, and diagnostics.
    </p>
  </div>

  <div class="sections-list">
    <!-- 1. System & Dependencies -->
    <div class="card section-card">
      <div class="card-head">
        <div class="head-title">
          <Cpu size={18} class="text-sky-400" />
          <h2 class="title-sm">1. System &amp; Video Encoders</h2>
        </div>
        <button
          type="button"
          class="btn btn-secondary btn-sm"
          on:click={() => detectFfmpeg()}
          disabled={$ffmpegInfo.checking}
        >
          <RefreshCw size={12} class={$ffmpegInfo.checking ? 'animate-spin' : ''} />
          <span>{$ffmpegInfo.checking ? 'Checking...' : 'Recheck FFmpeg'}</span>
        </button>
      </div>

      <div class="dependency-status">
        <div class="dep-icon" class:found={Boolean($ffmpegInfo.path)}>
          {#if $ffmpegInfo.checking && !$ffmpegInfo.checked}
            <RefreshCw size={18} class="animate-spin text-sky-400" />
          {:else if $ffmpegInfo.path}
            <CheckCircle size={18} />
          {:else}
            <AlertTriangle size={18} />
          {/if}
        </div>

        <div class="dep-info">
          <div class="dep-name">
            <strong>FFmpeg Video Engine</strong>
            {#if $ffmpegInfo.checking && !$ffmpegInfo.checked}
              <span class="badge badge-info">Detecting...</span>
            {:else if $ffmpegInfo.path}
              <span class="badge badge-success">Detected</span>
            {:else}
              <span class="badge badge-warning">Not Detected on PATH</span>
            {/if}
          </div>
          <p class="dep-path text-secondary">
            {#if $ffmpegInfo.checking && !$ffmpegInfo.checked}
              Scanning system PATH for FFmpeg...
            {:else}
              {$ffmpegInfo.path || 'FFmpeg is required to generate recap slideshows and video PIP overlays.'}
            {/if}
          </p>
        </div>
      </div>
    </div>

    <!-- 2. Local Storage & Privacy Management -->
    <div class="card section-card">
      <div class="head-title">
        <ShieldCheck size={18} class="text-emerald-400" />
        <h2 class="title-sm">2. Local Storage &amp; Privacy</h2>
      </div>

      <p class="text-secondary text-desc">
        BeReal Studio processes all archives entirely on your local machine. No photos, audio, metadata, or telemetry are ever uploaded or transmitted.
      </p>

      <div class="btn-actions-row">
        <button type="button" class="btn btn-secondary btn-sm" on:click={handleClearLogs}>
          <Trash2 size={13} />
          <span>Clear Activity Logs</span>
        </button>

        <button type="button" class="btn btn-danger btn-sm" on:click={handleResetDefaults}>
          <RotateCcw size={13} />
          <span>Reset All Configurations to Defaults</span>
        </button>
      </div>

      {#if resetStatus}
        <div class="status-banner">{resetStatus}</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .settings-view {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding-bottom: 40px;
  }

  .view-header {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .subtitle {
    font-size: 13.5px;
  }

  .sections-list {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .section-card {
    display: flex;
    flex-direction: column;
    gap: 16px;
    background: #111116;
  }

  .card-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .head-title {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .dependency-status {
    display: flex;
    align-items: center;
    gap: 14px;
    background: #0e0e11;
    border: 1px solid var(--border-subtle);
    padding: 12px 16px;
    border-radius: var(--radius-md);
  }

  .dep-icon {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--status-error-bg);
    color: var(--status-error);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .dep-icon.found {
    background: rgba(52, 211, 153, 0.15);
    color: #34d399;
  }

  .dep-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
  }

  .dep-name {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }

  .dep-path {
    font-size: 12px;
    font-family: var(--font-mono);
    word-break: break-all;
  }

  .text-desc {
    font-size: 13px;
    line-height: 1.5;
  }

  .btn-actions-row {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }

  .status-banner {
    background: #141419;
    border: 1px solid var(--border-subtle);
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    color: var(--status-info);
    font-family: var(--font-mono);
  }
</style>
