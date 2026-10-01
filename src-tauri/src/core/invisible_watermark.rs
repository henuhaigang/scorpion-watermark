use crate::core::config::InvisibleWatermark;
use blind_watermark::utils::embed_watermark_bytes;

pub fn embed_invisible_watermark(
    image: &mut image::DynamicImage,
    config: &InvisibleWatermark,
) -> Result<(), String> {
    if config.payload.is_empty() || config.key.is_empty() {
        return Ok(());
    }

    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("scorpion_wm_input.png");
    let output_path = temp_dir.join("scorpion_wm_output.png");

    image.save(&input_path).map_err(|e| format!("Failed to save temp image: {}", e))?;

    let watermark_bytes = config.payload.as_bytes();
    let seed = Some(config.key.as_bytes().iter().map(|&b| b as u64).sum());

    embed_watermark_bytes(&input_path, &output_path, watermark_bytes, seed)
        .map_err(|e| format!("Failed to embed watermark: {}", e))?;

    let result = image::open(&output_path).map_err(|e| format!("Failed to load watermarked image: {}", e))?;
    *image = result;

    let _ = std::fs::remove_file(&input_path);
    let _ = std::fs::remove_file(&output_path);

    Ok(())
}

/// 提取隐形水印。
///
/// 注意：当前实现**尚未完成**。`blind_watermark` 的提取需要知道嵌入时的
/// 载荷长度（bit 数），但这个长度没有被记录到图片里（而且默认剥离元数据），
/// 因此这里无法得知。旧代码硬编码 `0` 导致库内部 `assert!(wm_len > 0)`
/// panic，会让整个应用崩溃。
///
/// 要做完整需要先确定长度方案，例如：固定长度载荷（不足补零）、把长度写进
/// 载荷本身、或让用户在提取时同时提供原始载荷。详见 README「已知限制」。
pub fn extract_invisible_watermark(
    _image: &image::DynamicImage,
    _key: &str,
) -> Result<String, String> {
    Err("隐形水印提取尚未实现：缺少嵌入时的载荷长度信息".to_string())
}

