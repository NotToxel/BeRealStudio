<script lang="ts">
  import type { ProcessingStage } from '$lib/types';

  export let stage: ProcessingStage = 'Scanning';
  export let current: number = 0;
  export let total: number = 0;
  export let percentage: number = 0;
  export let currentFile: string | undefined = undefined;

  function stageLabel(s: ProcessingStage): string {
    switch (s) {
      case 'Scanning': return 'Scanning Archive';
      case 'Parsing': return 'Parsing Posts JSON';
      case 'Converting': return 'Converting & Processing Photos';
      case 'Compositing': return 'Compositing Combined Memories';
      case 'WritingExif': return 'Writing Metadata & EXIF';
      case 'Cleanup': return 'Cleaning Up Intermediates';
      case 'Complete': return 'Processing Finished';
      case 'LoadingAudio': return 'Analyzing Audio';
      case 'Geocoding': return 'Geocoding Locations';
      case 'RenderingFrames': return 'Rendering Frames & Overlays';
      case 'EncodingVideo': return 'Encoding Video with FFmpeg';
      default: return String(s);
    }
  }
</script>

<div class="progress-wrapper card">
  <div class="header">
    <div class="stage-info">
      <span class="stage-name">{stageLabel(stage)}</span>
      {#if currentFile}
        <span class="file-name" title={currentFile}>{currentFile}</span>
      {/if}
    </div>
    <div class="stats">
      {#if total > 0}
        <span class="fraction">{current} / {total}</span>
      {/if}
      <span class="percent">{Math.round(percentage)}%</span>
    </div>
  </div>

  <div class="track">
    <div class="fill" style="width: {Math.min(Math.max(percentage, 0), 100)}%;"></div>
  </div>
</div>

<style>
  .progress-wrapper {
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: var(--bg-card);
    padding: 18px 20px;
    border-radius: var(--radius-lg);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .stage-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stage-name {
    font-size: 14.5px;
    font-weight: 600;
    color: var(--text-main);
  }

  .file-name {
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    max-width: 450px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stats {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }

  .fraction {
    font-size: 13px;
    color: var(--text-secondary);
    font-family: var(--font-mono);
  }

  .percent {
    font-size: 18px;
    font-weight: 700;
    color: var(--text-main);
    font-family: var(--font-mono);
  }

  .track {
    width: 100%;
    height: 8px;
    background: #25252c;
    border-radius: 999px;
    overflow: hidden;
  }

  .fill {
    height: 100%;
    background: #ffffff;
    border-radius: 999px;
    transition: width 0.15s cubic-bezier(0.16, 1, 0.3, 1);
  }
</style>
