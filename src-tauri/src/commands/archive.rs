use anyhow::Result;
use std::path::Path;
use zip::ZipArchive;

use crate::pipeline::{parser, types::ArchiveInfo};

/// Scan an input path (folder or extracted zip) and return archive metadata.
#[tauri::command]
pub async fn scan_archive(path: String) -> Result<ArchiveInfo, String> {
    parser::scan_archive(&path).map_err(|e| e.to_string())
}

/// Extract a zip archive to a destination directory.
/// Returns the path of the extracted folder.
#[tauri::command]
pub async fn extract_zip(zip_path: String, dest_dir: String) -> Result<String, String> {
    let zip_path = Path::new(&zip_path);
    let dest = Path::new(&dest_dir);

    std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;

    let file = std::fs::File::open(zip_path)
        .map_err(|e| format!("Cannot open zip: {}", e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Invalid zip file: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match entry.enclosed_name() {
            Some(p) => dest.join(p),
            None => continue,
        };

        if entry.name().ends_with('/') {
            std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        }
    }

    Ok(dest.to_string_lossy().to_string())
}
