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
  ActiveJob,
  OfflineGeoDbStatus,
  DownloadProgressEvent,
  ActivityRecord,
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
  startPadding: 3.0,
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

// Archive scan persistence
export const lastScannedArchivePath = writable<string>('');

// Processing & Progress (Active Primary Single View)
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

// ─── Parallel Multi-Job Active Queue ─────────────────────────────────────────
export const activeJobs = writable<ActiveJob[]>([]);

export function createActiveJob(
  params: {
    type: 'toolkit' | 'recapper';
    title: string;
    inputPath: string;
    outputPath: string;
  }
): ActiveJob {
  const newJob: ActiveJob = {
    id: `job_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
    type: params.type,
    title: params.title,
    inputPath: params.inputPath,
    outputPath: params.outputPath,
    startTime: Date.now(),
    stage: 'Scanning',
    current: 0,
    total: 0,
    percentage: 0,
    status: 'running',
    logs: [],
  };

  activeJobs.update((list) => [newJob, ...list]);
  return newJob;
}

export function updateActiveJobProgress(jobId: string, event: ProgressEvent) {
  activeJobs.update((list) =>
    list.map((j) => {
      if (j.id !== jobId) return j;
      return {
        ...j,
        stage: event.stage,
        current: event.current,
        total: event.total,
        percentage: event.percentage,
        currentFile: event.currentFile ?? j.currentFile,
        status: event.stage === 'Complete' ? 'completed' : j.status,
      };
    })
  );
}

export function appendActiveJobLog(jobId: string, event: LogEvent) {
  activeJobs.update((list) =>
    list.map((j) => {
      if (j.id !== jobId) return j;
      return {
        ...j,
        logs: [...j.logs.slice(-200), event],
      };
    })
  );
}

export function completeActiveJob(jobId: string, result: ProcessingResult) {
  activeJobs.update((list) =>
    list.map((j) => {
      if (j.id !== jobId) return j;
      return {
        ...j,
        status: 'completed',
        percentage: 100,
        stage: 'Complete',
        result,
      };
    })
  );
}

export function cancelActiveJobById(jobId: string) {
  activeJobs.update((list) =>
    list.map((j) => {
      if (j.id !== jobId) return j;
      return {
        ...j,
        status: 'cancelled',
      };
    })
  );
}

export function errorActiveJob(jobId: string, errorMsg: string) {
  activeJobs.update((list) =>
    list.map((j) => {
      if (j.id !== jobId) return j;
      return {
        ...j,
        status: 'error',
        errorMessage: errorMsg,
      };
    })
  );
}

export function removeActiveJob(jobId: string) {
  activeJobs.update((list) => list.filter((j) => j.id !== jobId));
}

// ─── Offline Geocoding Database Stores ───────────────────────────────────────
export const offlineGeoDbStatus = writable<OfflineGeoDbStatus | null>(null);
export const isDownloadingGeoDb = writable<boolean>(false);
export const downloadGeoDbProgress = writable<DownloadProgressEvent | null>(null);

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
