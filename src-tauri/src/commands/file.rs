use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn open_file(app: tauri::AppHandle) -> Result<String, String> {
    let file_path = app.dialog().file().blocking_pick_file();

    match file_path {
        Some(path) => Ok(path.to_string()),
        None => Err("No file selected".to_string()),
    }
}

#[tauri::command]
pub async fn read_file(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| e.to_string())
}
