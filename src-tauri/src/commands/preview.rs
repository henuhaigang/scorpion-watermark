use crate::core::config::VisibleWatermark;
use crate::core::image_loader::load_image;
use crate::core::layout::{calculate_layout as calc_layout, LayoutResult};
use image::ImageFormat;

#[tauri::command]
pub async fn generate_preview(path: String) -> Result<Vec<u8>, String> {
    eprintln!("[generate_preview] >>> Received path: {}", path);
    
    // Check if file exists
    if !std::path::Path::new(&path).exists() {
        eprintln!("[generate_preview] File does not exist at path: {}", path);
        
        // Try to list the temp directory
        if let Some(parent) = std::path::Path::new(&path).parent() {
            eprintln!("[generate_preview] Parent dir: {:?}", parent);
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    eprintln!("[generate_preview]   Found: {:?}", entry.path());
                }
            }
        }
        
        return Err(format!("File not found: {}", path));
    }
    
    eprintln!("[generate_preview] File exists, loading image...");
    
    let img = load_image(&path)?;
    eprintln!("[generate_preview] Image loaded: {}x{}", img.width(), img.height());

    let max_dimension = 1200u32;
    let (width, height) = (img.width(), img.height());

    let preview = if width > max_dimension || height > max_dimension {
        img.resize(max_dimension, max_dimension, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    let mut buffer = Vec::new();
    preview
        .write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode preview: {}", e))?;

    eprintln!("[generate_preview] Generated preview {} bytes for {}", buffer.len(), path);

    Ok(buffer)
}

#[tauri::command]
pub async fn debug_list_temp_dir() -> Result<Vec<String>, String> {
    let temp_dir = std::env::temp_dir();
    eprintln!("[debug_list_temp_dir] Temp dir: {:?}", temp_dir);
    
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            eprintln!("[debug_list_temp_dir] Found: {:?}", path);
            files.push(path.to_string_lossy().to_string());
        }
    }
    
    Ok(files)
}

#[tauri::command]
pub async fn calculate_layout(
    canvas_width: u32,
    canvas_height: u32,
    config: VisibleWatermark,
) -> Result<LayoutResult, String> {
    eprintln!("[calculate_layout] canvas: {}x{}, config: {:?}", canvas_width, canvas_height, config);
    let result = calc_layout(canvas_width, canvas_height, &config);
    eprintln!("[calculate_layout] Result: {:?}", result);
    result
}
