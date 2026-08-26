<script lang="ts">
  import { getSafeImageSrc, getMediaDataUrl, globalPerspective, globalAudioSettings } from '$lib/memoriesStore';
  import Play from 'lucide-svelte/icons/play';
  import Pause from 'lucide-svelte/icons/pause';
  import Repeat from 'lucide-svelte/icons/repeat';
  import Film from 'lucide-svelte/icons/film';
  import Move from 'lucide-svelte/icons/move';
  import Camera from 'lucide-svelte/icons/camera';
  import User from 'lucide-svelte/icons/circle-user';
  import VolumeIcon from '../common/VolumeIcon.svelte';

  export let primarySrc: string | undefined = undefined;
  export let secondarySrc: string | undefined = undefined;
  export let btsSrc: string | undefined = undefined;
  export let isVideo: boolean = false;
  export let alt: string = 'BeReal Memory';
  export let interactive: boolean = true;
  export let dayNumberOverlay: string = '';
  export let badgeText: string = '';
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let allowPreviewSwap: boolean = true;
  export let forceSwapped: boolean | undefined = undefined;

  // Local user toggle override; if null, defaults to $globalPerspective
  let localSwappedOverride: boolean | null = null;
  $: swapped = forceSwapped !== undefined
    ? forceSwapped
    : (localSwappedOverride !== null ? localSwappedOverride : $globalPerspective === 'secondary');

  let isPlayingBts = false;
  let isBtsMuted = true;
  let isVideoMuted = true;
  let btsVideoEl: HTMLVideoElement | null = null;

  // Sync initial mute state from global settings
  $: isBtsMuted = $globalAudioSettings.defaultMuted;
  $: isVideoMuted = $globalAudioSettings.defaultMuted;

  // Track raw loaded data URLs for primary & secondary
  let primaryDataUrl = '';
  let secondaryDataUrl = '';
  let primaryError = false;
  let secondaryError = false;

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
    if ((!interactive && !allowPreviewSwap) || isDragging || wasDragged) return;
    e?.stopPropagation();
    localSwappedOverride = !swapped;
  }

  function toggleVideoMute(e: MouseEvent) {
    e.stopPropagation();
    isVideoMuted = !isVideoMuted;
    if (baseVideoEl) baseVideoEl.muted = isVideoMuted;
    if (pipVideoEl) pipVideoEl.muted = isVideoMuted;
  }

  function toggleBtsMute(e: MouseEvent) {
    e.stopPropagation();
    isBtsMuted = !isBtsMuted;
    if (btsVideoEl) btsVideoEl.muted = isBtsMuted;
  }

  async function toggleBts(e: MouseEvent) {
    if (!btsSrc) return;
    e.stopPropagation();
    isPlayingBts = !isPlayingBts;
    if (isPlayingBts) {
      await tick();
      if (btsVideoEl) {
        try {
          btsVideoEl.currentTime = 0;
          btsVideoEl.muted = isBtsMuted;
          btsVideoEl.volume = $globalAudioSettings.volume;
          await btsVideoEl.play();
        } catch (err) {
          console.warn('BTS video playback error:', err);
        }
      }
    } else if (btsVideoEl) {
      btsVideoEl.pause();
    }
  }

  function handleVideoEnded() {
    isPlayingBts = false;
  }

  async function handlePrimaryImgError() {
    if (primarySrc && !primaryDataUrl) {
      const dataUrl = await getMediaDataUrl(primarySrc);
      if (dataUrl) {
        primaryDataUrl = dataUrl;
        primaryError = false;
        return;
      }
    }
    primaryError = true;
  }

  async function handleSecondaryImgError() {
    if (secondarySrc && !secondaryDataUrl) {
      const dataUrl = await getMediaDataUrl(secondarySrc);
      if (dataUrl) {
        secondaryDataUrl = dataUrl;
        secondaryError = false;
        return;
      }
    }
    secondaryError = true;
  }

  // Drag & Move PIP handler with 4-Corner Snap & GPU-accelerated transforms
  let cachedContainerW = 0;
  let cachedContainerH = 0;
  let cachedPipW = 0;
  let cachedPipH = 0;
  let rafId: number | null = null;

  function startPipDrag(e: MouseEvent | TouchEvent) {
    if (!interactive || !containerEl) return;
    e.stopPropagation();
    isDragging = true;
    wasDragged = false;

    const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
    const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
    dragInitialClientX = clientX;
    dragInitialClientY = clientY;

    // Cache container and PIP element dimensions once at start of drag
    const cRect = containerEl.getBoundingClientRect();
    cachedContainerW = cRect.width;
    cachedContainerH = cRect.height;
    cachedPipW = cRect.width * 0.3047;
    cachedPipH = cachedPipW * (4 / 3);

    const pipEl = containerEl.querySelector('.pip-frame-wrapper') as HTMLElement;
    if (pipEl) {
      const pRect = pipEl.getBoundingClientRect();
      pipPosX = pRect.left - cRect.left;
      pipPosY = pRect.top - cRect.top;
      hasMovedCustom = true;
    }

    dragStartX = clientX - pipPosX;
    dragStartY = clientY - pipPosY;

    window.addEventListener('mousemove', onPipDragMove, { passive: true });
    window.addEventListener('mouseup', onPipDragEnd);
    window.addEventListener('touchmove', onPipDragMove, { passive: true });
    window.addEventListener('touchend', onPipDragEnd);
  }

  function onPipDragMove(e: MouseEvent | TouchEvent) {
    if (!isDragging) return;
    const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
    const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;

    if (Math.hypot(clientX - dragInitialClientX, clientY - dragInitialClientY) > 4) {
      wasDragged = true;
    }

    let newX = clientX - dragStartX;
    let newY = clientY - dragStartY;

    // Constrain inside container bounds with 6px margin using cached dimensions
    newX = Math.max(6, Math.min(newX, cachedContainerW - cachedPipW - 6));
    newY = Math.max(6, Math.min(newY, cachedContainerH - cachedPipH - 6));

    if (rafId) cancelAnimationFrame(rafId);
    rafId = requestAnimationFrame(() => {
      pipPosX = newX;
      pipPosY = newY;
    });
  }

  function onPipDragEnd() {
    if (!isDragging) return;
    isDragging = false;
    if (rafId) cancelAnimationFrame(rafId);

    window.removeEventListener('mousemove', onPipDragMove);
    window.removeEventListener('mouseup', onPipDragEnd);
    window.removeEventListener('touchmove', onPipDragMove);
    window.removeEventListener('touchend', onPipDragEnd);

    // If dragged, calculate nearest of the 4 corners and snap to it
    if (wasDragged && cachedContainerW > 0) {
      const centerX = pipPosX + cachedPipW / 2;
      const centerY = pipPosY + cachedPipH / 2;

      const isLeft = centerX < cachedContainerW / 2;
      const isTop = centerY < cachedContainerH / 2;

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

  function isMediaVideo(src?: string): boolean {
    if (!src) return false;
    const clean = src.split('?')[0].toLowerCase();
    return clean.endsWith('.mp4') || clean.endsWith('.mov') || clean.endsWith('.webm') || clean.includes('video/') || clean.startsWith('data:video/');
  }

  $: resolvedPrimary = primaryDataUrl || getSafeImageSrc(primarySrc);
  $: resolvedSecondary = secondaryDataUrl || getSafeImageSrc(secondarySrc);

  let baseVideoEl: HTMLVideoElement | null = null;
  let pipVideoEl: HTMLVideoElement | null = null;
  let isHovered = false;

  function handleMouseEnter() {
    isHovered = true;
    playVideoPreview();
  }

  function handleMouseLeave() {
    isHovered = false;
    pauseVideoPreview();
  }

  function playVideoPreview() {
    const vol = $globalAudioSettings.volume;
    if (baseVideoEl) {
      baseVideoEl.volume = vol;
      baseVideoEl.muted = isVideoMuted;
      baseVideoEl.play().catch(() => {});
    }
    if (pipVideoEl) {
      pipVideoEl.volume = vol;
      pipVideoEl.muted = isVideoMuted;
      pipVideoEl.play().catch(() => {});
    }
  }

  function pauseVideoPreview() {
    if (baseVideoEl) {
      baseVideoEl.pause();
      baseVideoEl.currentTime = 0;
    }
    if (pipVideoEl) {
      pipVideoEl.pause();
      pipVideoEl.currentTime = 0;
    }
  }

  $: largeImage = swapped ? resolvedSecondary : resolvedPrimary;
  $: smallPipImage = swapped ? resolvedPrimary : resolvedSecondary;
  $: safeBtsSrc = getSafeImageSrc(btsSrc);

  $: isBaseVideo = (isVideo || isMediaVideo(largeImage));
  $: isPipVideo = isMediaVideo(smallPipImage);
</script>

<div
  bind:this={containerEl}
  class="bereal-frame-container size-{size}"
  class:interactive
  class:is-hovered={isHovered}
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleMouseLeave}
  role="region"
  aria-label={alt}
>
  <!-- Large Base Canvas (Primary or Secondary when swapped) -->
  <div class="large-canvas-wrap" class:canvas-flipped={swapped}>
    {#if isPlayingBts && safeBtsSrc}
      <video
        bind:this={btsVideoEl}
        src={safeBtsSrc}
        class="media-layer base-video"
        autoplay
        playsinline
        muted={isBtsMuted}
        on:ended={handleVideoEnded}
      >
        <track kind="captions" />
      </video>
    {:else if isBaseVideo && largeImage}
      <video
        bind:this={baseVideoEl}
        src={largeImage}
        class="media-layer base-video"
        loop
        muted={isVideoMuted}
        playsinline
        preload="metadata"
      >
        <track kind="captions" />
      </video>
    {:else if largeImage}
      <img
        src={largeImage}
        {alt}
        class="media-layer base-image"
        loading="lazy"
        decoding="async"
        on:error={swapped ? handleSecondaryImgError : handlePrimaryImgError}
      />
    {:else}
      <div class="media-placeholder">
        <span class="placeholder-text">Photo Unavailable</span>
      </div>
    {/if}

    <!-- Video indicator badge when idle (not hovered & not playing BTS) -->
    {#if (isBaseVideo || isPipVideo) && !isHovered && !isPlayingBts}
      <div class="video-indicator-badge" title="Hover to play preview">
        <Play size={10} class="fill-current text-white" />
      </div>
    {/if}

    <!-- Video Audio Mute/Unmute Icon Button when hovered -->
    {#if (isBaseVideo || isPipVideo) && isHovered && !isPlayingBts}
      <div class="video-audio-cluster">
        <button
          type="button"
          class="video-audio-pill"
          class:is-muted={isVideoMuted}
          on:click={toggleVideoMute}
          title={isVideoMuted ? 'Unmute Video Audio' : 'Mute Video Audio'}
          aria-label="Toggle video audio"
        >
          <VolumeIcon muted={isVideoMuted} size={12} />
        </button>
      </div>
    {/if}

    <!-- Day Number Overlay (Memories Grid & Calendar cards) -->
    {#if dayNumberOverlay}
      <div class="day-number-overlay">
        <span>{dayNumberOverlay}</span>
      </div>
    {/if}

    <!-- Top-right badge (e.g. BTS indicator) -->
    {#if badgeText}
      <div class="corner-badge">
        <span>{badgeText}</span>
      </div>
    {/if}

    <!-- Small Inset PIP (Selfie / Secondary camera) - Hidden during BTS playback -->
    {#if smallPipImage && !isPlayingBts}
      <div
        class="pip-frame-wrapper corner-{pipCorner}"
        class:is-custom-pos={hasMovedCustom}
        class:is-dragging={isDragging}
        style={hasMovedCustom ? `transform: translate3d(${pipPosX}px, ${pipPosY}px, 0); top: 0; left: 0; right: auto; bottom: auto; will-change: transform;` : ''}
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
          {#if (swapped ? primaryError : secondaryError)}
            <div class="pip-glass-placeholder" title="Camera view unavailable">
              <Camera size={16} class="text-white/50 animate-pulse" />
            </div>
          {:else if isPipVideo}
            <video
              bind:this={pipVideoEl}
              src={smallPipImage}
              class="pip-image"
              loop
              muted
              playsinline
              preload="metadata"
            >
              <track kind="captions" />
            </video>
          {:else}
            <img
              src={smallPipImage}
              alt=""
              class="pip-image"
              loading="lazy"
              decoding="async"
              on:error={swapped ? handlePrimaryImgError : handleSecondaryImgError}
            />
          {/if}

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

    <!-- BTS Micro-Video Play Trigger & Audio Toggle -->
    {#if btsSrc}
      <div class="bts-controls-cluster">
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

        {#if isPlayingBts}
          <button
            type="button"
            class="bts-audio-pill"
            class:is-muted={isBtsMuted}
            on:click|stopPropagation={() => (isBtsMuted = !isBtsMuted)}
            title={isBtsMuted ? 'Unmute BTS Audio' : 'Mute BTS Audio'}
            aria-label="Toggle BTS audio"
          >
            <VolumeIcon muted={isBtsMuted} size={12} />
          </button>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .bereal-frame-container {
    position: relative;
    width: 100%;
    aspect-ratio: 3 / 4;
    border-radius: 16px;
    overflow: hidden;
    background: #0d0d12;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
    user-select: none;
    touch-action: none;
  }

  .bereal-frame-container.size-sm {
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
  }

  .bereal-frame-container.size-md {
    border-radius: 16px;
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

  /* Video indicator badge when idle */
  .video-indicator-badge {
    position: absolute;
    bottom: 8px;
    right: 8px;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.72);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.25);
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.6);
    pointer-events: none;
    z-index: 15;
    transition: opacity 0.15s ease, transform 0.15s ease;
  }

  /* Movable Inset PIP Positioning matching 100% exact BeReal measurements */
  .pip-frame-wrapper {
    position: absolute;
    width: 30.47%;
    aspect-ratio: 3 / 4;
    z-index: 25;
    transition: top 0.22s cubic-bezier(0.16, 1, 0.3, 1), left 0.22s cubic-bezier(0.16, 1, 0.3, 1), right 0.22s cubic-bezier(0.16, 1, 0.3, 1), bottom 0.22s cubic-bezier(0.16, 1, 0.3, 1), transform 0.22s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .pip-frame-wrapper.is-dragging,
  .pip-frame-wrapper.is-custom-pos {
    transition: none !important;
  }

  .pip-frame-wrapper.corner-top-left {
    top: 3.78%;
    left: 3.78%;
  }

  .pip-frame-wrapper.corner-top-right {
    top: 3.78%;
    right: 3.78%;
    left: auto;
  }

  .pip-frame-wrapper.corner-bottom-left {
    bottom: 3.78%;
    left: 3.78%;
    top: auto;
  }

  .pip-frame-wrapper.corner-bottom-right {
    bottom: 3.78%;
    right: 3.78%;
    top: auto;
    left: auto;
  }

  .pip-frame {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: #000000;
    padding: 0;
    box-sizing: border-box;
    transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.2s ease;
  }

  /* Size-scaled borders and circular corner radii matching 16.24% continuous curvature */
  .size-sm .pip-frame {
    border-radius: 5px;
    border: 1.5px solid #000000;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.85);
  }

  .size-md .pip-frame {
    border-radius: 9px;
    border: 2.5px solid #000000;
    box-shadow: 0 3px 10px rgba(0, 0, 0, 0.85);
  }

  .size-lg .pip-frame {
    border-radius: 18px;
    border: 4px solid #000000;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.9);
  }

  .pip-glass-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: radial-gradient(circle at center, #1e1e2c 0%, #0d0d14 100%);
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

  /* Video Audio Controls Cluster */
  .video-audio-cluster {
    position: absolute;
    bottom: 10px;
    left: 10px;
    display: flex;
    align-items: center;
    gap: 6px;
    z-index: 30;
    animation: fadeIn 0.15s ease-out;
  }

  .video-audio-pill {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: #ffffff;
    cursor: pointer;
    padding: 0;
    transition: all 0.2s ease;
  }

  .video-audio-pill:hover {
    background: rgba(255, 255, 255, 0.2);
    transform: scale(1.1);
  }

  .video-audio-pill.is-muted {
    color: #f87171;
    border-color: rgba(248, 113, 113, 0.4);
  }

  /* BTS Play Trigger & Audio Controls Cluster */
  .bts-controls-cluster {
    position: absolute;
    bottom: 10px;
    right: 10px;
    display: flex;
    align-items: center;
    gap: 6px;
    z-index: 30;
  }

  .bts-trigger-pill {
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
    transition: all 0.2s ease;
  }

  .bts-trigger-pill:hover {
    background: rgba(56, 189, 248, 0.9);
    color: #000000;
    border-color: #38bdf8;
    transform: scale(1.05);
  }

  .bts-trigger-pill.active {
    background: #38bdf8;
    color: #000000;
    border-color: #38bdf8;
  }

  .bts-audio-pill {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: #ffffff;
    cursor: pointer;
    padding: 0;
    transition: all 0.2s ease;
  }

  .bts-audio-pill:hover {
    background: rgba(255, 255, 255, 0.2);
    transform: scale(1.1);
  }

  .bts-audio-pill.is-muted {
    color: #f87171;
    border-color: rgba(248, 113, 113, 0.4);
  }
</style>
