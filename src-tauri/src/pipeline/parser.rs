use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

use crate::pipeline::types::{ArchiveInfo, BeRealPost, MediaAsset, MonthCount};

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserExportMeta {
    username: Option<String>,
    fullname: Option<String>,
}

/// Scan an input path (ZIP archive or unzipped directory) and return detailed BeReal archive metadata & validation.
pub fn scan_archive(path: &str) -> Result<ArchiveInfo> {
    let input_path = Path::new(path);
    if !input_path.exists() {
        return Ok(ArchiveInfo {
            is_valid: false,
            archive_type: if path.ends_with(".zip") { "Zip".into() } else { "Directory".into() },
            user_name: None,
            user_fullname: None,
            entry_count: 0,
            valid_post_count: 0,
            corrupted_post_count: 0,
            total_media_count: 0,
            found_media_count: 0,
            missing_media_count: 0,
            missing_files_sample: Vec::new(),
            earliest_date: None,
            latest_date: None,
            has_posts_json: false,
            has_photos_dir: false,
            has_user_json: false,
            has_videos: false,
            has_bts: false,
            monthly_histogram: Vec::new(),
            validation_errors: vec![format!("Path does not exist on disk: {}", path)],
            warnings: Vec::new(),
            posts_json_path: String::new(),
            media_base_path: String::new(),
        });
    }

    let is_zip = input_path.is_file()
        || input_path.extension().map(|e| e.eq_ignore_ascii_case("zip")).unwrap_or(false);

    if is_zip {
        scan_zip_archive(input_path)
    } else {
        scan_directory_archive(input_path)
    }
}

/// Scan a BeReal export ZIP file in-memory.
fn scan_zip_archive(zip_path: &Path) -> Result<ArchiveInfo> {
    let file = match File::open(zip_path) {
        Ok(f) => f,
        Err(e) => {
            return Ok(ArchiveInfo {
                is_valid: false,
                archive_type: "Zip".into(),
                user_name: None,
                user_fullname: None,
                entry_count: 0,
                valid_post_count: 0,
                corrupted_post_count: 0,
                total_media_count: 0,
                found_media_count: 0,
                missing_media_count: 0,
                missing_files_sample: Vec::new(),
                earliest_date: None,
                latest_date: None,
                has_posts_json: false,
                has_photos_dir: false,
                has_user_json: false,
                has_videos: false,
                has_bts: false,
                monthly_histogram: Vec::new(),
                validation_errors: vec![format!("Could not open ZIP file: {}", e)],
                warnings: Vec::new(),
                posts_json_path: String::new(),
                media_base_path: zip_path.to_string_lossy().to_string(),
            });
        }
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            return Ok(ArchiveInfo {
                is_valid: false,
                archive_type: "Zip".into(),
                user_name: None,
                user_fullname: None,
                entry_count: 0,
                valid_post_count: 0,
                corrupted_post_count: 0,
                total_media_count: 0,
                found_media_count: 0,
                missing_media_count: 0,
                missing_files_sample: Vec::new(),
                earliest_date: None,
                latest_date: None,
                has_posts_json: false,
                has_photos_dir: false,
                has_user_json: false,
                has_videos: false,
                has_bts: false,
                monthly_histogram: Vec::new(),
                validation_errors: vec![format!("Invalid or corrupted ZIP archive: {}", e)],
                warnings: Vec::new(),
                posts_json_path: String::new(),
                media_base_path: zip_path.to_string_lossy().to_string(),
            });
        }
    };

    let mut posts_entry_name: Option<String> = None;
    let mut user_entry_name: Option<String> = None;
    let mut zip_filenames = HashSet::new();
    let mut zip_entries_by_filename: HashMap<String, String> = HashMap::new();
    let mut has_photos_dir = false;

    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_string();
            let norm = name.replace('\\', "/");
            let norm_lower = norm.to_lowercase();

            if norm_lower.ends_with("posts.json") && posts_entry_name.is_none() {
                posts_entry_name = Some(name.clone());
            }
            if norm_lower.ends_with("user.json") && user_entry_name.is_none() {
                user_entry_name = Some(name.clone());
            }
            if norm_lower.contains("photos/") || norm_lower.contains("post/") || norm_lower.contains("bereal/") {
                has_photos_dir = true;
            }

            if let Some(fname) = Path::new(&norm).file_name() {
                let fname_str = fname.to_string_lossy().to_string();
                zip_entries_by_filename.insert(fname_str.to_lowercase(), norm.clone());
            }
            zip_filenames.insert(norm_lower);
        }
    }

    let mut validation_errors = Vec::new();
    let mut warnings = Vec::new();

    let posts_name = match posts_entry_name {
        Some(name) => name,
        None => {
            validation_errors.push(
                "Missing 'posts.json': The ZIP archive does not contain the required BeReal posts data file. Please ensure this is an official BeReal GDPR export."
                    .to_string(),
            );
            if !has_photos_dir {
                validation_errors.push(
                    "Missing photos directory: No 'Photos' or media folder was detected in this ZIP archive."
                        .to_string(),
                );
            }
            return Ok(ArchiveInfo {
                is_valid: false,
                archive_type: "Zip".into(),
                user_name: None,
                user_fullname: None,
                entry_count: 0,
                valid_post_count: 0,
                corrupted_post_count: 0,
                total_media_count: 0,
                found_media_count: 0,
                missing_media_count: 0,
                missing_files_sample: Vec::new(),
                earliest_date: None,
                latest_date: None,
                has_posts_json: false,
                has_photos_dir,
                has_user_json: user_entry_name.is_some(),
                has_videos: false,
                has_bts: false,
                monthly_histogram: Vec::new(),
                validation_errors,
                warnings,
                posts_json_path: String::new(),
                media_base_path: zip_path.to_string_lossy().to_string(),
            });
        }
    };

    // Read posts.json from zip
    let mut posts_data = String::new();
    if let Ok(mut entry) = archive.by_name(&posts_name) {
        if let Err(e) = entry.read_to_string(&mut posts_data) {
            validation_errors.push(format!("Failed to read 'posts.json' from ZIP: {}", e));
        }
    } else {
        validation_errors.push("Failed to access 'posts.json' in ZIP archive.".to_string());
    }

    if !validation_errors.is_empty() {
        return Ok(ArchiveInfo {
            is_valid: false,
            archive_type: "Zip".into(),
            user_name: None,
            user_fullname: None,
            entry_count: 0,
            valid_post_count: 0,
            corrupted_post_count: 0,
            total_media_count: 0,
            found_media_count: 0,
            missing_media_count: 0,
            missing_files_sample: Vec::new(),
            earliest_date: None,
            latest_date: None,
            has_posts_json: true,
            has_photos_dir,
            has_user_json: user_entry_name.is_some(),
            has_videos: false,
            has_bts: false,
            monthly_histogram: Vec::new(),
            validation_errors,
            warnings,
            posts_json_path: posts_name,
            media_base_path: zip_path.to_string_lossy().to_string(),
        });
    }

    // Parse user.json if present
    let has_user_json = user_entry_name.is_some();
    let mut user_name = None;
    let mut user_fullname = None;
    if let Some(ref user_name_entry) = user_entry_name {
        if let Ok(mut entry) = archive.by_name(user_name_entry) {
            let mut user_data = String::new();
            if entry.read_to_string(&mut user_data).is_ok() {
                if let Ok(user) = serde_json::from_str::<UserExportMeta>(&user_data) {
                    user_name = user.username;
                    user_fullname = user.fullname;
                }
            }
        }
    }

    // Parse posts.json
    let raw_posts: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&posts_data);
    let raw_posts = match raw_posts {
        Ok(p) => p,
        Err(e) => {
            return Ok(ArchiveInfo {
                is_valid: false,
                archive_type: "Zip".into(),
                user_name: user_name.clone(),
                user_fullname,
                entry_count: 0,
                valid_post_count: 0,
                corrupted_post_count: 0,
                total_media_count: 0,
                found_media_count: 0,
                missing_media_count: 0,
                missing_files_sample: Vec::new(),
                earliest_date: None,
                latest_date: None,
                has_posts_json: true,
                has_photos_dir,
                has_user_json,
                has_videos: false,
                has_bts: false,
                monthly_histogram: Vec::new(),
                validation_errors: vec![format!("'posts.json' is not valid JSON or not an array: {}", e)],
                warnings,
                posts_json_path: posts_name,
                media_base_path: zip_path.to_string_lossy().to_string(),
            });
        }
    };

    let mut valid_posts = Vec::with_capacity(raw_posts.len());
    let mut corrupted_posts = 0usize;

    for (i, val) in raw_posts.into_iter().enumerate() {
        match serde_json::from_value::<BeRealPost>(val) {
            Ok(post) => valid_posts.push(post),
            Err(e) => {
                corrupted_posts += 1;
                log::warn!("Skipping malformed post #{}: {}", i, e);
            }
        }
    }

    if valid_posts.is_empty() {
        validation_errors.push(
            "No valid BeReal memory records could be parsed from posts.json. The file structure does not match the expected format."
                .to_string(),
        );
    }

    // Check media files inside ZIP
    let (total_media, found_media, missing_media, missing_sample) =
        check_zip_media_presence(&valid_posts, &zip_filenames, &zip_entries_by_filename);

    if total_media > 0 && found_media == 0 {
        validation_errors.push(
            "All media files referenced in posts.json are missing from the ZIP archive. Check if the export finished downloading completely."
                .to_string(),
        );
    } else if missing_media > 0 {
        warnings.push(format!(
            "{} out of {} referenced media file(s) are missing from the archive.",
            missing_media, total_media
        ));
    }

    if corrupted_posts > 0 {
        warnings.push(format!(
            "{} entry/entries in posts.json contained malformed data and will be skipped.",
            corrupted_posts
        ));
    }

    let histogram = compute_monthly_histogram(&valid_posts);
    let mut has_videos = false;
    let mut has_bts = false;
    let mut earliest: Option<DateTime<Utc>> = None;
    let mut latest: Option<DateTime<Utc>> = None;

    for post in &valid_posts {
        if let Some(dt) = parse_taken_at(&post.taken_at) {
            match earliest {
                None => earliest = Some(dt),
                Some(e) if dt < e => earliest = Some(dt),
                _ => {}
            }
            match latest {
                None => latest = Some(dt),
                Some(l) if dt > l => latest = Some(dt),
                _ => {}
            }
        }
        if let Some(p) = &post.primary {
            if p.is_video() { has_videos = true; }
        }
        if let Some(s) = &post.secondary {
            if s.is_video() { has_videos = true; }
        }
        if post.bts_media.is_some() { has_bts = true; }
    }

    let is_valid = validation_errors.is_empty() && !valid_posts.is_empty();

    Ok(ArchiveInfo {
        is_valid,
        archive_type: "Zip".into(),
        user_name,
        user_fullname,
        entry_count: valid_posts.len(),
        valid_post_count: valid_posts.len(),
        corrupted_post_count: corrupted_posts,
        total_media_count: total_media,
        found_media_count: found_media,
        missing_media_count: missing_media,
        missing_files_sample: missing_sample,
        earliest_date: earliest.map(|d| d.format("%Y-%m-%d").to_string()),
        latest_date: latest.map(|d| d.format("%Y-%m-%d").to_string()),
        has_posts_json: true,
        has_photos_dir,
        has_user_json,
        has_videos,
        has_bts,
        monthly_histogram: histogram,
        validation_errors,
        warnings,
        posts_json_path: posts_name,
        media_base_path: zip_path.to_string_lossy().to_string(),
    })
}

/// Scan a BeReal export extracted directory.
fn scan_directory_archive(base_dir: &Path) -> Result<ArchiveInfo> {
    let posts_json_res = find_posts_json(base_dir);
    let mut validation_errors = Vec::new();
    let mut warnings = Vec::new();

    let posts_json_path = match posts_json_res {
        Ok(p) => p,
        Err(_) => {
            validation_errors.push(format!(
                "Missing 'posts.json': Could not find posts.json in '{}'. Make sure you selected the unzipped BeReal export folder.",
                base_dir.display()
            ));
            let has_photos = base_dir.join("Photos").exists() || base_dir.join("photos").exists();
            return Ok(ArchiveInfo {
                is_valid: false,
                archive_type: "Directory".into(),
                user_name: None,
                user_fullname: None,
                entry_count: 0,
                valid_post_count: 0,
                corrupted_post_count: 0,
                total_media_count: 0,
                found_media_count: 0,
                missing_media_count: 0,
                missing_files_sample: Vec::new(),
                earliest_date: None,
                latest_date: None,
                has_posts_json: false,
                has_photos_dir: has_photos,
                has_user_json: false,
                has_videos: false,
                has_bts: false,
                monthly_histogram: Vec::new(),
                validation_errors,
                warnings,
                posts_json_path: String::new(),
                media_base_path: base_dir.to_string_lossy().to_string(),
            });
        }
    };

    let media_base = posts_json_path.parent().unwrap_or(base_dir).to_path_buf();
    let has_photos_dir = media_base.join("Photos").exists()
        || media_base.join("photos").exists()
        || base_dir.join("Photos").exists()
        || base_dir.join("photos").exists();

    // Check user.json
    let mut user_name = None;
    let mut user_fullname = None;
    let user_json_candidates = [
        media_base.join("user.json"),
        base_dir.join("user.json"),
        media_base.join("User.json"),
    ];
    let mut has_user_json = false;
    for u_path in &user_json_candidates {
        if u_path.exists() {
            has_user_json = true;
            if let Ok(data) = std::fs::read_to_string(u_path) {
                if let Ok(user) = serde_json::from_str::<UserExportMeta>(&data) {
                    user_name = user.username;
                    user_fullname = user.fullname;
                    break;
                }
            }
        }
    }

    // Parse posts.json
    let data = match std::fs::read_to_string(&posts_json_path) {
        Ok(d) => d,
        Err(e) => {
            validation_errors.push(format!("Failed to read {}: {}", posts_json_path.display(), e));
            return Ok(ArchiveInfo {
                is_valid: false,
                archive_type: "Directory".into(),
                user_name,
                user_fullname,
                entry_count: 0,
                valid_post_count: 0,
                corrupted_post_count: 0,
                total_media_count: 0,
                found_media_count: 0,
                missing_media_count: 0,
                missing_files_sample: Vec::new(),
                earliest_date: None,
                latest_date: None,
                has_posts_json: true,
                has_photos_dir,
                has_user_json,
                has_videos: false,
                has_bts: false,
                monthly_histogram: Vec::new(),
                validation_errors,
                warnings,
                posts_json_path: posts_json_path.to_string_lossy().to_string(),
                media_base_path: media_base.to_string_lossy().to_string(),
            });
        }
    };

    let raw_posts: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&data);
    let raw_posts = match raw_posts {
        Ok(p) => p,
        Err(e) => {
            validation_errors.push(format!("'posts.json' is not valid JSON: {}", e));
            return Ok(ArchiveInfo {
                is_valid: false,
                archive_type: "Directory".into(),
                user_name,
                user_fullname,
                entry_count: 0,
                valid_post_count: 0,
                corrupted_post_count: 0,
                total_media_count: 0,
                found_media_count: 0,
                missing_media_count: 0,
                missing_files_sample: Vec::new(),
                earliest_date: None,
                latest_date: None,
                has_posts_json: true,
                has_photos_dir,
                has_user_json,
                has_videos: false,
                has_bts: false,
                monthly_histogram: Vec::new(),
                validation_errors,
                warnings,
                posts_json_path: posts_json_path.to_string_lossy().to_string(),
                media_base_path: media_base.to_string_lossy().to_string(),
            });
        }
    };

    let mut valid_posts = Vec::with_capacity(raw_posts.len());
    let mut corrupted_posts = 0usize;

    for (i, val) in raw_posts.into_iter().enumerate() {
        match serde_json::from_value::<BeRealPost>(val) {
            Ok(post) => valid_posts.push(post),
            Err(e) => {
                corrupted_posts += 1;
                log::warn!("Skipping malformed post #{}: {}", i, e);
            }
        }
    }

    if valid_posts.is_empty() {
        validation_errors.push(
            "No valid BeReal memory records could be found in posts.json. The file structure does not match expected BeReal data."
                .to_string(),
        );
    }

    // Check media presence on disk
    let (total_media, found_media, missing_media, missing_sample) =
        check_disk_media_presence(&valid_posts, &media_base);

    if total_media > 0 && found_media == 0 {
        validation_errors.push(
            "All media files referenced in posts.json are missing on disk. Check that the 'Photos' folder was extracted alongside posts.json."
                .to_string(),
        );
    } else if missing_media > 0 {
        warnings.push(format!(
            "{} out of {} referenced media file(s) could not be found on disk.",
            missing_media, total_media
        ));
    }

    if corrupted_posts > 0 {
        warnings.push(format!(
            "{} entry/entries in posts.json contained malformed data and will be skipped.",
            corrupted_posts
        ));
    }

    let histogram = compute_monthly_histogram(&valid_posts);
    let mut has_videos = false;
    let mut has_bts = false;
    let mut earliest: Option<DateTime<Utc>> = None;
    let mut latest: Option<DateTime<Utc>> = None;

    for post in &valid_posts {
        if let Some(dt) = parse_taken_at(&post.taken_at) {
            match earliest {
                None => earliest = Some(dt),
                Some(e) if dt < e => earliest = Some(dt),
                _ => {}
            }
            match latest {
                None => latest = Some(dt),
                Some(l) if dt > l => latest = Some(dt),
                _ => {}
            }
        }
        if let Some(p) = &post.primary {
            if p.is_video() { has_videos = true; }
        }
        if let Some(s) = &post.secondary {
            if s.is_video() { has_videos = true; }
        }
        if post.bts_media.is_some() { has_bts = true; }
    }

    let is_valid = validation_errors.is_empty() && !valid_posts.is_empty();

    Ok(ArchiveInfo {
        is_valid,
        archive_type: "Directory".into(),
        user_name,
        user_fullname,
        entry_count: valid_posts.len(),
        valid_post_count: valid_posts.len(),
        corrupted_post_count: corrupted_posts,
        total_media_count: total_media,
        found_media_count: found_media,
        missing_media_count: missing_media,
        missing_files_sample: missing_sample,
        earliest_date: earliest.map(|d| d.format("%Y-%m-%d").to_string()),
        latest_date: latest.map(|d| d.format("%Y-%m-%d").to_string()),
        has_posts_json: true,
        has_photos_dir,
        has_user_json,
        has_videos,
        has_bts,
        monthly_histogram: histogram,
        validation_errors,
        warnings,
        posts_json_path: posts_json_path.to_string_lossy().to_string(),
        media_base_path: media_base.to_string_lossy().to_string(),
    })
}

fn check_zip_media_presence(
    posts: &[BeRealPost],
    zip_filenames: &HashSet<String>,
    zip_entries_by_filename: &HashMap<String, String>,
) -> (usize, usize, usize, Vec<String>) {
    let mut total = 0usize;
    let mut found = 0usize;
    let mut missing = 0usize;
    let mut missing_sample = Vec::new();

    for post in posts {
        let assets = [
            &post.primary,
            &post.primary_placeholder,
            &post.secondary,
            &post.secondary_placeholder,
            &post.bts_media,
        ];
        for asset_opt in assets {
            if let Some(asset) = asset_opt {
                total += 1;
                let raw = asset.path.trim_start_matches('/').replace('\\', "/").to_lowercase();
                let filename = Path::new(&raw)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_lowercase())
                    .unwrap_or_default();

                let is_present = zip_filenames.contains(&raw)
                    || (!filename.is_empty() && zip_entries_by_filename.contains_key(&filename));

                if is_present {
                    found += 1;
                } else {
                    missing += 1;
                    if missing_sample.len() < 10 {
                        missing_sample.push(asset.path.clone());
                    }
                }
            }
        }
    }

    (total, found, missing, missing_sample)
}

fn check_disk_media_presence(
    posts: &[BeRealPost],
    media_base: &Path,
) -> (usize, usize, usize, Vec<String>) {
    let mut total = 0usize;
    let mut found = 0usize;
    let mut missing = 0usize;
    let mut missing_sample = Vec::new();

    for post in posts {
        let assets = [
            &post.primary,
            &post.primary_placeholder,
            &post.secondary,
            &post.secondary_placeholder,
            &post.bts_media,
        ];
        for asset_opt in assets {
            if let Some(asset) = asset_opt {
                total += 1;
                if resolve_media_path(asset, media_base).is_some() {
                    found += 1;
                } else {
                    missing += 1;
                    if missing_sample.len() < 10 {
                        missing_sample.push(asset.path.clone());
                    }
                }
            }
        }
    }

    (total, found, missing, missing_sample)
}

/// Find posts.json by searching common locations in the archive directory.
pub fn find_posts_json(base: &Path) -> Result<PathBuf> {
    // Direct: base/posts.json
    let direct = base.join("posts.json");
    if direct.exists() {
        return Ok(direct);
    }
    // Direct case-insensitive
    let direct_upper = base.join("Posts.json");
    if direct_upper.exists() {
        return Ok(direct_upper);
    }
    // One or two levels deep
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let candidate = p.join("posts.json");
                if candidate.exists() {
                    return Ok(candidate);
                }
                let candidate_upper = p.join("Posts.json");
                if candidate_upper.exists() {
                    return Ok(candidate_upper);
                }
                // Two levels deep
                if let Ok(sub_entries) = std::fs::read_dir(&p) {
                    for sub in sub_entries.flatten() {
                        let sub_p = sub.path();
                        if sub_p.is_dir() {
                            let sub_cand = sub_p.join("posts.json");
                            if sub_cand.exists() {
                                return Ok(sub_cand);
                            }
                        }
                    }
                }
            }
        }
    }
    anyhow::bail!(
        "Could not find posts.json in '{}'. Make sure you selected the correct folder from your BeReal data export.",
        base.display()
    )
}

/// Parse posts.json into a vector of BeRealPost structs.
pub fn parse_posts(json_path: &Path) -> Result<Vec<BeRealPost>> {
    let data = std::fs::read_to_string(json_path)
        .with_context(|| format!("Failed to read {}", json_path.display()))?;
    let posts: Vec<serde_json::Value> = serde_json::from_str(&data)
        .with_context(|| "posts.json is not valid JSON")?;

    let mut result = Vec::with_capacity(posts.len());
    for (i, val) in posts.into_iter().enumerate() {
        match serde_json::from_value::<BeRealPost>(val) {
            Ok(post) => result.push(post),
            Err(e) => {
                log::warn!("Skipping malformed entry #{}: {}", i, e);
            }
        }
    }
    Ok(result)
}

/// Compute a per-month histogram of BeReal counts.
pub fn compute_monthly_histogram(posts: &[BeRealPost]) -> Vec<MonthCount> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for post in posts {
        if let Some(dt) = parse_taken_at(&post.taken_at) {
            let key = dt.format("%Y-%m").to_string();
            *counts.entry(key).or_insert(0) += 1;
        }
    }
    let mut months: Vec<MonthCount> = counts
        .into_iter()
        .map(|(month, count)| MonthCount { month, count })
        .collect();
    months.sort_by(|a, b| a.month.cmp(&b.month));
    months
}

/// Resolve a media asset's path relative to the archive base directory.
pub fn resolve_media_path(asset: &MediaAsset, media_base: &Path) -> Option<PathBuf> {
    // Normalize: strip leading slash, handle both old /Photos/.../bereal/ and new Photos/.../post/
    let raw = asset.path.trim_start_matches('/').replace('\\', "/");
    // Try direct: base / raw
    let candidate = media_base.join(&raw);
    if candidate.exists() {
        return Some(candidate);
    }
    // Try without leading folder prefixes if nested
    let filename = Path::new(&raw).file_name()?;
    for subdir in &[
        "Photos/post",
        "Photos/bereal",
        "post",
        "bereal",
        "Photos",
        "photos",
        "photos/post",
        "photos/bereal",
    ] {
        let p = media_base.join(subdir).join(filename);
        if p.exists() {
            return Some(p);
        }
    }
    // Also check parent directory if media_base is in a subfolder
    if let Some(parent) = media_base.parent() {
        let p = parent.join(&raw);
        if p.exists() {
            return Some(p);
        }
        for subdir in &["Photos/post", "Photos/bereal", "post", "bereal", "Photos", "photos"] {
            let p = parent.join(subdir).join(filename);
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}

/// Parse a takenAt ISO 8601 string to DateTime<Utc>.
pub fn parse_taken_at(s: &str) -> Option<DateTime<Utc>> {
    // Try: "2024-03-15T14:30:01.123Z"
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    // Try without fractional seconds: "2024-03-15T14:30:01Z"
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ") {
        return Some(dt.with_timezone(&Utc));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_taken_at() {
        assert!(parse_taken_at("2024-03-15T14:30:01.123Z").is_some());
        assert!(parse_taken_at("2022-04-19T12:03:02.704Z").is_some());
        assert!(parse_taken_at("invalid").is_none());
    }

    #[test]
    fn test_monthly_histogram_order() {
        let posts = vec![
            BeRealPost {
                taken_at: "2024-03-15T12:00:00.000Z".to_string(),
                primary: None,
                primary_placeholder: None,
                secondary: None,
                secondary_placeholder: None,
                bts_media: None,
                location: None,
                caption: None,
                retake_counter: None,
                visibility: None,
            },
            BeRealPost {
                taken_at: "2024-01-10T08:00:00.000Z".to_string(),
                primary: None,
                primary_placeholder: None,
                secondary: None,
                secondary_placeholder: None,
                bts_media: None,
                location: None,
                caption: None,
                retake_counter: None,
                visibility: None,
            },
        ];
        let hist = compute_monthly_histogram(&posts);
        assert_eq!(hist[0].month, "2024-01");
        assert_eq!(hist[1].month, "2024-03");
    }

    #[test]
    fn test_scan_zip_archive_valid_and_invalid() {
        use std::io::Write;
        use zip::write::SimpleFileOptions;

        // Create temporary valid zip
        let tmp_zip_path = std::env::temp_dir().join(format!("test_bereal_{}.zip", std::process::id()));
        {
            let file = File::create(&tmp_zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

            zip.start_file("posts.json", options).unwrap();
            let sample_json = r#"[
                {
                    "takenAt": "2024-05-10T14:00:00.000Z",
                    "primary": { "path": "/Photos/photo1.webp", "mediaType": "image" },
                    "secondary": { "path": "/Photos/photo2.webp", "mediaType": "image" }
                }
            ]"#;
            zip.write_all(sample_json.as_bytes()).unwrap();

            zip.start_file("user.json", options).unwrap();
            let user_json = r#"{"username": "tester", "fullname": "Test User"}"#;
            zip.write_all(user_json.as_bytes()).unwrap();

            zip.start_file("Photos/photo1.webp", options).unwrap();
            zip.write_all(b"fake image data").unwrap();

            zip.finish().unwrap();
        }

        let info = scan_archive(&tmp_zip_path.to_string_lossy()).unwrap();
        assert!(info.is_valid);
        assert_eq!(info.archive_type, "Zip");
        assert_eq!(info.user_name.as_deref(), Some("tester"));
        assert_eq!(info.user_fullname.as_deref(), Some("Test User"));
        assert_eq!(info.valid_post_count, 1);
        assert_eq!(info.found_media_count, 1);
        assert_eq!(info.missing_media_count, 1); // photo2 is missing
        assert_eq!(info.warnings.len(), 1); // warning about missing photo2

        let _ = std::fs::remove_file(&tmp_zip_path);

        // Test non-existent path
        let bad_info = scan_archive("non_existent_archive_path.zip").unwrap();
        assert!(!bad_info.is_valid);
        assert_eq!(bad_info.validation_errors.len(), 1);
    }
}
