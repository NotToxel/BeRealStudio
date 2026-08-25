import { writable, derived, get } from 'svelte/store';
import type { ExplorerData, ExplorerMemory, ExplorerFilterState } from './types';
import { loadExplorerMemories, readMediaFileDataUrl, convertFileSrc, isTauri } from './tauri';
import { lastScannedArchivePath, currentArchive } from './stores';

export const initialFilterState: ExplorerFilterState = {
  searchQuery: '',
  selectedYear: 'all',
  selectedMonth: 'all',
  selectedCity: 'all',
  selectedCountry: 'all',
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

// ─── Derived Filtered Memories Store ──────────────────────────────────────────

export const filteredMemories = derived(
  [explorerData, explorerFilter],
  ([$data, $filter]) => {
    if (!$data || !$data.memories) return [];

    return $data.memories.filter((m) => {
      // 1. Text search (caption, location, date)
      if ($filter.searchQuery.trim()) {
        const q = $filter.searchQuery.toLowerCase().trim();
        const captionMatch = m.caption?.toLowerCase().includes(q) ?? false;
        const locationMatch = m.locationName?.toLowerCase().includes(q) ?? false;
        const dateMatch = m.dateFormatted.toLowerCase().includes(q);
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

      // 4. City filter
      if ($filter.selectedCity !== 'all' && m.city !== $filter.selectedCity) {
        return false;
      }

      // 5. Country filter
      if ($filter.selectedCountry !== 'all' && m.country !== $filter.selectedCountry) {
        return false;
      }

      // 6. Flags
      if ($filter.hasLocationOnly && !m.location) return false;
      if ($filter.hasBtsOnly && !m.btsPath) return false;
      if ($filter.hasCaptionOnly && (!m.caption || m.caption.trim().length === 0)) return false;
      if ($filter.retakesOnly && m.retakeCounter === 0) return false;

      return true;
    });
  }
);

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
export const memoriesByDate = derived(explorerData, ($data) => {
  const map = new Map<string, ExplorerMemory[]>();
  if (!$data) return map;

  for (const m of $data.memories) {
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
