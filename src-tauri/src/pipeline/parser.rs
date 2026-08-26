use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

use crate::pipeline::types::{ArchiveInfo, BeRealPost, MediaAsset, MissingFileInfo, MonthCount, RetakeStats};

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserExportMeta {
    username: Option<String>,
    fullname: Option<String>,
    profile_picture: Option<serde_json::Value>,
    avatar: Option<serde_json::Value>,
    photo_url: Option<serde_json::Value>,
}

fn extract_pic_val(v: &serde_json::Value) -> (Option<String>, Option<String>) {
    let mut direct_url = None;
    let mut target_path = None;

    if let Some(s) = v.as_str() {
        if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("data:image") {
            direct_url = Some(s.to_string());
        } else if !s.trim().is_empty() {
            target_path = Some(s.to_string());
        }
    } else if let Some(obj) = v.as_object() {
        if let Some(u) = obj.get("url").and_then(|val| val.as_str()) {
            if u.starts_with("http://") || u.starts_with("https://") || u.starts_with("data:image") {
                direct_url = Some(u.to_string());
            } else if !u.trim().is_empty() {
                target_path = Some(u.to_string());
            }
        }
        if target_path.is_none() {
            if let Some(p) = obj.get("path").and_then(|val| val.as_str()) {
                if !p.trim().is_empty() {
                    target_path = Some(p.to_string());
                }
            } else if let Some(p) = obj.get("mediaPath").and_then(|val| val.as_str()) {
                if !p.trim().is_empty() {
                    target_path = Some(p.to_string());
                }
            }
        }
    }

    (direct_url, target_path)
}

fn resolve_profile_picture(user: &UserExportMeta) -> (Option<String>, Option<String>) {
    if let Some(ref pv) = user.profile_picture {
        let res = extract_pic_val(pv);
        if res.0.is_some() || res.1.is_some() {
            return res;
        }
    }
    if let Some(ref av) = user.avatar {
        let res = extract_pic_val(av);
        if res.0.is_some() || res.1.is_some() {
            return res;
        }
    }
    if let Some(ref pu) = user.photo_url {
        let res = extract_pic_val(pu);
        if res.0.is_some() || res.1.is_some() {
            return res;
        }
    }
    (None, None)
}

fn bytes_to_data_url(bytes: &[u8], ext: &str) -> String {
    use base64::Engine;
    let mime = match ext.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "image/jpeg",
    };
    format!("data:{};base64,{}", mime, base64::engine::general_purpose::STANDARD.encode(bytes))
}

/// Scan an input path (ZIP archive or unzipped directory) and return detailed BeReal archive metadata & validation.
pub fn scan_archive(path: &str) -> Result<ArchiveInfo> {
    let input_path = Path::new(path);
    if !input_path.exists() {
        return Ok(ArchiveInfo {
            archive_type: if path.ends_with(".zip") { "Zip".into() } else { "Directory".into() },
            validation_errors: vec![format!("Path does not exist on disk: {}", path)],
            ..Default::default()
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
                archive_type: "Zip".into(),
                validation_errors: vec![format!("Could not open ZIP file: {}", e)],
                media_base_path: zip_path.to_string_lossy().to_string(),
                ..Default::default()
            });
        }
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            return Ok(ArchiveInfo {
                archive_type: "Zip".into(),
                validation_errors: vec![format!("Invalid or corrupted ZIP archive: {}", e)],
                media_base_path: zip_path.to_string_lossy().to_string(),
                ..Default::default()
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

            if norm_lower.ends_with("memories.json") {
                posts_entry_name = Some(name.clone());
            } else if norm_lower.ends_with("posts.json") && posts_entry_name.is_none() {
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
                archive_type: "Zip".into(),
                has_photos_dir,
                has_user_json: user_entry_name.is_some(),
                validation_errors,
                warnings,
                media_base_path: zip_path.to_string_lossy().to_string(),
                ..Default::default()
            });
        }
    };

    // Parse user.json if present
    let has_user_json = user_entry_name.is_some();
    let mut user_name = None;
    let mut user_fullname = None;
    let mut profile_picture_data_url = None;
    let mut target_profile_path: Option<String> = None;

    if let Some(ref user_name_entry) = user_entry_name {
        if let Ok(entry) = archive.by_name(user_name_entry) {
            let reader = std::io::BufReader::new(entry);
            if let Ok(user) = serde_json::from_reader::<_, UserExportMeta>(reader) {
                let (direct, target) = resolve_profile_picture(&user);
                user_name = user.username;
                user_fullname = user.fullname;
                profile_picture_data_url = direct;
                target_profile_path = target;
            }
        }
    }

    if profile_picture_data_url.is_none() {
        let mut found_pic_entry = None;
        let clean_tp = target_profile_path.as_deref().map(|s| s.trim_start_matches('/').trim_start_matches("./"));
        let target_filename = clean_tp.and_then(|s| Path::new(s).file_name().and_then(|f| f.to_str()));

        // 1. Try matching target path and filename
        if let Some(filename) = target_filename {
            for i in 0..archive.len() {
                if let Ok(entry) = archive.by_index(i) {
                    let entry_name = entry.name();
                    if entry_name.ends_with(filename)
                        || entry_name.eq_ignore_ascii_case(filename)
                        || clean_tp.map(|tp| entry_name.ends_with(tp)).unwrap_or(false)
                    {
                        found_pic_entry = Some(entry_name.to_string());
                        break;
                    }
                }
            }
        }

        // 2. Search for any file in profile/ or Photos/profile/ or matching profile/avatar
        if found_pic_entry.is_none() {
            for i in 0..archive.len() {
                if let Ok(entry) = archive.by_index(i) {
                    let name_lower = entry.name().to_lowercase();
                    let is_image = name_lower.ends_with(".webp")
                        || name_lower.ends_with(".jpg")
                        || name_lower.ends_with(".jpeg")
                        || name_lower.ends_with(".png");

                    if is_image && !entry.name().ends_with('/') {
                        if name_lower.contains("/profile/")
                            || name_lower.starts_with("profile/")
                            || name_lower.contains("profile_picture")
                            || name_lower.contains("profile-picture")
                            || name_lower.contains("avatar")
                        {
                            found_pic_entry = Some(entry.name().to_string());
                            break;
                        }
                    }
                }
            }
        }

        if let Some(entry_name) = found_pic_entry {
            if let Ok(mut entry) = archive.by_name(&entry_name) {
                if entry.size() > 0 && entry.size() < 4 * 1024 * 1024 {
                    use std::io::Read;
                    let mut pic_bytes = Vec::new();
                    if entry.read_to_end(&mut pic_bytes).is_ok() {
                        let ext = Path::new(&entry_name).extension().and_then(|e| e.to_str()).unwrap_or("webp");
                        profile_picture_data_url = Some(bytes_to_data_url(&pic_bytes, ext));
                    }
                }
            }
        }
    }

    // Stream parse posts.json directly from zip without loading full string into RAM
    let raw_posts: Vec<serde_json::Value> = match archive.by_name(&posts_name) {
        Ok(entry) => {
            let reader = std::io::BufReader::with_capacity(128 * 1024, entry);
            match serde_json::from_reader(reader) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(ArchiveInfo {
                        archive_type: "Zip".into(),
                        user_name: user_name.clone(),
                        user_fullname,
                        has_posts_json: true,
                        has_photos_dir,
                        has_user_json,
                        validation_errors: vec![format!("'posts.json' in ZIP is not valid JSON or not an array: {}", e)],
                        warnings,
                        posts_json_path: posts_name,
                        media_base_path: zip_path.to_string_lossy().to_string(),
                        ..Default::default()
                    });
                }
            }
        }
        Err(e) => {
            return Ok(ArchiveInfo {
                archive_type: "Zip".into(),
                has_posts_json: true,
                has_photos_dir,
                has_user_json,
                validation_errors: vec![format!("Failed to access 'posts.json' in ZIP: {}", e)],
                warnings,
                posts_json_path: posts_name,
                media_base_path: zip_path.to_string_lossy().to_string(),
                ..Default::default()
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
    let photo_histogram = compute_monthly_histogram_photos(&valid_posts);
    let video_histogram = compute_monthly_histogram_videos(&valid_posts);
    let mut has_videos = false;
    let mut has_bts = false;
    let mut earliest: Option<DateTime<Utc>> = None;
    let mut latest: Option<DateTime<Utc>> = None;
    let mut primary_photo_count = 0usize;
    let mut secondary_photo_count = 0usize;
    let mut primary_video_count = 0usize;
    let mut secondary_video_count = 0usize;
    let mut bts_count = 0usize;
    let mut with_location_count = 0usize;
    let mut with_caption_count = 0usize;
    let mut retake_sum = 0u64;
    let mut retake_min = u32::MAX;
    let mut retake_max = 0u32;
    let mut retake_count = 0usize;

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
            if p.is_video() { has_videos = true; primary_video_count += 1; }
            else { primary_photo_count += 1; }
        }
        if let Some(s) = &post.secondary {
            if s.is_video() { has_videos = true; secondary_video_count += 1; }
            else { secondary_photo_count += 1; }
        }
        if post.bts_media.is_some() { has_bts = true; bts_count += 1; }
        if post.location.is_some() { with_location_count += 1; }
        if post.caption.as_ref().map(|c| !c.is_empty()).unwrap_or(false) { with_caption_count += 1; }
        if let Some(r) = post.retake_counter {
            retake_sum += r as u64;
            retake_min = retake_min.min(r);
            retake_max = retake_max.max(r);
            retake_count += 1;
        }
    }

    let retake_stats = if retake_count > 0 {
        Some(RetakeStats {
            min: retake_min,
            max: retake_max,
            avg: retake_sum as f32 / retake_count as f32,
        })
    } else {
        None
    };

    let is_valid = validation_errors.is_empty() && !valid_posts.is_empty();

    Ok(ArchiveInfo {
        is_valid,
        archive_type: "Zip".into(),
        user_name,
        user_fullname,
        profile_picture_data_url,
        entry_count: valid_posts.len(),
        valid_post_count: valid_posts.len(),
        corrupted_post_count: corrupted_posts,
        total_media_count: total_media,
        found_media_count: found_media,
        missing_media_count: missing_media,
        missing_files_sample: missing_sample,
        earliest_date: earliest.map(|d| d.format("%Y-%m-%d").to_string()),
        latest_date: latest.map(|d| d.format("%Y-%m-%d").to_string()),
        primary_photo_count,
        secondary_photo_count,
        primary_video_count,
        secondary_video_count,
        bts_count,
        with_location_count,
        with_caption_count,
        retake_stats,
        has_posts_json: true,
        has_photos_dir,
        has_user_json,
        has_videos,
        has_bts,
        monthly_histogram: histogram,
        photo_monthly_histogram: photo_histogram,
        video_monthly_histogram: video_histogram,
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
            // Check if this directory is an image folder (e.g. Toolkit export output folder with images)
            let images = crate::pipeline::date_filter::filter_images_by_date(base_dir, None, None).unwrap_or_default();
            if !images.is_empty() {
                let mut histogram_map: HashMap<String, usize> = HashMap::new();
                let mut earliest: Option<DateTime<Utc>> = None;
                let mut latest: Option<DateTime<Utc>> = None;

                for img in &images {
                    if let Some(dt) = crate::pipeline::date_filter::extract_date_from_path(img) {
                        let month_key = dt.format("%Y-%m").to_string();
                        *histogram_map.entry(month_key).or_insert(0) += 1;

                        if earliest.is_none() || Some(dt) < earliest {
                            earliest = Some(dt);
                        }
                        if latest.is_none() || Some(dt) > latest {
                            latest = Some(dt);
                        }
                    }
                }

                let mut months: Vec<String> = histogram_map.keys().cloned().collect();
                months.sort();

                let monthly_histogram: Vec<MonthCount> = months
                    .into_iter()
                    .map(|month| {
                        let count = histogram_map.get(&month).copied().unwrap_or(0) as u32;
                        MonthCount {
                            month,
                            count,
                        }
                    })
                    .collect();

                let earliest_date = earliest.map(|d| d.format("%Y-%m-%d").to_string());
                let latest_date = latest.map(|d| d.format("%Y-%m-%d").to_string());

                return Ok(ArchiveInfo {
                    is_valid: true,
                    archive_type: "ImageFolder".into(),
                    entry_count: images.len(),
                    valid_post_count: images.len(),
                    corrupted_post_count: 0,
                    total_media_count: images.len(),
                    found_media_count: images.len(),
                    missing_media_count: 0,
                    missing_files_sample: Vec::new(),
                    earliest_date,
                    latest_date,
                    primary_photo_count: images.len(),
                    secondary_photo_count: 0,
                    primary_video_count: 0,
                    secondary_video_count: 0,
                    bts_count: 0,
                    with_location_count: 0,
                    with_caption_count: 0,
                    retake_stats: None,
                    has_posts_json: false,
                    has_photos_dir: false,
                    has_user_json: false,
                    has_videos: false,
                    has_bts: false,
                    monthly_histogram: monthly_histogram.clone(),
                    photo_monthly_histogram: monthly_histogram,
                    video_monthly_histogram: Vec::new(),
                    validation_errors: Vec::new(),
                    warnings: Vec::new(),
                    posts_json_path: String::new(),
                    media_base_path: base_dir.to_string_lossy().to_string(),
                    user_name: None,
                    user_fullname: None,
                    profile_picture_data_url: None,
                });
            }

            validation_errors.push(format!(
                "Missing 'posts.json': Could not find posts.json or photos in '{}'. Make sure you selected a valid BeReal export or photos folder.",
                base_dir.display()
            ));
            let has_photos = base_dir.join("Photos").exists() || base_dir.join("photos").exists();
            return Ok(ArchiveInfo {
                has_photos_dir: has_photos,
                validation_errors,
                warnings,
                media_base_path: base_dir.to_string_lossy().to_string(),
                ..Default::default()
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
    let mut profile_picture_data_url = None;
    let mut target_profile_path: Option<String> = None;

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
                    let (direct, target) = resolve_profile_picture(&user);
                    user_name = user.username;
                    user_fullname = user.fullname;
                    profile_picture_data_url = direct;
                    target_profile_path = target;
                    break;
                }
            }
        }
    }

    if profile_picture_data_url.is_none() {
        let mut candidate_paths = Vec::new();
        let clean_tp = target_profile_path.as_deref().map(|s| s.trim_start_matches('/').trim_start_matches("./"));
        let target_filename = clean_tp.and_then(|s| Path::new(s).file_name().and_then(|f| f.to_str()));

        // Check exact target paths
        if let Some(tp) = clean_tp {
            candidate_paths.push(media_base.join(tp));
            candidate_paths.push(base_dir.join(tp));
        }

        // Check target filename inside profile folders
        if let Some(filename) = target_filename {
            for base in &[&media_base, base_dir] {
                candidate_paths.push(base.join("profile").join(filename));
                candidate_paths.push(base.join("Photos").join("profile").join(filename));
                candidate_paths.push(base.join("photos").join("profile").join(filename));
                candidate_paths.push(base.join("Profile").join(filename));
                candidate_paths.push(base.join("Photos").join("Profile").join(filename));
                candidate_paths.push(base.join(filename));
            }
        }

        // Common profile picture folder/file locations in BeReal GDPR
        for base in &[&media_base, base_dir] {
            candidate_paths.push(base.join("Photos").join("profile_picture.webp"));
            candidate_paths.push(base.join("photos").join("profile_picture.webp"));
            candidate_paths.push(base.join("profile").join("profile_picture.webp"));
            candidate_paths.push(base.join("profile_picture.webp"));
            candidate_paths.push(base.join("profile_picture.jpg"));
            candidate_paths.push(base.join("profile-picture.webp"));
            candidate_paths.push(base.join("profile-picture.jpg"));
            candidate_paths.push(base.join("profilePicture.webp"));
            candidate_paths.push(base.join("profilePicture.jpg"));
            candidate_paths.push(base.join("avatar.webp"));
            candidate_paths.push(base.join("avatar.jpg"));
        }

        for cp in &candidate_paths {
            if cp.is_file() {
                if let Ok(bytes) = std::fs::read(cp) {
                    if !bytes.is_empty() && bytes.len() < 4 * 1024 * 1024 {
                        let ext = cp.extension().and_then(|e| e.to_str()).unwrap_or("webp");
                        profile_picture_data_url = Some(bytes_to_data_url(&bytes, ext));
                        break;
                    }
                }
            }
        }

        // If still none, scan `profile` subfolders for any image file
        if profile_picture_data_url.is_none() {
            let search_dirs = [
                media_base.join("Photos").join("profile"),
                media_base.join("photos").join("profile"),
                media_base.join("profile"),
                base_dir.join("Photos").join("profile"),
                base_dir.join("photos").join("profile"),
                base_dir.join("profile"),
            ];
            for sdir in &search_dirs {
                if sdir.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(sdir) {
                        for entry in entries.flatten() {
                            let p = entry.path();
                            if p.is_file() {
                                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                                if matches!(ext.as_str(), "webp" | "jpg" | "jpeg" | "png") {
                                    if let Ok(bytes) = std::fs::read(&p) {
                                        if !bytes.is_empty() && bytes.len() < 4 * 1024 * 1024 {
                                            profile_picture_data_url = Some(bytes_to_data_url(&bytes, &ext));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if profile_picture_data_url.is_some() {
                        break;
                    }
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
                user_name,
                user_fullname,
                has_posts_json: true,
                has_photos_dir,
                has_user_json,
                validation_errors,
                warnings,
                posts_json_path: posts_json_path.to_string_lossy().to_string(),
                media_base_path: media_base.to_string_lossy().to_string(),
                ..Default::default()
            });
        }
    };

    let raw_posts: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&data);
    let raw_posts = match raw_posts {
        Ok(p) => p,
        Err(e) => {
            validation_errors.push(format!("'posts.json' is not valid JSON: {}", e));
            return Ok(ArchiveInfo {
                user_name,
                user_fullname,
                has_posts_json: true,
                has_photos_dir,
                has_user_json,
                validation_errors,
                warnings,
                posts_json_path: posts_json_path.to_string_lossy().to_string(),
                media_base_path: media_base.to_string_lossy().to_string(),
                ..Default::default()
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
    let photo_histogram = compute_monthly_histogram_photos(&valid_posts);
    let video_histogram = compute_monthly_histogram_videos(&valid_posts);
    let mut has_videos = false;
    let mut has_bts = false;
    let mut earliest: Option<DateTime<Utc>> = None;
    let mut latest: Option<DateTime<Utc>> = None;
    let mut primary_photo_count = 0usize;
    let mut secondary_photo_count = 0usize;
    let mut primary_video_count = 0usize;
    let mut secondary_video_count = 0usize;
    let mut bts_count = 0usize;
    let mut with_location_count = 0usize;
    let mut with_caption_count = 0usize;
    let mut retake_sum = 0u64;
    let mut retake_min = u32::MAX;
    let mut retake_max = 0u32;
    let mut retake_count = 0usize;

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
            if p.is_video() { has_videos = true; primary_video_count += 1; }
            else { primary_photo_count += 1; }
        }
        if let Some(s) = &post.secondary {
            if s.is_video() { has_videos = true; secondary_video_count += 1; }
            else { secondary_photo_count += 1; }
        }
        if post.bts_media.is_some() { has_bts = true; bts_count += 1; }
        if post.location.is_some() { with_location_count += 1; }
        if post.caption.as_ref().map(|c| !c.is_empty()).unwrap_or(false) { with_caption_count += 1; }
        if let Some(r) = post.retake_counter {
            retake_sum += r as u64;
            retake_min = retake_min.min(r);
            retake_max = retake_max.max(r);
            retake_count += 1;
        }
    }

    let retake_stats = if retake_count > 0 {
        Some(RetakeStats {
            min: retake_min,
            max: retake_max,
            avg: retake_sum as f32 / retake_count as f32,
        })
    } else {
        None
    };

    let is_valid = validation_errors.is_empty() && !valid_posts.is_empty();

    Ok(ArchiveInfo {
        is_valid,
        archive_type: "Directory".into(),
        user_name,
        user_fullname,
        profile_picture_data_url,
        entry_count: valid_posts.len(),
        valid_post_count: valid_posts.len(),
        corrupted_post_count: corrupted_posts,
        total_media_count: total_media,
        found_media_count: found_media,
        missing_media_count: missing_media,
        missing_files_sample: missing_sample,
        earliest_date: earliest.map(|d| d.format("%Y-%m-%d").to_string()),
        latest_date: latest.map(|d| d.format("%Y-%m-%d").to_string()),
        primary_photo_count,
        secondary_photo_count,
        primary_video_count,
        secondary_video_count,
        bts_count,
        with_location_count,
        with_caption_count,
        retake_stats,
        has_posts_json: true,
        has_photos_dir,
        has_user_json,
        has_videos,
        has_bts,
        monthly_histogram: histogram,
        photo_monthly_histogram: photo_histogram,
        video_monthly_histogram: video_histogram,
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
) -> (usize, usize, usize, Vec<MissingFileInfo>) {
    let mut total = 0usize;
    let mut found = 0usize;
    let mut missing = 0usize;
    let mut missing_sample: Vec<MissingFileInfo> = Vec::new();

    for post in posts {
        let parsed_dt = parse_taken_at(&post.taken_at);
        let post_date = parsed_dt
            .map(|d| d.format("%Y-%m-%d").to_string());
        let post_timestamp = parsed_dt
            .map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string());
        let assets: [(&Option<MediaAsset>, &str); 5] = [
            (&post.primary,              "primary"),
            (&post.primary_placeholder,  "primary"),
            (&post.secondary,            "secondary"),
            (&post.secondary_placeholder,"secondary"),
            (&post.bts_media,            "bts"),
        ];
        for (asset_opt, cam_type) in assets {
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
                        missing_sample.push(MissingFileInfo {
                            path: asset.path.clone(),
                            date: post_date.clone(),
                            timestamp: post_timestamp.clone(),
                            camera_type: Some(cam_type.to_string()),
                        });
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
) -> (usize, usize, usize, Vec<MissingFileInfo>) {
    let mut total = 0usize;
    let mut found = 0usize;
    let mut missing = 0usize;
    let mut missing_sample: Vec<MissingFileInfo> = Vec::new();

    for post in posts {
        let parsed_dt = parse_taken_at(&post.taken_at);
        let post_date = parsed_dt
            .map(|d| d.format("%Y-%m-%d").to_string());
        let post_timestamp = parsed_dt
            .map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string());
        let assets: [(&Option<MediaAsset>, &str); 5] = [
            (&post.primary,              "primary"),
            (&post.primary_placeholder,  "primary"),
            (&post.secondary,            "secondary"),
            (&post.secondary_placeholder,"secondary"),
            (&post.bts_media,            "bts"),
        ];
        for (asset_opt, cam_type) in assets {
            if let Some(asset) = asset_opt {
                total += 1;
                if resolve_media_path(asset, media_base).is_some() {
                    found += 1;
                } else {
                    missing += 1;
                    if missing_sample.len() < 10 {
                        missing_sample.push(MissingFileInfo {
                            path: asset.path.clone(),
                            date: post_date.clone(),
                            timestamp: post_timestamp.clone(),
                            camera_type: Some(cam_type.to_string()),
                        });
                    }
                }
            }
        }
    }

    (total, found, missing, missing_sample)
}

/// Find memories.json or posts.json by searching common locations in the archive directory.
pub fn find_posts_json(base: &Path) -> Result<PathBuf> {
    // 1. Direct: base/memories.json (Highest priority for rich metadata)
    let direct_memories = base.join("memories.json");
    if direct_memories.exists() {
        return Ok(direct_memories);
    }

    // 2. Direct: base/posts.json
    let direct_posts = base.join("posts.json");
    if direct_posts.exists() {
        return Ok(direct_posts);
    }

    // 3. One or two levels deep
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                for name in &["memories.json", "posts.json"] {
                    let candidate = p.join(name);
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
                // Two levels deep
                if let Ok(sub_entries) = std::fs::read_dir(&p) {
                    for sub in sub_entries.flatten() {
                        let sub_p = sub.path();
                        if sub_p.is_dir() {
                            for name in &["memories.json", "posts.json"] {
                                let sub_cand = sub_p.join(name);
                                if sub_cand.exists() {
                                    return Ok(sub_cand);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    anyhow::bail!(
        "Could not find memories.json or posts.json in '{}'. Make sure you selected the correct folder from your BeReal data export.",
        base.display()
    )
}

/// Parse posts/memories JSON into a vector of BeRealPost structs.
pub fn parse_posts(json_path: &Path) -> Result<Vec<BeRealPost>> {
    let data = std::fs::read_to_string(json_path)
        .with_context(|| format!("Failed to read {}", json_path.display()))?;
    let root_val: serde_json::Value = serde_json::from_str(&data)
        .with_context(|| "Data file is not valid JSON")?;

    let posts: Vec<serde_json::Value> = match root_val {
        serde_json::Value::Array(arr) => arr,
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Array(arr)) = map.get("memories").or_else(|| map.get("posts")).or_else(|| map.get("data")) {
                arr.clone()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

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

/// Intelligently merge a primary list of posts (e.g. from memories.json) and secondary list (from posts.json)
pub fn merge_posts_and_memories(
    memories_posts: Vec<(BeRealPost, String)>,
    posts_json_posts: Vec<(BeRealPost, String)>,
) -> Vec<(BeRealPost, String)> {
    if memories_posts.is_empty() {
        return posts_json_posts;
    }
    if posts_json_posts.is_empty() {
        return memories_posts;
    }

    let mut posts_by_time: HashMap<String, (BeRealPost, String)> = HashMap::new();
    let mut posts_by_minute: HashMap<String, (BeRealPost, String)> = HashMap::new();
    let mut posts_by_media: HashMap<String, (BeRealPost, String)> = HashMap::new();

    for (p, raw) in posts_json_posts {
        posts_by_time.insert(p.taken_at.clone(), (p.clone(), raw.clone()));
        if p.taken_at.len() >= 16 {
            posts_by_minute.insert(p.taken_at[..16].to_string(), (p.clone(), raw.clone()));
        }
        if let Some(ref prim) = p.primary {
            if let Some(fname) = Path::new(&prim.path).file_name().and_then(|n| n.to_str()) {
                posts_by_media.insert(fname.to_lowercase(), (p.clone(), raw.clone()));
            }
        }
    }

    let mut merged = Vec::with_capacity(memories_posts.len() + 10);
    let mut used_post_times = HashSet::new();

    for (mut mem_post, mem_raw) in memories_posts {
        let matched = posts_by_time.get(&mem_post.taken_at)
            .or_else(|| {
                if mem_post.taken_at.len() >= 16 {
                    posts_by_minute.get(&mem_post.taken_at[..16])
                } else {
                    None
                }
            })
            .or_else(|| {
                mem_post.primary.as_ref().and_then(|prim| {
                    Path::new(&prim.path).file_name().and_then(|n| n.to_str()).and_then(|fname| {
                        posts_by_media.get(&fname.to_lowercase())
                    })
                })
            });

        if let Some((post_match, _)) = matched {
            used_post_times.insert(post_match.taken_at.clone());
            if mem_post.retake_counter.is_none() {
                mem_post.retake_counter = post_match.retake_counter;
            }
            if mem_post.visibility.is_none() {
                mem_post.visibility = post_match.visibility.clone();
            }
            if mem_post.primary.is_none() {
                mem_post.primary = post_match.primary.clone();
            }
            if mem_post.secondary.is_none() {
                mem_post.secondary = post_match.secondary.clone();
            }
            if mem_post.caption.is_none() {
                mem_post.caption = post_match.caption.clone();
            }
            if mem_post.location.is_none() {
                mem_post.location = post_match.location.clone();
            }
            if mem_post.bts_media.is_none() {
                mem_post.bts_media = post_match.bts_media.clone();
            }
        }

        merged.push((mem_post, mem_raw));
    }

    // Append any extra posts from posts.json that were not in memories.json
    for (p_time, (extra_post, extra_raw)) in posts_by_time {
        if !used_post_times.contains(&p_time) {
            merged.push((extra_post, extra_raw));
        }
    }

    merged
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

/// Compute a per-month histogram counting only posts where the primary is a photo (not a video).
pub fn compute_monthly_histogram_photos(posts: &[BeRealPost]) -> Vec<MonthCount> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for post in posts {
        let is_video = post.primary.as_ref().map(|p| p.is_video()).unwrap_or(false);
        if !is_video {
            if let Some(dt) = parse_taken_at(&post.taken_at) {
                let key = dt.format("%Y-%m").to_string();
                *counts.entry(key).or_insert(0) += 1;
            }
        }
    }
    let mut months: Vec<MonthCount> = counts
        .into_iter()
        .map(|(month, count)| MonthCount { month, count })
        .collect();
    months.sort_by(|a, b| a.month.cmp(&b.month));
    months
}

/// Compute a per-month histogram counting only posts where the primary is a video.
pub fn compute_monthly_histogram_videos(posts: &[BeRealPost]) -> Vec<MonthCount> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for post in posts {
        let is_video = post.primary.as_ref().map(|p| p.is_video()).unwrap_or(false);
        if is_video {
            if let Some(dt) = parse_taken_at(&post.taken_at) {
                let key = dt.format("%Y-%m").to_string();
                *counts.entry(key).or_insert(0) += 1;
            }
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
                ..Default::default()
            },
            BeRealPost {
                taken_at: "2024-01-10T08:00:00.000Z".to_string(),
                ..Default::default()
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

    #[test]
    fn test_bereal_post_flexible_is_late_deserialization() {
        // 1. camelCase with boolean true
        let json1 = r#"{"takenAt": "2024-08-26T16:00:00Z", "isLate": true, "notificationAt": "2024-08-26T14:00:00Z"}"#;
        let p1: BeRealPost = serde_json::from_str(json1).unwrap();
        assert_eq!(p1.is_late, Some(true));
        assert_eq!(p1.notification_at.as_deref(), Some("2024-08-26T14:00:00Z"));

        // 2. snake_case with boolean true
        let json2 = r#"{"taken_at": "2024-08-26T16:00:00Z", "is_late": true, "late_in_seconds": 7200}"#;
        let p2: BeRealPost = serde_json::from_str(json2).unwrap();
        assert_eq!(p2.is_late, Some(true));
        assert_eq!(p2.late_in_seconds, Some(7200));

        // 3. String boolean "true" and string seconds "3600"
        let json3 = r#"{"takenAt": "2024-08-26T15:00:00Z", "isLate": "true", "lateInSeconds": "3600"}"#;
        let p3: BeRealPost = serde_json::from_str(json3).unwrap();
        assert_eq!(p3.is_late, Some(true));
        assert_eq!(p3.late_in_seconds, Some(3600));

        // 4. Integer 1 for true
        let json4 = r#"{"takenAt": "2024-08-26T15:00:00Z", "isLate": 1}"#;
        let p4: BeRealPost = serde_json::from_str(json4).unwrap();
        assert_eq!(p4.is_late, Some(true));

        // 5. On-time with boolean false
        let json5 = r#"{"takenAt": "2024-08-26T14:02:00Z", "isLate": false}"#;
        let p5: BeRealPost = serde_json::from_str(json5).unwrap();
        assert_eq!(p5.is_late, Some(false));

        // 6. On-time with snake_case "is_late": false
        let json6 = r#"{"taken_at": "2024-08-26T14:02:00Z", "is_late": false}"#;
        let p6: BeRealPost = serde_json::from_str(json6).unwrap();
        assert_eq!(p6.is_late, Some(false));
    }

    #[test]
    fn test_merge_posts_and_memories() {
        let mem_json = r#"{"takenTime": "2026-01-01T12:31:24.396Z", "berealMoment": "2026-01-01T12:31:05.229Z", "isLate": false}"#;
        let mem_post: BeRealPost = serde_json::from_str(mem_json).unwrap();
        let mem_list = vec![(mem_post, mem_json.to_string())];

        let post_json = r#"{"takenAt": "2026-01-01T12:31:24.396Z", "retakeCounter": 3, "visibility": ["friends"]}"#;
        let post_item: BeRealPost = serde_json::from_str(post_json).unwrap();
        let post_list = vec![(post_item, post_json.to_string())];

        let merged = merge_posts_and_memories(mem_list, post_list);
        assert_eq!(merged.len(), 1);
        let (p, _) = &merged[0];
        assert_eq!(p.is_late, Some(false));
        assert_eq!(p.notification_at.as_deref(), Some("2026-01-01T12:31:05.229Z"));
        assert_eq!(p.retake_counter, Some(3));
        assert_eq!(p.visibility.as_ref().unwrap(), &vec!["friends".to_string()]);
    }

    #[test]
    fn test_merge_exact_user_samples() {
        let mem_json = r#"{
            "frontImage": {"bucket": "storage.bere.al", "height": 2000, "width": 1500, "path": "/Photos/4KxneKfqNuNOA08K31QeHa0l7MI2/post/FXZtPxP7F6fBqNna.webp", "mediaType": "image", "mimeType": "image/webp"},
            "backImage": {"bucket": "storage.bere.al", "height": 2000, "width": 1500, "path": "/Photos/4KxneKfqNuNOA08K31QeHa0l7MI2/post/qrd1qukCZ8u3f1oL.webp", "mediaType": "image", "mimeType": "image/webp"},
            "caption": "I may have had a dream BeReal went off",
            "isLate": true,
            "date": "2025-06-18T00:00:00.000Z",
            "takenTime": "2025-06-18T12:03:26.815Z",
            "berealMoment": "2025-06-18T11:40:05.293Z",
            "location": {"latitude": 51.462059020996094, "longitude": -0.2528020143508911}
        }"#;
        let mem_post: BeRealPost = serde_json::from_str(mem_json).unwrap();
        assert_eq!(mem_post.is_late, Some(true));
        assert_eq!(mem_post.taken_at, "2025-06-18T12:03:26.815Z");
        assert_eq!(mem_post.notification_at.as_deref(), Some("2025-06-18T11:40:05.293Z"));
        assert!(mem_post.primary.is_some());
        assert_eq!(mem_post.primary.as_ref().unwrap().path, "/Photos/4KxneKfqNuNOA08K31QeHa0l7MI2/post/qrd1qukCZ8u3f1oL.webp");

        let post_json = r#"{
            "primary": {"bucket": "storage.bere.al", "height": 2000, "width": 1500, "path": "/Photos/4KxneKfqNuNOA08K31QeHa0l7MI2/post/qrd1qukCZ8u3f1oL.webp", "mediaType": "image", "mimeType": "image/webp"},
            "secondary": {"bucket": "storage.bere.al", "height": 2000, "width": 1500, "path": "/Photos/4KxneKfqNuNOA08K31QeHa0l7MI2/post/FXZtPxP7F6fBqNna.webp", "mediaType": "image", "mimeType": "image/webp"},
            "retakeCounter": 0,
            "caption": "I may have had a dream BeReal went off",
            "location": {"latitude": 51.462059020996094, "longitude": -0.2528020143508911},
            "visibility": ["friends"],
            "takenAt": "2025-06-18T12:03:26.815Z"
        }"#;
        let post_item: BeRealPost = serde_json::from_str(post_json).unwrap();

        let merged = merge_posts_and_memories(vec![(mem_post, mem_json.to_string())], vec![(post_item, post_json.to_string())]);
        assert_eq!(merged.len(), 1);
        let (p, _) = &merged[0];
        assert_eq!(p.is_late, Some(true));
        assert_eq!(p.notification_at.as_deref(), Some("2025-06-18T11:40:05.293Z"));
        assert_eq!(p.retake_counter, Some(0));
        assert_eq!(p.visibility.as_ref().unwrap(), &vec!["friends".to_string()]);
    }
}
