use image::ImageFormat;
use exif::Reader;
use std::io::Cursor;

pub fn load_image(path: &str) -> Result<image::DynamicImage, String> {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();

    if ext == "heic" || ext == "heif" {
        #[cfg(target_os = "macos")]
        {
            return crate::core::heic_io::load_heic(path);
        }
        #[cfg(not(target_os = "macos"))]
        {
            return Err("HEIC is only supported on macOS".to_string());
        }
    }

    let data = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let format = guess_format(path, &data)?;

    let mut img = image::load_from_memory_with_format(&data, format)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    // Apply EXIF orientation for JPEG images
    if format == ImageFormat::Jpeg {
        img = apply_exif_orientation(img, &data)?;
    }

    Ok(img)
}

fn apply_exif_orientation(mut img: image::DynamicImage, data: &[u8]) -> Result<image::DynamicImage, String> {
    let mut cursor = Cursor::new(data);
    if let Ok(exif) = Reader::new().read_from_container(&mut cursor) {
        if let Some(field) = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
            if let exif::Value::Short(ref orient_vals) = field.value {
                if let Some(orient) = orient_vals.first() {
                    img = match *orient {
                        2 => img.fliph(),
                        3 => img.rotate180(),
                        4 => img.flipv(),
                        5 => img.fliph().rotate90(),
                        6 => img.rotate90(),
                        7 => img.fliph().rotate270(),
                        8 => img.rotate270(),
                        _ => img,
                    };
                }
            }
        }
    }
    Ok(img)
}

fn guess_format(path: &str, data: &[u8]) -> Result<ImageFormat, String> {
    if let Ok(fmt) = ImageFormat::from_path(path) {
        return Ok(fmt);
    }

    if data.len() >= 8 {
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Ok(ImageFormat::Jpeg);
        }
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Ok(ImageFormat::Png);
        }
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return Ok(ImageFormat::Gif);
        }
        if data.starts_with(b"RIFF") && data.len() >= 12 && &data[8..12] == b"WEBP" {
            return Ok(ImageFormat::WebP);
        }
    }

    Err("Unsupported image format".to_string())
}
