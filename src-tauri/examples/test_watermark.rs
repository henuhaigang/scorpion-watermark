use scorpion_watermark_lib::core::config::{TileMode, VisibleWatermark};
use scorpion_watermark_lib::core::visible_watermark::apply_visible_watermark;
use image::{DynamicImage, Rgb, RgbImage};

fn main() {
    let mut img = DynamicImage::ImageRgb8(RgbImage::from_pixel(800, 600, Rgb([255, 255, 255])));

    let config = VisibleWatermark {
        enabled: true,
        text: "仅供办理人保车险使用的时候".to_string(),
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
    };

    apply_visible_watermark(&mut img, &config).unwrap();
    img.save("/tmp/test_watermark.png").unwrap();
    println!("Test image saved to /tmp/test_watermark.png");
}