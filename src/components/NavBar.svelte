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

  let showDevMenu = false;

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
        >
          <Images size={15} />
          <span>Photos Suite</span>
        </button>

        <button
          type="button"
          class="core-nav-btn memories-btn"
          class:active={$currentView === 'memories'}
          on:click={() => navigate('memories')}
        >
          <Calendar size={15} />
          <span>Memories</span>
        </button>

        <button
          type="button"
          class="core-nav-btn recap-btn"
          class:active={$currentView === 'recapper-config'}
          on:click={() => navigate('recapper-config')}
        >
          <Film size={15} />
          <span>Recap Video</span>
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
  }

  .header-inner {
    max-width: 1200px;
    margin: 0 auto;
    padding: 8px 24px;
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 16px;
  }

  /* Zone 1: Left */
  .nav-zone-left {
    display: flex;
    align-items: center;
    justify-content: flex-start;
  }

  .brand-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 4px 6px;
    border-radius: var(--radius-md);
    color: inherit;
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
  }

  .brand-title {
    font-size: 14px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-main);
  }

  .brand-ver {
    font-size: 9.5px;
    font-family: var(--font-mono);
    color: var(--text-muted);
  }

  /* Zone 2: Center - Prominent Core Workflows */
  .nav-zone-center {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .primary-nav-group {
    display: flex;
    align-items: center;
    gap: 6px;
    background: #111116;
    padding: 3px 4px;
    border-radius: var(--radius-full);
    border: 1px solid rgba(255, 255, 255, 0.07);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
  }

  .core-nav-btn {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 6px 16px;
    border-radius: var(--radius-full);
    font-size: 13px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text-secondary);
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer;
    transition: all var(--transition-fast);
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
    gap: 10px;
  }

  .bg-queue-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    background: rgba(245, 158, 11, 0.12);
    border: 1px solid rgba(245, 158, 11, 0.4);
    border-radius: var(--radius-full);
    color: #fbbf24;
    cursor: pointer;
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
  }

  .queue-stage {
    font-weight: 500;
    color: #fef08a;
  }

  .queue-pct {
    font-weight: 700;
    color: #ffffff;
    background: rgba(0, 0, 0, 0.4);
    padding: 1px 4px;
    border-radius: 3px;
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
    gap: 2px;
    background: transparent;
  }

  .aux-nav-item {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    border-radius: var(--radius-md);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-muted);
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer;
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

  .core-nav-btn:disabled,
  .aux-nav-item:disabled,
  .brand-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  @media (max-width: 768px) {
    .header-inner {
      grid-template-columns: auto 1fr auto;
      padding: 8px 14px;
    }

    .aux-label {
      display: none;
    }

    .brand-title {
      display: none;
    }
  }
</style>
