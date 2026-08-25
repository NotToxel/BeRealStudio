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
  import FilePicker from '$components/FilePicker.svelte';
  import Toggle from '$components/Toggle.svelte';
  import FontPicker from '$components/FontPicker.svelte';
  import FontSizePicker from '$components/FontSizePicker.svelte';
  import Stepper from '$components/Stepper.svelte';
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
  $: previewShadow = Math.max(Math.round($recapperConfig.shadowStrength * PREVIEW_SCALE * 1.5), 0);

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

  function insertToken(token: string) {
    $recapperConfig.dateFormat = ($recapperConfig.dateFormat || '') + token;
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
      <span class="badge badge-violet">Live Preview</span>
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
          label="Output Video File"
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
            max={20}
            step={1}
            unit="px"
            presets={[
              { label: 'None', value: 0 },
              { label: 'Subtle', value: 3 },
              { label: 'Medium', value: 6 },
              { label: 'Strong', value: 10 },
            ]}
          />
        </div>

        <div class="divider"></div>

        <!-- Date Overlay -->
        <Toggle
          label="Display Date Stamp"
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
                <span class="label">Insert Format Tokens</span>
                <button type="button" class="btn-token-reset" on:click={resetDateFormat}>
                  <RotateCcw size={11} /> Reset
                </button>
              </div>
              <div class="tokens-wrap">
                {#each DATE_TOKENS as tok}
                  <button
                    type="button"
                    class="token-btn"
                    on:click={() => insertToken(tok.token)}
                    title="Insert {tok.token}"
                  >
                    <Plus size={11} class="text-purple-400" />
                    <span>{tok.label}</span>
                    <code class="token-code">{tok.token}</code>
                  </button>
                {/each}
              </div>
            </div>

            <div class="options-grid">
              <div class="field-group">
                <label for="date-fmt-input" class="label">Custom Pattern</label>
                <input
                  id="date-fmt-input"
                  type="text"
                  class="input-text font-mono"
                  bind:value={$recapperConfig.dateFormat}
                  placeholder="%d %B %Y"
                />
              </div>

              <div class="field-group">
                <label for="date-pos-select" class="label">Date Position</label>
                <select id="date-pos-select" class="input-select" bind:value={$recapperConfig.datePosition}>
                  <option value="BottomCenter">Bottom Center (Default)</option>
                  <option value="BottomLeft">Bottom Left</option>
                  <option value="BottomRight">Bottom Right</option>
                  <option value="TopRight">Top Right</option>
                </select>
              </div>
            </div>
          </div>
        {/if}

        <!-- Location Overlay -->
        <Toggle
          label="Display Location"
          bind:checked={$recapperConfig.locationEnabled}
          accentColor="emerald"
        />

        {#if $recapperConfig.locationEnabled}
          <div class="sub-options-box">
            <div class="options-grid">
              <div class="field-group">
                <label for="geo-mode-select" class="label">Geocoding Service</label>
                <select id="geo-mode-select" class="input-select" bind:value={$recapperConfig.geocodingMode}>
                  <option value="Online">Nominatim OpenStreetMap (Online)</option>
                  <option value="Offline">Offline Reverse Geocoding DB</option>
                </select>
              </div>

              <div class="field-group">
                <label for="loc-pos-select" class="label">Location Position</label>
                <select id="loc-pos-select" class="input-select" bind:value={$recapperConfig.locationPosition}>
                  <option value="BelowDate">Below Date (Default)</option>
                  <option value="AboveDate">Above Date</option>
                  <option value="BottomCenter">Bottom Center</option>
                  <option value="BottomLeft">Bottom Left</option>
                  <option value="BottomRight">Bottom Right</option>
                  <option value="TopRight">Top Right</option>
                </select>
              </div>
            </div>

            <RuleEditor bind:rules={$recapperConfig.locationRules} />
          </div>
        {/if}
      </div>

      <!-- 3. Pacing & Timing Settings -->
      <div class="card section-card">
        <div class="section-title-row">
          <Activity size={18} class="text-sky-400" />
          <h2 class="title-sm">3. Speed Transitions &amp; Pacing</h2>
        </div>

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
            </button>
          {/each}
        </div>

        <div class="options-grid">
          <Stepper
            label="Start Padding"
            bind:value={$recapperConfig.startPadding}
            min={0}
            max={10}
            step={0.5}
            unit="s"
            presets={[
              { label: '0s', value: 0 },
              { label: '1s', value: 1 },
              { label: '2s', value: 2 },
              { label: '3s', value: 3 },
            ]}
          />

          <Stepper
            label="End Padding"
            bind:value={$recapperConfig.endPadding}
            min={0}
            max={10}
            step={0.5}
            unit="s"
            presets={[
              { label: '0s', value: 0 },
              { label: '1s', value: 1 },
              { label: '2s', value: 2 },
              { label: '3s', value: 3 },
            ]}
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

      <!-- Start Button -->
      <div class="action-footer">
        <button
          type="button"
          class="btn btn-accent-violet btn-lg"
          class:btn-disabled-look={!isConfigValid}
          on:click={handleStart}
        >
          <Play size={16} fill="currentColor" />
          <span>Generate Recap Video &rarr;</span>
        </button>
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
          <span class="badge badge-violet">1440 &times; 1920</span>
        </div>

        <!-- Phone Mockup Canvas -->
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
                      filter: drop-shadow(0px {previewShadow}px {previewShadow}px rgba(0, 0, 0, 0.95));
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
                      filter: drop-shadow(0px {previewShadow}px {previewShadow}px rgba(0, 0, 0, 0.95));
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
                      filter: drop-shadow(0px {previewShadow}px {previewShadow}px rgba(0, 0, 0, 0.95));
                    "
                  >
                    {sampleLocation}
                  </div>
                {/if}
              </div>
            </div>
          </div>
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
    grid-template-columns: 1fr 340px;
    gap: 24px;
    align-items: start;
  }

  .form-column {
    display: flex;
    flex-direction: column;
    gap: 20px;
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
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    align-items: center;
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

  .sub-options-box {
    background: #0e0e11;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
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

  /* Speed Modes Grid */
  .speed-modes-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 8px;
  }

  .speed-mode-card {
    display: flex;
    flex-direction: column;
    gap: 4px;
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
    background: rgba(139, 92, 246, 0.14);
    border-color: rgba(139, 92, 246, 0.45);
    box-shadow: 0 0 10px rgba(139, 92, 246, 0.12);
  }

  .mode-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .mode-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-main);
  }

  .active-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #c084fc;
  }

  .mode-desc {
    font-size: 11px;
    line-height: 1.3;
  }

  /* FPS Pills */
  .fps-pills {
    display: flex;
    gap: 8px;
  }

  .fps-pill {
    padding: 6px 14px;
    background: #0f0f13;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .fps-pill:hover {
    background: #181820;
    color: var(--text-main);
  }

  .fps-pill.active {
    background: rgba(56, 189, 248, 0.18);
    color: #38bdf8;
    border-color: rgba(56, 189, 248, 0.4);
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
    border: 1px solid rgba(139, 92, 246, 0.25);
    box-shadow: 0 8px 32px rgba(139, 92, 246, 0.08);
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .preview-dot {
    width: 8px;
    height: 8px;
    background: #34d399;
    border-radius: 50%;
    box-shadow: 0 0 8px rgba(52, 211, 153, 0.8);
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .mockup-frame {
    width: 100%;
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
    background: linear-gradient(135deg, #1e293b 0%, #0f172a 50%, #090d16 100%);
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
    transition: font-size 0.1s ease, filter 0.1s ease;
  }

  .live-location-text {
    font-weight: 500;
    opacity: 0.95;
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
