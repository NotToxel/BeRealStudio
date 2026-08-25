<script lang="ts">
  import type { SpeedMode } from '$lib/types';
  import { onMount, onDestroy } from 'svelte';

  export let mode: SpeedMode;
  export let animated: boolean = false;
  export let count: number = 15;
  export let startPadding: number = 0;
  export let endPadding: number = 0;

  // ─── Curve math (mirrors timing.rs exactly) ──────────────────────────────────

  function computeWeights(m: SpeedMode, n: number): number[] {
    if (n <= 1) return [1];
    const weights: number[] = [];
    for (let i = 0; i < n; i++) {
      const t = i / (n - 1);
      let w: number;
      switch (m) {
        case 'Ramp': {
          const x = -1 + 2 * t;
          w = 1 + 2 * x * x;
          break;
        }
        case 'Even':
          w = 1;
          break;
        case 'Accelerate':
          w = 1 + 2.5 * (1 - t) * (1 - t);
          break;
        case 'Decelerate':
          w = 1 + 2.5 * t * t;
          break;
        case 'Wave':
          w = 1 + Math.abs(Math.sin(Math.PI * 3 * t));
          break;
        default:
          w = 1;
      }
      weights.push(w);
    }
    return weights;
  }

  const W = 104;
  const H = 32;

  function buildSparkline(m: SpeedMode): { path: string; fillPath: string } {
    const n = 60;
    const weights = computeWeights(m, n);
    const maxW = Math.max(...weights);
    const minW = Math.min(...weights);
    const range = maxW - minW || 1;

    const pts = weights.map((w, i) => {
      const x = (i / (n - 1)) * W;
      const y = H - 2 - ((w - minW) / range) * (H - 6);
      return [x, y];
    });

    const lineParts = pts.map(([x, y], i) => `${i === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`);
    const path = lineParts.join(' ');
    const fillPath = `${path} L${W},${H} L0,${H} Z`;

    return { path, fillPath };
  }

  // ─── Timeline Simulation with Padding ────────────────────────────────────────

  let animProgress = 0;
  let animFrame: number;
  let animStart: number | null = null;

  // Compute total simulated duration including start & end padding (in relative weight units)
  // Let photo sequence be approx 8 seconds of normalized motion weight
  const PHOTO_BASE_DUR = 6.0;
  $: totalSimDuration = startPadding + PHOTO_BASE_DUR + endPadding;
  $: animDurationMs = Math.max(3000, Math.min(8000, totalSimDuration * 600));

  function startAnimation() {
    if (!animated) return;
    function tick(now: number) {
      if (animStart === null) animStart = now;
      const elapsed = (now - animStart) % animDurationMs;
      animProgress = elapsed / animDurationMs;
      animFrame = requestAnimationFrame(tick);
    }
    animFrame = requestAnimationFrame(tick);
  }

  function stopAnimation() {
    if (animFrame) cancelAnimationFrame(animFrame);
  }

  onMount(() => {
    if (animated) startAnimation();
  });
  onDestroy(() => stopAnimation());

  $: if (animated) {
    stopAnimation();
    animStart = null;
    startAnimation();
  }

  $: weights = computeWeights(mode, count);
  $: maxW = Math.max(...weights);

  // Time segment portions [0..1]
  $: startRatio = totalSimDuration > 0 ? startPadding / totalSimDuration : 0;
  $: endRatio = totalSimDuration > 0 ? endPadding / totalSimDuration : 0;
  $: photoRatio = totalSimDuration > 0 ? PHOTO_BASE_DUR / totalSimDuration : 1;

  // Dot position along whole timeline track [0..1]
  $: dotX = (() => {
    if (animProgress < startRatio && startRatio > 0) {
      // In Start Padding Hold
      const t = animProgress / startRatio;
      return t * 0.14; // start padding area (left 14%)
    } else if (animProgress > 1 - endRatio && endRatio > 0) {
      // In End Padding Hold
      const t = (animProgress - (1 - endRatio)) / endRatio;
      return 0.86 + t * 0.14; // end padding area (right 14%)
    } else {
      // In Photo Sequence
      const normProgress = startRatio > 0
        ? (animProgress - startRatio) / photoRatio
        : (animProgress / (1 - endRatio || 1));
      
      const total = weights.reduce((a, b) => a + b, 0);
      let cum = 0;
      let photoT = 0;
      for (let i = 0; i < weights.length; i++) {
        const next = cum + weights[i];
        if (normProgress <= next / total) {
          const t = (normProgress - cum / total) / (weights[i] / total);
          photoT = (i + t) / count;
          break;
        }
        cum += weights[i];
      }

      const leftBound = startRatio > 0 ? 0.15 : 0;
      const rightBound = endRatio > 0 ? 0.85 : 1;
      return leftBound + photoT * (rightBound - leftBound);
    }
  })();

  $: sparkline = buildSparkline(mode);

  const accentColors: Record<SpeedMode, string> = {
    Ramp: '#38bdf8',
    Even: '#a3e635',
    Accelerate: '#fb923c',
    Decelerate: '#c084fc',
    Wave: '#f472b6',
  };
  $: accent = accentColors[mode] ?? '#38bdf8';
</script>

{#if animated}
  <div class="timeline-wrap">
    <div class="timeline-head">
      <span class="timeline-label">Timeline Pacing &amp; Hold Dynamics:</span>
      <div class="timeline-badges">
        {#if startPadding > 0}
          <span class="pad-badge pad-start font-mono">Start Hold: {startPadding}s</span>
        {/if}
        <span class="mode-badge font-mono" style="color: {accent};">{mode} Curve</span>
        {#if endPadding > 0}
          <span class="pad-badge pad-end font-mono">End Hold: {endPadding}s</span>
        {/if}
      </div>
    </div>

    <div class="timeline-track">
      <!-- Start Padding Hold Block -->
      {#if startPadding > 0}
        <div
          class="padding-block start-pad-block"
          style="width: 14%;"
          title="Start Hold: {startPadding}s holding first frame"
        >
          <span class="pad-block-label">Hold</span>
        </div>
      {/if}

      <!-- Main Photo Sequence Bars -->
      <div class="photo-sequence-track" style="flex: 1;">
        {#each weights as w, i}
          {@const barH = 6 + (w / maxW) * 16}
          <div
            class="slot-bar"
            style="
              height: {barH}px;
              background: {accent};
              opacity: {0.35 + (w / maxW) * 0.65};
            "
          ></div>
        {/each}
      </div>

      <!-- End Padding Hold Block -->
      {#if endPadding > 0}
        <div
          class="padding-block end-pad-block"
          style="width: 14%;"
          title="End Hold: {endPadding}s holding final frame"
        >
          <span class="pad-block-label">Hold</span>
        </div>
      {/if}

      <!-- Glowing Animated Dot -->
      <div
        class="timeline-dot"
        style="
          left: calc({dotX * 100}% - 5px);
          background: {accent};
          box-shadow: 0 0 12px {accent}, 0 0 4px #ffffff;
        "
      ></div>
    </div>
  </div>
{:else}
  <svg
    class="sparkline"
    width={W}
    height={H}
    viewBox="0 0 {W} {H}"
    xmlns="http://www.w3.org/2000/svg"
    aria-label="{mode} speed curve"
  >
    <defs>
      <linearGradient id="grad-{mode}" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color={accent} stop-opacity="0.35" />
        <stop offset="100%" stop-color={accent} stop-opacity="0.02" />
      </linearGradient>
    </defs>
    <path d={sparkline.fillPath} fill="url(#grad-{mode})" />
    <path
      d={sparkline.path}
      fill="none"
      stroke={accent}
      stroke-width="1.5"
      stroke-linejoin="round"
      stroke-linecap="round"
    />
  </svg>
{/if}

<style>
  .sparkline {
    display: block;
    flex-shrink: 0;
  }

  .timeline-wrap {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
  }

  .timeline-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
  }

  .timeline-label {
    font-size: 11px;
    color: var(--text-muted);
    letter-spacing: 0.01em;
  }

  .timeline-badges {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .pad-badge {
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.07);
    color: var(--text-secondary);
  }

  .mode-badge {
    font-size: 10.5px;
    font-weight: 600;
  }

  .timeline-track {
    position: relative;
    display: flex;
    align-items: flex-end;
    gap: 3px;
    height: 32px;
    padding: 0 4px;
    background: #07070a;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .padding-block {
    height: 24px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px dashed rgba(255, 255, 255, 0.18);
    border-radius: 2px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 2px;
  }

  .pad-block-label {
    font-size: 9px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .photo-sequence-track {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 100%;
  }

  .slot-bar {
    flex: 1;
    border-radius: 2px 2px 0 0;
    transition: background 0.08s;
    min-width: 0;
  }

  .timeline-dot {
    position: absolute;
    bottom: 2px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    transform: translateX(0);
    transition: left 0.016s linear;
    z-index: 10;
  }
</style>
