export type ViewMode =
  | 'home'
  | 'toolkit-config'
  | 'recapper-config'
  | 'memories'
  | 'processing'
  | 'complete'
  | 'activity'
  | 'settings'
  | 'about';

export interface ActivityRecord {
  id: string;
  type: 'toolkit' | 'recapper';
  title: string;
  timestamp: string;
  outputPath: string;
  inputPath: string;
  durationSecs: number;
  status: 'success' | 'warning' | 'error' | 'cancelled';
  itemCount: number;
  memoriesCount?: number;
  dateRange?: string;
  details?: string;
}

export type MediaFilter = 'All' | 'PhotosOnly' | 'VideosOnly';
export type OutputFormat = 'Jpeg' | 'WebP' | 'Png';
export type CombineMode = 'PictureInPicture' | 'SideBySide';
export type SpeedMode = 'Ramp' | 'Even' | 'Accelerate' | 'Decelerate' | 'Wave';
export type GeocodingMode = 'Online' | 'Offline';
export type TextPosition =
  | 'TopLeft'
  | 'TopCenter'
  | 'TopRight'
  | 'MiddleLeft'
  | 'MiddleCenter'
  | 'MiddleRight'
  | 'BottomLeft'
  | 'BottomCenter'
  | 'BottomRight'
  | 'BelowDate'
  | 'AboveDate';

export interface LocationRule {
  comment?: string;
  condition: 'Default' | Record<string, string>;
  format: string;
}

export interface ToolkitConfig {
  inputPath: string;
  outputPath: string;
  dateRangeStart?: string;
  dateRangeEnd?: string;
  mediaFilter?: MediaFilter;
  convertFormat: OutputFormat;
  quality: number;
  createCombined: boolean;
  combineMode: CombineMode;
  createReversed: boolean;
  createMotionPhotos: boolean;
  createLivePhotos?: boolean;
  embedExif: boolean;
  keepOriginalFilename: boolean;
  cleanupIntermediates: boolean;
}

export interface RecapperConfig {
  inputFolder: string;
  outputPath: string;
  musicPath: string;
  dateRangeStart?: string;
  dateRangeEnd?: string;
  resolution: [number, number];
  fps: number;
  startPadding: number;
  endPadding: number;
  speedMode: SpeedMode;
  dateEnabled: boolean;
  dateFormat: string;
  datePosition: TextPosition;
  dateOffset: [number, number];
  fontPath: string;
  fontSize: number;
  shadowStrength: number;
  locationEnabled: boolean;
  locationPosition: TextPosition;
  locationOffset: [number, number];
  locationRules: LocationRule[];
  geocodingMode: GeocodingMode;
  minDurationSecs: number;
  maxDurationSecs: number;
}

export interface MonthCount {
  month: string; // "YYYY-MM"
  count: number;
}

export interface MissingFileInfo {
  path: string;
  date?: string;
  timestamp?: string;
  cameraType?: string; // 'primary' | 'secondary' | 'bts'
}

export interface RetakeStats {
  min: number;
  max: number;
  avg: number;
}

export interface ArchiveInfo {
  isValid: boolean;
  archiveType: 'Zip' | 'Directory';
  userName?: string;
  userFullname?: string;
  profilePictureDataUrl?: string;
  entryCount: number;
  validPostCount: number;
  corruptedPostCount: number;
  totalMediaCount: number;
  foundMediaCount: number;
  missingMediaCount: number;
  missingFilesSample: MissingFileInfo[];
  earliestDate?: string;
  latestDate?: string;
  // Rich media breakdown
  primaryPhotoCount: number;
  secondaryPhotoCount: number;
  primaryVideoCount: number;
  secondaryVideoCount: number;
  btsCount: number;
  withLocationCount: number;
  withCaptionCount: number;
  retakeStats?: RetakeStats;
  hasPostsJson: boolean;
  hasPhotosDir: boolean;
  hasUserJson: boolean;
  hasVideos: boolean;
  hasBts: boolean;
  monthlyHistogram: MonthCount[];
  photoMonthlyHistogram: MonthCount[];
  videoMonthlyHistogram: MonthCount[];
  validationErrors: string[];
  warnings: string[];
  postsJsonPath: string;
  mediaBasePath: string;
}

export type ProcessingStage =
  | 'Scanning'
  | 'Extracting'
  | 'Parsing'
  | 'Converting'
  | 'Compositing'
  | 'WritingExif'
  | 'Cleanup'
  | 'Complete'
  | 'LoadingAudio'
  | 'Geocoding'
  | 'RenderingFrames'
  | 'EncodingVideo';

export interface ProgressEvent {
  jobId?: string;
  stage: ProcessingStage;
  current: number;
  total: number;
  percentage: number;
  currentFile?: string;
}

export interface LogEvent {
  jobId?: string;
  level: 'Info' | 'Warn' | 'Error' | 'Debug';
  message: string;
  timestamp: string;
}

export interface DownloadProgressEvent {
  bytesDownloaded: number;
  totalBytes: number;
  percentage: number;
  speedMbps: number;
  status: string;
}

export interface GeoTierInfo {
  id: string; // 'cities15000' | 'cities5000' | 'cities500'
  name: string; // 'Lite' | 'Standard' | 'Ultra Detailed'
  subtitle: string;
  minPopulation: number;
  approxCities: string;
  approxDownloadMb: number;
  isInstalled: boolean;
  isActive: boolean;
  fileSizeBytes: number;
  cityCount: number;
  path: string;
}

export interface OfflineGeoDbStatus {
  isInstalled: boolean;
  activeTier: string;
  fileSizeBytes: number;
  cityCount: number;
  path: string;
  version: string;
  tiers: GeoTierInfo[];
}

export interface ActiveJob {
  id: string;
  type: 'toolkit' | 'recapper';
  title: string;
  inputPath: string;
  outputPath: string;
  startTime: number;
  memoriesCount?: number;
  dateRange?: string;
  stage: ProcessingStage;
  current: number;
  total: number;
  percentage: number;
  currentFile?: string;
  status: 'running' | 'completed' | 'cancelled' | 'error';
  errorMessage?: string;
  logs: LogEvent[];
  result?: ProcessingResult;
}

export interface ProcessingResult {
  entriesProcessed: number;
  filesConverted: number;
  combinedCreated: number;
  reversedCreated: number;
  motionPhotosCreated: number;
  livePhotosCreated?: number;
  filesSkipped: number;
  errors: string[];
  durationSecs: number;
  outputPath: string;
}

export interface FontInfo {
  family: string;
  style: string;
  path: string;
}

export interface AppSettings {
  toolkit: ToolkitConfig;
  recapper: RecapperConfig;
  lastInputPath?: string;
  lastOutputPath?: string;
}

export interface AudioAnalysis {
  duration: number;
  sampleRate: number;
  channels: number;
  waveform: number[];
}

export interface DestinationStatus {
  exists: boolean;
  isDirectory: boolean;
  isFile: boolean;
  fileCount: number;
}

export interface HardwareAccelerationInfo {
  gpuName: string;
  encoderName: string;
  isGpuAccelerated: boolean;
  cpuCores: number;
  parallelThreads: number;
}

// ─── Memories Explorer Types ──────────────────────────────────────────────────

export interface ExplorerMemory {
  id: string;
  index: number;
  takenAt: string;
  dateFormatted: string;
  dayNumber: string;
  monthKey: string; // "YYYY-MM"
  year: number;
  month: number;
  day: number;
  timeFormatted: string;
  isLate: boolean;
  lateDuration?: string;
  retakeCounter: number;
  caption?: string;
  location?: {
    latitude: number;
    longitude: number;
  };
  locationName?: string;
  suburb?: string;
  city?: string;
  country?: string;
  primaryPath?: string;
  secondaryPath?: string;
  btsPath?: string;
  isVideo: boolean;
  width?: number;
  height?: number;
}

export interface ExplorerData {
  memories: ExplorerMemory[];
  totalCount: number;
  uniqueYears: number[];
  uniqueMonths: string[];
  uniqueSuburbs: string[];
  uniqueCities: string[];
  uniqueCountries: string[];
  userName?: string;
  userFullname?: string;
  profilePictureDataUrl?: string;
  mediaBasePath: string;
}

export interface ExportSinglePostOptions {
  memoryIndex: number;
  primaryPath: string;
  secondaryPath?: string;
  btsPath?: string;
  outputPath: string;
  exportType: 'combined_pip' | 'combined_sidebyside' | 'primary_only' | 'secondary_only' | 'bts_only' | 'motion_photo';
  format: 'Jpeg' | 'WebP' | 'Png';
  quality: number;
  embedExif: boolean;
  takenAt?: string;
  latitude?: number;
  longitude?: number;
  caption?: string;
}

export interface ExplorerFilterState {
  searchQuery: string;
  selectedYear: number | 'all';
  selectedMonth: string | 'all'; // "YYYY-MM" or "all"
  selectedCountry: string | 'all';
  selectedCity: string | 'all';
  selectedSuburb: string | 'all';
  hasLocationOnly: boolean;
  hasBtsOnly: boolean;
  hasCaptionOnly: boolean;
  hasVideoOnly: boolean;
}

export interface MemoryHeaderSettings {
  showLocation: boolean;
  locationFormat: 'city_country' | 'suburb_city_country' | 'suburb_city' | 'city_only' | 'full' | 'custom';
  customLocationText?: string;
  showTimeTag: boolean;
  timeTagFormat: 'time_only' | 'date_only' | 'late_duration' | 'datetime' | 'custom';
  customTimeTagText?: string;
}

