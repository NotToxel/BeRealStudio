export type ViewMode = 'home' | 'toolkit-config' | 'recapper-config' | 'processing' | 'complete' | 'settings' | 'about';

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
  convertFormat: OutputFormat;
  quality: number;
  createCombined: boolean;
  combineMode: CombineMode;
  createReversed: boolean;
  createMotionPhotos: boolean;
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
}

export interface MonthCount {
  month: string; // "YYYY-MM"
  count: number;
}

export interface ArchiveInfo {
  isValid: boolean;
  archiveType: 'Zip' | 'Directory';
  userName?: string;
  userFullname?: string;
  entryCount: number;
  validPostCount: number;
  corruptedPostCount: number;
  totalMediaCount: number;
  foundMediaCount: number;
  missingMediaCount: number;
  missingFilesSample: string[];
  earliestDate?: string;
  latestDate?: string;
  hasPostsJson: boolean;
  hasPhotosDir: boolean;
  hasUserJson: boolean;
  hasVideos: boolean;
  hasBts: boolean;
  monthlyHistogram: MonthCount[];
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
  stage: ProcessingStage;
  current: number;
  total: number;
  percentage: number;
  currentFile?: string;
}

export interface LogEvent {
  level: 'Info' | 'Warn' | 'Error' | 'Debug';
  message: string;
  timestamp: string;
}

export interface ProcessingResult {
  entriesProcessed: number;
  filesConverted: number;
  combinedCreated: number;
  reversedCreated: number;
  motionPhotosCreated: number;
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
