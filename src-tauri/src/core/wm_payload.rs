use crate::core::config::InvisibleWatermark;
use crc32fast::hash as crc32;
use image::DynamicImage;
use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// 载荷固定 32 字节 = 256 bit。
///
/// **定长是关键**：`blind_watermark` 的提取接口需要调用方告知水印的 bit 长度，
/// 早期版本这里传的是 0，导致库内部 `assert!(wm_len > 0)` panic。
/// 定长后提取时长度已知，问题从根上消失。
pub const PAYLOAD_BYTES: usize = 32;
pub const PAYLOAD_BITS: usize = PAYLOAD_BYTES * 8;

/// 魔数 'S'（Scorpion）。用于区分「本工具的水印」与「恰好有内容的图」
const MAGIC: u8 = 0x53;
/// 载荷格式版本
pub const VERSION: u8 = 1;

// 字段偏移（字节）
const OFF_MAGIC: usize = 0;
const OFF_VERSION: usize = 1;
const OFF_CONTENT: usize = 2; // 8 字节感知哈希
const OFF_USER: usize = 10; // 4 字节归属者哈希
const OFF_TIMESTAMP: usize = 14; // 4 字节时间戳
const OFF_NONCE: usize = 18; // 4 字节随机数
const OFF_FILL: usize = 22; // 6 字节随机填充
const OFF_CRC: usize = 28; // 4 字节校验和
const CRC_COVER: usize = 28; // 校验和覆盖前 28 字节

/// 解码出的水印载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedPayload {
    pub version: u8,
    /// 感知哈希（8 字节），用于判断内容是否被编辑
    pub content_hash: [u8; 8],
    /// 归属者哈希（4 字节）。只存哈希不存原文，无法反推回标识内容
    pub user_hash: [u8; 4],
    pub timestamp: u32,
    pub nonce: u32,
}

/// 把归属者标识散列为 4 字节。
///
/// 只保存哈希而非原文：即使水印被提取，也无法反推出用户 ID / 订单号。
/// 鉴定时用户重新输入同样的标识，算出同样哈希即可比对。
pub fn user_hash(identifier: &str) -> [u8; 4] {
    let d = Sha256::digest(identifier.trim().as_bytes());
    [d[0], d[1], d[2], d[3]]
}

/// 从密钥派生水印算法使用的种子。
///
/// 旧实现是 `key.bytes().sum()` —— 字节求和几乎没有密钥强度，
/// 任何人都能猜到种子并伪造水印。改用 SHA-256 前 8 字节。
pub fn derive_seed(key: &str) -> u64 {
    let d = Sha256::digest(key.trim().as_bytes());
    u64::from_le_bytes([d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]])
}

/// 感知哈希（average hash）：缩放到 8×8 灰度，逐位与均值比较，得到 64 bit。
///
/// 为什么不用像素的 SHA-256：任何一次 JPEG 重存都会改变像素字节，
/// 用密码学哈希会把正常导出误判成「被篡改」。感知哈希对有损压缩稳定、
/// 对明显编辑敏感，适合做**信号**而非证据。
///
/// 局限：8×8 非常粗，只能感知整体构图变化，局部小范围修图可能测不出来。
/// 挡不住有心的攻击者，只用于给出倾向性判断。
pub fn perceptual_hash(image: &DynamicImage) -> [u8; 8] {
    let gray = image.to_luma8();
    let small = image::imageops::resize(&gray, 8, 8, FilterType::Triangle);

    let mut values: Vec<u8> = small.pixels().map(|p| p[0]).collect();
    // 用中位数而非均值做阈值：中位数对 JPEG 有损压缩引入的偏亮/偏暗更稳定，
    // 均值会被压缩噪声整体拉动，导致大量比特翻转
    values.sort_unstable();
    let median = values[values.len() / 2];

    let mut out = [0u8; 8];
    for i in 0..64 {
        if small.get_pixel((i % 8) as u32, (i / 8) as u32)[0] > median {
            out[i / 8] |= 1 << (7 - (i % 8));
        }
    }
    out
}

/// 两个感知哈希的汉明距离与相似度。
pub fn hash_similarity(a: &[u8; 8], b: &[u8; 8]) -> (u32, f32) {
    let mut diff = 0u32;
    for i in 0..8 {
        diff += (a[i] ^ b[i]).count_ones();
    }
    (diff, 1.0 - diff as f32 / 64.0)
}

/// 构造载荷字节。
fn build_payload(content_hash: [u8; 8], user_hash: [u8; 4], timestamp: u32, nonce: u32) -> Vec<u8> {
    let mut buf = vec![0u8; PAYLOAD_BYTES];
    buf[OFF_MAGIC] = MAGIC;
    buf[OFF_VERSION] = VERSION;
    buf[OFF_CONTENT..OFF_CONTENT + 8].copy_from_slice(&content_hash);
    buf[OFF_USER..OFF_USER + 4].copy_from_slice(&user_hash);
    buf[OFF_TIMESTAMP..OFF_TIMESTAMP + 4].copy_from_slice(&timestamp.to_le_bytes());
    buf[OFF_NONCE..OFF_NONCE + 4].copy_from_slice(&nonce.to_le_bytes());
    // 填充位用伪随机（来自 nonce 与时间戳），避免全零导致某些块的嵌入异常
    let mut filler = Sha256::new();
    filler.update(timestamp.to_le_bytes());
    filler.update(nonce.to_le_bytes());
    let f = filler.finalize();
    buf[OFF_FILL..OFF_FILL + 6].copy_from_slice(&f[0..6]);

    let sum = crc32(&buf[0..CRC_COVER]);
    buf[OFF_CRC..OFF_CRC + 4].copy_from_slice(&sum.to_le_bytes());
    buf
}

/// 解析载荷。返回 `None` 表示这不是本工具的水印（魔数/校验和不符）。
pub fn parse_payload(bytes: &[u8]) -> Option<DecodedPayload> {
    if bytes.len() < PAYLOAD_BYTES {
        return None;
    }
    if bytes[OFF_MAGIC] != MAGIC {
        return None;
    }
    // 版本不认识的载荷直接判为不匹配，避免把别的格式误当成有效
    if bytes[OFF_VERSION] != VERSION {
        return None;
    }
    let want = u32::from_le_bytes(bytes[OFF_CRC..OFF_CRC + 4].try_into().ok()?);
    if crc32(&bytes[0..CRC_COVER]) != want {
        return None;
    }

    let mut content_hash = [0u8; 8];
    content_hash.copy_from_slice(&bytes[OFF_CONTENT..OFF_CONTENT + 8]);
    let mut uh = [0u8; 4];
    uh.copy_from_slice(&bytes[OFF_USER..OFF_USER + 4]);

    Some(DecodedPayload {
        version: bytes[OFF_VERSION],
        content_hash,
        user_hash: uh,
        timestamp: u32::from_le_bytes(bytes[OFF_TIMESTAMP..OFF_TIMESTAMP + 4].try_into().ok()?),
        nonce: u32::from_le_bytes(bytes[OFF_NONCE..OFF_NONCE + 4].try_into().ok()?),
    })
}

/// 依据配置与当前图像内容生成待嵌入的载荷。
pub fn make_payload(image: &DynamicImage, config: &InvisibleWatermark) -> Vec<u8> {
    let content_hash = perceptual_hash(image);
    let uh = if config.payload.trim().is_empty() {
        [0u8; 4]
    } else {
        user_hash(&config.payload)
    };
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as u32)
        .unwrap_or(0);
    // nonce 由密钥与时间派生，保证同一密钥下不同次嵌入的载荷不同
    let nonce = (derive_seed(&config.key) ^ timestamp as u64) as u32;
    build_payload(content_hash, uh, timestamp, nonce)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    fn blank(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgb8(RgbImage::from_pixel(w, h, Rgb([120, 120, 120])))
    }

    #[test]
    fn test_payload_roundtrip() {
        let content = perceptual_hash(&blank(64, 64));
        let bytes = build_payload(content, user_hash("订单-12345"), 1_700_000_000, 0xABCD);

        assert_eq!(bytes.len(), PAYLOAD_BYTES);
        let decoded = parse_payload(&bytes).expect("应能解析");
        assert_eq!(decoded.content_hash, content);
        assert_eq!(decoded.user_hash, user_hash("订单-12345"));
        assert_eq!(decoded.timestamp, 1_700_000_000);
        assert_eq!(decoded.nonce, 0xABCD);
        assert_eq!(decoded.version, VERSION);
    }

    #[test]
    fn test_parse_rejects_foreign_data() {
        // 全零：魔数不符
        assert!(parse_payload(&[0u8; PAYLOAD_BYTES]).is_none());
        // 单比特翻转：crc32 应能发现
        let mut bytes = build_payload([1, 2, 3, 4, 5, 6, 7, 8], [9, 9, 9, 9], 1, 2);
        bytes[5] ^= 0x01;
        assert!(parse_payload(&bytes).is_none(), "比特翻转应被 crc32 检出");
    }

    /// 合成一张有结构、但频率不高（接近真实照片）的测试图
    fn photo_like(w: u32, h: u32) -> DynamicImage {
        let mut img = RgbImage::new(w, h);
        for (x, y, p) in img.enumerate_pixels_mut() {
            let fx = x as f32 / w as f32;
            let fy = y as f32 / h as f32;
            // 平滑渐变 + 几个柔和色块，模拟真实照片的低频结构
            let sky = (1.0 - fy) * 140.0 + 60.0;
            let hill = if fy > 0.55 + fx * 0.12 { 70.0 + fy * 40.0 } else { 255.0 };
            let sun = if (fx - 0.72).powi(2) + (fy - 0.22).powi(2) < 0.012 {
                90.0
            } else {
                0.0
            };
            *p = Rgb([
                (sky.min(hill) - sun).clamp(0.0, 255.0) as u8,
                (sky * 0.82).min(hill).clamp(0.0, 255.0) as u8,
                (sky * 0.55 + 40.0).min(hill + 20.0).clamp(0.0, 255.0) as u8,
            ]);
        }
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn test_perceptual_hash_survives_recompression() {
        // 感知哈希应对 JPEG 有损压缩稳定 —— 这是不用密码学哈希的原因。
        // 导出流程必然经过 JPEG 编码，若此处不稳定，鉴定功能会全程误报。
        let original = photo_like(320, 240);
        let h1 = perceptual_hash(&original);

        for quality in [95u8, 75, 50] {
            let mut buf = Vec::new();
            image::codecs::jpeg::JpegEncoder::new_with_quality(
                std::io::Cursor::new(&mut buf),
                quality,
            )
            .encode(original.to_rgb8().as_raw(), 320, 240, image::ExtendedColorType::Rgb8)
            .unwrap();
            let reloaded = image::load_from_memory(&buf).unwrap();
            let (dist, sim) = hash_similarity(&h1, &perceptual_hash(&reloaded));
            println!("JPEG q={quality}: dist={dist} sim={sim:.3}");
            assert!(sim >= 0.90, "JPEG q={quality} 后相似度过低：sim={sim:.3}");
        }
    }

    #[test]
    fn test_perceptual_hash_detects_big_change() {
        let a = photo_like(320, 240);
        // 上下颠倒 + 翻转，模拟明显的构图级编辑
        let b = DynamicImage::ImageRgb8(image::imageops::flip_vertical(&a.to_rgb8()));
        let (dist, sim) = hash_similarity(&perceptual_hash(&a), &perceptual_hash(&b));
        println!("上下颠倒: dist={dist} sim={sim:.3}");
        assert!(sim < 0.75, "构图级改动应显著降低相似度，实际 sim={sim:.3}");
    }

    #[test]
    fn test_seed_derivation_is_key_dependent() {
        // 旧的字节求和会让 "ab" 与 "ba" 得到相同种子
        assert_ne!(derive_seed("ab"), derive_seed("ba"));
        assert_eq!(derive_seed("  key  "), derive_seed("key"), "应忽略首尾空白");
    }
}
