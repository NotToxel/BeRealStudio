use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn silent_command<S: AsRef<std::ffi::OsStr>>(program: S) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// Detect FFmpeg on PATH and return its path.
pub fn detect_ffmpeg() -> Result<PathBuf> {
    let cmd = if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" };
    let output = silent_command(cmd)
        .arg("-version")
        .output()
        .with_context(|| {
            "FFmpeg not found. Please install FFmpeg and ensure it is on your PATH.\n\
             Download: https://ffmpeg.org/download.html"
        })?;

    if output.status.success() {
        // Find the actual path via `where`/`which`
        let which_cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
        if let Ok(out) = silent_command(which_cmd).arg(cmd).output() {
            if let Ok(path_str) = String::from_utf8(out.stdout) {
                let path = PathBuf::from(path_str.lines().next().unwrap_or(cmd).trim());
                return Ok(path);
            }
        }
        Ok(PathBuf::from(cmd))
    } else {
        anyhow::bail!("FFmpeg found but returned an error. Please reinstall FFmpeg.");
    }
}

/// Combine two video files using Picture-in-Picture overlay via FFmpeg.
/// Secondary is scaled to 1/3.333, given rounded corners, placed at (55,55).
pub fn combine_videos_pip(
    primary: &Path,
    secondary: &Path,
    output: &Path,
    progress_cb: impl Fn(f32),
) -> Result<()> {
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let ffmpeg = detect_ffmpeg().context("FFmpeg required for video combining")?;

    // Exact proportional PIP overlay matching official BeReal layout
    let filter = format!(
        "[1:v]scale=main_w*0.3047:main_w*0.3047*4/3[pip_raw];\
         [pip_raw]format=rgba,geq=\
           r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':\
           a='if(gt(hypot(X-W/2,Y-H/2),hypot(W/2,H/2)*0.95),0,255)'[pip];\
         [0:v][pip]overlay=W*0.0378:W*0.0378[out]"
    );

    let status = silent_command(&ffmpeg)
        .args([
            "-i", primary.to_str().unwrap_or(""),
            "-i", secondary.to_str().unwrap_or(""),
            "-filter_complex", &filter,
            "-map", "[out]",
            "-map", "0:a?",
            "-c:v", "libx264",
            "-preset", "medium",
            "-crf", "23",
            "-c:a", "aac",
            "-shortest",
            "-y",
            output.to_str().unwrap_or(""),
        ])
        .status()
        .context("Failed to launch FFmpeg for video combining")?;

    if !status.success() {
        anyhow::bail!("FFmpeg video combine failed with exit code: {}", status);
    }
    progress_cb(1.0);
    Ok(())
}

/// Combine two video files side by side via FFmpeg.
pub fn combine_videos_side_by_side(
    primary: &Path,
    secondary: &Path,
    output: &Path,
) -> Result<()> {
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let ffmpeg = detect_ffmpeg().context("FFmpeg required for video combining")?;

    let filter = "[0:v][1:v]hstack=inputs=2[out]";

    let status = silent_command(&ffmpeg)
        .args([
            "-i", primary.to_str().unwrap_or(""),
            "-i", secondary.to_str().unwrap_or(""),
            "-filter_complex", filter,
            "-map", "[out]",
            "-map", "0:a?",
            "-c:v", "libx264",
            "-preset", "medium",
            "-crf", "23",
            "-c:a", "aac",
            "-shortest",
            "-y",
            output.to_str().unwrap_or(""),
        ])
        .status()
        .context("Failed to launch FFmpeg for side-by-side video combining")?;

    if !status.success() {
        anyhow::bail!("FFmpeg side-by-side video combine failed with exit code: {}", status);
    }
    Ok(())
}

/// Set video file metadata (creation date) via FFmpeg.
pub fn set_video_metadata(video_path: &Path, datetime: &chrono::DateTime<chrono::Utc>) -> Result<()> {
    // Set file timestamps
    let ts = filetime::FileTime::from_unix_time(datetime.timestamp(), 0);
    filetime::set_file_times(video_path, ts, ts)
        .context("Failed to set file timestamps")?;

    // Try FFmpeg metadata (optional — don't fail if FFmpeg not available)
    if let Ok(ffmpeg) = detect_ffmpeg() {
        let date_str = datetime.format("%Y-%m-%dT%H:%M:%S").to_string();
        let temp_out = video_path.with_extension("_meta_tmp.mp4");
        let status = silent_command(&ffmpeg)
            .args([
                "-i", video_path.to_str().unwrap_or(""),
                "-metadata", &format!("creation_time={}", date_str),
                "-metadata", &format!("date={}", date_str),
                "-c", "copy",
                "-y",
                temp_out.to_str().unwrap_or(""),
            ])
            .status();

        if let Ok(s) = status {
            if s.success() && temp_out.exists() {
                let _ = std::fs::rename(&temp_out, video_path);
            } else {
                let _ = std::fs::remove_file(&temp_out);
            }
        }
    }
    Ok(())
}

/// Get the duration of a video file in seconds using FFprobe.
pub fn get_video_duration(path: &Path) -> Result<f64> {
    let output = silent_command("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            path.to_str().unwrap_or(""),
        ])
        .output()
        .context("ffprobe not found")?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse ffprobe output")?;
    let dur_str = json["format"]["duration"]
        .as_str()
        .context("No duration in ffprobe output")?;
    dur_str.parse::<f64>().context("Cannot parse duration")
}
