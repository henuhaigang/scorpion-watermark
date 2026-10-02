use crate::core::font::{font, scale_for, LINE_HEIGHT_BASE};

use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, text_size};

pub struct RenderParams<'a> {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    /// 可包含 `\n` 的多行文本
    pub text: &'a str,
    pub font_size: f32,
    pub line_spacing: f32,
    pub color: &'a str,
    pub opacity: f32,
}

pub fn render_text(image: &mut DynamicImage, params: &RenderParams) -> Result<(), String> {
    let font = font()?;
    let scale = scale_for(font, params.font_size);

    let lines: Vec<&str> = params.text.split('\n').collect();
    if lines.is_empty() {
        return Ok(());
    }

    let padding = params.font_size * 0.3;
    let line_height = params.font_size * (LINE_HEIGHT_BASE + params.line_spacing.max(0.0));

    // 用 imageproc 的 text_size 测量真实墨迹范围，避免缓冲区过小切掉字形
    let mut max_line_width = 0.0f32;
    let mut max_line_height = 0.0f32;
    for line in &lines {
        let (w, h) = text_size(scale, font, line);
        max_line_width = max_line_width.max(w as f32);
        max_line_height = max_line_height.max(h as f32);
    }

    let block_height = line_height * lines.len() as f32;
    let buf_width = ((max_line_width + padding * 2.0).ceil() as u32).max(1);
    // 额外留出单行墨迹高度作为余量，确保任何字体度量下都不会裁切
    let buf_height =
        ((block_height + padding * 2.0 + max_line_height).ceil() as u32).max(1);

    let mut text_buf = RgbaImage::from_pixel(buf_width, buf_height, Rgba([0, 0, 0, 0]));

    let parsed_color = parse_color(params.color)?;
    let alpha = (params.opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    let fill_color = Rgba([parsed_color[0], parsed_color[1], parsed_color[2], alpha]);

    let mut line_y = padding;
    for line in &lines {
        draw_text_mut(
            &mut text_buf,
            fill_color,
            padding.round() as i32,
            line_y.round() as i32,
            scale,
            font,
            line,
        );
        line_y += line_height;
    }

    let rotated_buf = rotate_image(&text_buf, params.rotation);

    let paste_x = (params.x - rotated_buf.width() as f32 / 2.0).round() as i32;
    let paste_y = (params.y - rotated_buf.height() as f32 / 2.0).round() as i32;

    overlay_image(image, &rotated_buf, paste_x, paste_y);

    Ok(())
}

fn rotate_image(img: &RgbaImage, angle_deg: f32) -> RgbaImage {
    if angle_deg == 0.0 {
        return img.clone();
    }

    let angle_rad = angle_deg.to_radians();
    let cos = angle_rad.cos();
    let sin = angle_rad.sin();

    let (w, h) = (img.width() as f32, img.height() as f32);
    let new_w = ((w * cos.abs() + h * sin.abs()).ceil() as u32).max(1);
    let new_h = ((w * sin.abs() + h * cos.abs()).ceil() as u32).max(1);

    let mut rotated = RgbaImage::from_pixel(new_w, new_h, Rgba([0, 0, 0, 0]));

    let cx = w / 2.0;
    let cy = h / 2.0;
    let ncx = new_w as f32 / 2.0;
    let ncy = new_h as f32 / 2.0;

    for y in 0..new_h {
        for x in 0..new_w {
            let dx = x as f32 - ncx;
            let dy = y as f32 - ncy;

            // 反向映射：目标像素 -> 源像素
            let src_x = dx * cos + dy * sin + cx;
            let src_y = -dx * sin + dy * cos + cy;

            if src_x >= 0.0 && src_x < w && src_y >= 0.0 && src_y < h {
                let sx = (src_x.floor() as u32).min(img.width() - 1);
                let sy = (src_y.floor() as u32).min(img.height() - 1);
                let fx = src_x - sx as f32;
                let fy = src_y - sy as f32;

                let p00 = *img.get_pixel(sx, sy);
                let p10 = *img.get_pixel((sx + 1).min(img.width() - 1), sy);
                let p01 = *img.get_pixel(sx, (sy + 1).min(img.height() - 1));
                let p11 = *img.get_pixel(
                    (sx + 1).min(img.width() - 1),
                    (sy + 1).min(img.height() - 1),
                );

                rotated.put_pixel(x, y, bilinear_interpolate(p00, p10, p01, p11, fx, fy));
            }
        }
    }

    rotated
}

fn bilinear_interpolate(
    p00: Rgba<u8>,
    p10: Rgba<u8>,
    p01: Rgba<u8>,
    p11: Rgba<u8>,
    fx: f32,
    fy: f32,
) -> Rgba<u8> {
    let w00 = (1.0 - fx) * (1.0 - fy);
    let w10 = fx * (1.0 - fy);
    let w01 = (1.0 - fx) * fy;
    let w11 = fx * fy;

    let mix = |a: u8, b: u8, c: u8, d: u8| {
        (a as f32 * w00 + b as f32 * w10 + c as f32 * w01 + d as f32 * w11).round() as u8
    };

    Rgba([
        mix(p00[0], p10[0], p01[0], p11[0]),
        mix(p00[1], p10[1], p01[1], p11[1]),
        mix(p00[2], p10[2], p01[2], p11[2]),
        mix(p00[3], p10[3], p01[3], p11[3]),
    ])
}

/// 标准的 source-over alpha 合成。
///
/// 旧实现把 alpha 自己乘了两次（`a * a`），导致不透明底图的水印区域
/// alpha 从 255 掉到约 201，导出 JPEG 时出现半透明，PNG 输出则整体发虚，
/// 且与前端 Konva 预览的合成结果不一致。
fn overlay_image(base: &mut DynamicImage, overlay: &RgbaImage, x: i32, y: i32) {
    let mut base_rgba = base.to_rgba8();
    let (base_w, base_h) = (base_rgba.width() as i32, base_rgba.height() as i32);
    let (ov_w, ov_h) = (overlay.width() as i32, overlay.height() as i32);

    for oy in 0..ov_h {
        for ox in 0..ov_w {
            let bx = x + ox;
            let by = y + oy;
            if bx < 0 || bx >= base_w || by < 0 || by >= base_h {
                continue;
            }

            let src = *overlay.get_pixel(ox as u32, oy as u32);
            if src[3] == 0 {
                continue;
            }
            let dst = *base_rgba.get_pixel(bx as u32, by as u32);

            let sa = src[3] as f32 / 255.0;
            let da = dst[3] as f32 / 255.0;
            let oa = sa + da * (1.0 - sa);

            if oa <= 0.0 {
                base_rgba.put_pixel(bx as u32, by as u32, Rgba([0, 0, 0, 0]));
                continue;
            }

            let ch = |s: u8, d: u8| -> u8 {
                let v = (s as f32 * sa + d as f32 * da * (1.0 - sa)) / oa;
                v.round().clamp(0.0, 255.0) as u8
            };

            base_rgba.put_pixel(
                bx as u32,
                by as u32,
                Rgba([
                    ch(src[0], dst[0]),
                    ch(src[1], dst[1]),
                    ch(src[2], dst[2]),
                    (oa * 255.0).round().clamp(0.0, 255.0) as u8,
                ]),
            );
        }
    }

    *base = DynamicImage::ImageRgba8(base_rgba);
}

fn parse_color(hex: &str) -> Result<Rgba<u8>, String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(format!("Invalid color: {}", hex));
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
    let b = u8::from_str_radix(&hex[4..5 + 1], 16).map_err(|e| e.to_string())?;

    Ok(Rgba([r, g, b, 255]))
}