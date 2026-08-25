<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { APP_VERSION } from '$lib/version';
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
    checkOfflineGeoDb,
    downloadOfflineGeoDb,
    setActiveGeoDbTier,
    deleteOfflineGeoDb,
    onDownloadProgress,
  } from '$lib/tauri';
  import { offlineGeoDbStatus, isDownloadingGeoDb, downloadGeoDbProgress } from '$lib/stores';
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
  import Database from 'lucide-svelte/icons/database';
  import Loader2 from 'lucide-svelte/icons/loader-circle';

  let statusMessage = '';
  let isSuccessStatus = true;
  let showResetModal = false;

  onMount(async () => {
    if (!$ffmpegInfo.checked) {
      await detectFfmpeg();
    }
    try {
      const dbStatus = await checkOfflineGeoDb();
      offlineGeoDbStatus.set(dbStatus);
    } catch (e) {
      console.warn('Failed to check offline geodb:', e);
    }
  });

  async function handleDownloadTier(tierId: string) {
    isDownloadingGeoDb.set(true);
    const unlisten = await onDownloadProgress((p) => {
      downloadGeoDbProgress.set(p);
      if (p.percentage >= 100) {
        setTimeout(async () => {
          isDownloadingGeoDb.set(false);
          const status = await checkOfflineGeoDb();
          offlineGeoDbStatus.set(status);
          unlisten();
          showToast('Offline geocoding database installed successfully.');
        }, 800);
      }
    });

    try {
      await downloadOfflineGeoDb(tierId);
      const status = await checkOfflineGeoDb();
      offlineGeoDbStatus.set(status);
    } catch (e) {
      showToast(`Database download failed: ${e}`, false);
    } finally {
      isDownloadingGeoDb.set(false);
    }
  }

  async function handleSetActiveTier(tierId: string) {
    try {
      const status = await setActiveGeoDbTier(tierId);
      offlineGeoDbStatus.set(status);
      showToast(`Active geocoding dataset switched to ${status.activeTier}.`);
    } catch (e) {
      showToast(`Switch failed: ${e}`, false);
    }
  }

  async function handleDeleteTier(tierId: string) {
    try {
      const status = await deleteOfflineGeoDb(tierId);
      offlineGeoDbStatus.set(status);
      showToast('Dataset deleted.');
    } catch (e) {
      showToast(`Delete failed: ${e}`, false);
    }
  }

  const DEFAULT_SETTINGS_TIERS = [
    {
      id: 'cities15000',
      name: 'Lite',
      subtitle: 'Major Cities (>15,000 Pop)',
      minPopulation: 15000,
      approxCities: '25,000+ Cities',
      approxDownloadMb: 2.5,
      isInstalled: false,
      isActive: false,
      fileSizeBytes: 0,
      cityCount: 0,
      path: '',
    },
    {
      id: 'cities5000',
      name: 'Standard',
      subtitle: 'Towns & Cities (>5,000 Pop)',
      minPopulation: 5000,
      approxCities: '55,000+ Towns',
      approxDownloadMb: 4.5,
      isInstalled: false,
      isActive: false,
      fileSizeBytes: 0,
      cityCount: 0,
      path: '',
    },
    {
      id: 'cities500',
      name: 'Ultra Detailed',
      subtitle: 'Villages & Towns (>500 Pop)',
      minPopulation: 500,
      approxCities: '200,000+ Towns & Villages',
      approxDownloadMb: 12.5,
      isInstalled: false,
      isActive: true,
      fileSizeBytes: 0,
      cityCount: 0,
      path: '',
    },
  ];

  $: settingsTiers = $offlineGeoDbStatus?.tiers?.length ? $offlineGeoDbStatus.tiers : DEFAULT_SETTINGS_TIERS;

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
        version: APP_VERSION,
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

    <!-- 3. Offline Reverse Geocoding Datasets -->
    <div class="card section-card">
      <div class="card-head">
        <div class="head-title">
          <Database size={18} class="text-amber-400" />
          <h2 class="title-sm">3. Offline Reverse Geocoding Datasets</h2>
        </div>
      </div>

      <p class="text-secondary text-desc">
        Download one or multiple GeoNames datasets to enable sub-millisecond reverse geocoding with 100% offline privacy. You can switch active precision tiers at any time.
      </p>

      <div class="tiers-settings-list">
        {#each settingsTiers as tier}
          <div class="tier-setting-row card" class:active={tier.isActive}>
            <div class="tier-setting-left">
              <div class="tier-setting-icon" class:installed={tier.isInstalled}>
                {#if tier.isInstalled}
                  <CheckCircle size={16} class="text-emerald-400" />
                {:else}
                  <Database size={16} class="text-secondary" />
                {/if}
              </div>
              <div class="tier-setting-info">
                <div class="tier-setting-title-row">
                  <span class="font-semibold text-white text-sm">{tier.name} Dataset</span>
                  <span class="text-xs text-muted">({tier.subtitle})</span>
                  {#if tier.isActive}
                    <span class="badge badge-success font-mono text-xs">✓ Active In Memory</span>
                  {:else if tier.isInstalled}
                    <span class="badge badge-subtle font-mono text-xs">Installed</span>
                  {:else}
                    <span class="badge badge-yellow font-mono text-xs">~{tier.approxDownloadMb} MB Download</span>
                  {/if}
                </div>
                <span class="text-xs text-muted font-mono">
                  {tier.approxCities} &bull; {tier.isInstalled ? `${(tier.fileSizeBytes / 1048576).toFixed(1)} MB on disk` : `~${tier.approxDownloadMb} MB archive`}
                </span>
              </div>
            </div>

            <div class="tier-setting-actions">
              {#if tier.isInstalled}
                {#if !tier.isActive}
                  <button
                    type="button"
                    class="btn btn-secondary btn-xs"
                    on:click={() => handleSetActiveTier(tier.id)}
                    disabled={$isDownloadingGeoDb}
                  >
                    <span>Set Active</span>
                  </button>
                {/if}
                <button
                  type="button"
                  class="btn btn-ghost btn-xs text-muted hover:text-white"
                  on:click={() => handleDownloadTier(tier.id)}
                  disabled={$isDownloadingGeoDb}
                  title="Update or re-download dataset"
                >
                  <RefreshCw size={11} class={$isDownloadingGeoDb ? 'animate-spin' : ''} />
                  <span>Update</span>
                </button>
                <button
                  type="button"
                  class="btn btn-danger btn-xs"
                  on:click={() => handleDeleteTier(tier.id)}
                  disabled={$isDownloadingGeoDb}
                  title="Delete dataset file from disk"
                >
                  <Trash2 size={11} />
                  <span>Delete</span>
                </button>
              {:else}
                <button
                  type="button"
                  class="btn btn-accent-yellow btn-xs"
                  on:click={() => handleDownloadTier(tier.id)}
                  disabled={$isDownloadingGeoDb}
                >
                  <Download size={11} />
                  <span>Download (~{tier.approxDownloadMb} MB)</span>
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>

      {#if $isDownloadingGeoDb}
        <div class="geo-download-card card" style="margin-top: 8px;">
          <div class="geo-download-head">
            <div class="geo-download-title-row">
              <Loader2 size={16} class="animate-spin text-amber-400" />
              <span class="font-bold text-white text-sm">Downloading Geocoding Dataset...</span>
            </div>
            <span class="badge badge-yellow font-mono text-xs">
              {$downloadGeoDbProgress ? `${$downloadGeoDbProgress.percentage.toFixed(2)}%` : 'Streaming...'}
            </span>
          </div>
          <p class="text-xs text-secondary">
            {$downloadGeoDbProgress?.status || 'Connecting to GeoNames server...'}
          </p>
          <div class="geo-download-track">
            <div
              class="geo-download-fill"
              style="width: {Math.min(Math.max($downloadGeoDbProgress?.percentage || 5, 0), 100)}%;"
            ></div>
          </div>
        </div>
      {/if}
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

  {#if showResetModal}
    <div
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      transition:fade={{ duration: 160 }}
      on:click={() => (showResetModal = false)}
      on:keydown={(e) => e.key === 'Escape' && (showResetModal = false)}
    >
      <div
        class="modal-card"
        transition:scale={{ duration: 200, start: 0.94 }}
      >
        <div class="modal-head">
          <AlertTriangle size={20} class="text-rose-400" />
          <h3 class="title-sm font-bold text-white">Reset All Settings?</h3>
        </div>
        <p class="text-secondary text-xs">
          This will restore all Photo Processing and Recap Video preferences, formatting rules, quality thresholds, and directory paths to factory defaults.
        </p>
        <div class="modal-actions">
          <button type="button" class="btn btn-secondary btn-sm" on:click={() => (showResetModal = false)}>
            Cancel
          </button>
          <button type="button" class="btn btn-danger btn-sm" on:click={handleConfirmReset}>
            <RotateCcw size={13} />
            <span>Confirm Reset</span>
          </button>
        </div>
      </div>
    </div>
  {/if}
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

  /* Multi-Tier Dataset List */
  .tiers-settings-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .tier-setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 14px;
    background: #0e0e12;
    border: 1px solid var(--border-subtle);
    padding: 12px 16px;
    border-radius: var(--radius-md);
    flex-wrap: wrap;
    transition: all var(--transition-fast);
  }

  .tier-setting-row:hover {
    background: #13131a;
  }

  .tier-setting-row.active {
    border-color: rgba(16, 185, 129, 0.4);
    background: rgba(16, 185, 129, 0.04);
  }

  .tier-setting-left {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    flex: 1;
  }

  .tier-setting-icon {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: #181822;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .tier-setting-icon.installed {
    background: rgba(16, 185, 129, 0.12);
  }

  .tier-setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .tier-setting-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .tier-setting-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .geo-download-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: #14141d;
    border: 1px solid rgba(245, 158, 11, 0.35);
    padding: 12px 16px;
    border-radius: var(--radius-md);
  }

  .geo-download-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .geo-download-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .geo-download-track {
    width: 100%;
    height: 6px;
    background: #09090d;
    border-radius: 999px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .geo-download-fill {
    height: 100%;
    background: linear-gradient(90deg, #f59e0b, #ffe600);
    border-radius: 999px;
    box-shadow: 0 0 10px rgba(255, 230, 0, 0.5);
    transition: width 0.15s ease;
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
