use crate::core::config::{TileMode, VisibleWatermark};
use crate::core::layout::calculate_layout;
use crate::core::text_renderer::{render_text, RenderParams};
use image::DynamicImage;

pub fn apply_visible_watermark(
    image: &mut DynamicImage,
    config: &VisibleWatermark,
) -> Result<(), String> {
    if !config.enabled || config.text.is_empty() {
        return Ok(());
    }

    let layout = calculate_layout(image.width(), image.height(), config)?;
    if layout.items.is_empty() {
        return Ok(());
    }

    let mut render_at = |x: f32, y: f32, rotation: f32| -> Result<(), String> {
        render_text(
            image,
            &RenderParams {
                x,
                y,
                rotation,
                text: &layout.text,
                font_size: layout.font_size,
                line_spacing: config.line_spacing,
                color: &config.color,
                opacity: config.opacity,
            },
        )
    };

    match config.mode {
        TileMode::Tile => {
            for item in &layout.items {
                render_at(item.x, item.y, item.rotation)?;
            }
        }
        TileMode::Single => {
            if let Some(first) = layout.items.first() {
                render_at(first.x, first.y, first.rotation)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::font::LINE_HEIGHT_BASE;
    use crate::core::layout::LayoutResult;
        use crate::core::config::VisibleWatermark;
    use image::{Rgba, RgbaImage};

    fn test_config(text: &str) -> VisibleWatermark {
        VisibleWatermark {
            enabled: true,
            text: text.to_string(),
            mode: TileMode::Tile,
            angle: -45.0,
            opacity: 0.5,
            font_size: 36.0,
            font_size_ratio: 3.0,
            line_spacing: 0.5,
            block_gap_ratio: 1.0,
            color: "#000000".to_string(),
            stroke_color: None,
            stroke_width: 0.0,
            shadow: None,
            position: None,
            custom_xy: None,
        }
    }

    #[test]
    fn test_layout_never_truncates_across_lengths() {
        let cases = [
            "机密",
            "机密文件",
            "仅供办理车险使用",
            "仅供用来办理车险使用",
            "仅供办理人保车险使用的时候",
            "仅供办理人保车险使用的时候此文件仅作内部使用不得外传",
        ];
        for text in cases {
            for ratio in [1.0f32, 3.0, 5.0] {
                let mut config = test_config(text);
                config.font_size_ratio = ratio;
                let layout = calculate_layout(1200, 900, &config).unwrap();
                let joined: String = layout.lines.concat();
                let pct = layout.font_size / 1200.0 * 100.0;
                println!(
                    "len={:2} ratio={:.1} lines={} font={:.0}px({:.2}%) intact={} shrink={}",
                    text.chars().count(),
                    ratio,
                    layout.lines.len(),
                    layout.font_size,
                    pct,
                    joined == text,
                    layout.font_auto_shrunk
                );
                assert_eq!(joined, text, "文字必须完整，不得截断");
                assert!(layout.lines.len() <= 3);
                assert!(pct >= 1.5 - 0.01, "字号不得低于 1.5%");
            }
        }
    }

    #[test]
    fn test_layout_13_chars_across_canvas_sizes() {
        let text = "仅供办理人保车险使用的时候";
        for w in [800u32, 1200, 2000, 4000] {
            let config = test_config(text);
            let layout = calculate_layout(w, 600, &config).unwrap();
            let joined: String = layout.lines.concat();
            println!(
                "W={} font_size={:.1} lines={:?} block={:.0}x{:.0} intact={} joined_len={}",
                w,
                layout.font_size,
                layout.lines,
                layout.text_width,
                layout.text_height,
                joined == text,
                joined.chars().count()
            );
        }
    }

    #[test]
    fn test_layout_prefers_single_line_when_it_fits_at_ratio_font() {
        // 9 个字在默认比例下 9×36=324px ≤ 420px 上限，应保持单行
        let text = "仅用来办理车险使用";
        let config = test_config(text);
        let layout = calculate_layout(1200, 900, &config).unwrap();
        println!(
            "text={} lines={} font={:.1} shrink={}",
            text,
            layout.lines.len(),
            layout.font_size,
            layout.font_auto_shrunk
        );
        assert_eq!(layout.lines.len(), 1, "设定字号下放得下就该单行");
        assert!(!layout.font_auto_shrunk, "不应缩小字号");
        let joined: String = layout.lines.concat();
        assert_eq!(joined, text, "文字不应被截断");
    }

    #[test]
    fn test_layout_shrinks_to_keep_single_line_when_still_readable() {
        // 13 个字在设定字号下需要 13×36=468px > 420px 单行上限。
        // 缩到 420/13≈32px 仍高于可读阈值(24px)，因此应缩小字号保持单行，
        // 而不是换行 —— 用户期望「文字变多时自动缩小让水印完整显示」。
        let text = "仅供办理人保车险使用的时候";
        let config = test_config(text);
        let layout = calculate_layout(1200, 900, &config).unwrap();
        println!(
            "text={} lines={:?} font={:.1} shrink={}",
            text,
            layout.lines,
            layout.font_size,
            layout.font_auto_shrunk
        );
        assert_eq!(layout.lines.len(), 1, "缩到可读阈值以内就该保持单行");
        assert!(
            layout.font_size >= 1200.0 * 0.02,
            "缩小后的字号不应低于可读阈值，实际 {:.1}",
            layout.font_size
        );
        assert!(
            layout.font_size < 36.0,
            "确实应缩小字号（设定值 36），实际 {:.1}",
            layout.font_size
        );
        let joined: String = layout.lines.concat();
        assert_eq!(joined, text, "文字不应被截断");
    }

    #[test]
    fn test_layout_wraps_when_single_line_would_be_too_small() {
        // 26 个字若压成单行需 420/26≈16px，低于可读阈值(24px)。
        // 此时应改用 2 行换取更大的字号，保证看得见。
        let text = "仅供办理人保车险使用的时候此文件仅作内部使用不得外传";
        let config = test_config(text);
        let layout = calculate_layout(1200, 900, &config).unwrap();
        println!(
            "text={} lines={:?} font={:.1}",
            text, layout.lines, layout.font_size
        );
        assert!(
            layout.lines.len() > 1,
            "单行需要缩得太小，应换行而不是硬塞"
        );
        assert!(
            layout.font_size >= 1200.0 * 0.02,
            "换行时字号不应低于可读阈值，实际 {:.1}",
            layout.font_size
        );
        let joined: String = layout.lines.concat();
        assert_eq!(joined, text, "文字不应被截断");
    }

    #[test]
    fn test_line_spacing_zero_keeps_single_line_height() {
        // 行间距 0 表示紧凑单倍行距：行高应等于字号本身，而不是归零导致各行重叠
        let text = "仅供办理人保车险使用的时候";
        let mut config = test_config(text);
        config.line_spacing = 0.0;
        let layout = calculate_layout(1200, 900, &config).unwrap();
        assert!(
            layout.text_height >= layout.font_size * layout.lines.len() as f32 - 0.01,
            "行间距为 0 时行高仍应至少为一个字号，实际 text_height={:.1} font_size={:.1} lines={}",
            layout.text_height,
            layout.font_size,
            layout.lines.len()
        );
    }

    #[test]
    fn test_line_spacing_monotonic() {
        // 行间距越大，行高越高
        let text = "仅供办理人保车险使用的时候";
        let mut prev = 0.0f32;
        for spacing in [0.0f32, 0.5, 1.0, 1.5] {
            let mut config = test_config(text);
            config.line_spacing = spacing;
            let layout = calculate_layout(1200, 900, &config).unwrap();
            assert!(
                layout.text_height > prev,
                "行间距 {} 的行高({})应大于上一档({})",
                spacing,
                layout.text_height,
                prev
            );
            prev = layout.text_height;
        }
    }

    #[test]
    fn test_layout_wraps_when_font_would_be_too_small() {
        // 26 个字：单行会让字号低于可读阈值，应换行换取更大字号
        let text = "仅供办理人保车险使用的时候此文件仅作内部使用";
        let config = test_config(text);
        let layout = calculate_layout(1200, 900, &config).unwrap();
        println!(
            "lines={:?} font={:.1} ({:.2}%) shrink={}",
            layout.lines,
            layout.font_size,
            layout.font_size / 12.0,
            layout.font_auto_shrunk
        );
        assert!(layout.lines.len() > 1, "超长文本应换行以保持字号可读");
        let joined: String = layout.lines.concat();
        assert_eq!(joined, text, "文字不应被截断");
    }

    #[test]
    fn test_layout_keeps_short_text_on_one_line() {
        let config = test_config("机密文件");
        let layout = calculate_layout(800, 600, &config).unwrap();
        assert_eq!(layout.lines.len(), 1);
    }

    #[test]
    fn test_watermark_renders_pixels() {
        let mut img = DynamicImage::ImageRgba8(RgbaImage::new(800, 600));
        let before: Vec<u8> = img.to_rgba8().as_raw().clone();
        apply_visible_watermark(&mut img, &test_config("机密文件")).unwrap();
        let after_img = img.to_rgba8();
        let changed = before
            .chunks_exact(4)
            .zip(after_img.as_raw().chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        println!("changed pixels: {}", changed);
        assert!(changed > 100, "watermark should change many pixels");
    }

    #[test]
    fn test_long_text_watermark_renders() {
        let mut img = DynamicImage::ImageRgba8(RgbaImage::new(800, 600));
        let before: Vec<u8> = img.to_rgba8().as_raw().clone();
        apply_visible_watermark(&mut img, &test_config("仅用来办理车险使用")).unwrap();
        let after_img = img.to_rgba8();
        let changed = before
            .chunks_exact(4)
            .zip(after_img.as_raw().chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        println!("long text changed pixels: {}", changed);
        assert!(changed > 500, "long text watermark should render");
    }

    #[test]
    fn test_alpha_stays_opaque_after_overlay() {
        let mut img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            800,
            600,
            Rgba([255, 255, 255, 255]),
        ));
        apply_visible_watermark(&mut img, &test_config("机密文件")).unwrap();
        let out = img.to_rgba8();
        let min_alpha = out.pixels().map(|p| p.0[3]).min().unwrap();
        println!("min alpha after overlay: {}", min_alpha);
        assert_eq!(min_alpha, 255, "不透明底图叠加水印后 alpha 应保持 255");
    }

 
 
    #[test]
    fn test_line_spacing_zero_is_tightest_line_height() {
        // 行间距 0 应给出最紧凑的行高（恰为一个字号）
        let text = "仅供办理人保车险使用的时候此文件仅作内部使用";
        let mut c = test_config(text);
        c.line_spacing = 0.0;
        let l = calculate_layout(1200, 900, &c).unwrap();
        let expected = l.font_size * l.lines.len() as f32;
        assert!(
            (l.text_height - expected).abs() < 0.01,
            "行间距 0 时行高应恰为一个字号，实际 text_height={} 期望={} font={} lines={}",
            l.text_height,
            expected,
            l.font_size,
            l.lines.len()
        );
    }

    /// 取网格中相邻两行的中心距（去重后取最小正间隔）
    fn row_step(c: &VisibleWatermark) -> (LayoutResult, f32) {
        let mut c = c.clone();
        c.angle = 0.0; // 正交网格，才能直接用 y 坐标量行距
        let l = calculate_layout(1200, 900, &c).unwrap();
        let mut ys: Vec<f32> = l.items.iter().map(|i| i.y).collect();
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ys.dedup_by(|a, b| (*a - *b).abs() < 0.01);
        let step = ys.windows(2).map(|w| w[1] - w[0]).fold(f32::MAX, f32::min);
        (l, step)
    }

    #[test]
    fn test_block_gap_zero_makes_rows_touch() {
        // 滑块语义是「两行水印之间留多少空白」，0 必须真正紧贴。
        // 曾用 max(字号×滑块值, 行高) 兜底，导致 0 时仍留一个行高的空白。
        for ls in [0.0f32, 0.5, 1.5] {
            let mut c = test_config("机密文件");
            c.line_spacing = ls;
            c.block_gap_ratio = 0.0;
            let (l, step) = row_step(&c);
            let ink_gap = step - l.font_size;
            assert!(
                ink_gap.abs() < 0.01,
                "行间距 {} 时 gap=0 应紧贴，实际残留空白 {}",
                ls,
                ink_gap
            );
        }
    }

    #[test]
    fn test_block_gap_scales_row_spacing() {
        // 单行块：行中心距 = 字号 + 块间距。块间距每加 1.0 即多一个字号。
        for gap in [0.0f32, 0.5, 1.0, 2.0] {
            let mut c = test_config("机密文件");
            c.block_gap_ratio = gap;
            let (l, step) = row_step(&c);
            let expected = l.font_size + l.font_size * gap;
            assert!(
                (step - expected).abs() < 0.01,
                "gap {} 的行中心距应为 {}，实际 {}",
                gap,
                expected,
                step
            );
        }
    }

    #[test]
    fn test_block_gap_zero_with_multiline_block() {
        // 多行块：行中心距 = 墨迹高度 = 字号 + (行数-1)×行高
        for ls in [0.0f32, 0.5, 1.5] {
            let mut c = test_config("仅供办理人保车险使用的时候此文件仅作内部使用");
            c.line_spacing = ls;
            c.block_gap_ratio = 0.0;
            let (l, step) = row_step(&c);
            let line_height = l.font_size * (LINE_HEIGHT_BASE + ls);
            let expected = l.font_size + line_height * (l.lines.len() - 1) as f32;
            assert!(
                (step - expected).abs() < 0.01,
                "多行块(行间距 {}) gap=0 的行中心距应为墨迹高度 {}，实际 {}",
                ls,
                expected,
                step
            );
        }
    }

    /// 四角与四边中点到最近水印中心的距离，减去水印半尺寸。
    /// 返回值 ≤ 0 表示该处被水印覆盖；正值即空白缺口。
    fn worst_edge_gap(c: &VisibleWatermark) -> (f32, String) {
        let w = 1200.0f32;
        let h = 900.0f32;
        let l = calculate_layout(w as u32, h as u32, c).unwrap();
        let half = (l.text_width / 2.0).max(l.text_height / 2.0);
        let pts = [
            ("左上", 0.0, 0.0),
            ("右上", w, 0.0),
            ("左下", 0.0, h),
            ("右下", w, h),
            ("上边中点", w / 2.0, 0.0),
            ("下边中点", w / 2.0, h),
            ("左边中点", 0.0, h / 2.0),
            ("右边中点", w, h / 2.0),
        ];
        let mut worst = f32::MIN;
        let mut worst_name = String::new();
        for (name, px, py) in pts {
            let d = l
                .items
                .iter()
                .map(|it| ((it.x - px).powi(2) + (it.y - py).powi(2)).sqrt())
                .fold(f32::MAX, f32::min);
            let gap = d - half;
            if gap > worst {
                worst = gap;
                worst_name = name.to_string();
            }
        }
        (worst, worst_name)
    }

    #[test]
    fn test_no_blank_at_canvas_corners() {
        // 网格在旋转坐标系下生成，早期按对角线估算行列数并按圆形半径裁剪，
        // 导致画布四角落在裁剪边界上被剔掉，左上/右上出现空白。
        // 字号越大、行距越大越明显（实测字号 96px 时左上缺 21.7px）。
        for ratio in [0.5f32, 1.0, 3.0, 5.0, 6.0, 8.0] {
            for angle in [-90.0f32, -45.0, -30.0, 0.0, 15.0, 45.0, 90.0] {
                let mut c = test_config("机密文件");
                c.font_size_ratio = ratio;
                c.angle = angle;
                let (gap, name) = worst_edge_gap(&c);
                assert!(
                    gap <= 0.0,
                    "字号比例 {}、角度 {} 时 {} 有 {:.1}px 空白",
                    ratio,
                    angle,
                    name,
                    gap
                );
            }
        }
    }

    #[test]
    fn test_no_blank_at_canvas_corners_multiline() {
        // 多行块与不同行间距下同样不得留白
        for ls in [0.0f32, 0.5, 1.5] {
            for gap in [0.0f32, 1.0, 3.0] {
                let mut c = test_config("仅供办理人保车险使用的时候此文件仅作内部使用");
                c.line_spacing = ls;
                c.block_gap_ratio = gap;
                let (g, name) = worst_edge_gap(&c);
                assert!(
                    g <= 0.0,
                    "行间距 {}、水印间距 {} 时 {} 有 {:.1}px 空白",
                    ls,
                    gap,
                    name,
                    g
                );
            }
        }
    }

    #[test]
    fn test_grid_columns_not_missing() {
        // Rust 整数除法向零截断，`-count_x / 2` 会让范围左右差一列，
        // 即画布一侧整列缺失。这里检查左右跨度之差不超过一个间距
        // （奇数行刻意错开半格，故允许半格误差）。
        for angle in [-90.0f32, -45.0, -30.0, 0.0, 45.0, 90.0] {
            let mut c = test_config("机密文件");
            c.angle = angle;
            let l = calculate_layout(1200, 900, &c).unwrap();
            let cx = 600.0f32;
            let min_x = l.items.iter().map(|i| i.x).fold(f32::MAX, f32::min);
            let max_x = l.items.iter().map(|i| i.x).fold(f32::MIN, f32::max);
            let left = cx - min_x;
            let right = max_x - cx;
            assert!(
                (left - right).abs() <= l.text_width + 1.0,
                "角度 {} 时左右跨度相差过大：左 {:.1} 右 {:.1}（字宽 {:.1}）",
                angle,
                left,
                right,
                l.text_width
            );
        }
    }
}
