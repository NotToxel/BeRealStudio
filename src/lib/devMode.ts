import {
  archiveMetadata,
  currentArchive,
  toolkitConfig,
  recapperConfig,
  activityHistory,
  activeJobs,
  unreadActivityCount,
} from './stores';
import type { ArchiveInfo, MonthCount, ActiveJob, ActivityRecord } from './types';

export const isDev = import.meta.env.DEV;

export function loadDemoArchive() {
  const months: MonthCount[] = [
    { month: '2024-01', count: 8 },
    { month: '2024-02', count: 11 },
    { month: '2024-03', count: 14 },
    { month: '2024-04', count: 18 },
    { month: '2024-05', count: 22 },
    { month: '2024-06', count: 19 },
    { month: '2024-07', count: 16 },
    { month: '2024-08', count: 25 },
    { month: '2024-09', count: 20 },
    { month: '2024-10', count: 15 },
    { month: '2024-11', count: 12 },
    { month: '2024-12', count: 10 },
  ];

  const totalEntries = months.reduce((s, m) => s + m.count, 0);

  const mockMeta: ArchiveInfo = {
    isValid: true,
    archiveType: 'Zip',
    userName: 'alex',
    userFullname: 'Alex Developer',
    entryCount: totalEntries,
    validPostCount: totalEntries,
    corruptedPostCount: 0,
    totalMediaCount: totalEntries * 2,
    foundMediaCount: totalEntries * 2,
    missingMediaCount: 0,
    missingFilesSample: [],
    primaryPhotoCount: totalEntries,
    secondaryPhotoCount: totalEntries,
    primaryVideoCount: 0,
    secondaryVideoCount: 0,
    btsCount: Math.floor(totalEntries * 0.4),
    withLocationCount: Math.floor(totalEntries * 0.8),
    withCaptionCount: Math.floor(totalEntries * 0.6),
    earliestDate: '2024-01-02T14:22:10.000Z',
    latestDate: '2024-12-31T23:58:45.000Z',
    hasPostsJson: true,
    hasPhotosDir: true,
    hasUserJson: true,
    hasVideos: false,
    hasBts: true,
    monthlyHistogram: months,
    validationErrors: [],
    warnings: [],
    postsJsonPath: 'C:\\Users\\Developer\\Downloads\\BeReal_GDPR_Demo\\posts.json',
    mediaBasePath: 'C:\\Users\\Developer\\Downloads\\BeReal_GDPR_Demo\\Photos',
  };

  currentArchive.set(mockMeta);
  toolkitConfig.update((cfg) => ({
    ...cfg,
    inputPath: 'C:\\Users\\Developer\\Downloads\\BeReal_Archive_2024_GDPR.zip',
    outputPath: 'C:\\Users\\Developer\\Downloads\\BeReal_Photos',
    dateRangeStart: '2024-01-01',
    dateRangeEnd: '2024-12-31',
    convertFormat: 'Jpeg',
    quality: 92,
    createCombined: true,
    createReversed: false,
    createMotionPhotos: true,
    embedExif: true,
  }));
}

export function loadDemoRecapper() {
  const waveform: number[] = [];
  for (let i = 0; i < 120; i++) {
    // Generate realistic rhythm pulse envelope
    const base = Math.sin((i / 120) * Math.PI * 4) * 0.4 + 0.5;
    const noise = Math.sin(i * 1.7) * 0.15 + (Math.random() * 0.1 - 0.05);
    waveform.push(Math.max(0.08, Math.min(1.0, base + noise)));
  }

  recapperConfig.update((cfg) => ({
    ...cfg,
    inputFolder: 'C:\\Users\\Developer\\Downloads\\BeReal_Photos\\combined',
    outputPath: 'C:\\Users\\Developer\\Videos\\BeReal_2024_Recap.mp4',
    musicPath: 'C:\\Users\\Developer\\Music\\Summer_Memories_2024.mp3',
    fps: 30,
    speedMode: 'Ramp',
    startPadding: 3.0,
    endPadding: 3.0,
    dateEnabled: true,
    dateFormat: '%d %B %Y',
    locationEnabled: true,
    locationPosition: 'BelowDate',
    locationRules: [
      { comment: 'United Kingdom formatting', condition: { Country: 'gb' }, format: '{city}, UK' },
      { comment: 'United States formatting', condition: { Country: 'us' }, format: '{city}, {state}' },
      { comment: 'Default fallback', condition: 'Default', format: '{city}, {country}' },
    ],
  }));

  return waveform;
}

export function loadDemoHistory() {
  const demoRecords: ActivityRecord[] = [
    {
      id: 'demo_act_1',
      type: 'toolkit',
      title: 'Photo Processing (190 Memories)',
      timestamp: new Date(Date.now() - 1000 * 60 * 42).toISOString(),
      outputPath: 'C:\\Users\\Developer\\Downloads\\BeReal_Photos',
      inputPath: 'C:\\Users\\Developer\\Downloads\\BeReal_Archive_2024_GDPR.zip',
      durationSecs: 24.8,
      status: 'success',
      itemCount: 380,
      memoriesCount: 190,
      dateRange: '1 Jan 2024 – 31 Dec 2024',
      details: '1 Jan 2024 – 31 Dec 2024 • Processed in 24.8s',
    },
    {
      id: 'demo_act_2',
      type: 'recapper',
      title: 'Recap Video (30 FPS)',
      timestamp: new Date(Date.now() - 1000 * 60 * 120).toISOString(),
      outputPath: 'C:\\Users\\Developer\\Videos\\BeReal_2024_Recap.mp4',
      inputPath: 'C:\\Users\\Developer\\Downloads\\BeReal_Photos\\combined',
      durationSecs: 41.2,
      status: 'success',
      itemCount: 190,
      memoriesCount: 190,
      dateRange: 'Full Year 2024',
      details: 'Generated in 41.2s • H.264 / AAC 1440x1920',
    },
    {
      id: 'demo_act_3',
      type: 'toolkit',
      title: 'Photo Processing (48 Memories)',
      timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24 * 2).toISOString(),
      outputPath: 'C:\\Users\\Developer\\Downloads\\BeReal_Summer_2024',
      inputPath: 'C:\\Users\\Developer\\Downloads\\BeReal_Archive_Summer.zip',
      durationSecs: 6.4,
      status: 'success',
      itemCount: 96,
      memoriesCount: 48,
      dateRange: '1 Jun 2024 – 31 Aug 2024',
      details: '1 Jun 2024 – 31 Aug 2024 • Processed in 6.4s',
    },
  ];

  activityHistory.set(demoRecords);
  unreadActivityCount.set(demoRecords.length);
}

export function loadDemoActiveJobs() {
  const mockJobs: ActiveJob[] = [
    {
      id: 'demo_job_1',
      type: 'toolkit',
      title: 'Photo Processing (190 Memories)',
      inputPath: 'C:\\Users\\Developer\\Downloads\\BeReal_Archive_2024_GDPR.zip',
      outputPath: 'C:\\Users\\Developer\\Downloads\\BeReal_Photos',
      memoriesCount: 190,
      dateRange: '1 Jan 2024 – 31 Dec 2024',
      startTime: Date.now() - 1000 * 18,
      stage: 'Compositing',
      current: 124,
      total: 190,
      percentage: 65.26,
      currentFile: 'Photos/post_2024-08-24_1830.jpg',
      status: 'running',
      logs: [
        { level: 'Info', message: 'Scanning archive ZIP with streaming JSON reader...', timestamp: new Date().toISOString() },
        { level: 'Info', message: 'Found 190 valid post memory pairs across 12 months.', timestamp: new Date().toISOString() },
        { level: 'Info', message: 'Spawning Rayon threadpool with 8 CPU worker cores.', timestamp: new Date().toISOString() },
        { level: 'Info', message: 'Compositing Picture-in-Picture lenses (60px radius, 7px border)...', timestamp: new Date().toISOString() },
      ],
    },
  ];

  activeJobs.set(mockJobs);
}

export function loadAllDemoData() {
  loadDemoArchive();
  loadDemoRecapper();
  loadDemoHistory();
  loadDemoActiveJobs();
}

export function clearAllDemoData() {
  currentArchive.set(null);
  activeJobs.set([]);
  activityHistory.set([]);
  unreadActivityCount.set(0);
}
