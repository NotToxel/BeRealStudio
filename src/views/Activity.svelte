<script lang="ts">
  import {
    currentView,
    activityHistory,
    clearActivityHistory,
    deleteActivityRecord,
  } from '$lib/stores';
  import { openPath } from '$lib/tauri';
  import History from 'lucide-svelte/icons/history';
  import FolderOpen from 'lucide-svelte/icons/folder-open';
  import ExternalLink from 'lucide-svelte/icons/external-link';
  import Trash2 from 'lucide-svelte/icons/trash-2';
  import Film from 'lucide-svelte/icons/film';
  import Camera from 'lucide-svelte/icons/camera';
  import ArrowLeft from 'lucide-svelte/icons/arrow-left';
  import CheckCircle from 'lucide-svelte/icons/circle-check';
  import Clock from 'lucide-svelte/icons/clock';
  import AlertTriangle from 'lucide-svelte/icons/triangle-alert';

  let showClearConfirm = false;

  function formatTime(isoStr: string): string {
    try {
      const d = new Date(isoStr);
      return d.toLocaleString('en-GB', {
        day: 'numeric',
        month: 'short',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return isoStr;
    }
  }

  function getRelativeTime(isoStr: string): string {
    try {
      const diffMs = Date.now() - new Date(isoStr).getTime();
      const diffMins = Math.floor(diffMs / 60000);
      if (diffMins < 1) return 'Just now';
      if (diffMins < 60) return `${diffMins}m ago`;
      const diffHours = Math.floor(diffMins / 60);
      if (diffHours < 24) return `${diffHours}h ago`;
      const diffDays = Math.floor(diffHours / 24);
      return `${diffDays}d ago`;
    } catch {
      return '';
    }
  }

  async function handleOpen(path: string) {
    if (!path) return;
    try {
      await openPath(path);
    } catch (e) {
      alert(`Could not open path:\n${path}\n\nError: ${e}`);
    }
  }

  function handleClearAll() {
    clearActivityHistory();
    showClearConfirm = false;
  }
</script>

<div class="activity-view">
  <!-- Top Bar -->
  <div class="top-nav">
    <button type="button" class="btn btn-ghost btn-sm" on:click={() => currentView.set('home')}>
      <ArrowLeft size={14} />
      <span>Back to Home</span>
    </button>
    <div class="header-titles">
      <h1 class="title-md font-bold">Activity &amp; Generation History</h1>
      <span class="badge badge-sky font-mono">{$activityHistory.length} runs</span>
    </div>
    {#if $activityHistory.length > 0}
      <button
        type="button"
        class="btn btn-ghost btn-sm text-muted hover:text-red-400"
        on:click={() => (showClearConfirm = true)}
      >
        <Trash2 size={13} />
        <span>Clear History</span>
      </button>
    {/if}
  </div>

  <!-- Clear Confirmation Dialog Modal -->
  {#if showClearConfirm}
    <div class="modal-backdrop">
      <div class="modal-card">
        <div class="modal-head">
          <AlertTriangle size={20} class="text-amber-400" />
          <h3 class="title-sm font-bold text-white">Clear Activity History?</h3>
        </div>
        <p class="modal-body text-secondary text-sm">
          Are you sure you want to clear all generation logs and activity records? Your actual output files on disk will NOT be deleted.
        </p>
        <div class="modal-actions">
          <button type="button" class="btn btn-secondary btn-sm" on:click={() => (showClearConfirm = false)}>
            Cancel
          </button>
          <button type="button" class="btn btn-danger btn-sm" on:click={handleClearAll}>
            Yes, Clear History
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- History Items List -->
  {#if $activityHistory.length === 0}
    <div class="empty-state card">
      <div class="empty-icon-circle">
        <History size={28} class="text-secondary" />
      </div>
      <h2 class="title-sm font-semibold text-white">No Output Generations Yet</h2>
      <p class="empty-desc text-secondary text-sm">
        Whenever you process photos or generate a recap video, your outputs, timestamps, and quick file links will be logged here.
      </p>
      <div class="empty-actions">
        <button type="button" class="btn btn-accent-yellow btn-sm" on:click={() => currentView.set('toolkit-config')}>
          <Camera size={14} />
          <span>Process Photos</span>
        </button>
        <button type="button" class="btn btn-accent-violet btn-sm" on:click={() => currentView.set('recapper-config')}>
          <Film size={14} />
          <span>Create Recap Video</span>
        </button>
      </div>
    </div>
  {:else}
    <div class="history-list">
      {#each $activityHistory as item}
        <div class="activity-card card">
          <div class="activity-head">
            <div class="type-group">
              <div class="type-icon {item.type === 'toolkit' ? 'icon-yellow' : 'icon-violet'}">
                {#if item.type === 'toolkit'}
                  <Camera size={16} />
                {:else}
                  <Film size={16} />
                {/if}
              </div>
              <div class="type-titles">
                <span class="activity-title font-semibold text-white">{item.title}</span>
                <div class="time-row">
                  <Clock size={11} class="text-muted" />
                  <span class="time-text text-muted">{formatTime(item.timestamp)}</span>
                  <span class="rel-time text-muted font-mono">({getRelativeTime(item.timestamp)})</span>
                </div>
              </div>
            </div>

            <div class="head-right">
              <span class="badge badge-success font-mono">
                <CheckCircle size={11} /> {item.itemCount} items
              </span>
              <button
                type="button"
                class="btn-delete"
                title="Remove from history"
                on:click={() => deleteActivityRecord(item.id)}
              >
                <Trash2 size={13} />
              </button>
            </div>
          </div>

          {#if item.details}
            <div class="activity-details font-mono text-xs text-secondary">
              {item.details}
            </div>
          {/if}

          <!-- Paths & Quick Open Links -->
          <div class="paths-box">
            <div class="path-row">
              <span class="path-label">Output:</span>
              <span class="path-val font-mono" title={item.outputPath}>{item.outputPath}</span>
            </div>
            {#if item.inputPath}
              <div class="path-row">
                <span class="path-label">Source:</span>
                <span class="path-val font-mono text-muted" title={item.inputPath}>{item.inputPath}</span>
              </div>
            {/if}
          </div>

          <!-- Quick Action Buttons -->
          <div class="card-actions">
            <button
              type="button"
              class="btn btn-secondary btn-sm"
              on:click={() => handleOpen(item.outputPath)}
            >
              <FolderOpen size={13} />
              <span>Show in File Explorer</span>
            </button>

            <button
              type="button"
              class="btn btn-ghost btn-sm"
              on:click={() => handleOpen(item.outputPath)}
            >
              <ExternalLink size={13} />
              <span>Open Directly</span>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .activity-view {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-bottom: 40px;
  }

  .top-nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px;
  }

  .header-titles {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 14px;
    padding: 48px 24px;
    background: #111116;
  }

  .empty-icon-circle {
    width: 60px;
    height: 60px;
    border-radius: 50%;
    background: #181822;
    border: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .empty-desc {
    max-width: 440px;
    line-height: 1.5;
  }

  .empty-actions {
    display: flex;
    gap: 10px;
    margin-top: 8px;
    flex-wrap: wrap;
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .activity-card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: #111116;
    border: 1px solid var(--border-subtle);
    padding: 16px 20px;
    border-radius: var(--radius-md);
  }

  .activity-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
  }

  .type-group {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .type-icon {
    width: 36px;
    height: 36px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .icon-yellow {
    background: rgba(255, 230, 0, 0.12);
    color: #ffe600;
    border: 1px solid rgba(255, 230, 0, 0.25);
  }

  .icon-violet {
    background: rgba(139, 92, 246, 0.12);
    color: #c084fc;
    border: 1px solid rgba(139, 92, 246, 0.25);
  }

  .type-titles {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .activity-title {
    font-size: 13.5px;
  }

  .time-row {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
  }

  .rel-time {
    font-size: 10.5px;
  }

  .head-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn-delete {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 4px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    transition: all var(--transition-fast);
  }

  .btn-delete:hover {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
  }

  .activity-details {
    padding: 4px 8px;
    background: #0c0c10;
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.03);
    width: fit-content;
  }

  .paths-box {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: #0c0c10;
    border: 1px solid var(--border-subtle);
    padding: 8px 12px;
    border-radius: var(--radius-sm);
  }

  .path-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-size: 11.5px;
  }

  .path-label {
    color: var(--text-secondary);
    font-weight: 500;
    width: 50px;
    flex-shrink: 0;
  }

  .path-val {
    color: var(--text-main);
    word-break: break-all;
  }

  .card-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    padding-top: 4px;
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
