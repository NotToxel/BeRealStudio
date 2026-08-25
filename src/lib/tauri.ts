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
  ActivityRecord,
  DestinationStatus,
  HardwareAccelerationInfo,
} from './types';

export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new Error(`[Tauri Mock] Command '${cmd}' called outside Tauri runtime.`);
  }
  return await invoke<T>(cmd, args);
}

// Archive & Scanning
export async function scanArchive(path: string): Promise<ArchiveInfo> {
  return await safeInvoke<ArchiveInfo>('scan_archive', { path });
}

export async function extractZip(zipPath: string, destDir: string): Promise<string> {
  return await safeInvoke<string>('extract_zip', { zipPath, destDir });
}

// Processing
export async function startToolkit(config: ToolkitConfig, jobId?: string): Promise<ProcessingResult> {
  return await safeInvoke<ProcessingResult>('start_toolkit', { config, jobId });
}

export async function checkToolkitConflicts(config: ToolkitConfig): Promise<DestinationStatus> {
  return await safeInvoke<DestinationStatus>('check_toolkit_conflicts', { config });
}

export async function cancelToolkit(): Promise<void> {
  return await safeInvoke<void>('cancel_toolkit');
}

export async function startRecapper(config: RecapperConfig, jobId?: string): Promise<ProcessingResult> {
  return await safeInvoke<ProcessingResult>('start_recapper', { config, jobId });
}

export async function cancelRecapper(): Promise<void> {
  return await safeInvoke<void>('cancel_recapper');
}

export async function cancelJob(jobId: string): Promise<boolean> {
  return await safeInvoke<boolean>('cancel_job', { jobId });
}

export async function listActiveJobs(): Promise<string[]> {
  return await safeInvoke<string[]>('list_active_jobs');
}

// Settings
export async function loadSettings(): Promise<AppSettings> {
  return await safeInvoke<AppSettings>('load_settings');
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return await safeInvoke<void>('save_settings', { settings });
}

export async function resetSettings(): Promise<AppSettings> {
  return await safeInvoke<AppSettings>('reset_settings');
}

export async function loadActivityHistory(): Promise<ActivityRecord[]> {
  try {
    return await safeInvoke<ActivityRecord[]>('load_activity_history');
  } catch {
    return [];
  }
}

export async function saveActivityHistory(history: ActivityRecord[]): Promise<void> {
  try {
    await safeInvoke<void>('save_activity_history', { history });
  } catch (e) {
    console.warn('Failed to save activity history to native store:', e);
  }
}

export async function clearNativeActivityHistory(): Promise<void> {
  try {
    await safeInvoke<void>('clear_activity_history');
  } catch (e) {
    console.warn('Failed to clear native activity history:', e);
  }
}

import { ffmpegInfo } from './stores';

// System
export async function checkFfmpeg(): Promise<string> {
  return await safeInvoke<string>('check_ffmpeg');
}

export async function checkExiftool(): Promise<string> {
  return await safeInvoke<string>('check_exiftool');
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
  return await safeInvoke<FontInfo[]>('list_system_fonts');
}

export async function checkDestinationStatus(path: string): Promise<DestinationStatus> {
  return await safeInvoke<DestinationStatus>('check_destination_status', { path });
}

// Debug logs
export async function exportDebugLog(outputPath: string): Promise<string> {
  return await safeInvoke<string>('export_debug_log', { outputPath });
}

export async function getDebugLogs(): Promise<LogEvent[]> {
  return await safeInvoke<LogEvent[]>('get_debug_logs');
}

export async function clearDebugLogs(): Promise<void> {
  return await safeInvoke<void>('clear_debug_logs');
}

export async function cleanupCancelledOutput(path: string): Promise<void> {
  return await safeInvoke<void>('cleanup_cancelled_output', { path });
}

export async function checkHardwareAcceleration(): Promise<HardwareAccelerationInfo> {
  return await safeInvoke<HardwareAccelerationInfo>('check_hardware_acceleration');
}

// File / Folder Pickers
export async function pickFolder(title?: string): Promise<string | null> {
  if (!isTauri()) return null;
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
  if (!isTauri()) return null;
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
  if (!isTauri()) return null;
  const selected = await save({
    title,
    defaultPath,
    filters: [{ name: 'Allowed Files', extensions }],
  });
  return typeof selected === 'string' ? selected : null;
}

export async function showInFolder(path: string): Promise<void> {
  if (!isTauri()) return;
  try {
    await safeInvoke('show_in_folder', { path });
  } catch (e) {
    console.warn('Native show_in_folder fallback to openPath:', e);
    await openPath(path);
  }
}

export async function openFile(path: string): Promise<void> {
  if (!isTauri()) return;
  try {
    await safeInvoke('open_file', { path });
  } catch (e) {
    console.warn('Native open_file fallback to openPath:', e);
    await openPath(path);
  }
}

export async function revealInFolder(path: string): Promise<void> {
  await showInFolder(path);
}

// Geocoding Database
export async function checkOfflineGeoDb(): Promise<OfflineGeoDbStatus> {
  return await safeInvoke<OfflineGeoDbStatus>('check_offline_geodb');
}

export async function downloadOfflineGeoDb(tier?: string): Promise<void> {
  return await safeInvoke<void>('download_offline_geodb', { tier });
}

export async function setActiveGeoDbTier(tier: string): Promise<OfflineGeoDbStatus> {
  return await safeInvoke<OfflineGeoDbStatus>('set_active_geodb_tier', { tier });
}

export async function deleteOfflineGeoDb(tier?: string): Promise<OfflineGeoDbStatus> {
  return await safeInvoke<OfflineGeoDbStatus>('delete_offline_geodb', { tier });
}

export async function analyzeAudio(path: string, buckets: number = 120): Promise<AudioAnalysis> {
  return await safeInvoke<AudioAnalysis>('analyze_audio', { path, buckets });
}

// Generic Typed Event Subscription Wrapper
export async function subscribeToEvent<T>(
  event: string,
  cb: (payload: T) => void
): Promise<UnlistenFn> {
  if (!isTauri()) {
    return () => {};
  }
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
