import type { ToolkitConfig, RecapperConfig, ArchiveInfo } from './types';

export interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

/**
 * Validate Photo Processing Toolkit configuration.
 */
export function validateToolkitConfig(
  config: ToolkitConfig,
  archiveInfo?: ArchiveInfo | null
): ValidationResult {
  const errors: string[] = [];
  const warnings: string[] = [];

  if (!config.inputPath.trim()) {
    errors.push('Please specify a BeReal export ZIP archive or folder path.');
  }

  if (!config.outputPath.trim()) {
    errors.push('Please specify an output folder destination.');
  } else if (config.inputPath && config.outputPath.trim() === config.inputPath.trim()) {
    errors.push('Output directory cannot be identical to the input path.');
  }

  if (config.quality < 50 || config.quality > 100) {
    errors.push('Quality must be between 50% and 100%.');
  }

  if (archiveInfo) {
    if (archiveInfo.validationErrors?.length) {
      errors.push(...archiveInfo.validationErrors);
    }
    if (archiveInfo.warnings?.length) {
      warnings.push(...archiveInfo.warnings);
    }
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings,
  };
}

/**
 * Validate Recap Video configuration.
 */
export function validateRecapperConfig(
  config: RecapperConfig,
  archiveInfo?: ArchiveInfo | null
): ValidationResult {
  const errors: string[] = [];
  const warnings: string[] = [];

  if (!config.inputFolder.trim()) {
    errors.push('Please select a folder containing photos or a BeReal export.');
  }

  if (!config.musicPath.trim()) {
    errors.push('Please select a background audio track (MP3, WAV, M4A, AAC, FLAC, OGG).');
  }

  if (!config.outputPath.trim()) {
    errors.push('Please specify an output video destination path.');
  }

  if (config.startPadding < 0 || config.startPadding > 15) {
    warnings.push('Start hold duration is typically between 0s and 10s.');
  }

  if (config.endPadding < 0 || config.endPadding > 15) {
    warnings.push('End hold duration is typically between 0s and 10s.');
  }

  if (archiveInfo && archiveInfo.validationErrors?.length) {
    errors.push(...archiveInfo.validationErrors);
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings,
  };
}
