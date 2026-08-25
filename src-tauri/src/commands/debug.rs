use std::{io::Write, path::PathBuf};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn export_debug_log(
    output_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let logs = state
        .log_buffer
        .lock()
        .map_err(|e| format!("Failed to acquire log lock: {}", e))?
        .clone();

    let dest = PathBuf::from(&output_path);
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut file = std::fs::File::create(&dest)
        .map_err(|e| format!("Failed to create log file: {}", e))?;

    writeln!(file, "=== BeReal Studio Debug Log ===").map_err(|e| e.to_string())?;
    writeln!(
        file,
        "Timestamp: {}",
        chrono::Utc::now().to_rfc3339()
    )
    .map_err(|e| e.to_string())?;
    writeln!(file, "Log Entries: {}", logs.len()).map_err(|e| e.to_string())?;
    writeln!(file, "--------------------------------------------------\n")
        .map_err(|e| e.to_string())?;

    for log_entry in logs {
        writeln!(
            file,
            "[{}] [{:?}] {}",
            log_entry.timestamp, log_entry.level, log_entry.message
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_debug_logs(
    state: State<'_, AppState>,
) -> Result<Vec<crate::pipeline::types::LogEvent>, String> {
    let logs = state
        .log_buffer
        .lock()
        .map_err(|e| format!("Failed to acquire log lock: {}", e))?
        .clone();
    Ok(logs)
}

#[tauri::command]
pub async fn clear_debug_logs(state: State<'_, AppState>) -> Result<(), String> {
    let mut logs = state
        .log_buffer
        .lock()
        .map_err(|e| format!("Failed to acquire log lock: {}", e))?;
    logs.clear();
    Ok(())
}
