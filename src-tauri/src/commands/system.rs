use tauri::{AppHandle, State};

use crate::{
    pipeline::{
        types::{FontInfo, OfflineGeoDbStatus},
        video_ops::detect_ffmpeg,
    },
    recapper::{
        font_resolver::list_system_fonts as get_system_fonts,
        geocoder::{check_offline_geodb_status, delete_offline_geodb_file, download_offline_geodb_stream},
    },
    state::AppState,
};

#[tauri::command]
pub async fn check_ffmpeg() -> Result<String, String> {
    match detect_ffmpeg() {
        Ok(path) => Ok(path.to_string_lossy().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn check_exiftool() -> Result<String, String> {
    match crate::pipeline::exif_writer::detect_exiftool() {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("ExifTool not found in PATH or standard directories".into()),
    }
}

#[tauri::command]
pub async fn list_system_fonts() -> Result<Vec<FontInfo>, String> {
    Ok(get_system_fonts())
}

#[tauri::command]
pub async fn cancel_job(job_id: String, state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.cancel_job(&job_id))
}

#[tauri::command]
pub async fn list_active_jobs(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.list_active_jobs())
}

#[tauri::command]
pub async fn check_offline_geodb(app: AppHandle) -> Result<OfflineGeoDbStatus, String> {
    Ok(check_offline_geodb_status(&app))
}

#[tauri::command]
pub async fn download_offline_geodb(app: AppHandle, tier: Option<String>) -> Result<(), String> {
    download_offline_geodb_stream(app, tier).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_active_geodb_tier(app: AppHandle, tier: String) -> Result<OfflineGeoDbStatus, String> {
    crate::recapper::geocoder::set_active_tier_file(&app, &tier).map_err(|e| e.to_string())?;
    Ok(check_offline_geodb_status(&app))
}

#[tauri::command]
pub async fn delete_offline_geodb(app: AppHandle, tier: Option<String>) -> Result<OfflineGeoDbStatus, String> {
    delete_offline_geodb_file(&app, tier).map_err(|e| e.to_string())?;
    Ok(check_offline_geodb_status(&app))
}

#[tauri::command]
pub async fn analyze_audio(path: String, buckets: Option<usize>) -> Result<crate::recapper::audio::AudioAnalysis, String> {
    let p = std::path::Path::new(&path);
    crate::recapper::audio::analyze_audio(p, buckets.unwrap_or(120)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn show_in_folder(path: String) -> Result<(), String> {
    let _p = std::path::Path::new(&path);
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let clean_path = path.replace('/', "\\");
        let p_clean = std::path::Path::new(&clean_path);

        if p_clean.is_file() {
            let _ = Command::new("explorer")
                .args(["/select,", &clean_path])
                .spawn()
                .map_err(|e| e.to_string())?;
        } else if p_clean.is_dir() {
            let _ = Command::new("explorer")
                .arg(&clean_path)
                .spawn()
                .map_err(|e| e.to_string())?;
        } else if let Some(parent) = p_clean.parent() {
            if parent.exists() {
                let _ = Command::new("explorer")
                    .arg(parent.to_string_lossy().as_ref())
                    .spawn()
                    .map_err(|e| e.to_string())?;
            } else {
                let _ = Command::new("explorer")
                    .arg(&clean_path)
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
        } else {
            let _ = Command::new("explorer")
                .arg(&clean_path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if p.is_file() {
            let _ = Command::new("open").args(["-R", &path]).spawn().map_err(|e| e.to_string())?;
        } else {
            let _ = Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let target = if p.is_file() { p.parent().unwrap_or(p) } else { p };
        let _ = Command::new("xdg-open").arg(target).spawn().map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub use crate::pipeline::types::DestinationStatus;

#[tauri::command]
pub async fn check_destination_status(path: String) -> Result<DestinationStatus, String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Ok(DestinationStatus {
            exists: false,
            is_directory: false,
            is_file: false,
            file_count: 0,
        });
    }

    if p.is_file() {
        return Ok(DestinationStatus {
            exists: true,
            is_directory: false,
            is_file: true,
            file_count: 1,
        });
    }

    // For directories (Toolkit export):
    // Check if BeReal Studio subfolders or generated files already exist in this destination directory.
    let mut conflicting_count = 0;
    let target_subdirs = ["singles", "combined", "combined_reversed"];
    for sub in &target_subdirs {
        let sub_dir = p.join(sub);
        if sub_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&sub_dir) {
                conflicting_count += entries.flatten().filter(|e| e.path().is_file()).count();
            }
        }
    }

    // Also check root of target folder for BeReal post naming patterns (e.g. YYYY-MM-DD... or *_combined.*)
    if let Ok(entries) = std::fs::read_dir(p) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                let name = entry_path.file_name().unwrap_or_default().to_string_lossy();
                if name.contains("_combined")
                    || name.contains("_primary")
                    || name.contains("_secondary")
                    || name.contains("_bts")
                {
                    conflicting_count += 1;
                }
            }
        }
    }

    Ok(DestinationStatus {
        exists: conflicting_count > 0,
        is_directory: true,
        is_file: false,
        file_count: conflicting_count,
    })
}

#[tauri::command]
pub async fn cleanup_cancelled_output(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Ok(());
    }

    if p.is_file() {
        let _ = std::fs::remove_file(p);
        return Ok(());
    }

    if p.is_dir() {
        let singles = p.join("singles");
        let combined = p.join("combined");
        let reversed = p.join("combined_reversed");
        if singles.exists() { let _ = std::fs::remove_dir_all(singles); }
        if combined.exists() { let _ = std::fs::remove_dir_all(combined); }
        if reversed.exists() { let _ = std::fs::remove_dir_all(reversed); }

        if let Ok(entries) = std::fs::read_dir(p) {
            if entries.count() == 0 {
                let _ = std::fs::remove_dir(p);
            }
        }
    }
    Ok(())
}
