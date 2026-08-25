<script lang="ts">
  import { currentView, isProcessing } from '$lib/stores';
  import type { ViewMode } from '$lib/types';
  import Home from 'lucide-svelte/icons/house';
  import Images from 'lucide-svelte/icons/images';
  import Film from 'lucide-svelte/icons/film';
  import Settings from 'lucide-svelte/icons/settings';
  import Info from 'lucide-svelte/icons/info';

  function navigate(mode: ViewMode) {
    if ($isProcessing) return;
    currentView.set(mode);
  }
</script>

<header class="app-header">
  <div class="header-inner">
    <!-- Brand / Logo -->
    <button
      type="button"
      class="brand-btn"
      on:click={() => navigate('home')}
      disabled={$isProcessing}
    >
      <div class="logo-mark">
        <span class="logo-dot"></span>
      </div>
      <div class="brand-text">
        <span class="brand-title">BeReal Studio</span>
        <span class="brand-ver">v0.1.0</span>
      </div>
    </button>

    <!-- Nav Links -->
    <nav class="nav-links">
      <button
        type="button"
        class="nav-item"
        class:active={$currentView === 'home'}
        on:click={() => navigate('home')}
        disabled={$isProcessing}
      >
        <Home size={14} />
        <span>Home</span>
      </button>

      <button
        type="button"
        class="nav-item"
        class:active={$currentView === 'toolkit-config'}
        on:click={() => navigate('toolkit-config')}
        disabled={$isProcessing}
      >
        <Images size={14} />
        <span>Photos</span>
      </button>

      <button
        type="button"
        class="nav-item"
        class:active={$currentView === 'recapper-config'}
        on:click={() => navigate('recapper-config')}
        disabled={$isProcessing}
      >
        <Film size={14} />
        <span>Recap Video</span>
      </button>

      <button
        type="button"
        class="nav-item"
        class:active={$currentView === 'settings'}
        on:click={() => navigate('settings')}
        disabled={$isProcessing}
      >
        <Settings size={14} />
        <span>Settings</span>
      </button>

      <button
        type="button"
        class="nav-item"
        class:active={$currentView === 'about'}
        on:click={() => navigate('about')}
        disabled={$isProcessing}
      >
        <Info size={14} />
        <span>About</span>
      </button>
    </nav>
  </div>
</header>

<style>
  .app-header {
    width: 100%;
    background: rgba(9, 9, 11, 0.85);
    backdrop-filter: blur(14px);
    border-bottom: 1px solid var(--border-subtle);
    position: sticky;
    top: 0;
    z-index: 100;
  }

  .header-inner {
    max-width: 1100px;
    margin: 0 auto;
    padding: 10px 36px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .brand-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 4px 0;
    color: inherit;
  }

  .logo-mark {
    width: 24px;
    height: 24px;
    background: linear-gradient(135deg, #18181d 0%, #101014 100%);
    border: 1px solid rgba(255, 230, 0, 0.3);
    border-radius: 7px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 0 10px rgba(255, 230, 0, 0.15);
  }

  .logo-dot {
    width: 9px;
    height: 9px;
    background: #ffe600;
    border-radius: 50%;
    box-shadow: 0 0 8px rgba(255, 230, 0, 0.8);
  }

  .brand-text {
    display: flex;
    align-items: baseline;
    gap: 6px;
  }

  .brand-title {
    font-size: 14.5px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-main);
  }

  .brand-ver {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-muted);
  }

  .nav-links {
    display: flex;
    align-items: center;
    gap: 4px;
    background: #111115;
    padding: 3px;
    border-radius: var(--radius-full);
    border: 1px solid var(--border-subtle);
  }

  .nav-item {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    border-radius: var(--radius-full);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .nav-item:hover:not(:disabled) {
    color: var(--text-main);
  }

  .nav-item.active {
    background: #ffffff;
    color: #09090b;
    font-weight: 600;
    box-shadow: 0 2px 8px rgba(255, 255, 255, 0.2);
  }

  .nav-item:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
</style>
