use image::{DynamicImage, ImageFormat};

pub fn strip_metadata(image: &mut DynamicImage, format: ImageFormat) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();

    match format {
        ImageFormat::Jpeg => {
            image.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Jpeg)
                .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
        }
        ImageFormat::Png => {
            image.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Png)
                .map_err(|e| format!("Failed to encode PNG: {}", e))?;
        }
        _ => {
            image.write_to(&mut std::io::Cursor::new(&mut buffer), format)
                .map_err(|e| format!("Failed to encode image: {}", e))?;
        }
    }

    Ok(buffer)
}
