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
