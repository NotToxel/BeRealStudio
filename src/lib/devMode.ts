import {
  archiveMetadata,
  currentArchive,
  toolkitConfig,
  recapperConfig,
  activityHistory,
  activeJobs,
  unreadActivityCount,
  lastScannedArchivePath,
  defaultToolkitConfig,
  defaultRecapperConfig,
} from './stores';
import { explorerData, calendarCurrentMonth, memoriesLoadError, isLoadingMemories } from './memoriesStore';
import type { ArchiveInfo, MonthCount, ActiveJob, ActivityRecord, ExplorerMemory } from './types';

export const isDev = import.meta.env.DEV;

export function isDemoExplicitlyRequested(): boolean {
  if (typeof window === 'undefined') return false;
  const searchParams = new URLSearchParams(window.location.search);
  return searchParams.get('demo') === '1' || sessionStorage.getItem('bereal_studio_demo_active') === '1';
}

export function loadDemoArchive() {
  const months: MonthCount[] = [
    { month: '2022-04', count: 12 },
    { month: '2022-05', count: 28 },
    { month: '2022-06', count: 25 },
    { month: '2022-07', count: 30 },
    { month: '2022-08', count: 27 },
    { month: '2022-09', count: 24 },
    { month: '2022-10', count: 29 },
    { month: '2022-11', count: 26 },
    { month: '2022-12', count: 28 },
    { month: '2023-01', count: 29 },
    { month: '2023-02', count: 26 },
    { month: '2023-03', count: 30 },
    { month: '2023-04', count: 28 },
    { month: '2023-05', count: 31 },
    { month: '2023-06', count: 29 },
    { month: '2023-07', count: 31 },
    { month: '2023-08', count: 30 },
    { month: '2023-09', count: 28 },
    { month: '2023-10', count: 30 },
    { month: '2023-11', count: 27 },
    { month: '2023-12', count: 29 },
    { month: '2024-01', count: 30 },
    { month: '2024-02', count: 28 },
    { month: '2024-03', count: 31 },
    { month: '2024-04', count: 29 },
    { month: '2024-05', count: 31 },
    { month: '2024-06', count: 28 },
    { month: '2024-07', count: 30 },
    { month: '2024-08', count: 31 },
    { month: '2024-09', count: 27 },
    { month: '2024-10', count: 29 },
    { month: '2024-11', count: 26 },
    { month: '2024-12', count: 28 },
  ];

  const totalEntries = months.reduce((s, m) => s + m.count, 0);

  const mockMeta: ArchiveInfo = {
    isValid: true,
    archiveType: 'Zip',
    userName: 'toxel',
    userFullname: 'Caleb Lim',
    profilePictureDataUrl: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><defs><linearGradient id='g' x1='0' y1='0' x2='1' y2='1'><stop offset='0%' stop-color='%2338bdf8'/><stop offset='100%' stop-color='%23a855f7'/></linearGradient></defs><circle cx='50' cy='50' r='50' fill='url(%23g)'/><text x='50' y='62' font-family='sans-serif' font-size='42' font-weight='bold' fill='white' text-anchor='middle'>T</text></svg>",
    entryCount: totalEntries,
    validPostCount: totalEntries,
    corruptedPostCount: 0,
    totalMediaCount: totalEntries * 2,
    foundMediaCount: totalEntries * 2,
    missingMediaCount: 0,
    missingFilesSample: [],
    primaryPhotoCount: totalEntries,
    secondaryPhotoCount: totalEntries,
    primaryVideoCount: 14,
    secondaryVideoCount: 14,
    btsCount: Math.floor(totalEntries * 0.75),
    withLocationCount: Math.floor(totalEntries * 0.88),
    withCaptionCount: Math.floor(totalEntries * 0.62),
    retakeStats: { min: 0, max: 9, avg: 1.6 },
    earliestDate: '2022-04-19T12:03:02.704Z',
    latestDate: '2024-12-31T23:58:45.000Z',
    hasPostsJson: true,
    hasPhotosDir: true,
    hasUserJson: true,
    hasVideos: true,
    hasBts: true,
    monthlyHistogram: months,
    photoMonthlyHistogram: months.map((m) => ({ month: m.month, count: Math.max(1, m.count - 2) })),
    videoMonthlyHistogram: [
      { month: '2023-06', count: 4 },
      { month: '2023-10', count: 3 },
      { month: '2024-03', count: 4 },
      { month: '2024-08', count: 3 },
    ],
    validationErrors: [],
    warnings: [],
    postsJsonPath: 'C:\\Development\\BeRealStudio\\archive\\bereal-gdpr-photo-toolkit\\posts.json',
    mediaBasePath: 'C:\\Users\\cl\\Downloads\\BeReal_Archive_GDPR\\Photos',
  };

  currentArchive.set(mockMeta);
  lastScannedArchivePath.set('C:\\Users\\cl\\Downloads\\BeReal_Archive_GDPR.zip');
  toolkitConfig.update((cfg) => ({
    ...cfg,
    inputPath: 'C:\\Users\\cl\\Downloads\\BeReal_Archive_GDPR.zip',
    outputPath: 'C:\\Users\\cl\\Pictures\\BeReal_Photos',
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

export function loadDemoExplorer() {
  const demoMemories: ExplorerMemory[] = [
    {
      id: 'demo-1',
      index: 0,
      takenAt: '2024-08-25T16:05:00.000Z',
      dateFormatted: '25 August 2024',
      dayNumber: '25',
      monthKey: '2024-08',
      year: 2024,
      month: 8,
      day: 25,
      timeFormatted: '16:05',
      isLate: false,
      lateDuration: undefined,
      retakeCounter: 4,
      caption: 'Coding BeReal Studio frontend & Rust backend!',
      location: { latitude: 51.5074, longitude: -0.1278 },
      locationName: 'London, United Kingdom',
      city: 'London',
      country: 'United Kingdom',
      primaryPath: 'https://images.unsplash.com/photo-1517694712202-14dd9538aa97?w=600&auto=format&fit=crop&q=80',
      secondaryPath: 'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=300&auto=format&fit=crop&q=80',
      btsPath: undefined,
      isVideo: false,
    },
    {
      id: 'demo-2',
      index: 1,
      takenAt: '2024-08-24T18:30:00.000Z',
      dateFormatted: '24 August 2024',
      dayNumber: '24',
      monthKey: '2024-08',
      year: 2024,
      month: 8,
      day: 24,
      timeFormatted: '18:30',
      isLate: false,
      lateDuration: undefined,
      retakeCounter: 0,
      caption: 'Evening walk along the river Thames',
      location: { latitude: 51.5055, longitude: -0.0754 },
      locationName: 'Tower Bridge, United Kingdom',
      city: 'London',
      country: 'United Kingdom',
      primaryPath: 'https://images.unsplash.com/photo-1513635269975-59663e0ac1ad?w=600&auto=format&fit=crop&q=80',
      secondaryPath: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=300&auto=format&fit=crop&q=80',
      btsPath: undefined,
      isVideo: false,
    },
    {
      id: 'demo-3',
      index: 2,
      takenAt: '2024-08-23T13:15:00.000Z',
      dateFormatted: '23 August 2024',
      dayNumber: '23',
      monthKey: '2024-08',
      year: 2024,
      month: 8,
      day: 23,
      timeFormatted: '13:15',
      isLate: true,
      lateDuration: '2h Late',
      retakeCounter: 2,
      caption: 'Lunch with the dev team',
      location: { latitude: 51.5137, longitude: -0.1303 },
      locationName: 'Soho, United Kingdom',
      city: 'London',
      country: 'United Kingdom',
      primaryPath: 'https://images.unsplash.com/photo-1555396273-367ea4eb4db5?w=600&auto=format&fit=crop&q=80',
      secondaryPath: 'https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=300&auto=format&fit=crop&q=80',
      btsPath: undefined,
      isVideo: false,
    },
    {
      id: 'demo-4',
      index: 3,
      takenAt: '2024-08-22T19:45:00.000Z',
      dateFormatted: '22 August 2024',
      dayNumber: '22',
      monthKey: '2024-08',
      year: 2024,
      month: 8,
      day: 22,
      timeFormatted: '19:45',
      isLate: false,
      lateDuration: undefined,
      retakeCounter: 1,
      caption: 'Sunset in the park',
      location: { latitude: 51.5073, longitude: -0.1657 },
      locationName: 'Hyde Park, United Kingdom',
      city: 'London',
      country: 'United Kingdom',
      primaryPath: 'https://images.unsplash.com/photo-1470240731273-7821a6eeb6bd?w=600&auto=format&fit=crop&q=80',
      secondaryPath: 'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=300&auto=format&fit=crop&q=80',
      btsPath: undefined,
      isVideo: false,
    },
    {
      id: 'demo-5',
      index: 4,
      takenAt: '2024-08-18T12:31:00.000Z',
      dateFormatted: '18 August 2024',
      dayNumber: '18',
      monthKey: '2024-08',
      year: 2024,
      month: 8,
      day: 18,
      timeFormatted: '12:31',
      isLate: false,
      lateDuration: undefined,
      retakeCounter: 0,
      caption: 'Margate seaside cliffs!',
      location: { latitude: 51.3896, longitude: 1.3868 },
      locationName: 'Margate, United Kingdom',
      city: 'Margate',
      country: 'United Kingdom',
      primaryPath: 'https://images.unsplash.com/photo-1507525428034-b723cf961d3e?w=600&auto=format&fit=crop&q=80',
      secondaryPath: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=300&auto=format&fit=crop&q=80',
      btsPath: undefined,
      isVideo: false,
    },
  ];

  memoriesLoadError.set(null);
  isLoadingMemories.set(false);
  explorerData.set({
    memories: demoMemories,
    totalCount: demoMemories.length,
    uniqueYears: [2024],
    uniqueMonths: ['2024-08'],
    uniqueSuburbs: ['Soho', 'Old Town'],
    uniqueCities: ['London', 'Margate'],
    uniqueCountries: ['United Kingdom'],
    userName: 'toxel',
    userFullname: 'Caleb Lim',
    profilePictureDataUrl: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><defs><linearGradient id='g' x1='0' y1='0' x2='1' y2='1'><stop offset='0%' stop-color='%2338bdf8'/><stop offset='100%' stop-color='%23a855f7'/></linearGradient></defs><circle cx='50' cy='50' r='50' fill='url(%23g)'/><text x='50' y='62' font-family='sans-serif' font-size='42' font-weight='bold' fill='white' text-anchor='middle'>T</text></svg>",
    mediaBasePath: 'C:\\Users\\cl\\Downloads\\BeReal_Archive_GDPR\\Photos',
  });
  calendarCurrentMonth.set('2024-08');
}

export function loadAllDemoData() {
  if (typeof window !== 'undefined') {
    sessionStorage.setItem('bereal_studio_demo_active', '1');
  }
  loadDemoArchive();
  loadDemoExplorer();
  loadDemoRecapper();
  loadDemoHistory();
  loadDemoActiveJobs();
}

export function clearAllDemoData() {
  if (typeof window !== 'undefined') {
    sessionStorage.removeItem('bereal_studio_demo_active');
    try {
      const url = new URL(window.location.href);
      if (url.searchParams.has('demo')) {
        url.searchParams.delete('demo');
        window.history.replaceState({}, '', url.pathname + (url.search ? url.search : ''));
      }
    } catch {
      // Ignore URL parse error
    }
  }

  currentArchive.set(null);
  lastScannedArchivePath.set('');
  explorerData.set(null);
  toolkitConfig.set({ ...defaultToolkitConfig });
  recapperConfig.set({ ...defaultRecapperConfig });
  activeJobs.update((jobs) => jobs.filter((j) => !j.id.startsWith('demo_')));
  activityHistory.update((hist) => hist.filter((r) => !r.id.startsWith('demo_')));
  unreadActivityCount.set(0);
}
