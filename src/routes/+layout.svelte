<script lang="ts">
  import '../styles/global.css';
  import { currentView } from '$lib/stores';
  import TitleBar from '$components/TitleBar.svelte';
  import NavBar from '$components/NavBar.svelte';
  import ErrorModal from '$components/ErrorModal.svelte';
  import MemoryContextMenu from '$components/memories/MemoryContextMenu.svelte';
  import ExportMemoryModal from '$components/memories/ExportMemoryModal.svelte';
  import MemoryFeedModal from '$components/memories/MemoryFeedModal.svelte';
</script>

<svelte:window on:contextmenu={(e) => e.preventDefault()} />

<div class="app-container">
  <TitleBar />
  <NavBar />
  <main class="main-content" class:is-memories-view={$currentView === 'memories'}>
    <slot />
  </main>
</div>

<ErrorModal />
<MemoryContextMenu />
<ExportMemoryModal />
<MemoryFeedModal />

<style>
  :global(body) {
    background-color: #0a0a0c;
  }
</style>
