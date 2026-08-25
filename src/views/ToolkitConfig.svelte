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
  import Mountain from 'lucide-svelte/icons/mountain';
  import FileCode from 'lucide-svelte/icons/file-code';
  import HelpCircle from 'lucide-svelte/icons/circle-help';
  import ChevronDown from 'lucide-svelte/icons/chevron-down';
  import ChevronUp from 'lucide-svelte/icons/chevron-up';
  import RefreshCw from 'lucide-svelte/icons/refresh-cw';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import MessageSquare from 'lucide-svelte/icons/message-square';
  import Clapperboard from 'lucide-svelte/icons/clapperboard';
  import Film from 'lucide-svelte/icons/film';
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
                <!-- 1. Total Memories -->
                <div class="meta-item">
                  <div class="meta-icon-label">
                    <Camera size={12} class="text-amber-400" />
                    <span class="meta-label">Total Memories</span>
                  </div>
                  <strong class="text-amber-400 font-mono">{$archiveMetadata.validPostCount}</strong>
                </div>

                <!-- 2. BTS Videos -->
                <div class="meta-item">
                  <div class="meta-icon-label">
                    <Clapperboard size={12} class="text-sky-400" />
                    <span class="meta-label">BTS Videos</span>
                  </div>
                  <strong class="text-sky-400 font-mono">{$archiveMetadata.btsCount}</strong>
                </div>

                <!-- 3. Dual Videos -->
                <div class="meta-item">
                  <div class="meta-icon-label">
                    <Film size={12} class="text-indigo-400" />
                    <span class="meta-label">Dual Videos</span>
                  </div>
                  <strong class="text-indigo-400 font-mono">
                    {$archiveMetadata.primaryVideoCount + $archiveMetadata.secondaryVideoCount}
                  </strong>
                </div>

                <!-- 4. GPS Location -->
                <div class="meta-item">
                  <div class="meta-icon-label">
                    <MapPin size={12} class="text-emerald-400" />
                    <span class="meta-label">With GPS</span>
                  </div>
                  <strong class="text-emerald-400 font-mono">{$archiveMetadata.withLocationCount}</strong>
                </div>

                <!-- 5. Captions -->
                <div class="meta-item">
                  <div class="meta-icon-label">
                    <MessageSquare size={12} class="text-purple-400" />
                    <span class="meta-label">With Captions</span>
                  </div>
                  <strong class="text-purple-300 font-mono">{$archiveMetadata.withCaptionCount}</strong>
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
                  <div class="gdpr-guide-head">
                    <HelpCircle size={15} class="text-sky-400 flex-shrink-0" />
                    <strong>How to Download Your Authentic Archive from BeReal:</strong>
                  </div>
                  <ol class="gdpr-step-list">
                    <li>
                      <span class="step-badge font-mono">1</span>
                      <span>Tap your <strong>Profile icon</strong> in the top-right corner.</span>
                    </li>
                    <li>
                      <span class="step-badge font-mono">2</span>
                      <span>Tap <strong>Help</strong> &rarr; Select <strong>Contact Us</strong>.</span>
                    </li>
                    <li>
                      <span class="step-badge font-mono">3</span>
                      <span>Select <strong>Ask a Question</strong> &rarr; <strong>Troubleshooting</strong> &rarr; <strong>Other</strong>.</span>
                    </li>
                    <li>
                      <span class="step-badge font-mono">4</span>
                      <span>Tap <strong>Contact Us</strong> at the bottom of the article.</span>
                    </li>
                    <li>
                      <span class="step-badge font-mono">5</span>
                      <span>Select Topic: <strong>"I'd like to request a copy of my data"</strong>.</span>
                    </li>
                    <li>
                      <span class="step-badge font-mono">6</span>
                      <span>Type a message with at least <strong>10 characters</strong> and submit your request.</span>
                    </li>
                    <li>
                      <span class="step-badge font-mono">7</span>
                      <span>BeReal will email you a secure download link containing your raw archive ZIP.</span>
                    </li>
                  </ol>
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
          label="Embed EXIF &amp; GPS Metadata"
          description="Restores original capture date, GPS tags, and camera metadata"
          tooltip="Embeds authentic ISO capture timestamps, geolocation coordinates, and IPTC tags into JPEG/WebP photos."
          icon={MapPin}
          bind:checked={$toolkitConfig.embedExif}
          accentColor="emerald"
        />

        <Toggle
          label="Live &amp; Motion Photos"
          description="Embeds BTS micro-video clips into interactive moving photos"
          tooltip="Muxes BTS video clips into photos compatible with Samsung Gallery (SEFH) and Google Photos (GCamera XMP)."
          icon={Film}
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
          label="Dual-Camera Compositing"
          description="Merges primary and selfie cameras into unified memory photos"
          tooltip="Combines both camera views into single composite photos using Picture-in-Picture or Side-by-Side layout."
          icon={Layers}
          bind:checked={$toolkitConfig.createCombined}
          accentColor="yellow"
        />

        {#if $toolkitConfig.createCombined}
          <div class="sub-options-box">
            <div class="field-group">
              <span class="label">Dual-Camera Combination Mode</span>
              <div class="combine-mode-segmented">
                <button
                  type="button"
                  class="combine-mode-btn"
                  class:active={$toolkitConfig.combineMode === 'PictureInPicture'}
                  on:click={() => ($toolkitConfig.combineMode = 'PictureInPicture')}
                >
                  <div class="mini-diagram pip-mini">
                    <span class="mini-bg"></span>
                    <span class="mini-inset"></span>
                  </div>
                  <div class="mode-info">
                    <span class="mode-name">Picture-in-Picture</span>
                    <span class="mode-subtext">Authentic corner overlay</span>
                  </div>
                </button>

                <button
                  type="button"
                  class="combine-mode-btn"
                  class:active={$toolkitConfig.combineMode === 'SideBySide'}
                  on:click={() => ($toolkitConfig.combineMode = 'SideBySide')}
                >
                  <div class="mini-diagram sbs-mini">
                    <span class="mini-left"></span>
                    <span class="mini-right"></span>
                  </div>
                  <div class="mode-info">
                    <span class="mode-name">Side-by-Side</span>
                    <span class="mode-subtext">Split dual view layout</span>
                  </div>
                </button>
              </div>
            </div>

            <div class="divider"></div>

            <Toggle
              label="Generate Reversed Angles"
              description="Exports secondary-perspective memories with swapped cameras"
              tooltip="Exports additional memories where the selfie camera is the background canvas and the main camera is the overlay."
              icon={Repeat}
              bind:checked={$toolkitConfig.createReversed}
              accentColor="violet"
            />
          </div>
        {/if}
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

          <!-- Animated Morphing Canvas Frame -->
          <div class="mockup-frame" class:is-sbs={$toolkitConfig.combineMode === 'SideBySide'}>
            <div
              class="morph-stage"
              class:mode-pip={$toolkitConfig.combineMode === 'PictureInPicture'}
              class:mode-sbs={$toolkitConfig.combineMode === 'SideBySide'}
            >
              <!-- Primary Camera Layer (Environment / Landscape) -->
              <div
                class="cam-layer layer-primary"
                class:is-swapped={previewTab === 'reversed'}
              >
                <div class="camera-badge" title={previewTab === 'reversed' ? 'Person Silhouette (Selfie)' : 'Landscape & Environment (Main Camera)'}>
                  {#if previewTab === 'reversed'}
                    <User size={13} class="badge-icon text-purple-300" />
                  {:else}
                    <Mountain size={13} class="badge-icon text-sky-300" />
                  {/if}
                </div>
              </div>

              <!-- Secondary / Selfie Camera Layer (Person Silhouette) -->
              <div
                class="cam-layer layer-secondary"
                class:is-swapped={previewTab === 'reversed'}
              >
                <div class="pip-lens-circle"></div>
                <div class="camera-badge" title={previewTab === 'reversed' ? 'Landscape & Environment (Main Camera)' : 'Person Silhouette (Selfie)'}>
                  {#if previewTab === 'reversed'}
                    <Mountain size={13} class="badge-icon text-sky-300" />
                  {:else}
                    <User size={13} class="badge-icon text-purple-300" />
                  {/if}
                </div>
              </div>
            </div>
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

        <!-- Start Processing Button -->
        <div class="action-footer preview-action-footer">
          <button
            type="button"
            class="btn btn-accent-yellow btn-lg w-full"
            class:btn-disabled-look={!isConfigValid}
            on:click={handleStart}
          >
            <Play size={16} fill="currentColor" />
            <span>Start Processing Archive &rarr;</span>
          </button>
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
    align-items: start;
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
    flex-direction: column;
    gap: 8px;
    background: rgba(56, 189, 248, 0.07);
    border: 1px solid rgba(56, 189, 248, 0.22);
    padding: 12px 14px;
    border-radius: var(--radius-md);
    font-size: 11.5px;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .gdpr-guide-head {
    display: flex;
    align-items: center;
    gap: 7px;
    color: #e0f2fe;
    font-size: 12px;
  }

  .gdpr-step-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-left: 0;
    margin: 0;
  }

  .gdpr-step-list li {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    color: var(--text-secondary);
  }

  .step-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 17px;
    height: 17px;
    border-radius: 50%;
    background: rgba(56, 189, 248, 0.2);
    border: 1px solid rgba(56, 189, 248, 0.4);
    color: #38bdf8;
    font-size: 10px;
    font-weight: 700;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .action-footer {
    display: flex;
    justify-content: flex-end;
    padding-top: 10px;
  }

  .preview-action-footer {
    display: flex;
    width: 100%;
    padding-top: 6px;
  }

  .preview-action-footer .w-full {
    width: 100%;
    justify-content: center;
    padding: 12px 18px;
    font-size: 13.5px;
    font-weight: 700;
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

  /* ── Combine Mode Segmented Cards ── */
  .combine-mode-segmented {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .combine-mode-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 12px;
    background: #0e0e13;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: all var(--transition-fast);
  }

  .combine-mode-btn:hover {
    background: #15151e;
    border-color: rgba(255, 255, 255, 0.15);
  }

  .combine-mode-btn.active {
    background: rgba(255, 230, 0, 0.08);
    border-color: rgba(255, 230, 0, 0.4);
    box-shadow: 0 0 14px rgba(255, 230, 0, 0.12);
  }

  .mini-diagram {
    width: 26px;
    height: 32px;
    background: #1c1c26;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 4px;
    position: relative;
    flex-shrink: 0;
    overflow: hidden;
  }

  .combine-mode-btn.active .mini-diagram {
    border-color: rgba(255, 230, 0, 0.6);
  }

  .pip-mini .mini-bg {
    position: absolute;
    inset: 0;
    background: #1e293b;
  }

  .pip-mini .mini-inset {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 10px;
    height: 13px;
    background: #ffe600;
    border-radius: 2px;
    border: 1px solid #000000;
  }

  .sbs-mini {
    display: flex;
    gap: 1px;
    background: #000000;
  }

  .sbs-mini .mini-left {
    flex: 1;
    background: #334155;
    height: 100%;
  }

  .sbs-mini .mini-right {
    flex: 1;
    background: #ffe600;
    height: 100%;
  }

  .mode-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .mode-name {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-main);
  }

  .combine-mode-btn.active .mode-name {
    color: #ffe600;
  }

  .mode-subtext {
    font-size: 11px;
    color: var(--text-muted);
    line-height: 1.25;
  }

  /* ── Morphing Stage Canvas ── */
  .mockup-frame {
    width: 100%;
    max-width: 270px;
    margin: 0 auto;
    aspect-ratio: 3 / 4;
    background: #000000;
    border-radius: var(--radius-lg);
    border: 2px solid var(--border-medium);
    overflow: hidden;
    position: relative;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.7);
    transition: aspect-ratio 0.45s cubic-bezier(0.16, 1, 0.3, 1), max-width 0.45s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .mockup-frame.is-sbs {
    aspect-ratio: 4 / 3;
    max-width: 320px;
  }

  .morph-stage {
    position: relative;
    width: 100%;
    height: 100%;
  }

  .cam-layer {
    position: absolute;
    box-sizing: border-box;
    overflow: hidden;
    transition: all 0.48s cubic-bezier(0.16, 1, 0.3, 1);
  }

  /* Mode: Picture-in-Picture */
  .morph-stage.mode-pip .layer-primary {
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    border-radius: 0;
    border-right: none;
    z-index: 1;
    background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  }

  .morph-stage.mode-pip .layer-primary.is-swapped {
    background: linear-gradient(135deg, #312e81 0%, #1e1b4b 100%);
  }

  .morph-stage.mode-pip .layer-secondary {
    top: 14px;
    left: 14px;
    width: 32%;
    aspect-ratio: 3 / 4;
    border-radius: 12px;
    border: 2.5px solid #000000;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.85);
    z-index: 10;
    background: linear-gradient(135deg, #312e81 0%, #1e1b4b 100%);
  }

  .morph-stage.mode-pip .layer-secondary.is-swapped {
    background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  }

  /* Mode: Side-by-Side */
  .morph-stage.mode-sbs .layer-primary {
    top: 0;
    left: 0;
    width: calc(50% - 1px);
    height: 100%;
    border-radius: 0;
    border-right: 2px solid rgba(0, 0, 0, 0.95);
    z-index: 1;
    background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  }

  .morph-stage.mode-sbs .layer-primary.is-swapped {
    background: linear-gradient(135deg, #312e81 0%, #1e1b4b 100%);
  }

  .morph-stage.mode-sbs .layer-secondary {
    top: 0;
    left: 50%;
    width: 50%;
    height: 100%;
    aspect-ratio: auto;
    border-radius: 0;
    border: none;
    box-shadow: none;
    z-index: 1;
    background: linear-gradient(135deg, #312e81 0%, #1e1b4b 100%);
  }

  .morph-stage.mode-sbs .layer-secondary.is-swapped {
    background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  }

  /* Smooth Unified Camera Badges */
  .camera-badge {
    position: absolute;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: rgba(0, 0, 0, 0.72);
    backdrop-filter: blur(6px);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 4px;
    color: #ffffff;
    white-space: nowrap;
    user-select: none;
    transition: all 0.48s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .morph-stage.mode-pip .layer-primary .camera-badge {
    bottom: 12px;
    left: 12px;
    padding: 3px 8px;
    font-size: 11px;
    font-weight: 500;
  }

  .morph-stage.mode-pip .layer-secondary .camera-badge {
    top: 6px;
    left: 6px;
    padding: 2px 5px;
    font-size: 9.5px;
    font-weight: 700;
  }

  .morph-stage.mode-sbs .layer-primary .camera-badge,
  .morph-stage.mode-sbs .layer-secondary .camera-badge {
    bottom: 10px;
    left: 10px;
    padding: 3px 7px;
    font-size: 10.5px;
    font-weight: 600;
  }

  .pip-lens-circle {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #0f172a;
    border: 1.5px solid #475569;
    transition: opacity 0.35s ease, transform 0.45s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .morph-stage.mode-sbs .pip-lens-circle {
    opacity: 0;
    transform: translate(-50%, -50%) scale(0.4);
    pointer-events: none;
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
