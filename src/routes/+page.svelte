<script lang="ts">
  import { onMount } from 'svelte';
  import { currentView, toolkitConfig, recapperConfig } from '$lib/stores';
  import { loadSettings, saveSettings, detectFfmpeg } from '$lib/tauri';
  import Home from '../views/Home.svelte';
  import ToolkitConfig from '../views/ToolkitConfig.svelte';
  import RecapperConfig from '../views/RecapperConfig.svelte';
  import Processing from '../views/Processing.svelte';
  import Complete from '../views/Complete.svelte';
  import Settings from '../views/Settings.svelte';
  import Activity from '../views/Activity.svelte';
  import About from '../views/About.svelte';

  onMount(async () => {
    // Proactively detect FFmpeg and load saved configurations
    detectFfmpeg().catch(() => {});

    try {
      const saved = await loadSettings();
      if (saved.toolkit.inputPath) {
        toolkitConfig.set(saved.toolkit);
      }
      if (saved.recapper.inputFolder) {
        recapperConfig.set(saved.recapper);
      }
    } catch (e) {
      console.warn('Could not load saved settings:', e);
    }
  });

  // Auto-save settings on config changes
  $: {
    if ($toolkitConfig && $recapperConfig) {
      saveSettings({
        toolkit: $toolkitConfig,
        recapper: $recapperConfig,
        lastInputPath: $toolkitConfig.inputPath,
        lastOutputPath: $toolkitConfig.outputPath,
      }).catch(() => {});
    }
  }
</script>

{#if $currentView === 'home'}
  <Home />
{:else if $currentView === 'toolkit-config'}
  <ToolkitConfig />
{:else if $currentView === 'recapper-config'}
  <RecapperConfig />
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
