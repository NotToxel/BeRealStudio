/**
 * Intelligent Error Diagnosis & Resolution Helper
 * Translates low-level OS errors and native engine exceptions into human-friendly explanations.
 */

export interface ErrorDiagnosis {
  title: string;
  category: 'Permission' | 'LockedFile' | 'NotFound' | 'DiskSpace' | 'FFmpeg' | 'CorruptArchive' | 'General';
  categoryLabel: string;
  explanation: string;
  suggestion: string;
  code?: string;
  rawDetails: string;
}

export function diagnoseError(rawError: unknown, contextTitle?: string): ErrorDiagnosis {
  const errStr = typeof rawError === 'string' 
    ? rawError 
    : (rawError instanceof Error ? rawError.message : JSON.stringify(rawError));

  const lower = errStr.toLowerCase();

  // 1. OS Error 5 / Access Denied / Permission Denied
  if (lower.includes('os error 5') || lower.includes('access is denied') || lower.includes('permissiondenied') || lower.includes('permission denied')) {
    return {
      title: contextTitle || 'Permission Denied (Access Denied)',
      category: 'Permission',
      categoryLabel: 'Windows OS Error 5 • Access Denied',
      code: 'OS_ERROR_5',
      explanation: 'Windows prevented BeReal Studio from writing or modifying files in the specified location.',
      suggestion: 
        '• Choose an output folder within your personal user directory (e.g. Downloads, Pictures, or Desktop).\n' +
        '• Ensure you are not writing directly to the drive root (C:\\) or protected system directories (Program Files, Windows).\n' +
        '• If the folder already exists, verify it is not marked as Read-Only in its Windows folder properties.\n' +
        '• Check if Windows Defender "Controlled Folder Access" or third-party anti-virus is restricting file writes.',
      rawDetails: errStr,
    };
  }

  // 2. OS Error 32 / Sharing Violation (File locked by another process)
  if (lower.includes('os error 32') || lower.includes('sharing violation') || lower.includes('used by another process') || lower.includes('lock')) {
    return {
      title: contextTitle || 'File In Use / Locked by Another Program',
      category: 'LockedFile',
      categoryLabel: 'Windows OS Error 32 • Sharing Violation',
      code: 'OS_ERROR_32',
      explanation: 'An existing file in the target directory is currently open or locked by another program (e.g., Windows Photos, VLC, or File Explorer).',
      suggestion: 
        '• Close any image viewers, video players, or editing software currently displaying files in the output directory.\n' +
        '• Close any File Explorer windows that have the preview pane active on the target folder.\n' +
        '• Try renaming or selecting a new destination folder.',
      rawDetails: errStr,
    };
  }

  // 3. OS Error 2 / File or Path Not Found
  if (lower.includes('os error 2') || lower.includes('not found') || lower.includes('no such file') || lower.includes('cannot find the file')) {
    return {
      title: contextTitle || 'File or Folder Not Found',
      category: 'NotFound',
      categoryLabel: 'Windows OS Error 2 • File Not Found',
      code: 'OS_ERROR_2',
      explanation: 'The application was unable to find the specified archive, image folder, audio track, or target directory.',
      suggestion: 
        '• Verify that the file or folder has not been moved, renamed, or deleted.\n' +
        '• If using a removable drive or USB stick, verify it is firmly connected.\n' +
        '• Re-select the file using the browse button in the app.',
      rawDetails: errStr,
    };
  }

  // 4. OS Error 112 / Disk Full
  if (lower.includes('os error 112') || lower.includes('not enough space') || lower.includes('disk full') || lower.includes('out of disk space')) {
    return {
      title: contextTitle || 'Disk Space Exhausted',
      category: 'DiskSpace',
      categoryLabel: 'Windows OS Error 112 • Disk Full',
      code: 'OS_ERROR_112',
      explanation: 'There is insufficient free disk space on the selected storage drive to extract photos or encode video.',
      suggestion: 
        '• Free up disk space on the drive or select an output folder on a drive with more available capacity.\n' +
        '• Empty your Recycle Bin or delete temporary working files.',
      rawDetails: errStr,
    };
  }

  // 5. FFmpeg Missing or Execution Error
  if (lower.includes('ffmpeg') || lower.includes('pipe') || lower.includes('muxer')) {
    return {
      title: contextTitle || 'FFmpeg Video Encoder Issue',
      category: 'FFmpeg',
      categoryLabel: 'Video Pipeline • FFmpeg',
      code: 'FFMPEG_ERROR',
      explanation: 'An issue occurred while communicating with or executing the FFmpeg video processing binary.',
      suggestion: 
        '• Ensure FFmpeg is installed and accessible in your system PATH (Run: winget install Gyan.FFmpeg on Windows or brew install ffmpeg on macOS).\n' +
        '• Check the Settings view to verify the detected FFmpeg binary path.\n' +
        '• Ensure your audio track is a valid, uncorrupted audio format (MP3, WAV, M4A, FLAC).',
      rawDetails: errStr,
    };
  }

  // 6. Corrupted Archive or Malformed JSON
  if (lower.includes('posts.json') || lower.includes('zip') || lower.includes('corrupted') || lower.includes('invalid format') || lower.includes('eof')) {
    return {
      title: contextTitle || 'Archive Data Parsing Issue',
      category: 'CorruptArchive',
      categoryLabel: 'Archive Integrity • GDPR Parse Error',
      code: 'ARCHIVE_PARSE_ERROR',
      explanation: 'The provided BeReal export ZIP file is incomplete or posts.json metadata is malformed.',
      suggestion: 
        '• Ensure the ZIP download from BeReal completed 100% and is not partially downloaded.\n' +
        '• Try extracting the ZIP manually with Windows Explorer or 7-Zip, then select the extracted folder directly.\n' +
        '• Verify that posts.json exists inside the root of the archive.',
      rawDetails: errStr,
    };
  }

  // General Fallback
  return {
    title: contextTitle || 'Operation Encountered an Error',
    category: 'General',
    categoryLabel: 'General Exception',
    explanation: 'An unexpected system or processing error occurred during execution.',
    suggestion: 
      '• Review the detailed technical trace below for specifics.\n' +
      '• Verify that your input files exist and output directories are accessible.\n' +
      '• You can export a comprehensive debug log from the Settings or Complete screen.',
    rawDetails: errStr,
  };
}
