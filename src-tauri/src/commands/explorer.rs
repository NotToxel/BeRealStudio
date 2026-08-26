use anyhow::Result;
use chrono::{DateTime, Datelike, Local, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::{AppHandle, Manager};
use zip::ZipArchive;

use crate::{
    pipeline::{
        exif_writer, image_ops, motion_photo,
        parser::{self, parse_taken_at},
        types::{BeRealPost, Location, LocationRule, OutputFormat, RuleCondition},
    },
    recapper::geocoder,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplorerMemory {
    pub id: String,
    pub index: usize,
    pub taken_at: String,
    pub date_formatted: String,
    pub day_number: String,
    pub month_key: String,
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub time_formatted: String,
    pub is_late: bool,
    pub late_duration: Option<String>,
    pub late_exact: Option<String>,
    pub late_in_seconds: Option<i64>,
    pub retake_counter: u32,
    pub caption: Option<String>,
    pub location: Option<Location>,
    pub location_name: Option<String>,
    pub suburb: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub primary_path: Option<String>,
    pub secondary_path: Option<String>,
    pub bts_path: Option<String>,
    pub is_video: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub raw_json: Option<String>,
    pub debug_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplorerData {
    pub memories: Vec<ExplorerMemory>,
    pub total_count: usize,
    pub unique_years: Vec<i32>,
    pub unique_months: Vec<String>,
    pub unique_suburbs: Vec<String>,
    pub unique_cities: Vec<String>,
    pub unique_countries: Vec<String>,
    pub user_name: Option<String>,
    pub user_fullname: Option<String>,
    pub profile_picture_data_url: Option<String>,
    pub media_base_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSinglePostOptions {
    pub memory_index: usize,
    pub primary_path: String,
    pub secondary_path: Option<String>,
    pub bts_path: Option<String>,
    pub output_path: String,
    pub export_type: String, // "combined_pip", "combined_sidebyside", "primary_only", "secondary_only", "bts_only", "motion_photo"
    pub format: String,      // "Jpeg", "WebP", "Png"
    pub quality: u8,
    pub embed_exif: bool,
    pub taken_at: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub caption: Option<String>,
}

/// Command: Load and parse all memories for the Explorer view with parallel Rayon processing, smart media index, and offline geocoding.
#[tauri::command]
pub async fn load_explorer_memories(
    app: AppHandle,
    archive_path: String,
) -> Result<ExplorerData, String> {
    load_explorer_memories_inner(&app, &archive_path).map_err(|e| e.to_string())
}

/// Command: Read an image file as a Data URL for guaranteed, cross-platform webview rendering.
#[tauri::command]
pub async fn read_media_file_data_url(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpeg")
        .to_lowercase();

    let mime = match ext.as_str() {
        "webp" => "image/webp",
        "png" => "image/png",
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        _ => "image/jpeg",
    };

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

fn load_explorer_memories_inner(
    app: &AppHandle,
    archive_path_str: &str,
) -> Result<ExplorerData> {
    let input_path = Path::new(archive_path_str);
    if !input_path.exists() {
        anyhow::bail!("Selected archive or directory does not exist: {}", archive_path_str);
    }

    let (working_dir, all_raw_posts, user_name, user_fullname, profile_pic) = if input_path.is_file() {
        // ZIP archive -> cache extract to app cache dir
        let cache_root = app
            .path()
            .app_cache_dir()
            .unwrap_or_else(|_| PathBuf::from("./cache"))
            .join("bereal_explorer_cache");

        fs::create_dir_all(&cache_root)?;

        let hash = format!("{:x}", md5_digest(archive_path_str.as_bytes()));
        let dest_dir = cache_root.join(hash);
        fs::create_dir_all(&dest_dir)?;

        // Instant Cache Hit: Return cached explorer JSON if available, already geocoded, contains raw_json, and has evaluated late metadata
        let cache_json_file = dest_dir.join("explorer_cache.json");
        if cache_json_file.exists() {
            if let Ok(file) = File::open(&cache_json_file) {
                let reader = BufReader::with_capacity(128 * 1024, file);
                if let Ok(cached_data) = serde_json::from_reader::<_, ExplorerData>(reader) {
                    let has_valid_raw_json = cached_data.memories.first().and_then(|m| m.raw_json.as_ref()).is_some();
                    let has_late_evaluated = cached_data.memories.iter().any(|m| m.is_late || m.late_duration.is_some());
                    let needs_regeocode = cached_data.memories.iter().any(|m| {
                        m.location.is_some() && (m.city.is_none() || m.location_name.as_deref().unwrap_or("").contains('°'))
                    });

                    if has_valid_raw_json && has_late_evaluated && !needs_regeocode && !cached_data.memories.is_empty() {
                        return Ok(cached_data);
                    }
                }
            }
        }

        let memories_json_candidate = dest_dir.join("memories.json");
        let posts_json_candidate = dest_dir.join("posts.json");

        // Extract archive files into cache
        let zip_file = File::open(input_path)?;
        let mut archive = ZipArchive::new(zip_file)?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let name = entry.name().to_string();
            let norm = name.replace('\\', "/");
            let out_path = dest_dir.join(&norm);

            if entry.is_dir() {
                let _ = fs::create_dir_all(&out_path);
            } else {
                if !out_path.exists() || out_path.metadata().map(|m| m.len()).unwrap_or(0) == 0 {
                    if let Some(parent) = out_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if let Ok(mut out) = File::create(&out_path) {
                        let _ = std::io::copy(&mut entry, &mut out);
                    }
                }
            }
        }

        let archive_info = parser::scan_archive(&dest_dir.to_string_lossy())?;
        let memories_posts = if memories_json_candidate.exists() {
            parse_posts_from_path(&memories_json_candidate).unwrap_or_default()
        } else {
            Vec::new()
        };
        let posts_json_posts = if posts_json_candidate.exists() {
            parse_posts_from_path(&posts_json_candidate).unwrap_or_default()
        } else {
            Vec::new()
        };

        let raw_posts = if !memories_posts.is_empty() || !posts_json_posts.is_empty() {
            parser::merge_posts_and_memories(memories_posts, posts_json_posts)
        } else {
            let fallback_file = find_json_in_dir(&dest_dir)?;
            parse_posts_from_path(&fallback_file)?
        };

        (
            dest_dir,
            raw_posts,
            archive_info.user_name,
            archive_info.user_fullname,
            archive_info.profile_picture_data_url,
        )
    } else {
        // Directory
        let archive_info = parser::scan_archive(archive_path_str)?;
        let mem_cand = input_path.join("memories.json");
        let post_cand = input_path.join("posts.json");
        let memories_posts = if mem_cand.exists() {
            parse_posts_from_path(&mem_cand).unwrap_or_default()
        } else {
            Vec::new()
        };
        let posts_json_posts = if post_cand.exists() {
            parse_posts_from_path(&post_cand).unwrap_or_default()
        } else {
            Vec::new()
        };

        let raw_posts = if !memories_posts.is_empty() || !posts_json_posts.is_empty() {
            parser::merge_posts_and_memories(memories_posts, posts_json_posts)
        } else {
            let fallback_file = find_json_in_dir(input_path)?;
            parse_posts_from_path(&fallback_file)?
        };

        (
            input_path.to_path_buf(),
            raw_posts,
            archive_info.user_name,
            archive_info.user_fullname,
            archive_info.profile_picture_data_url,
        )
    };

    // Deduplicate posts by taken_at timestamp and media paths
    let mut seen_keys = HashSet::new();
    let mut posts = Vec::with_capacity(all_raw_posts.len());
    for (p, raw_json_str) in all_raw_posts {
        let prim_str = p.primary.as_ref().map(|a| a.path.as_str()).unwrap_or("");
        let sec_str = p.secondary.as_ref().map(|a| a.path.as_str()).unwrap_or("");
        let dedupe_key = format!("{}:{}:{}", p.taken_at, prim_str, sec_str);
        if seen_keys.insert(dedupe_key) {
            posts.push((p, raw_json_str));
        }
    }

    // Pre-build index of all media files in working_dir (lowercase filename -> path)
    let media_index = Arc::new(build_media_file_index(&working_dir));

    let default_rules = Arc::new(vec![LocationRule {
        comment: Some("Explorer default rule".into()),
        condition: RuleCondition::Default,
        format: "{city}, {country}".into(),
    }]);

    let working_dir_arc = Arc::new(working_dir.clone());

    // Process all posts in parallel across CPU cores with Rayon
    let memories: Vec<ExplorerMemory> = posts
        .par_iter()
        .enumerate()
        .map(|(idx, (post, raw_json_str))| {
            let dt = parse_taken_at(&post.taken_at).unwrap_or_else(Utc::now);
            let local_dt: DateTime<Local> = DateTime::from(dt);

            let year = local_dt.year();
            let month = local_dt.month();
            let day = local_dt.day();
            let month_key = format!("{:04}-{:02}", year, month);

            // Fast indexed media lookup
            let primary_path = post
                .primary
                .as_ref()
                .and_then(|a| resolve_indexed_media_path(a, &working_dir_arc, &media_index))
                .map(|p| p.to_string_lossy().to_string());

            let secondary_path = post
                .secondary
                .as_ref()
                .and_then(|a| resolve_indexed_media_path(a, &working_dir_arc, &media_index))
                .map(|p| p.to_string_lossy().to_string());

            let bts_path = post
                .bts_media
                .as_ref()
                .and_then(|a| resolve_indexed_media_path(a, &working_dir_arc, &media_index))
                .map(|p| p.to_string_lossy().to_string());

            let is_video = post
                .primary
                .as_ref()
                .map(|p| p.is_video())
                .unwrap_or(false);

            // Fast Offline Location & Geocoding
            let mut location_name = None;
            let mut suburb = None;
            let mut city = None;
            let mut country = None;

            if let Some(ref loc) = post.location {
                let resolved = geocoder::resolve_location(
                    loc.latitude,
                    loc.longitude,
                    &default_rules,
                    &crate::pipeline::types::GeocodingMode::Offline,
                    Some(app),
                );

                if !resolved.is_empty() {
                    location_name = Some(resolved.clone());
                    let parts: Vec<&str> = resolved.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                    if parts.len() >= 3 {
                        suburb = Some(parts[0].to_string());
                        city = Some(parts[1].to_string());
                        country = Some(parts[parts.len() - 1].to_string());
                    } else if parts.len() == 2 {
                        city = Some(parts[0].to_string());
                        country = Some(parts[1].to_string());
                    } else if parts.len() == 1 {
                        city = Some(parts[0].to_string());
                    }
                }
            }

            let date_formatted = local_dt.format("%d %B %Y").to_string();
            let day_number = format!("{}", day);
            let time_formatted = local_dt.format("%H:%M").to_string();

            let retake_counter = post.retake_counter.unwrap_or(0);
            
            // Calculate moment difference between notification time and actual post time
            let notif_dt = post.notification_at.as_deref().and_then(parser::parse_taken_at);
            let post_dt = parser::parse_taken_at(&post.taken_at);
            let moment_diff_sec = match (notif_dt, post_dt) {
                (Some(n), Some(p)) => {
                    let d = (p - n).num_seconds();
                    if d > 0 { Some(d) } else { None }
                }
                _ => None,
            };

            // A post is determined to be late when the boolean tag is true (or fallback if absent)
            let is_late = match post.is_late {
                Some(b) => b,
                None => {
                    if let Some(s) = post.late_in_seconds {
                        s > 0
                    } else if let Some(diff) = moment_diff_sec {
                        diff > 120
                    } else {
                        false
                    }
                }
            };

            let (late_duration, late_exact, late_seconds) = if is_late {
                let late_sec = moment_diff_sec
                    .or(post.late_in_seconds)
                    .unwrap_or(0)
                    .max(0);

                if late_sec > 0 {
                    let hrs = late_sec / 3600;
                    let mins = (late_sec % 3600) / 60;
                    let secs = late_sec % 60;

                    let dur_str = if hrs > 0 {
                        format!("{}h late", hrs)
                    } else if mins > 0 {
                        format!("{}m late", mins)
                    } else {
                        format!("{}s late", secs)
                    };

                    let exact_str = if hrs > 0 && mins > 0 {
                        format!("{} hr {} min late", hrs, mins)
                    } else if hrs > 0 {
                        format!("{} hr late", hrs)
                    } else if mins > 0 && secs > 0 {
                        format!("{} min {} sec late", mins, secs)
                    } else if mins > 0 {
                        format!("{} min late", mins)
                    } else {
                        format!("{} sec late", secs)
                    };

                    (Some(dur_str), Some(exact_str), Some(late_sec))
                } else {
                    (Some("Late".to_string()), Some("Posted after BeReal moment".to_string()), Some(0))
                }
            } else {
                (None, None, None)
            };

            let debug_info = format!(
                "is_late: {:?} | raw_is_late: {:?} | late_sec: {:?} | notif: {:?} | taken: {}",
                is_late, post.is_late, late_seconds, post.notification_at, post.taken_at
            );

            if idx < 10 || is_late {
                println!(
                    "[EXPLORER_DEBUG] Post #{}: taken_at='{}' raw_is_late={:?} raw_late_sec={:?} notif_at={:?} => IS_LATE={} late_dur={:?}",
                    idx, post.taken_at, post.is_late, post.late_in_seconds, post.notification_at, is_late, late_duration
                );
            }

            ExplorerMemory {
                id: format!("bereal-{}", idx),
                index: idx,
                taken_at: post.taken_at.clone(),
                date_formatted,
                day_number,
                month_key,
                year,
                month,
                day,
                time_formatted,
                is_late,
                late_duration,
                late_exact,
                late_in_seconds: late_seconds.or(post.late_in_seconds),
                retake_counter,
                caption: post.caption.clone(),
                location: post.location.clone(),
                location_name,
                suburb,
                city,
                country,
                primary_path,
                secondary_path,
                bts_path,
                is_video,
                width: post.primary.as_ref().and_then(|p| p.width),
                height: post.primary.as_ref().and_then(|p| p.height),
                raw_json: Some(raw_json_str.clone()),
                debug_info: Some(debug_info),
            }
        })
        .collect();

    let mut years_set = HashSet::new();
    let mut months_set = HashSet::new();
    let mut suburbs_set = HashSet::new();
    let mut cities_set = HashSet::new();
    let mut countries_set = HashSet::new();

    for m in &memories {
        years_set.insert(m.year);
        months_set.insert(m.month_key.clone());
        if let Some(sub) = &m.suburb {
            suburbs_set.insert(sub.clone());
        }
        if let Some(c) = &m.city {
            cities_set.insert(c.clone());
        }
        if let Some(ctry) = &m.country {
            countries_set.insert(ctry.clone());
        }
    }

    let mut unique_years: Vec<i32> = years_set.into_iter().collect();
    unique_years.sort();

    let mut unique_months: Vec<String> = months_set.into_iter().collect();
    unique_months.sort();

    let mut unique_suburbs: Vec<String> = suburbs_set.into_iter().collect();
    unique_suburbs.sort();

    let mut unique_cities: Vec<String> = cities_set.into_iter().collect();
    unique_cities.sort();

    let mut unique_countries: Vec<String> = countries_set.into_iter().collect();
    unique_countries.sort();

    let total_count = memories.len();

    let data = ExplorerData {
        memories,
        total_count,
        unique_years,
        unique_months,
        unique_suburbs,
        unique_cities,
        unique_countries,
        user_name,
        user_fullname,
        profile_picture_data_url: profile_pic,
        media_base_path: working_dir.to_string_lossy().to_string(),
    };

    // Cache computed data to disk for instant sub-millisecond retrieval on next launch
    let cache_file = working_dir.join("explorer_cache.json");
    if let Ok(serialized) = serde_json::to_string(&data) {
        let _ = fs::write(cache_file, serialized);
    }

    Ok(data)
}

/// Helper: Scan directory recursively and build a map of lowercase filename -> absolute PathBuf
fn build_media_file_index(root: &Path) -> HashMap<String, PathBuf> {
    let mut map = HashMap::new();
    collect_files_recursive(root, &mut map);
    map
}

fn collect_files_recursive(dir: &Path, map: &mut HashMap<String, PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                    map.insert(fname.to_lowercase(), path.clone());
                }
            } else if path.is_dir() {
                collect_files_recursive(&path, map);
            }
        }
    }
}

/// Helper: Resolve media path using fast O(1) filename index with fallbacks
fn resolve_indexed_media_path(
    asset: &crate::pipeline::types::MediaAsset,
    media_base: &Path,
    index: &HashMap<String, PathBuf>,
) -> Option<PathBuf> {
    let raw = asset.path.trim_start_matches('/').replace('\\', "/");
    let candidate = media_base.join(&raw);
    if candidate.exists() {
        return Some(candidate);
    }

    if let Some(fname) = Path::new(&raw).file_name().and_then(|n| n.to_str()) {
        let fname_lower = fname.to_lowercase();
        if let Some(indexed) = index.get(&fname_lower) {
            return Some(indexed.clone());
        }
    }

    // Standard fallback
    parser::resolve_media_path(asset, media_base)
}

/// Command: Export a single memory card to disk on demand (PIP, Side-by-Side, or single camera).
#[tauri::command]
pub async fn export_single_memory(
    _app: AppHandle,
    opts: ExportSinglePostOptions,
) -> Result<String, String> {
    export_single_memory_inner(opts)
        .map_err(|e| e.to_string())
}

fn export_single_memory_inner(opts: ExportSinglePostOptions) -> Result<String> {
    let out_dest = Path::new(&opts.output_path);
    if let Some(parent) = out_dest.parent() {
        fs::create_dir_all(parent)?;
    }

    let prim_path = Path::new(&opts.primary_path);
    if !prim_path.exists() {
        anyhow::bail!("Primary photo file does not exist: {}", opts.primary_path);
    }

    let dt = opts
        .taken_at
        .as_deref()
        .and_then(parse_taken_at)
        .unwrap_or_else(Utc::now);

    let location = if let (Some(latitude), Some(longitude)) = (opts.latitude, opts.longitude) {
        Some(Location {
            latitude,
            longitude,
        })
    } else {
        None
    };

    let fmt = match opts.format.to_lowercase().as_str() {
        "webp" => OutputFormat::WebP,
        "png" => OutputFormat::Png,
        _ => OutputFormat::Jpeg,
    };

    match opts.export_type.as_str() {
        "primary_only" => {
            image_ops::convert_image(prim_path, out_dest, &fmt, opts.quality)?;
            if opts.embed_exif && matches!(fmt, OutputFormat::Jpeg) {
                let _ = exif_writer::write_metadata(out_dest, &dt, location.as_ref(), opts.caption.as_deref());
            }
        }
        "secondary_only" => {
            if let Some(sec_str) = &opts.secondary_path {
                let sec_path = Path::new(sec_str);
                if sec_path.exists() {
                    image_ops::convert_image(sec_path, out_dest, &fmt, opts.quality)?;
                    if opts.embed_exif && matches!(fmt, OutputFormat::Jpeg) {
                        let _ = exif_writer::write_metadata(out_dest, &dt, location.as_ref(), opts.caption.as_deref());
                    }
                } else {
                    anyhow::bail!("Secondary photo not found: {}", sec_str);
                }
            } else {
                anyhow::bail!("No secondary photo available for this memory.");
            }
        }
        "combined_sidebyside" => {
            if let Some(sec_str) = &opts.secondary_path {
                let sec_path = Path::new(sec_str);
                if sec_path.exists() {
                    let combined = image_ops::combine_side_by_side(prim_path, sec_path)?;
                    let rgb = combined.to_rgb8();
                    image_ops::save_rgb_image(&rgb, out_dest, &fmt, opts.quality)?;
                    if opts.embed_exif && matches!(fmt, OutputFormat::Jpeg) {
                        let _ = exif_writer::write_metadata(out_dest, &dt, location.as_ref(), opts.caption.as_deref());
                    }
                } else {
                    image_ops::convert_image(prim_path, out_dest, &fmt, opts.quality)?;
                    if opts.embed_exif && matches!(fmt, OutputFormat::Jpeg) {
                        let _ = exif_writer::write_metadata(out_dest, &dt, location.as_ref(), opts.caption.as_deref());
                    }
                }
            }
        }
        "bts_only" => {
            if let Some(bts_str) = &opts.bts_path {
                let bts_path = Path::new(bts_str);
                if bts_path.exists() {
                    fs::copy(bts_path, out_dest)?;
                } else {
                    anyhow::bail!("BTS video file not found on disk: {}", bts_str);
                }
            } else {
                anyhow::bail!("No BTS video available for this memory.");
            }
        }
        "motion_photo" => {
            if let Some(sec_str) = &opts.secondary_path {
                let sec_path = Path::new(sec_str);
                if sec_path.exists() {
                    let combined = image_ops::combine_pip(prim_path, sec_path)?;
                    let rgb = combined.to_rgb8();
                    image_ops::save_rgb_image(&rgb, out_dest, &OutputFormat::Jpeg, opts.quality)?;
                } else {
                    image_ops::convert_image(prim_path, out_dest, &OutputFormat::Jpeg, opts.quality)?;
                }
            } else {
                image_ops::convert_image(prim_path, out_dest, &OutputFormat::Jpeg, opts.quality)?;
            }

            if opts.embed_exif {
                let _ = exif_writer::write_metadata(out_dest, &dt, location.as_ref(), opts.caption.as_deref());
            }

            if let Some(bts_str) = &opts.bts_path {
                let bts_path = Path::new(bts_str);
                if bts_path.exists() {
                    let _ = motion_photo::create_motion_photo(out_dest, bts_path);
                }
            }
        }
        _ => {
            if let Some(sec_str) = &opts.secondary_path {
                let sec_path = Path::new(sec_str);
                if sec_path.exists() {
                    let combined = image_ops::combine_pip(prim_path, sec_path)?;
                    let rgb = combined.to_rgb8();
                    image_ops::save_rgb_image(&rgb, out_dest, &fmt, opts.quality)?;
                    if opts.embed_exif && matches!(fmt, OutputFormat::Jpeg) {
                        let _ = exif_writer::write_metadata(out_dest, &dt, location.as_ref(), opts.caption.as_deref());
                    }
                } else {
                    image_ops::convert_image(prim_path, out_dest, &fmt, opts.quality)?;
                    if opts.embed_exif && matches!(fmt, OutputFormat::Jpeg) {
                        let _ = exif_writer::write_metadata(out_dest, &dt, location.as_ref(), opts.caption.as_deref());
                    }
                }
            } else {
                image_ops::convert_image(prim_path, out_dest, &fmt, opts.quality)?;
                if opts.embed_exif && matches!(fmt, OutputFormat::Jpeg) {
                    let _ = exif_writer::write_metadata(out_dest, &dt, location.as_ref(), opts.caption.as_deref());
                }
            }
        }
    }

    Ok(out_dest.to_string_lossy().to_string())
}

/// Helper: Find memories.json or posts.json in directory
fn find_json_in_dir(dir: &Path) -> Result<PathBuf> {
    // 1. Direct priority check: memories.json
    let memories = dir.join("memories.json");
    if memories.exists() {
        return Ok(memories);
    }

    // 2. Direct fallback check: posts.json
    let posts = dir.join("posts.json");
    if posts.exists() {
        return Ok(posts);
    }

    // Search immediate subdirectories for memories.json first
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.eq_ignore_ascii_case("memories.json") {
                        return Ok(path);
                    }
                }
            } else if path.is_dir() {
                let sub_memories = path.join("memories.json");
                if sub_memories.exists() {
                    return Ok(sub_memories);
                }
            }
        }
    }

    // Fallback: search for posts.json in subdirectories
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.eq_ignore_ascii_case("posts.json") {
                        return Ok(path);
                    }
                }
            } else if path.is_dir() {
                let sub_posts = path.join("posts.json");
                if sub_posts.exists() {
                    return Ok(sub_posts);
                }
            }
        }
    }

    anyhow::bail!("Could not find memories.json or posts.json in {}", dir.display())
}

/// Helper: Parse BeRealPost vector with raw JSON strings from a JSON file path
fn parse_posts_from_path(path: &Path) -> Result<Vec<(BeRealPost, String)>> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(128 * 1024, file);
    let root_val: serde_json::Value = serde_json::from_reader(reader)?;

    let raw_posts: Vec<serde_json::Value> = match root_val {
        serde_json::Value::Array(arr) => arr,
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Array(arr)) = map.get("posts").or_else(|| map.get("memories")).or_else(|| map.get("data")) {
                arr.clone()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

    let mut posts = Vec::with_capacity(raw_posts.len());
    for val in raw_posts {
        let raw_str = serde_json::to_string(&val).unwrap_or_default();
        if let Ok(mut p) = serde_json::from_value::<BeRealPost>(val.clone()) {
            if p.taken_at.is_empty() {
                if let Some(ref d) = p.date {
                    p.taken_at = d.clone();
                }
            }
            // Check direct raw field fallbacks if serde left something as None
            if p.is_late.is_none() {
                if let Some(val_late) = val.get("isLate")
                    .or_else(|| val.get("is_late"))
                    .or_else(|| val.get("late"))
                    .or_else(|| val.get("wasLate"))
                    .or_else(|| val.get("isLatePost"))
                {
                    if let Some(b) = val_late.as_bool() {
                        p.is_late = Some(b);
                    } else if let Some(i) = val_late.as_i64() {
                        p.is_late = Some(i != 0);
                    } else if let Some(s) = val_late.as_str() {
                        p.is_late = Some(s.eq_ignore_ascii_case("true") || s == "1");
                    }
                }
            }

            if p.late_in_seconds.is_none() {
                if let Some(val_sec) = val.get("lateInSeconds")
                    .or_else(|| val.get("late_in_seconds"))
                    .or_else(|| val.get("secondsLate"))
                    .or_else(|| val.get("lateSeconds"))
                    .or_else(|| val.get("lateTime"))
                {
                    if let Some(i) = val_sec.as_i64() {
                        p.late_in_seconds = Some(i);
                    } else if let Some(s) = val_sec.as_str() {
                        p.late_in_seconds = s.trim().parse::<i64>().ok();
                    }
                }
            }

            if p.notification_at.is_none() {
                if let Some(val_notif) = val.get("notificationAt")
                    .or_else(|| val.get("momentAt"))
                    .or_else(|| val.get("notification_at"))
                    .or_else(|| val.get("berealMoment"))
                    .or_else(|| val.get("notificationDate"))
                {
                    if let Some(s) = val_notif.as_str() {
                        p.notification_at = Some(s.to_string());
                    }
                }
            }

            posts.push((p, raw_str));
        }
    }
    Ok(posts)
}

fn md5_digest(data: &[u8]) -> u128 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish() as u128
}
