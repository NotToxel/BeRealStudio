<script lang="ts">
  import type { MonthCount } from '$lib/types';
  import Calendar from 'lucide-svelte/icons/calendar';
  import Activity from 'lucide-svelte/icons/activity';
  import RotateCcw from 'lucide-svelte/icons/rotate-ccw';
  import Sparkles from 'lucide-svelte/icons/sparkles';

  export let histogram: MonthCount[] = [];
  export let minDate: string = '';
  export let maxDate: string = '';
  export let startDate: string = '';
  export let endDate: string = '';
  export let totalCount: number = 0;
  export let selectedCount: number = 0;

  // ─── SVG Coordinate System & Dimensions ──────────────────────────────────────
  const SVG_W = 1000;
  const SVG_H = 130;
  const PAD_X = 36;
  const PAD_TOP = 20;
  const PAD_BOTTOM = 34;
  const CHART_W = SVG_W - PAD_X * 2;
  const CHART_H = SVG_H - PAD_TOP - PAD_BOTTOM;

  $: maxCount = Math.max(...histogram.map((h) => h.count), 1);

  // Month date range helper
  function isMonthInRange(monthStr: string): boolean {
    if (!startDate && !endDate) return true;
    const mStart = `${monthStr}-01`;
    const mEnd = `${monthStr}-31`;
    if (startDate && mEnd < startDate) return false;
    if (endDate && mStart > endDate) return false;
    return true;
  }

  // Reactively calculate selected entries count from histogram
  $: {
    if (!startDate && !endDate) {
      selectedCount = totalCount;
    } else if (histogram.length > 0) {
      const count = histogram
        .filter((h) => isMonthInRange(h.month))
        .reduce((sum, h) => sum + h.count, 0);
      selectedCount = count;
    } else {
      selectedCount = totalCount;
    }
  }

  // ─── Smooth Curve Math (Monotone Spline / Catmull-Rom) ───────────────────────

  interface Point {
    x: number;
    y: number;
    count: number;
    month: string;
    label: string;
    isFirstOfYear: boolean;
  }

  const MONTH_NAMES = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  function formatMonthShort(mStr: string): string {
    const parts = mStr.split('-');
    if (parts.length < 2) return mStr;
    const mIdx = parseInt(parts[1], 10) - 1;
    const yr = parts[0].slice(2);
    return `${MONTH_NAMES[mIdx]} '${yr}`;
  }

  function formatMonthFull(mStr: string): string {
    const parts = mStr.split('-');
    if (parts.length < 2) return mStr;
    const mIdx = parseInt(parts[1], 10) - 1;
    const yr = parts[0];
    const fullMonths = [
      'January', 'February', 'March', 'April', 'May', 'June',
      'July', 'August', 'September', 'October', 'November', 'December'
    ];
    return `${fullMonths[mIdx]} ${yr}`;
  }

  $: points = (() => {
    const n = histogram.length;
    if (n === 0) return [];
    return histogram.map((h, i) => {
      const x = n === 1 ? SVG_W / 2 : PAD_X + (i / (n - 1)) * CHART_W;
      const normalized = Math.max(h.count / maxCount, 0.04);
      const y = SVG_H - PAD_BOTTOM - normalized * CHART_H;
      const isFirstOfYear = h.month.endsWith('-01');
      return {
        x,
        y,
        count: h.count,
        month: h.month,
        label: formatMonthShort(h.month),
        isFirstOfYear,
      };
    });
  })();

  // Generate smooth cubic bezier SVG path from points
  function buildSmoothPath(pts: Point[]): { pathStr: string; areaStr: string } {
    if (pts.length === 0) return { pathStr: '', areaStr: '' };
    if (pts.length === 1) {
      const p = pts[0];
      const pathStr = `M ${p.x - 20} ${p.y} L ${p.x + 20} ${p.y}`;
      const areaStr = `M ${p.x - 20} ${p.y} L ${p.x + 20} ${p.y} L ${p.x + 20} ${SVG_H - PAD_BOTTOM} L ${p.x - 20} ${SVG_H - PAD_BOTTOM} Z`;
      return { pathStr, areaStr };
    }

    let d = `M ${pts[0].x.toFixed(1)} ${pts[0].y.toFixed(1)}`;
    for (let i = 0; i < pts.length - 1; i++) {
      const p0 = pts[i === 0 ? 0 : i - 1];
      const p1 = pts[i];
      const p2 = pts[i + 1];
      const p3 = pts[i + 2 >= pts.length ? pts.length - 1 : i + 2];

      const cp1x = p1.x + (p2.x - p0.x) / 6;
      const cp1y = p1.y + (p2.y - p0.y) / 6;
      const cp2x = p2.x - (p3.x - p1.x) / 6;
      const cp2y = p2.y - (p3.y - p1.y) / 6;

      d += ` C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${p2.x.toFixed(1)} ${p2.y.toFixed(1)}`;
    }

    const baselineY = SVG_H - PAD_BOTTOM;
    const areaStr = `${d} L ${pts[pts.length - 1].x.toFixed(1)} ${baselineY} L ${pts[0].x.toFixed(1)} ${baselineY} Z`;
    return { pathStr: d, areaStr };
  }

  $: curvePaths = buildSmoothPath(points);

  // ─── Date / Coordinate Conversion Helpers ─────────────────────────────────────

  function dateToX(dateStr: string): number {
    if (!dateStr || points.length === 0) return PAD_X;
    if (points.length === 1) return points[0].x;

    const firstMonth = histogram[0].month;
    const lastMonth = histogram[histogram.length - 1].month;

    const minT = new Date(`${firstMonth}-01`).getTime();
    const lastParts = lastMonth.split('-');
    const lastY = parseInt(lastParts[0], 10);
    const lastM = parseInt(lastParts[1], 10);
    const lastDay = new Date(lastY, lastM, 0).getDate();
    const maxT = new Date(`${lastMonth}-${lastDay}`).getTime();

    const curT = new Date(dateStr).getTime();
    if (isNaN(curT)) return PAD_X;

    const ratio = Math.max(0, Math.min(1, (curT - minT) / (maxT - minT || 1)));
    return PAD_X + ratio * CHART_W;
  }

  function xToDate(x: number): string {
    if (points.length === 0) return '';
    const ratio = Math.max(0, Math.min(1, (x - PAD_X) / (CHART_W || 1)));

    const firstMonth = histogram[0].month;
    const lastMonth = histogram[histogram.length - 1].month;

    const minT = new Date(`${firstMonth}-01`).getTime();
    const lastParts = lastMonth.split('-');
    const lastY = parseInt(lastParts[0], 10);
    const lastM = parseInt(lastParts[1], 10);
    const lastDay = new Date(lastY, lastM, 0).getDate();
    const maxT = new Date(`${lastMonth}-${lastDay}`).getTime();

    const targetT = minT + ratio * (maxT - minT);
    const targetDate = new Date(targetT);
    return formatDate(targetDate);
  }

  function formatDate(d: Date): string {
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  }

  // Active selection box coordinates
  $: selStartX = startDate ? dateToX(startDate) : PAD_X;
  $: selEndX = endDate ? dateToX(endDate) : PAD_X + CHART_W;
  $: boxLeft = Math.min(selStartX, selEndX);
  $: boxRight = Math.max(selStartX, selEndX);
  $: boxWidth = Math.max(boxRight - boxLeft, 2);

  // ─── Click & Drag Selection State ───────────────────────────────────────────

  let svgElement: SVGSVGElement | null = null;
  let isDragging = false;
  let dragAnchorX: number | null = null;
  let dragMode: 'new' | 'left-handle' | 'right-handle' | 'pan' = 'new';
  let hoveredPoint: Point | null = null;
  let cursorSvgX = 0;
  let cursorSvgY = 0;

  function getSvgPoint(e: MouseEvent): { x: number; y: number } {
    if (!svgElement) return { x: 0, y: 0 };
    const rect = svgElement.getBoundingClientRect();
    const scaleX = SVG_W / rect.width;
    const scaleY = SVG_H / rect.height;
    return {
      x: (e.clientX - rect.left) * scaleX,
      y: (e.clientY - rect.top) * scaleY,
    };
  }

  function handleSvgMouseDown(e: MouseEvent) {
    if (histogram.length === 0) return;
    const { x } = getSvgPoint(e);
    isDragging = true;

    const leftDist = Math.abs(x - boxLeft);
    const rightDist = Math.abs(x - boxRight);

    if (leftDist <= 14) {
      dragMode = 'left-handle';
      dragAnchorX = boxRight; // preserve opposite side
    } else if (rightDist <= 14) {
      dragMode = 'right-handle';
      dragAnchorX = boxLeft;
    } else if (x > boxLeft + 8 && x < boxRight - 8) {
      dragMode = 'pan';
      dragAnchorX = x;
    } else {
      // Start a brand new drag selection box from this exact point
      dragMode = 'new';
      dragAnchorX = x;
      const clickedDate = xToDate(x);
      startDate = clickedDate;
      endDate = clickedDate;
    }

    activePreset = 'custom';
  }

  function handleWindowMouseMove(e: MouseEvent) {
    if (!svgElement) return;
    if (!isDragging) {
      const rect = svgElement.getBoundingClientRect();
      if (
        e.clientX < rect.left - 20 ||
        e.clientX > rect.right + 20 ||
        e.clientY < rect.top - 20 ||
        e.clientY > rect.bottom + 20
      ) {
        if (hoveredPoint !== null) hoveredPoint = null;
        return;
      }
    }

    const { x, y } = getSvgPoint(e);
    cursorSvgX = x;
    cursorSvgY = y;

    // Track hovered marker
    if (points.length > 0) {
      const closest = points.reduce((best, p) => {
        const d = Math.abs(p.x - x);
        return d < Math.abs(best.x - x) ? p : best;
      }, points[0]);
      hoveredPoint = Math.abs(closest.x - x) < 30 ? closest : null;
    }

    if (!isDragging || dragAnchorX === null) return;

    if (dragMode === 'new' || dragMode === 'left-handle' || dragMode === 'right-handle') {
      const minX = Math.min(dragAnchorX, x);
      const maxX = Math.max(dragAnchorX, x);
      startDate = xToDate(minX);
      endDate = xToDate(maxX);
    } else if (dragMode === 'pan') {
      const deltaX = x - dragAnchorX;
      dragAnchorX = x;
      const curLeft = boxLeft;
      const curRight = boxRight;
      const newLeft = Math.max(PAD_X, Math.min(PAD_X + CHART_W - boxWidth, curLeft + deltaX));
      const newRight = newLeft + boxWidth;
      startDate = xToDate(newLeft);
      endDate = xToDate(newRight);
    }
  }

  function handleWindowMouseUp() {
    if (isDragging) {
      isDragging = false;
      dragAnchorX = null;
      // If start and end are identical single click, expand to full month of that point
      if (startDate === endDate && hoveredPoint) {
        handlePointClick(hoveredPoint);
      }
    }
  }

  function handlePointClick(p: Point) {
    activePreset = 'custom';
    startDate = `${p.month}-01`;
    const parts = p.month.split('-');
    const y = parseInt(parts[0], 10);
    const m = parseInt(parts[1], 10);
    const lastDay = new Date(y, m, 0).getDate();
    endDate = `${p.month}-${String(lastDay).padStart(2, '0')}`;
  }

  // ─── Presets ────────────────────────────────────────────────────────────────

  let activePreset: 'all' | '30d' | '6m' | '1y' | 'custom' = 'all';

  function setPreset(preset: 'all' | '30d' | '6m' | '1y') {
    activePreset = preset;
    if (!maxDate && histogram.length === 0) return;
    const reference = maxDate ? new Date(maxDate) : new Date();

    if (preset === 'all') {
      startDate = minDate || (histogram.length > 0 ? `${histogram[0].month}-01` : '');
      endDate = maxDate || (histogram.length > 0 ? `${histogram[histogram.length - 1].month}-28` : '');
      return;
    }

    const end = new Date(reference);
    const start = new Date(reference);

    if (preset === '30d') {
      start.setDate(end.getDate() - 30);
    } else if (preset === '6m') {
      start.setMonth(end.getMonth() - 6);
    } else if (preset === '1y') {
      start.setFullYear(end.getFullYear() - 1);
    }

    startDate = formatDate(start);
    endDate = formatDate(end);
  }

  $: isFiltered = Boolean((startDate && startDate !== minDate) || (endDate && endDate !== maxDate));
</script>

<svelte:window on:mousemove={handleWindowMouseMove} on:mouseup={handleWindowMouseUp} />

<div class="range-picker card">
  <!-- Header Bar -->
  <div class="header">
    <div class="title-group">
      <Activity size={16} class="text-amber-400" />
      <span class="title-sm font-semibold">Date Range &amp; Timeline</span>
      {#if totalCount > 0}
        <span class="badge {isFiltered ? 'badge-yellow' : 'badge-neutral'} font-mono">
          <Sparkles size={11} />
          {selectedCount} of {totalCount} Memories ({Math.round((selectedCount / totalCount) * 100 || 100)}%)
        </span>
      {/if}
    </div>

    <!-- Quick Presets Pills -->
    <div class="presets">
      <button
        type="button"
        class="preset-btn"
        class:active={!isFiltered || activePreset === 'all'}
        on:click={() => setPreset('all')}
      >
        All Time
      </button>
      <button
        type="button"
        class="preset-btn"
        class:active={activePreset === '30d'}
        on:click={() => setPreset('30d')}
      >
        Past 30 Days
      </button>
      <button
        type="button"
        class="preset-btn"
        class:active={activePreset === '6m'}
        on:click={() => setPreset('6m')}
      >
        Past 6 Months
      </button>
      <button
        type="button"
        class="preset-btn"
        class:active={activePreset === '1y'}
        on:click={() => setPreset('1y')}
      >
        Past 1 Year
      </button>
      {#if isFiltered}
        <button
          type="button"
          class="reset-btn"
          on:click={() => setPreset('all')}
          title="Reset to Full Timeline"
        >
          <RotateCcw size={11} />
          <span>Reset</span>
        </button>
      {/if}
    </div>
  </div>

  <!-- Continuous Smooth Curve Timeline -->
  {#if histogram.length > 0}
    <div class="timeline-interactive-area">
      <!-- Instruction & Static Hover Status Bar (Fixed height to prevent any layout shift) -->
      <div class="timeline-hint-row">
        <span class="hint-text">
          Click and drag across the timeline to select a date range • Drag boundary handles to adjust
        </span>
        <span class="hover-stat font-mono">
          {#if hoveredPoint}
            <span class="text-amber-300">{formatMonthFull(hoveredPoint.month)}: <strong>{hoveredPoint.count} Memories</strong></span>
          {:else}
            <span class="stat-placeholder">Hover Timeline for Details</span>
          {/if}
        </span>
      </div>

      <!-- SVG Continuous Line & Area Canvas -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="svg-canvas-container" on:mousedown={handleSvgMouseDown}>
        <svg
          bind:this={svgElement}
          class="timeline-svg"
          viewBox="0 0 {SVG_W} {SVG_H}"
          preserveAspectRatio="none"
        >
          <defs>
            <!-- Background Area Gradient -->
            <linearGradient id="area-grad-dimmed" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#38bdf8" stop-opacity="0.12" />
              <stop offset="100%" stop-color="#38bdf8" stop-opacity="0.0" />
            </linearGradient>

            <!-- Active Selected Area Gradient -->
            <linearGradient id="area-grad-active" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#ffe600" stop-opacity="0.38" />
              <stop offset="60%" stop-color="#f59e0b" stop-opacity="0.15" />
              <stop offset="100%" stop-color="#d97706" stop-opacity="0.0" />
            </linearGradient>

            <!-- Clip path for the active selection box -->
            <clipPath id="selection-clip">
              <rect x={boxLeft} y="0" width={boxWidth} height={SVG_H} />
            </clipPath>
          </defs>

          <!-- 1. Grid Horizontal Baseline & Reference Grid -->
          <line
            x1={PAD_X}
            y1={SVG_H - PAD_BOTTOM}
            x2={SVG_W - PAD_X}
            y2={SVG_H - PAD_BOTTOM}
            stroke="rgba(255, 255, 255, 0.12)"
            stroke-width="1"
          />

          <!-- 2. Base Dimmed Continuous Curve (Outside Selection) -->
          <path
            d={curvePaths.areaStr}
            fill="url(#area-grad-dimmed)"
            opacity="0.6"
          />
          <path
            d={curvePaths.pathStr}
            fill="none"
            stroke="rgba(255, 255, 255, 0.22)"
            stroke-width="1.8"
            stroke-linecap="round"
          />

          <!-- 3. Active Glowing In-Range Curve (Clipped by Selection Box) -->
          <g clip-path="url(#selection-clip)">
            <path
              d={curvePaths.areaStr}
              fill="url(#area-grad-active)"
            />
            <path
              d={curvePaths.pathStr}
              fill="none"
              stroke="#ffe600"
              stroke-width="2.5"
              stroke-linecap="round"
              style="filter: drop-shadow(0 0 6px rgba(255, 230, 0, 0.6));"
            />
          </g>

          <!-- 4. Interactive Selection Box -->
          <g class="selection-group">
            <!-- Box Background -->
            <rect
              x={boxLeft}
              y={PAD_TOP - 6}
              width={boxWidth}
              height={CHART_H + 12}
              fill="rgba(255, 230, 0, 0.07)"
              stroke="rgba(255, 230, 0, 0.2)"
              stroke-width="1"
              rx="4"
              class="selection-rect"
            />

            <!-- Left Handle -->
            <g class="handle left-handle" transform="translate({boxLeft}, 0)">
              <line
                x1="0"
                y1={PAD_TOP - 8}
                x2="0"
                y2={SVG_H - PAD_BOTTOM + 2}
                stroke="#ffe600"
                stroke-width="2"
                stroke-linecap="round"
              />
              <rect
                x="-5"
                y={(PAD_TOP + SVG_H - PAD_BOTTOM) / 2 - 12}
                width="10"
                height="24"
                rx="4"
                fill="#ffe600"
                stroke="#111116"
                stroke-width="1.5"
              />
              <!-- Handle Grip Lines -->
              <line
                x1="-1.5"
                y1={(PAD_TOP + SVG_H - PAD_BOTTOM) / 2 - 5}
                x2="-1.5"
                y2={(PAD_TOP + SVG_H - PAD_BOTTOM) / 2 + 5}
                stroke="#000000"
                stroke-width="1"
              />
              <line
                x1="1.5"
                y1={(PAD_TOP + SVG_H - PAD_BOTTOM) / 2 - 5}
                x2="1.5"
                y2={(PAD_TOP + SVG_H - PAD_BOTTOM) / 2 + 5}
                stroke="#000000"
                stroke-width="1"
              />
            </g>

            <!-- Right Handle -->
            <g class="handle right-handle" transform="translate({boxRight}, 0)">
              <line
                x1="0"
                y1={PAD_TOP - 8}
                x2="0"
                y2={SVG_H - PAD_BOTTOM + 2}
                stroke="#ffe600"
                stroke-width="2"
                stroke-linecap="round"
              />
              <rect
                x="-5"
                y={(PAD_TOP + SVG_H - PAD_BOTTOM) / 2 - 12}
                width="10"
                height="24"
                rx="4"
                fill="#ffe600"
                stroke="#111116"
                stroke-width="1.5"
              />
              <line
                x1="-1.5"
                y1={(PAD_TOP + SVG_H - PAD_BOTTOM) / 2 - 5}
                x2="-1.5"
                y2={(PAD_TOP + SVG_H - PAD_BOTTOM) / 2 + 5}
                stroke="#000000"
                stroke-width="1"
              />
              <line
                x1="1.5"
                y1={(PAD_TOP + SVG_H - PAD_BOTTOM) / 2 - 5}
                x2="1.5"
                y2={(PAD_TOP + SVG_H - PAD_BOTTOM) / 2 + 5}
                stroke="#000000"
                stroke-width="1"
              />
            </g>
          </g>

          <!-- 5. Month Markers & Baseline Labels -->
          {#each points as p, i}
            {@const inRange = isMonthInRange(p.month)}
            {@const showLabel = i === 0 || i === points.length - 1 || p.isFirstOfYear || (points.length < 18 ? true : i % 3 === 0)}

            <!-- Month Tick Line -->
            <line
              x1={p.x}
              y1={SVG_H - PAD_BOTTOM}
              x2={p.x}
              y2={SVG_H - PAD_BOTTOM + (p.isFirstOfYear ? 6 : 3)}
              stroke={p.isFirstOfYear ? 'rgba(255,255,255,0.4)' : 'rgba(255,255,255,0.15)'}
              stroke-width={p.isFirstOfYear ? 1.5 : 1}
            />

            <!-- Marker Dot on Curve -->
            <circle
              cx={p.x}
              cy={p.y}
              r={inRange ? 3.5 : 2}
              fill={inRange ? '#ffe600' : 'rgba(255, 255, 255, 0.3)'}
              stroke="#0a0a0e"
              stroke-width="1.5"
              class="point-circle"
              class:point-active={inRange}
              class:point-hovered={hoveredPoint?.month === p.month}
            />

            <!-- Month Label Text -->
            {#if showLabel}
              <text
                x={p.x}
                y={SVG_H - 12}
                text-anchor="middle"
                font-size="10"
                font-family="monospace"
                fill={p.isFirstOfYear ? '#ffffff' : inRange ? '#ffe600' : 'rgba(255, 255, 255, 0.45)'}
                font-weight={p.isFirstOfYear || inRange ? '600' : '400'}
              >
                {p.label}
              </text>
            {/if}
          {/each}

          <!-- 6. Cursor Tracking Guide Line -->
          {#if isDragging || hoveredPoint}
            <line
              x1={cursorSvgX}
              y1={PAD_TOP - 8}
              x2={cursorSvgX}
              y2={SVG_H - PAD_BOTTOM}
              stroke="rgba(255, 230, 0, 0.5)"
              stroke-dasharray="3,3"
              stroke-width="1"
            />
          {/if}
        </svg>
      </div>
    </div>
  {/if}

  <!-- Dual Interactive Date Input Pickers -->
  <div class="inputs-row">
    <div class="input-field">
      <label for="start-date">
        <Calendar size={13} class="text-amber-400" />
        <span>From Date (Start)</span>
      </label>
      <input
        id="start-date"
        type="date"
        class="input-text font-mono"
        bind:value={startDate}
        on:change={() => (activePreset = 'custom')}
        min={minDate}
        max={endDate || maxDate}
      />
    </div>

    <div class="input-field">
      <label for="end-date">
        <Calendar size={13} class="text-amber-400" />
        <span>To Date (End)</span>
      </label>
      <input
        id="end-date"
        type="date"
        class="input-text font-mono"
        bind:value={endDate}
        on:change={() => (activePreset = 'custom')}
        min={startDate || minDate}
        max={maxDate}
      />
    </div>
  </div>
</div>

<style>
  .range-picker {
    display: flex;
    flex-direction: column;
    gap: 14px;
    background: #111116;
    padding: 16px 18px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .presets {
    display: flex;
    align-items: center;
    gap: 4px;
    background: #0d0d12;
    padding: 3px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .preset-btn {
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-muted);
    padding: 3px 8px;
    font-size: 11px;
    font-weight: 500;
    border-radius: calc(var(--radius-sm) - 2px);
    cursor: pointer;
    transition: all var(--transition-fast);
    white-space: nowrap;
  }

  .preset-btn:hover {
    color: var(--text-main);
    background: #191922;
  }

  .preset-btn.active {
    background: rgba(255, 230, 0, 0.16);
    color: #ffe600;
    border-color: rgba(255, 230, 0, 0.3);
    font-weight: 600;
  }

  .reset-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: rgba(239, 68, 68, 0.12);
    border: 1px solid rgba(239, 68, 68, 0.25);
    color: #f87171;
    padding: 3px 8px;
    font-size: 11px;
    border-radius: calc(var(--radius-sm) - 2px);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .reset-btn:hover {
    background: rgba(239, 68, 68, 0.22);
    color: #fca5a5;
  }

  /* Timeline Interactive Area */
  .timeline-interactive-area {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: #09090d;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 10px 12px 6px 12px;
  }

  .timeline-hint-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: nowrap;
    height: 20px;
    min-height: 20px;
    max-height: 20px;
    font-size: 11px;
    color: var(--text-muted);
    overflow: hidden;
    box-sizing: border-box;
  }

  .hint-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .hover-stat {
    font-size: 11px;
    white-space: nowrap;
    flex-shrink: 0;
    margin-left: 10px;
    text-align: right;
  }

  .stat-placeholder {
    color: var(--text-muted);
    opacity: 0.6;
  }

  .svg-canvas-container {
    width: 100%;
    height: 110px;
    cursor: crosshair;
    user-select: none;
    position: relative;
  }

  .timeline-svg {
    width: 100%;
    height: 100%;
    overflow: visible;
  }

  .selection-rect {
    cursor: grab;
  }

  .selection-rect:active {
    cursor: grabbing;
  }

  .handle {
    cursor: ew-resize;
    transition: filter var(--transition-fast);
  }

  .handle:hover {
    filter: drop-shadow(0 0 6px rgba(255, 230, 0, 0.8));
  }

  .point-circle {
    transition: r 0.15s ease, fill 0.15s ease;
  }

  .point-active {
    filter: drop-shadow(0 0 4px rgba(255, 230, 0, 0.6));
  }

  .point-hovered {
    r: 6 !important;
    fill: #38bdf8 !important;
    filter: drop-shadow(0 0 8px #38bdf8) !important;
  }

  /* Dual Inputs */
  .inputs-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .input-field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .input-field label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11.5px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .input-text {
    padding: 6px 10px;
    font-size: 12.5px;
    background: #0d0d12;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-main);
    transition: border-color var(--transition-fast);
  }

  .input-text:focus {
    border-color: rgba(255, 230, 0, 0.4);
    outline: none;
  }

  .badge-neutral {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-secondary);
  }
</style>
