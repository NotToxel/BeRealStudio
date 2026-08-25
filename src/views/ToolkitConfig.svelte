<script lang="ts">
  import { onMount } from 'svelte';
  import {
    currentView,
    toolkitConfig,
    archiveMetadata,
    lastScannedArchivePath,
    isProcessing,
    progressState,
    liveLogs,
    activeError,
  } from '$lib/stores';
  import {
    scanArchive,
    startToolkit,
    onToolkitProgress,
    onToolkitLog,
  } from '$lib/tauri';
  import FolderArchive from 'lucide-svelte/icons/folder-archive';
  import Sliders from 'lucide-svelte/icons/sliders';
  import Layers from 'lucide-svelte/icons/layers';
  import Play from 'lucide-svelte/icons/play';
  import ArrowLeft from 'lucide-svelte/icons/arrow-left';
  import Eye from 'lucide-svelte/icons/eye';
  import Camera from 'lucide-svelte/icons/camera';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import Repeat from 'lucide-svelte/icons/repeat';
  import CheckCircle from 'lucide-svelte/icons/circle-check';
  import AlertTriangle from 'lucide-svelte/icons/triangle-alert';
  import XCircle from 'lucide-svelte/icons/circle-x';
  import User from 'lucide-svelte/icons/circle-user';
  import FileCode from 'lucide-svelte/icons/file-code';
  import HelpCircle from 'lucide-svelte/icons/circle-help';
  import ChevronDown from 'lucide-svelte/icons/chevron-down';
  import ChevronUp from 'lucide-svelte/icons/chevron-up';
  import RefreshCw from 'lucide-svelte/icons/refresh-cw';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import MessageSquare from 'lucide-svelte/icons/message-square';
  import Clapperboard from 'lucide-svelte/icons/clapperboard';
  import FilePicker from '$components/FilePicker.svelte';
  import Toggle from '$components/Toggle.svelte';
  import Stepper from '$components/Stepper.svelte';
  import DateRangePicker from '$components/DateRangePicker.svelte';

  let scanning = false;
  let previewTab: 'standard' | 'reversed' = 'standard';
  let showMissingDetails = false;

  function formatDisplayDate(dStr?: string): string {
    if (!dStr) return '—';
    try {
      const d = new Date(dStr);
      if (isNaN(d.getTime())) return dStr;
      return d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' });
    } catch {
      return dStr;
    }
  }

  async function handleArchiveChange(path: string) {
    if (!path) return;
    // Skip re-scan if this path was already successfully scanned
    if (path === $lastScannedArchivePath && $archiveMetadata && $archiveMetadata.isValid) {
      return;
    }
    $lastScannedArchivePath = path;
    scanning = true;
    showMissingDetails = false;
    try {
      const meta = await scanArchive(path);
      archiveMetadata.set(meta);

      if (!$toolkitConfig.outputPath) {
        $toolkitConfig.outputPath = path.endsWith('.zip')
          ? path.replace(/\.zip$/i, '_processed')
          : `${path}_processed`;
      }
    } catch (e: any) {
      archiveMetadata.set(null);
      activeError.set({
        title: 'Archive Scan Failed',
        message: 'Could not parse the provided BeReal data archive.',
        details: String(e),
      });
    } finally {
      scanning = false;
    }
  }

  $: if ($toolkitConfig.inputPath && ($toolkitConfig.inputPath !== $lastScannedArchivePath || !$archiveMetadata)) {
    handleArchiveChange($toolkitConfig.inputPath);
  }

  let missingInputPath = false;
  let missingOutputPath = false;

  $: isArchiveValid = Boolean($archiveMetadata && $archiveMetadata.isValid && $archiveMetadata.entryCount > 0);
  $: isConfigValid = Boolean($toolkitConfig.inputPath && $toolkitConfig.outputPath && isArchiveValid && !scanning);

  async function handleStart() {
    missingInputPath = !$toolkitConfig.inputPath;
    missingOutputPath = !$toolkitConfig.outputPath;

    if (!isConfigValid) {
      if (!isArchiveValid && $archiveMetadata) {
        activeError.set({
          title: 'Invalid BeReal Export',
          message: 'The selected archive cannot be processed because it is missing essential required BeReal data.',
          details: $archiveMetadata.validationErrors.join('\n') || 'No valid post memories found in archive.',
        });
        return;
      }

      const firstMissingId = missingInputPath
        ? 'toolkit-input-path'
        : 'toolkit-output-path';

      const el = document.getElementById(firstMissingId);
      if (el) {
        el.scrollIntoView({ behavior: 'smooth', block: 'center' });
        const inputEl = el.querySelector('input');
        if (inputEl) inputEl.focus();
      }
      return;
    }

    liveLogs.set([]);
    progressState.set({
      stage: 'Scanning',
      current: 0,
      total: 0,
      percentage: 0,
    });
    isProcessing.set(true);
    currentView.set('processing');

    const unlistenProgress = await onToolkitProgress((p) => {
      progressState.set(p);
      if (p.stage === 'Complete') {
        isProcessing.set(false);
        currentView.set('complete');
      }
    });

    const unlistenLog = await onToolkitLog((l) => {
      liveLogs.update((logs) => [...logs, l]);
    });

    try {
      await startToolkit($toolkitConfig);
    } catch (e: any) {
      isProcessing.set(false);
      activeError.set({
        title: 'Processing Failed',
        message: 'An error occurred during BeReal photo processing.',
        details: String(e),
      });
    }
  }
</script>

<div class="config-view">
  <!-- Top Bar -->
  <div class="top-nav">
    <button type="button" class="btn btn-ghost btn-sm" on:click={() => currentView.set('home')}>
      <ArrowLeft size={14} />
      <span>Back to Home</span>
    </button>
    <div class="header-titles">
      <h1 class="title-md font-bold">Photo Processing Suite</h1>
      <span class="badge badge-yellow">Live Preview</span>
    </div>
  </div>

  <div class="main-layout">
    <!-- Left Column: Settings Form -->
    <div class="form-column">
      <!-- 1. Input / Output Targets -->
      <div class="card section-card">
        <div class="section-title-row">
          <FolderArchive size={18} class="text-amber-400" />
          <h2 class="title-sm">1. Select Archive &amp; Output</h2>
        </div>

        <FilePicker
          id="toolkit-input-path"
          label="BeReal Archive (ZIP File or Extracted Folder)"
          placeholder="Select your BeReal GDPR .zip or unzipped folder..."
          isDirectory={false}
          allowBoth={true}
          fileExtensions={['zip']}
          dialogTitle="Select BeReal Data Archive"
          required={true}
          isMissing={missingInputPath}
          bind:value={$toolkitConfig.inputPath}
        />

        <FilePicker
          id="toolkit-output-path"
          label="Output Destination Directory"
          placeholder="Select output folder for processed photos..."
          isDirectory={true}
          dialogTitle="Select Output Directory"
          required={true}
          isMissing={missingOutputPath}
          bind:value={$toolkitConfig.outputPath}
        />

        {#if scanning}
          <div class="status-box-scanning">
            <span class="spinner-icon flex items-center">
              <RefreshCw size={16} class="text-amber-400" />
            </span>
            <span>Scanning archive and verifying BeReal export integrity...</span>
          </div>
        {:else if $archiveMetadata}
          {#if $archiveMetadata.isValid}
            <!-- Valid Archive Diagnostic Card -->
            <div class="diagnostic-card valid-card">
              <div class="diag-header">
                <div class="badge-row">
                  <span class="status-pill status-pill-success">
                    <CheckCircle size={13} />
                    <span>Valid BeReal Archive</span>
                  </span>
                  <span class="type-pill">
                    {$archiveMetadata.archiveType === 'Zip' ? 'ZIP Archive' : 'Extracted Folder'}
                  </span>
                </div>
                {#if $archiveMetadata.userName}
                  <div class="user-pill">
                    <User size={13} class="text-sky-400" />
                    <span class="font-medium">@{$archiveMetadata.userName}</span>
                    {#if $archiveMetadata.userFullname}
                      <span class="text-muted text-xs">({$archiveMetadata.userFullname})</span>
                    {/if}
                  </div>
                {/if}
              </div>

              <div class="metadata-banner">
                <div class="meta-item">
                  <div class="meta-icon-label">
                    <Camera size={12} class="text-amber-400" />
                    <span class="meta-label">Total Memories</span>
                  </div>
                  <strong class="text-amber-400 font-mono">{$archiveMetadata.validPostCount}</strong>
                </div>

                <div class="meta-item">
                  <div class="meta-icon-label">
                    <CheckCircle size={12} class="text-emerald-400" />
                    <span class="meta-label">Primary / Selfie</span>
                  </div>
                  <strong class="text-emerald-400 font-mono">
                    {$archiveMetadata.primaryPhotoCount} / {$archiveMetadata.secondaryPhotoCount}
                  </strong>
                </div>

                <div class="meta-item">
                  <div class="meta-icon-label">
                    <Clapperboard size={12} class="text-sky-400" />
                    <span class="meta-label">BTS / Videos</span>
                  </div>
                  <strong class="text-sky-400 font-mono">
                    {$archiveMetadata.btsCount} BTS · {$archiveMetadata.primaryVideoCount + $archiveMetadata.secondaryVideoCount} vids
                  </strong>
                </div>

                <div class="meta-item">
                  <div class="meta-icon-label">
                    <MapPin size={12} class="text-purple-400" />
                    <span class="meta-label">With GPS / Captions</span>
                  </div>
                  <strong class="text-purple-300 font-mono">
                    {$archiveMetadata.withLocationCount} GPS · {$archiveMetadata.withCaptionCount} caps
                  </strong>
                </div>

                <!-- Span info full row -->
                <div class="meta-span-row">
                  <span class="meta-label">Date Span:</span>
                  <span class="meta-span-val font-mono">
                    {formatDisplayDate($archiveMetadata.earliestDate)} &rarr; {formatDisplayDate($archiveMetadata.latestDate)}
                  </span>
                  {#if $archiveMetadata.retakeStats}
                    <span class="meta-retake-pill">
                      Avg {$archiveMetadata.retakeStats.avg.toFixed(1)} retakes (max {$archiveMetadata.retakeStats.max})
                    </span>
                  {/if}
                </div>
              </div>

              {#if $archiveMetadata.warnings && $archiveMetadata.warnings.length > 0}
                <div class="warning-callout">
                  <button type="button" class="warning-header-btn" on:click={() => showMissingDetails = !showMissingDetails}>
                    <div class="warning-title">
                      <AlertTriangle size={14} class="text-amber-400" />
                      <span>{$archiveMetadata.warnings[0]}</span>
                    </div>
                    {#if $archiveMetadata.missingFilesSample && $archiveMetadata.missingFilesSample.length > 0}
                      <span class="btn-toggle-details">
                        <span>{showMissingDetails ? 'Hide details' : 'Show missing files'}</span>
                        {#if showMissingDetails}
                          <ChevronUp size={12} />
                        {:else}
                          <ChevronDown size={12} />
                        {/if}
                      </span>
                    {/if}
                  </button>
                  {#if showMissingDetails && $archiveMetadata.missingFilesSample && $archiveMetadata.missingFilesSample.length > 0}
                    <div class="missing-files-list">
                      <div class="missing-files-title">Sample missing files &amp; associated metadata:</div>
                      {#each $archiveMetadata.missingFilesSample as sample}
                        <div class="missing-file-item font-mono">
                          {#if sample.date}
                            <span class="mf-date">[{sample.date}]</span>
                          {/if}
                          {#if sample.cameraType}
                            <span class="mf-cam">[{sample.cameraType}]</span>
                          {/if}
                          <span class="mf-path">{sample.path}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {:else}
            <!-- Invalid Archive Diagnostic Card -->
            <div class="diagnostic-card invalid-card">
              <div class="diag-header">
                <div class="badge-row">
                  <span class="status-pill status-pill-error">
                    <XCircle size={13} />
                    <span>Invalid BeReal Export</span>
                  </span>
                  <span class="type-pill">
                    {$archiveMetadata.archiveType === 'Zip' ? 'ZIP Archive' : 'Extracted Folder'}
                  </span>
                </div>
              </div>

              <div class="error-callout-body">
                <div class="error-lead">
                  This archive cannot be processed because it is missing essential required BeReal export files:
                </div>
                <ul class="error-reasons-list">
                  {#each $archiveMetadata.validationErrors as errorMsg}
                    <li>
                      <AlertTriangle size={13} class="text-red-400 flex-shrink-0" />
                      <span>{errorMsg}</span>
                    </li>
                  {/each}
                </ul>

                <div class="gdpr-guide-tip">
                  <HelpCircle size={14} class="text-sky-400 flex-shrink-0" />
                  <div>
                    <strong>How to download your authentic archive from BeReal:</strong><br />
                    Profile icon &rarr; <strong>Help</strong> &rarr; <strong>Contact Us</strong> &rarr; <strong>Ask a Question</strong> &rarr; <strong>Troubleshooting</strong> &rarr; <strong>Other</strong> &rarr; <strong>Contact Us</strong> &rarr; Topic: <strong>"I'd like to request a copy of my data"</strong> (enter &ge;10 characters in message and submit).
                  </div>
                </div>
              </div>
            </div>
          {/if}
        {/if}
      </div>

      <!-- 2. Date Range Filtering -->
      {#if $archiveMetadata && $archiveMetadata.isValid && $archiveMetadata.monthlyHistogram.length > 0}
        <DateRangePicker
          histogram={$archiveMetadata.monthlyHistogram}
          minDate={$archiveMetadata.earliestDate}
          maxDate={$archiveMetadata.latestDate}
          bind:startDate={$toolkitConfig.dateRangeStart}
          bind:endDate={$toolkitConfig.dateRangeEnd}
          totalCount={$archiveMetadata.entryCount}
        />
      {/if}

      <!-- 3. Image Formatting & Conversion -->
      <div class="card section-card">
        <div class="section-title-row">
          <Sliders size={18} class="text-sky-400" />
          <h2 class="title-sm">2. Image Format &amp; Metadata</h2>
        </div>

        <div class="options-grid">
          <div class="field-group format-field-group">
            <span class="label">Target Format</span>
            <div class="custom-format-selector">
              <button
                type="button"
                class="format-pill"
                class:active={$toolkitConfig.convertFormat === 'Jpeg'}
                on:click={() => ($toolkitConfig.convertFormat = 'Jpeg')}
              >
                <span class="pill-name">JPEG</span>
                <span class="pill-ext">.jpg</span>
              </button>
              <button
                type="button"
                class="format-pill"
                class:active={$toolkitConfig.convertFormat === 'WebP'}
                on:click={() => ($toolkitConfig.convertFormat = 'WebP')}
              >
                <span class="pill-name">WebP</span>
                <span class="pill-ext">.webp</span>
              </button>
              <button
                type="button"
                class="format-pill"
                class:active={$toolkitConfig.convertFormat === 'Png'}
                on:click={() => ($toolkitConfig.convertFormat = 'Png')}
              >
                <span class="pill-name">PNG</span>
                <span class="pill-ext">.png</span>
              </button>
            </div>
          </div>

          {#if $toolkitConfig.convertFormat === 'Jpeg'}
            <Stepper
              label="JPEG Quality"
              bind:value={$toolkitConfig.quality}
              min={50}
              max={100}
              step={5}
              unit="%"
              presets={[
                { label: '100%', value: 100 },
                { label: '90%', value: 90 },
                { label: '80%', value: 80 },
                { label: '70%', value: 70 },
              ]}
              accentColor="yellow"
            />
          {/if}
        </div>

        <div class="divider"></div>

        <Toggle
          label="Inject Original EXIF &amp; GPS Metadata"
          bind:checked={$toolkitConfig.embedExif}
          accentColor="emerald"
        />

        <Toggle
          label="Mux Samsung / Google Motion Photos"
          bind:checked={$toolkitConfig.createMotionPhotos}
          accentColor="cyan"
        />
      </div>

      <!-- 4. Compositing & Layout -->
      <div class="card section-card">
        <div class="section-title-row">
          <Layers size={18} class="text-purple-400" />
          <h2 class="title-sm">3. Camera Compositing &amp; Layout</h2>
        </div>

        <Toggle
          label="Generate Combined Memories"
          bind:checked={$toolkitConfig.createCombined}
          accentColor="yellow"
        />

        {#if $toolkitConfig.createCombined}
          <div class="sub-options-box">
            <div class="field-group">
              <label for="combine-select" class="label">Dual-Camera Combination Mode</label>
              <select id="combine-select" class="input-select" bind:value={$toolkitConfig.combineMode}>
                <option value="PictureInPicture">Picture-in-Picture</option>
                <option value="SideBySide">Side-by-Side</option>
              </select>
            </div>

            <div class="divider"></div>

            <Toggle
              label="Generate Reversed Memories"
              bind:checked={$toolkitConfig.createReversed}
              accentColor="violet"
            />
          </div>
        {/if}
      </div>

      <!-- Start Button -->
      <div class="action-footer">
        <button
          type="button"
          class="btn btn-accent-yellow btn-lg"
          class:btn-disabled-look={!isConfigValid}
          on:click={handleStart}
        >
          <Play size={16} fill="currentColor" />
          <span>Start Processing Archive &rarr;</span>
        </button>
      </div>
    </div>

    <!-- Right Column: Live Output Preview Panel -->
    <div class="preview-column">
      <div class="card preview-sticky-card">
        <div class="preview-header">
          <div class="title-group">
            <span class="preview-dot"></span>
            <span class="title-sm font-semibold">
              {$toolkitConfig.createCombined ? 'Compositing Preview' : 'Metadata Preview'}
            </span>
          </div>
          <span class="badge badge-yellow format-badge">
            {$toolkitConfig.convertFormat === 'Jpeg'
              ? `JPEG • ${$toolkitConfig.quality}%`
              : $toolkitConfig.convertFormat === 'Png'
              ? 'PNG • Lossless'
              : 'WebP • Lossless'}
          </span>
        </div>

        {#if $toolkitConfig.createCombined}
          <!-- Preview View Mode Selector (Standard vs Reversed) -->
          {#if $toolkitConfig.createReversed}
            <div class="preview-mode-tabs">
              <button
                type="button"
                class="tab-btn"
                class:active={previewTab === 'standard'}
                on:click={() => (previewTab = 'standard')}
              >
                Standard Output
              </button>
              <button
                type="button"
                class="tab-btn active-reversed"
                class:active={previewTab === 'reversed'}
                on:click={() => (previewTab = 'reversed')}
              >
                <Repeat size={11} />
                Reversed Output
              </button>
            </div>
          {/if}

          <!-- Canvas Frame -->
          <div class="mockup-frame" class:is-sbs={$toolkitConfig.combineMode === 'SideBySide'}>
            {#if $toolkitConfig.combineMode === 'PictureInPicture'}
              <!-- PIP Layout Canvas -->
              <div class="pip-canvas">
                <!-- Main Background Canvas -->
                <div
                  class="canvas-main-bg"
                  class:is-secondary={previewTab === 'reversed'}
                >
                  <div class="camera-tag">
                    <Camera size={12} />
                    <span>{previewTab === 'reversed' ? 'Secondary Camera (Selfie)' : 'Primary Camera (Main)'}</span>
                  </div>
                </div>

                <!-- Overlaid PIP Window (at authentic 55,55 corner) -->
                <div
                  class="canvas-pip-overlay"
                  class:is-primary={previewTab === 'reversed'}
                >
                  <div class="pip-tag">
                    <span>{previewTab === 'reversed' ? 'Primary' : 'Selfie'}</span>
                  </div>
                  <div class="pip-lens-circle"></div>
                </div>
              </div>
            {:else}
              <!-- Side by Side Layout Canvas -->
              <div class="sbs-canvas">
                <div class="sbs-pane" class:pane-left={true}>
                  <span class="pane-label">{previewTab === 'reversed' ? 'Secondary' : 'Primary'}</span>
                </div>
                <div class="sbs-divider-line"></div>
                <div class="sbs-pane" class:pane-right={true}>
                  <span class="pane-label">{previewTab === 'reversed' ? 'Primary' : 'Secondary'}</span>
                </div>
              </div>
            {/if}
          </div>
        {:else}
          <!-- Standalone Individual Photos Canvas (No Combination) -->
          <div class="standalone-canvas-grid">
            <div class="standalone-photo-card">
              <div class="standalone-photo-bg primary-preview">
                <div class="camera-tag">
                  <Camera size={12} />
                  <span>primary/</span>
                </div>
              </div>
              <span class="photo-card-label">Primary Photo (Main)</span>
            </div>

            <div class="standalone-photo-card">
              <div class="standalone-photo-bg secondary-preview">
                <div class="camera-tag">
                  <Camera size={12} />
                  <span>secondary/</span>
                </div>
              </div>
              <span class="photo-card-label">Secondary Photo (Selfie)</span>
            </div>
          </div>
        {/if}

        <!-- Live Metadata & Active Features Pillbox -->
        <div class="preview-info-box">
          <div class="info-row">
            <span class="info-label">Active Output:</span>
            <strong class="info-val font-mono text-yellow-400">
              {#if $toolkitConfig.createCombined}
                {$toolkitConfig.combineMode === 'PictureInPicture'
                  ? (previewTab === 'reversed' ? 'combined_reversed/' : 'combined/')
                  : (previewTab === 'reversed' ? 'side_by_side_reversed/' : 'side_by_side/')}
              {:else}
                primary/ &amp; secondary/ (Standalone)
              {/if}
            </strong>
          </div>

          <div class="info-row">
            <span class="info-label">Mode:</span>
            <span class="text-secondary text-xs">
              {$toolkitConfig.createCombined ? 'Dual-Camera Compositing' : 'EXIF & Format Restoration Only'}
            </span>
          </div>

          {#if $toolkitConfig.embedExif}
            <div class="feature-tag tag-emerald">
              <Sparkles size={12} />
              <span>EXIF Timestamps &amp; GPS Injected</span>
            </div>
          {/if}

          {#if $toolkitConfig.createMotionPhotos}
            <div class="feature-tag tag-cyan">
              <Sparkles size={12} />
              <span>Samsung SEFH &amp; Google Motion Photo Muxed</span>
            </div>
          {/if}

          {#if $toolkitConfig.createCombined && $toolkitConfig.createReversed}
            <div class="feature-tag tag-violet">
              <Repeat size={12} />
              <span>Reversed Dual Angles Exported</span>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .config-view {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-bottom: 40px;
  }

  .top-nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .header-titles {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .main-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 340px;
    gap: 20px;
    align-items: start;
  }

  .form-column {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-width: 0;
  }

  .section-card {
    display: flex;
    flex-direction: column;
    gap: 14px;
    background: #111116;
  }

  .section-title-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .options-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 12px;
    align-items: center;
  }

  .field-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .field-group .label {
    font-size: 13px;
    font-weight: 500;
  }

  .format-field-group {
    max-width: 280px;
  }

  .custom-format-selector {
    display: flex;
    background: #0c0c10;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 3px;
    gap: 3px;
    height: 34px;
    align-items: center;
  }

  .format-pill {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 3px 6px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: calc(var(--radius-sm) - 1px);
    cursor: pointer;
    transition: all var(--transition-fast);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  .format-pill:hover {
    color: var(--text-main);
    background: #181822;
  }

  .format-pill.active {
    background: rgba(255, 230, 0, 0.16);
    color: #ffe600;
    border-color: rgba(255, 230, 0, 0.35);
    box-shadow: 0 0 10px rgba(255, 230, 0, 0.12);
  }

  .pill-name {
    font-size: 12px;
  }

  .pill-ext {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-muted);
  }

  .format-pill.active .pill-ext {
    color: rgba(255, 230, 0, 0.7);
  }

  .metadata-banner {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: 8px;
    background: #0d0d10;
    border: 1px solid var(--border-subtle);
    padding: 10px;
    border-radius: var(--radius-md);
  }

  .meta-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    text-align: center;
    align-items: center;
    background: #111116;
    padding: 8px;
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.03);
  }

  .meta-icon-label {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }

  .meta-label {
    font-size: 10.5px;
    color: var(--text-muted);
  }

  .meta-span-row {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding-top: 6px;
    border-top: 1px solid rgba(255, 255, 255, 0.04);
    font-size: 11.5px;
  }

  .meta-span-val {
    color: var(--text-main);
    font-weight: 500;
  }

  .meta-retake-pill {
    font-size: 10.5px;
    color: #fb923c;
    background: rgba(251, 146, 60, 0.1);
    border: 1px solid rgba(251, 146, 60, 0.25);
    padding: 1px 7px;
    border-radius: var(--radius-full);
  }

  .missing-files-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 8px;
    max-height: 180px;
    overflow-y: auto;
  }

  .missing-files-title {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 600;
    margin-bottom: 2px;
  }

  .missing-file-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    background: rgba(0, 0, 0, 0.35);
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid rgba(239, 68, 68, 0.15);
    word-break: break-all;
  }

  .mf-date {
    color: #38bdf8;
    font-size: 10px;
    flex-shrink: 0;
  }

  .mf-cam {
    color: #fb923c;
    font-size: 10px;
    flex-shrink: 0;
  }

  .mf-path {
    color: #fca5a5;
  }

  .status-box-scanning {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    background: rgba(255, 230, 0, 0.05);
    border: 1px dashed rgba(255, 230, 0, 0.3);
    padding: 14px;
    border-radius: var(--radius-md);
    font-size: 13px;
    color: var(--text-main);
  }

  .spinner-icon {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .diagnostic-card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
    border-radius: var(--radius-md);
    transition: all var(--transition-fast);
  }

  .valid-card {
    background: rgba(16, 185, 129, 0.04);
    border: 1px solid rgba(16, 185, 129, 0.25);
  }

  .invalid-card {
    background: rgba(239, 68, 68, 0.06);
    border: 1px solid rgba(239, 68, 68, 0.35);
    box-shadow: 0 0 16px rgba(239, 68, 68, 0.08);
  }

  .diag-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
  }

  .badge-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-radius: var(--radius-full);
    font-size: 11.5px;
    font-weight: 600;
    letter-spacing: 0.01em;
  }

  .status-pill-success {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.35);
  }

  .status-pill-error {
    background: rgba(239, 68, 68, 0.18);
    color: #f87171;
    border: 1px solid rgba(239, 68, 68, 0.45);
  }

  .type-pill {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: var(--radius-full);
    background: #181820;
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
  }

  .user-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    background: #181820;
    padding: 3px 10px;
    border-radius: var(--radius-full);
    border: 1px solid var(--border-subtle);
  }

  .warning-callout {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: rgba(245, 158, 11, 0.08);
    border: 1px solid rgba(245, 158, 11, 0.25);
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    font-size: 12px;
  }

  .warning-header-btn {
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    background: transparent;
    border: none;
    width: 100%;
    text-align: left;
    padding: 0;
    font-family: inherit;
    color: inherit;
  }

  .warning-title {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #fbbf24;
    font-weight: 500;
  }

  .btn-toggle-details {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: transparent;
    border: none;
    color: #fbbf24;
    font-size: 11px;
    cursor: pointer;
    opacity: 0.85;
    transition: opacity 0.15s;
  }

  .btn-toggle-details:hover {
    opacity: 1;
    text-decoration: underline;
  }

  .missing-files-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: rgba(0, 0, 0, 0.3);
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    max-height: 120px;
    overflow-y: auto;
  }

  .missing-files-title {
    font-size: 11px;
    color: var(--text-muted);
  }

  .missing-file-item {
    font-size: 11px;
    color: #fca5a5;
  }

  .error-callout-body {
    display: flex;
    flex-direction: column;
    gap: 10px;
    font-size: 12.5px;
  }

  .error-lead {
    color: #fca5a5;
    font-weight: 500;
  }

  .error-reasons-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-left: 0;
  }

  .error-reasons-list li {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    color: #fecaca;
    font-size: 12px;
    line-height: 1.4;
  }

  .gdpr-guide-tip {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    background: rgba(56, 189, 248, 0.08);
    border: 1px solid rgba(56, 189, 248, 0.2);
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    font-size: 11.5px;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .action-footer {
    display: flex;
    justify-content: flex-end;
    padding-top: 10px;
  }

  /* ── Right Column / Live Mockup ── */
  .preview-column {
    position: sticky;
    top: 20px;
  }

  .preview-sticky-card {
    display: flex;
    flex-direction: column;
    gap: 14px;
    background: #111115;
    padding: 18px;
    border-radius: var(--radius-lg);
    border: 1px solid rgba(255, 230, 0, 0.25);
    box-shadow: 0 8px 32px rgba(255, 230, 0, 0.08);
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    flex-wrap: nowrap;
  }

  .preview-dot {
    width: 8px;
    height: 8px;
    background: #ffe600;
    border-radius: 50%;
    box-shadow: 0 0 8px rgba(255, 230, 0, 0.8);
    flex-shrink: 0;
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 8px;
    white-space: nowrap;
    min-width: 0;
  }

  .format-badge {
    white-space: nowrap;
    flex-shrink: 0;
  }

  .preview-mode-tabs {
    display: flex;
    background: #0d0d10;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 2px;
    gap: 2px;
  }

  .tab-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 5px 8px;
    font-size: 11.5px;
    font-weight: 600;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .tab-btn:hover {
    color: var(--text-main);
  }

  .tab-btn.active {
    background: #1a1a24;
    color: #ffe600;
  }

  .tab-btn.active.active-reversed {
    background: rgba(139, 92, 246, 0.2);
    color: #c084fc;
    border: 1px solid rgba(139, 92, 246, 0.4);
  }

  .mockup-frame {
    width: 100%;
    aspect-ratio: 3 / 4;
    background: #000000;
    border-radius: var(--radius-lg);
    border: 2px solid var(--border-medium);
    overflow: hidden;
    position: relative;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.7);
  }

  .mockup-frame.is-sbs {
    aspect-ratio: 4 / 3;
  }

  .pip-canvas {
    width: 100%;
    height: 100%;
    position: relative;
  }

  .canvas-main-bg {
    width: 100%;
    height: 100%;
    background: linear-gradient(135deg, #1e293b 0%, #0f172a 50%, #090d16 100%);
    position: relative;
    display: flex;
    align-items: flex-end;
    padding: 12px;
    transition: background 0.2s ease;
  }

  .canvas-main-bg.is-secondary {
    background: linear-gradient(135deg, #312e81 0%, #1e1b4b 50%, #090720 100%);
  }

  .camera-tag {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(6px);
    border: 1px solid rgba(255, 255, 255, 0.15);
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    color: #ffffff;
  }

  .canvas-pip-overlay {
    position: absolute;
    top: 14px;
    left: 14px;
    width: 32%;
    aspect-ratio: 3 / 4;
    background: linear-gradient(135deg, #334155 0%, #1e293b 100%);
    border: 2.5px solid #000000;
    border-radius: 12px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.8);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 6px;
    transition: all 0.2s ease;
  }

  .canvas-pip-overlay.is-primary {
    background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
    border-color: #000000;
  }

  .pip-tag {
    font-size: 9.5px;
    font-weight: 700;
    color: #ffffff;
    background: rgba(0, 0, 0, 0.6);
    padding: 1px 4px;
    border-radius: 3px;
    align-self: flex-start;
  }

  .pip-lens-circle {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #0f172a;
    border: 1.5px solid #475569;
    align-self: center;
    margin-bottom: 8px;
  }

  /* Side by side layout */
  .sbs-canvas {
    width: 100%;
    height: 100%;
    display: flex;
  }

  .sbs-pane {
    flex: 1;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #1e293b;
    position: relative;
  }

  .pane-right {
    background: #0f172a;
  }

  .pane-label {
    font-size: 11px;
    font-weight: 600;
    color: #ffffff;
    background: rgba(0, 0, 0, 0.6);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .sbs-divider-line {
    width: 2px;
    height: 100%;
    background: #000000;
  }

  .preview-info-box {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: #0d0d10;
    border: 1px solid var(--border-subtle);
    padding: 12px;
    border-radius: var(--radius-md);
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
  }

  .info-label {
    color: var(--text-secondary);
  }

  .feature-tag {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
  }

  /* Standalone Individual Photos Grid */
  .standalone-canvas-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    width: 100%;
  }

  .standalone-photo-card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: center;
  }

  .standalone-photo-bg {
    width: 100%;
    aspect-ratio: 3 / 4;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-medium);
    display: flex;
    align-items: flex-end;
    padding: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
  }

  .primary-preview {
    background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  }

  .secondary-preview {
    background: linear-gradient(135deg, #312e81 0%, #1e1b4b 100%);
  }

  .photo-card-label {
    font-size: 11px;
    color: var(--text-secondary);
    font-weight: 500;
    text-align: center;
  }

  .tag-emerald {
    background: rgba(16, 185, 129, 0.12);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.25);
  }

  .tag-cyan {
    background: rgba(6, 182, 212, 0.12);
    color: #38bdf8;
    border: 1px solid rgba(6, 182, 212, 0.25);
  }

  .tag-violet {
    background: rgba(139, 92, 246, 0.12);
    color: #c084fc;
    border: 1px solid rgba(139, 92, 246, 0.25);
  }

  @media (max-width: 900px) {
    .main-layout {
      grid-template-columns: 1fr;
    }
    .preview-column {
      position: static;
    }
  }
</style>
