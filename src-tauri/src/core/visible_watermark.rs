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
            line_spacing: 2.0,
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
    fn test_layout_wraps_without_shrinking_when_font_is_readable() {
        // 13 个字 13×36=468px > 420px 单行上限。
        // 应换行，但**保持用户设定的字号**，而不是缩到很小硬塞进一行。
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
        assert_eq!(layout.lines.len(), 2, "单行放不下应换行");
        assert!(
            layout.font_size >= 1200.0 * 0.02,
            "换行时字号不应低于可读阈值，实际 {:.1}",
            layout.font_size
        );
        let joined: String = layout.lines.concat();
        assert_eq!(joined, text, "文字不应被截断");
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
}