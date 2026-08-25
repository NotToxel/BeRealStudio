<script lang="ts">
  import '../styles/global.css';
  import TitleBar from '$components/TitleBar.svelte';
  import NavBar from '$components/NavBar.svelte';
  import ErrorModal from '$components/ErrorModal.svelte';
</script>

<div class="app-container">
  <TitleBar />
  <NavBar />
  <main class="main-content">
    <slot />
  </main>
</div>

<ErrorModal />

<style>
  :global(body) {
    background-color: #0a0a0c;
  }
</style>
