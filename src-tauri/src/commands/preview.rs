use crate::core::config::VisibleWatermark;
use crate::core::image_loader::load_image;
use crate::core::layout::{calculate_layout as calc_layout, LayoutResult};
use image::ImageFormat;

#[tauri::command]
pub async fn generate_preview(path: String) -> Result<Vec<u8>, String> {
    let img = load_image(&path)?;

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

    Ok(buffer)
}

#[tauri::command]
pub async fn calculate_layout(
    canvas_width: u32,
    canvas_height: u32,
    config: VisibleWatermark,
) -> Result<LayoutResult, String> {
    calc_layout(canvas_width, canvas_height, &config)
}
