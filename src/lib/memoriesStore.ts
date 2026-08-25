import { writable, derived, get } from 'svelte/store';
import type { ExplorerData, ExplorerMemory, ExplorerFilterState } from './types';
import { loadExplorerMemories, readMediaFileDataUrl, convertFileSrc, isTauri } from './tauri';
import { lastScannedArchivePath, currentArchive } from './stores';

export const initialFilterState: ExplorerFilterState = {
  searchQuery: '',
  selectedYear: 'all',
  selectedMonth: 'all',
  selectedCountry: 'all',
  selectedCity: 'all',
  selectedSuburb: 'all',
  hasLocationOnly: false,
  hasBtsOnly: false,
  hasCaptionOnly: false,
  retakesOnly: false,
};

// ─── Core Stores ──────────────────────────────────────────────────────────────

export const explorerData = writable<ExplorerData | null>(null);
export const isLoadingMemories = writable<boolean>(false);
export const memoriesLoadError = writable<string | null>(null);

export const activeExplorerView = writable<'grid' | 'calendar'>('grid');
export const activeFeedMemory = writable<ExplorerMemory | null>(null);
export const activeFeedIndex = writable<number | null>(null);

export const explorerFilter = writable<ExplorerFilterState>({ ...initialFilterState });

// Current month viewed in the Calendar view ("YYYY-MM")
export const calendarCurrentMonth = writable<string>('');

// ─── Derived Location & Date Counts Store ─────────────────────────────────────
export interface ExplorerFilterCounts {
  byYear: Map<number, number>;
  byMonth: Map<string, number>;
  byCountry: Map<string, number>;
  byCity: Map<string, number>;
  bySuburb: Map<string, number>;
}

export const explorerFilterCounts = derived(explorerData, ($data): ExplorerFilterCounts => {
  const counts: ExplorerFilterCounts = {
    byYear: new Map(),
    byMonth: new Map(),
    byCountry: new Map(),
    byCity: new Map(),
    bySuburb: new Map(),
  };

  if (!$data || !$data.memories) return counts;

  for (const m of $data.memories) {
    counts.byYear.set(m.year, (counts.byYear.get(m.year) || 0) + 1);
    counts.byMonth.set(m.monthKey, (counts.byMonth.get(m.monthKey) || 0) + 1);
    if (m.country) counts.byCountry.set(m.country, (counts.byCountry.get(m.country) || 0) + 1);
    if (m.city) counts.byCity.set(m.city, (counts.byCity.get(m.city) || 0) + 1);
    if (m.suburb) counts.bySuburb.set(m.suburb, (counts.bySuburb.get(m.suburb) || 0) + 1);
  }

  return counts;
});

// ─── Derived Filtered Memories Store ──────────────────────────────────────────

export const filteredMemories = derived<[typeof explorerData, typeof explorerFilter], ExplorerMemory[]>(
  [explorerData, explorerFilter],
  ([$data, $filter]) => {
    if (!$data || !$data.memories) return [];

    return $data.memories.filter((m: ExplorerMemory) => {
      // 1. Text search (caption, location, suburb, city, country, date)
      if ($filter.searchQuery.trim()) {
        const q = $filter.searchQuery.toLowerCase().trim();
        const captionMatch = m.caption ? m.caption.toLowerCase().includes(q) : false;
        const locationMatch =
          (m.locationName ? m.locationName.toLowerCase().includes(q) : false) ||
          (m.suburb ? m.suburb.toLowerCase().includes(q) : false) ||
          (m.city ? m.city.toLowerCase().includes(q) : false) ||
          (m.country ? m.country.toLowerCase().includes(q) : false);
        const dateMatch =
          m.dateFormatted.toLowerCase().includes(q) ||
          m.takenAt.toLowerCase().includes(q) ||
          m.monthKey.includes(q);
        if (!captionMatch && !locationMatch && !dateMatch) return false;
      }

      // 2. Year filter
      if ($filter.selectedYear !== 'all' && m.year !== $filter.selectedYear) {
        return false;
      }

      // 3. Month filter
      if ($filter.selectedMonth !== 'all' && m.monthKey !== $filter.selectedMonth) {
        return false;
      }

      // 4. Country filter
      if ($filter.selectedCountry !== 'all' && m.country !== $filter.selectedCountry) {
        return false;
      }

      // 5. City filter
      if ($filter.selectedCity !== 'all' && m.city !== $filter.selectedCity) {
        return false;
      }

      // 6. Suburb filter
      if ($filter.selectedSuburb !== 'all' && m.suburb !== $filter.selectedSuburb) {
        return false;
      }

      // 7. Flags
      if ($filter.hasLocationOnly && !m.location && !m.locationName) return false;
      if ($filter.hasBtsOnly && !m.btsPath) return false;
      if ($filter.hasCaptionOnly && (!m.caption || m.caption.trim().length === 0)) return false;
      if ($filter.retakesOnly && m.retakeCounter === 0) return false;

      return true;
    });
  }
);

// Count of currently active filters for clear UI indication
export const activeFilterCount = derived(explorerFilter, ($f) => {
  let count = 0;
  if ($f.searchQuery.trim()) count++;
  if ($f.selectedYear !== 'all') count++;
  if ($f.selectedMonth !== 'all') count++;
  if ($f.selectedCountry !== 'all') count++;
  if ($f.selectedCity !== 'all') count++;
  if ($f.selectedSuburb !== 'all') count++;
  if ($f.hasLocationOnly) count++;
  if ($f.hasBtsOnly) count++;
  if ($f.hasCaptionOnly) count++;
  if ($f.retakesOnly) count++;
  return count;
});

// Group filtered memories by month for fast month jumping
export const memoriesByMonth = derived(filteredMemories, ($memories) => {
  const map = new Map<string, ExplorerMemory[]>();
  for (const m of $memories) {
    if (!map.has(m.monthKey)) {
      map.set(m.monthKey, []);
    }
    map.get(m.monthKey)!.push(m);
  }
  return map;
});

// Group memories by date string ("YYYY-MM-DD") for the Calendar day cells
export const memoriesByDate = derived(filteredMemories, ($memories) => {
  const map = new Map<string, ExplorerMemory[]>();
  for (const m of $memories) {
    const dateStr = `${m.year}-${String(m.month).padStart(2, '0')}-${String(m.day).padStart(2, '0')}`;
    if (!map.has(dateStr)) {
      map.set(dateStr, []);
    }
    map.get(dateStr)!.push(m);
  }
  return map;
});

// ─── Actions & Helpers ────────────────────────────────────────────────────────

/**
 * Load memories from the given path (or fallback to last scanned path).
 */
export async function loadMemories(path?: string): Promise<boolean> {
  const targetPath = path || get(lastScannedArchivePath);
  if (!targetPath) {
    memoriesLoadError.set('No BeReal archive or folder selected.');
    return false;
  }

  isLoadingMemories.set(true);
  memoriesLoadError.set(null);

  try {
    const data = await loadExplorerMemories(targetPath);
    explorerData.set(data);

    // Default calendar month to latest month in dataset
    if (data.uniqueMonths.length > 0) {
      calendarCurrentMonth.set(data.uniqueMonths[data.uniqueMonths.length - 1]);
    }

    isLoadingMemories.set(false);
    return true;
  } catch (err: any) {
    console.error('Failed to load explorer memories:', err);
    memoriesLoadError.set(err?.message || String(err));
    isLoadingMemories.set(false);
    return false;
  }
}

const mediaDataUrlCache = new Map<string, string>();

/**
 * Resolve a local disk image path to a safe browser/Tauri file asset URL.
 */
export function getSafeImageSrc(filePath?: string): string {
  if (!filePath) return '';
  if (filePath.startsWith('http://') || filePath.startsWith('https://') || filePath.startsWith('data:')) {
    return filePath;
  }
  if (mediaDataUrlCache.has(filePath)) {
    return mediaDataUrlCache.get(filePath)!;
  }
  if (isTauri()) {
    return convertFileSrc(filePath);
  }
  return filePath;
}

/**
 * Fetch Data URL for a local media file with caching.
 */
export async function getMediaDataUrl(filePath?: string): Promise<string> {
  if (!filePath) return '';
  if (filePath.startsWith('http://') || filePath.startsWith('https://') || filePath.startsWith('data:')) {
    return filePath;
  }
  if (mediaDataUrlCache.has(filePath)) {
    return mediaDataUrlCache.get(filePath)!;
  }

  if (isTauri()) {
    try {
      const dataUrl = await readMediaFileDataUrl(filePath);
      if (dataUrl) {
        mediaDataUrlCache.set(filePath, dataUrl);
        return dataUrl;
      }
    } catch (e) {
      console.warn('Failed to read data URL for:', filePath, e);
    }
  }

  return getSafeImageSrc(filePath);
}

/**
 * Open full Feed view focused on a specific memory index.
 */
export function openFeedAt(memory: ExplorerMemory) {
  activeFeedMemory.set(memory);
  activeFeedIndex.set(memory.index);
}

export function closeFeed() {
  activeFeedMemory.set(null);
  activeFeedIndex.set(null);
}

export function nextFeedMemory() {
  const current = get(activeFeedMemory);
  const data = get(filteredMemories);
  if (!current || data.length === 0) return;

  const currentIdx = data.findIndex((m) => m.id === current.id);
  if (currentIdx !== -1 && currentIdx < data.length - 1) {
    activeFeedMemory.set(data[currentIdx + 1]);
    activeFeedIndex.set(data[currentIdx + 1].index);
  }
}

export function prevFeedMemory() {
  const current = get(activeFeedMemory);
  const data = get(filteredMemories);
  if (!current || data.length === 0) return;

  const currentIdx = data.findIndex((m) => m.id === current.id);
  if (currentIdx > 0) {
    activeFeedMemory.set(data[currentIdx - 1]);
    activeFeedIndex.set(data[currentIdx - 1].index);
  }
}

export function resetFilters() {
  explorerFilter.set({ ...initialFilterState });
}

// ─── Custom Context Menu State ────────────────────────────────────────────────
export interface ContextMenuState {
  isOpen: boolean;
  x: number;
  y: number;
  memory: ExplorerMemory | null;
}

export const contextMenuState = writable<ContextMenuState>({
  isOpen: false,
  x: 0,
  y: 0,
  memory: null,
});

export function openContextMenu(e: MouseEvent, memory: ExplorerMemory) {
  e.preventDefault();
  e.stopPropagation();

  // Clamp coordinates to viewport dimensions
  const menuWidth = 230;
  const menuHeight = 280;
  const x = Math.min(e.clientX, window.innerWidth - menuWidth - 10);
  const y = Math.min(e.clientY, window.innerHeight - menuHeight - 10);

  contextMenuState.set({
    isOpen: true,
    x: Math.max(10, x),
    y: Math.max(10, y),
    memory,
  });
}

export function closeContextMenu() {
  contextMenuState.update((s) => ({ ...s, isOpen: false }));
}

// ─── Export Preferences & Modal State ─────────────────────────────────────────
export interface ExportPreferences {
  exportType: 'combined_pip' | 'combined_sidebyside' | 'primary_only' | 'secondary_only' | 'bts_only' | 'motion_photo';
  format: 'Jpeg' | 'WebP' | 'Png';
  quality: number;
  embedExif: boolean;
  embedGps: boolean;
  isDefaultSet: boolean;
}

const defaultExportPreferences: ExportPreferences = {
  exportType: 'combined_pip',
  format: 'Jpeg',
  quality: 92,
  embedExif: true,
  embedGps: true,
  isDefaultSet: false,
};

function loadSavedExportPreferences(): ExportPreferences {
  if (typeof window !== 'undefined') {
    try {
      const saved = localStorage.getItem('bereal_export_preferences');
      if (saved) return { ...defaultExportPreferences, ...JSON.parse(saved) };
    } catch {}
  }
  return { ...defaultExportPreferences };
}

export const exportPreferences = writable<ExportPreferences>(loadSavedExportPreferences());

if (typeof window !== 'undefined') {
  exportPreferences.subscribe((val) => {
    try {
      localStorage.setItem('bereal_export_preferences', JSON.stringify(val));
    } catch {}
  });
}

export const exportModalState = writable<{
  isOpen: boolean;
  memory: ExplorerMemory | null;
}>({
  isOpen: false,
  memory: null,
});

export function openExportModal(memory: ExplorerMemory) {
  exportModalState.set({
    isOpen: true,
    memory,
  });
}

export function closeExportModal() {
  exportModalState.set({
    isOpen: false,
    memory: null,
  });
}
