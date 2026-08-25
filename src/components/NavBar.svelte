<script lang="ts">
  import { currentView, isProcessing } from '$lib/stores';
  import type { ViewMode } from '$lib/types';
  import Home from 'lucide-svelte/icons/house';
  import Images from 'lucide-svelte/icons/images';
  import Film from 'lucide-svelte/icons/film';
  import History from 'lucide-svelte/icons/history';
  import Settings from 'lucide-svelte/icons/settings';
  import Info from 'lucide-svelte/icons/info';
  import { activityHistory } from '$lib/stores';

  function navigate(mode: ViewMode) {
    if ($isProcessing) return;
    currentView.set(mode);
  }
</script>

<header class="app-header">
  <div class="header-inner">
    <!-- Zone 1 (Left): Brand identity & Quick Home -->
    <div class="nav-zone-left">
      <button
        type="button"
        class="brand-btn"
        on:click={() => navigate('home')}
        disabled={$isProcessing}
        title="Return to Home Dashboard"
      >
        <div class="logo-mark">
          <span class="logo-dot"></span>
        </div>
        <div class="brand-text">
          <span class="brand-title">BeReal Studio</span>
          <span class="brand-ver">v1.1.0</span>
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
          disabled={$isProcessing}
        >
          <Images size={15} />
          <span>Photos</span>
        </button>

        <button
          type="button"
          class="core-nav-btn recap-btn"
          class:active={$currentView === 'recapper-config'}
          on:click={() => navigate('recapper-config')}
          disabled={$isProcessing}
        >
          <Film size={15} />
          <span>Recap Video</span>
        </button>
      </div>
    </div>

    <!-- Zone 3 (Right): Auxiliary Navigation -->
    <div class="nav-zone-right">
      <div class="aux-nav-group">
        <button
          type="button"
          class="aux-nav-item"
          class:active={$currentView === 'home'}
          on:click={() => navigate('home')}
          disabled={$isProcessing}
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
          disabled={$isProcessing}
          title="Activity & History"
        >
          <History size={15} />
          <span class="aux-label">Activity</span>
          {#if $activityHistory.length > 0}
            <span class="nav-count-badge">{$activityHistory.length}</span>
          {/if}
        </button>

        <button
          type="button"
          class="aux-nav-item"
          class:active={$currentView === 'settings'}
          on:click={() => navigate('settings')}
          disabled={$isProcessing}
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
          disabled={$isProcessing}
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
