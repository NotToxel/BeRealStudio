use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{atomic::{AtomicUsize, Ordering}, Arc},
    time::Instant,
};

use chrono::DateTime;
use rayon::prelude::*;
use tauri::{AppHandle, State};
use zip::ZipArchive;

use crate::{
    pipeline::{
        cleanup,
        date_filter,
        exif_writer,
        image_ops,
        motion_photo,
        parser,
        types::*,
    },
    state::{AppState, ProgressEmitter},
};

/// Main Tauri command: run the photo toolkit processing pipeline.
#[tauri::command]
pub async fn start_toolkit(
    config: ToolkitConfig,
    job_id: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ProcessingResult, String> {
    let log_buf = state.log_buffer.clone();
    let abort_flag = state.abort_flag.clone();

    let (emitter, jid_clean) = if let Some(jid) = job_id {
        let job_flag = state.register_job(&jid);
        let em = ProgressEmitter::with_job(app.clone(), log_buf, abort_flag, job_flag, jid.clone(), "toolkit");
        (em, Some(jid))
    } else {
        state.clear_abort();
        let em = ProgressEmitter::new(app.clone(), log_buf, abort_flag, "toolkit");
        (em, None)
    };

    let res = run_toolkit(config, emitter).await;

    if let Some(ref jid) = jid_clean {
        state.unregister_job(jid);
    }

    res.map_err(|e| e.to_string())
}

/// Check whether running this toolkit export will actually overwrite any existing files on disk.
#[tauri::command]
pub async fn check_toolkit_conflicts(config: ToolkitConfig) -> Result<DestinationStatus, String> {
    let out_base = Path::new(&config.output_path);
    if !out_base.exists() {
        return Ok(DestinationStatus {
            exists: false,
            is_directory: true,
            is_file: false,
            file_count: 0,
        });
    }

    let input_path = Path::new(&config.input_path);
    if !input_path.exists() {
        return Ok(DestinationStatus {
            exists: false,
            is_directory: true,
            is_file: false,
            file_count: 0,
        });
    }

    let is_zip = input_path.is_file()
        || input_path.extension().map(|e| e.eq_ignore_ascii_case("zip")).unwrap_or(false);

    let all_posts = if is_zip {
        if let Ok(file) = File::open(input_path) {
            if let Ok(mut archive) = ZipArchive::new(file) {
                let mut found_posts = Vec::new();
                for i in 0..archive.len() {
                    if let Ok(mut entry) = archive.by_index(i) {
                        let name = entry.name().to_string();
                        if name.ends_with("posts.json") || name.ends_with("memories.json") {
                            let mut buf = Vec::new();
                            if std::io::Read::read_to_end(&mut entry, &mut buf).is_ok() {
                                if let Ok(posts) = serde_json::from_slice::<Vec<BeRealPost>>(&buf) {
                                    found_posts = posts;
                                    break;
                                }
                            }
                        }
                    }
                }
                found_posts
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        let json_path = input_path.join("posts.json");
        let alt_json_path = input_path.join("memories.json");
        if json_path.exists() {
            parser::parse_posts(&json_path).unwrap_or_default()
        } else if alt_json_path.exists() {
            parser::parse_posts(&alt_json_path).unwrap_or_default()
        } else {
            Vec::new()
        }
    };

    let posts = date_filter::filter_by_date_range(
        all_posts,
        config.date_range_start.as_deref(),
        config.date_range_end.as_deref(),
    );

    let dir_singles = out_base.join("singles");
    let dir_combined = out_base.join("combined");
    let dir_reversed = out_base.join("combined_reversed");

    let mut conflicting_files_count = 0;
    let ext = config.convert_format.extension();

    for post in &posts {
        let dt = match DateTime::parse_from_rfc3339(&post.taken_at) {
            Ok(d) => d.with_timezone(&chrono::Utc),
            Err(_) => match chrono::NaiveDateTime::parse_from_str(&post.taken_at, "%Y-%m-%dT%H:%M:%S%.fZ") {
                Ok(ndt) => DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc),
                Err(_) => continue,
            },
        };
        let time_str = dt.format("%Y-%m-%dT%H-%M-%S").to_string();

        // 1. Singles
        if post.primary.is_some() {
            let p_ext = if post.primary.as_ref().map(|p| p.is_video()).unwrap_or(false) { "mp4" } else { ext };
            if dir_singles.join(format!("{}_primary.{}", time_str, p_ext)).exists() {
                conflicting_files_count += 1;
            }
        }
        if post.secondary.is_some() {
            let s_ext = if post.secondary.as_ref().map(|s| s.is_video()).unwrap_or(false) { "mp4" } else { ext };
            if dir_singles.join(format!("{}_secondary.{}", time_str, s_ext)).exists() {
                conflicting_files_count += 1;
            }
        }
        if post.bts_media.is_some() {
            if dir_singles.join(format!("{}_bts.mp4", time_str)).exists() {
                conflicting_files_count += 1;
            }
        }

        // 2. Combined
        if config.create_combined {
            if dir_combined.join(format!("{}_combined.{}", time_str, ext)).exists()
                || dir_combined.join(format!("{}_combined.mp4", time_str)).exists()
            {
                conflicting_files_count += 1;
            }
        }

        // 3. Reversed
        if config.create_reversed {
            if dir_reversed.join(format!("{}_combined_reversed.{}", time_str, ext)).exists()
                || dir_reversed.join(format!("{}_combined_reversed.mp4", time_str)).exists()
            {
                conflicting_files_count += 1;
            }
        }
    }

    Ok(DestinationStatus {
        exists: conflicting_files_count > 0,
        is_directory: true,
        is_file: false,
        file_count: conflicting_files_count,
    })
}

/// Cancel an in-progress toolkit run.
#[tauri::command]
pub async fn cancel_toolkit(state: State<'_, AppState>) -> Result<(), String> {
    state.request_abort();
    Ok(())
}

async fn run_toolkit(
    config: ToolkitConfig,
    emitter: ProgressEmitter,
) -> anyhow::Result<ProcessingResult> {
    let start = Instant::now();
    emitter.info("Starting BeReal Studio photo processing...");

    let input_path = Path::new(&config.input_path);
    if !input_path.exists() {
        anyhow::bail!("Input path does not exist: {}", config.input_path);
    }

    let is_zip = input_path.is_file()
        || input_path.extension().map(|e| e.eq_ignore_ascii_case("zip")).unwrap_or(false);

    // If input is a ZIP archive, automatically extract it safely to a temporary working folder
    let (working_dir, is_temp_dir): (PathBuf, bool) = if is_zip {
        emitter.emit_progress(&ProgressEvent {
            job_id: None,
            stage: ProcessingStage::Extracting,
            current: 0,
            total: 0,
            percentage: 1.0,
            current_file: None,
        });
        emitter.info(format!(
            "Extracting ZIP archive '{}' to working directory...",
            input_path.file_name().unwrap_or_default().to_string_lossy()
        ));

        let temp_dir = std::env::temp_dir().join(format!(
            "bereal_studio_extract_{}_{}",
            std::process::id(),
            Instant::now().elapsed().as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir)?;

        let file = File::open(input_path)?;
        let mut archive = ZipArchive::new(file)?;
        let total_entries = archive.len();

        for i in 0..total_entries {
            if emitter.is_aborted() {
                let _ = std::fs::remove_dir_all(&temp_dir);
                anyhow::bail!("Processing cancelled by user during extraction.");
            }

            let mut entry = archive.by_index(i)?;
            let outpath = match entry.enclosed_name() {
                Some(p) => temp_dir.join(p),
                None => continue,
            };

            if entry.name().ends_with('/') || entry.name().ends_with('\\') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut out = File::create(&outpath)?;
                std::io::copy(&mut entry, &mut out)?;
            }

            if i % 30 == 0 || i == total_entries - 1 {
                let pct = 1.0 + (i as f32 / total_entries.max(1) as f32) * 5.0;
                emitter.emit_progress(&ProgressEvent {
            job_id: None,
                    stage: ProcessingStage::Extracting,
                    current: i + 1,
                    total: total_entries,
                    percentage: pct,
                    current_file: Some(entry.name().to_string()),
                });
            }
        }

        emitter.info(format!("Extracted {} items from ZIP archive.", total_entries));
        (temp_dir, true)
    } else {
        (input_path.to_path_buf(), false)
    };

    // ── Stage 1: Parse JSON ───────────────────────────────────────────────────
    emitter.emit_progress(&ProgressEvent {
        job_id: None,
        stage: ProcessingStage::Scanning,
        current: 0,
        total: 0,
        percentage: 6.0,
        current_file: None,
    });
    emitter.info("Scanning BeReal export directory...");

    let posts_json = match parser::find_posts_json(&working_dir) {
        Ok(p) => p,
        Err(e) => {
            if is_temp_dir {
                let _ = std::fs::remove_dir_all(&working_dir);
            }
            return Err(e);
        }
    };
    let media_base = posts_json.parent().unwrap_or(&working_dir).to_path_buf();

    emitter.emit_progress(&ProgressEvent {
        job_id: None,
        stage: ProcessingStage::Parsing,
        current: 0,
        total: 0,
        percentage: 7.0,
        current_file: None,
    });
    emitter.info("Parsing posts.json...");

    let all_posts = match parser::parse_posts(&posts_json) {
        Ok(p) => p,
        Err(e) => {
            if is_temp_dir {
                let _ = std::fs::remove_dir_all(&working_dir);
            }
            return Err(e);
        }
    };
    emitter.info(format!("Found {} BeReal entries.", all_posts.len()));

    // ── Stage 2: Date filter ──────────────────────────────────────────────────
    let posts = date_filter::filter_by_date_range(
        all_posts,
        config.date_range_start.as_deref(),
        config.date_range_end.as_deref(),
    );
    if config.date_range_start.is_some() || config.date_range_end.is_some() {
        emitter.info(format!("{} entries after date filtering.", posts.len()));
    }

    if posts.is_empty() {
        emitter.warn("No entries matched the selected date range.");
        if is_temp_dir {
            let _ = std::fs::remove_dir_all(&working_dir);
        }
        return Ok(ProcessingResult {
            entries_processed: 0,
            files_converted: 0,
            combined_created: 0,
            reversed_created: 0,
            motion_photos_created: 0,
            files_skipped: 0,
            errors: vec![],
            duration_secs: start.elapsed().as_secs_f64(),
            output_path: config.output_path.clone(),
        });
    }

    // ── Stage 3: Set up output directories ───────────────────────────────────
    let out_base = Path::new(&config.output_path);
    let dir_singles = out_base.join("singles");
    let dir_combined = out_base.join("combined");
    let dir_reversed = out_base.join("combined_reversed");
    std::fs::create_dir_all(&dir_singles)
        .map_err(|e| anyhow::anyhow!("Cannot create directory '{}' ({}). Please check folder write permissions or select an alternative output directory.", dir_singles.display(), e))?;
    if config.create_combined {
        std::fs::create_dir_all(&dir_combined)
            .map_err(|e| anyhow::anyhow!("Cannot create directory '{}' ({}). Please check folder write permissions or select an alternative output directory.", dir_combined.display(), e))?;
    }
    if config.create_reversed {
        std::fs::create_dir_all(&dir_reversed)
            .map_err(|e| anyhow::anyhow!("Cannot create directory '{}' ({}). Please check folder write permissions or select an alternative output directory.", dir_reversed.display(), e))?;
    }
    emitter.info(format!("Output: {}", out_base.display()));

    // ── Stage 4: Process each entry (parallel) ───────────────────────────────
    let total = posts.len();
    let counter = Arc::new(AtomicUsize::new(0));
    let errors: Arc<std::sync::Mutex<Vec<String>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let files_converted = Arc::new(AtomicUsize::new(0));
    let combined_created = Arc::new(AtomicUsize::new(0));
    let reversed_created = Arc::new(AtomicUsize::new(0));
    let motion_photos_created = Arc::new(AtomicUsize::new(0));
    let files_skipped = Arc::new(AtomicUsize::new(0));
    // Collect pairs for combination phase
    let pairs: Arc<std::sync::Mutex<Vec<(PathBuf, PathBuf, DateTime<chrono::Utc>, Option<Location>, Option<String>, Option<PathBuf>)>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));

    emitter.info("Processing individual files...");
    emitter.emit_progress(&ProgressEvent {
        job_id: None,
        stage: ProcessingStage::Converting,
        current: 0,
        total,
        percentage: 5.0,
        current_file: None,
    });

    // Process in parallel with Rayon
    posts.par_iter().for_each(|post| {
        if emitter.is_aborted() { return; }

        let taken_at_str = &post.taken_at;
        let dt = match parser::parse_taken_at(taken_at_str) {
            Some(dt) => dt,
            None => {
                if let Ok(mut e) = errors.lock() {
                    e.push(format!("Cannot parse date '{}', skipping entry.", taken_at_str));
                }
                files_skipped.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };
        let time_str = dt.format("%Y-%m-%dT%H-%M-%S").to_string();

        let mut prim_out: Option<PathBuf> = None;
        let mut sec_out: Option<PathBuf> = None;
        let mut bts_out: Option<PathBuf> = None;

        for (asset_opt, role) in [
            (&post.primary, "primary"),
            (&post.secondary, "secondary"),
            (&post.bts_media, "bts"),
        ] {
            let asset = match asset_opt {
                Some(a) => a,
                None => continue,
            };

            let src = match parser::resolve_media_path(asset, &media_base) {
                Some(p) => p,
                None => {
                    let msg = format!("File not found: {}", Path::new(&asset.path).file_name().unwrap_or_default().to_string_lossy());
                    emitter.warn(&msg);
                    files_skipped.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
            };

            let filename_base = if config.keep_original_filename {
                let orig = src.file_stem().unwrap_or_default().to_string_lossy();
                format!("{}_{}_{}.", time_str, role, orig)
            } else {
                format!("{}_{}", time_str, role)
            };

            let result: anyhow::Result<PathBuf> = if asset.is_video() {
                let ext = src.extension().unwrap_or_default().to_string_lossy();
                let dest = dir_singles.join(format!("{}.{}", filename_base, ext));
                std::fs::copy(&src, &dest)
                    .map(|_| {
                        let ft = filetime::FileTime::from_unix_time(dt.timestamp(), 0);
                        let _ = filetime::set_file_times(&dest, ft, ft);
                        if config.embed_exif {
                            let _ = exif_writer::write_metadata(&dest, &dt, post.location.as_ref(), post.caption.as_deref());
                        }
                        dest.clone()
                    })
                    .map_err(|e| e.into())
            } else {
                let ext = config.convert_format.extension();
                let dest = dir_singles.join(format!("{}.{}", filename_base, ext));
                image_ops::convert_image(&src, &dest, &config.convert_format, config.quality)
                    .map(|_| {
                        let ft = filetime::FileTime::from_unix_time(dt.timestamp(), 0);
                        let _ = filetime::set_file_times(&dest, ft, ft);
                        if config.embed_exif && matches!(config.convert_format, OutputFormat::Jpeg) {
                            if let Err(e) = exif_writer::write_metadata(&dest, &dt, post.location.as_ref(), post.caption.as_deref()) {
                                emitter.warn(format!("EXIF write error for {}: {}", dest.display(), e));
                            }
                        }
                        dest.clone()
                    })
            };

            match result {
                Ok(out_path) => {
                    files_converted.fetch_add(1, Ordering::Relaxed);
                    match role {
                        "primary" => prim_out = Some(out_path),
                        "secondary" => sec_out = Some(out_path),
                        "bts" => bts_out = Some(out_path),
                        _ => {}
                    }
                }
                Err(e) => {
                    if let Ok(mut errs) = errors.lock() {
                        errs.push(format!("Error processing {}: {}", src.display(), e));
                    }
                    files_skipped.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        // Collect pairs for combination phase
        if let (Some(p), Some(s)) = (prim_out, sec_out) {
            if let Ok(mut pairs_lock) = pairs.lock() {
                pairs_lock.push((p, s, dt, post.location.clone(), post.caption.clone(), bts_out));
            }
        }

        let done = counter.fetch_add(1, Ordering::Relaxed) + 1;
        let pct = 5.0 + (done as f32 / total as f32) * 60.0;
        emitter.emit_progress(&ProgressEvent {
            job_id: None,
            stage: ProcessingStage::Converting,
            current: done,
            total,
            percentage: pct,
            current_file: Some(time_str.clone()),
        });
    });

    if emitter.is_aborted() {
        if is_temp_dir {
            let _ = std::fs::remove_dir_all(&working_dir);
        }
        emitter.warn("Processing cancelled by user.");
        anyhow::bail!("Photo processing cancelled by user.");
    }

    // ── Stage 5: Create combined images ──────────────────────────────────────
    let pairs_vec = pairs.lock().unwrap().clone();
    let combo_total = pairs_vec.len();

    if (config.create_combined || config.create_reversed) && !pairs_vec.is_empty() {
        emitter.info("Creating combined images in parallel...");
        let combo_counter = Arc::new(AtomicUsize::new(0));

        emitter.emit_progress(&ProgressEvent {
            job_id: None,
            stage: ProcessingStage::Compositing,
            current: 0,
            total: combo_total,
            percentage: 65.0,
            current_file: None,
        });

        pairs_vec.par_iter().for_each(|(prim_path, sec_path, dt, location, caption, bts_path)| {
            if emitter.is_aborted() {
                return;
            }

            let timestamp = dt.format("%Y-%m-%dT%H-%M-%S").to_string();

            if config.create_combined {
                let ext = config.convert_format.extension();
                let dest = dir_combined.join(format!("{}_combined.{}", timestamp, ext));
                match combine_and_save(prim_path, sec_path, &dest, &config, dt, location.as_ref(), caption.as_deref()) {
                    Ok(_) => {
                        combined_created.fetch_add(1, Ordering::Relaxed);
                        // Motion photo
                        if config.create_motion_photos {
                            if let Some(bts) = bts_path {
                                if bts.exists() && matches!(config.convert_format, OutputFormat::Jpeg) {
                                    match motion_photo::create_motion_photo(&dest, bts) {
                                        Ok(_) => { motion_photos_created.fetch_add(1, Ordering::Relaxed); }
                                        Err(e) => emitter.warn(format!("Motion photo failed for {}: {}", dest.display(), e)),
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if let Ok(mut errs) = errors.lock() {
                            errs.push(format!("Combined image error: {}", e));
                        }
                    }
                }
            }

            if config.create_reversed {
                let ext = config.convert_format.extension();
                let dest = dir_reversed.join(format!("{}_combined_reversed.{}", timestamp, ext));
                if let Err(e) = combine_and_save(sec_path, prim_path, &dest, &config, dt, location.as_ref(), caption.as_deref()) {
                    if let Ok(mut errs) = errors.lock() {
                        errs.push(format!("Reversed combined error: {}", e));
                    }
                } else {
                    reversed_created.fetch_add(1, Ordering::Relaxed);
                }
            }

            let done = combo_counter.fetch_add(1, Ordering::Relaxed) + 1;
            let pct = 65.0 + (done as f32 / combo_total as f32) * 25.0;
            emitter.emit_progress(&ProgressEvent {
                job_id: None,
                stage: ProcessingStage::Compositing,
                current: done,
                total: combo_total,
                percentage: pct,
                current_file: Some(timestamp),
            });
        });
    }

    // ── Stage 6: Cleanup ──────────────────────────────────────────────────────
    if config.cleanup_intermediates {
        emitter.info("Cleaning up intermediate files...");
        let _ = cleanup::cleanup_intermediates(&dir_combined, &config.convert_format);
        let _ = cleanup::cleanup_intermediates(&dir_reversed, &config.convert_format);
        let _ = cleanup::remove_backup_files(&dir_singles);
        let _ = cleanup::remove_backup_files(&dir_combined);
        let _ = cleanup::remove_backup_files(&dir_reversed);
    }

    if is_temp_dir {
        let _ = std::fs::remove_dir_all(&working_dir);
    }

    let result = make_result(
        counter.load(Ordering::Relaxed),
        files_converted.load(Ordering::Relaxed),
        combined_created.load(Ordering::Relaxed),
        reversed_created.load(Ordering::Relaxed),
        motion_photos_created.load(Ordering::Relaxed),
        files_skipped.load(Ordering::Relaxed),
        errors.lock().unwrap().clone(),
        start.elapsed().as_secs_f64(),
        &config.output_path,
    );

    emitter.info(format!(
        "Complete! Processed: {}, Combined: {}, Skipped: {}, Errors: {} ({:.1}s)",
        result.entries_processed,
        result.combined_created,
        result.files_skipped,
        result.errors.len(),
        result.duration_secs,
    ));
    emitter.emit_progress(&ProgressEvent {
        job_id: None,
        stage: ProcessingStage::Complete,
        current: total,
        total,
        percentage: 100.0,
        current_file: None,
    });

    Ok(result)
}

fn combine_and_save(
    primary: &Path,
    secondary: &Path,
    dest: &Path,
    config: &ToolkitConfig,
    dt: &DateTime<chrono::Utc>,
    location: Option<&Location>,
    caption: Option<&str>,
) -> anyhow::Result<()> {
    let combined = match config.combine_mode {
        CombineMode::PictureInPicture => image_ops::combine_pip(primary, secondary)?,
        CombineMode::SideBySide => image_ops::combine_side_by_side(primary, secondary)?,
    };
    let rgb = combined.to_rgb8();
    image_ops::save_rgb_image(&rgb, dest, &config.convert_format, config.quality)?;

    let ft = filetime::FileTime::from_unix_time(dt.timestamp(), 0);
    let _ = filetime::set_file_times(dest, ft, ft);

    if config.embed_exif && matches!(config.convert_format, OutputFormat::Jpeg) {
        let _ = exif_writer::write_metadata(dest, dt, location, caption);
    }
    Ok(())
}

fn make_result(
    entries: usize,
    converted: usize,
    combined: usize,
    reversed: usize,
    motion: usize,
    skipped: usize,
    errors: Vec<String>,
    duration: f64,
    output_path: &str,
) -> ProcessingResult {
    ProcessingResult {
        entries_processed: entries,
        files_converted: converted,
        combined_created: combined,
        reversed_created: reversed,
        motion_photos_created: motion,
        files_skipped: skipped,
        errors,
        duration_secs: duration,
        output_path: output_path.to_string(),
    }
}
