import { writable } from 'svelte/store';
import type {
  ViewMode,
  ToolkitConfig,
  RecapperConfig,
  ArchiveInfo,
  ProcessingResult,
  ProgressEvent,
  LogEvent,
  AppSettings,
} from './types';

// Default configurations
export const defaultToolkitConfig: ToolkitConfig = {
  inputPath: '',
  outputPath: '',
  dateRangeStart: undefined,
  dateRangeEnd: undefined,
  convertFormat: 'Jpeg',
  quality: 90,
  createCombined: true,
  combineMode: 'PictureInPicture',
  createReversed: false,
  createMotionPhotos: false,
  embedExif: true,
  keepOriginalFilename: false,
  cleanupIntermediates: true,
};

export const defaultRecapperConfig: RecapperConfig = {
  inputFolder: '',
  outputPath: '',
  musicPath: '',
  dateRangeStart: undefined,
  dateRangeEnd: undefined,
  resolution: [1440, 1920],
  fps: 30,
  startPadding: 2.0,
  endPadding: 3.0,
  speedMode: 'Ramp',
  dateEnabled: true,
  dateFormat: '%d %B %Y',
  datePosition: 'BottomCenter',
  dateOffset: [0, -150],
  fontPath: '',
  fontSize: 100,
  shadowStrength: 5,
  locationEnabled: true,
  locationPosition: 'BelowDate',
  locationOffset: [0, 0],
  locationRules: [
    {
      comment: 'Default fallback',
      condition: 'Default',
      format: '{city}, {country}',
    },
  ],
  geocodingMode: 'Online',
};

// Navigation & Active Mode
export const currentView = writable<ViewMode>('home');
export const activeFeature = writable<'toolkit' | 'recapper'>('toolkit');

// Config Stores
export const toolkitConfig = writable<ToolkitConfig>({ ...defaultToolkitConfig });
export const recapperConfig = writable<RecapperConfig>({ ...defaultRecapperConfig });

// Scan Metadata
export const currentArchive = writable<ArchiveInfo | null>(null);
export const archiveMetadata = currentArchive;
export const isScanning = writable<boolean>(false);

// Archive scan persistence — tracks the last successfully scanned path so we
// don't re-scan the same archive when switching tabs.
export const lastScannedArchivePath = writable<string>('');

// Processing & Progress
export const isProcessing = writable<boolean>(false);
export const progressState = writable<ProgressEvent>({
  stage: 'Scanning',
  current: 0,
  total: 0,
  percentage: 0,
  currentFile: undefined,
});
export const liveLogs = writable<LogEvent[]>([]);
export const processingResult = writable<ProcessingResult | null>(null);

// Error Modal
export interface ModalError {
  title: string;
  message: string;
  details?: string;
}
export const activeError = writable<ModalError | null>(null);

// FFmpeg Detection Store
export const ffmpegInfo = writable<{ path: string | null; checking: boolean; checked: boolean }>({
  path: null,
  checking: false,
  checked: false,
});

// Activity History Store
function loadStoredActivity(): ActivityRecord[] {
  if (typeof window === 'undefined') return [];
  try {
    const raw = localStorage.getItem('bereal_studio_activity_history');
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

export const activityHistory = writable<ActivityRecord[]>(loadStoredActivity());

if (typeof window !== 'undefined') {
  activityHistory.subscribe((list) => {
    try {
      localStorage.setItem('bereal_studio_activity_history', JSON.stringify(list));
    } catch {
      // Ignore storage quota errors
    }
  });
}

export function recordActivity(entry: Omit<ActivityRecord, 'id' | 'timestamp'>) {
  const newRecord: ActivityRecord = {
    ...entry,
    id: `act_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
    timestamp: new Date().toISOString(),
  };
  activityHistory.update((list) => [newRecord, ...list.slice(0, 49)]);
}

export function clearActivityHistory() {
  activityHistory.set([]);
}

export function deleteActivityRecord(id: string) {
  activityHistory.update((list) => list.filter((item) => item.id !== id));
}
