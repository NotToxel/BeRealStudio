<script lang="ts">
  import type { MonthCount } from '$lib/types';
  import Calendar from 'lucide-svelte/icons/calendar';
  import Activity from 'lucide-svelte/icons/activity';
  import RotateCcw from 'lucide-svelte/icons/rotate-ccw';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import ChevronLeft from 'lucide-svelte/icons/chevron-left';
  import ChevronRight from 'lucide-svelte/icons/chevron-right';
  import ChevronDown from 'lucide-svelte/icons/chevron-down';
  import ChevronUp from 'lucide-svelte/icons/chevron-up';
  import X from 'lucide-svelte/icons/circle-x';
  import Check from 'lucide-svelte/icons/circle-check';

  export let histogram: MonthCount[] = [];
  export let minDate: string = '';
  export let maxDate: string = '';
  export let startDate: string = '';
  export let endDate: string = '';
  export let totalCount: number = 0;
  export let selectedCount: number = 0;
  export let accentColor: 'yellow' | 'purple' = 'yellow';
  export let title: string = 'Date Range & Timeline';

  $: isPurple = accentColor === 'purple';
  $: activeColor = isPurple ? '#c084fc' : '#ffe600';
  $: activeGradient0 = isPurple ? '#c084fc' : '#ffe600';
  $: activeGradient1 = isPurple ? '#a855f7' : '#f59e0b';
  $: activeGradient2 = isPurple ? '#7e22ce' : '#d97706';
  $: activeDropShadow = isPurple ? 'drop-shadow(0 0 6px rgba(192, 132, 252, 0.75))' : 'drop-shadow(0 0 6px rgba(255, 230, 0, 0.6))';
  $: activeBoxFill = isPurple ? 'rgba(168, 85, 247, 0.09)' : 'rgba(255, 230, 0, 0.07)';
  $: activeBoxStroke = isPurple ? 'rgba(168, 85, 247, 0.35)' : 'rgba(255, 230, 0, 0.2)';

  // ─── SVG Coordinate System & Dimensions ──────────────────────────────────────
  const SVG_W = 1000;
  const SVG_H = 130;
  const PAD_X = 36;
  const PAD_TOP = 20;
  const PAD_BOTTOM = 34;
  const CHART_W = SVG_W - PAD_X * 2;
  const CHART_H = SVG_H - PAD_TOP - PAD_BOTTOM;

  $: maxCount = Math.max(...histogram.map((h) => h.count), 1);
  $: normalizedMinDate = minDate ? minDate.slice(0, 10) : '';
  $: normalizedMaxDate = maxDate ? maxDate.slice(0, 10) : '';

  // Month date range helper
  function isMonthInRange(monthStr: string): boolean {
    if (!startDate && !endDate) return true;
    const mStart = `${monthStr}-01`;
    const mEnd = `${monthStr}-31`;
    if (startDate && mEnd < startDate) return false;
    if (endDate && mStart > endDate) return false;
    return true;
  }

  // Precise day-level memory count calculator that dynamically updates with slider dragging
  function computeSelectedCount(
    hist: MonthCount[],
    start: string,
    end: string,
    total: number,
    minD: string,
    maxD: string
  ): number {
    if (!start && !end) return total;
    if (minD && maxD && start <= minD && end >= maxD) return total;
    if (hist.length === 0) return total;

    const sDate = start ? new Date(start) : (minD ? new Date(minD) : new Date(`${hist[0].month}-01`));
    const eDate = end ? new Date(end) : (maxD ? new Date(maxD) : new Date());

    const sTime = sDate.getTime();
    const eTime = eDate.getTime();

    if (isNaN(sTime) || isNaN(eTime) || sTime > eTime) {
      return total;
    }

    let calculated = 0;

    for (const h of hist) {
      const parts = h.month.split('-');
      if (parts.length < 2) continue;
      const y = parseInt(parts[0], 10);
      const m = parseInt(parts[1], 10);
      const totalDays = new Date(y, m, 0).getDate();

      const mStart = new Date(y, m - 1, 1).getTime();
      const mEnd = new Date(y, m - 1, totalDays).getTime();

      // Check overlap
      if (eTime < mStart || sTime > mEnd) {
        // Month completely outside range
        continue;
      }

      if (sTime <= mStart && eTime >= mEnd) {
        // Month completely inside range
        calculated += h.count;
        continue;
      }

      // Month partially overlaps
      const overlapStart = Math.max(sTime, mStart);
      const overlapEnd = Math.min(eTime, mEnd);

      const msPerDay = 1000 * 60 * 60 * 24;
      const overlapDays = Math.max(1, Math.round((overlapEnd - overlapStart) / msPerDay) + 1);
      const fraction = Math.min(1, overlapDays / totalDays);
      calculated += Math.round(fraction * h.count);
    }

    return Math.max(0, Math.min(calculated, total));
  }

  // Reactively calculate selected entries count from histogram with continuous day-level precision
  $: selectedCount = computeSelectedCount(
    histogram,
    startDate,
    endDate,
    totalCount,
    normalizedMinDate,
    normalizedMaxDate
  );

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
  const FULL_MONTH_NAMES = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ];

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
    return `${FULL_MONTH_NAMES[mIdx]} ${yr}`;
  }

  function formatDisplayDate(dateStr?: string): string {
    if (!dateStr) return '';
    try {
      const [y, m, d] = dateStr.split('-').map((v) => parseInt(v, 10));
      if (!y || !m || !d) return dateStr;
      return `${d} ${MONTH_NAMES[m - 1]} ${y}`;
    } catch {
      return dateStr;
    }
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
      dragMode = 'new';
      dragAnchorX = x;
      const clickedDate = xToDate(x);
      startDate = clickedDate;
      endDate = clickedDate;
    }

    closePopovers();
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
      if (startDate === endDate && hoveredPoint) {
        handlePointClick(hoveredPoint);
      }
    }
  }

  function handlePointClick(p: Point) {
    startDate = `${p.month}-01`;
    const parts = p.month.split('-');
    const y = parseInt(parts[0], 10);
    const m = parseInt(parts[1], 10);
    const lastDay = new Date(y, m, 0).getDate();
    endDate = `${p.month}-${String(lastDay).padStart(2, '0')}`;
  }

  // ─── Available Years & Months Computation ───────────────────────────────────
  $: availableYears = (() => {
    const yearsSet = new Set<string>();
    for (const h of histogram) {
      const y = h.month.split('-')[0];
      if (y) yearsSet.add(y);
    }
    if (yearsSet.size === 0) {
      if (normalizedMinDate) yearsSet.add(normalizedMinDate.slice(0, 4));
      if (normalizedMaxDate) yearsSet.add(normalizedMaxDate.slice(0, 4));
    }
    return Array.from(yearsSet).sort((a, b) => b.localeCompare(a));
  })();

  $: availableMonths = histogram.map((h) => ({
    month: h.month,
    year: h.month.split('-')[0],
    shortLabel: formatMonthShort(h.month),
    fullLabel: formatMonthFull(h.month),
    count: h.count,
  }));

  function setYearRecap(year: string) {
    startDate = `${year}-01-01`;
    endDate = `${year}-12-31`;
    closePopovers();
  }

  function setMonthPreset(monthStr: string) {
    if (!monthStr) return;
    startDate = `${monthStr}-01`;
    const parts = monthStr.split('-');
    const y = parseInt(parts[0], 10);
    const m = parseInt(parts[1], 10);
    const lastDay = new Date(y, m, 0).getDate();
    endDate = `${monthStr}-${String(lastDay).padStart(2, '0')}`;
    closePopovers();
  }

  function setPreset(preset: 'all' | '30d' | '6m' | '1y') {
    if (!normalizedMaxDate && histogram.length === 0) return;
    const reference = normalizedMaxDate ? new Date(normalizedMaxDate) : new Date();

    if (preset === 'all') {
      startDate = normalizedMinDate || (histogram.length > 0 ? `${histogram[0].month}-01` : '');
      endDate = normalizedMaxDate || (histogram.length > 0 ? `${histogram[histogram.length - 1].month}-28` : '');
      closePopovers();
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
    closePopovers();
  }

  $: isFiltered = Boolean((startDate && startDate !== normalizedMinDate) || (endDate && endDate !== normalizedMaxDate));

  // Determine active preset reactively
  $: activePreset = (() => {
    if (!startDate && !endDate) return 'all';
    if (startDate === normalizedMinDate && endDate === normalizedMaxDate) return 'all';
    for (const y of availableYears) {
      if (startDate === `${y}-01-01` && (endDate === `${y}-12-31` || (normalizedMaxDate && normalizedMaxDate.startsWith(y) && endDate === normalizedMaxDate))) {
        return `year-${y}`;
      }
    }
    for (const m of availableMonths) {
      const parts = m.month.split('-');
      const y = parseInt(parts[0], 10);
      const mNum = parseInt(parts[1], 10);
      const lastDay = new Date(y, mNum, 0).getDate();
      if (startDate === `${m.month}-01` && endDate === `${m.month}-${String(lastDay).padStart(2, '0')}`) {
        return `month-${m.month}`;
      }
    }
    return 'custom';
  })();

  $: activeMonthLabel = (() => {
    if (activePreset.startsWith('month-')) {
      const mKey = activePreset.replace('month-', '');
      const found = availableMonths.find((m) => m.month === mKey);
      return found ? `${found.fullLabel} (${found.count})` : formatMonthFull(mKey);
    }
    return 'Jump to Month...';
  })();

  // ─── Custom Popovers State ──────────────────────────────────────────────────
  let showMonthDropdown = false;
  let activeCalendarPicker: 'start' | 'end' | null = null;
  let calViewYear = 2024;
  let calViewMonth = 0; // 0..11

  function closePopovers() {
    showMonthDropdown = false;
    activeCalendarPicker = null;
  }

  function openCalendar(target: 'start' | 'end') {
    showMonthDropdown = false;
    activeCalendarPicker = target;
    const initialDateStr = target === 'start' ? startDate : endDate;
    if (initialDateStr) {
      const [y, m] = initialDateStr.split('-').map((v) => parseInt(v, 10));
      calViewYear = y;
      calViewMonth = m - 1;
    } else if (normalizedMaxDate) {
      const [y, m] = normalizedMaxDate.split('-').map((v) => parseInt(v, 10));
      calViewYear = y;
      calViewMonth = m - 1;
    } else {
      const now = new Date();
      calViewYear = now.getFullYear();
      calViewMonth = now.getMonth();
    }
  }

  function nextCalMonth() {
    if (calViewMonth === 11) {
      calViewMonth = 0;
      calViewYear += 1;
    } else {
      calViewMonth += 1;
    }
  }

  function prevCalMonth() {
    if (calViewMonth === 0) {
      calViewMonth = 11;
      calViewYear -= 1;
    } else {
      calViewMonth -= 1;
    }
  }

  // Calendar matrix calculation
  interface CalCell {
    dayNum: number;
    dateStr: string;
    isCurrentMonth: boolean;
    isInRange: boolean;
    isStart: boolean;
    isEnd: boolean;
    isDisabled: boolean;
  }

  $: calDays = (() => {
    const days: CalCell[] = [];
    const firstDayIndex = (new Date(calViewYear, calViewMonth, 1).getDay() + 6) % 7; // Monday = 0
    const daysInCurMonth = new Date(calViewYear, calViewMonth + 1, 0).getDate();
    const daysInPrevMonth = new Date(calViewYear, calViewMonth, 0).getDate();

    // Previous month padding
    for (let i = firstDayIndex - 1; i >= 0; i--) {
      const dayNum = daysInPrevMonth - i;
      const prevM = calViewMonth === 0 ? 12 : calViewMonth;
      const prevY = calViewMonth === 0 ? calViewYear - 1 : calViewYear;
      const dateStr = `${prevY}-${String(prevM).padStart(2, '0')}-${String(dayNum).padStart(2, '0')}`;
      days.push({
        dayNum,
        dateStr,
        isCurrentMonth: false,
        isInRange: Boolean(startDate && endDate && dateStr >= startDate && dateStr <= endDate),
        isStart: dateStr === startDate,
        isEnd: dateStr === endDate,
        isDisabled: Boolean((normalizedMinDate && dateStr < normalizedMinDate) || (normalizedMaxDate && dateStr > normalizedMaxDate)),
      });
    }

    // Current month days
    for (let d = 1; d <= daysInCurMonth; d++) {
      const dateStr = `${calViewYear}-${String(calViewMonth + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
      days.push({
        dayNum: d,
        dateStr,
        isCurrentMonth: true,
        isInRange: Boolean(startDate && endDate && dateStr >= startDate && dateStr <= endDate),
        isStart: dateStr === startDate,
        isEnd: dateStr === endDate,
        isDisabled: Boolean((normalizedMinDate && dateStr < normalizedMinDate) || (normalizedMaxDate && dateStr > normalizedMaxDate)),
      });
    }

    // Next month padding to fill grid
    const remaining = (7 - (days.length % 7)) % 7;
    for (let n = 1; n <= remaining; n++) {
      const nextM = calViewMonth === 11 ? 1 : calViewMonth + 2;
      const nextY = calViewMonth === 11 ? calViewYear + 1 : calViewYear;
      const dateStr = `${nextY}-${String(nextM).padStart(2, '0')}-${String(n).padStart(2, '0')}`;
      days.push({
        dayNum: n,
        dateStr,
        isCurrentMonth: false,
        isInRange: Boolean(startDate && endDate && dateStr >= startDate && dateStr <= endDate),
        isStart: dateStr === startDate,
        isEnd: dateStr === endDate,
        isDisabled: Boolean((normalizedMinDate && dateStr < normalizedMinDate) || (normalizedMaxDate && dateStr > normalizedMaxDate)),
      });
    }

    return days;
  })();

  function handleSelectCalDay(cell: CalCell) {
    if (cell.isDisabled) return;
    if (activeCalendarPicker === 'start') {
      startDate = cell.dateStr;
      if (endDate && endDate < startDate) {
        endDate = startDate;
      }
      activeCalendarPicker = 'end'; // seamlessly guide user to select end date
    } else if (activeCalendarPicker === 'end') {
      if (startDate && cell.dateStr < startDate) {
        endDate = startDate;
        startDate = cell.dateStr;
      } else {
        endDate = cell.dateStr;
      }
      activeCalendarPicker = null; // complete selection
    }
  }

  // Click outside helper
  function handleDocumentClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.popover-container') && !target.closest('.custom-picker-trigger') && !target.closest('.month-select-wrapper')) {
      closePopovers();
    }
  }
</script>

<svelte:window
  on:mousemove={handleWindowMouseMove}
  on:mouseup={handleWindowMouseUp}
  on:click={handleDocumentClick}
/>

<div class="range-picker card" class:theme-purple={isPurple}>
  <!-- Header Bar -->
  <div class="header">
    <div class="title-group">
      <Activity size={16} class={isPurple ? 'text-purple-400' : 'text-amber-400'} />
      <span class="title-sm font-semibold">{title}</span>
      {#if totalCount > 0}
        <span class="badge {isFiltered ? (isPurple ? 'badge-purple' : 'badge-yellow') : 'badge-neutral'} font-mono">
          <Sparkles size={11} />
          {selectedCount} of {totalCount} Memories ({Math.round((selectedCount / totalCount) * 100 || 100)}%)
        </span>
      {/if}
    </div>

    <!-- Quick Presets Pills (Relative & Year Recaps) -->
    <div class="presets-row-wrap">
      <div class="presets">
        <button
          type="button"
          class="preset-btn"
          class:active={!isFiltered || activePreset === 'all'}
          on:click={() => setPreset('all')}
        >
          All Time
        </button>

        {#each availableYears as yr}
          <button
            type="button"
            class="preset-btn preset-recap"
            class:active={activePreset === `year-${yr}`}
            on:click={() => setYearRecap(yr)}
            title="Lock date range to entire {yr} calendar year"
          >
            <Sparkles size={10} />
            <span>{yr} Recap</span>
          </button>
        {/each}

        <button
          type="button"
          class="preset-btn"
          class:active={activePreset === '30d'}
          on:click={() => setPreset('30d')}
        >
          Past 30D
        </button>
        <button
          type="button"
          class="preset-btn"
          class:active={activePreset === '6m'}
          on:click={() => setPreset('6m')}
        >
          Past 6M
        </button>
        <button
          type="button"
          class="preset-btn"
          class:active={activePreset === '1y'}
          on:click={() => setPreset('1y')}
        >
          Past 1Y
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

      <!-- Custom Styled Month Quick Selector Popover -->
      {#if availableMonths.length > 0}
        <div class="month-selector-group">
          <div class="month-select-wrapper">
            <button
              type="button"
              class="month-select-trigger font-mono"
              class:active={showMonthDropdown || activePreset.startsWith('month-')}
              on:click|stopPropagation={() => {
                showMonthDropdown = !showMonthDropdown;
                activeCalendarPicker = null;
              }}
              title="Quick select specific archive month"
            >
              <Calendar size={12} class="text-sky-400" />
              <span class="truncate">{activeMonthLabel}</span>
              {#if showMonthDropdown}
                <ChevronUp size={12} class="text-muted" />
              {:else}
                <ChevronDown size={12} class="text-muted" />
              {/if}
            </button>

            {#if showMonthDropdown}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="custom-dropdown-popover popover-container" on:click|stopPropagation>
                <div class="dropdown-popover-header">
                  <span class="font-bold text-white text-xs">Available Months</span>
                  <span class="badge badge-neutral text-xs font-mono">{availableMonths.length} Months</span>
                </div>
                <div class="dropdown-popover-list">
                  {#each availableMonths as m}
                    <button
                      type="button"
                      class="dropdown-popover-item font-mono"
                      class:active={activePreset === `month-${m.month}`}
                      on:click={() => setMonthPreset(m.month)}
                    >
                      <span class="m-full-label">{m.fullLabel}</span>
                      <span class="m-count-pill">{m.count} memories</span>
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <!-- Quick Month Pills (Latest 3 available months) -->
          <div class="quick-month-pills">
            {#each availableMonths.slice(-3).reverse() as m}
              <button
                type="button"
                class="month-pill-btn"
                class:active={activePreset === `month-${m.month}`}
                on:click={() => setMonthPreset(m.month)}
                title="Select {m.fullLabel} ({m.count} memories)"
              >
                {m.shortLabel}
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Continuous Smooth Curve Timeline -->
  {#if histogram.length > 0}
    <div class="timeline-interactive-area">
      <!-- Concise Hover Status & Guidance Bar -->
      <div class="timeline-hint-row">
        <span class="hint-text">
          Drag timeline across to select range • Drag handles to adjust
        </span>
        <span class="hover-stat font-mono">
          {#if hoveredPoint}
            <span class="text-amber-300">{formatMonthFull(hoveredPoint.month)}: <strong>{hoveredPoint.count} Memories</strong></span>
          {:else}
            <span class="stat-placeholder">Hover timeline for details</span>
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
            <linearGradient id="area-grad-dimmed" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color={isPurple ? '#c084fc' : '#38bdf8'} stop-opacity="0.12" />
              <stop offset="100%" stop-color={isPurple ? '#c084fc' : '#38bdf8'} stop-opacity="0.0" />
            </linearGradient>

            <linearGradient id="area-grad-active-{accentColor}" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color={activeGradient0} stop-opacity={isPurple ? '0.45' : '0.38'} />
              <stop offset="60%" stop-color={activeGradient1} stop-opacity={isPurple ? '0.2' : '0.15'} />
              <stop offset="100%" stop-color={activeGradient2} stop-opacity="0.0" />
            </linearGradient>

            <clipPath id="selection-clip-{accentColor}">
              <rect x={boxLeft} y="0" width={boxWidth} height={SVG_H} />
            </clipPath>
          </defs>

          <!-- 1. Grid Horizontal Baseline -->
          <line
            x1={PAD_X}
            y1={SVG_H - PAD_BOTTOM}
            x2={SVG_W - PAD_X}
            y2={SVG_H - PAD_BOTTOM}
            stroke="rgba(255, 255, 255, 0.12)"
            stroke-width="1"
          />

          <!-- 2. Base Dimmed Continuous Curve -->
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

          <!-- 3. Active Glowing In-Range Curve -->
          <g clip-path="url(#selection-clip-{accentColor})">
            <path
              d={curvePaths.areaStr}
              fill="url(#area-grad-active-{accentColor})"
            />
            <path
              d={curvePaths.pathStr}
              fill="none"
              stroke={activeColor}
              stroke-width="2.5"
              stroke-linecap="round"
              style="filter: {activeDropShadow};"
            />
          </g>

          <!-- 4. Interactive Selection Box -->
          <g class="selection-group">
            <rect
              x={boxLeft}
              y={PAD_TOP - 6}
              width={boxWidth}
              height={CHART_H + 12}
              fill={activeBoxFill}
              stroke={activeBoxStroke}
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
                stroke={activeColor}
                stroke-width="2"
                stroke-linecap="round"
              />
              <rect
                x="-5"
                y={(PAD_TOP + SVG_H - PAD_BOTTOM) / 2 - 12}
                width="10"
                height="24"
                rx="4"
                fill={activeColor}
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

            <!-- Right Handle -->
            <g class="handle right-handle" transform="translate({boxRight}, 0)">
              <line
                x1="0"
                y1={PAD_TOP - 8}
                x2="0"
                y2={SVG_H - PAD_BOTTOM + 2}
                stroke={activeColor}
                stroke-width="2"
                stroke-linecap="round"
              />
              <rect
                x="-5"
                y={(PAD_TOP + SVG_H - PAD_BOTTOM) / 2 - 12}
                width="10"
                height="24"
                rx="4"
                fill={activeColor}
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

            <line
              x1={p.x}
              y1={SVG_H - PAD_BOTTOM}
              x2={p.x}
              y2={SVG_H - PAD_BOTTOM + (p.isFirstOfYear ? 6 : 3)}
              stroke={p.isFirstOfYear ? 'rgba(255,255,255,0.4)' : 'rgba(255,255,255,0.15)'}
              stroke-width={p.isFirstOfYear ? 1.5 : 1}
            />

            <circle
              cx={p.x}
              cy={p.y}
              r={inRange ? 3.5 : 2}
              fill={inRange ? activeColor : 'rgba(255, 255, 255, 0.3)'}
              stroke="#0a0a0e"
              stroke-width="1.5"
              class="point-circle"
              class:point-active={inRange}
              class:point-hovered={hoveredPoint?.month === p.month}
            />

            {#if showLabel}
              <text
                x={p.x}
                y={SVG_H - 12}
                text-anchor="middle"
                font-size="10"
                font-family="monospace"
                fill={p.isFirstOfYear ? '#ffffff' : inRange ? activeColor : 'rgba(255, 255, 255, 0.45)'}
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
              stroke={isPurple ? 'rgba(192, 132, 252, 0.5)' : 'rgba(255, 230, 0, 0.5)'}
              stroke-dasharray="3,3"
              stroke-width="1"
            />
          {/if}
        </svg>
      </div>
    </div>
  {/if}

  <!-- Dual Interactive Native-Feeling Calendar Inputs -->
  <div class="calendar-inputs-row">
    <!-- From Date Trigger Card -->
    <div class="cal-input-container">
      <span class="input-label-row">
        <Calendar size={13} class={isPurple ? 'text-purple-400' : 'text-amber-400'} />
        <span>From Date (Start)</span>
      </span>
      <button
        type="button"
        class="custom-picker-trigger font-mono"
        class:active={activeCalendarPicker === 'start'}
        on:click|stopPropagation={() => openCalendar('start')}
      >
        <div class="trigger-left">
          <span class="cal-badge">FROM</span>
          <span class="trigger-date-text">{startDate ? formatDisplayDate(startDate) : 'Choose start date...'}</span>
        </div>
        <div class="trigger-right">
          <Calendar size={14} class={isPurple ? 'text-purple-400' : 'text-amber-400'} />
        </div>
      </button>
    </div>

    <!-- To Date Trigger Card -->
    <div class="cal-input-container">
      <span class="input-label-row">
        <Calendar size={13} class={isPurple ? 'text-purple-400' : 'text-amber-400'} />
        <span>To Date (End)</span>
      </span>
      <button
        type="button"
        class="custom-picker-trigger font-mono"
        class:active={activeCalendarPicker === 'end'}
        on:click|stopPropagation={() => openCalendar('end')}
      >
        <div class="trigger-left">
          <span class="cal-badge cal-badge-to">TO</span>
          <span class="trigger-date-text">{endDate ? formatDisplayDate(endDate) : 'Choose end date...'}</span>
        </div>
        <div class="trigger-right">
          <Calendar size={14} class={isPurple ? 'text-purple-400' : 'text-amber-400'} />
        </div>
      </button>
    </div>

    <!-- Floating Native-Feeling Calendar Matrix Popover -->
    {#if activeCalendarPicker !== null}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="calendar-matrix-popover popover-container" on:click|stopPropagation>
        <!-- Popover Top Mode Tabs & Close -->
        <div class="cal-popover-top-bar">
          <div class="cal-mode-tabs font-mono">
            <button
              type="button"
              class="cal-mode-tab"
              class:active={activeCalendarPicker === 'start'}
              on:click={() => (activeCalendarPicker = 'start')}
            >
              <span class="cal-tab-dot dot-from"></span>
              <span>Start Date</span>
            </button>
            <button
              type="button"
              class="cal-mode-tab"
              class:active={activeCalendarPicker === 'end'}
              on:click={() => (activeCalendarPicker = 'end')}
            >
              <span class="cal-tab-dot dot-to"></span>
              <span>End Date</span>
            </button>
          </div>

          <button
            type="button"
            class="cal-close-btn"
            on:click={closePopovers}
            title="Close Calendar"
          >
            <X size={14} />
          </button>
        </div>

        <!-- Month Navigation Bar -->
        <div class="cal-month-nav-row">
          <button type="button" class="cal-nav-btn" on:click={prevCalMonth} title="Previous Month">
            <ChevronLeft size={16} />
          </button>
          <span class="cal-month-heading font-mono font-bold">
            {FULL_MONTH_NAMES[calViewMonth]} {calViewYear}
          </span>
          <button type="button" class="cal-nav-btn" on:click={nextCalMonth} title="Next Month">
            <ChevronRight size={16} />
          </button>
        </div>

        <!-- Days of Week Header -->
        <div class="cal-weekdays-row font-mono">
          <span>Mo</span>
          <span>Tu</span>
          <span>We</span>
          <span>Th</span>
          <span>Fr</span>
          <span>Sa</span>
          <span>Su</span>
        </div>

        <!-- Calendar Days Grid -->
        <div class="cal-grid">
          {#each calDays as cell}
            <button
              type="button"
              class="cal-day-cell font-mono"
              class:other-month={!cell.isCurrentMonth}
              class:in-range={cell.isInRange}
              class:is-start={cell.isStart}
              class:is-end={cell.isEnd}
              class:is-disabled={cell.isDisabled}
              disabled={cell.isDisabled}
              on:click={() => handleSelectCalDay(cell)}
              title={cell.dateStr}
            >
              <span>{cell.dayNum}</span>
            </button>
          {/each}
        </div>

        <!-- Popover Quick Action Footer -->
        <div class="cal-popover-footer">
          <div class="cal-footer-info font-mono text-xs text-muted">
            {#if startDate && endDate}
              <span>{formatDisplayDate(startDate)} – {formatDisplayDate(endDate)}</span>
            {:else if startDate}
              <span>From {formatDisplayDate(startDate)}</span>
            {/if}
          </div>

          <div class="cal-footer-actions">
            <button
              type="button"
              class="btn-cal-action"
              on:click={() => setPreset('all')}
            >
              Reset All
            </button>
            <button
              type="button"
              class="btn-cal-action btn-cal-action-done"
              on:click={closePopovers}
            >
              <Check size={12} />
              <span>Done</span>
            </button>
          </div>
        </div>
      </div>
    {/if}
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
    position: relative;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
    row-gap: 10px;
    width: 100%;
  }

  .title-group {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
  }

  .presets-row-wrap {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    max-width: 100%;
  }

  .presets {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 3px;
    background: #0d0d12;
    padding: 3px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    max-width: 100%;
  }

  .preset-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
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

  .preset-btn.preset-recap {
    color: #fde047;
  }

  .preset-btn.preset-recap.active {
    background: rgba(255, 230, 0, 0.22);
    border-color: rgba(255, 230, 0, 0.45);
    box-shadow: 0 0 10px rgba(255, 230, 0, 0.15);
  }

  /* ── Custom Styled Month Selector & Popover ── */
  .month-selector-group {
    display: flex;
    align-items: center;
    gap: 6px;
    background: #0d0d12;
    padding: 3px 5px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    position: relative;
  }

  .month-select-wrapper {
    position: relative;
  }

  .month-select-trigger {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: #15151c;
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #38bdf8;
    padding: 3px 10px;
    font-size: 11px;
    font-weight: 600;
    border-radius: var(--radius-sm);
    cursor: pointer;
    outline: none;
    transition: all var(--transition-fast);
  }

  .month-select-trigger:hover,
  .month-select-trigger.active {
    border-color: rgba(56, 189, 248, 0.45);
    background: #1c1c26;
  }

  .custom-dropdown-popover {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    width: 240px;
    background: #111116;
    border: 1px solid rgba(56, 189, 248, 0.3);
    border-radius: var(--radius-md);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.8), 0 0 15px rgba(56, 189, 248, 0.1);
    z-index: 100;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .dropdown-popover-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 10px;
    background: #0d0d12;
    border-bottom: 1px solid var(--border-subtle);
  }

  .dropdown-popover-list {
    max-height: 220px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding: 4px;
    gap: 2px;
  }

  .dropdown-popover-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: transparent;
    border: none;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 11px;
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
  }

  .dropdown-popover-item:hover {
    background: #1a1a24;
    color: #ffffff;
  }

  .dropdown-popover-item.active {
    background: rgba(56, 189, 248, 0.18);
    color: #38bdf8;
    font-weight: 700;
  }

  .m-count-pill {
    font-size: 10px;
    color: var(--text-muted);
    background: rgba(255, 255, 255, 0.05);
    padding: 1px 5px;
    border-radius: 3px;
  }

  .quick-month-pills {
    display: flex;
    align-items: center;
    gap: 3px;
  }

  .month-pill-btn {
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-secondary);
    padding: 2px 6px;
    font-size: 10.5px;
    font-family: var(--font-mono);
    font-weight: 500;
    border-radius: calc(var(--radius-sm) - 2px);
    cursor: pointer;
    transition: all var(--transition-fast);
    white-space: nowrap;
  }

  .month-pill-btn:hover {
    color: #38bdf8;
    background: rgba(56, 189, 248, 0.1);
  }

  .month-pill-btn.active {
    background: rgba(56, 189, 248, 0.18);
    color: #38bdf8;
    border-color: rgba(56, 189, 248, 0.35);
    font-weight: 700;
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
    height: 18px;
    font-size: 11px;
    color: var(--text-muted);
    overflow: hidden;
  }

  .hint-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .hover-stat {
    font-size: 11px;
    white-space: nowrap;
    flex-shrink: 0;
    margin-left: 10px;
  }

  .svg-canvas-container {
    position: relative;
    width: 100%;
    height: 130px;
    cursor: crosshair;
    user-select: none;
  }

  .timeline-svg {
    width: 100%;
    height: 100%;
    display: block;
    overflow: visible;
  }

  /* ── Native-like Dual Calendar Cards & Matrix Popover ── */
  .calendar-inputs-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    position: relative;
  }

  @media (max-width: 640px) {
    .calendar-inputs-row {
      grid-template-columns: 1fr;
    }
  }

  .cal-input-container {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .input-label-row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11.5px;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .custom-picker-trigger {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #0a0a0f;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: var(--radius-sm);
    padding: 8px 12px;
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
    width: 100%;
    color: #ffffff;
  }

  .custom-picker-trigger:hover {
    border-color: rgba(255, 230, 0, 0.4);
    background: #121218;
  }

  .custom-picker-trigger.active {
    border-color: #ffe600;
    box-shadow: 0 0 12px rgba(255, 230, 0, 0.2);
    background: #14141c;
  }

  .trigger-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .cal-badge {
    font-size: 9.5px;
    font-weight: 700;
    background: rgba(255, 230, 0, 0.18);
    color: #ffe600;
    padding: 1px 5px;
    border-radius: 3px;
    border: 1px solid rgba(255, 230, 0, 0.35);
  }

  .cal-badge.cal-badge-to {
    background: rgba(56, 189, 248, 0.18);
    color: #38bdf8;
    border-color: rgba(56, 189, 248, 0.35);
  }

  .trigger-date-text {
    font-size: 13px;
    font-weight: 600;
    letter-spacing: -0.2px;
  }

  /* ── Floating Native Calendar Modal / Popover ── */
  .calendar-matrix-popover {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    width: 320px;
    background: #111116;
    border: 1px solid rgba(255, 230, 0, 0.35);
    border-radius: var(--radius-md);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.85), 0 0 20px rgba(255, 230, 0, 0.12);
    z-index: 120;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .cal-popover-top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 8px;
  }

  .cal-mode-tabs {
    display: inline-flex;
    align-items: center;
    background: #09090d;
    padding: 2px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    gap: 2px;
  }

  .cal-mode-tab {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 500;
    padding: 3px 8px;
    border-radius: calc(var(--radius-sm) - 2px);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .cal-mode-tab:hover {
    color: var(--text-main);
  }

  .cal-mode-tab.active {
    background: #1a1a24;
    color: #ffffff;
    font-weight: 600;
    border-color: rgba(255, 255, 255, 0.1);
  }

  .cal-tab-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .dot-from {
    background: #ffe600;
  }

  .dot-to {
    background: #38bdf8;
  }

  .cal-month-nav-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 4px;
  }

  .cal-nav-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .cal-nav-btn:hover {
    background: #20202c;
    color: #ffffff;
  }

  .cal-month-heading {
    font-size: 12.5px;
    color: #ffffff;
    min-width: 110px;
    text-align: center;
  }

  .cal-close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 4px;
    border-radius: 3px;
  }

  .cal-close-btn:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.1);
  }

  .cal-weekdays-row {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    text-align: center;
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
  }

  .cal-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 2px;
  }

  .cal-day-cell {
    aspect-ratio: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    color: #ffffff;
    font-size: 11.5px;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-fast);
    padding: 0;
  }

  .cal-day-cell:hover:not(:disabled) {
    background: #252533;
    border-color: rgba(255, 230, 0, 0.4);
  }

  .cal-day-cell.other-month {
    color: rgba(255, 255, 255, 0.2);
  }

  .cal-day-cell.in-range {
    background: rgba(255, 230, 0, 0.14);
    color: #ffe600;
    border-radius: 0;
  }

  .cal-day-cell.is-start {
    background: #ffe600 !important;
    color: #0a0a0f !important;
    font-weight: 700;
    border-radius: 4px 0 0 4px;
    box-shadow: 0 0 8px rgba(255, 230, 0, 0.5);
  }

  .cal-day-cell.is-end {
    background: #ffe600 !important;
    color: #0a0a0f !important;
    font-weight: 700;
    border-radius: 0 4px 4px 0;
    box-shadow: 0 0 8px rgba(255, 230, 0, 0.5);
  }

  .cal-day-cell.is-start.is-end {
    border-radius: 4px !important;
  }

  .cal-day-cell.is-disabled {
    opacity: 0.2;
    cursor: not-allowed;
  }

  .cal-popover-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 1px solid var(--border-subtle);
    padding-top: 8px;
    margin-top: 2px;
  }

  .cal-footer-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .btn-cal-action {
    background: transparent;
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    padding: 3px 8px;
    font-size: 11px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .btn-cal-action:hover {
    background: #1f1f2a;
    color: #ffffff;
  }

  .btn-cal-action-done {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: #ffe600;
    color: #0a0a0f;
    border-color: #ffe600;
    font-weight: 700;
  }

  .btn-cal-action-done:hover {
    background: #e6cf00;
    color: #000000;
  }

  /* ── Purple Visual Identity Theme Overrides ── */
  .theme-purple .preset-btn.active {
    background: rgba(168, 85, 247, 0.2);
    color: #c084fc;
    border-color: rgba(168, 85, 247, 0.4);
  }

  .theme-purple .preset-btn.preset-recap {
    color: #d8b4fe;
  }

  .theme-purple .preset-btn.preset-recap.active {
    background: rgba(168, 85, 247, 0.28);
    border-color: rgba(168, 85, 247, 0.55);
    box-shadow: 0 0 10px rgba(168, 85, 247, 0.28);
  }

  .theme-purple .cal-badge {
    background: rgba(168, 85, 247, 0.2);
    color: #c084fc;
    border-color: rgba(168, 85, 247, 0.4);
  }

  .theme-purple .custom-picker-trigger:hover {
    border-color: rgba(168, 85, 247, 0.45);
  }

  .theme-purple .custom-picker-trigger.active {
    border-color: #c084fc;
    box-shadow: 0 0 12px rgba(168, 85, 247, 0.25);
  }

  .theme-purple .calendar-matrix-popover {
    border-color: rgba(168, 85, 247, 0.45);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.85), 0 0 20px rgba(168, 85, 247, 0.18);
  }

  .theme-purple .dot-from {
    background: #c084fc;
  }

  .theme-purple .cal-day-cell:hover:not(:disabled) {
    border-color: rgba(168, 85, 247, 0.5);
  }

  .theme-purple .cal-day-cell.in-range {
    background: rgba(168, 85, 247, 0.18);
    color: #c084fc;
  }

  .theme-purple .cal-day-cell.is-start,
  .theme-purple .cal-day-cell.is-end {
    background: #c084fc !important;
    color: #0a0a0f !important;
    box-shadow: 0 0 8px rgba(192, 132, 252, 0.65);
  }

  .theme-purple .btn-cal-action-done {
    background: #c084fc;
    color: #0a0a0f;
    border-color: #c084fc;
  }

  .theme-purple .btn-cal-action-done:hover {
    background: #a855f7;
    color: #000000;
  }
</style>
