<script lang="ts">
  import { onMount, afterUpdate } from 'svelte';
  import type { LogEvent } from '$lib/types';

  export let logs: LogEvent[] = [];
  export let title: string = 'Activity Log';
  export let maxHeight: string = '240px';

  let logContainer: HTMLElement;
  let autoScroll = true;

  function handleScroll() {
    if (!logContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = logContainer;
    autoScroll = scrollHeight - (scrollTop + clientHeight) < 40;
  }

  afterUpdate(() => {
    if (autoScroll && logContainer) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  });

  function copyLogs() {
    const text = logs
      .map((l) => `[${l.timestamp}] [${l.level}] ${l.message}`)
      .join('\n');
    navigator.clipboard.writeText(text);
  }

  function formatTime(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toTimeString().split(' ')[0];
    } catch {
      return '';
    }
  }
</script>

<div class="console-card card">
  <div class="console-header">
    <div class="title-group">
      <span class="dot"></span>
      <span class="title-sm">{title}</span>
      <span class="log-count">{logs.length} events</span>
    </div>
    <button type="button" class="btn btn-ghost btn-sm" on:click={copyLogs} title="Copy logs to clipboard">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
      </svg>
      Copy
    </button>
  </div>

  <div
    class="log-viewer"
    style="max-height: {maxHeight};"
    bind:this={logContainer}
    on:scroll={handleScroll}
  >
    {#if logs.length === 0}
      <div class="empty-state">Waiting for activity...</div>
    {:else}
      {#each logs as log}
        <div class="log-line {log.level.toLowerCase()}">
          <span class="time">{formatTime(log.timestamp)}</span>
          <span class="level-tag">[{log.level}]</span>
          <span class="msg">{log.message}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .console-card {
    display: flex;
    flex-direction: column;
    background: #0d0d10;
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-subtle);
    padding: 0;
    overflow: hidden;
  }

  .console-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 16px;
    background: #131317;
    border-bottom: 1px solid var(--border-subtle);
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .dot {
    width: 7px;
    height: 7px;
    background: var(--status-success);
    border-radius: 50%;
  }

  .log-count {
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .log-viewer {
    padding: 12px 16px;
    overflow-y: auto;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.6;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .empty-state {
    color: var(--text-muted);
    font-style: italic;
    padding: 10px 0;
  }

  .log-line {
    display: flex;
    align-items: baseline;
    gap: 8px;
    word-break: break-all;
  }

  .time {
    color: var(--text-muted);
    font-size: 11px;
    flex-shrink: 0;
  }

  .level-tag {
    font-size: 10.5px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .msg {
    color: var(--text-main);
  }

  /* Log levels */
  .log-line.info .level-tag {
    color: var(--text-secondary);
  }

  .log-line.warn {
    color: var(--status-warning);
  }
  .log-line.warn .level-tag {
    color: var(--status-warning);
  }
  .log-line.warn .msg {
    color: var(--status-warning);
  }

  .log-line.error {
    color: var(--status-error);
  }
  .log-line.error .level-tag {
    color: var(--status-error);
  }
  .log-line.error .msg {
    color: var(--status-error);
  }
</style>
