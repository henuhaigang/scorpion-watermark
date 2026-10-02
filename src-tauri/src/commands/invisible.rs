use crate::core::image_loader::load_image;
use crate::core::invisible_watermark::{
    extract_invisible_watermark as extract_wm, VerificationResult,
};

/// 从图片提取隐形水印并给出鉴定结论。
///
/// `expected_identifier` 可选：传入后会把归属者哈希与该标识比对，
/// 用于确认这份可疑副本究竟发给了谁。
#[tauri::command]
pub async fn extract_invisible_watermark(
    path: String,
    key: String,
    expected_identifier: Option<String>,
) -> Result<VerificationResult, String> {
    let image = load_image(&path)?;
    extract_wm(
        &image,
        &key,
        expected_identifier.as_deref().filter(|s| !s.trim().is_empty()),
    )
}