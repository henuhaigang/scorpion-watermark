use crate::core::image_loader::load_image;
use crate::core::invisible_watermark::extract_invisible_watermark as extract_wm;

#[tauri::command]
pub async fn extract_invisible_watermark(
    path: String,
    key: String,
) -> Result<String, String> {
    let image = load_image(&path)?;
    extract_wm(&image, &key)
}
