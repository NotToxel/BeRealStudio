<script lang="ts">
  import type { ProcessingStage } from '$lib/types';
  import Check from 'lucide-svelte/icons/check';
  import Loader2 from 'lucide-svelte/icons/loader-circle';
  import Circle from 'lucide-svelte/icons/circle';

  export let stage: ProcessingStage = 'Scanning';
  export let current: number = 0;
  export let total: number = 0;
  export let percentage: number = 0;
  export let currentFile: string | undefined = undefined;

  function stageLabel(s: ProcessingStage): string {
    switch (s) {
      case 'Extracting': return 'Extracting Archive';
      case 'Scanning': return 'Scanning Archive';
      case 'Parsing': return 'Parsing Posts JSON';
      case 'Converting': return 'Converting & Processing Photos';
      case 'Compositing': return 'Compositing Combined Memories';
      case 'WritingExif': return 'Writing Metadata & EXIF';
      case 'Cleanup': return 'Cleaning Up Intermediates';
      case 'Complete': return 'Processing Finished';
      case 'LoadingAudio': return 'Analyzing Audio & Beats';
      case 'Geocoding': return 'Geocoding Locations';
      case 'RenderingFrames': return 'Rendering Frames & Overlays';
      case 'EncodingVideo': return 'Encoding Video with FFmpeg';
      default: return String(s);
    }
  }

  // Detect pipeline mode based on stage
  $: isRecapper = ['LoadingAudio', 'Geocoding', 'RenderingFrames', 'EncodingVideo'].includes(stage);

  interface StageDef {
    id: string;
    name: string;
    startPct: number;
    endPct: number;
  }

  $: stages = isRecapper
    ? [
        { id: 'audio', name: 'Audio Analysis', startPct: 0, endPct: 15 },
        { id: 'geo', name: 'Reverse Geocoding', startPct: 15, endPct: 30 },
        { id: 'render', name: 'Frame Rendering', startPct: 30, endPct: 70 },
        { id: 'encode', name: 'FFmpeg Encoding', startPct: 70, endPct: 100 },
      ]
    : [
        { id: 'scan', name: 'Archive Scan', startPct: 0, endPct: 10 },
        { id: 'convert', name: 'Photo Conversion', startPct: 10, endPct: 65 },
        { id: 'composite', name: 'Camera Compositing', startPct: 65, endPct: 95 },
        { id: 'finish', name: 'Finalization', startPct: 95, endPct: 100 },
      ];

  function getStageProgress(st: StageDef, overallPct: number) {
    if (overallPct >= st.endPct) return 100;
    if (overallPct <= st.startPct) return 0;
    return Math.min(100, Math.max(0, ((overallPct - st.startPct) / (st.endPct - st.startPct)) * 100));
  }
</script>

<div class="progress-wrapper card">
  <!-- Unified Top Progress Bar -->
  <div class="header">
    <div class="stage-info">
      <div class="title-row">
        <span class="stage-name">{stageLabel(stage)}</span>
        <span class="badge badge-yellow stage-pill font-mono">{stage}</span>
      </div>
      {#if currentFile}
        <span class="file-name" title={currentFile}>{currentFile}</span>
      {/if}
    </div>
    <div class="stats">
      {#if total > 0}
        <span class="fraction">{current} / {total} items</span>
      {/if}
      <span class="percent font-mono">{percentage.toFixed(2)}%</span>
    </div>
  </div>

  <div class="track">
    <div class="fill" style="width: {Math.min(Math.max(percentage, 0), 100)}%;"></div>
  </div>

  <!-- Distinct Multi-Stage Pipeline Progress Bars -->
  <div class="stages-breakdown">
    {#each stages as st, idx}
      {@const stageProgress = getStageProgress(st, percentage)}
      {@const isDone = stageProgress >= 100}
      {@const isActive = stageProgress > 0 && stageProgress < 100}
      {@const isPending = stageProgress === 0}

      <div class="stage-card" class:is-active={isActive} class:is-done={isDone} class:is-pending={isPending}>
        <div class="stage-card-head">
          <div class="stage-indicator">
            {#if isDone}
              <div class="indicator-icon icon-done">
                <Check size={11} strokeWidth={3} />
              </div>
            {:else if isActive}
              <div class="indicator-icon icon-active">
                <Loader2 size={11} class="animate-spin text-amber-400" />
              </div>
            {:else}
              <div class="indicator-icon icon-pending">
                <span class="step-num">{idx + 1}</span>
              </div>
            {/if}
            <span class="stage-step-name">{st.name}</span>
          </div>

          <span class="stage-pct-label font-mono">
            {stageProgress.toFixed(1)}%
          </span>
        </div>

        <div class="stage-track">
          <div
            class="stage-fill"
            class:fill-done={isDone}
            class:fill-active={isActive}
            style="width: {stageProgress}%;"
          ></div>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .progress-wrapper {
    display: flex;
    flex-direction: column;
    gap: 16px;
    background: #111116;
    border: 1px solid var(--border-subtle);
    padding: 20px 22px;
    border-radius: var(--radius-lg);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .stage-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .stage-name {
    font-size: 15px;
    font-weight: 700;
    color: var(--text-main);
  }

  .stage-pill {
    font-size: 11px;
  }

  .file-name {
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    max-width: 420px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stats {
    display: flex;
    align-items: baseline;
    gap: 12px;
    flex-shrink: 0;
  }

  .fraction {
    font-size: 13px;
    color: var(--text-secondary);
    font-family: var(--font-mono);
  }

  .percent {
    font-size: 22px;
    font-weight: 800;
    color: #ffe600;
    font-family: var(--font-mono);
    text-shadow: 0 0 16px rgba(255, 230, 0, 0.4);
  }

  .track {
    width: 100%;
    height: 10px;
    background: #181822;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    overflow: hidden;
    position: relative;
  }

  .fill {
    height: 100%;
    background: linear-gradient(90deg, #f59e0b 0%, #ffe600 50%, #10b981 100%);
    border-radius: 999px;
    box-shadow: 0 0 14px rgba(255, 230, 0, 0.6);
    transition: width 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  /* Distinct Stages Breakdown Grid */
  .stages-breakdown {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 10px;
    margin-top: 4px;
  }

  @media (max-width: 860px) {
    .stages-breakdown {
      grid-template-columns: 1fr 1fr;
    }
  }

  .stage-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: #0b0b0f;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 10px 12px;
    transition: all var(--transition-fast);
  }

  .stage-card.is-active {
    border-color: rgba(245, 158, 11, 0.5);
    background: rgba(245, 158, 11, 0.06);
    box-shadow: 0 0 14px rgba(245, 158, 11, 0.1);
  }

  .stage-card.is-done {
    border-color: rgba(52, 211, 153, 0.3);
    background: rgba(52, 211, 153, 0.04);
  }

  .stage-card.is-pending {
    opacity: 0.55;
  }

  .stage-card-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 6px;
  }

  .stage-indicator {
    display: flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
  }

  .indicator-icon {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .icon-done {
    background: #10b981;
    color: #000000;
  }

  .icon-active {
    background: rgba(245, 158, 11, 0.2);
    border: 1px solid rgba(245, 158, 11, 0.5);
  }

  .icon-pending {
    background: #1a1a24;
    border: 1px solid var(--border-subtle);
  }

  .step-num {
    font-size: 9.5px;
    font-weight: 700;
    color: var(--text-muted);
  }

  .stage-step-name {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .stage-pct-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .stage-card.is-active .stage-pct-label {
    color: #fbbf24;
  }

  .stage-card.is-done .stage-pct-label {
    color: #34d399;
  }

  .stage-track {
    width: 100%;
    height: 4px;
    background: #181822;
    border-radius: 999px;
    overflow: hidden;
  }

  .stage-fill {
    height: 100%;
    border-radius: 999px;
    transition: width 0.18s ease;
  }

  .stage-fill.fill-active {
    background: #f59e0b;
    box-shadow: 0 0 6px rgba(245, 158, 11, 0.8);
  }

  .stage-fill.fill-done {
    background: #10b981;
  }
</style>
