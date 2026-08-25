import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';
export { openPath };
import type {
  ArchiveInfo,
  ToolkitConfig,
  RecapperConfig,
  ProcessingResult,
  AppSettings,
  FontInfo,
  ProgressEvent,
  LogEvent,
} from './types';

// Archive & Scanning
export async function scanArchive(path: string): Promise<ArchiveInfo> {
  return await invoke<ArchiveInfo>('scan_archive', { path });
}

export async function extractZip(zipPath: string, destDir: string): Promise<string> {
  return await invoke<string>('extract_zip', { zipPath, destDir });
}

// Processing
export async function startToolkit(config: ToolkitConfig): Promise<ProcessingResult> {
  return await invoke<ProcessingResult>('start_toolkit', { config });
}

export async function cancelToolkit(): Promise<void> {
  return await invoke<void>('cancel_toolkit');
}

export async function startRecapper(config: RecapperConfig): Promise<ProcessingResult> {
  return await invoke<ProcessingResult>('start_recapper', { config });
}

export async function cancelRecapper(): Promise<void> {
  return await invoke<void>('cancel_recapper');
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

export async function revealInFolder(path: string): Promise<void> {
  await openPath(path);
}

// Event Subscriptions
export async function onToolkitProgress(
  cb: (event: ProgressEvent) => void
): Promise<UnlistenFn> {
  return await listen<ProgressEvent>('toolkit-progress', (e) => cb(e.payload));
}

export async function onToolkitLog(
  cb: (event: LogEvent) => void
): Promise<UnlistenFn> {
  return await listen<LogEvent>('toolkit-log', (e) => cb(e.payload));
}

export async function onRecapperProgress(
  cb: (event: ProgressEvent) => void
): Promise<UnlistenFn> {
  return await listen<ProgressEvent>('recapper-progress', (e) => cb(e.payload));
}

export async function onRecapperLog(
  cb: (event: LogEvent) => void
): Promise<UnlistenFn> {
  return await listen<LogEvent>('recapper-log', (e) => cb(e.payload));
}
