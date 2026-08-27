<script lang="ts">
  import { tick, onMount } from 'svelte';
  import { getSafeImageSrc, getMediaDataUrl, globalPerspective, globalAudioSettings, showMemoryDebugBadges } from '$lib/memoriesStore';
  import Play from 'lucide-svelte/icons/play';
  import Pause from 'lucide-svelte/icons/pause';
  import Repeat from 'lucide-svelte/icons/repeat';
  import Film from 'lucide-svelte/icons/film';
  import Move from 'lucide-svelte/icons/move';
  import Camera from 'lucide-svelte/icons/camera';
  import User from 'lucide-svelte/icons/circle-user';
  import Plus from 'lucide-svelte/icons/plus';
  import Minus from 'lucide-svelte/icons/minus';
  import RotateCcw from 'lucide-svelte/icons/rotate-ccw';
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
  export let isLate: boolean | undefined = undefined;
  export let lateDuration: string | undefined = undefined;
  export let lateExact: string | undefined = undefined;
  export let takenAt: string = '';
  export let rawJson: string | undefined = undefined;
  export let debugInfo: string | undefined = undefined;
  export let showDebugBadge: boolean | undefined = undefined;

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
    if (!interactive || size !== 'lg' || !containerEl) return;
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
  let isPlaying = false;
  let currentTime = 0;
  let duration = 0;
  let isSeeking = false;
  let playbackTrackEl: HTMLElement | null = null;

  $: hasVideoContent = isBaseVideo || isPipVideo || isPlayingBts;

  function handleMouseEnter() {
    isHovered = true;
    if (size !== 'lg' && !isPlaying) {
      playVideo();
    }
  }

  function handleMouseLeave() {
    isHovered = false;
    if (size !== 'lg') {
      pauseVideo();
    }
  }

  function handleCanvasClick(e: MouseEvent) {
    if (hasVideoContent && size === 'lg' && zoomLevel <= 1.01 && !wasDragged) {
      togglePlayPause(e);
    }
  }

  function playVideo() {
    isPlaying = true;
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
    if (btsVideoEl && isPlayingBts) {
      btsVideoEl.volume = vol;
      btsVideoEl.muted = isBtsMuted;
      btsVideoEl.play().catch(() => {});
    }
  }

  function pauseVideo() {
    isPlaying = false;
    if (baseVideoEl) baseVideoEl.pause();
    if (pipVideoEl) pipVideoEl.pause();
    if (btsVideoEl && isPlayingBts) btsVideoEl.pause();
  }

  function togglePlayPause(e?: Event) {
    e?.stopPropagation();
    if (isPlayingBts) {
      if (btsVideoEl) {
        if (btsVideoEl.paused) {
          btsVideoEl.play().catch(() => {});
          isPlaying = true;
        } else {
          btsVideoEl.pause();
          isPlaying = false;
        }
      }
      return;
    }

    if (isPlaying) {
      pauseVideo();
    } else {
      playVideo();
    }
  }

  function handleTimeUpdate() {
    if (isSeeking) return;
    if (isPlayingBts && btsVideoEl) {
      currentTime = btsVideoEl.currentTime;
      duration = btsVideoEl.duration || duration;
    } else if (baseVideoEl) {
      currentTime = baseVideoEl.currentTime;
      duration = baseVideoEl.duration || pipVideoEl?.duration || duration;
    } else if (pipVideoEl) {
      currentTime = pipVideoEl.currentTime;
      duration = pipVideoEl.duration || duration;
    }
  }

  function handleLoadedMetadata(e: Event) {
    const el = e.currentTarget as HTMLVideoElement;
    if (el && el.duration) {
      duration = el.duration;
    }
  }

  function handleScrubberPointerDown(e: PointerEvent) {
    if (e.button !== 0 || !duration) return;
    e.stopPropagation();
    isSeeking = true;
    try {
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    } catch {}
    seekToPoint(e);
  }

  function handleScrubberPointerMove(e: PointerEvent) {
    if (!isSeeking || !duration) return;
    e.stopPropagation();
    seekToPoint(e);
  }

  function handleScrubberPointerUp(e: PointerEvent) {
    if (!isSeeking) return;
    e.stopPropagation();
    isSeeking = false;
    try {
      (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    } catch {}
    if (isPlaying) {
      playVideo();
    }
  }

  function seekToPoint(e: PointerEvent | MouseEvent) {
    if (!playbackTrackEl || !duration) return;
    const rect = playbackTrackEl.getBoundingClientRect();
    const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    const targetTime = ratio * duration;
    currentTime = targetTime;

    if (isPlayingBts && btsVideoEl) {
      btsVideoEl.currentTime = targetTime;
    } else {
      if (baseVideoEl) baseVideoEl.currentTime = targetTime;
      if (pipVideoEl) pipVideoEl.currentTime = targetTime;
    }
  }

  function formatVideoTime(sec: number): string {
    if (!sec || isNaN(sec)) return '0:00';
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${m}:${s < 10 ? '0' : ''}${s}`;
  }

  $: largeImage = swapped ? resolvedSecondary : resolvedPrimary;
  $: smallPipImage = swapped ? resolvedPrimary : resolvedSecondary;
  $: safeBtsSrc = getSafeImageSrc(btsSrc);

  $: isBaseVideo = (isVideo || isMediaVideo(largeImage));
  $: isPipVideo = isMediaVideo(smallPipImage);

  // Native Platform Zoom & Pan Engine
  let zoomLevel = 1.0;
  let panX = 0;
  let panY = 0;
  let isPanning = false;
  let panStartX = 0;
  let panStartY = 0;
  let panRafId: number | null = null;
  let canvasWrapEl: HTMLElement | null = null;

  // Reset zoom on media or perspective switch
  $: if (primarySrc || secondarySrc || forceSwapped !== undefined) {
    resetZoom();
  }

  function resetZoom(e?: Event) {
    e?.stopPropagation();
    zoomLevel = 1.0;
    panX = 0;
    panY = 0;
    isPanning = false;
  }

  function zoomIn(e?: Event) {
    e?.stopPropagation();
    zoomLevel = Math.min(4.0, Math.round((zoomLevel + 0.5) * 10) / 10);
    clampPan();
  }

  function zoomOut(e?: Event) {
    e?.stopPropagation();
    zoomLevel = Math.max(1.0, Math.round((zoomLevel - 0.5) * 10) / 10);
    if (zoomLevel <= 1.01) {
      panX = 0;
      panY = 0;
    } else {
      clampPan();
    }
  }

  function clampPan() {
    if (zoomLevel <= 1.01) {
      panX = 0;
      panY = 0;
      return;
    }
    const el = canvasWrapEl || containerEl;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const baseW = rect.width / zoomLevel;
    const baseH = rect.height / zoomLevel;
    const maxPanX = (baseW * (zoomLevel - 1)) / 2;
    const maxPanY = (baseH * (zoomLevel - 1)) / 2;
    panX = Math.max(-maxPanX, Math.min(maxPanX, panX));
    panY = Math.max(-maxPanY, Math.min(maxPanY, panY));
  }

  function handleWheelZoom(e: WheelEvent) {
    // Trackpad pinch gesture (e.ctrlKey) or Ctrl + Mouse Wheel
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      e.stopPropagation();

      const zoomFactor = -e.deltaY * 0.008;
      const nextZoom = Math.min(4.0, Math.max(1.0, Math.round((zoomLevel + zoomFactor) * 100) / 100));

      if (nextZoom <= 1.01) {
        resetZoom();
      } else {
        zoomLevel = nextZoom;
        clampPan();
      }
    } else if (zoomLevel > 1.01) {
      // 2-finger trackpad panning / mouse wheel scroll when zoomed
      e.preventDefault();
      e.stopPropagation();
      panX -= e.deltaX;
      panY -= e.deltaY;
      clampPan();
    }
  }

  function handleDoubleClickZoom(e: MouseEvent) {
    e.stopPropagation();

    if (zoomLevel > 1.05) {
      resetZoom();
    } else {
      const el = canvasWrapEl || containerEl;
      if (el) {
        const rect = el.getBoundingClientRect();
        const clickX = e.clientX - rect.left - rect.width / 2;
        const clickY = e.clientY - rect.top - rect.height / 2;
        zoomLevel = 2.5;
        panX = -clickX * 0.6;
        panY = -clickY * 0.6;
        clampPan();
      } else {
        zoomLevel = 2.5;
      }
    }
  }

  let isDraggingCanvas = false;
  let panDragStartX = 0;
  let panDragStartY = 0;
  let initialPanX = 0;
  let initialPanY = 0;

  function handlePanPointerDown(e: PointerEvent) {
    if (zoomLevel <= 1.01 || e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    isPanning = true;
    isDraggingCanvas = true;
    panDragStartX = e.clientX;
    panDragStartY = e.clientY;
    initialPanX = panX;
    initialPanY = panY;
    try {
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    } catch {}
  }

  function handlePanPointerMove(e: PointerEvent) {
    if (!isDraggingCanvas) return;
    e.preventDefault();
    e.stopPropagation();
    const deltaX = e.clientX - panDragStartX;
    const deltaY = e.clientY - panDragStartY;
    panX = initialPanX + deltaX;
    panY = initialPanY + deltaY;
    clampPan();
  }

  function handlePanPointerUp(e: PointerEvent) {
    if (!isDraggingCanvas) return;
    e.preventDefault();
    e.stopPropagation();
    isDraggingCanvas = false;
    isPanning = false;
    try {
      (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    } catch {}
    clampPan();
  }
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
  <div
    bind:this={canvasWrapEl}
    class="large-canvas-wrap"
    class:canvas-flipped={swapped}
    class:is-zoomed={zoomLevel > 1.01}
    class:is-panning={isPanning}
    role="presentation"
    style={zoomLevel > 1.01 ? `transform: translate3d(${panX}px, ${panY}px, 0) scale(${zoomLevel}); transform-origin: center center; will-change: transform; cursor: ${isPanning ? 'grabbing' : 'grab'};` : ''}
    on:wheel={handleWheelZoom}
    on:dblclick={handleDoubleClickZoom}
    on:pointerdown={handlePanPointerDown}
    on:pointermove={handlePanPointerMove}
    on:pointerup={handlePanPointerUp}
    on:pointercancel={handlePanPointerUp}
    on:lostpointercapture={handlePanPointerUp}
    on:click={handleCanvasClick}
  >
    {#if isPlayingBts && safeBtsSrc}
      <video
        bind:this={btsVideoEl}
        src={safeBtsSrc}
        class="media-layer base-video"
        autoplay
        playsinline
        draggable="false"
        muted={isBtsMuted}
        on:timeupdate={handleTimeUpdate}
        on:loadedmetadata={handleLoadedMetadata}
        on:play={() => (isPlaying = true)}
        on:pause={() => (isPlaying = false)}
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
        draggable="false"
        on:timeupdate={handleTimeUpdate}
        on:loadedmetadata={handleLoadedMetadata}
        on:play={() => (isPlaying = true)}
        on:pause={() => (isPlaying = false)}
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
        draggable="false"
        on:error={swapped ? handleSecondaryImgError : handlePrimaryImgError}
      />
    {:else}
      <div class="media-placeholder">
        <span class="placeholder-text">Photo Unavailable</span>
      </div>
    {/if}
  </div>

  <!-- Inset & Floating UI Overlays (Direct children of frame container, invariant to canvas zoom/pan) -->

    <!-- Video indicator badge when paused -->
    {#if (isBaseVideo || isPipVideo) && !isPlaying && !isPlayingBts}
      <button
        type="button"
        class="video-indicator-badge"
        on:click={togglePlayPause}
        title={size === 'lg' ? 'Click to play video' : 'Hover to play preview'}
        aria-label="Play video"
      >
        <Play size={size === 'lg' ? 12 : 10} class="fill-current text-white" />
      </button>
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

    <!-- Top-right badge (e.g. BTS indicator / interactive play trigger) -->
    {#if badgeText}
      {#if btsSrc}
        <div class="corner-badge-wrap">
          <button
            type="button"
            class="corner-badge bts-corner-btn"
            class:active={isPlayingBts}
            on:click|stopPropagation={toggleBts}
            title={isPlayingBts ? 'Pause BTS micro-video' : 'Play BTS micro-video'}
            aria-label="Play BTS micro-video"
          >
            {#if isPlayingBts}
              <Pause size={size === 'lg' ? 12 : 11} class="bts-icon" />
              <span>BTS Playing</span>
            {:else}
              <Film size={size === 'lg' ? 12 : 11} class="bts-icon" />
              <span>{badgeText}</span>
            {/if}
          </button>

          {#if isPlayingBts}
            <button
              type="button"
              class="bts-corner-audio-btn"
              class:is-muted={isBtsMuted}
              on:click|stopPropagation={() => (isBtsMuted = !isBtsMuted)}
              title={isBtsMuted ? 'Unmute BTS Audio' : 'Mute BTS Audio'}
              aria-label="Toggle BTS audio"
            >
              <VolumeIcon muted={isBtsMuted} size={size === 'lg' ? 12 : 10} />
            </button>
          {/if}
        </div>
      {:else}
        <div class="corner-badge">
          <span>{badgeText}</span>
        </div>
      {/if}
    {/if}

    <!-- Formal Memory Timing & Metadata Inspector Badge (Hidden behind $showMemoryDebugBadges flag) -->
    {#if (showDebugBadge ?? $showMemoryDebugBadges) && (isLate !== undefined || debugInfo || rawJson)}
      <div class="post-dev-debug-badge">
        <div class="debug-badge-inner {isLate ? 'is-late' : 'is-ontime'}">
          <span class="status-dot"></span>
          <span class="debug-text">{isLate ? `LATE ${lateDuration ? '• ' + lateDuration : ''}` : 'ON TIME'}</span>
          {#if takenAt}
            <span class="debug-time">{takenAt.slice(11, 16)}</span>
          {/if}
        </div>

        <!-- Rich Diagnostics Tooltip Popover on Hover -->
        <div class="debug-popover-card">
          <div class="popover-header">
            <span class="popover-badge {isLate ? 'badge-late' : 'badge-ontime'}">
              {isLate ? '⚠️ Late Submission' : '✓ On-Time Delivery'}
            </span>
            {#if takenAt}
              <span class="popover-time font-mono">{takenAt.slice(11, 19)} UTC</span>
            {/if}
          </div>

          {#if debugInfo || lateExact}
            <div class="popover-detail-row">
              <span class="detail-label">Timing Offset:</span>
              <span class="detail-value">{lateExact || debugInfo}</span>
            </div>
          {/if}

          {#if rawJson}
            <div class="popover-json-preview">
              <pre class="raw-json-text">{rawJson.slice(0, 180)}{rawJson.length > 180 ? '...' : ''}</pre>
            </div>
          {/if}
        </div>
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
          title={size === 'lg' ? "Click to swap cameras • Drag to move" : "Click to swap cameras"}
          aria-label={size === 'lg' ? "Selfie camera inset — Click to swap, drag to move" : "Selfie camera inset — Click to swap"}
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

    <!-- BTS Micro-Video Play Trigger & Audio Toggle (Only shown in interactive modes) -->
    {#if btsSrc && interactive}
      <div
        class="bts-controls-cluster"
        class:pos-left={pipCorner === 'bottom-right' && !hasMovedCustom}
      >
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

    <!-- Video Playback Control Bar with Interactive Scrubber, Play/Pause, Scrubber, Timestamp, and Audio -->
    {#if hasVideoContent && (size === 'lg' || isHovered || isPlaying)}
      <div
        class="video-playback-controls-bar"
        class:is-active={isHovered || isPlaying || isSeeking || size === 'lg'}
      >
        <!-- Play / Pause Button -->
        <button
          type="button"
          class="playback-play-btn"
          on:click={togglePlayPause}
          title={isPlaying ? 'Pause Video' : 'Play Video'}
          aria-label={isPlaying ? 'Pause Video' : 'Play Video'}
        >
          {#if isPlaying}
            <Pause size={12} class="fill-current" />
          {:else}
            <Play size={12} class="fill-current" />
          {/if}
        </button>

        <!-- Interactive Scrubber Progress Bar -->
        <div
          bind:this={playbackTrackEl}
          class="playback-progress-track"
          class:is-seeking={isSeeking}
          role="slider"
          tabindex="0"
          aria-label="Video Playback Progress"
          aria-valuenow={Math.round(currentTime)}
          aria-valuemin="0"
          aria-valuemax={Math.round(duration)}
          on:pointerdown={handleScrubberPointerDown}
          on:pointermove={handleScrubberPointerMove}
          on:pointerup={handleScrubberPointerUp}
          on:pointercancel={handleScrubberPointerUp}
        >
          <div class="progress-rail-bg"></div>
          <div
            class="progress-fill-bar"
            style="width: {duration > 0 ? Math.min(100, Math.max(0, (currentTime / duration) * 100)) : 0}%;"
          ></div>
          <div
            class="progress-scrub-thumb"
            style="left: {duration > 0 ? Math.min(100, Math.max(0, (currentTime / duration) * 100)) : 0}%;"
          ></div>
        </div>

        <!-- Current / Total Time Display -->
        <span class="playback-time-text">
          {formatVideoTime(currentTime)} / {formatVideoTime(duration)}
        </span>

        <!-- Audio Mute / Unmute Button -->
        <button
          type="button"
          class="playback-audio-btn"
          class:is-muted={isPlayingBts ? isBtsMuted : isVideoMuted}
          on:click={isPlayingBts ? toggleBtsMute : toggleVideoMute}
          title={(isPlayingBts ? isBtsMuted : isVideoMuted) ? 'Unmute Audio' : 'Mute Audio'}
          aria-label="Toggle audio"
        >
          <VolumeIcon muted={isPlayingBts ? isBtsMuted : isVideoMuted} size={12} />
        </button>
      </div>
    {/if}

    <!-- Interactive Floating Zoom Toolbar (Only shown when zoomed in) -->
    {#if zoomLevel > 1.01}
      <div class="zoom-floating-controls is-zoomed">
        <button
          type="button"
          class="zoom-btn"
          disabled={zoomLevel <= 1.01}
          on:click={zoomOut}
          title="Zoom Out (-)"
          aria-label="Zoom Out"
        >
          <Minus size={12} />
        </button>

        <button
          type="button"
          class="zoom-level-pill"
          on:click={resetZoom}
          title="Reset Zoom (Double-Click image or press 0)"
          aria-label="Reset Zoom"
        >
          <span>{Math.round(zoomLevel * 100)}%</span>
        </button>

        <button
          type="button"
          class="zoom-btn"
          disabled={zoomLevel >= 4.0}
          on:click={zoomIn}
          title="Zoom In (+)"
          aria-label="Zoom In"
        >
          <Plus size={12} />
        </button>

        <button
          type="button"
          class="zoom-btn reset-btn"
          on:click={resetZoom}
          title="Reset Zoom to 100%"
          aria-label="Reset Zoom"
        >
          <RotateCcw size={11} />
        </button>
      </div>
    {/if}
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

  .corner-badge-wrap {
    position: absolute;
    top: 9px;
    right: 9px;
    display: flex;
    align-items: center;
    gap: 5px;
    z-index: 25;
  }

  /* Corner badge (e.g. retakes count, post count, or clickable BTS button) */
  .corner-badge {
    position: absolute;
    top: 9px;
    right: 9px;
    padding: 4px 10px;
    background: rgba(0, 0, 0, 0.78);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-full);
    font-size: 11px;
    font-weight: 700;
    color: #ffffff;
    z-index: 25;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    box-sizing: border-box;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.6);
  }

  .corner-badge-wrap .corner-badge {
    position: static;
  }

  .size-lg .corner-badge {
    top: 12px;
    right: 12px;
    padding: 5px 12px;
    font-size: 11.5px;
    gap: 6px;
  }

  .bts-corner-btn {
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    outline: none;
    user-select: none;
  }

  .bts-corner-btn:hover {
    background: rgba(56, 189, 248, 0.9);
    color: #000000;
    border-color: #38bdf8;
    transform: scale(1.05);
    box-shadow: 0 2px 12px rgba(56, 189, 248, 0.45);
  }

  .bts-corner-btn.active {
    background: #38bdf8;
    color: #000000;
    border-color: #38bdf8;
    box-shadow: 0 0 14px rgba(56, 189, 248, 0.6);
  }

  .bts-corner-audio-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.78);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: #ffffff;
    cursor: pointer;
    padding: 0;
    transition: all 0.2s ease;
  }

  .size-lg .bts-corner-audio-btn {
    width: 26px;
    height: 26px;
  }

  .bts-corner-audio-btn:hover {
    background: rgba(255, 255, 255, 0.22);
    transform: scale(1.08);
  }

  .bts-corner-audio-btn.is-muted {
    color: #f87171;
    border-color: rgba(248, 113, 113, 0.4);
  }

  :global(.bts-icon) {
    flex-shrink: 0;
  }

  /* On-Post Dev Debug Extraction Badge */
  .post-dev-debug-badge {
    position: absolute;
    top: 8px;
    left: 8px;
    z-index: 35;
    pointer-events: auto;
    cursor: default;
  }

  .debug-badge-inner {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 8px;
    border-radius: var(--radius-full);
    font-size: 9px;
    font-weight: 700;
    backdrop-filter: blur(16px);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.75);
    letter-spacing: 0.03em;
    text-transform: uppercase;
    transition: transform 0.15s ease, box-shadow 0.15s ease;
  }

  .post-dev-debug-badge:hover .debug-badge-inner {
    transform: scale(1.04);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.9);
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    display: inline-block;
  }

  .debug-badge-inner.is-late {
    background: rgba(20, 10, 10, 0.85);
    border: 1px solid rgba(248, 113, 113, 0.5);
    color: #fca5a5;
  }

  .debug-badge-inner.is-late .status-dot {
    background: #ef4444;
    box-shadow: 0 0 6px #ef4444;
  }

  .debug-badge-inner.is-ontime {
    background: rgba(6, 22, 16, 0.85);
    border: 1px solid rgba(52, 211, 153, 0.5);
    color: #6ee7b7;
  }

  .debug-badge-inner.is-ontime .status-dot {
    background: #10b981;
    box-shadow: 0 0 6px #10b981;
  }

  .debug-time {
    font-family: var(--font-mono);
    font-size: 8.5px;
    opacity: 0.85;
    padding-left: 2px;
  }

  /* Popover Hover Card */
  .debug-popover-card {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    width: 240px;
    background: rgba(15, 17, 23, 0.96);
    backdrop-filter: blur(20px);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 10px 12px;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.85);
    display: flex;
    flex-direction: column;
    gap: 8px;
    opacity: 0;
    visibility: hidden;
    transform: translateY(-4px);
    transition: all 0.18s cubic-bezier(0.16, 1, 0.3, 1);
    pointer-events: none;
    z-index: 50;
  }

  .post-dev-debug-badge:hover .debug-popover-card {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
    pointer-events: auto;
  }

  .popover-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .popover-badge {
    font-size: 10px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: var(--radius-xs);
  }

  .badge-late {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .badge-ontime {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.3);
  }

  .popover-time {
    font-size: 10px;
    color: var(--text-muted);
  }

  .popover-detail-row {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 11px;
  }

  .detail-label {
    color: var(--text-muted);
    font-size: 10px;
  }

  .detail-value {
    color: var(--text-primary);
    font-weight: 500;
  }

  .popover-json-preview {
    background: rgba(0, 0, 0, 0.4);
    border-radius: var(--radius-xs);
    padding: 6px 8px;
    max-height: 80px;
    overflow-y: auto;
  }

  .raw-json-text {
    font-family: var(--font-mono);
    font-size: 9.5px;
    color: #94a3b8;
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .size-sm .debug-badge-inner {
    font-size: 8px;
    padding: 1px 5px;
  }

  /* Video indicator badge when paused */
  .video-indicator-badge {
    position: absolute;
    bottom: 8px;
    right: 8px;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.25);
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.6);
    pointer-events: auto;
    cursor: pointer;
    z-index: 15;
    transition: opacity 0.15s ease, transform 0.15s ease, background 0.15s ease;
    padding: 0;
  }

  .video-indicator-badge:hover {
    background: #38bdf8;
    color: #000000;
    border-color: #38bdf8;
    transform: scale(1.12);
  }

  .size-lg .video-indicator-badge {
    width: 36px;
    height: 36px;
    bottom: 14px;
    right: 14px;
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
    transition: all 0.2s ease;
  }

  .bts-controls-cluster.pos-left {
    right: auto;
    left: 10px;
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

  /* Video Playback Scrubber & Control Bar */
  .video-playback-controls-bar {
    position: absolute;
    bottom: 10px;
    left: 8px;
    right: 8px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    background: rgba(8, 8, 14, 0.92);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: var(--radius-full);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.8);
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.2s ease, transform 0.2s ease;
    z-index: 32;
  }

  .bereal-frame-container:hover .video-playback-controls-bar,
  .video-playback-controls-bar.is-active {
    opacity: 1;
    pointer-events: auto;
  }

  .size-lg .zoom-floating-controls {
    bottom: 44px;
  }

  .playback-play-btn,
  .playback-audio-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: #ffffff;
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
    transition: all 0.15s ease;
  }

  .playback-play-btn:hover,
  .playback-audio-btn:hover {
    background: #38bdf8;
    color: #000000;
    border-color: #38bdf8;
    transform: scale(1.1);
  }

  .playback-audio-btn.is-muted {
    color: #f87171;
    border-color: rgba(248, 113, 113, 0.4);
  }

  .playback-progress-track {
    position: relative;
    flex: 1;
    height: 18px;
    display: flex;
    align-items: center;
    cursor: pointer;
    user-select: none;
    touch-action: none;
  }

  .progress-rail-bg {
    position: absolute;
    left: 0;
    right: 0;
    height: 4px;
    background: rgba(255, 255, 255, 0.18);
    border-radius: 999px;
    transition: height 0.15s ease;
  }

  .playback-progress-track:hover .progress-rail-bg,
  .playback-progress-track.is-seeking .progress-rail-bg {
    height: 5px;
  }

  .progress-fill-bar {
    position: absolute;
    left: 0;
    height: 4px;
    background: #38bdf8;
    border-radius: 999px;
    pointer-events: none;
    transition: height 0.15s ease;
  }

  .playback-progress-track:hover .progress-fill-bar,
  .playback-progress-track.is-seeking .progress-fill-bar {
    height: 5px;
  }

  .progress-scrub-thumb {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%) scale(0);
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #ffffff;
    box-shadow: 0 0 8px rgba(56, 189, 248, 0.8);
    pointer-events: none;
    transition: transform 0.15s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .playback-progress-track:hover .progress-scrub-thumb,
  .playback-progress-track.is-seeking .progress-scrub-thumb {
    transform: translate(-50%, -50%) scale(1);
  }

  .playback-time-text {
    font-size: 10px;
    font-weight: 600;
    font-family: var(--font-mono);
    color: #a1a1aa;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .large-canvas-wrap.is-zoomed {
    transition: none !important;
    user-select: none;
    touch-action: none;
  }

  /* Floating Zoom Control Bar */
  .zoom-floating-controls {
    position: absolute;
    bottom: 12px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 3px 6px;
    background: rgba(10, 10, 16, 0.92);
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-full);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.8);
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.18s ease, transform 0.18s ease;
    z-index: 40;
  }

  .zoom-floating-controls.is-zoomed {
    opacity: 1;
    pointer-events: auto;
  }

  .zoom-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #ffffff;
    cursor: pointer;
    padding: 0;
    transition: all 0.15s ease;
  }

  .zoom-btn:hover:not(:disabled) {
    background: #38bdf8;
    color: #000000;
    border-color: #38bdf8;
    transform: scale(1.1);
  }

  .zoom-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .zoom-level-pill {
    padding: 2px 7px;
    font-size: 10px;
    font-weight: 700;
    font-family: var(--font-mono);
    color: #ffffff;
    background: transparent;
    border: none;
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: background 0.15s ease, color 0.15s ease;
  }

  .zoom-level-pill:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #38bdf8;
  }

  .zoom-btn.reset-btn {
    width: 22px;
    height: 22px;
    background: rgba(248, 113, 113, 0.15);
    border-color: rgba(248, 113, 113, 0.3);
    color: #f87171;
  }

  .zoom-btn.reset-btn:hover {
    background: #ef4444;
    color: #ffffff;
    border-color: #ef4444;
  }
</style>
