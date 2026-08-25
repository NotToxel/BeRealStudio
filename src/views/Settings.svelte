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
    saveSettings,
    resetSettings,
    clearDebugLogs,
    saveFilePicker,
    pickFile,
  } from '$lib/tauri';
  import Cpu from 'lucide-svelte/icons/cpu';
  import RotateCcw from 'lucide-svelte/icons/rotate-ccw';
  import Trash2 from 'lucide-svelte/icons/trash-2';
  import CheckCircle from 'lucide-svelte/icons/circle-check';
  import AlertTriangle from 'lucide-svelte/icons/triangle-alert';
  import RefreshCw from 'lucide-svelte/icons/refresh-cw';
  import ShieldCheck from 'lucide-svelte/icons/shield-check';
  import Download from 'lucide-svelte/icons/download';
  import Upload from 'lucide-svelte/icons/upload';
  import Save from 'lucide-svelte/icons/save';
  import Sliders from 'lucide-svelte/icons/sliders';

  let statusMessage = '';
  let isSuccessStatus = true;
  let showResetModal = false;

  onMount(async () => {
    if (!$ffmpegInfo.checked) {
      await detectFfmpeg();
    }
  });

  function showToast(msg: string, success = true) {
    statusMessage = msg;
    isSuccessStatus = success;
    setTimeout(() => (statusMessage = ''), 4000);
  }

  async function handleManualSave() {
    try {
      await saveSettings({
        toolkit: $toolkitConfig,
        recapper: $recapperConfig,
        lastInputPath: $toolkitConfig.inputPath,
        lastOutputPath: $toolkitConfig.outputPath,
      });
      showToast('All configurations saved successfully.');
    } catch (e) {
      showToast(`Save failed: ${e}`, false);
    }
  }

  async function handleExportSettings() {
    try {
      const configData = {
        version: '1.0.0',
        exportedAt: new Date().toISOString(),
        toolkit: $toolkitConfig,
        recapper: $recapperConfig,
      };
      const jsonStr = JSON.stringify(configData, null, 2);

      const filePath = await saveFilePicker('Export BeReal Studio Settings', 'bereal_studio_settings.json', ['json']);
      if (filePath) {
        // In browser or Tauri, write or trigger download
        const blob = new Blob([jsonStr], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filePath.split(/[/\\]/).pop() || 'bereal_studio_settings.json';
        a.click();
        URL.revokeObjectURL(url);
        showToast('Settings exported successfully.');
      }
    } catch (e) {
      showToast(`Export failed: ${e}`, false);
    }
  }

  async function handleImportSettings() {
    try {
      const selected = await pickFile('Import BeReal Studio Settings', ['json']);
      if (selected) {
        // If native path returned, let user input or use file upload element
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.json,application/json';
        input.onchange = async (e) => {
          const file = (e.target as HTMLInputElement).files?.[0];
          if (!file) return;
          const text = await file.text();
          const parsed = JSON.parse(text);
          if (parsed.toolkit) toolkitConfig.set({ ...$toolkitConfig, ...parsed.toolkit });
          if (parsed.recapper) recapperConfig.set({ ...$recapperConfig, ...parsed.recapper });
          await handleManualSave();
          showToast('Settings imported and applied successfully!');
        };
        input.click();
      }
    } catch (e) {
      showToast(`Import failed: ${e}`, false);
    }
  }

  async function handleConfirmReset() {
    showResetModal = false;
    try {
      await resetSettings();
      toolkitConfig.set({ ...defaultToolkitConfig });
      recapperConfig.set({ ...defaultRecapperConfig });
      showToast('All configurations have been restored to factory defaults.');
    } catch (e) {
      showToast(`Reset failed: ${e}`, false);
    }
  }

  async function handleClearLogs() {
    try {
      await clearDebugLogs();
      liveLogs.set([]);
      showToast('Debug and live logs cleared.');
    } catch (e) {
      showToast(`Clear failed: ${e}`, false);
    }
  }
</script>

<div class="settings-view">
  <div class="view-header">
    <h1 class="title-lg font-bold">Application Settings</h1>
    <p class="subtitle text-secondary">
      System dependencies, persistent preferences, export/import, and privacy management.
    </p>
  </div>

  {#if statusMessage}
    <div class="status-banner {isSuccessStatus ? 'status-success' : 'status-error'}">
      <CheckCircle size={14} />
      <span>{statusMessage}</span>
    </div>
  {/if}

  <!-- Reset Confirmation Modal -->
  {#if showResetModal}
    <div class="modal-backdrop">
      <div class="modal-card">
        <div class="modal-head">
          <AlertTriangle size={22} class="text-amber-400" />
          <h3 class="title-sm font-bold text-white">Reset All Settings to Defaults?</h3>
        </div>
        <p class="modal-body text-secondary text-sm">
          Are you sure you want to restore all photo processing and recap video options to factory defaults? Your processed photos on disk will not be affected.
        </p>
        <div class="modal-actions">
          <button type="button" class="btn btn-secondary btn-sm" on:click={() => (showResetModal = false)}>
            Cancel
          </button>
          <button type="button" class="btn btn-danger btn-sm" on:click={handleConfirmReset}>
            <RotateCcw size={13} />
            <span>Yes, Reset to Defaults</span>
          </button>
        </div>
      </div>
    </div>
  {/if}

  <div class="sections-list">
    <!-- 1. Configuration Management (Export / Import / Save) -->
    <div class="card section-card">
      <div class="card-head">
        <div class="head-title">
          <Sliders size={18} class="text-sky-400" />
          <h2 class="title-sm">1. Preferences &amp; Presets Management</h2>
        </div>
      </div>

      <p class="text-secondary text-desc">
        Backup your preferred video pacing, location rules, quality thresholds, and directory locations to a JSON file or restore a previous preset.
      </p>

      <div class="btn-actions-row">
        <button type="button" class="btn btn-primary btn-sm" on:click={handleManualSave}>
          <Save size={13} />
          <span>Save Current Settings</span>
        </button>

        <button type="button" class="btn btn-secondary btn-sm" on:click={handleExportSettings}>
          <Download size={13} />
          <span>Export Settings JSON</span>
        </button>

        <button type="button" class="btn btn-secondary btn-sm" on:click={handleImportSettings}>
          <Upload size={13} />
          <span>Import Settings JSON</span>
        </button>
      </div>
    </div>

    <!-- 2. System & Dependencies -->
    <div class="card section-card">
      <div class="card-head">
        <div class="head-title">
          <Cpu size={18} class="text-sky-400" />
          <h2 class="title-sm">2. System &amp; Video Encoders</h2>
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

    <!-- 3. Local Storage & Privacy Management -->
    <div class="card section-card">
      <div class="head-title">
        <ShieldCheck size={18} class="text-emerald-400" />
        <h2 class="title-sm">3. Local Storage &amp; Factory Reset</h2>
      </div>

      <p class="text-secondary text-desc">
        BeReal Studio processes all archives entirely on your local machine. No photos, audio, metadata, or telemetry are ever uploaded or transmitted.
      </p>

      <div class="btn-actions-row">
        <button type="button" class="btn btn-secondary btn-sm" on:click={handleClearLogs}>
          <Trash2 size={13} />
          <span>Clear Memory Logs</span>
        </button>

        <button type="button" class="btn btn-danger btn-sm" on:click={() => (showResetModal = true)}>
          <RotateCcw size={13} />
          <span>Reset All Configurations to Defaults...</span>
        </button>
      </div>
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

  .status-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-radius: var(--radius-md);
    font-size: 12.5px;
    font-weight: 500;
  }

  .status-success {
    background: rgba(52, 211, 153, 0.12);
    border: 1px solid rgba(52, 211, 153, 0.3);
    color: #34d399;
  }

  .status-error {
    background: rgba(239, 68, 68, 0.12);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #f87171;
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

  /* Modal Backdrop */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 999;
    padding: 20px;
  }

  .modal-card {
    background: #131318;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    padding: 24px;
    max-width: 440px;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 14px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
  }

  .modal-head {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 6px;
  }
</style>
