use crate::core::config::VisibleWatermark;
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};

pub fn add_pdf_watermark(
    input_path: &str,
    output_path: &str,
    config: &VisibleWatermark,
) -> Result<(), String> {
    if !config.enabled || config.text.is_empty() {
        std::fs::copy(input_path, output_path).map_err(|e| e.to_string())?;
        return Ok(());
    }

    let mut doc = Document::load(input_path).map_err(|e| format!("Failed to load PDF: {}", e))?;

    let pages = doc.get_pages();
    let page_ids: Vec<u32> = pages.keys().copied().collect();

    for page_id in page_ids {
        let content = generate_watermark_content(config)?;
        let new_stream = Stream::new(dictionary! {}, content.encode().unwrap());
        let new_stream_id = doc.add_object(new_stream);

        let page_dict = doc.get_object_mut((page_id, 0)).map_err(|e| e.to_string())?
            .as_dict_mut().map_err(|e| e.to_string())?;

        page_dict.set("Contents", Object::Reference(new_stream_id));
    }

    doc.save(output_path).map_err(|e| format!("Failed to save PDF: {}", e))?;

    Ok(())
}

fn generate_watermark_content(config: &VisibleWatermark) -> Result<Content, String> {
    let mut operations = Vec::new();

    operations.push(Operation::new("q", vec![]));
    operations.push(Operation::new("BT", vec![]));

    let font_size = config.font_size;

    operations.push(Operation::new(
        "Tf",
        vec!["F1".into(), font_size.into()],
    ));

    let color = parse_color(&config.color)?;
    operations.push(Operation::new(
        "rg",
        vec![
            (color[0] as f32 / 255.0).into(),
            (color[1] as f32 / 255.0).into(),
            (color[2] as f32 / 255.0).into(),
        ],
    ));

    let text = &config.text;
    let text_bytes = text.as_bytes();
    let hex_string: String = text_bytes.iter().map(|b| format!("{:02X}", b)).collect();

    let x = 100.0;
    let y = 100.0;
    operations.push(Operation::new(
        "Tm",
        vec![1.into(), 0.into(), 0.into(), 1.into(), x.into(), y.into()],
    ));

    operations.push(Operation::new(
        "Tj",
        vec![Object::string_literal(hex_string)],
    ));

    operations.push(Operation::new("ET", vec![]));
    operations.push(Operation::new("Q", vec![]));

    Ok(Content { operations })
}

fn parse_color(hex: &str) -> Result<[u8; 3], String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(format!("Invalid color: {}", hex));
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;

    Ok([r, g, b])
}
