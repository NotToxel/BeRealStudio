<script lang="ts">
  import { getSafeImageSrc, getMediaDataUrl } from '$lib/memoriesStore';
  import Play from 'lucide-svelte/icons/play';
  import Pause from 'lucide-svelte/icons/pause';
  import Repeat from 'lucide-svelte/icons/repeat';
  import Film from 'lucide-svelte/icons/film';
  import Move from 'lucide-svelte/icons/move';

  export let primarySrc: string | undefined = undefined;
  export let secondarySrc: string | undefined = undefined;
  export let btsSrc: string | undefined = undefined;
  export let isVideo: boolean = false;
  export let alt: string = 'BeReal Memory';
  export let interactive: boolean = true;
  export let dayNumberOverlay: string = '';
  export let badgeText: string = '';
  export let size: 'sm' | 'md' | 'lg' = 'md';

  // Camera perspective state: if swapped is true, secondary is large base, primary is small PIP
  let swapped = false;
  let isPlayingBts = false;
  let btsVideoEl: HTMLVideoElement | null = null;

  // Track raw loaded data URLs for primary & secondary
  let primaryDataUrl = '';
  let secondaryDataUrl = '';

  // PIP position: 4 corner presets or free percentage (x, y in %)
  type PipCorner = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
  let pipCorner: PipCorner = 'top-left';
  let isDragging = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let pipPosX = 10; // px
  let pipPosY = 10; // px
  let containerEl: HTMLElement | null = null;
  let hasMovedCustom = false;

  let dragInitialClientX = 0;
  let dragInitialClientY = 0;
  let wasDragged = false;

  function toggleSwap(e?: Event) {
    if (!interactive || isDragging || wasDragged) return;
    e?.stopPropagation();
    swapped = !swapped;
  }

  function toggleBts(e: MouseEvent) {
    if (!btsSrc) return;
    e.stopPropagation();
    if (btsVideoEl) {
      if (isPlayingBts) {
        btsVideoEl.pause();
        isPlayingBts = false;
      } else {
        btsVideoEl.play();
        isPlayingBts = true;
      }
    }
  }

  function handleVideoEnded() {
    isPlayingBts = false;
  }

  async function handlePrimaryImgError() {
    if (primarySrc && !primaryDataUrl) {
      const dataUrl = await getMediaDataUrl(primarySrc);
      if (dataUrl) primaryDataUrl = dataUrl;
    }
  }

  async function handleSecondaryImgError() {
    if (secondarySrc && !secondaryDataUrl) {
      const dataUrl = await getMediaDataUrl(secondarySrc);
      if (dataUrl) secondaryDataUrl = dataUrl;
    }
  }

  // Drag & Move PIP handler with 4-Corner Snap
  function startPipDrag(e: MouseEvent | TouchEvent) {
    if (!interactive) return;
    e.stopPropagation();
    isDragging = true;
    wasDragged = false;

    const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
    const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
    dragInitialClientX = clientX;
    dragInitialClientY = clientY;

    // If currently anchored to a corner, initialize pipPosX/Y from element rect
    if (!hasMovedCustom && containerEl) {
      const pipEl = containerEl.querySelector('.pip-frame-wrapper') as HTMLElement;
      if (pipEl) {
        const cRect = containerEl.getBoundingClientRect();
        const pRect = pipEl.getBoundingClientRect();
        pipPosX = pRect.left - cRect.left;
        pipPosY = pRect.top - cRect.top;
      }
    }

    dragStartX = clientX - pipPosX;
    dragStartY = clientY - pipPosY;

    window.addEventListener('mousemove', onPipDragMove);
    window.addEventListener('mouseup', onPipDragEnd);
    window.addEventListener('touchmove', onPipDragMove, { passive: false });
    window.addEventListener('touchend', onPipDragEnd);
  }

  function onPipDragMove(e: MouseEvent | TouchEvent) {
    if (!isDragging || !containerEl) return;
    const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
    const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;

    if (Math.hypot(clientX - dragInitialClientX, clientY - dragInitialClientY) > 5) {
      wasDragged = true;
    }

    const rect = containerEl.getBoundingClientRect();
    const pipW = rect.width * 0.29;
    const pipH = pipW * (4 / 3);

    let newX = clientX - dragStartX;
    let newY = clientY - dragStartY;

    // Constrain inside container bounds with 8px margin
    newX = Math.max(8, Math.min(newX, rect.width - pipW - 8));
    newY = Math.max(8, Math.min(newY, rect.height - pipH - 8));

    pipPosX = newX;
    pipPosY = newY;
    hasMovedCustom = true;
  }

  function onPipDragEnd() {
    if (!isDragging) return;
    isDragging = false;
    window.removeEventListener('mousemove', onPipDragMove);
    window.removeEventListener('mouseup', onPipDragEnd);
    window.removeEventListener('touchmove', onPipDragMove);
    window.removeEventListener('touchend', onPipDragEnd);

    // If dragged, calculate nearest of the 4 corners and snap to it
    if (wasDragged && containerEl) {
      const rect = containerEl.getBoundingClientRect();
      const pipW = rect.width * 0.29;
      const pipH = pipW * (4 / 3);
      const centerX = pipPosX + pipW / 2;
      const centerY = pipPosY + pipH / 2;

      const isLeft = centerX < rect.width / 2;
      const isTop = centerY < rect.height / 2;

      if (isTop && isLeft) {
        pipCorner = 'top-left';
      } else if (isTop && !isLeft) {
        pipCorner = 'top-right';
      } else if (!isTop && isLeft) {
        pipCorner = 'bottom-left';
      } else {
        pipCorner = 'bottom-right';
      }

      // Smoothly transition from custom drag position to snapped corner
      hasMovedCustom = false;

      // Reset wasDragged after event loop cycle to prevent firing click handler
      setTimeout(() => {
        wasDragged = false;
      }, 50);
    }
  }

  function cycleCorner(e: MouseEvent) {
    if (!interactive) return;
    e.stopPropagation();
    const corners: PipCorner[] = ['top-left', 'top-right', 'bottom-right', 'bottom-left'];
    const nextIdx = (corners.indexOf(pipCorner) + 1) % corners.length;
    pipCorner = corners[nextIdx];
    hasMovedCustom = false;
  }

  $: resolvedPrimary = primaryDataUrl || getSafeImageSrc(primarySrc);
  $: resolvedSecondary = secondaryDataUrl || getSafeImageSrc(secondarySrc);

  $: largeImage = swapped ? resolvedSecondary : resolvedPrimary;
  $: smallPipImage = swapped ? resolvedPrimary : resolvedSecondary;
  $: safeBtsSrc = getSafeImageSrc(btsSrc);
</script>

<div
  bind:this={containerEl}
  class="bereal-frame-container size-{size}"
  class:interactive
>
  <!-- Large Base Canvas (Primary or Secondary when swapped) -->
  <div class="large-canvas-wrap" class:canvas-flipped={swapped}>
    {#if isPlayingBts && safeBtsSrc}
      <video
        bind:this={btsVideoEl}
        src={safeBtsSrc}
        class="media-layer base-video"
        autoplay
        loop
        playsinline
        on:ended={handleVideoEnded}
      >
        <track kind="captions" />
      </video>
    {:else if isVideo && largeImage}
      <video
        src={largeImage}
        class="media-layer base-video"
        autoplay
        loop
        muted
        playsinline
      >
        <track kind="captions" />
      </video>
    {:else if largeImage}
      <img
        src={largeImage}
        {alt}
        class="media-layer base-image"
        loading="lazy"
        on:error={swapped ? handleSecondaryImgError : handlePrimaryImgError}
      />
    {:else}
      <div class="media-placeholder">
        <span class="placeholder-text">Photo Unavailable</span>
      </div>
    {/if}

    <!-- Day Number Overlay (Memories Grid & Calendar cards) -->
    {#if dayNumberOverlay}
      <div class="day-number-overlay">
        <span>{dayNumberOverlay}</span>
      </div>
    {/if}

    <!-- Top-right badge (e.g. retakes (2) or BTS) -->
    {#if badgeText}
      <div class="corner-badge">
        <span>{badgeText}</span>
      </div>
    {/if}

    <!-- Small Inset PIP (Selfie / Secondary camera) -->
    {#if smallPipImage}
      <div
        class="pip-frame-wrapper corner-{pipCorner}"
        class:is-custom-pos={hasMovedCustom}
        style={hasMovedCustom ? `left: ${pipPosX}px; top: ${pipPosY}px;` : ''}
      >
        <div
          class="pip-frame"
          class:pip-dragging={isDragging}
          role="button"
          tabindex="0"
          on:click={toggleSwap}
          on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSwap(e)}
          on:mousedown={startPipDrag}
          on:touchstart={startPipDrag}
          title="Click to swap cameras • Drag to move anywhere"
          aria-label="Selfie camera inset — Click to swap, drag to move"
        >
          <img
            src={smallPipImage}
            alt="Selfie inset"
            class="pip-image"
            loading="lazy"
            on:error={swapped ? handlePrimaryImgError : handleSecondaryImgError}
          />

          {#if interactive}
            <div class="pip-overlay-tools">
              <div class="pip-tool-icon swap-icon" title="Swap camera perspectives">
                <Repeat size={10} />
              </div>
              <button
                type="button"
                class="pip-tool-icon move-icon"
                on:click|stopPropagation={cycleCorner}
                title="Move to next corner"
                aria-label="Move to next corner"
              >
                <Move size={10} />
              </button>
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <!-- BTS Micro-Video Play Trigger -->
    {#if btsSrc}
      <button
        type="button"
        class="bts-trigger-pill"
        class:active={isPlayingBts}
        on:click={toggleBts}
        title="Play BTS micro-video"
        aria-label="Play BTS micro-video"
      >
        {#if isPlayingBts}
          <Pause size={12} />
          <span>BTS Playing</span>
        {:else}
          <Film size={12} />
          <span>BTS</span>
        {/if}
      </button>
    {/if}
  </div>
</div>

<style>
  .bereal-frame-container {
    position: relative;
    width: 100%;
    aspect-ratio: 3 / 4;
    border-radius: 20px;
    overflow: hidden;
    background: #0d0d12;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
    user-select: none;
    touch-action: none;
  }

  .bereal-frame-container.size-sm {
    border-radius: 12px;
  }

  .bereal-frame-container.size-lg {
    border-radius: 24px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.8);
  }

  .large-canvas-wrap {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    transition: transform 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  .media-layer {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .media-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #15151f;
    color: var(--text-muted);
    font-size: 11.5px;
  }

  /* Day Number Overlay (e.g. 19, 20, Today) */
  .day-number-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
    z-index: 10;
  }

  .day-number-overlay span {
    font-size: 26px;
    font-weight: 800;
    color: #ffffff;
    text-shadow: 0 2px 8px rgba(0, 0, 0, 0.8), 0 0 20px rgba(0, 0, 0, 0.9);
    letter-spacing: -0.02em;
  }

  .size-sm .day-number-overlay span {
    font-size: 16px;
  }

  .size-lg .day-number-overlay span {
    font-size: 38px;
  }

  /* Corner badge (e.g. retakes count or post count) */
  .corner-badge {
    position: absolute;
    top: 8px;
    right: 8px;
    padding: 2px 7px;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 12px;
    font-size: 10.5px;
    font-weight: 700;
    color: #ffffff;
    z-index: 15;
  }

  /* Movable Inset PIP Positioning */
  .pip-frame-wrapper {
    position: absolute;
    width: 29%;
    aspect-ratio: 3 / 4;
    z-index: 25;
    transition: top 0.22s cubic-bezier(0.16, 1, 0.3, 1), left 0.22s cubic-bezier(0.16, 1, 0.3, 1), right 0.22s cubic-bezier(0.16, 1, 0.3, 1), bottom 0.22s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .pip-frame-wrapper.is-custom-pos {
    transition: none;
  }

  .pip-frame-wrapper.corner-top-left {
    top: 12px;
    left: 12px;
  }

  .pip-frame-wrapper.corner-top-right {
    top: 12px;
    right: 12px;
    left: auto;
  }

  .pip-frame-wrapper.corner-bottom-left {
    bottom: 12px;
    left: 12px;
    top: auto;
  }

  .pip-frame-wrapper.corner-bottom-right {
    bottom: 12px;
    right: 12px;
    top: auto;
    left: auto;
  }

  .size-sm .pip-frame-wrapper.corner-top-left {
    top: 6px;
    left: 6px;
  }

  .size-lg .pip-frame-wrapper.corner-top-left {
    top: 16px;
    left: 16px;
  }

  .pip-frame {
    position: relative;
    width: 100%;
    height: 100%;
    border-radius: 12px;
    overflow: hidden;
    border: 2.5px solid #000000;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.85);
    cursor: grab;
    background: #000000;
    padding: 0;
    transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.2s ease;
  }

  .pip-frame:hover {
    transform: scale(1.05);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.95);
  }

  .pip-frame.pip-dragging {
    cursor: grabbing;
    transform: scale(1.08);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.95);
  }

  .pip-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    pointer-events: none;
  }

  .pip-overlay-tools {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.25);
    display: flex;
    justify-content: space-between;
    padding: 4px;
    opacity: 0;
    transition: opacity 0.15s ease;
    pointer-events: auto;
  }

  .pip-frame:hover .pip-overlay-tools {
    opacity: 1;
  }

  .pip-tool-icon {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(4px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #ffffff;
    cursor: pointer;
    padding: 0;
    transition: transform 0.15s ease, background 0.15s ease;
  }

  .pip-tool-icon:hover {
    background: #ffe600;
    color: #000000;
    transform: scale(1.15);
  }

  /* BTS Play Trigger Pill */
  .bts-trigger-pill {
    position: absolute;
    bottom: 10px;
    right: 10px;
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-full);
    font-size: 10.5px;
    font-weight: 700;
    color: #ffffff;
    cursor: pointer;
    z-index: 30;
    transition: all 0.2s ease;
  }

  .bts-trigger-pill:hover {
    background: rgba(255, 230, 0, 0.9);
    color: #000000;
    border-color: #ffe600;
    transform: scale(1.05);
  }

  .bts-trigger-pill.active {
    background: #ffe600;
    color: #000000;
    border-color: #ffe600;
  }
</style>
