<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import {
    currentView,
    toolkitConfig,
    recapperConfig,
    recapperResult,
    isProcessing,
    progressState,
    liveLogs,
    activeError,
    activeJobs,
    disambiguateOutputPath,
    createActiveJob,
    updateActiveJobProgress,
    appendActiveJobLog,
    completeActiveJob,
    errorActiveJob,
    offlineGeoDbStatus,
    isDownloadingGeoDb,
    downloadGeoDbProgress,
    recordActivity,
    getPreferredRecapInputFolder,
    getSensibleRecapOutputPath,
    currentArchive,
    archiveMetadata,
  } from '$lib/stores';
  import {
    scanArchive,
    startRecapper,
    onRecapperProgress,
    onRecapperLog,
    onJobProgress,
    onJobLog,
    checkOfflineGeoDb,
    downloadOfflineGeoDb,
    setActiveGeoDbTier,
    onDownloadProgress,
    analyzeAudio,
    checkDestinationStatus,
  } from '$lib/tauri';
  import Modal from '$components/Modal.svelte';
  import type { SpeedMode, ArchiveInfo } from '$lib/types';
  import Music from 'lucide-svelte/icons/music';
  import Type from 'lucide-svelte/icons/type';
  import Activity from 'lucide-svelte/icons/activity';
  import Play from 'lucide-svelte/icons/play';
  import ArrowLeft from 'lucide-svelte/icons/arrow-left';
  import Plus from 'lucide-svelte/icons/plus';
  import RotateCcw from 'lucide-svelte/icons/rotate-ccw';
  import AlertTriangle from 'lucide-svelte/icons/triangle-alert';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import Calendar from 'lucide-svelte/icons/calendar';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import Cloud from 'lucide-svelte/icons/cloud';
  import Database from 'lucide-svelte/icons/database';
  import Download from 'lucide-svelte/icons/download';
  import CheckCircle from 'lucide-svelte/icons/circle-check';
  import HardDrive from 'lucide-svelte/icons/hard-drive';
  import Loader2 from 'lucide-svelte/icons/loader-circle';
  import User from 'lucide-svelte/icons/circle-user';
  import Mountain from 'lucide-svelte/icons/mountain';
  import FilePicker from '$components/FilePicker.svelte';
  import DateRangePicker from '$components/DateRangePicker.svelte';
  import Toggle from '$components/Toggle.svelte';
  import FontPicker from '$components/FontPicker.svelte';
  import FontSizePicker from '$components/FontSizePicker.svelte';
  import Stepper from '$components/Stepper.svelte';
  import SpeedCurvePreview from '$components/SpeedCurvePreview.svelte';
  import { BUILTIN_FONT_OPTIONS } from '$lib/fonts';
  import RuleEditor from '$components/RuleEditor.svelte';

  let audioWaveform: number[] = [];
  let isAnalyzingAudio = false;
  let selectedMemoriesCount = 0;
  let recapperArchiveMeta: ArchiveInfo | null = null;
  let isScanningInputFolder = false;
  let lastScannedFolder = '';

  async function handleFolderScan(folderPath: string) {
    if (!folderPath || folderPath.trim().length === 0) {
      recapperArchiveMeta = null;
      lastScannedFolder = '';
      return;
    }
    if (folderPath === lastScannedFolder && recapperArchiveMeta?.isValid) {
      return;
    }
    lastScannedFolder = folderPath;
    isScanningInputFolder = true;
    try {
      const meta = await scanArchive(folderPath);
      if (meta.isValid && meta.entryCount > 0) {
        recapperArchiveMeta = meta;
      } else {
        recapperArchiveMeta = null;
      }
    } catch (e) {
      recapperArchiveMeta = null;
    } finally {
      isScanningInputFolder = false;
    }
  }

  let folderDebounce: ReturnType<typeof setTimeout> | null = null;
  $: if ($recapperConfig.inputFolder !== lastScannedFolder || (!recapperArchiveMeta && $recapperConfig.inputFolder)) {
    if (folderDebounce) clearTimeout(folderDebounce);
    folderDebounce = setTimeout(() => {
      handleFolderScan($recapperConfig.inputFolder);
    }, 250);
  }

  $: if ($recapperConfig.musicPath) {
    loadWaveform($recapperConfig.musicPath);
  } else {
    audioWaveform = [];
  }

  async function loadWaveform(path: string) {
    if (!path) {
      audioWaveform = [];
      return;
    }
    try {
      isAnalyzingAudio = true;
      const analysis = await analyzeAudio(path, 100);
      audioWaveform = analysis.waveform;
    } catch (e) {
      const fallback: number[] = [];
      for (let i = 0; i < 100; i++) {
        const base = Math.sin((i / 100) * Math.PI * 4) * 0.35 + 0.45;
        const noise = Math.sin(i * 1.8) * 0.15;
        fallback.push(Math.max(0.08, Math.min(1.0, base + noise)));
      }
      audioWaveform = fallback;
    } finally {
      isAnalyzingAudio = false;
    }
  }

  // Sample Preview Data
  const sampleDate = new Date();
  const sampleLocation = 'London, United Kingdom';

  function formatSampleDate(fmt: string): string {
    const months = [
      'January', 'February', 'March', 'April', 'May', 'June',
      'July', 'August', 'September', 'October', 'November', 'December'
    ];
    const shortMonths = [
      'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
      'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'
    ];

    const d = sampleDate.getDate();
    const m = sampleDate.getMonth();
    const y = sampleDate.getFullYear();

    let res = fmt || '%d %B %Y';
    res = res.replace(/%d/g, String(d).padStart(2, '0'));
    res = res.replace(/%B/g, months[m]);
    res = res.replace(/%b/g, shortMonths[m]);
    res = res.replace(/%m/g, String(m + 1).padStart(2, '0'));
    res = res.replace(/%Y/g, String(y));
    res = res.replace(/%y/g, String(y).slice(2));
    return res;
  }

  $: currentCssFont = (() => {
    const match = BUILTIN_FONT_OPTIONS.find(
      (b) => b.id === $recapperConfig.fontPath || b.name === $recapperConfig.fontPath
    );
    return match ? match.cssFont : 'font-inter';
  })();

  $: previewDateStr = formatSampleDate($recapperConfig.dateFormat);

  const PREVIEW_SCALE = 0.1875;
  $: previewFontSize = Math.max(Math.round($recapperConfig.fontSize * PREVIEW_SCALE), 10);
  
  // High-contrast multi-layer text shadow calculation for prominent live preview
  $: textShadowCss = (() => {
    const s = $recapperConfig.shadowStrength;
    if (!s || s <= 0) return 'none';
    const d1 = Math.max(1, Math.round(s * 0.18));
    const b1 = Math.max(2, Math.round(s * 0.35));
    const d2 = Math.max(2, Math.round(s * 0.45));
    const b2 = Math.max(4, Math.round(s * 0.9));
    const b3 = Math.max(6, Math.round(s * 1.6));
    return `0 ${d1}px ${b1}px rgba(0, 0, 0, 0.98), 0 ${d2}px ${b2}px rgba(0, 0, 0, 0.9), 0 0 ${b3}px rgba(0, 0, 0, 0.85)`;
  })();

  // Date format tokens helper
  const DATE_TOKENS = [
    { label: 'Day (01-31)', token: '%d' },
    { label: 'Month Name', token: '%B' },
    { label: 'Short Month', token: '%b' },
    { label: 'Month (01-12)', token: '%m' },
    { label: 'Year (YYYY)', token: '%Y' },
    { label: 'Year (YY)', token: '%y' },
    { label: 'Slash (/)', token: '/' },
    { label: 'Dash (-)', token: '-' },
    { label: 'Dot (.)', token: '.' },
    { label: 'Space', token: ' ' },
  ];

  const DATE_PRESETS = [
    { label: '24 August 2026', format: '%d %B %Y' },
    { label: 'Aug 24, 2026', format: '%b %d, %Y' },
    { label: '24/08/2026', format: '%d/%m/%Y' },
    { label: '2026-08-24', format: '%Y-%m-%d' },
    { label: 'August 24', format: '%B %d' },
  ];

  let customPatternInput: HTMLInputElement;

  function insertToken(token: string) {
    if (customPatternInput) {
      const start = customPatternInput.selectionStart ?? ($recapperConfig.dateFormat?.length || 0);
      const end = customPatternInput.selectionEnd ?? start;
      const current = $recapperConfig.dateFormat || '';
      $recapperConfig.dateFormat = current.slice(0, start) + token + current.slice(end);
      setTimeout(() => {
        customPatternInput.focus();
        const newPos = start + token.length;
        customPatternInput.setSelectionRange(newPos, newPos);
      }, 0);
    } else {
      $recapperConfig.dateFormat = ($recapperConfig.dateFormat || '') + token;
    }
  }

  function resetDateFormat() {
    $recapperConfig.dateFormat = '%d %B %Y';
  }

  const SPEED_MODES: { id: SpeedMode; name: string; desc: string }[] = [
    { id: 'Ramp', name: 'Ramp Curve', desc: 'Slow start & finale, fast dynamic middle' },
    { id: 'Even', name: 'Even Pace', desc: 'Equal constant duration across all photos' },
    { id: 'Accelerate', name: 'Accelerate', desc: 'Cinematic slow buildup speeding to finale' },
    { id: 'Decelerate', name: 'Decelerate', desc: 'High-energy start slowing to emotional finale' },
    { id: 'Wave', name: 'Rhythm Wave', desc: 'Pulsating rhythmic speed wave' },
  ];

  const DEFAULT_GEO_TIERS = [
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

  let selectedTierId = 'cities500';

  $: availableTiers = $offlineGeoDbStatus?.tiers?.length ? $offlineGeoDbStatus.tiers : DEFAULT_GEO_TIERS;
  $: selectedTier = availableTiers.find((t) => t.id === selectedTierId) || availableTiers[2];

  onMount(async () => {
    if ($recapperConfig.inputFolder && (!recapperArchiveMeta || !recapperArchiveMeta.isValid)) {
      handleFolderScan($recapperConfig.inputFolder);
    }
    try {
      const status = await checkOfflineGeoDb();
      offlineGeoDbStatus.set(status);
      if (status.activeTier) {
        selectedTierId = status.activeTier;
      }
    } catch (e) {
      console.warn('Failed to check offline geodb:', e);
    }
  });

  async function handleSelectTier(tierId: string) {
    selectedTierId = tierId;
    const tierInfo = availableTiers.find((t) => t.id === tierId);
    if (tierInfo?.isInstalled) {
      try {
        const status = await setActiveGeoDbTier(tierId);
        offlineGeoDbStatus.set(status);
      } catch (e) {
        console.warn('Failed to switch active tier:', e);
      }
    }
  }

  async function handleDownloadSelectedTier() {
    isDownloadingGeoDb.set(true);
    const unlisten = await onDownloadProgress((p) => {
      downloadGeoDbProgress.set(p);
      if (p.percentage >= 100) {
        setTimeout(async () => {
          isDownloadingGeoDb.set(false);
          const status = await checkOfflineGeoDb();
          offlineGeoDbStatus.set(status);
          unlisten();
        }, 800);
      }
    });

    try {
      await downloadOfflineGeoDb(selectedTierId);
      const status = await checkOfflineGeoDb();
      offlineGeoDbStatus.set(status);
    } catch (e: any) {
      alert(`Offline database download error:\n${e}`);
    } finally {
      isDownloadingGeoDb.set(false);
    }
  }

  // Location position toggle helper: checked = AboveDate, unchecked = BelowDate
  $: locationAboveDate = $recapperConfig.locationPosition === 'AboveDate';
  function handleLocationPositionToggle(checked: boolean) {
    $recapperConfig.locationPosition = checked ? 'AboveDate' : 'BelowDate';
  }

  function selectToolkitFolder(sub: 'combined' | 'singles' | 'combined_reversed' | 'root') {
    if (!$toolkitConfig.outputPath) return;
    const sep = $toolkitConfig.outputPath.includes('\\') ? '\\' : '/';
    const base = $toolkitConfig.outputPath.replace(/[\\/]+$/, '');
    let target = base;
    if (sub !== 'root') {
      target = `${base}${sep}${sub}`;
    }
    $recapperConfig.inputFolder = target;
    if (!$recapperConfig.outputPath) {
      $recapperConfig.outputPath = getSensibleRecapOutputPath(target);
    }
  }

  // Auto-generate sensible default video destination whenever input folder changes if output path is empty
  $: if ($recapperConfig.inputFolder && !$recapperConfig.outputPath) {
    $recapperConfig.outputPath = getSensibleRecapOutputPath($recapperConfig.inputFolder);
  }

  let missingInputFolder = false;
  let missingMusicPath = false;
  let missingOutputPath = false;

  $: isConfigValid = Boolean($recapperConfig.inputFolder && $recapperConfig.musicPath && $recapperConfig.outputPath);

  function formatDisplayDate(dStr?: string): string {
    if (!dStr) return '';
    try {
      const d = new Date(dStr);
      if (isNaN(d.getTime())) return dStr;
      return d.toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' });
    } catch {
      return dStr;
    }
  }

  $: dateRangeLabel = (() => {
    if ($recapperConfig.dateRangeStart && $recapperConfig.dateRangeEnd) {
      return `${formatDisplayDate($recapperConfig.dateRangeStart)} – ${formatDisplayDate($recapperConfig.dateRangeEnd)}`;
    } else if ($recapperConfig.dateRangeStart) {
      return `From ${formatDisplayDate($recapperConfig.dateRangeStart)}`;
    } else if ($recapperConfig.dateRangeEnd) {
      return `Until ${formatDisplayDate($recapperConfig.dateRangeEnd)}`;
    }
    return 'Full Timeline';
  })();

  let showOverwriteModal = false;

  async function handleStartRecapper() {
    missingInputFolder = !$recapperConfig.inputFolder || $recapperConfig.inputFolder.trim() === '';
    missingMusicPath = !$recapperConfig.musicPath || $recapperConfig.musicPath.trim() === '';
    missingOutputPath = !$recapperConfig.outputPath || $recapperConfig.outputPath.trim() === '';

    if (!isConfigValid) {
      const firstMissingId = missingInputFolder
        ? 'recapper-input-folder'
        : missingMusicPath
        ? 'recapper-music-path'
        : 'recapper-output-path';

      const el = document.getElementById(firstMissingId);
      if (el) {
        el.scrollIntoView({ behavior: 'smooth', block: 'center' });
        const inputEl = el.querySelector('input');
        if (inputEl) inputEl.focus();
      }
      return;
    }

    try {
      const destStatus = await checkDestinationStatus($recapperConfig.outputPath);
      if (destStatus.exists) {
        showOverwriteModal = true;
        return;
      }
    } catch (e) {
      console.warn('Destination status check error:', e);
    }

    executeRecapperStart();
  }

  async function executeRecapperStart() {
    showOverwriteModal = false;
    const finalOutputPath = $recapperConfig.outputPath;

    // Create Active Job
    const job = createActiveJob({
      type: 'recapper',
      title: 'Recap Video Generation',
      inputPath: $recapperConfig.inputFolder,
      outputPath: finalOutputPath,
      dateRange: dateRangeLabel,
    });

    // Also update legacy single-job stores
    liveLogs.set([]);
    progressState.set({
      jobId: job.id,
      stage: 'Scanning',
      current: 0,
      total: 0,
      percentage: 0,
    });
    isProcessing.set(true);

    // Targeted listeners
    const unlistenProgress = await onJobProgress(job.id, (p) => {
      updateActiveJobProgress(job.id, p);
      progressState.set(p);
    });

    const unlistenLog = await onJobLog(job.id, (l) => {
      appendActiveJobLog(job.id, l);
      liveLogs.update((logs) => [...logs.slice(-499), l]);
    });

    // Start background processing
    startRecapper($recapperConfig, job.id)
      .then((res) => {
        completeActiveJob(job.id, res);
        recapperResult.set(res);
        recordActivity({
          type: 'recapper',
          title: 'Recap Video Generation',
          outputPath: finalOutputPath,
          inputPath: $recapperConfig.inputFolder,
          durationSecs: res.durationSecs,
          status: 'success',
          itemCount: res.filesConverted || res.entriesProcessed,
          memoriesCount: res.filesConverted || res.entriesProcessed,
          dateRange: dateRangeLabel,
          details: `${dateRangeLabel} • Generated in ${res.durationSecs.toFixed(1)}s`,
        });
        if (get(currentView) === 'processing') {
          currentView.set('complete');
        }
      })
      .catch((e: any) => {
        errorActiveJob(job.id, String(e));
        activeError.set({
          title: 'Recapper Error',
          message: 'An error occurred during recap video generation.',
          details: String(e),
        });
        if (get(currentView) === 'processing') {
          currentView.set('recapper-config');
        }
      })
      .finally(() => {
        isProcessing.set(false);
        unlistenProgress();
        unlistenLog();
      });

    // Navigate to processing view or activity
    currentView.set('processing');
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
      <h1 class="title-md font-bold">Recap Video Generator</h1>
    </div>
  </div>

  <div class="main-layout">
    <!-- Left Column: Settings Form -->
    <div class="form-column">
      <!-- 1. Input & Music Files -->
      <div class="card section-card" id="media-sources-section">
        <div class="section-title-row">
          <Music size={18} class="text-purple-400" />
          <h2 class="title-sm">1. Media Sources</h2>
        </div>

        {#if $toolkitConfig.outputPath}
          <div class="toolkit-source-suggestion">
            <div class="suggestion-header">
              <div class="suggestion-left">
                <Sparkles size={13} class="text-purple-400" />
                <span class="suggestion-title">Photo Suite Output Detected</span>
              </div>
              <span class="suggestion-path font-mono text-muted">{$toolkitConfig.outputPath}</span>
            </div>
            <div class="suggestion-actions">
              <span class="suggestion-label">Quick Select Folder:</span>
              <button
                type="button"
                class="quick-pill-btn"
                class:active={$recapperConfig.inputFolder.endsWith('combined') && !$recapperConfig.inputFolder.endsWith('combined_reversed')}
                on:click={() => selectToolkitFolder('combined')}
                title="Use Picture-in-Picture composite photos"
              >
                combined/
              </button>
              <button
                type="button"
                class="quick-pill-btn"
                class:active={$recapperConfig.inputFolder.endsWith('singles')}
                on:click={() => selectToolkitFolder('singles')}
                title="Use single photos"
              >
                singles/
              </button>
              {#if $toolkitConfig.createReversed}
                <button
                  type="button"
                  class="quick-pill-btn"
                  class:active={$recapperConfig.inputFolder.endsWith('combined_reversed')}
                  on:click={() => selectToolkitFolder('combined_reversed')}
                  title="Use reversed perspective photos"
                >
                  combined_reversed/
                </button>
              {/if}
              <button
                type="button"
                class="quick-pill-btn"
                class:active={$recapperConfig.inputFolder === $toolkitConfig.outputPath}
                on:click={() => selectToolkitFolder('root')}
                title="Use root output folder (auto-discovers subfolders)"
              >
                Root Folder
              </button>
            </div>
          </div>
        {/if}

        <FilePicker
          id="recapper-input-folder"
          label="Input Images Folder"
          placeholder="Select folder containing your processed BeReal photos..."
          isDirectory={true}
          dialogTitle="Select Processed Images Folder"
          required={true}
          accentColor="purple"
          isMissing={missingInputFolder}
          bind:value={$recapperConfig.inputFolder}
        />

        <FilePicker
          id="recapper-music-path"
          label="Soundtrack Audio File"
          placeholder="Select audio track (MP3, WAV, M4A, AAC, FLAC, OGG)..."
          isDirectory={false}
          fileExtensions={['mp3', 'wav', 'm4a', 'aac', 'flac', 'ogg']}
          dialogTitle="Select Soundtrack Audio"
          required={true}
          accentColor="purple"
          isMissing={missingMusicPath}
          bind:value={$recapperConfig.musicPath}
        />

        <FilePicker
          id="recapper-output-path"
          label="Output Video Destination (.mp4)"
          placeholder="Choose destination path for the rendered MP4..."
          isDirectory={false}
          fileExtensions={['mp4']}
          dialogTitle="Save Recap Video"
          required={true}
          accentColor="purple"
          isMissing={missingOutputPath}
          bind:value={$recapperConfig.outputPath}
        />
      </div>

      <!-- 2. Date Range & Timeline Filter -->
      <DateRangePicker
        title="2. Date Range & Timeline Filter"
        accentColor="purple"
        histogram={recapperArchiveMeta?.monthlyHistogram || $archiveMetadata?.monthlyHistogram || []}
        minDate={recapperArchiveMeta?.earliestDate?.slice(0, 10) || $archiveMetadata?.earliestDate?.slice(0, 10) || ''}
        maxDate={recapperArchiveMeta?.latestDate?.slice(0, 10) || $archiveMetadata?.latestDate?.slice(0, 10) || ''}
        totalCount={recapperArchiveMeta?.entryCount || $archiveMetadata?.entryCount || 0}
        bind:startDate={$recapperConfig.dateRangeStart}
        bind:endDate={$recapperConfig.dateRangeEnd}
        bind:selectedCount={selectedMemoriesCount}
      />

      <!-- 3. Typography & Overlays -->
      <div class="card section-card">
        <div class="section-title-row">
          <Type size={18} class="text-purple-400" />
          <h2 class="title-sm">3. Typography &amp; Visual Overlays</h2>
        </div>

        <FontPicker
          label="Font Family"
          bind:selectedPath={$recapperConfig.fontPath}
        />

        <div class="options-grid">
          <FontSizePicker
            label="Font Size"
            bind:value={$recapperConfig.fontSize}
          />

          <Stepper
            label="Text Shadow"
            bind:value={$recapperConfig.shadowStrength}
            min={0}
            max={35}
            step={2}
            unit="px"
            presets={[
              { label: 'Off', value: 0 },
              { label: 'Low', value: 4 },
              { label: 'Med', value: 12 },
              { label: 'High', value: 24 },
            ]}
            accentColor="violet"
          />
        </div>

        <div class="divider"></div>

        <!-- Date Overlay Section -->
        <div class="overlay-section">
          <Toggle
            label="Display Date Stamp"
            description="Overlays formatted capture timestamp onto recap slides"
            tooltip="Renders the authentic post capture date with custom typography on each photo frame."
            icon={Calendar}
            bind:checked={$recapperConfig.dateEnabled}
            accentColor="violet"
          />

          {#if $recapperConfig.dateEnabled}
            <div class="sub-options-box">
              <!-- Format Presets -->
              <div class="field-group">
                <span class="label">Date Format Presets</span>
                <div class="presets-wrap">
                  {#each DATE_PRESETS as p}
                    <button
                      type="button"
                      class="preset-pill"
                      class:active={$recapperConfig.dateFormat === p.format}
                      on:click={() => ($recapperConfig.dateFormat = p.format)}
                    >
                      {p.label}
                    </button>
                  {/each}
                </div>
              </div>

              <!-- Token Inserters -->
              <div class="field-group">
                <div class="tokens-header">
                  <span class="label">Insert Format Tokens (at cursor)</span>
                  <button type="button" class="btn-token-reset" on:click={resetDateFormat}>
                    <RotateCcw size={11} /> Reset Pattern
                  </button>
                </div>
                <div class="tokens-wrap">
                  {#each DATE_TOKENS as tok}
                    <button
                      type="button"
                      class="token-btn"
                      on:click={() => insertToken(tok.token)}
                      title="Insert {tok.token} at cursor position"
                    >
                      <Plus size={11} class="text-purple-400" />
                      <span>{tok.label}</span>
                      <code class="token-code">{tok.token}</code>
                    </button>
                  {/each}
                </div>
              </div>

              <div class="field-group">
                <label for="date-fmt-input" class="label">Custom Pattern</label>
                <input
                  id="date-fmt-input"
                  type="text"
                  class="input-text font-mono"
                  bind:this={customPatternInput}
                  bind:value={$recapperConfig.dateFormat}
                  placeholder="%d %B %Y"
                />
                <div class="token-preview">
                  <Sparkles size={11} class="text-purple-400" />
                  <span>Preview: </span>
                  <strong class="text-purple-300 font-mono">{formatSampleDate($recapperConfig.dateFormat)}</strong>
                </div>
              </div>

              <!-- Visual Screen Position Selector -->
              <div class="field-group">
                <span class="label">Screen Position</span>
                <div class="pos-visual-grid">
                  {#each [
                    { id: 'BottomCenter', label: 'Bottom Center', desc: 'Default baseline' },
                    { id: 'BottomLeft', label: 'Bottom Left', desc: 'Lower-left' },
                    { id: 'BottomRight', label: 'Bottom Right', desc: 'Lower-right' },
                    { id: 'TopRight', label: 'Top Right', desc: 'Header angle' },
                  ] as pos}
                    <button
                      type="button"
                      class="pos-card-btn"
                      class:active={$recapperConfig.datePosition === pos.id}
                      on:click={() => ($recapperConfig.datePosition = pos.id as import('$lib/types').TextPosition)}
                    >
                      <div class="pos-mini-screen">
                        <div class="screen-pip-hint"></div>
                        <div class="pos-screen-dot pos-dot-{pos.id.toLowerCase()}"></div>
                      </div>
                      <div class="pos-card-info">
                        <span class="pos-card-title">{pos.label}</span>
                        <span class="pos-card-desc">{pos.desc}</span>
                      </div>
                    </button>
                  {/each}
                </div>
              </div>
            </div>
          {/if}
        </div>

        <div class="divider"></div>

        <!-- Location Overlay Section -->
        <div class="overlay-section">
          <Toggle
            label="Display Location"
            description="Reverse geocodes GPS coordinates into readable city and country names"
            tooltip="Looks up location coordinates from photo EXIF tags and displays formatted location text."
            icon={MapPin}
            bind:checked={$recapperConfig.locationEnabled}
            accentColor="emerald"
          />

          {#if $recapperConfig.locationEnabled}
            <div class="sub-options-box">
              <!-- Visual Geocoding Engine Selector -->
              <div class="field-group">
                <span class="label">Geocoding Service Engine</span>
                <div class="geo-service-grid">
                  <button
                    type="button"
                    class="geo-service-btn"
                    class:active={$recapperConfig.geocodingMode === 'Online'}
                    on:click={() => ($recapperConfig.geocodingMode = 'Online')}
                  >
                    <div class="geo-icon-circle icon-cloud">
                      <Cloud size={16} />
                    </div>
                    <div class="geo-btn-titles">
                      <span class="geo-btn-title">Nominatim OpenStreetMap</span>
                      <span class="geo-btn-sub">Global reverse geocoding API (Online)</span>
                    </div>
                  </button>

                  <button
                    type="button"
                    class="geo-service-btn"
                    class:active={$recapperConfig.geocodingMode === 'Offline'}
                    on:click={() => ($recapperConfig.geocodingMode = 'Offline')}
                  >
                    <div class="geo-icon-circle icon-db">
                      <Database size={16} />
                    </div>
                    <div class="geo-btn-titles">
                      <span class="geo-btn-title">Offline Reverse Database</span>
                      <span class="geo-btn-sub">Local offline location database lookup</span>
                    </div>
                  </button>
                </div>
              </div>

              <div class="divider"></div>

              <!-- Visual Stack Order Selector -->
              <div class="field-group">
                <span class="label">Location Stack Order</span>
                <div class="stack-order-grid">
                  <button
                    type="button"
                    class="stack-order-btn"
                    class:active={$recapperConfig.locationPosition === 'BelowDate'}
                    on:click={() => ($recapperConfig.locationPosition = 'BelowDate')}
                  >
                    <div class="stack-preview-box">
                      <span class="stack-pill pill-date">25 August 2026</span>
                      <span class="stack-pill pill-loc">📍 London, United Kingdom</span>
                    </div>
                    <div class="stack-btn-titles">
                      <span class="stack-btn-title">Below Date (Default)</span>
                      <span class="stack-btn-sub">Date on top, location underneath</span>
                    </div>
                  </button>

                  <button
                    type="button"
                    class="stack-order-btn"
                    class:active={$recapperConfig.locationPosition === 'AboveDate'}
                    on:click={() => ($recapperConfig.locationPosition = 'AboveDate')}
                  >
                    <div class="stack-preview-box">
                      <span class="stack-pill pill-loc">📍 London, United Kingdom</span>
                      <span class="stack-pill pill-date">25 August 2026</span>
                    </div>
                    <div class="stack-btn-titles">
                      <span class="stack-btn-title">Above Date</span>
                      <span class="stack-btn-sub">Location on top, date underneath</span>
                    </div>
                  </button>
                </div>
              </div>

              {#if $recapperConfig.geocodingMode === 'Offline'}
                <!-- Precision Tier Selector -->
                <div class="field-group">
                  <div class="tier-header-row">
                    <span class="label">Database Precision Tier</span>
                    <span class="text-xs text-muted">Select precision level &amp; switch instantly</span>
                  </div>

                  <div class="geo-tiers-grid">
                    {#each availableTiers as tier}
                      <button
                        type="button"
                        class="geo-tier-card"
                        class:active={selectedTierId === tier.id}
                        class:installed={tier.isInstalled}
                        on:click={() => handleSelectTier(tier.id)}
                      >
                        <div class="tier-card-top">
                          <span class="tier-name font-semibold">{tier.name}</span>
                          {#if tier.isInstalled}
                            <span class="badge {tier.isActive ? 'badge-success' : 'badge-subtle'} text-xs font-mono">
                              {#if tier.isActive}✓ Active{:else}Installed{/if}
                            </span>
                          {:else}
                            <span class="badge badge-yellow text-xs font-mono">~{tier.approxDownloadMb} MB</span>
                          {/if}
                        </div>
                        <span class="tier-sub text-xs text-secondary">{tier.subtitle}</span>
                        <span class="tier-cities text-xs font-mono text-muted">{tier.approxCities}</span>
                      </button>
                    {/each}
                  </div>
                </div>

                <!-- Current Selected Tier Status Card -->
                {#if $isDownloadingGeoDb}
                  <div class="geo-download-card card">
                    <div class="geo-download-head">
                      <div class="geo-download-title-row">
                        <Loader2 size={16} class="animate-spin text-amber-400" />
                        <span class="font-bold text-white text-sm">Downloading {selectedTier?.name || 'Offline'} Dataset...</span>
                      </div>
                      <span class="badge badge-yellow font-mono text-xs">
                        {$downloadGeoDbProgress ? `${$downloadGeoDbProgress.percentage.toFixed(2)}%` : 'Connecting...'}
                      </span>
                    </div>

                    <p class="text-xs text-secondary">
                      {$downloadGeoDbProgress?.status || `Connecting to GeoNames server (~${selectedTier?.approxDownloadMb || 12.5} MB)...`}
                    </p>

                    <div class="geo-download-track">
                      <div
                        class="geo-download-fill"
                        style="width: {Math.min(Math.max($downloadGeoDbProgress?.percentage || 5, 0), 100)}%;"
                      ></div>
                    </div>

                    <div class="geo-download-meta font-mono text-xs text-muted">
                      {#if $downloadGeoDbProgress}
                        <span>{($downloadGeoDbProgress.bytesDownloaded / 1048576).toFixed(1)} MB / {($downloadGeoDbProgress.totalBytes / 1048576).toFixed(1)} MB</span>
                        <span>&bull;</span>
                        <span>{$downloadGeoDbProgress.speedMbps.toFixed(1)} MB/s</span>
                      {:else}
                        <span>Initializing stream...</span>
                      {/if}
                    </div>
                  </div>
                {:else if selectedTier?.isInstalled}
                  <div class="geo-installed-card card">
                    <div class="installed-left">
                      <div class="installed-icon-circle">
                        <CheckCircle size={16} class="text-emerald-400" />
                      </div>
                      <div class="installed-titles">
                        <div class="installed-title-row">
                          <span class="font-bold text-white text-sm">{selectedTier.name} Dataset Active &amp; Ready</span>
                          <span class="badge badge-success font-mono text-xs">{selectedTier.approxCities}</span>
                        </div>
                        <span class="text-xs text-muted font-mono">
                          {(selectedTier.fileSizeBytes / 1048576).toFixed(1)} MB on disk &bull; Sub-millisecond lookup &bull; 100% Offline
                        </span>
                      </div>
                    </div>
                    <button
                      type="button"
                      class="btn btn-ghost btn-xs text-muted hover:text-white"
                      on:click={handleDownloadSelectedTier}
                      title="Update or re-download dataset"
                    >
                      <RotateCcw size={12} />
                      <span>Re-download</span>
                    </button>
                  </div>
                {:else}
                  <div class="geo-prompt-card card">
                    <div class="geo-prompt-left">
                      <div class="geo-prompt-icon-circle">
                        <HardDrive size={18} class="text-amber-400" />
                      </div>
                      <div class="geo-prompt-titles">
                        <div class="geo-prompt-title-row">
                          <span class="font-bold text-white text-sm">{selectedTier?.name || 'Precision'} Dataset Required</span>
                          <span class="badge badge-yellow font-mono text-xs">~{selectedTier?.approxDownloadMb || 12.5} MB</span>
                        </div>
                        <p class="text-xs text-secondary">
                          Download the {selectedTier?.subtitle || 'GeoNames'} dataset ({selectedTier?.approxCities || '200,000+ towns'}) for instant reverse geocoding.
                        </p>
                      </div>
                    </div>

                    <button
                      type="button"
                      class="btn btn-accent-yellow btn-sm flex-shrink-0"
                      on:click={handleDownloadSelectedTier}
                    >
                      <Download size={14} />
                      <span>Download {selectedTier?.name || ''} Dataset (~{selectedTier?.approxDownloadMb || 12.5} MB)</span>
                    </button>
                  </div>
                {/if}
              {/if}

              <RuleEditor bind:rules={$recapperConfig.locationRules} />
            </div>
          {/if}
        </div>
      </div>

      <!-- 4. Pacing & Timing Settings -->
      <div class="card section-card">
        <div class="section-title-row">
          <Activity size={18} class="text-sky-400" />
          <h2 class="title-sm">4. Speed Transitions &amp; Pacing</h2>
        </div>

        <!-- Speed Mode Cards Grid with Inline Sparklines -->
        <div class="speed-modes-grid">
          {#each SPEED_MODES as m}
            <button
              type="button"
              class="speed-mode-card"
              class:active={$recapperConfig.speedMode === m.id}
              on:click={() => ($recapperConfig.speedMode = m.id)}
            >
              <div class="mode-head">
                <span class="mode-name">{m.name}</span>
                {#if $recapperConfig.speedMode === m.id}
                  <span class="active-dot"></span>
                {/if}
              </div>
              <span class="mode-desc text-secondary">{m.desc}</span>
              <div class="mode-sparkline-wrap">
                <SpeedCurvePreview mode={m.id} />
              </div>
            </button>
          {/each}
        </div>

        <!-- Animated Timeline Preview for the Selected Mode with Start & End Padding -->
        <div class="timeline-preview-card">
          <SpeedCurvePreview
            mode={$recapperConfig.speedMode}
            animated={true}
            startPadding={$recapperConfig.startPadding}
            endPadding={$recapperConfig.endPadding}
            waveform={audioWaveform}
          />
        </div>

        <div class="options-grid">
          <Stepper
            label="Start Hold Padding"
            bind:value={$recapperConfig.startPadding}
            min={0}
            max={10}
            step={0.5}
            unit="s"
            presets={[
              { label: 'Off', value: 0 },
              { label: '1.5s', value: 1.5 },
              { label: '3s', value: 3 },
              { label: '5s', value: 5 },
            ]}
            accentColor="cyan"
          />

          <Stepper
            label="End Hold Padding"
            bind:value={$recapperConfig.endPadding}
            min={0}
            max={10}
            step={0.5}
            unit="s"
            presets={[
              { label: 'Off', value: 0 },
              { label: '1.5s', value: 1.5 },
              { label: '3s', value: 3 },
              { label: '5s', value: 5 },
            ]}
            accentColor="cyan"
          />
        </div>

        <div class="options-grid">
          <Stepper
            label="Min Video Length"
            bind:value={$recapperConfig.minDurationSecs}
            min={0}
            max={300}
            step={5}
            unit="s"
            presets={[
              { label: 'Auto', value: 0 },
              { label: '15s', value: 15 },
              { label: '30s', value: 30 },
              { label: '60s', value: 60 },
            ]}
            accentColor="violet"
          />

          <Stepper
            label="Max Video Length"
            bind:value={$recapperConfig.maxDurationSecs}
            min={0}
            max={600}
            step={5}
            unit="s"
            presets={[
              { label: 'Full Track', value: 0 },
              { label: '30s', value: 30 },
              { label: '60s', value: 60 },
              { label: '120s', value: 120 },
            ]}
            accentColor="violet"
          />
        </div>

        <div class="divider"></div>

        <!-- Framerate Selector -->
        <div class="field-group">
          <span class="label">Framerate</span>
          <div class="fps-pills">
            {#each [24, 30, 60] as fpsVal}
              <button
                type="button"
                class="fps-pill"
                class:active={$recapperConfig.fps === fpsVal}
                on:click={() => ($recapperConfig.fps = fpsVal)}
              >
                {fpsVal} FPS
              </button>
            {/each}
          </div>
        </div>
      </div>
    </div>

    <!-- Right Column: Live Updating Preview Mockup -->
    <div class="preview-column">
      <div class="card preview-sticky-card">
        <div class="preview-header">
          <div class="title-group">
            <span class="preview-dot"></span>
            <span class="title-sm font-semibold">Live Preview</span>
          </div>
        </div>

        <!-- Live Mockup Canvas (3:4 portrait) -->
        <div class="mockup-frame">
          <div class="mockup-screen">
            <div class="simulated-photo">
              <!-- Primary Background Camera Badge (dynamically positioned to avoid text overlay clipping) -->
              <div
                class="camera-badge badge-primary"
                class:badge-pos-top-right={$recapperConfig.datePosition !== 'TopRight'}
                class:badge-pos-bottom-left={$recapperConfig.datePosition === 'TopRight'}
                title="Landscape & Environment (Main Camera)"
              >
                <Mountain size={13} class="badge-icon text-sky-300" />
              </div>

              <!-- Secondary PIP in top-left -->
              <div class="simulated-pip">
                <div class="pip-lens-circle"></div>
                <div class="camera-badge badge-secondary" title="Person Silhouette (Selfie)">
                  <User size={13} class="badge-icon text-purple-300" />
                </div>
              </div>

              <!-- Overlaid Text Elements with Smooth Layout & Reordering Transitions -->
              <div
                class="overlay-container pos-{$recapperConfig.datePosition.toLowerCase()}"
                class:loc-is-above={$recapperConfig.locationPosition === 'AboveDate'}
                class:loc-is-below={$recapperConfig.locationPosition !== 'AboveDate'}
              >
                {#if $recapperConfig.dateEnabled && previewDateStr}
                  <div
                    class="live-date-text {currentCssFont}"
                    style="
                      font-size: {previewFontSize}px;
                      text-shadow: {textShadowCss};
                    "
                  >
                    {previewDateStr}
                  </div>
                {/if}

                {#if $recapperConfig.locationEnabled}
                  <div
                    class="live-location-text {currentCssFont}"
                    style="
                      font-size: {Math.max(Math.round(previewFontSize * 0.82), 9)}px;
                      text-shadow: {textShadowCss};
                    "
                  >
                    {sampleLocation}
                  </div>
                {/if}
              </div>
            </div>
          </div>
        </div>

        <!-- Generate Video Action Button -->
        <div class="action-footer preview-action-footer">
          <button
            type="button"
            class="btn btn-accent-violet btn-lg w-full"
            class:btn-disabled-look={!isConfigValid}
            on:click={handleStartRecapper}
          >
            <Play size={16} fill="currentColor" />
            <span>Generate Recap Video &rarr;</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</div>

<!-- Overwrite Confirmation Modal -->
<Modal
  bind:open={showOverwriteModal}
  title="Overwrite Existing Video?"
  maxWidth="460px"
>
  <div class="overwrite-modal-content">
    <div class="overwrite-modal-icon-wrap">
      <AlertTriangle size={26} class="text-purple-400" />
    </div>
    <div class="overwrite-modal-text">
      <p class="text-white font-semibold">
        A video file already exists at the specified export destination:
      </p>
      <p class="text-secondary text-xs font-mono path-preview-box">
        {$recapperConfig.outputPath}
      </p>
      <p class="text-muted text-xs">
        Would you like to overwrite and replace the existing video?
      </p>
    </div>
  </div>

  <svelte:fragment slot="footer">
    <div class="modal-actions-row">
      <button type="button" class="btn btn-secondary btn-sm" on:click={() => (showOverwriteModal = false)}>
        Cancel
      </button>
      <button type="button" class="btn btn-accent-violet btn-sm" on:click={executeRecapperStart}>
        <RotateCcw size={14} />
        <span>Overwrite Video</span>
      </button>
    </div>
  </svelte:fragment>
</Modal>

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
  }

  .field-group .label {
    font-size: 13px;
    font-weight: 500;
  }

  .align-center-toggle {
    justify-content: center;
    padding-top: 18px;
  }

  .sub-options-box {
    background: #0e0e11;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .overlay-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  /* Date Tokens & Presets */
  .presets-wrap {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .preset-pill {
    padding: 5px 10px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    background: #15151b;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .preset-pill:hover {
    color: var(--text-main);
    background: #1e1e26;
    border-color: var(--border-medium);
  }

  .preset-pill.active {
    background: rgba(139, 92, 246, 0.2);
    color: #c084fc;
    border-color: rgba(139, 92, 246, 0.4);
  }

  .tokens-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .btn-token-reset {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: transparent;
    border: none;
    font-size: 11.5px;
    color: var(--text-muted);
    cursor: pointer;
  }

  .btn-token-reset:hover {
    color: var(--text-main);
  }

  .tokens-wrap {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .token-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: #15151c;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    font-size: 11.5px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .token-btn:hover {
    background: #1f1f28;
    color: var(--text-main);
    border-color: rgba(139, 92, 246, 0.35);
  }

  .token-code {
    font-family: var(--font-mono);
    font-size: 10.5px;
    background: #09090b;
    padding: 1px 4px;
    border-radius: 3px;
    color: #c084fc;
  }

  .token-preview {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11.5px;
    color: var(--text-muted);
    margin-top: 4px;
  }

  /* Offline Precision Tier Selector */
  .tier-header-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
    flex-wrap: wrap;
  }

  .geo-tiers-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 10px;
    margin-top: 4px;
  }

  .geo-tier-card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px 12px;
    background: #13131a;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: all var(--transition-fast);
    position: relative;
  }

  .geo-tier-card:hover {
    background: #191924;
    border-color: var(--border-medium);
  }

  .geo-tier-card.active {
    background: #181826;
    border-color: rgba(245, 158, 11, 0.6);
    box-shadow: 0 0 14px rgba(245, 158, 11, 0.15);
  }

  .tier-card-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 6px;
  }

  .tier-name {
    font-size: 13px;
    color: var(--text-main);
  }

  .geo-tier-card.active .tier-name {
    color: #ffe600;
  }

  .tier-sub {
    font-size: 10.5px;
    line-height: 1.25;
  }

  .tier-cities {
    font-size: 10px;
    margin-top: 2px;
  }

  /* Offline Geocoding Dataset Cards */
  .geo-download-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: #14141d;
    border: 1px solid rgba(245, 158, 11, 0.35);
    padding: 14px 16px;
    border-radius: var(--radius-md);
    box-shadow: 0 4px 16px rgba(245, 158, 11, 0.08);
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

  .geo-download-meta {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .geo-installed-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    background: rgba(16, 185, 129, 0.06);
    border: 1px solid rgba(16, 185, 129, 0.25);
    padding: 12px 16px;
    border-radius: var(--radius-md);
  }

  .installed-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .installed-icon-circle {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: rgba(16, 185, 129, 0.12);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .installed-titles {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .installed-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .geo-prompt-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    background: #14141c;
    border: 1px solid rgba(245, 158, 11, 0.3);
    padding: 14px 18px;
    border-radius: var(--radius-md);
    flex-wrap: wrap;
  }

  .geo-prompt-left {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
    flex: 1;
  }

  .geo-prompt-icon-circle {
    width: 38px;
    height: 38px;
    border-radius: var(--radius-sm);
    background: rgba(245, 158, 11, 0.12);
    border: 1px solid rgba(245, 158, 11, 0.25);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .geo-prompt-titles {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .geo-prompt-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  /* Speed Modes Grid */
  .speed-modes-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 8px;
  }

  .speed-mode-card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px 12px;
    background: #0f0f13;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: all var(--transition-fast);
  }

  .speed-mode-card:hover {
    background: #16161d;
    border-color: var(--border-medium);
  }

  .speed-mode-card.active {
    background: rgba(139, 92, 246, 0.12);
    border-color: rgba(139, 92, 246, 0.45);
    box-shadow: 0 0 12px rgba(139, 92, 246, 0.15);
  }

  .mode-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .mode-name {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-main);
  }

  .mode-desc {
    font-size: 11px;
    line-height: 1.35;
    min-height: 28px;
  }

  .mode-sparkline-wrap {
    margin-top: 4px;
    display: flex;
    justify-content: center;
  }

  .active-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #c084fc;
  }

  .timeline-preview-card {
    background: #0b0b0f;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 12px 14px;
  }

  /* Framerate Pills */
  .fps-pills {
    display: flex;
    gap: 6px;
  }

  .fps-pill {
    padding: 6px 16px;
    background: #15151c;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .fps-pill:hover {
    background: #1e1e26;
    color: var(--text-main);
  }

  .fps-pill.active {
    background: rgba(139, 92, 246, 0.2);
    color: #c084fc;
    border-color: rgba(139, 92, 246, 0.4);
  }

  /* Preview column */
  .preview-column {
    position: sticky;
    top: 16px;
    align-self: start;
  }

  .preview-sticky-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: #111116;
    padding: 14px 16px;
    border-radius: var(--radius-lg);
  }

  .preview-action-footer {
    display: flex;
    width: 100%;
    padding-top: 4px;
  }

  .preview-action-footer .w-full {
    width: 100%;
    justify-content: center;
    padding: 10px 16px;
    font-size: 13px;
    font-weight: 700;
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .preview-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #a855f7;
    box-shadow: 0 0 8px rgba(168, 85, 247, 0.8);
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .mockup-frame {
    width: 100%;
    max-width: min(220px, 30vh);
    margin: 0 auto;
    aspect-ratio: 3 / 4;
    background: #000000;
    border-radius: var(--radius-lg);
    border: 2px solid var(--border-medium);
    overflow: hidden;
    position: relative;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.7);
  }

  .mockup-screen {
    width: 100%;
    height: 100%;
    position: relative;
  }

  .simulated-photo {
    width: 100%;
    height: 100%;
    background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .simulated-pip {
    position: absolute;
    top: 14px;
    left: 14px;
    width: 30%;
    aspect-ratio: 3 / 4;
    background: linear-gradient(135deg, #312e81 0%, #1e1b4b 100%);
    border: 2.5px solid #000000;
    border-radius: 12px;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.85);
    z-index: 10;
  }

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
  }

  .badge-primary {
    padding: 3px 8px;
    font-size: 11px;
    font-weight: 500;
    z-index: 2;
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .badge-primary.badge-pos-top-right {
    top: 14px;
    right: 14px;
    bottom: auto;
    left: auto;
  }

  .badge-primary.badge-pos-bottom-left {
    bottom: 14px;
    left: 14px;
    top: auto;
    right: auto;
  }

  .badge-secondary {
    top: 6px;
    left: 6px;
    padding: 2px 5px;
    font-size: 9.5px;
    font-weight: 700;
    z-index: 12;
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
  }

  /* Overlay text positions & smooth animated transitions */
  .overlay-container {
    position: absolute;
    inset: 0;
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    pointer-events: none;
    gap: 4px;
    z-index: 5;
    transition: all 0.42s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .overlay-container.pos-bottomcenter {
    justify-content: flex-end;
    align-items: center;
    text-align: center;
    padding-bottom: 14px;
  }

  .overlay-container.pos-bottomleft {
    justify-content: flex-end;
    align-items: flex-start;
    text-align: left;
    padding-bottom: 14px;
    padding-left: 14px;
  }

  .overlay-container.pos-bottomright {
    justify-content: flex-end;
    align-items: flex-end;
    text-align: right;
    padding-bottom: 14px;
    padding-right: 14px;
  }

  .overlay-container.pos-topright {
    justify-content: flex-start;
    align-items: flex-end;
    text-align: right;
    padding-top: 14px;
    padding-right: 14px;
    max-width: 70%;
    left: auto;
    right: 0;
  }

  .live-date-text,
  .live-location-text {
    color: #ffffff;
    font-weight: 700;
    line-height: 1.15;
    letter-spacing: -0.01em;
    user-select: none;
    transition: font-size 0.3s cubic-bezier(0.16, 1, 0.3, 1),
                transform 0.38s cubic-bezier(0.16, 1, 0.3, 1),
                opacity 0.25s ease,
                text-shadow 0.25s ease;
    will-change: font-size, transform;
  }

  .live-date-text {
    order: 2;
  }

  .live-location-text {
    order: 3;
    font-weight: 500;
    opacity: 0.95;
  }

  /* Animated stack order reordering */
  .overlay-container.loc-is-above .live-date-text {
    order: 2;
  }

  .overlay-container.loc-is-above .live-location-text {
    order: 1;
  }

  .overlay-container.loc-is-below .live-date-text {
    order: 1;
  }

  .overlay-container.loc-is-below .live-location-text {
    order: 2;
  }

  /* Visual Screen Position Grid */
  .pos-visual-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
  }

  @media (max-width: 600px) {
    .pos-visual-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  .pos-card-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    background: #09090d;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 10px 8px;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .pos-card-btn:hover {
    background: #14141c;
    border-color: rgba(255, 255, 255, 0.2);
  }

  .pos-card-btn.active {
    background: rgba(139, 92, 246, 0.12);
    border-color: #8b5cf6;
    box-shadow: 0 0 14px rgba(139, 92, 246, 0.2);
  }

  .pos-mini-screen {
    width: 32px;
    height: 48px;
    background: #181822;
    border: 1.5px solid #2e2e3e;
    border-radius: 4px;
    position: relative;
    box-shadow: inset 0 0 4px rgba(0, 0, 0, 0.5);
  }

  .screen-pip-hint {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 8px;
    height: 11px;
    background: #3a3a4c;
    border-radius: 2px;
  }

  .pos-screen-dot {
    position: absolute;
    width: 14px;
    height: 3px;
    border-radius: 2px;
    background: #8b5cf6;
    box-shadow: 0 0 6px #8b5cf6;
  }

  .pos-dot-bottomcenter {
    bottom: 4px;
    left: 50%;
    transform: translateX(-50%);
  }

  .pos-dot-bottomleft {
    bottom: 4px;
    left: 4px;
    width: 10px;
  }

  .pos-dot-bottomright {
    bottom: 4px;
    right: 4px;
    width: 10px;
  }

  .pos-dot-topright {
    top: 4px;
    right: 4px;
    width: 10px;
  }

  .pos-card-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .pos-card-title {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-main);
    text-align: center;
  }

  .pos-card-desc {
    font-size: 9.5px;
    color: var(--text-muted);
  }

  /* Geocoding Service Grid */
  .geo-service-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  @media (max-width: 600px) {
    .geo-service-grid {
      grid-template-columns: 1fr;
    }
  }

  .geo-service-btn {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    background: #09090d;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: all var(--transition-fast);
  }

  .geo-service-btn:hover {
    background: #14141c;
    border-color: rgba(255, 255, 255, 0.2);
  }

  .geo-service-btn.active {
    background: rgba(16, 185, 129, 0.1);
    border-color: #10b981;
    box-shadow: 0 0 14px rgba(16, 185, 129, 0.15);
  }

  .geo-btn-left {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .geo-icon-circle {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .icon-cloud {
    background: rgba(56, 189, 248, 0.15);
    color: #38bdf8;
    border: 1px solid rgba(56, 189, 248, 0.3);
  }

  .icon-db {
    background: rgba(168, 85, 247, 0.15);
    color: #c084fc;
    border: 1px solid rgba(168, 85, 247, 0.3);
  }

  .geo-btn-titles {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .geo-btn-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-main);
  }

  .geo-btn-sub {
    font-size: 11px;
    color: var(--text-muted);
  }

  /* Stack Order Grid */
  .stack-order-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  @media (max-width: 600px) {
    .stack-order-grid {
      grid-template-columns: 1fr;
    }
  }

  .stack-order-btn {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px 14px;
    background: #09090d;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: all var(--transition-fast);
  }

  .stack-order-btn:hover {
    background: #14141c;
    border-color: rgba(255, 255, 255, 0.2);
  }

  .stack-order-btn.active {
    background: rgba(56, 189, 248, 0.1);
    border-color: #38bdf8;
    box-shadow: 0 0 14px rgba(56, 189, 248, 0.15);
  }

  .stack-preview-box {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: #161620;
    border: 1px dashed rgba(255, 255, 255, 0.12);
    border-radius: var(--radius-sm);
    padding: 8px 10px;
  }

  .stack-pill {
    font-size: 10.5px;
    font-family: var(--font-mono);
    padding: 2px 6px;
    border-radius: 3px;
    text-align: center;
  }

  .pill-date {
    background: rgba(139, 92, 246, 0.2);
    color: #c084fc;
    border: 1px solid rgba(139, 92, 246, 0.4);
    font-weight: 600;
  }

  .pill-loc {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.4);
    font-weight: 500;
  }

  .stack-btn-titles {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stack-btn-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-main);
  }

  .stack-btn-sub {
    font-size: 11px;
    color: var(--text-muted);
  }

  /* Photo Suite Output Suggestion Box */
  .toolkit-source-suggestion {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: #0d0d12;
    border: 1px solid rgba(168, 85, 247, 0.28);
    border-radius: var(--radius-md);
    padding: 10px 12px;
    box-shadow: 0 4px 16px rgba(168, 85, 247, 0.08);
  }

  .suggestion-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-wrap: wrap;
  }

  .suggestion-left {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .suggestion-title {
    font-size: 12px;
    font-weight: 600;
    color: #c084fc;
  }

  .suggestion-path {
    font-size: 11px;
    word-break: break-all;
    max-width: 100%;
  }

  .suggestion-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .suggestion-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .quick-pill-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 9px;
    font-size: 11px;
    font-family: var(--font-mono);
    font-weight: 600;
    background: #14141c;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .quick-pill-btn:hover {
    color: var(--text-main);
    background: #1c1c28;
    border-color: rgba(255, 255, 255, 0.2);
  }

  .quick-pill-btn.active {
    background: rgba(168, 85, 247, 0.18);
    border-color: rgba(168, 85, 247, 0.45);
    color: #c084fc;
    box-shadow: 0 0 10px rgba(168, 85, 247, 0.15);
  }

  /* Font Class Bindings */
  .font-inter { font-family: 'font-inter', sans-serif; }
  .font-roboto { font-family: 'font-roboto', sans-serif; }
  .font-outfit { font-family: 'font-outfit', sans-serif; }
  .font-bebas { font-family: 'font-bebas', sans-serif; letter-spacing: 0.04em; }
  .font-playfair { font-family: 'font-playfair', serif; }
  .font-jetbrains { font-family: 'font-jetbrains', monospace; }
  .font-caveat { font-family: 'font-caveat', cursive; }

  .overwrite-modal-content {
    display: flex;
    gap: 16px;
    align-items: flex-start;
  }

  .overwrite-modal-icon-wrap {
    width: 44px;
    height: 44px;
    border-radius: var(--radius-md);
    background: rgba(168, 85, 247, 0.12);
    border: 1px solid rgba(168, 85, 247, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .overwrite-modal-text {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }

  .path-preview-box {
    background: #09090d;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    word-break: break-all;
    max-height: 80px;
    overflow-y: auto;
  }

  .modal-actions-row {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    width: 100%;
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
