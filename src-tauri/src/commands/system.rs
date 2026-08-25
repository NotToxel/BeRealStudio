use crate::{
    pipeline::{types::FontInfo, video_ops::detect_ffmpeg},
    recapper::font_resolver::list_system_fonts as get_system_fonts,
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
