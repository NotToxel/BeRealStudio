use anyhow::{Context, Result};
use image::RgbImage;
use std::{
    io::{BufWriter, Write},
    path::Path,
    process::{Command, Stdio},
};

use crate::pipeline::types::RecapperConfig;
use crate::pipeline::video_ops::detect_ffmpeg;

/// Detect the best available video encoder (NVIDIA NVENC, Intel QSV, AMD AMF, Apple VideoToolbox, or CPU libx264).
pub fn detect_best_encoder(ffmpeg: &Path) -> (Vec<String>, &'static str) {
    if let Ok(output) = Command::new(ffmpeg).args(["-hide_banner", "-encoders"]).output() {
        let text = String::from_utf8_lossy(&output.stdout);
        // 1. NVIDIA NVENC GPU
        if text.contains("h264_nvenc") {
            return (
                vec![
                    "-c:v".into(), "h264_nvenc".into(),
                    "-preset".into(), "p4".into(),
                    "-cq".into(), "23".into(),
                ],
                "NVIDIA NVENC (GPU)",
            );
        }
        // 2. Apple VideoToolbox GPU
        if text.contains("h264_videotoolbox") {
            return (
                vec![
                    "-c:v".into(), "h264_videotoolbox".into(),
                    "-q:v".into(), "65".into(),
                ],
                "Apple VideoToolbox (GPU)",
            );
        }
        // 3. Intel QuickSync GPU
        if text.contains("h264_qsv") {
            return (
                vec![
                    "-c:v".into(), "h264_qsv".into(),
                    "-global_quality".into(), "23".into(),
                ],
                "Intel QuickSync (GPU)",
            );
        }
        // 4. AMD AMF GPU
        if text.contains("h264_amf") {
            return (
                vec![
                    "-c:v".into(), "h264_amf".into(),
                    "-quality".into(), "balanced".into(),
                ],
                "AMD AMF (GPU)",
            );
        }
    }

    // High-performance CPU fallback with multi-threaded libx264
    (
        vec![
            "-c:v".into(), "libx264".into(),
            "-preset".into(), "veryfast".into(),
            "-crf".into(), "23".into(),
            "-threads".into(), "0".into(),
        ],
        "libx264 (Multi-threaded CPU)",
    )
}

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

    let (encoder_args, encoder_name) = detect_best_encoder(&ffmpeg);
    log::info!("🎬 Recapper video encoder selected: {}", encoder_name);

    let mut base_cmd = Command::new(&ffmpeg);
    base_cmd.args([
        "-f", "rawvideo",
        "-pixel_format", "rgb24",
        "-video_size", &format!("{}x{}", width, height),
        "-framerate", &fps.to_string(),
        "-i", "pipe:0",                           // Video from stdin
        "-i", audio_path.to_str().unwrap_or(""),  // Audio file
    ]);
    base_cmd.args(&encoder_args);
    base_cmd.args([
        "-pix_fmt", "yuv420p",
        "-c:a", "aac",
        "-b:a", "192k",
        "-shortest",
        "-y",
        output_path.to_str().unwrap_or(""),
    ]);

    let mut child = base_cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to launch FFmpeg for streaming encoding")?;

    let child_stdin = child.stdin.take().context("Failed to open FFmpeg stdin")?;
    let mut writer = BufWriter::with_capacity(1024 * 1024 * 4, child_stdin);

    let mut frames_written = 0u64;

    while let Some((img, duration)) = frame_generator()? {
        let n_frames = (duration * fps as f64).round() as u64;
        let raw_bytes = img.as_raw();

        for _ in 0..n_frames {
            if let Err(e) = writer.write_all(raw_bytes) {
                // If FFmpeg exited early, drop writer and wait for output to extract actual FFmpeg error log
                drop(writer);
                if let Ok(out) = child.wait_with_output() {
                    let err_tail = String::from_utf8_lossy(&out.stderr);
                    let last_lines = err_tail.lines().rev().take(8).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
                    anyhow::bail!("FFmpeg aborted early: {}\n{}", e, last_lines);
                }
                return Err(e).context("Failed to stream frame bytes to FFmpeg stdin");
            }
            frames_written += 1;

            if total_expected_frames > 0 {
                let pct = (frames_written as f32 / total_expected_frames as f32).min(1.0);
                progress_cb(pct);
            }
        }
    }

    let _ = writer.flush();
    drop(writer);

    let output = child.wait_with_output().context("FFmpeg encoding process failed")?;
    if !output.status.success() {
        let err_tail = String::from_utf8_lossy(&output.stderr);
        let last_lines = err_tail.lines().rev().take(8).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
        anyhow::bail!("FFmpeg encoding failed ({}):\n{}", output.status, last_lines);
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
