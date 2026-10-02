#[allow(dead_code)]
mod commands;
#[allow(dead_code)]
pub mod core;

#[allow(unexpected_cfgs)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::file::open_file,
            commands::file::read_file,
            commands::preview::generate_preview,
            commands::preview::calculate_layout,
            commands::export::export_file,
            commands::preset::save_preset,
            commands::preset::load_preset,
            commands::preset::delete_preset,
            commands::preset::list_presets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
