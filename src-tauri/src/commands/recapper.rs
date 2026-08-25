use std::{path::Path, time::Instant};
use tauri::{AppHandle, State};

use crate::{
    pipeline::{
        date_filter,
        types::*,
        parser::parse_taken_at,
    },
    recapper::{
        audio,
        timing,
        geocoder,
        frame_renderer,
        video_encoder,
    },
    state::{AppState, ProgressEmitter},
};

/// Main Tauri command: run the Recapper video generation pipeline.
#[tauri::command]
pub async fn start_recapper(
    config: RecapperConfig,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ProcessingResult, String> {
    state.clear_abort();
    let log_buf = state.log_buffer.clone();
    let abort_flag = state.abort_flag.clone();
    let emitter = ProgressEmitter::new(app.clone(), log_buf, abort_flag, "recapper");

    run_recapper(config, emitter).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_recapper(state: State<'_, AppState>) -> Result<(), String> {
    state.request_abort();
    Ok(())
}

async fn run_recapper(
    config: RecapperConfig,
    emitter: ProgressEmitter,
) -> anyhow::Result<ProcessingResult> {
    let start = Instant::now();
    emitter.info("Starting BeReal Recapper...");

    // ── Step 1: Collect and filter images ────────────────────────────────────
    emitter.emit_progress(&ProgressEvent {
        stage: ProcessingStage::Scanning,
        current: 0,
        total: 0,
        percentage: 0.0,
        current_file: None,
    });

    let image_paths = date_filter::filter_images_by_date(
        Path::new(&config.input_folder),
        config.date_range_start.as_deref(),
        config.date_range_end.as_deref(),
    )?;

    if image_paths.is_empty() {
        anyhow::bail!("No images found in '{}' for the selected date range.", config.input_folder);
    }
    emitter.info(format!("Found {} images for the recap.", image_paths.len()));

    // ── Step 2: Load audio duration ───────────────────────────────────────────
    emitter.emit_progress(&ProgressEvent {
        stage: ProcessingStage::LoadingAudio,
        current: 0,
        total: 0,
        percentage: 5.0,
        current_file: None,
    });
    emitter.info("Analysing audio...");
    let audio_duration = audio::get_audio_duration(Path::new(&config.music_path))?;
    emitter.info(format!("Audio duration: {:.1}s", audio_duration));

    // ── Step 3: Calculate per-image durations ─────────────────────────────────
    let durations = timing::calculate_durations(
        audio_duration,
        image_paths.len(),
        config.start_padding,
        config.end_padding,
        &config.speed_mode,
    );

    // ── Step 4: Geocode if needed (sequential due to rate limit) ─────────────
    let total = image_paths.len();
    let mut location_strings: Vec<String> = vec![String::new(); total];

    if config.location_enabled {
        emitter.emit_progress(&ProgressEvent {
            stage: ProcessingStage::Geocoding,
            current: 0,
            total,
            percentage: 8.0,
            current_file: None,
        });
        emitter.info("Geocoding locations (this may take a while for large archives)...");

        for (i, img_path) in image_paths.iter().enumerate() {
            if emitter.is_aborted() { break; }
            // Try to read GPS from EXIF
            if let Some(loc_str) = read_gps_from_exif(img_path) {
                if let Some((lat, lon)) = parse_coords(&loc_str) {
                    let resolved = geocoder::resolve_location(lat, lon, &config.location_rules, &config.geocoding_mode);
                    location_strings[i] = resolved;
                }
            }
            let pct = 8.0 + (i as f32 / total as f32) * 12.0;
            emitter.emit_progress(&ProgressEvent {
                stage: ProcessingStage::Geocoding,
                current: i + 1,
                total,
                percentage: pct,
                current_file: img_path.file_name().map(|n| n.to_string_lossy().to_string()),
            });
        }
    }

    // ── Step 5: Render frames ─────────────────────────────────────────────────
    emitter.info("Rendering frames...");
    emitter.emit_progress(&ProgressEvent {
        stage: ProcessingStage::RenderingFrames,
        current: 0,
        total,
        percentage: 20.0,
        current_file: None,
    });

    let mut rendered_frames: Vec<(image::RgbImage, f64)> = Vec::with_capacity(total);

    for (i, (img_path, duration)) in image_paths.iter().zip(durations.iter()).enumerate() {
        if emitter.is_aborted() { break; }

        // Get date string from filename or EXIF
        let date_str = if config.date_enabled {
            extract_date_from_filename(img_path)
                .map(|dt| dt.format(&config.date_format).to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        let loc_str = &location_strings[i];

        match frame_renderer::render_frame(img_path, &config, &date_str, loc_str) {
            Ok(frame) => rendered_frames.push((frame, *duration)),
            Err(e) => emitter.warn(format!("Frame render error for {}: {}", img_path.display(), e)),
        }

        let pct = 20.0 + (i as f32 / total as f32) * 50.0;
        emitter.emit_progress(&ProgressEvent {
            stage: ProcessingStage::RenderingFrames,
            current: i + 1,
            total,
            percentage: pct,
            current_file: img_path.file_name().map(|n| n.to_string_lossy().to_string()),
        });
    }

    if emitter.is_aborted() {
        emitter.warn("Recapper cancelled.");
        return Ok(make_result(total, &config.output_path, start.elapsed().as_secs_f64()));
    }

    // ── Step 6: Encode video ──────────────────────────────────────────────────
    emitter.info("Encoding video...");
    let out_path = Path::new(&config.output_path);
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let n_frames = rendered_frames.len();
    video_encoder::encode_slideshow(
        &rendered_frames,
        Path::new(&config.music_path),
        out_path,
        &config,
        |pct| {
            let _ = emitter.emit_progress(&ProgressEvent {
                stage: ProcessingStage::EncodingVideo,
                current: (pct * n_frames as f32) as usize,
                total: n_frames,
                percentage: 70.0 + pct * 29.0,
                current_file: None,
            });
        },
    )?;

    emitter.info(format!(
        "Recap complete! Output: {} ({:.1}s)",
        config.output_path,
        start.elapsed().as_secs_f64()
    ));
    emitter.emit_progress(&ProgressEvent {
        stage: ProcessingStage::Complete,
        current: total,
        total,
        percentage: 100.0,
        current_file: None,
    });

    Ok(make_result(total, &config.output_path, start.elapsed().as_secs_f64()))
}

fn make_result(entries: usize, output_path: &str, duration: f64) -> ProcessingResult {
    ProcessingResult {
        entries_processed: entries,
        files_converted: entries,
        combined_created: 0,
        reversed_created: 0,
        motion_photos_created: 0,
        files_skipped: 0,
        errors: vec![],
        duration_secs: duration,
        output_path: output_path.to_string(),
    }
}

fn read_gps_from_exif(_path: &Path) -> Option<String> {
    // Try to read existing GPS from EXIF as a "lat,lon" string
    // Returns None if no GPS in EXIF (simplified — just try extracting from filename)
    None // Placeholder: GPS is typically in the output images from the toolkit phase
}

fn parse_coords(s: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() == 2 {
        let lat = parts[0].trim().parse::<f64>().ok()?;
        let lon = parts[1].trim().parse::<f64>().ok()?;
        return Some((lat, lon));
    }
    None
}

fn extract_date_from_filename(path: &Path) -> Option<chrono::DateTime<chrono::Utc>> {
    let stem = path.file_stem()?.to_str()?;
    if stem.len() < 19 { return None; }
    let prefix = &stem[..19];
    let normalized = format!("{}T{}Z", &prefix[..10], prefix[11..].replace('-', ":"));
    parse_taken_at(&normalized)
}
