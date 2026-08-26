<script lang="ts">
  import { onMount } from 'svelte';
  import Modal from '$components/Modal.svelte';
  import Toggle from '$components/Toggle.svelte';
  import { APP_VERSION } from '$lib/version';
  import { isDev } from '$lib/devMode';
  import {
    toolkitConfig,
    recapperConfig,
    liveLogs,
    ffmpegInfo,
    defaultToolkitConfig,
    defaultRecapperConfig,
    offlineGeoDbStatus,
    isDownloadingGeoDb,
    downloadGeoDbProgress,
    hwInfoStore,
    exiftoolStore,
  } from '$lib/stores';
  import {
    detectFfmpeg,
    checkExiftool,
    checkHardwareAcceleration,
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
  import type { HardwareAccelerationInfo } from '$lib/types';
  import Cpu from 'lucide-svelte/icons/cpu';
  import Sparkles from 'lucide-svelte/icons/sparkles';
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
  import Camera from 'lucide-svelte/icons/camera';
  import Eye from 'lucide-svelte/icons/eye';
  import Clock from 'lucide-svelte/icons/clock';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import Copy from 'lucide-svelte/icons/copy';
  import Bug from 'lucide-svelte/icons/bug';
  import VolumeIcon from '$components/common/VolumeIcon.svelte';
  import Images from 'lucide-svelte/icons/images';
  import Calendar from 'lucide-svelte/icons/calendar';
  import {
    memoryHeaderSettings,
    globalAudioSettings,
    showMemoryDebugBadges,
    replaceLocationPlaceholders,
    replaceTimeTagPlaceholders,
    formatMemoryLocation,
    formatMemoryTimeTag,
  } from '$lib/memoriesStore';
  import type { ExplorerMemory } from '$lib/types';

  const samplePreviewMemory: ExplorerMemory = {
    id: 'sample-preview',
    index: 0,
    takenAt: '2024-08-26T14:20:00.000Z',
    dateFormatted: '26 August 2024',
    dayNumber: '26',
    monthKey: '2024-08',
    year: 2024,
    month: 8,
    day: 26,
    timeFormatted: '14:20',
    isLate: true,
    lateDuration: '45m late',
    lateExact: '45 min late',
    lateInSeconds: 2700,
    retakeCounter: 0,
    caption: 'Great afternoon!',
    location: { latitude: 51.5136, longitude: -0.1365 },
    locationName: 'Soho, London, England',
    suburb: 'Soho',
    city: 'London',
    country: 'England',
    isVideo: false,
  };

  let statusMessage = '';
  let isSuccessStatus = true;
  let showResetModal = false;
  let checkingExiftool = false;
  let checkingHwInfo = false;

  async function handleCopyDiagnostics() {
    const hw = $hwInfoStore;
    const ff = $ffmpegInfo;
    const exif = $exiftoolStore;
    const text = [
      `=== BeReal Studio System Diagnostics ===`,
      `App Version: ${APP_VERSION}`,
      `CPU Cores: ${hw?.cpuCores ?? 'Unknown'} (Threads: ${hw?.parallelThreads ?? 'Unknown'})`,
      `Hardware GPU Acceleration: ${hw?.isGpuAccelerated ? 'Yes (' + hw.encoderName + ')' : 'CPU Only'}`,
      `FFmpeg Path: ${ff.path ?? 'Not Detected'}`,
      `ExifTool Path: ${exif.path ?? 'Using Native Rust EXIF Engine'}`,
      `Active Offline GeoDB: ${$offlineGeoDbStatus?.activeTier ?? 'Default Global Baseline'}`,
    ].join('\n');
    try {
      await navigator.clipboard.writeText(text);
      showToast('System diagnostics copied to clipboard!');
    } catch {
      showToast('Failed to copy to clipboard', false);
    }
  }

  async function handleClearThumbnailCache() {
    try {
      if (typeof window !== 'undefined') {
        localStorage.removeItem('bereal_studio_media_cache');
      }
      showToast('Media and explorer cache cleared successfully.');
    } catch (e) {
      showToast(`Clear cache failed: ${e}`, false);
    }
  }

  async function detectHwInfo(force = false) {
    if (!force && $hwInfoStore) return;
    checkingHwInfo = true;
    try {
      const info = await checkHardwareAcceleration();
      hwInfoStore.set(info);
    } catch (e) {
      console.warn('Failed to detect hardware acceleration:', e);
      hwInfoStore.set({
        gpuName: 'Standard Multi-Core CPU',
        encoderName: 'libx264 (Software CPU)',
        isGpuAccelerated: false,
        cpuCores: typeof navigator !== 'undefined' ? (navigator.hardwareConcurrency || 8) : 8,
        parallelThreads: typeof navigator !== 'undefined' ? (navigator.hardwareConcurrency || 8) : 8,
      });
    } finally {
      checkingHwInfo = false;
    }
  }

  async function detectExiftoolHandler(force = false) {
    if (!force && $exiftoolStore.checked) return;
    checkingExiftool = true;
    try {
      const p = await checkExiftool();
      exiftoolStore.set({ path: p, checked: true });
    } catch {
      exiftoolStore.set({ path: null, checked: true });
    } finally {
      checkingExiftool = false;
    }
  }

  onMount(() => {
    // Run diagnostics asynchronously without blocking component lifecycle
    Promise.allSettled([
      !$ffmpegInfo.checked ? detectFfmpeg() : Promise.resolve(null),
      !$exiftoolStore.checked ? detectExiftoolHandler() : Promise.resolve(),
      !$hwInfoStore ? detectHwInfo() : Promise.resolve(),
      (!$offlineGeoDbStatus?.tiers?.length)
        ? checkOfflineGeoDb()
            .then((dbStatus) => offlineGeoDbStatus.set(dbStatus))
            .catch((e) => console.warn('Failed to check offline geodb:', e))
        : Promise.resolve(),
    ]);
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
        <div class="head-actions-row">
          <button
            type="button"
            class="btn btn-secondary btn-sm"
            on:click={handleCopyDiagnostics}
            title="Copy full hardware, encoder, and dependency info to clipboard"
          >
            <Copy size={13} />
            <span>Copy Diagnostics</span>
          </button>
          <button
            type="button"
            class="btn btn-secondary btn-sm"
            on:click={handleClearThumbnailCache}
            title="Clear cached thumbnails & geocoded index"
          >
            <Trash2 size={13} />
            <span>Clear Cache</span>
          </button>
        </div>
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

      <div class="dependency-status" style="margin-top: 10px;">
        <div class="dep-icon" class:found={Boolean($exiftoolStore.path)}>
          {#if checkingExiftool}
            <RefreshCw size={18} class="animate-spin text-sky-400" />
          {:else if $exiftoolStore.path}
            <CheckCircle size={18} />
          {:else}
            <AlertTriangle size={18} />
          {/if}
        </div>

        <div class="dep-info">
          <div class="dep-name">
            <strong>ExifTool Metadata Sidecar</strong>
            {#if checkingExiftool}
              <span class="badge badge-info">Detecting...</span>
            {:else if $exiftoolStore.path}
              <span class="badge badge-success">Detected</span>
            {:else}
              <span class="badge badge-warning">Fallback to Native Rust EXIF</span>
            {/if}
          </div>
          <p class="dep-path text-secondary">
            {#if checkingExiftool}
              Scanning system for ExifTool binary...
            {:else}
              {$exiftoolStore.path || 'Using built-in native Rust EXIF/IPTC engine. Install ExifTool for enhanced multi-tag synchronization.'}
            {/if}
          </p>
        </div>
      </div>

      <!-- Hardware Acceleration Card (Statically rendered with loading indicator) -->
      <div class="dependency-status" style="margin-top: 10px;">
        <div class="dep-icon" class:found={Boolean($hwInfoStore?.isGpuAccelerated)}>
          {#if !$hwInfoStore && checkingHwInfo}
            <RefreshCw size={18} class="animate-spin text-sky-400" />
          {:else if $hwInfoStore}
            <Sparkles size={18} class={$hwInfoStore.isGpuAccelerated ? 'text-emerald-400' : 'text-sky-400'} />
          {:else}
            <RefreshCw size={18} class="animate-spin text-sky-400" />
          {/if}
        </div>

        <div class="dep-info">
          <div class="dep-name">
            <strong>Hardware Acceleration &amp; Parallelism</strong>
            {#if !$hwInfoStore}
              <span class="badge badge-info">Detecting...</span>
            {:else if $hwInfoStore.isGpuAccelerated}
              <span class="badge badge-success">GPU Accelerated</span>
            {:else}
              <span class="badge badge-info">Multi-Core CPU ({$hwInfoStore.cpuCores} Threads)</span>
            {/if}
          </div>
          <p class="dep-path text-secondary">
            {#if !$hwInfoStore}
              Probing GPU video encoder (NVENC / QuickSync / AMF / VideoToolbox)...
            {:else}
              Video Encoder: <strong class="text-white">{$hwInfoStore.encoderName}</strong> &bull; Rayon Parallelism: <strong class="text-white">{$hwInfoStore.parallelThreads} Active CPU Worker Threads</strong>
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
                  <Download size={16} class="text-muted" />
                {/if}
              </div>

              <div class="tier-setting-info">
                <div class="tier-setting-title-row">
                  <span class="font-bold text-white text-sm">{tier.name}</span>
                  {#if tier.isActive}
                    <span class="badge badge-success font-mono text-xs">Active Dataset</span>
                  {:else if tier.isInstalled}
                    <span class="badge badge-info font-mono text-xs">Installed</span>
                  {/if}
                </div>
                <span class="text-xs text-secondary">{tier.subtitle} &bull; {tier.approxCities}</span>
                <span class="text-xs text-muted font-mono">
                  {#if tier.isInstalled}
                    Disk size: {(tier.fileSizeBytes / (1024 * 1024)).toFixed(2)} MB &bull; {tier.cityCount.toLocaleString()} locations loaded
                  {:else}
                    Download: ~{tier.approxDownloadMb} MB (uncompressed JSON index)
                  {/if}
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

    <!-- 4. Memories & Feed Display Settings -->
    <div class="card section-card">
      <div class="card-head">
        <div class="head-title">
          <Eye size={18} class="text-sky-400" />
          <h2 class="title-sm">4. Memories &amp; Feed Header Customization</h2>
        </div>
        <span class="badge badge-sky font-mono text-xs">Feed View</span>
      </div>

      <p class="text-secondary text-desc">
        Customize how location and time information are formatted directly above photos in the Memories viewer, matching authentic BeReal design.
      </p>

      <div class="settings-grid-2col">
        <!-- Location Display Toggle & Rich Format Selector -->
        <div class="setting-item-box">
          <Toggle
            label="Location Display"
            description="Show reverse-geocoded location directly above photos in the feed"
            icon={MapPin}
            accentColor="emerald"
            bind:checked={$memoryHeaderSettings.showLocation}
          />

          {#if $memoryHeaderSettings.showLocation}
            <div class="format-choice-list">
              <label class="format-choice-card" class:selected={$memoryHeaderSettings.locationFormat === 'city_country'}>
                <input type="radio" name="locFormat" value="city_country" bind:group={$memoryHeaderSettings.locationFormat} />
                <div class="choice-text">
                  <span class="choice-title">City, Country</span>
                  <span class="choice-sample">London, England</span>
                </div>
              </label>

              <label class="format-choice-card" class:selected={$memoryHeaderSettings.locationFormat === 'suburb_city_country'}>
                <input type="radio" name="locFormat" value="suburb_city_country" bind:group={$memoryHeaderSettings.locationFormat} />
                <div class="choice-text">
                  <span class="choice-title">Area, City, Country</span>
                  <span class="choice-sample">Soho, London, England</span>
                </div>
              </label>

              <label class="format-choice-card" class:selected={$memoryHeaderSettings.locationFormat === 'suburb_city'}>
                <input type="radio" name="locFormat" value="suburb_city" bind:group={$memoryHeaderSettings.locationFormat} />
                <div class="choice-text">
                  <span class="choice-title">Area, City</span>
                  <span class="choice-sample">Soho, London</span>
                </div>
              </label>

              <label class="format-choice-card" class:selected={$memoryHeaderSettings.locationFormat === 'city_only'}>
                <input type="radio" name="locFormat" value="city_only" bind:group={$memoryHeaderSettings.locationFormat} />
                <div class="choice-text">
                  <span class="choice-title">City Only</span>
                  <span class="choice-sample">London</span>
                </div>
              </label>

              <label class="format-choice-card" class:selected={$memoryHeaderSettings.locationFormat === 'custom'}>
                <input type="radio" name="locFormat" value="custom" bind:group={$memoryHeaderSettings.locationFormat} />
                <div class="choice-text">
                  <span class="choice-title">Custom Location Text</span>
                  <span class="choice-sample">Display custom text everywhere</span>
                </div>
              </label>

              {#if $memoryHeaderSettings.locationFormat === 'custom'}
                <div class="custom-header-input-wrap">
                  <input
                    type="text"
                    class="input custom-text-field"
                    placeholder="e.g. {'{suburb}'}, {'{city}'} or {'{city}'}, {'{country}'}"
                    bind:value={$memoryHeaderSettings.customLocationText}
                  />
                  <div class="placeholder-chips-row">
                    <span class="placeholder-label">Insert:</span>
                    <button type="button" class="placeholder-chip" on:click={() => ($memoryHeaderSettings.customLocationText = ($memoryHeaderSettings.customLocationText || '') + '{suburb}')}>{'{suburb}'}</button>
                    <button type="button" class="placeholder-chip" on:click={() => ($memoryHeaderSettings.customLocationText = ($memoryHeaderSettings.customLocationText || '') + '{city}')}>{'{city}'}</button>
                    <button type="button" class="placeholder-chip" on:click={() => ($memoryHeaderSettings.customLocationText = ($memoryHeaderSettings.customLocationText || '') + '{country}')}>{'{country}'}</button>
                    <button type="button" class="placeholder-chip" on:click={() => ($memoryHeaderSettings.customLocationText = ($memoryHeaderSettings.customLocationText || '') + '{location}')}>{'{location}'}</button>
                    <button type="button" class="placeholder-chip" on:click={() => ($memoryHeaderSettings.customLocationText = ($memoryHeaderSettings.customLocationText || '') + '{lat}')}>{'{lat}'}</button>
                    <button type="button" class="placeholder-chip" on:click={() => ($memoryHeaderSettings.customLocationText = ($memoryHeaderSettings.customLocationText || '') + '{lng}')}>{'{lng}'}</button>
                  </div>
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- Time & Late Tag Toggle & Rich Format Selector -->
        <div class="setting-item-box">
          <Toggle
            label="Time / Date Subtitle"
            description="Show timestamp, progressive date, or late duration next to location"
            icon={Clock}
            accentColor="yellow"
            bind:checked={$memoryHeaderSettings.showTimeTag}
          />

          {#if $memoryHeaderSettings.showTimeTag}
            <div class="format-choice-list">
              <label class="format-choice-card" class:selected={$memoryHeaderSettings.timeTagFormat === 'late_duration'}>
                <input type="radio" name="timeFormat" value="late_duration" bind:group={$memoryHeaderSettings.timeTagFormat} />
                <div class="choice-text">
                  <span class="choice-title">Smart Progressive (Recommended)</span>
                  <span class="choice-sample">26 Aug • 14:20 (or full date for past years)</span>
                </div>
              </label>

              <label class="format-choice-card" class:selected={$memoryHeaderSettings.timeTagFormat === 'datetime'}>
                <input type="radio" name="timeFormat" value="datetime" bind:group={$memoryHeaderSettings.timeTagFormat} />
                <div class="choice-text">
                  <span class="choice-title">Date &amp; Time</span>
                  <span class="choice-sample">26 Aug • 14:20</span>
                </div>
              </label>

              <label class="format-choice-card" class:selected={$memoryHeaderSettings.timeTagFormat === 'date_only'}>
                <input type="radio" name="timeFormat" value="date_only" bind:group={$memoryHeaderSettings.timeTagFormat} />
                <div class="choice-text">
                  <span class="choice-title">Date Only</span>
                  <span class="choice-sample">26 Aug (or 26 Aug 2024)</span>
                </div>
              </label>

              <label class="format-choice-card" class:selected={$memoryHeaderSettings.timeTagFormat === 'time_only'}>
                <input type="radio" name="timeFormat" value="time_only" bind:group={$memoryHeaderSettings.timeTagFormat} />
                <div class="choice-text">
                  <span class="choice-title">Time Only</span>
                  <span class="choice-sample">14:20</span>
                </div>
              </label>

              <label class="format-choice-card" class:selected={$memoryHeaderSettings.timeTagFormat === 'custom'}>
                <input type="radio" name="timeFormat" value="custom" bind:group={$memoryHeaderSettings.timeTagFormat} />
                <div class="choice-text">
                  <span class="choice-title">Custom Subtitle Text</span>
                  <span class="choice-sample">Use placeholders like {'{late}'} • {'{date}'}</span>
                </div>
              </label>

              {#if $memoryHeaderSettings.timeTagFormat === 'custom'}
                <div class="custom-header-input-wrap">
                  <input
                    type="text"
                    class="input custom-text-field"
                    placeholder="e.g. {'{late}'} • {'{date}'} or {'{time}'} • {'{date}'}"
                    bind:value={$memoryHeaderSettings.customTimeTagText}
                  />
                  <div class="placeholder-chips-row">
                    <span class="placeholder-label">Insert:</span>
                    <button type="button" class="placeholder-chip" on:click={() => ($memoryHeaderSettings.customTimeTagText = ($memoryHeaderSettings.customTimeTagText || '') + '{date}')}>{'{date}'}</button>
                    <button type="button" class="placeholder-chip" on:click={() => ($memoryHeaderSettings.customTimeTagText = ($memoryHeaderSettings.customTimeTagText || '') + '{time}')}>{'{time}'}</button>
                    <button type="button" class="placeholder-chip" on:click={() => ($memoryHeaderSettings.customTimeTagText = ($memoryHeaderSettings.customTimeTagText || '') + '{late}')}>{'{late}'}</button>
                    <button type="button" class="placeholder-chip" on:click={() => ($memoryHeaderSettings.customTimeTagText = ($memoryHeaderSettings.customTimeTagText || '') + '{late_exact}')}>{'{late_exact}'}</button>
                    <button type="button" class="placeholder-chip" on:click={() => ($memoryHeaderSettings.customTimeTagText = ($memoryHeaderSettings.customTimeTagText || '') + '{full_date}')}>{'{full_date}'}</button>
                    <button type="button" class="placeholder-chip" on:click={() => ($memoryHeaderSettings.customTimeTagText = ($memoryHeaderSettings.customTimeTagText || '') + '{year}')}>{'{year}'}</button>
                  </div>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>

      <!-- Additional Late Tag & Pill Visibility Toggles -->
      <div class="setting-item-box" style="margin-top: 16px;">
        <h3 class="title-xs" style="margin-bottom: 10px; color: var(--text-main); font-weight: 600;">Late Submission Indicators</h3>
        <div style="display: flex; flex-direction: column; gap: 10px;">
          <Toggle
            label="Append Late Duration in Post Header"
            description="Display how late you were next to the timestamp (e.g. • 45m late) matching standard theme text"
            icon={Clock}
            accentColor="yellow"
            bind:checked={$memoryHeaderSettings.showLateAddition}
          />
          <Toggle
            label="Show Late Pills on Memories Grid"
            description="Display the late duration badge on memory thumbnail cards in the grid view"
            icon={Images}
            accentColor="yellow"
            bind:checked={$memoryHeaderSettings.showLatePillsInGrid}
          />
          <Toggle
            label="Show Late Pills in Calendar View"
            description="Display the late submission pill directly inside calendar day cells"
            icon={Calendar}
            accentColor="yellow"
            bind:checked={$memoryHeaderSettings.showLatePillsInCalendar}
          />
        </div>
      </div>

      <!-- Live Header Preview -->
      <div class="header-live-preview-box">
        <span class="preview-box-label">LIVE HEADER PREVIEW</span>
        <div class="preview-header-row">
          <div class="preview-avatar">
            <span>N</span>
          </div>
          <div class="preview-text-col">
            <span class="preview-username">nottoxel</span>
            <div class="preview-subtitle">
              {#if $memoryHeaderSettings.showLocation}
                <span class="preview-loc">
                  {formatMemoryLocation(samplePreviewMemory, $memoryHeaderSettings) || 'London, England'}
                </span>
              {/if}
              {#if $memoryHeaderSettings.showLocation && $memoryHeaderSettings.showTimeTag}
                <span class="preview-bullet">•</span>
              {/if}
              {#if $memoryHeaderSettings.showTimeTag}
                <span class="preview-time">
                  {formatMemoryTimeTag(samplePreviewMemory, $memoryHeaderSettings) || '14:20'}
                </span>
              {/if}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 5. Memory Audio & Video Playback Settings Overhaul -->
    <div class="card section-card">
      <div class="head-title">
        <VolumeIcon size={18} className="text-sky-400" />
        <h2 class="title-sm">5. Memory Audio &amp; Video Playback Defaults</h2>
      </div>

      <p class="text-secondary text-desc">
        Configure default autoplay sound behavior and volume faders for BeReal videos and BTS (Behind-The-Scenes) micro-clips in the memories viewer.
      </p>

      <div class="settings-grid-2col">
        <!-- Column 1: Autoplay Sound Mode -->
        <div class="setting-item-box">
          <Toggle
            label="Mute Video Autoplay"
            description="Start videos and BTS micro-clips in silent mode when hovered"
            icon={VolumeIcon}
            accentColor="cyan"
            bind:checked={$globalAudioSettings.defaultMuted}
          />

          <div class="format-choice-list mt-3">
            <label class="format-choice-card" class:selected={$globalAudioSettings.defaultMuted}>
              <input
                type="radio"
                name="muteMode"
                value={true}
                checked={$globalAudioSettings.defaultMuted}
                on:change={() => ($globalAudioSettings.defaultMuted = true)}
              />
              <div class="choice-text">
                <span class="choice-title">Always Muted (Recommended)</span>
                <span class="choice-sample">Videos play quietly on hover with a click-to-unmute audio pill</span>
              </div>
            </label>

            <label class="format-choice-card" class:selected={!$globalAudioSettings.defaultMuted}>
              <input
                type="radio"
                name="muteMode"
                value={false}
                checked={!$globalAudioSettings.defaultMuted}
                on:change={() => ($globalAudioSettings.defaultMuted = false)}
              />
              <div class="choice-text">
                <span class="choice-title">Unmuted on Hover</span>
                <span class="choice-sample">Automatically plays full audio as soon as your cursor hovers over the card</span>
              </div>
            </label>
          </div>
        </div>

        <!-- Column 2: Master Volume with Vertical Fader -->
        <div class="setting-item-box volume-fader-box">
          <div class="volume-card-header">
            <div class="volume-title-group">
              <span class="volume-title">Master Playback Volume</span>
              <span class="volume-subtitle">Adjust output gain for all video files &amp; BTS audio</span>
            </div>
            <span class="volume-level-badge font-mono">
              {Math.round($globalAudioSettings.volume * 100)}%
            </span>
          </div>

          <div class="vertical-fader-container">
            <!-- Vertical Range Slider -->
            <div class="vertical-slider-track-wrap">
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                class="vertical-range-fader"
                bind:value={$globalAudioSettings.volume}
                aria-label="Master audio volume level"
              />
            </div>

            <!-- Visual Scale Labels & Quick Jump Buttons -->
            <div class="fader-scale-column">
              <button
                type="button"
                class="fader-level-btn"
                class:active={$globalAudioSettings.volume >= 0.95}
                on:click={() => ($globalAudioSettings.volume = 1.0)}
              >
                <span>100%</span>
                <span class="level-desc">Max Volume</span>
              </button>

              <button
                type="button"
                class="fader-level-btn"
                class:active={$globalAudioSettings.volume >= 0.75 && $globalAudioSettings.volume < 0.95}
                on:click={() => ($globalAudioSettings.volume = 0.8)}
              >
                <span>80%</span>
                <span class="level-desc">Default</span>
              </button>

              <button
                type="button"
                class="fader-level-btn"
                class:active={$globalAudioSettings.volume >= 0.45 && $globalAudioSettings.volume < 0.75}
                on:click={() => ($globalAudioSettings.volume = 0.5)}
              >
                <span>50%</span>
                <span class="level-desc">Half</span>
              </button>

              <button
                type="button"
                class="fader-level-btn"
                class:active={$globalAudioSettings.volume > 0.05 && $globalAudioSettings.volume < 0.45}
                on:click={() => ($globalAudioSettings.volume = 0.25)}
              >
                <span>25%</span>
                <span class="level-desc">Quiet</span>
              </button>

              <button
                type="button"
                class="fader-level-btn fader-mute-btn"
                class:active={$globalAudioSettings.volume <= 0.05}
                on:click={() => ($globalAudioSettings.volume = 0.0)}
              >
                <span>0%</span>
                <span class="level-desc">Mute</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 6. Local Storage & Privacy Management -->
    <div class="card section-card">
      <div class="head-title">
        <ShieldCheck size={18} class="text-emerald-400" />
        <h2 class="title-sm">6. Local Storage &amp; Factory Reset</h2>
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

    {#if isDev}
      <!-- 7. Developer Tools & Diagnostics (Dev Mode Only) -->
      <div class="card section-card">
        <div class="head-title">
          <Bug size={18} class="text-amber-400" />
          <h2 class="title-sm">7. Developer Tools &amp; Metadata Inspector</h2>
        </div>

        <p class="text-secondary text-desc">
          Feature flags and advanced metadata inspection tools for troubleshooting and dataset analysis. (Only visible in development environment).
        </p>

        <div class="settings-grid-2col">
          <div class="setting-item-box">
            <Toggle
              label="Memory Timing &amp; Metadata Inspector"
              description="Display interactive badges on memory cards with live BeReal moment alerts, timing offsets, and JSON inspector tooltips"
              icon={Bug}
              accentColor="cyan"
              bind:checked={$showMemoryDebugBadges}
            />
          </div>
        </div>
      </div>
    {/if}
  </div>

  <Modal bind:open={showResetModal} title="Reset All Settings?" maxWidth="440px">
    <svelte:fragment slot="title">
      <div class="modal-head">
        <AlertTriangle size={20} class="text-rose-400" />
        <h3 class="title-sm font-bold text-white">Reset All Settings?</h3>
      </div>
    </svelte:fragment>

    <p class="text-secondary text-xs">
      This will restore all Photo Processing and Recap Video preferences, formatting rules, quality thresholds, and directory paths to factory defaults.
    </p>

    <svelte:fragment slot="footer">
      <button type="button" class="btn btn-secondary btn-sm" on:click={() => (showResetModal = false)}>
        Cancel
      </button>
      <button type="button" class="btn btn-danger btn-sm" on:click={handleConfirmReset}>
        <RotateCcw size={13} />
        <span>Confirm Reset</span>
      </button>
    </svelte:fragment>
  </Modal>
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
    gap: 12px;
    flex-wrap: wrap;
  }

  .head-actions-row {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
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

  /* Memories Header Settings Styles */
  .settings-grid-2col {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 16px;
  }

  .setting-item-box {
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: #09090e;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 14px;
  }

  .format-choice-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 4px;
  }

  .format-choice-card {
    position: relative;
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 10px;
    background: #14141d;
    border: 1.5px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .format-choice-card input {
    margin-top: 3px;
    accent-color: #38bdf8;
  }

  .format-choice-card:hover {
    background: #1c1c28;
    border-color: var(--border-medium);
  }

  .format-choice-card.selected {
    background: rgba(56, 189, 248, 0.1);
    border-color: #38bdf8;
  }

  .choice-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .choice-title {
    font-size: 12px;
    font-weight: 700;
    color: #ffffff;
  }

  .choice-sample {
    font-size: 11px;
    color: var(--text-secondary);
    font-family: var(--font-mono);
  }

  .custom-header-input-wrap {
    margin-top: 4px;
    padding: 0 4px;
  }

  .custom-text-field {
    width: 100%;
    padding: 6px 10px;
    font-size: 12px;
    background: #14141d;
    border: 1px solid rgba(56, 189, 248, 0.4);
    border-radius: var(--radius-sm);
    color: #ffffff;
  }

  .custom-text-field:focus {
    border-color: #38bdf8;
    box-shadow: 0 0 0 2px rgba(56, 189, 248, 0.2);
    outline: none;
  }

  .placeholder-chips-row {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
    margin-top: 6px;
  }

  .placeholder-label {
    font-size: 10.5px;
    font-weight: 600;
    color: var(--text-muted);
  }

  .placeholder-chip {
    padding: 2px 6px;
    background: #181824;
    border: 1px solid rgba(56, 189, 248, 0.25);
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: #38bdf8;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .placeholder-chip:hover {
    background: rgba(56, 189, 248, 0.15);
    border-color: #38bdf8;
    transform: scale(1.04);
  }

  .header-live-preview-box {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: #09090d;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 14px 16px;
    position: relative;
  }

  .preview-box-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--text-muted);
  }

  .preview-header-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    background: #000000;
    border-radius: var(--radius-md);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  .preview-avatar {
    width: 36px;
    height: 36px;
    border-radius: 999px;
    background: linear-gradient(135deg, #0ea5e9, #6366f1);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #ffffff;
    font-weight: 700;
    font-size: 14px;
  }

  .preview-text-col {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .preview-username {
    font-size: 14.5px;
    font-weight: 700;
    color: #ffffff;
    letter-spacing: -0.01em;
  }

  .preview-subtitle {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12.5px;
    color: #a1a1aa;
    font-weight: 500;
  }

  .preview-loc {
    color: #d4d4d8;
  }

  .preview-bullet {
    color: #71717a;
    font-size: 10px;
  }

  .preview-time {
    color: #a1a1aa;
  }

  /* Audio & Video Playback Section Overhaul */
  .volume-fader-box {
    display: flex;
    flex-direction: column;
    gap: 14px;
    background: #0d0d12;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 16px 18px;
  }

  .volume-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding-bottom: 10px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .volume-title-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .volume-title {
    font-size: 13.5px;
    font-weight: 700;
    color: #ffffff;
  }

  .volume-subtitle {
    font-size: 11.5px;
    color: var(--text-secondary);
  }

  .volume-level-badge {
    font-size: 13px;
    font-weight: 800;
    color: #38bdf8;
    background: rgba(56, 189, 248, 0.12);
    border: 1px solid rgba(56, 189, 248, 0.3);
    padding: 3px 10px;
    border-radius: var(--radius-full);
    letter-spacing: -0.02em;
  }

  .vertical-fader-container {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 10px 6px 6px 6px;
    min-height: 180px;
  }

  .vertical-slider-track-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 170px;
    width: 44px;
    background: #14141d;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 22px;
    padding: 12px 0;
    box-shadow: inset 0 2px 8px rgba(0, 0, 0, 0.6);
  }

  .vertical-range-fader {
    writing-mode: vertical-lr;
    direction: rtl;
    width: 8px;
    height: 140px;
    appearance: none;
    background: #0a0a0f;
    border-radius: 999px;
    outline: none;
    cursor: pointer;
    margin: 0;
  }

  .vertical-range-fader::-webkit-slider-runnable-track {
    width: 8px;
    height: 140px;
    background: linear-gradient(to top, rgba(56, 189, 248, 0.2), #38bdf8);
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  .vertical-range-fader::-webkit-slider-thumb {
    appearance: none;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: #ffffff;
    border: 3px solid #38bdf8;
    box-shadow: 0 0 12px rgba(56, 189, 248, 0.8), 0 2px 6px rgba(0, 0, 0, 0.8);
    cursor: grab;
    transition: transform 0.12s ease, box-shadow 0.12s ease;
    margin-left: -8px;
  }

  .vertical-range-fader::-webkit-slider-thumb:active {
    cursor: grabbing;
    transform: scale(1.15);
    box-shadow: 0 0 18px rgba(56, 189, 248, 1), 0 2px 8px rgba(0, 0, 0, 0.9);
  }

  .fader-scale-column {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    height: 170px;
    flex: 1;
    gap: 4px;
  }

  .fader-level-btn {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: #12121a;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 11.5px;
    font-weight: 600;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .fader-level-btn:hover {
    background: #181824;
    border-color: rgba(255, 255, 255, 0.2);
    color: #ffffff;
  }

  .fader-level-btn.active {
    background: rgba(56, 189, 248, 0.14);
    border-color: rgba(56, 189, 248, 0.4);
    color: #38bdf8;
    font-weight: 700;
  }

  .fader-level-btn .level-desc {
    font-size: 10.5px;
    font-weight: 400;
    color: var(--text-muted);
  }

  .fader-level-btn.active .level-desc {
    color: #7dd3fc;
  }
</style>
