import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import { openPath, openUrl } from '@tauri-apps/plugin-opener';
export { openPath, openUrl };
import type {
  ArchiveInfo,
  ToolkitConfig,
  RecapperConfig,
  ProcessingResult,
  AppSettings,
  FontInfo,
  ProgressEvent,
  LogEvent,
  OfflineGeoDbStatus,
  DownloadProgressEvent,
  AudioAnalysis,
} from './types';

// Archive & Scanning
export async function scanArchive(path: string): Promise<ArchiveInfo> {
  return await invoke<ArchiveInfo>('scan_archive', { path });
}

export async function extractZip(zipPath: string, destDir: string): Promise<string> {
  return await invoke<string>('extract_zip', { zipPath, destDir });
}

// Processing
export async function startToolkit(config: ToolkitConfig, jobId?: string): Promise<ProcessingResult> {
  return await invoke<ProcessingResult>('start_toolkit', { config, jobId });
}

export async function cancelToolkit(): Promise<void> {
  return await invoke<void>('cancel_toolkit');
}

export async function startRecapper(config: RecapperConfig, jobId?: string): Promise<ProcessingResult> {
  return await invoke<ProcessingResult>('start_recapper', { config, jobId });
}

export async function cancelRecapper(): Promise<void> {
  return await invoke<void>('cancel_recapper');
}

export async function cancelJob(jobId: string): Promise<boolean> {
  return await invoke<boolean>('cancel_job', { jobId });
}

export async function listActiveJobs(): Promise<string[]> {
  return await invoke<string[]>('list_active_jobs');
}

// Settings
export async function loadSettings(): Promise<AppSettings> {
  return await invoke<AppSettings>('load_settings');
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return await invoke<void>('save_settings', { settings });
}

export async function resetSettings(): Promise<AppSettings> {
  return await invoke<AppSettings>('reset_settings');
}

import { ffmpegInfo } from './stores';

// System
export async function checkFfmpeg(): Promise<string> {
  return await invoke<string>('check_ffmpeg');
}

export async function detectFfmpeg(): Promise<string | null> {
  ffmpegInfo.update((s) => ({ ...s, checking: true }));
  try {
    const p = await checkFfmpeg();
    ffmpegInfo.set({ path: p || null, checking: false, checked: true });
    return p || null;
  } catch {
    ffmpegInfo.set({ path: null, checking: false, checked: true });
    return null;
  }
}

export async function listSystemFonts(): Promise<FontInfo[]> {
  return await invoke<FontInfo[]>('list_system_fonts');
}

// Debug logs
export async function exportDebugLog(outputPath: string): Promise<string> {
  return await invoke<string>('export_debug_log', { outputPath });
}

export async function getDebugLogs(): Promise<LogEvent[]> {
  return await invoke<LogEvent[]>('get_debug_logs');
}

export async function clearDebugLogs(): Promise<void> {
  return await invoke<void>('clear_debug_logs');
}

// File / Folder Pickers
export async function pickFolder(title?: string): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: title || 'Select Folder',
  });
  return typeof selected === 'string' ? selected : null;
}

export async function pickFile(
  title: string,
  extensions: string[]
): Promise<string | null> {
  const selected = await open({
    directory: false,
    multiple: false,
    title,
    filters: [{ name: 'Allowed Files', extensions }],
  });
  return typeof selected === 'string' ? selected : null;
}

export async function saveFilePicker(
  title: string,
  defaultPath?: string,
  extensions: string[] = ['json']
): Promise<string | null> {
  const selected = await save({
    title,
    defaultPath,
    filters: [{ name: 'Allowed Files', extensions }],
  });
  return typeof selected === 'string' ? selected : null;
}

export async function showInFolder(path: string): Promise<void> {
  try {
    await invoke('show_in_folder', { path });
  } catch (e) {
    console.warn('Native show_in_folder fallback to openPath:', e);
    await openPath(path);
  }
}

export async function revealInFolder(path: string): Promise<void> {
  await showInFolder(path);
}


// Geocoding Database
export async function checkOfflineGeoDb(): Promise<OfflineGeoDbStatus> {
  return await invoke<OfflineGeoDbStatus>('check_offline_geodb');
}

export async function downloadOfflineGeoDb(tier?: string): Promise<void> {
  return await invoke<void>('download_offline_geodb', { tier });
}

export async function setActiveGeoDbTier(tier: string): Promise<OfflineGeoDbStatus> {
  return await invoke<OfflineGeoDbStatus>('set_active_geodb_tier', { tier });
}

export async function deleteOfflineGeoDb(tier?: string): Promise<OfflineGeoDbStatus> {
  return await invoke<OfflineGeoDbStatus>('delete_offline_geodb', { tier });
}

export async function analyzeAudio(path: string, buckets: number = 120): Promise<AudioAnalysis> {
  return await invoke<AudioAnalysis>('analyze_audio', { path, buckets });
}

// Generic Typed Event Subscription Wrapper
export async function subscribeToEvent<T>(
  event: string,
  cb: (payload: T) => void
): Promise<UnlistenFn> {
  return await listen<T>(event, (e) => cb(e.payload));
}

// Event Subscriptions
export async function onToolkitProgress(
  cb: (event: ProgressEvent) => void
): Promise<UnlistenFn> {
  return await subscribeToEvent<ProgressEvent>('toolkit-progress', cb);
}

export async function onToolkitLog(
  cb: (event: LogEvent) => void
): Promise<UnlistenFn> {
  return await subscribeToEvent<LogEvent>('toolkit-log', cb);
}

export async function onRecapperProgress(
  cb: (event: ProgressEvent) => void
): Promise<UnlistenFn> {
  return await subscribeToEvent<ProgressEvent>('recapper-progress', cb);
}

export async function onRecapperLog(
  cb: (event: LogEvent) => void
): Promise<UnlistenFn> {
  return await subscribeToEvent<LogEvent>('recapper-log', cb);
}

export async function onJobProgress(
  jobId: string,
  cb: (event: ProgressEvent) => void
): Promise<UnlistenFn> {
  return await subscribeToEvent<ProgressEvent>(`job-progress-${jobId}`, cb);
}

export async function onJobLog(
  jobId: string,
  cb: (event: LogEvent) => void
): Promise<UnlistenFn> {
  return await subscribeToEvent<LogEvent>(`job-log-${jobId}`, cb);
}

export async function onDownloadProgress(
  cb: (event: DownloadProgressEvent) => void
): Promise<UnlistenFn> {
  return await subscribeToEvent<DownloadProgressEvent>('download-progress', cb);
}
