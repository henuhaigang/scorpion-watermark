use crate::core::config::VisibleWatermark;
use crate::core::font::{
    font, measure_text_width, missing_glyphs, scale_for, BLOCK_GAP_MIN_OF_LINE,
    COMFORTABLE_FONT_RATIO, LINE_HEIGHT_BASE, MAX_FONT_RATIO, MAX_LINES, MAX_LINE_WIDTH_RATIO,
    MIN_FONT_RATIO,
};
use ab_glyph::{FontRef, PxScale};
use serde::{Deserialize, Serialize};

/// 宽度比较容差（px）。字号恰好等于「宽度上限 / 每行字数」时，累计宽度会因
/// 浮点误差略微超出上限。若「判断能否放下」与「实际拆分」两处阈值不一致，
/// 会出现判定单行放得下、渲染时却又被切开的矛盾。
const WIDTH_TOLERANCE: f32 = 1.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutResult {
    pub items: Vec<LayoutItem>,
    pub font_size: f32,
    /// 最宽一行的宽度（像素）
    pub text_width: f32,
    /// 实际生效的块间距（像素），已含「不小于块内行距」的下限修正
    pub block_gap: f32,
    /// 整个文字块（所有行）的高度（像素）
    pub text_height: f32,
    /// 以 `\n` 拼接的多行文本
    pub text: String,
    pub lines: Vec<String>,
    /// 字号是否因为「保证完整显示 + 保持可读」而被自动缩小过
    pub font_auto_shrunk: bool,
    /// 字体子集中缺失的字符，需要提示用户
    pub missing_glyphs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutItem {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

pub fn calculate_layout(
    canvas_width: u32,
    canvas_height: u32,
    config: &VisibleWatermark,
) -> Result<LayoutResult, String> {
    if !config.enabled || config.text.is_empty() {
        return Ok(LayoutResult {
            items: vec![],
            font_size: 0.0,
            text_width: 0.0,
            text_height: 0.0,
            block_gap: 0.0,
            text: config.text.clone(),
            lines: vec![],
            font_auto_shrunk: false,
            missing_glyphs: vec![],
        });
    }

    let font = font()?;

    let w = canvas_width as f32;
    let min_font = w * MIN_FONT_RATIO;
    let max_font = w * MAX_FONT_RATIO;
    let max_line_width = w * MAX_LINE_WIDTH_RATIO;
    let comfortable_font = w * COMFORTABLE_FONT_RATIO;
    let line_spacing = config.line_spacing.max(0.0);

    // 用户按比例设定的字号
    let ratio_font = (w * config.font_size_ratio / 100.0).clamp(min_font, max_font);

    let (font_size, target_lines) = choose_font_and_lines(
        font,
        &config.text,
        max_line_width,
        ratio_font,
        comfortable_font,
        min_font,
    );

    let font_auto_shrunk = font_size < ratio_font - 0.5;

    let scale = scale_for(font, font_size);
    let lines = wrap_text(font, scale, &config.text, max_line_width, target_lines);

    let line_height = font_size * (LINE_HEIGHT_BASE + line_spacing);
    let text_width = lines
        .iter()
        .map(|l| measure_text_width(font, scale, l))
        .fold(0.0f32, f32::max);
    let text_height = line_height * lines.len() as f32;

    // 间距基于字号，与文本长度解耦，长短文本的视觉密度保持一致。
    // 块间距不得小于块内行距，否则相邻块的文字会比同一块的行更密，
    // 看起来像文字被拦腰截断。
    let min_gap = line_height * BLOCK_GAP_MIN_OF_LINE;
    let gap = (font_size * config.block_gap_ratio).max(min_gap);
    let spacing_x = text_width + gap;
    let spacing_y = text_height + gap;

    // 保留角度符号，使网格排布方向与字形旋转方向一致
    let angle_rad = config.angle.to_radians();
    let cos = angle_rad.cos();
    let sin = angle_rad.sin();

    let diagonal = ((canvas_width as f32).powi(2) + (canvas_height as f32).powi(2)).sqrt();
    let radius = diagonal / 2.0;

    let count_x = (diagonal / spacing_x).ceil() as i32 + 2;
    let count_y = (diagonal / spacing_y).ceil() as i32 + 2;

    let cx = canvas_width as f32 / 2.0;
    let cy = canvas_height as f32 / 2.0;

    let mut items = Vec::new();

    for i in (-count_x / 2)..(count_x / 2 + 1) {
        for j in (-count_y / 2)..(count_y / 2 + 1) {
            let offset_x = if j % 2 == 0 { 0.0 } else { spacing_x * 0.5 };
            let grid_x = i as f32 * spacing_x + offset_x;
            let grid_y = j as f32 * spacing_y;

            let dist = (grid_x * grid_x + grid_y * grid_y).sqrt();
            if dist > radius {
                continue;
            }

            let rotated_x = grid_x * cos - grid_y * sin;
            let rotated_y = grid_x * sin + grid_y * cos;

            items.push(LayoutItem {
                x: cx + rotated_x,
                y: cy + rotated_y,
                rotation: config.angle,
            });
        }
    }

    Ok(LayoutResult {
        items,
        font_size,
        text_width,
        block_gap: gap,
        text_height,
        text: lines.join("\n"),
        lines,
        font_auto_shrunk,
        missing_glyphs: missing_glyphs(&config.text)
            .into_iter()
            .map(|c| c.to_string())
            .collect(),
    })
}

/// 决定最终的行数与字号。
///
/// 优先级：
/// 1. 从单行开始试，逐行增加，取**行数最少、且字号不低于可读阈值**的方案。
///    单行也需要缩字号才放得下时，只要缩完仍然可读，就用单行 —— 这样文字
///    越少行越直观，符合「让水印完整显示成一行」的预期。
/// 2. 如果缩到单行会低于可读阈值，则改用多行换取更大的字号，避免水印
///    小到看不见（这是文字极长时的取舍）。
/// 3. 兜底：保持用户设定的字号，取能放下全文的最少行数
///
/// 文字永远完整显示，任何情况下都不截断。
fn choose_font_and_lines(
    font: &FontRef,
    text: &str,
    max_line_width: f32,
    ratio_font: f32,
    comfortable_font: f32,
    min_font: f32,
) -> (f32, usize) {
    let font_for = |lines: usize| -> f32 {
        let per_line = max_segment_len(text, lines) as f32;
        (max_line_width / per_line.max(1.0)).clamp(min_font, ratio_font)
    };

    // 从 1 行开始：优先单行（必要时缩小字号），再考虑多行
    for lines in 1..=MAX_LINES {
        let f = font_for(lines);
        if f >= comfortable_font && fits_in_lines(font, text, f, lines, max_line_width) {
            return (f, lines);
        }
    }

    // 兜底：维持用户设定的字号，用能放下全文的最少行数
    for lines in 1..=MAX_LINES {
        if fits_in_lines(font, text, ratio_font, lines, max_line_width) {
            return (ratio_font, lines);
        }
    }

    (font_for(MAX_LINES), MAX_LINES)
}

/// 均分成 `lines` 行后，最长那一行有几个字符。
fn max_segment_len(text: &str, lines: usize) -> usize {
    text.split('\n')
        .map(|seg| {
            let n = seg.chars().count();
            (n as f32 / lines as f32).ceil() as usize
        })
        .max()
        .unwrap_or(1)
        .max(1)
}

/// 把每个显式换行段均分成 `lines` 份后，是否每一行都不超过宽度上限。
fn fits_in_lines(
    font: &FontRef,
    text: &str,
    font_size: f32,
    lines: usize,
    max_line_width: f32,
) -> bool {
    let limit = max_line_width + WIDTH_TOLERANCE;
    let scale = scale_for(font, font_size);
    text.split('\n').all(|segment| {
        let chars: Vec<char> = segment.chars().collect();
        split_evenly(&chars, lines)
            .iter()
            .all(|part| measure_text_width(font, scale, &part.iter().collect::<String>()) <= limit)
    })
}

/// 按目标行数排版：优先把整段均分成 `target_lines` 行，
/// 若某行仍超宽则再按宽度切一次，保证文字完整且不超宽。
fn wrap_text(
    font: &FontRef,
    scale: PxScale,
    text: &str,
    max_width: f32,
    target_lines: usize,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    for segment in text.split('\n') {
        let chars: Vec<char> = segment.chars().collect();
        if chars.is_empty() {
            lines.push(String::new());
            continue;
        }

        for part in split_evenly(&chars, target_lines) {
            // 均分后仍超宽（例如超长英文单词）则按宽度再切
            for piece in split_by_width(font, scale, &part, max_width) {
                lines.push(piece);
            }
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// 把字符尽量均分成 `n` 份，余数分给靠前的份。
fn split_evenly(chars: &[char], n: usize) -> Vec<Vec<char>> {
    let n = n.max(1).min(chars.len());
    let base = chars.len() / n;
    let remainder = chars.len() % n;

    let mut parts = Vec::with_capacity(n);
    let mut start = 0usize;
    for i in 0..n {
        let take = base + usize::from(i < remainder);
        let end = (start + take).min(chars.len());
        parts.push(chars[start..end].to_vec());
        start = end;
    }
    parts
}

/// 按可用宽度做最后的兜底切分。
fn split_by_width(font: &FontRef, scale: PxScale, chars: &[char], max_width: f32) -> Vec<String> {
    if chars.is_empty() {
        return vec![String::new()];
    }

    let mut out = Vec::new();
    let mut current = String::new();
    let mut width = 0.0f32;

    for &ch in chars {
        let w = measure_text_width(font, scale, &ch.to_string());
        if !current.is_empty() && width + w > max_width + WIDTH_TOLERANCE {
            out.push(std::mem::take(&mut current));
            width = 0.0;
        }
        current.push(ch);
        width += w;
    }
    if !current.is_empty() {
        out.push(current);
    }

    out
}
