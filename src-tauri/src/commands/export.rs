use crate::core::config::{OutputFormat, WatermarkConfig};
use crate::core::pdf_watermark::add_pdf_watermark;
use crate::core::pipeline::process_image;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn export_file(
    app: tauri::AppHandle,
    input_path: String,
    config: WatermarkConfig,
) -> Result<(), String> {
    let output_path = app.dialog().file().blocking_save_file();

    let mut output_path = match output_path {
        Some(path) => path.to_string(),
        None => return Err("No output path selected".to_string()),
    };

    let ext = input_path.rsplit('.').next().unwrap_or("").to_lowercase();

    if ext == "pdf" {
        if !output_path.ends_with(".pdf") {
            output_path.push_str(".pdf");
        }
        add_pdf_watermark(&input_path, &output_path, &config.visible)?;
    } else {
        let target_ext = match config.output.format {
            OutputFormat::Jpeg => "jpg",
            OutputFormat::Png => "png",
            OutputFormat::Heic => "heic",
            OutputFormat::SameAsInput => {
                if ext == "heic" || ext == "heif" {
                    "heic"
                } else if ext == "png" {
                    "png"
                } else {
                    "jpg"
                }
            }
        };

        if !output_path.ends_with(&format!(".{}", target_ext)) {
            output_path.push_str(&format!(".{}", target_ext));
        }

        process_image(&input_path, &output_path, &config)?;
    }

    Ok(())
}
