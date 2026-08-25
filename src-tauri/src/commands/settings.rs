use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::pipeline::types::AppSettings;

const SETTINGS_KEY: &str = "settings";
const SETTINGS_FILE: &str = "settings.json";

#[tauri::command]
pub async fn load_settings(app: AppHandle) -> Result<AppSettings, String> {
    let store = app
        .store(SETTINGS_FILE)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    if let Some(val) = store.get(SETTINGS_KEY) {
        if let Ok(settings) = serde_json::from_value::<AppSettings>(val) {
            return Ok(settings);
        }
    }

    Ok(AppSettings::default())
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings, app: AppHandle) -> Result<(), String> {
    let store = app
        .store(SETTINGS_FILE)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    let val = serde_json::to_value(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    store.set(SETTINGS_KEY, val);
    store
        .save()
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn reset_settings(app: AppHandle) -> Result<AppSettings, String> {
    let store = app
        .store(SETTINGS_FILE)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    let default_settings = AppSettings::default();
    let val = serde_json::to_value(&default_settings)
        .map_err(|e| format!("Failed to serialize defaults: {}", e))?;

    store.set(SETTINGS_KEY, val);
    store
        .save()
        .map_err(|e| format!("Failed to save store: {}", e))?;

    Ok(default_settings)
}

const ACTIVITY_KEY: &str = "activity_history";
const ACTIVITY_FILE: &str = "activity_history.json";

#[tauri::command]
pub async fn load_activity_history(app: AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let store = app
        .store(ACTIVITY_FILE)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    if let Some(val) = store.get(ACTIVITY_KEY) {
        if let Ok(history) = serde_json::from_value::<Vec<serde_json::Value>>(val) {
            return Ok(history);
        }
    }

    Ok(vec![])
}

#[tauri::command]
pub async fn save_activity_history(history: Vec<serde_json::Value>, app: AppHandle) -> Result<(), String> {
    let store = app
        .store(ACTIVITY_FILE)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    let val = serde_json::to_value(&history)
        .map_err(|e| format!("Failed to serialize activity history: {}", e))?;

    store.set(ACTIVITY_KEY, val);
    store
        .save()
        .map_err(|e| format!("Failed to save activity store: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn clear_activity_history(app: AppHandle) -> Result<(), String> {
    let store = app
        .store(ACTIVITY_FILE)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    store.set(ACTIVITY_KEY, serde_json::json!([]));
    store
        .save()
        .map_err(|e| format!("Failed to clear activity store: {}", e))?;

    Ok(())
}
