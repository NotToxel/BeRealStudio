use anyhow::{Context, Result};
use image::RgbImage;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use crate::pipeline::types::RecapperConfig;
use crate::pipeline::video_ops::detect_ffmpeg;

/// Encode a slideshow by piping raw RGB frames into FFmpeg.
/// Durations control how many frames each image occupies.
pub fn encode_slideshow(
    frames: &[(RgbImage, f64)], // (rendered frame, duration in seconds)
    audio_path: &Path,
    output_path: &Path,
    config: &RecapperConfig,
    progress_cb: impl Fn(f32),
) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let ffmpeg = detect_ffmpeg().context("FFmpeg required for video encoding")?;
    let (width, height) = config.resolution;
    let fps = config.fps;

    // Calculate total frame count across all images
    let total_frames: u64 = frames
        .iter()
        .map(|(_, dur)| (dur * fps as f64).round() as u64)
        .sum();

    // Spawn FFmpeg: read rawvideo from stdin, audio from file
    let mut child = Command::new(&ffmpeg)
        .args([
            "-f", "rawvideo",
            "-pixel_format", "rgb24",
            "-video_size", &format!("{}x{}", width, height),
            "-framerate", &fps.to_string(),
            "-i", "pipe:0",                           // Video from stdin
            "-i", audio_path.to_str().unwrap_or(""),  // Audio file
            "-c:v", "libx264",
            "-preset", "medium",
            "-crf", "23",
            "-pix_fmt", "yuv420p",
            "-c:a", "aac",
            "-b:a", "192k",
            "-shortest",
            "-threads", "0",
            "-y",
            output_path.to_str().unwrap_or(""),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to launch FFmpeg for encoding")?;

    let stdin = child.stdin.as_mut().context("Failed to open FFmpeg stdin")?;

    let mut frames_written = 0u64;

    for (img, duration) in frames {
        let n_frames = (duration * fps as f64).round() as u64;
        let raw_bytes = img.as_raw();

        for _ in 0..n_frames {
            stdin
                .write_all(raw_bytes)
                .context("Failed to write frame to FFmpeg")?;
            frames_written += 1;

            let pct = frames_written as f32 / total_frames as f32;
            progress_cb(pct);
        }
    }

    // Close stdin to signal EOF to FFmpeg
    drop(child.stdin.take());

    let status = child.wait().context("FFmpeg encoding process failed")?;
    if !status.success() {
        anyhow::bail!("FFmpeg encoding failed with exit code: {}", status);
    }

    Ok(())
}
