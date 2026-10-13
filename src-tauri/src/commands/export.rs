use crate::core::config::{OutputFormat, WatermarkConfig};
use crate::core::pdf_watermark::add_pdf_watermark;
use crate::core::pipeline::process_image;
use tauri_plugin_dialog::DialogExt;
use std::path::Path;
use percent_encoding::percent_decode_str;

fn url_to_path(url: &str) -> String {
    let s = url.strip_prefix("file://").unwrap_or(url);
    let decoded = percent_decode_str(s).decode_utf8_lossy();
    decoded.into_owned()
}

#[tauri::command]
pub async fn export_file(
    app: tauri::AppHandle,
    input_path: String,
    config: WatermarkConfig,
) -> Result<(), String> {
    let input_path_buf = Path::new(&input_path);
    let input_stem = input_path_buf
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("watermarked");
    let input_ext = input_path_buf
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let target_ext = match config.output.format {
        OutputFormat::Jpeg => "jpg",
        OutputFormat::Png => "png",
        OutputFormat::Heic => "heic",
        OutputFormat::SameAsInput => {
            if input_ext == "heic" || input_ext == "heif" {
                "heic"
            } else if input_ext == "png" {
                "png"
            } else if input_ext == "pdf" {
                "pdf"
            } else {
                "jpg"
            }
        }
    };

    let default_name = format!("{}_watermarked.{}", input_stem, target_ext);

    let output_path = app
        .dialog()
        .file()
        .set_file_name(&default_name)
        .add_filter("Image", &[target_ext])
        .blocking_save_file();

    let output_path = match output_path {
        Some(path) => url_to_path(&path.to_string()),
        None => return Err("No output path selected".to_string()),
    };

    // Ensure the output path has the correct target extension
    let output_path = {
        let path_buf = Path::new(&output_path);
        let parent = path_buf.parent().unwrap_or_else(|| Path::new("."));
        let user_stem = path_buf.file_stem().and_then(|s| s.to_str()).unwrap_or(&default_name[..default_name.len()-target_ext.len()-1]);
        parent.join(format!("{}.{}", user_stem, target_ext)).to_string_lossy().to_string()
    };

    if input_ext == "pdf" {
        add_pdf_watermark(&input_path, &output_path, &config.visible)?;
    } else {
        process_image(&input_path, &output_path, &config)?;
    }

    Ok(())
}
