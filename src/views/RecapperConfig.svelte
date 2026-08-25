<script lang="ts">
  import {
    currentView,
    recapperConfig,
    isProcessing,
    progressState,
    liveLogs,
    activeError,
  } from '$lib/stores';
  import {
    startRecapper,
    onRecapperProgress,
    onRecapperLog,
  } from '$lib/tauri';
  import type { SpeedMode } from '$lib/types';
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
  import FilePicker from '$components/FilePicker.svelte';
  import Toggle from '$components/Toggle.svelte';
  import FontPicker from '$components/FontPicker.svelte';
  import FontSizePicker from '$components/FontSizePicker.svelte';
  import Stepper from '$components/Stepper.svelte';
  import SpeedCurvePreview from '$components/SpeedCurvePreview.svelte';
  import { BUILTIN_FONT_OPTIONS } from '$lib/fonts';
  import RuleEditor from '$components/RuleEditor.svelte';

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

  // Location position toggle helper: checked = AboveDate, unchecked = BelowDate
  $: locationAboveDate = $recapperConfig.locationPosition === 'AboveDate';
  function handleLocationPositionToggle(checked: boolean) {
    $recapperConfig.locationPosition = checked ? 'AboveDate' : 'BelowDate';
  }

  let missingInputFolder = false;
  let missingMusicPath = false;
  let missingOutputPath = false;

  $: isConfigValid = Boolean($recapperConfig.inputFolder && $recapperConfig.musicPath && $recapperConfig.outputPath);

  async function handleStart() {
    missingInputFolder = !$recapperConfig.inputFolder;
    missingMusicPath = !$recapperConfig.musicPath;
    missingOutputPath = !$recapperConfig.outputPath;

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

    liveLogs.set([]);
    progressState.set({
      stage: 'Scanning',
      current: 0,
      total: 0,
      percentage: 0,
    });
    isProcessing.set(true);
    currentView.set('processing');

    const unlistenProgress = await onRecapperProgress((p) => {
      progressState.set(p);
      if (p.stage === 'Complete') {
        isProcessing.set(false);
        currentView.set('complete');
      }
    });

    const unlistenLog = await onRecapperLog((l) => {
      liveLogs.update((logs) => [...logs, l]);
    });

    try {
      await startRecapper($recapperConfig);
    } catch (e: any) {
      isProcessing.set(false);
      activeError.set({
        title: 'Recapper Error',
        message: 'An error occurred during recap video generation.',
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
      <h1 class="title-md font-bold">Recap Video Generator</h1>
    </div>
  </div>

  <div class="main-layout">
    <!-- Left Column: Settings Form -->
    <div class="form-column">
      <!-- 1. Input & Music Files -->
      <div class="card section-card" id="media-sources-section">
        <div class="section-title-row">
          <Music size={18} class="text-amber-400" />
          <h2 class="title-sm">1. Media Sources</h2>
        </div>

        <FilePicker
          id="recapper-input-folder"
          label="Input Images Folder"
          placeholder="Select folder containing your processed BeReal photos..."
          isDirectory={true}
          dialogTitle="Select Processed Images Folder"
          required={true}
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
          isMissing={missingOutputPath}
          bind:value={$recapperConfig.outputPath}
        />
      </div>

      <!-- 2. Typography & Overlays -->
      <div class="card section-card">
        <div class="section-title-row">
          <Type size={18} class="text-purple-400" />
          <h2 class="title-sm">2. Typography &amp; Visual Overlays</h2>
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
                      on:click={() => ($recapperConfig.datePosition = pos.id)}
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
                <div class="geo-warning-banner">
                  <AlertTriangle size={13} class="text-amber-400 flex-shrink-0" />
                  <span>Offline geocoding database is not yet bundled. The engine will automatically fall back to online Nominatim.</span>
                </div>
              {/if}

              <RuleEditor bind:rules={$recapperConfig.locationRules} />
            </div>
          {/if}
        </div>
      </div>

      <!-- 3. Pacing & Timing Settings -->
      <div class="card section-card">
        <div class="section-title-row">
          <Activity size={18} class="text-sky-400" />
          <h2 class="title-sm">3. Speed Transitions &amp; Pacing</h2>
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

        <!-- Phone Mockup Canvas (9:16 portrait) -->
        <div class="mockup-frame">
          <div class="mockup-screen">
            <div class="simulated-photo">
              <!-- Secondary PIP in top-left -->
              <div class="simulated-pip">
                <div class="pip-camera-lens"></div>
              </div>

              <!-- Overlaid Text Elements -->
              <div
                class="overlay-container pos-{$recapperConfig.datePosition.toLowerCase()}"
                class:pos-loc-above={$recapperConfig.locationPosition === 'AboveDate'}
                class:pos-loc-below={$recapperConfig.locationPosition === 'BelowDate'}
              >
                {#if $recapperConfig.locationPosition === 'AboveDate' && $recapperConfig.locationEnabled}
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

                {#if $recapperConfig.locationPosition !== 'AboveDate' && $recapperConfig.locationEnabled}
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
            on:click={handleStart}
          >
            <Play size={16} fill="currentColor" />
            <span>Generate Recap Video &rarr;</span>
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

  .geo-warning-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(245, 158, 11, 0.08);
    border: 1px solid rgba(245, 158, 11, 0.25);
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    color: var(--text-main);
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
    gap: 12px;
    background: #111116;
    padding: 16px;
    max-height: calc(100vh - 100px);
    overflow-y: auto;
  }

  .preview-action-footer {
    display: flex;
    width: 100%;
    padding-top: 4px;
  }

  .preview-action-footer .w-full {
    width: 100%;
    justify-content: center;
    padding: 12px 18px;
    font-size: 13.5px;
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
    max-width: 270px;
    margin: 0 auto;
    aspect-ratio: 9 / 16;
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
    background: radial-gradient(circle at 75% 30%, #475569 0%, #1e293b 40%, #0f172a 75%, #080c14 100%);
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .simulated-pip {
    position: absolute;
    top: 14px;
    left: 14px;
    width: 28%;
    aspect-ratio: 3 / 4;
    background: #334155;
    border: 2px solid #000000;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .pip-camera-lens {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #1e293b;
    border: 1px solid #475569;
  }

  /* Overlay text positions */
  .overlay-container {
    position: absolute;
    width: 100%;
    padding: 16px;
    display: flex;
    flex-direction: column;
    pointer-events: none;
    gap: 4px;
  }

  .overlay-container.pos-bottomcenter {
    bottom: 12px;
    left: 0;
    align-items: center;
    text-align: center;
  }

  .overlay-container.pos-bottomleft {
    bottom: 12px;
    left: 12px;
    align-items: flex-start;
    text-align: left;
  }

  .overlay-container.pos-bottomright {
    bottom: 12px;
    right: 12px;
    align-items: flex-end;
    text-align: right;
  }

  .overlay-container.pos-topright {
    top: 14px;
    right: 14px;
    left: auto;
    width: auto;
    max-width: 65%;
    align-items: flex-end;
    text-align: right;
  }

  .live-date-text, .live-location-text {
    color: #ffffff;
    font-weight: 700;
    line-height: 1.15;
    letter-spacing: -0.01em;
    user-select: none;
  }

  .live-location-text {
    font-weight: 500;
    opacity: 0.95;
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

  /* Font Class Bindings */
  .font-inter { font-family: 'font-inter', sans-serif; }
  .font-roboto { font-family: 'font-roboto', sans-serif; }
  .font-outfit { font-family: 'font-outfit', sans-serif; }
  .font-bebas { font-family: 'font-bebas', sans-serif; letter-spacing: 0.04em; }
  .font-playfair { font-family: 'font-playfair', serif; }
  .font-jetbrains { font-family: 'font-jetbrains', monospace; }
  .font-caveat { font-family: 'font-caveat', cursive; }

  @media (max-width: 900px) {
    .main-layout {
      grid-template-columns: 1fr;
    }
    .preview-column {
      position: static;
    }
  }
</style>
