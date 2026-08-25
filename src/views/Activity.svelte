<script lang="ts">
  import {
    currentView,
    activeJobs,
    cancelActiveJobById,
    removeActiveJob,
    activityHistory,
    clearActivityHistory,
    deleteActivityRecord,
    isProcessing,
    progressState,
    activeFeature,
  } from '$lib/stores';
  import { openPath, cancelJob, cancelToolkit, cancelRecapper } from '$lib/tauri';
  import type { ActiveJob } from '$lib/types';
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
  import Loader2 from 'lucide-svelte/icons/loader-circle';
  import XCircle from 'lucide-svelte/icons/circle-x';
  import Terminal from 'lucide-svelte/icons/terminal';
  import ChevronDown from 'lucide-svelte/icons/chevron-down';
  import ChevronUp from 'lucide-svelte/icons/chevron-up';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import Layers from 'lucide-svelte/icons/layers';

  let showClearConfirm = false;
  let expandedLogs: Record<string, boolean> = {};

  function toggleLogs(jobId: string) {
    expandedLogs[jobId] = !expandedLogs[jobId];
    expandedLogs = { ...expandedLogs };
  }

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

  function getRelativeTime(isoStr: string | number): string {
    try {
      const timestamp = typeof isoStr === 'number' ? isoStr : new Date(isoStr).getTime();
      const diffMs = Date.now() - timestamp;
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

  async function handleCancelSpecificJob(job: ActiveJob) {
    try {
      await cancelJob(job.id);
      cancelActiveJobById(job.id);
    } catch (e) {
      console.warn(`Failed to cancel job ${job.id}:`, e);
    }
  }

  async function handleCancelLegacy() {
    try {
      if ($activeFeature === 'toolkit') {
        await cancelToolkit();
      } else {
        await cancelRecapper();
      }
    } catch (e) {
      console.warn('Failed to cancel active task:', e);
    } finally {
      isProcessing.set(false);
    }
  }

  $: runningJobs = $activeJobs.filter((j) => j.status === 'running');
  $: recentJobs = $activeJobs.filter((j) => j.status !== 'running');
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
      <span class="badge badge-sky font-mono">
        {$activityHistory.length} {$activityHistory.length === 1 ? 'Run' : 'Runs'}
      </span>
    </div>
    {#if $activityHistory.length > 0}
      <button
        type="button"
        class="btn btn-ghost btn-sm text-muted hover:text-red-400"
        on:click={() => (showClearConfirm = true)}
        title="Clear All Generation History"
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

  <!-- Active Concurrent Jobs Queue -->
  {#if runningJobs.length > 0}
    <div class="active-jobs-section">
      <div class="section-header">
        <div class="section-header-left">
          <Sparkles size={16} class="text-amber-400" />
          <h2 class="title-sm font-bold text-white">Active Operations Queue</h2>
          <span class="badge badge-yellow font-mono">{runningJobs.length} {runningJobs.length === 1 ? 'Job' : 'Jobs'} Running in Parallel</span>
        </div>
      </div>

      <div class="active-jobs-grid">
        {#each runningJobs as job (job.id)}
          <div class="active-job-card card">
            <div class="job-card-head">
              <div class="job-info-left">
                <div class="job-type-icon {job.type === 'toolkit' ? 'icon-yellow' : 'icon-violet'}">
                  {#if job.type === 'toolkit'}
                    <Camera size={16} />
                  {:else}
                    <Film size={16} />
                  {/if}
                </div>
                <div class="job-titles">
                  <div class="job-title-row">
                    <span class="job-title font-bold text-white">{job.title}</span>
                    <span class="badge badge-{job.type === 'toolkit' ? 'yellow' : 'violet'} font-mono">
                      {job.stage}
                    </span>
                  </div>
                  <div class="job-meta-row text-xs text-muted">
                    <span>Started {getRelativeTime(job.startTime)}</span>
                    <span>&bull;</span>
                    <span class="font-mono">{job.current} / {job.total} items</span>
                    {#if job.currentFile}
                      <span>&bull;</span>
                      <span class="job-current-file font-mono text-secondary" title={job.currentFile}>
                        {job.currentFile}
                      </span>
                    {/if}
                  </div>
                </div>
              </div>

              <div class="job-actions-right">
                <button
                  type="button"
                  class="btn btn-ghost btn-sm"
                  on:click={() => toggleLogs(job.id)}
                  title="Toggle Live Log Output"
                >
                  <Terminal size={13} />
                  <span>Logs ({job.logs.length})</span>
                  {#if expandedLogs[job.id]}
                    <ChevronUp size={12} />
                  {:else}
                    <ChevronDown size={12} />
                  {/if}
                </button>
                <button
                  type="button"
                  class="btn btn-danger btn-sm"
                  on:click={() => handleCancelSpecificJob(job)}
                >
                  <XCircle size={13} />
                  <span>Cancel</span>
                </button>
              </div>
            </div>

            <!-- Multi-Stage Progress Bar -->
            <div class="job-progress-wrap">
              <div class="job-progress-info">
                <span class="job-progress-label text-xs text-secondary font-medium">Pipeline Execution</span>
                <span class="job-progress-pct font-mono font-bold text-amber-400">{job.percentage.toFixed(2)}%</span>
              </div>
              <div class="job-track">
                <div class="job-fill {job.type === 'toolkit' ? 'fill-yellow' : 'fill-violet'}" style="width: {Math.min(Math.max(job.percentage, 0), 100)}%;"></div>
              </div>
            </div>

            <!-- Expandable Live Terminal Console -->
            {#if expandedLogs[job.id]}
              <div class="job-logs-console">
                <div class="logs-console-head">
                  <span class="logs-console-title font-mono text-xs text-muted">Real-Time Event Stream</span>
                  <span class="badge badge-dark font-mono text-xs">{job.logs.length} events</span>
                </div>
                <div class="logs-scroll-area">
                  {#if job.logs.length === 0}
                    <div class="log-empty font-mono text-xs text-muted">Awaiting telemetry logs...</div>
                  {:else}
                    {#each job.logs as log}
                      <div class="log-line log-{log.level.toLowerCase()}">
                        <span class="log-ts font-mono">[{log.timestamp.slice(11, 19)}]</span>
                        <span class="log-lvl font-mono">[{log.level}]</span>
                        <span class="log-msg font-mono">{log.message}</span>
                      </div>
                    {/each}
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Single-Job Legacy Active Banner (if applicable) -->
  {#if $isProcessing && runningJobs.length === 0}
    <div class="active-task-banner card">
      <div class="active-task-head">
        <div class="active-task-left">
          <div class="active-task-spinner">
            <Loader2 size={20} class="animate-spin text-amber-400" />
          </div>
          <div class="active-task-titles">
            <div class="title-row">
              <span class="active-title font-bold text-white">
                {$activeFeature === 'toolkit' ? 'Photo Processing Suite' : 'Recap Video Generator'}
              </span>
              <span class="badge badge-yellow font-mono">{$progressState.stage || 'Processing'}</span>
            </div>
            <span class="active-desc text-secondary text-xs font-mono">
              {$progressState.current} of {$progressState.total} items completed &bull; <strong class="text-amber-300 font-mono">{$progressState.percentage.toFixed(2)}%</strong>
            </span>
          </div>
        </div>

        <div class="active-task-actions">
          <button type="button" class="btn btn-accent-yellow btn-sm" on:click={() => currentView.set('processing')}>
            <ExternalLink size={13} />
            <span>View Live Terminal</span>
          </button>
          <button type="button" class="btn btn-danger btn-sm" on:click={handleCancelLegacy}>
            <XCircle size={13} />
            <span>Cancel</span>
          </button>
        </div>
      </div>

      <div class="active-task-track">
        <div class="active-task-fill" style="width: {Math.min(Math.max($progressState.percentage, 0), 100)}%;"></div>
      </div>
    </div>
  {/if}

  <!-- Recent Session Results -->
  {#if recentJobs.length > 0}
    <div class="recent-session-section">
      <div class="section-header">
        <div class="section-header-left">
          <Layers size={16} class="text-sky-400" />
          <h2 class="title-sm font-bold text-white">Session Tasks</h2>
        </div>
      </div>

      <div class="recent-jobs-list">
        {#each recentJobs as job (job.id)}
          <div class="recent-job-card card">
            <div class="recent-job-left">
              <div class="status-indicator status-{job.status}">
                {#if job.status === 'completed'}
                  <CheckCircle size={15} class="text-emerald-400" />
                {:else if job.status === 'cancelled'}
                  <XCircle size={15} class="text-amber-400" />
                {:else}
                  <AlertTriangle size={15} class="text-red-400" />
                {/if}
              </div>
              <div class="recent-job-titles">
                <span class="font-semibold text-white text-sm">{job.title}</span>
                <span class="text-xs text-muted font-mono">
                  {job.status === 'completed' ? 'Finished successfully' : job.status === 'cancelled' ? 'Cancelled by user' : job.errorMessage || 'Error encountered'} &bull; {getRelativeTime(job.startTime)}
                </span>
              </div>
            </div>

            <div class="recent-job-actions">
              {#if job.status === 'completed' && job.outputPath}
                <button
                  type="button"
                  class="btn btn-secondary btn-sm"
                  on:click={() => handleOpen(job.outputPath)}
                >
                  <FolderOpen size={13} />
                  <span>Open Folder</span>
                </button>
              {/if}
              <button
                type="button"
                class="btn-delete"
                title="Dismiss Task"
                on:click={() => removeActiveJob(job.id)}
              >
                <Trash2 size={13} />
              </button>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- History Items List -->
  {#if $activityHistory.length === 0 && runningJobs.length === 0 && recentJobs.length === 0}
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
  {:else if $activityHistory.length > 0}
    <div class="history-list">
      <div class="section-header">
        <div class="section-header-left">
          <History size={16} class="text-secondary" />
          <h2 class="title-sm font-bold text-white">Archived Generation History</h2>
        </div>
      </div>

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
                <CheckCircle size={11} /> {item.itemCount} {item.itemCount === 1 ? 'Item' : 'Items'}
              </span>
              <button
                type="button"
                class="btn-delete"
                title="Remove From History"
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

  /* Active Task Banner */
  .active-task-banner {
    background: #15151c;
    border: 1px solid rgba(245, 158, 11, 0.4);
    border-radius: var(--radius-lg);
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: 0 4px 24px rgba(245, 158, 11, 0.12);
  }

  .active-task-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }

  .active-task-left {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
  }

  .active-task-spinner {
    width: 38px;
    height: 38px;
    border-radius: var(--radius-md);
    background: rgba(245, 158, 11, 0.12);
    border: 1px solid rgba(245, 158, 11, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .active-task-titles {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .active-title {
    font-size: 15px;
  }

  .active-task-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .active-task-track {
    width: 100%;
    height: 6px;
    background: #09090c;
    border-radius: 999px;
    overflow: hidden;
  }

  .active-task-fill {
    height: 100%;
    background: linear-gradient(90deg, #f59e0b, #ffe600);
    border-radius: 999px;
    box-shadow: 0 0 10px rgba(255, 230, 0, 0.6);
    transition: width 0.18s ease;
  }

  /* Active Jobs Section */
  .active-jobs-section {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 2px;
  }

  .section-header-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .active-jobs-grid {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .active-job-card {
    background: #14141c;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-lg);
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
    position: relative;
    overflow: hidden;
  }

  .active-job-card::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(90deg, #ffe600, #c084fc, #38bdf8);
  }

  .job-card-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }

  .job-info-left {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .job-type-icon {
    width: 36px;
    height: 36px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .job-titles {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .job-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .job-title {
    font-size: 14.5px;
  }

  .job-meta-row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .job-current-file {
    max-width: 260px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .job-actions-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .job-progress-wrap {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .job-progress-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .job-track {
    width: 100%;
    height: 6px;
    background: #0a0a0e;
    border-radius: 999px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.04);
  }

  .job-fill {
    height: 100%;
    border-radius: 999px;
    transition: width 0.15s ease;
  }

  .fill-yellow {
    background: linear-gradient(90deg, #f59e0b, #ffe600);
    box-shadow: 0 0 10px rgba(255, 230, 0, 0.5);
  }

  .fill-violet {
    background: linear-gradient(90deg, #8b5cf6, #c084fc);
    box-shadow: 0 0 10px rgba(192, 132, 252, 0.5);
  }

  /* Log Console inside Job Card */
  .job-logs-console {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: #09090d;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 10px 14px;
    margin-top: 2px;
  }

  .logs-console-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    padding-bottom: 6px;
  }

  .logs-scroll-area {
    display: flex;
    flex-direction: column;
    gap: 3px;
    max-height: 140px;
    overflow-y: auto;
  }

  .log-line {
    font-size: 11px;
    line-height: 1.4;
    display: flex;
    gap: 6px;
  }

  .log-ts { color: #64748b; flex-shrink: 0; }
  .log-lvl { font-weight: 600; flex-shrink: 0; }
  .log-info { color: #94a3b8; }
  .log-info .log-lvl { color: #38bdf8; }
  .log-warn { color: #fde047; }
  .log-warn .log-lvl { color: #facc15; }
  .log-error { color: #f87171; }
  .log-error .log-lvl { color: #ef4444; }

  /* Recent Session Section */
  .recent-session-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .recent-jobs-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .recent-job-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #111116;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    gap: 12px;
  }

  .recent-job-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .status-indicator {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #181822;
  }

  .recent-job-titles {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .recent-job-actions {
    display: flex;
    align-items: center;
    gap: 8px;
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
