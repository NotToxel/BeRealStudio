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
    use std::io::Read;

    let zip_path = Path::new(&zip_path);
    let dest = Path::new(&dest_dir);

    std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;

    let file = std::fs::File::open(zip_path)
        .map_err(|e| format!("Cannot open zip: {}", e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Invalid zip file: {}", e))?;

    // Protection limits against zip bombs / malicious archives
    const MAX_ENTRIES: usize = 50_000;
    const MAX_TOTAL_BYTES: u64 = 50 * 1024 * 1024 * 1024; // 50 GB
    const MAX_SINGLE_FILE_BYTES: u64 = 5 * 1024 * 1024 * 1024; // 5 GB

    if archive.len() > MAX_ENTRIES {
        return Err(format!("ZIP contains too many entries ({} > max allowed {})", archive.len(), MAX_ENTRIES));
    }

    let mut total_extracted_bytes: u64 = 0;

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
            let out = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
            let mut writer = std::io::BufWriter::with_capacity(64 * 1024, out);
            let mut bounded_reader = (&mut entry).take(MAX_SINGLE_FILE_BYTES + 1);
            let copied = std::io::copy(&mut bounded_reader, &mut writer).map_err(|e| e.to_string())?;

            if copied > MAX_SINGLE_FILE_BYTES {
                return Err(format!("Extracted file exceeds maximum permitted size of 5 GB: {}", outpath.display()));
            }

            total_extracted_bytes += copied;
            if total_extracted_bytes > MAX_TOTAL_BYTES {
                return Err("Total decompressed ZIP data exceeds maximum permitted quota of 50 GB".to_string());
            }
        }
    }

    Ok(dest.to_string_lossy().to_string())
}
