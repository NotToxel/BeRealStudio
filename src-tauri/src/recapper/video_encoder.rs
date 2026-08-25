use anyhow::{Context, Result};
use image::RgbImage;
use std::{
    io::{BufWriter, Write},
    path::Path,
    process::{Command, Stdio},
};

use crate::pipeline::types::RecapperConfig;
use crate::pipeline::video_ops::detect_ffmpeg;

/// Encode a slideshow by streaming rendered RGB frames on-demand into FFmpeg stdin.
/// Memory usage is strictly bounded to a single active frame buffer (zero-copy buffer streaming).
pub fn encode_slideshow_streaming<F>(
    total_expected_frames: u64,
    mut frame_generator: F,
    audio_path: &Path,
    output_path: &Path,
    config: &RecapperConfig,
    progress_cb: impl Fn(f32),
) -> Result<()>
where
    F: FnMut() -> Result<Option<(RgbImage, f64)>>,
{
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let ffmpeg = detect_ffmpeg().context("FFmpeg required for video encoding")?;
    let (width, height) = config.resolution;
    let fps = config.fps;

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
        .context("Failed to launch FFmpeg for streaming encoding")?;

    let child_stdin = child.stdin.take().context("Failed to open FFmpeg stdin")?;
    let mut writer = BufWriter::with_capacity(1024 * 1024 * 4, child_stdin);

    let mut frames_written = 0u64;

    while let Some((img, duration)) = frame_generator()? {
        let n_frames = (duration * fps as f64).round() as u64;
        let raw_bytes = img.as_raw();

        for _ in 0..n_frames {
            writer
                .write_all(raw_bytes)
                .context("Failed to stream frame bytes to FFmpeg stdin")?;
            frames_written += 1;

            if total_expected_frames > 0 {
                let pct = (frames_written as f32 / total_expected_frames as f32).min(1.0);
                progress_cb(pct);
            }
        }
    }

    writer.flush().context("Failed to flush frame stream to FFmpeg")?;
    drop(writer);

    let status = child.wait().context("FFmpeg encoding process failed")?;
    if !status.success() {
        anyhow::bail!("FFmpeg streaming encoding failed with exit code: {}", status);
    }

    Ok(())
}

/// Encode a slideshow by piping raw RGB frames into FFmpeg (memory buffered compatibility).
pub fn encode_slideshow(
    frames: &[(RgbImage, f64)],
    audio_path: &Path,
    output_path: &Path,
    config: &RecapperConfig,
    progress_cb: impl Fn(f32),
) -> Result<()> {
    let fps = config.fps;
    let total_frames: u64 = frames
        .iter()
        .map(|(_, dur)| (dur * fps as f64).round() as u64)
        .sum();

    let mut idx = 0;
    encode_slideshow_streaming(
        total_frames,
        || {
            if idx < frames.len() {
                let (img, dur) = &frames[idx];
                idx += 1;
                Ok(Some((img.clone(), *dur)))
            } else {
                Ok(None)
            }
        },
        audio_path,
        output_path,
        config,
        progress_cb,
    )
}
