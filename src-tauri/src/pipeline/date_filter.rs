use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use std::path::{Path, PathBuf};

use crate::pipeline::{
    parser::parse_taken_at,
    types::BeRealPost,
};

/// Filter a list of posts to those whose takenAt falls within [start, end] (inclusive).
/// None on either bound means unbounded in that direction.
pub fn filter_by_date_range(
    posts: Vec<BeRealPost>,
    start: Option<&str>,
    end: Option<&str>,
) -> Vec<BeRealPost> {
    let start_dt: Option<DateTime<Utc>> = start.and_then(|s| parse_date_bound(s, false));
    let end_dt: Option<DateTime<Utc>> = end.and_then(|s| parse_date_bound(s, true));

    posts
        .into_iter()
        .filter(|post| {
            let taken = match parse_taken_at(&post.taken_at) {
                Some(dt) => dt,
                None => return true, // don't filter out posts with unparseable dates
            };
            let after_start = start_dt.map(|s| taken >= s).unwrap_or(true);
            let before_end = end_dt.map(|e| taken <= e).unwrap_or(true);
            after_start && before_end
        })
        .collect()
}

/// Filter image paths in a directory by the date encoded in their filename prefix (YYYY-MM-DDTHH-MM-SS).
/// Falls back to subfolders (combined/, singles/, combined_reversed/) if the root contains no direct images,
/// and falls back to file modification time if the name doesn't match date pattern.
pub fn filter_images_by_date(
    image_dir: &Path,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Vec<PathBuf>> {
    let start_dt: Option<DateTime<Utc>> = start.and_then(|s| parse_date_bound(s, false));
    let end_dt: Option<DateTime<Utc>> = end.and_then(|s| parse_date_bound(s, true));

    // Helper to collect direct image files from a directory
    let collect_direct_images = |dir: &Path| -> Vec<PathBuf> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.is_file()
                        && p.extension()
                            .map(|e| matches!(e.to_str(), Some("jpg" | "jpeg" | "png" | "webp")))
                            .unwrap_or(false)
                })
                .collect()
        } else {
            Vec::new()
        }
    };

    // 1. Check direct image children
    let mut raw_images = collect_direct_images(image_dir);

    // 2. If no direct images in root, check standard Photo Toolkit output subdirectories
    if raw_images.is_empty() {
        let combined_dir = image_dir.join("combined");
        let singles_dir = image_dir.join("singles");
        let reversed_dir = image_dir.join("combined_reversed");

        if combined_dir.is_dir() {
            raw_images = collect_direct_images(&combined_dir);
        }
        if raw_images.is_empty() && singles_dir.is_dir() {
            raw_images = collect_direct_images(&singles_dir);
        }
        if raw_images.is_empty() && reversed_dir.is_dir() {
            raw_images = collect_direct_images(&reversed_dir);
        }
        // 3. Fallback: scan immediate subdirectories recursively
        if raw_images.is_empty() {
            if let Ok(sub_entries) = std::fs::read_dir(image_dir) {
                for sub in sub_entries.flatten() {
                    let sub_path = sub.path();
                    if sub_path.is_dir() {
                        let sub_imgs = collect_direct_images(&sub_path);
                        raw_images.extend(sub_imgs);
                    }
                }
            }
        }
    }

    let mut paths: Vec<PathBuf> = raw_images
        .into_iter()
        .filter(|p| {
            let dt = extract_date_from_path(p);
            let after_start = start_dt.map(|s| dt.map(|d| d >= s).unwrap_or(true)).unwrap_or(true);
            let before_end = end_dt.map(|e| dt.map(|d| d <= e).unwrap_or(true)).unwrap_or(true);
            after_start && before_end
        })
        .collect();

    paths.sort();
    Ok(paths)
}

/// Parse a YYYY-MM-DD date string into a DateTime<Utc>.
/// If `is_end`, returns end-of-day (23:59:59); otherwise start-of-day (00:00:00).
fn parse_date_bound(s: &str, is_end: bool) -> Option<DateTime<Utc>> {
    let date = NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()?;
    let time = if is_end {
        chrono::NaiveTime::from_hms_opt(23, 59, 59)?
    } else {
        chrono::NaiveTime::from_hms_opt(0, 0, 0)?
    };
    let naive = date.and_time(time);
    Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}

/// Try to extract a DateTime from a filename like "2024-03-15T14-30-00_primary.jpg".
fn extract_date_from_path(path: &Path) -> Option<DateTime<Utc>> {
    let stem = path.file_stem()?.to_str()?;
    // Take the first 19 chars: "2024-03-15T14-30-00"
    if stem.len() < 19 {
        return None;
    }
    let prefix = &stem[..19];
    // Replace hyphens after T with colons for time portion
    let normalized = if prefix.len() >= 11 {
        let date_part = &prefix[..10];
        let time_part = prefix[11..].replace('-', ":");
        format!("{}T{}Z", date_part, time_part)
    } else {
        return None;
    };
    parse_taken_at(&normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::types::BeRealPost;

    fn make_post(taken_at: &str) -> BeRealPost {
        BeRealPost {
            taken_at: taken_at.to_string(),
            primary: None,
            primary_placeholder: None,
            secondary: None,
            secondary_placeholder: None,
            bts_media: None,
            location: None,
            caption: None,
            retake_counter: None,
            visibility: None,
        }
    }

    #[test]
    fn test_filter_date_range() {
        let posts = vec![
            make_post("2024-01-15T12:00:00.000Z"),
            make_post("2024-06-20T10:00:00.000Z"),
            make_post("2024-12-01T08:00:00.000Z"),
        ];
        let filtered = filter_by_date_range(posts, Some("2024-03-01"), Some("2024-09-30"));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].taken_at, "2024-06-20T10:00:00.000Z");
    }

    #[test]
    fn test_filter_no_bounds() {
        let posts = vec![
            make_post("2024-01-15T12:00:00.000Z"),
            make_post("2024-06-20T10:00:00.000Z"),
        ];
        let filtered = filter_by_date_range(posts, None, None);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_extract_date_from_path() {
        let p = PathBuf::from("2024-03-15T14-30-00_primary.jpg");
        let dt = extract_date_from_path(&p);
        assert!(dt.is_some());
    }

    #[test]
    fn test_filter_images_subfolder_fallback() {
        let temp_dir = std::env::temp_dir().join(format!("test_bereal_filter_{}", std::process::id()));
        let combined_dir = temp_dir.join("combined");
        let _ = std::fs::create_dir_all(&combined_dir);
        let test_file = combined_dir.join("2024-03-15T14-30-00_combined.jpg");
        let _ = std::fs::write(&test_file, b"fake image");

        let found = filter_images_by_date(&temp_dir, None, None).expect("filter should succeed");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], test_file);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
