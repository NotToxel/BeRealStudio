use std::{path::Path, time::Instant};
use tauri::{AppHandle, State};

use crate::{
    pipeline::{
        date_filter,
        exif_writer,
        types::*,
        parser::parse_taken_at,
    },
    recapper::{
        audio,
        timing,
        geocoder,
        font_resolver,
        frame_renderer,
        video_encoder,
    },
    state::{AppState, ProgressEmitter},
};

/// Main Tauri command: run the Recapper video generation pipeline.
#[tauri::command]
pub async fn start_recapper(
    config: RecapperConfig,
    job_id: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ProcessingResult, String> {
    let log_buf = state.log_buffer.clone();
    let abort_flag = state.abort_flag.clone();

    let (emitter, jid_clean) = if let Some(jid) = job_id {
        let job_flag = state.register_job(&jid);
        let em = ProgressEmitter::with_job(app.clone(), log_buf, abort_flag, job_flag, jid.clone(), "recapper");
        (em, Some(jid))
    } else {
        state.clear_abort();
        let em = ProgressEmitter::new(app.clone(), log_buf, abort_flag, "recapper");
        (em, None)
    };

    let res = tauri::async_runtime::spawn_blocking(move || run_recapper(config, emitter))
        .await
        .map_err(|e| format!("Task execution failed: {}", e))?;

    if let Some(ref jid) = jid_clean {
        state.unregister_job(jid);
    }

    res.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_recapper(state: State<'_, AppState>) -> Result<(), String> {
    state.request_abort();
    Ok(())
}

fn run_recapper(
    config: RecapperConfig,
    emitter: ProgressEmitter,
) -> anyhow::Result<ProcessingResult> {
    let start = Instant::now();
    emitter.info("Starting BeReal Recapper...");

    // ── Step 1: Collect and filter images ────────────────────────────────────
    emitter.emit_progress(&ProgressEvent {
        job_id: None,
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
        job_id: None,
        stage: ProcessingStage::LoadingAudio,
        current: 0,
        total: 0,
        percentage: 5.0,
        current_file: None,
    });
    emitter.info("Analysing audio...");
    let raw_audio_duration = audio::get_audio_duration(Path::new(&config.music_path))?;
    let mut audio_duration = raw_audio_duration;
    if let Some(min_d) = config.min_duration_secs {
        if min_d > 0.0 && audio_duration < min_d {
            audio_duration = min_d;
        }
    }
    if let Some(max_d) = config.max_duration_secs {
        if max_d > 0.0 && audio_duration > max_d {
            audio_duration = max_d;
        }
    }
    emitter.info(format!("Slideshow target duration: {:.1}s (audio: {:.1}s)", audio_duration, raw_audio_duration));

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
            job_id: None,
            stage: ProcessingStage::Geocoding,
            current: 0,
            total,
            percentage: 8.0,
            current_file: None,
        });

        let is_offline = config.geocoding_mode == GeocodingMode::Offline;
        if is_offline {
            if let Err(e) = geocoder::load_spatial_grid(&emitter.app, None) {
                emitter.warn(format!("Could not pre-load offline geodata: {}", e));
            } else {
                let count = geocoder::spatial_grid_city_count();
                emitter.info(format!("Loaded offline GeoNames database ({} cities in RAM) for instantaneous reverse geocoding.", count));
            }
        } else {
            emitter.info("Using online Nominatim reverse geocoder (1 req/sec rate limit)...");
        }

        // 1. Batch extract GPS from all images in a single call using ExifTool
        let gps_map = if let Some(tool) = exif_writer::detect_exiftool() {
            emitter.info("Extracting GPS coordinates from image EXIF metadata...");
            exif_writer::extract_gps_batch(&tool, &image_paths)
        } else {
            std::collections::HashMap::new()
        };

        if !gps_map.is_empty() {
            emitter.info(format!("Found GPS metadata on {} of {} images.", gps_map.len(), total));
        }

        // 2. Resolve locations instantly (sub-millisecond RAM lookups)
        for (i, img_path) in image_paths.iter().enumerate() {
            if emitter.is_aborted() { break; }

            if let Some(&(lat, lon)) = gps_map.get(img_path) {
                let resolved = geocoder::resolve_location(lat, lon, &config.location_rules, &config.geocoding_mode, Some(&emitter.app));
                location_strings[i] = resolved;
            }

            let pct = 8.0 + (i as f32 / total as f32) * 12.0;
            emitter.emit_progress(&ProgressEvent {
                job_id: None,
                stage: ProcessingStage::Geocoding,
                current: i + 1,
                total,
                percentage: pct,
                current_file: img_path.file_name().map(|n| n.to_string_lossy().to_string()),
            });
        }
    }

    // ── Step 5 & 6: On-Demand Streaming Frame Rendering & Hardware Video Encoding ──
    let font = font_resolver::load_font(&config.font_path).unwrap_or_else(|_| {
        font_resolver::load_font("inter").expect("Default font must be available")
    });

    let out_path = Path::new(&config.output_path);
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("Cannot create directory '{}' ({}). Please check folder write permissions or select an alternative output directory.", parent.display(), e))?;
    }

    let fps = config.fps;
    let total_video_frames: u64 = durations
        .iter()
        .map(|dur| (dur * fps as f64).round() as u64)
        .sum();

    let ffmpeg_bin = crate::pipeline::video_ops::detect_ffmpeg()?;
    let (_enc_args, encoder_name) = video_encoder::detect_best_encoder(&ffmpeg_bin);
    emitter.info(format!("Encoding video with hardware encoder: {} (zero-copy memory streaming)...", encoder_name));

    emitter.emit_progress(&ProgressEvent {
        job_id: None,
        stage: ProcessingStage::RenderingFrames,
        current: 0,
        total,
        percentage: 20.0,
        current_file: None,
    });

    let mut current_img_idx = 0usize;
    let emitter_clone = emitter.clone();
    let res = video_encoder::encode_slideshow_streaming(
        total_video_frames,
        || {
            if emitter_clone.is_aborted() || current_img_idx >= total {
                return Ok(None);
            }
            let i = current_img_idx;
            current_img_idx += 1;

            let img_path = &image_paths[i];
            let duration = durations[i];

            let date_str = if config.date_enabled {
                extract_date_from_filename(img_path)
                    .map(|dt| dt.format(&config.date_format).to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            };
            let loc_str = &location_strings[i];

            let frame = match frame_renderer::render_frame_with_font(img_path, &config, &date_str, loc_str, &font) {
                Ok(f) => f,
                Err(e) => {
                    emitter_clone.warn(format!("Frame render error for {}: {}", img_path.display(), e));
                    let (w, h) = config.resolution;
                    image::RgbImage::new(w, h)
                }
            };

            let render_pct = 20.0 + (i as f32 / total as f32) * 55.0; // 20% -> 75%
            emitter_clone.emit_progress(&ProgressEvent {
                job_id: None,
                stage: ProcessingStage::RenderingFrames,
                current: i + 1,
                total,
                percentage: render_pct,
                current_file: img_path.file_name().map(|n| n.to_string_lossy().to_string()),
            });

            Ok(Some((frame, duration)))
        },
        Path::new(&config.music_path),
        out_path,
        &config,
        |stream_pct| {
            let encode_pct = 75.0 + stream_pct * 24.0; // 75% -> 99%
            let current_frames = (stream_pct * total_video_frames as f32) as usize;
            emitter_clone.emit_progress(&ProgressEvent {
                job_id: None,
                stage: ProcessingStage::EncodingVideo,
                current: current_frames,
                total: total_video_frames as usize,
                percentage: encode_pct,
                current_file: None,
            });
        },
    );

    if emitter.is_aborted() || res.is_err() {
        if out_path.exists() {
            let _ = std::fs::remove_file(out_path);
        }
        if emitter.is_aborted() {
            emitter.warn("Recap video generation cancelled by user.");
            anyhow::bail!("Recap video generation cancelled by user.");
        }
    }
    res?;

    emitter.info(format!(
        "Recap complete! Output: {} ({:.1}s)",
        config.output_path,
        start.elapsed().as_secs_f64()
    ));
    emitter.emit_progress(&ProgressEvent {
        job_id: None,
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
        live_photos_created: 0,
        files_skipped: 0,
        errors: vec![],
        duration_secs: duration,
        output_path: output_path.to_string(),
    }
}



fn extract_date_from_filename(path: &Path) -> Option<chrono::DateTime<chrono::Utc>> {
    let stem = path.file_stem()?.to_str()?;
    if stem.len() < 19 { return None; }
    let prefix = &stem[..19];
    let normalized = format!("{}T{}Z", &prefix[..10], prefix[11..].replace('-', ":"));
    parse_taken_at(&normalized)
}
