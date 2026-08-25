<script lang="ts">
  import { onMount } from 'svelte';
  import { currentView, toolkitConfig, recapperConfig, activityHistory, registerNativeActivitySync } from '$lib/stores';
  import { loadSettings, saveSettings, detectFfmpeg, loadActivityHistory, saveActivityHistory, clearNativeActivityHistory } from '$lib/tauri';
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import Home from '../views/Home.svelte';
  import ToolkitConfig from '../views/ToolkitConfig.svelte';
  import RecapperConfig from '../views/RecapperConfig.svelte';
  import MemoriesView from '../views/MemoriesView.svelte';
  import Processing from '../views/Processing.svelte';
  import Complete from '../views/Complete.svelte';
  import Settings from '../views/Settings.svelte';
  import Activity from '../views/Activity.svelte';
  import About from '../views/About.svelte';

  let isHydrated = false;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  function persistSettingsNow() {
    if (!isHydrated || !$toolkitConfig || !$recapperConfig) return;
    const payload = {
      toolkit: $toolkitConfig,
      recapper: $recapperConfig,
      lastInputPath: $toolkitConfig.inputPath,
      lastOutputPath: $toolkitConfig.outputPath,
    };
    saveSettings(payload).catch(() => {});
    try {
      localStorage.setItem('bereal_studio_live_settings', JSON.stringify(payload));
    } catch {}
  }

  onMount(async () => {
    // Immediate hydration from fast browser storage
    try {
      const cached = localStorage.getItem('bereal_studio_live_settings');
      if (cached) {
        const parsed = JSON.parse(cached);
        if (parsed.toolkit) toolkitConfig.update((c) => ({ ...c, ...parsed.toolkit }));
        if (parsed.recapper) recapperConfig.update((c) => ({ ...c, ...parsed.recapper }));
      }
    } catch {}

    // Register native dual-layer disk persistence for activity history
    registerNativeActivitySync(saveActivityHistory, clearNativeActivityHistory);

    // Hydrate native settings from disk if available
    try {
      const savedSettings = await loadSettings();
      if (savedSettings) {
        if (savedSettings.toolkit) {
          toolkitConfig.update((current) => ({ ...current, ...savedSettings.toolkit }));
        }
        if (savedSettings.recapper) {
          recapperConfig.update((current) => ({ ...current, ...savedSettings.recapper }));
        }
      }
    } catch (e) {
      console.warn('Could not load native settings:', e);
    } finally {
      isHydrated = true;
    }

    // Hydrate native activity history from disk if available
    try {
      const diskHistory = await loadActivityHistory();
      if (diskHistory && diskHistory.length > 0) {
        activityHistory.update((local) => {
          const ids = new Set(local.map((r) => r.id));
          const merged = [...local];
          for (const item of diskHistory) {
            if (!ids.has(item.id)) {
              merged.push(item);
              ids.add(item.id);
            }
          }
          return merged.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()).slice(0, 50);
        });
      }
    } catch (e) {
      console.warn('Could not load native activity history:', e);
    }

    // URL-based view routing and demo loading for automated screenshot captures
    if (typeof window !== 'undefined') {
      const urlParams = new URLSearchParams(window.location.search);
      if (urlParams.get('demo') === '1') {
        const { loadAllDemoData } = await import('$lib/devMode');
        loadAllDemoData();
      }
      const requestedView = urlParams.get('view') as any;
      if (requestedView && ['home', 'toolkit-config', 'recapper-config', 'memories', 'activity', 'settings', 'about', 'processing', 'complete'].includes(requestedView)) {
        currentView.set(requestedView);
      }
    }
  });

  // Continuous auto-save whenever any option, slider, toggle, or field changes
  $: if (isHydrated && $toolkitConfig && $recapperConfig) {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      persistSettingsNow();
    }, 200);
  }
</script>

<svelte:window on:beforeunload={persistSettingsNow} />

{#key $currentView}
  <div class="view-transition-stage" in:fly={{ y: 6, duration: 220, easing: cubicOut }}>
    {#if $currentView === 'home'}
      <Home />
    {:else if $currentView === 'toolkit-config'}
      <ToolkitConfig />
    {:else if $currentView === 'recapper-config'}
      <RecapperConfig />
    {:else if $currentView === 'memories'}
      <MemoriesView />
    {:else if $currentView === 'processing'}
      <Processing />
    {:else if $currentView === 'complete'}
      <Complete />
    {:else if $currentView === 'activity'}
      <Activity />
    {:else if $currentView === 'settings'}
      <Settings />
    {:else if $currentView === 'about'}
      <About />
    {/if}
  </div>
{/key}

<style>
  .view-transition-stage {
    width: 100%;
    height: 100%;
  }
</style>
