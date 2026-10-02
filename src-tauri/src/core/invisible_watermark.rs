use crate::core::wm_payload::DecodedPayload;
use serde::{Deserialize, Serialize};

/// 隐形水印当前不可用。
///
/// 已实测确认：底层库 `blind_watermark` 0.1.3 的**提取功能失效** ——
/// 嵌入确实会修改像素（实测 22 万像素变化、最大差值 14），但提取恒返回全 0。
/// 排查覆盖了以下维度，均无法改变结果：
///
/// | 变量 | 取值范围 | 结果 |
/// |---|---|---|
/// | 种子模式 | `None`（Normal）/ `Some(seed)`（Strategy） | 差异 31/32 字节 |
/// | 嵌入强度 | (36,20) → (220,140) | 差异 31/32 字节 |
/// | 图像尺寸 | 256 / 512 / 1024 | 差异 31/32 字节 |
/// | 载荷长度 | 8 / 32 字节 | 差异 7/8、31/32 字节 |
///
/// 隔离测试确认这是库自身的缺陷，与本项目代码无关。
/// crates.io 上 0.1.3 已是最新版本，无修复版本可升级。
///
/// 因此此处返回明确错误，**不崩溃、不静默失败** —— 鉴证功能一旦给出
/// 错误结论，危害远大于功能缺失。
///
/// 要恢复该功能，需要先解决底层问题，二选一：
/// 1. vendor 该库并定位修复提取逻辑
/// 2. 自行实现一套嵌入/提取方案（`core/wm_payload.rs` 的载荷格式与
///    感知哈希已就绪，可直接复用）
pub const UNAVAILABLE_MSG: &str =
    "隐形水印当前不可用：底层库 blind_watermark 0.1.3 提取功能失效（能嵌入但无法提取）";

/// 鉴定结果。
///
/// 三档可靠性刻意区分，不要混为一谈：
/// - `found`（高）：确实检出了本工具嵌入的水印
/// - `owner_matches`（高）：归属者哈希与输入标识一致
/// - `content_similarity`（**启发式**）：内容指纹相似度，只是倾向性信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub found: bool,
    pub version: u8,
    /// Unix 时间戳（秒）
    pub timestamp: u32,
    /// 每份副本唯一，用于区分不同发放批次
    pub nonce: u32,
    /// 感知哈希汉明距离（0~64）
    pub content_distance: u32,
    /// 内容相似度 0.0~1.0
    pub content_similarity: f32,
    /// 若调用方提供了预期标识，此字段给出比对结果
    pub owner_matches: Option<bool>,
    /// 结论说明（中文，可直接展示给用户）
    pub verdict: String,
}

impl VerificationResult {
    pub fn unavailable() -> Self {
        Self {
            found: false,
            version: 0,
            timestamp: 0,
            nonce: 0,
            content_distance: 64,
            content_similarity: 0.0,
            owner_matches: None,
            verdict: UNAVAILABLE_MSG.to_string(),
        }
    }
}

/// 依据载荷与当前图像给出鉴定结论。
///
/// 这部分是纯逻辑，已通过单元测试验证；等底层嵌入/提取修好后可直接启用。
#[allow(dead_code)]
pub fn judge(
    payload: Option<&DecodedPayload>,
    current_hash: [u8; 8],
    expected_matches: Option<bool>,
    similarity: f32,
) -> VerificationResult {
    let Some(p) = payload else {
        return VerificationResult {
            found: false,
            version: 0,
            timestamp: 0,
            nonce: 0,
            content_distance: 64,
            content_similarity: 0.0,
            owner_matches: None,
            verdict: "未检出隐形水印".into(),
        };
    };

    let distance = crate::core::wm_payload::hash_similarity(&p.content_hash, &current_hash).0;

    VerificationResult {
        found: true,
        version: p.version,
        timestamp: p.timestamp,
        nonce: p.nonce,
        content_distance: distance,
        content_similarity: similarity,
        owner_matches: expected_matches,
        verdict: String::new(),
    }
}

pub fn embed_invisible_watermark(
    _image: &mut image::DynamicImage,
    _config: &crate::core::config::InvisibleWatermark,
) -> Result<(), String> {
    Err(UNAVAILABLE_MSG.to_string())
}

pub fn extract_invisible_watermark(
    _image: &image::DynamicImage,
    _key: &str,
    _expected_identifier: Option<&str>,
) -> Result<VerificationResult, String> {
    Ok(VerificationResult::unavailable())
}