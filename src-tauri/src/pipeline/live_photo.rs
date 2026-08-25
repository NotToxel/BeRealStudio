use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::pipeline::{
    exif_writer,
    types::Location,
    video_ops,
};

/// Create an Apple Photos compatible Live Photo pair (.jpg + .mov)
/// with matching Apple Content Identifier UUIDs in the MakerNote and QuickTime container.
pub fn create_apple_live_photo_pair(
    composite_image_path: &Path,
    bts_video_path: &Path,
    output_dir: &Path,
    base_name: &str,
    datetime: &DateTime<Utc>,
    location: Option<&Location>,
    caption: Option<&str>,
) -> Result<(PathBuf, PathBuf)> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create live photo directory: {}", output_dir.display()))?;
    }

    // 1. Generate unique Apple Asset Identifier (UUID v4 in uppercase)
    let asset_id = uuid::Uuid::new_v4().to_string().to_uppercase();

    // 2. Prepare destination paths (.jpg and .mov with matching stem)
    let dest_jpg = output_dir.join(format!("{}.jpg", base_name));
    let dest_mov = output_dir.join(format!("{}.mov", base_name));

    // 3. Copy image to live_photos destination
    std::fs::copy(composite_image_path, &dest_jpg)
        .with_context(|| format!("Failed to copy composite image to {}", dest_jpg.display()))?;

    // 4. Inject Apple MakerNote Tag 17 (ContentIdentifier) into the JPEG
    exif_writer::write_metadata_with_apple_id(
        &dest_jpg,
        datetime,
        location,
        caption,
        Some(&asset_id),
    ).with_context(|| format!("Failed to write Apple Live Photo metadata to {}", dest_jpg.display()))?;

    // 5. Re-mux BTS video into QuickTime (.mov) container with com.apple.quicktime.content.identifier
    if let Ok(ffmpeg) = video_ops::detect_ffmpeg() {
        let status = Command::new(&ffmpeg)
            .args([
                "-i", bts_video_path.to_str().unwrap_or(""),
                "-c", "copy",
                "-movflags", "use_metadata_tags",
                "-metadata:g", &format!("com.apple.quicktime.content.identifier={}", asset_id),
                "-f", "mov",
                "-y",
                dest_mov.to_str().unwrap_or(""),
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                // If ExifTool is available, also set ContentIdentifier tag on MOV for maximum compatibility
                if let Some(exiftool) = exif_writer::detect_exiftool() {
                    let _ = exif_writer::write_metadata_exiftool(
                        &exiftool,
                        &dest_mov,
                        datetime,
                        location,
                        caption,
                        true,
                        Some(&asset_id),
                    );
                }
            }
            _ => {
                // Fallback: copy video file directly and set timestamps
                let _ = std::fs::copy(bts_video_path, &dest_mov);
            }
        }
    } else {
        // Fallback if FFmpeg is unavailable
        std::fs::copy(bts_video_path, &dest_mov)
            .with_context(|| format!("Failed to copy BTS video to {}", dest_mov.display()))?;
    }

    // 6. Synchronize filesystem timestamps on both files
    let ft = filetime::FileTime::from_unix_time(datetime.timestamp(), 0);
    let _ = filetime::set_file_times(&dest_jpg, ft, ft);
    let _ = filetime::set_file_times(&dest_mov, ft, ft);

    Ok((dest_jpg, dest_mov))
}
