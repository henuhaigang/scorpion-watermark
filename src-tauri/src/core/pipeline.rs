use crate::core::config::{OutputFormat, WatermarkConfig};
use crate::core::image_loader::load_image;
use crate::core::invisible_watermark::embed_invisible_watermark;
use crate::core::metadata_cleaner::strip_metadata;
use crate::core::visible_watermark::apply_visible_watermark;
use image::ImageFormat;

#[derive(Debug, Clone, Copy, PartialEq)]
enum OutputImageType {
    Jpeg,
    Png,
    Heic,
}

pub fn process_image(
    input_path: &str,
    output_path: &str,
    config: &WatermarkConfig,
) -> Result<(), String> {
    let mut image = load_image(input_path)?;

    apply_visible_watermark(&mut image, &config.visible)?;

    if let Some(ref invisible) = config.invisible {
        embed_invisible_watermark(&mut image, invisible)?;
    }

    let output_type = determine_output_type(input_path, &config.output.format)?;

    if output_type == OutputImageType::Heic {
        #[cfg(target_os = "macos")]
        {
            crate::core::heic_io::save_heic(&image, output_path)?;
            return Ok(());
        }
        #[cfg(not(target_os = "macos"))]
        {
            return Err("HEIC output is only supported on macOS".to_string());
        }
    }

    let image_format = match output_type {
        OutputImageType::Jpeg => ImageFormat::Jpeg,
        OutputImageType::Png => ImageFormat::Png,
        OutputImageType::Heic => unreachable!(),
    };

    let encoded = if config.output.strip_metadata {
        strip_metadata(&mut image, image_format)?
    } else {
        let mut buffer = Vec::new();
        image.write_to(&mut std::io::Cursor::new(&mut buffer), image_format)
            .map_err(|e| format!("Failed to encode image: {}", e))?;
        buffer
    };

    std::fs::write(output_path, encoded).map_err(|e| format!("Failed to write output: {}", e))?;

    Ok(())
}

fn determine_output_type(
    input_path: &str,
    format: &OutputFormat,
) -> Result<OutputImageType, String> {
    match format {
        OutputFormat::Jpeg => Ok(OutputImageType::Jpeg),
        OutputFormat::Png => Ok(OutputImageType::Png),
        OutputFormat::Heic => Ok(OutputImageType::Heic),
        OutputFormat::SameAsInput => {
            let ext = input_path.rsplit('.').next().unwrap_or("").to_lowercase();
            match ext.as_str() {
                "jpg" | "jpeg" => Ok(OutputImageType::Jpeg),
                "png" => Ok(OutputImageType::Png),
                "heic" | "heif" => Ok(OutputImageType::Heic),
                _ => Err(format!("Unsupported output format for: {}", ext)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::{OutputConfig, TileMode, VisibleWatermark};
    use image::{Rgb, RgbImage, Rgba, RgbaImage};

    fn test_config(format: OutputFormat) -> WatermarkConfig {
        WatermarkConfig {
            visible: VisibleWatermark {
                enabled: true,
                text: "机密文件".to_string(),
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
            },
            invisible: None,
            output: OutputConfig {
                format,
                strip_metadata: true,
            },
        }
    }

    fn count_non_white(path: &str) -> u64 {
        let img = image::open(path).unwrap().to_rgba8();
        img.pixels()
            .filter(|p| p.0[0] < 200 || p.0[1] < 200 || p.0[2] < 200)
            .count() as u64
    }

    #[test]
    fn test_export_png_writes_watermark() {
        let dir = std::env::temp_dir();
        let input = dir.join("wm_e2e_in.png");
        let output = dir.join("wm_e2e_out.png");

        // 白色底图，水印应为黑色
        RgbaImage::from_pixel(800, 600, Rgba([255, 255, 255, 255]))
            .save(&input)
            .unwrap();

        process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &test_config(OutputFormat::Png),
        )
        .expect("process_image should succeed");

        let non_white = count_non_white(output.to_str().unwrap());
        println!("exported png non-white pixels: {}", non_white);
        assert!(non_white > 500, "exported PNG should contain watermark pixels");
    }

    #[test]
    fn test_export_jpeg_writes_watermark() {
        let dir = std::env::temp_dir();
        let input = dir.join("wm_e2e_in.jpg");
        let output = dir.join("wm_e2e_out.jpg");

        RgbImage::from_pixel(800, 600, Rgb([255, 255, 255]))
            .save(&input)
            .unwrap();

        process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &test_config(OutputFormat::SameAsInput),
        )
        .expect("process_image should succeed");

        let non_white = count_non_white(output.to_str().unwrap());
        println!("exported jpeg non-white pixels: {}", non_white);
        assert!(non_white > 500, "exported JPEG should contain watermark pixels");
    }

    #[test]
    fn test_export_rgba_input_to_jpeg() {
        let dir = std::env::temp_dir();
        let input = dir.join("wm_e2e_alpha.png");
        let output = dir.join("wm_e2e_alpha_out.jpg");

        RgbaImage::from_pixel(800, 600, Rgba([255, 255, 255, 128]))
            .save(&input)
            .unwrap();

        let result = process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &test_config(OutputFormat::Jpeg),
        );

        match &result {
            Ok(_) => println!("RGBA->JPEG export OK, non-white: {}", count_non_white(output.to_str().unwrap())),
            Err(e) => println!("RGBA->JPEG export FAILED: {}", e),
        }
        result.expect("RGBA input exported as JPEG should not fail");
    }
}
