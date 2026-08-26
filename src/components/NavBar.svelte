<script lang="ts">
  import { onMount } from 'svelte';
  import { APP_DISPLAY_VERSION } from '$lib/version';
  import { currentView, isProcessing, progressState, activeFeature, activityHistory, activeJobs, unreadActivityCount, currentArchive } from '$lib/stores';
  import { isDev, loadAllDemoData, clearAllDemoData } from '$lib/devMode';
  import type { ViewMode } from '$lib/types';
  import Home from 'lucide-svelte/icons/house';
  import Images from 'lucide-svelte/icons/images';
  import Film from 'lucide-svelte/icons/film';
  import Calendar from 'lucide-svelte/icons/calendar';
  import History from 'lucide-svelte/icons/history';
  import Settings from 'lucide-svelte/icons/settings';
  import Info from 'lucide-svelte/icons/info';
  import Loader2 from 'lucide-svelte/icons/loader-circle';
  import Sparkles from 'lucide-svelte/icons/sparkles';

  function navigate(mode: ViewMode) {
    currentView.set(mode);
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === ',') {
      e.preventDefault();
      navigate('settings');
      return;
    }
    if (isDev && e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'd') {
      e.preventDefault();
      if ($currentArchive) {
        clearAllDemoData();
      } else {
        loadAllDemoData();
      }
    }
  }

  $: runningJobs = $activeJobs.filter((j) => j.status === 'running');
</script>

<svelte:window on:keydown={handleKeydown} />

<header class="app-header">
  <div class="header-inner">
    <!-- Zone 1 (Left): Brand identity & Quick Home -->
    <div class="nav-zone-left">
      <button
        type="button"
        class="brand-btn"
        on:click={() => navigate('home')}
        title="Return to Home Dashboard"
      >
        <div class="logo-mark">
          <span class="logo-dot"></span>
        </div>
        <div class="brand-text">
          <span class="brand-title">BeReal Studio</span>
          <span class="brand-ver">{APP_DISPLAY_VERSION}</span>
        </div>
      </button>
    </div>

    <!-- Zone 2 (Center): Prominent Core Workflows -->
    <div class="nav-zone-center">
      <div class="primary-nav-group">
        <button
          type="button"
          class="core-nav-btn photos-btn"
          class:active={$currentView === 'toolkit-config'}
          on:click={() => navigate('toolkit-config')}
          title="Photos Processing Suite"
        >
          <Images size={15} />
          <span class="core-btn-label">Photos<span class="core-btn-sub"> Suite</span></span>
        </button>

        <button
          type="button"
          class="core-nav-btn memories-btn"
          class:active={$currentView === 'memories'}
          on:click={() => navigate('memories')}
          title="Memories Explorer"
        >
          <Calendar size={15} />
          <span class="core-btn-label">Memories</span>
        </button>

        <button
          type="button"
          class="core-nav-btn recap-btn"
          class:active={$currentView === 'recapper-config'}
          on:click={() => navigate('recapper-config')}
          title="Recap Video Generator"
        >
          <Film size={15} />
          <span class="core-btn-label">Recap<span class="core-btn-sub"> Video</span></span>
        </button>
      </div>
    </div>

    <!-- Zone 3 (Right): Auxiliary Navigation & Background Queue Pill -->
    <div class="nav-zone-right">
      {#if runningJobs.length > 1}
        <button
          type="button"
          class="bg-queue-pill multi-jobs"
          class:active={$currentView === 'activity'}
          on:click={() => currentView.set('activity')}
          title="{runningJobs.length} operations running in parallel — Click to manage queue"
        >
          <Sparkles size={13} class="text-amber-400 animate-pulse" />
          <div class="queue-text">
            <span class="queue-stage">{runningJobs.length} Parallel Jobs</span>
            <span class="queue-pct font-mono">Running</span>
          </div>
        </button>
      {:else if runningJobs.length === 1}
        <button
          type="button"
          class="bg-queue-pill"
          class:active={$currentView === 'activity' || $currentView === 'processing'}
          on:click={() => currentView.set('activity')}
          title="Background task in progress — Click to manage in Activity"
        >
          <Loader2 size={13} class="animate-spin text-amber-400" />
          <div class="queue-text">
            <span class="queue-stage">{runningJobs[0].stage || 'Processing'}</span>
            <span class="queue-pct font-mono">{runningJobs[0].percentage.toFixed(2)}%</span>
          </div>
        </button>
      {:else if $isProcessing}
        <button
          type="button"
          class="bg-queue-pill"
          class:active={$currentView === 'processing'}
          on:click={() => currentView.set('processing')}
          title="Background task in progress — Click to view live operation"
        >
          <Loader2 size={13} class="animate-spin text-amber-400" />
          <div class="queue-text">
            <span class="queue-stage">{$progressState.stage || 'Processing'}</span>
            <span class="queue-pct font-mono">{$progressState.percentage.toFixed(2)}%</span>
          </div>
        </button>
      {/if}

      <div class="aux-nav-group">
        <button
          type="button"
          class="aux-nav-item"
          class:active={$currentView === 'home'}
          on:click={() => navigate('home')}
          title="Home"
        >
          <Home size={15} />
          <span class="aux-label">Home</span>
        </button>

        <button
          type="button"
          class="aux-nav-item"
          class:active={$currentView === 'activity'}
          on:click={() => navigate('activity')}
          title="Activity & History"
        >
          <History size={15} />
          <span class="aux-label">Activity</span>
          {#if $unreadActivityCount > 0}
            <span class="nav-count-badge">{$unreadActivityCount}</span>
          {/if}
        </button>

        <button
          type="button"
          class="aux-nav-item"
          class:active={$currentView === 'settings'}
          on:click={() => navigate('settings')}
          title="Settings"
        >
          <Settings size={15} />
          <span class="aux-label">Settings</span>
        </button>

        <button
          type="button"
          class="aux-nav-item"
          class:active={$currentView === 'about'}
          on:click={() => navigate('about')}
          title="About & Privacy"
        >
          <Info size={15} />
          <span class="aux-label">About</span>
        </button>
      </div>
    </div>
  </div>
</header>

<style>
  .app-header {
    width: 100%;
    background: rgba(10, 10, 14, 0.92);
    backdrop-filter: blur(16px);
    border-bottom: 1px solid var(--border-subtle);
    position: sticky;
    top: 0;
    z-index: 100;
    box-sizing: border-box;
  }

  .header-inner {
    max-width: 1200px;
    margin: 0 auto;
    padding: 6px 14px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    box-sizing: border-box;
    width: 100%;
  }

  /* Zone 1: Left */
  .nav-zone-left {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    flex-shrink: 0;
  }

  .brand-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 4px 6px;
    border-radius: var(--radius-md);
    color: inherit;
    white-space: nowrap;
    flex-shrink: 0;
    transition: opacity var(--transition-fast);
  }

  .brand-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .logo-mark {
    width: 22px;
    height: 22px;
    background: linear-gradient(135deg, #1c1c24 0%, #121217 100%);
    border: 1px solid rgba(255, 230, 0, 0.4);
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 0 10px rgba(255, 230, 0, 0.2);
    flex-shrink: 0;
  }

  .logo-dot {
    width: 8px;
    height: 8px;
    background: #ffe600;
    border-radius: 50%;
    box-shadow: 0 0 8px rgba(255, 230, 0, 0.85);
  }

  .brand-text {
    display: flex;
    align-items: baseline;
    gap: 6px;
    white-space: nowrap;
  }

  .brand-title {
    font-size: 14px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-main);
    white-space: nowrap;
  }

  .brand-ver {
    font-size: 9.5px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    white-space: nowrap;
  }

  /* Zone 2: Center - Prominent Core Workflows */
  .nav-zone-center {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 1;
  }

  .primary-nav-group {
    display: flex;
    align-items: center;
    gap: 3px;
    background: #111116;
    padding: 3px 4px;
    border-radius: var(--radius-full);
    border: 1px solid rgba(255, 255, 255, 0.07);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .core-nav-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 13px;
    border-radius: var(--radius-full);
    font-size: 12.5px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text-secondary);
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    transition: all var(--transition-fast);
  }

  .core-btn-label {
    white-space: nowrap;
  }

  .core-nav-btn:hover:not(:disabled) {
    color: var(--text-main);
    background: rgba(255, 255, 255, 0.05);
  }

  .core-nav-btn.photos-btn.active {
    background: rgba(255, 230, 0, 0.16);
    color: #ffe600;
    border-color: rgba(255, 230, 0, 0.35);
    box-shadow: 0 0 14px rgba(255, 230, 0, 0.2);
  }

  .core-nav-btn.memories-btn.active {
    background: rgba(56, 189, 248, 0.18);
    color: #38bdf8;
    border-color: rgba(56, 189, 248, 0.4);
    box-shadow: 0 0 14px rgba(56, 189, 248, 0.2);
  }

  .core-nav-btn.recap-btn.active {
    background: rgba(139, 92, 246, 0.18);
    color: #c084fc;
    border-color: rgba(139, 92, 246, 0.4);
    box-shadow: 0 0 14px rgba(139, 92, 246, 0.2);
  }

  /* Zone 3: Right - Auxiliary */
  .nav-zone-right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    flex-shrink: 0;
  }

  .bg-queue-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 9px;
    background: rgba(245, 158, 11, 0.12);
    border: 1px solid rgba(245, 158, 11, 0.4);
    border-radius: var(--radius-full);
    color: #fbbf24;
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    transition: all var(--transition-fast);
    animation: queuePulse 2s infinite ease-in-out;
  }

  .bg-queue-pill:hover {
    background: rgba(245, 158, 11, 0.22);
    border-color: rgba(245, 158, 11, 0.7);
    transform: translateY(-1px);
    box-shadow: 0 0 12px rgba(245, 158, 11, 0.3);
  }

  .bg-queue-pill.active {
    background: rgba(245, 158, 11, 0.28);
    border-color: #f59e0b;
    box-shadow: 0 0 14px rgba(245, 158, 11, 0.4);
  }

  .bg-queue-pill.multi-jobs {
    background: linear-gradient(135deg, rgba(245, 158, 11, 0.18), rgba(168, 85, 247, 0.18));
    border-color: rgba(168, 85, 247, 0.5);
    box-shadow: 0 0 14px rgba(168, 85, 247, 0.2);
  }

  .queue-text {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11.5px;
    line-height: 1;
    white-space: nowrap;
  }

  .queue-stage {
    font-weight: 500;
    color: #fef08a;
    white-space: nowrap;
    max-width: 90px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .queue-pct {
    font-weight: 700;
    color: #ffffff;
    background: rgba(0, 0, 0, 0.4);
    padding: 1px 4px;
    border-radius: 3px;
    white-space: nowrap;
  }

  @keyframes queuePulse {
    0%, 100% {
      border-color: rgba(245, 158, 11, 0.4);
    }
    50% {
      border-color: rgba(245, 158, 11, 0.8);
    }
  }

  .aux-nav-group {
    display: flex;
    align-items: center;
    gap: 3px;
    background: transparent;
    flex-shrink: 0;
  }

  .aux-nav-item {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 8px;
    border-radius: var(--radius-md);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-muted);
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    transition: all var(--transition-fast);
  }

  .aux-nav-item:hover:not(:disabled) {
    color: var(--text-main);
    background: #15151c;
  }

  .aux-nav-item.active {
    color: var(--text-main);
    background: #191922;
    border-color: var(--border-subtle);
  }

  .nav-count-badge {
    padding: 1px 5px;
    background: rgba(56, 189, 248, 0.2);
    color: #38bdf8;
    border-radius: var(--radius-full);
    font-size: 10px;
    font-family: var(--font-mono);
    font-weight: 700;
  }

  /* Textual labels on Aux items are hidden by default to keep the bar compact & avoid clipping */
  .aux-label {
    display: none;
  }

  @media (min-width: 1320px) {
    .aux-label {
      display: inline;
    }
  }

  .core-nav-btn:disabled,
  .aux-nav-item:disabled,
  .brand-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  @media (max-width: 1080px) {
    .brand-ver {
      display: none;
    }
  }

  @media (max-width: 950px) {
    .core-btn-sub {
      display: none;
    }
    .header-inner {
      padding: 6px 10px;
    }
  }

  @media (max-width: 820px) {
    .brand-title {
      display: none;
    }
    .queue-stage {
      display: none;
    }
    .core-nav-btn {
      padding: 5px 9px;
      font-size: 12px;
      gap: 5px;
    }
  }

  @media (max-width: 620px) {
    .core-btn-label {
      display: none;
    }
    .core-nav-btn {
      padding: 6px 8px;
    }
  }
</style>
