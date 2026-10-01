use crate::core::config::WatermarkConfig;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

fn get_presets_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let presets_dir = app_dir.join("presets");
    fs::create_dir_all(&presets_dir).map_err(|e| format!("Failed to create presets dir: {}", e))?;

    Ok(presets_dir)
}

fn get_preset_path(app: &tauri::AppHandle, name: &str) -> Result<PathBuf, String> {
    let presets_dir = get_presets_dir(app)?;
    Ok(presets_dir.join(format!("{}.json", name)))
}

#[tauri::command]
pub async fn save_preset(
    app: tauri::AppHandle,
    name: String,
    config: WatermarkConfig,
) -> Result<(), String> {
    let path = get_preset_path(&app, &name)?;
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to save preset: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn load_preset(
    app: tauri::AppHandle,
    name: String,
) -> Result<WatermarkConfig, String> {
    let path = get_preset_path(&app, &name)?;
    let json = fs::read_to_string(&path).map_err(|e| format!("Failed to load preset: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse preset: {}", e))
}

#[tauri::command]
pub async fn delete_preset(
    app: tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    let path = get_preset_path(&app, &name)?;
    fs::remove_file(&path).map_err(|e| format!("Failed to delete preset: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn list_presets(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let presets_dir = get_presets_dir(&app)?;

    let mut presets = Vec::new();
    let entries = fs::read_dir(&presets_dir).map_err(|e| format!("Failed to read presets dir: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                presets.push(name.to_string());
            }
        }
    }

    presets.sort();
    Ok(presets)
}
