use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use std::sync::OnceLock;

const FONT_DATA: &[u8] = include_bytes!("../../assets/fonts/WatermarkSC-Regular.ttf");

/// 行高基准（相对字号）。行间距在此基础上叠加：
/// `行高 = 字号 × (LINE_HEIGHT_BASE + 行间距)`
///
/// 因此行间距 = 0 表示紧凑的单倍行距（不会重叠），数值越大越松。
/// 若沿用 `字号 × 系数 × 行间距` 的写法，行间距为 0 会让行高归零导致各行叠在一起。
pub const LINE_HEIGHT_BASE: f32 = 1.0;
/// 单行文字最长占画布宽度的比例
pub const MAX_LINE_WIDTH_RATIO: f32 = 0.35;
/// 字号下限（相对画布宽度），低于此值不再缩小，改为增加行数
pub const MIN_FONT_RATIO: f32 = 0.015;
/// 字号上限（相对画布宽度）
pub const MAX_FONT_RATIO: f32 = 0.08;
/// 可读字号阈值（相对画布宽度）。
///
/// 排版策略：单行能容纳全文且缩到该阈值以上时用单行；
/// 否则改用多行以换取更大的字号，避免水印小到看不见。
pub const COMFORTABLE_FONT_RATIO: f32 = 0.02;
/// 文字块最多允许的行数
pub const MAX_LINES: usize = 3;

static FONT: OnceLock<Result<FontRef<'static>, String>> = OnceLock::new();

/// 解析并缓存字体。字体数据是编译期内嵌的常量，解析一次即可复用，
/// 避免每次调参都重新解析 cmap / hmtx 等表。
pub fn font() -> Result<&'static FontRef<'static>, String> {
    FONT
        .get_or_init(|| {
            FontRef::try_from_slice(FONT_DATA).map_err(|e| format!("Failed to load font: {}", e))
        })
        .as_ref()
        .map_err(|e| e.clone())
}

/// 单个字符的前进宽度（像素）。
///
/// 注意：ab_glyph 的缩放因子是 `scale / height_unscaled`，
/// 必须走 `ScaleFont::h_advance`，不能直接用 `h_advance_unscaled * scale.x`。
pub fn char_width(font: &FontRef, scale: PxScale, ch: char) -> f32 {
    let scaled = font.as_scaled(scale);
    scaled.h_advance(font.glyph_id(ch))
}

/// 整段文字的前进宽度（像素），不含字距修正。
pub fn measure_text_width(font: &FontRef, scale: PxScale, text: &str) -> f32 {
    let scaled = font.as_scaled(scale);
    let mut total = 0.0;
    for ch in text.chars() {
        total += scaled.h_advance(font.glyph_id(ch));
    }
    if total == 0.0 {
        text.chars().count() as f32 * scale.x
    } else {
        total
    }
}

/// 把「字号（em 像素）」转换成 ab_glyph 的 `PxScale`。
///
/// ab_glyph 把 `PxScale` 定义为**字体行框高度**（`ascent - descent`）对应的像素数，
/// 而界面上的「字号」以及 CSS `font-size` 指的是 **em 字号**。多数中文字体的行框
/// 明显大于 em 框（Noto Sans SC 是 1448/1000，Apple Heiti SC 是 1000/1000），
/// 所以不能直接用 `PxScale::from(font_size)`——那会让实际字形比预期小 30% 左右，
/// 并且与前端 Konva 用 CSS 字号渲染的结果对不上。
///
/// 统一走这个函数，保证「字号」在任意字体下的含义一致，也保证预览与导出一致。
pub fn scale_for(font: &FontRef, font_size: f32) -> PxScale {
    let units_per_em = font.units_per_em().unwrap_or(1000.0);
    PxScale::from(font_size * font.height_unscaled() / units_per_em)
}

/// 字体子集里缺失的字符。
///
/// 字库是裁剪过的（见 `scripts/build_font.py`），GB2312 之外的生僻字可能
/// 不在子集内。缺字形时后端会画成豆腐块，前端则逐字回退到系统字体，
/// 两边表现不一致，所以要把缺字暴露出来。
pub fn missing_glyphs(text: &str) -> Vec<char> {
    let Ok(font) = font() else {
        return Vec::new();
    };
    let mut missing: Vec<char> = Vec::new();
    for ch in text.chars() {
        if ch.is_whitespace() {
            continue;
        }
        // GlyphId(0) 是 .notdef，表示字库里没有这个字形
        if font.glyph_id(ch).0 == 0 && !missing.contains(&ch) {
            missing.push(ch);
        }
    }
    missing
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 前后端必须使用同一个字体文件，否则预览与导出的字形会不一致。
    /// 重新生成字体请使用 `python3 scripts/build_font.py`。
    #[test]
    fn test_backend_and_frontend_font_identical() {
        let frontend: &[u8] = include_bytes!("../../../public/fonts/WatermarkSC-Regular.ttf");
        assert_eq!(
            FONT_DATA.len(),
            frontend.len(),
            "前后端字体文件大小不一致，请运行 scripts/build_font.py 重新生成"
        );
        assert!(
            FONT_DATA == frontend,
            "前后端字体文件内容不一致，请运行 scripts/build_font.py 重新生成"
        );
    }

    #[test]
    fn test_common_watermark_text_has_no_missing_glyphs() {
        let samples = [
            "仅供用来办理车险使用",
            "仅供办理人保车险使用的时候",
            "此文件仅作内部使用不得外传",
            "机密文件",
            "中国人民财产保险股份有限公司",
            "2026-08-02 (测试) #1",
        ];
        for text in samples {
            let missing = missing_glyphs(text);
            assert!(missing.is_empty(), "字体缺少字形 {:?}：{}", missing, text);
        }
    }

    #[test]
    fn test_reports_missing_glyphs() {
        // 生僻字不在 GB2312 子集内，应被检测出来
        let missing = missing_glyphs("仅供使用🀄");
        assert!(!missing.is_empty(), "应检测到缺失字形");
    }
}