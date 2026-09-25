<script lang="ts">
  import type { ExplorerMemory } from '$lib/types';
  import { getMediaDataUrl, getSafeImageSrc, globalPerspective } from '$lib/memoriesStore';
  import Images from 'lucide-svelte/icons/images';

  export let memory: ExplorerMemory;

  $: mainPath = $globalPerspective === 'secondary' ? memory.secondaryPath : memory.primaryPath;
  $: insetPath = $globalPerspective === 'secondary' ? memory.primaryPath : memory.secondaryPath;
  const isVideo = (path?: string) => !!path && /\.(mp4|mov|m4v|webm)(\?|$)/i.test(path);

  async function useFileFallback(event: Event, path: string | undefined) {
    if (!path) return;
    const image = event.currentTarget as HTMLImageElement;
    if (image.dataset.fallbackAttempted) return;
    image.dataset.fallbackAttempted = 'true';
    try {
      const dataUrl = await getMediaDataUrl(path);
      if (image.isConnected) image.src = dataUrl;
    } catch { /* Keep the image placeholder when the archive file is unavailable. */ }
  }
</script>

<span class="map-memory-thumbnail" aria-hidden="true">
  {#if mainPath}
    {#key mainPath}
      {#if isVideo(mainPath)}<video class="main-camera" src={getSafeImageSrc(mainPath)} muted playsinline preload="metadata"><track kind="captions" /></video>
      {:else}<img class="main-camera" src={getSafeImageSrc(mainPath)} on:error={(event) => useFileFallback(event, mainPath)} alt="" loading="lazy" decoding="async" />{/if}
    {/key}
  {:else}
    <Images size={18} />
  {/if}
  {#if insetPath}
    <span class="inset-frame">
      {#key insetPath}
        {#if isVideo(insetPath)}<video class="inset-camera" src={getSafeImageSrc(insetPath)} muted playsinline preload="metadata"><track kind="captions" /></video>
        {:else}<img class="inset-camera" src={getSafeImageSrc(insetPath)} on:error={(event) => useFileFallback(event, insetPath)} alt="" loading="lazy" decoding="async" />{/if}
      {/key}
    </span>
  {/if}
</span>

<style>
  .map-memory-thumbnail { position: relative; display: grid; place-items: center; width: 100%; height: 100%; overflow: hidden; flex: none; border-radius: inherit; background: #294b63; color: #c9e1ee; }
  .main-camera { display: block; width: 100%; height: 100%; object-fit: cover; }
  .inset-frame { position: absolute; left: 3.78%; top: 3.78%; width: 42%; aspect-ratio: 3 / 4; overflow: hidden; border: var(--map-thumbnail-pip-border, 2.5px) solid #000; border-radius: var(--map-thumbnail-pip-radius, 9px); background: #000; box-shadow: 0 3px 10px #000d; }
  .inset-camera { display: block; width: 100%; height: 100%; object-fit: cover; }
</style>
