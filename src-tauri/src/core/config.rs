use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatermarkConfig {
    pub visible: VisibleWatermark,
    pub invisible: Option<InvisibleWatermark>,
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisibleWatermark {
    pub enabled: bool,
    pub text: String,
    pub mode: TileMode,
    pub angle: f32,
    pub opacity: f32,
    pub font_size: f32,
    pub font_size_ratio: f32,
    pub line_spacing: f32,
    pub color: String,
    pub stroke_color: Option<String>,
    pub stroke_width: f32,
    pub shadow: Option<ShadowConfig>,
    pub position: Option<GridPosition>,
    pub custom_xy: Option<[f32; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvisibleWatermark {
    pub payload: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputConfig {
    pub format: OutputFormat,
    pub strip_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShadowConfig {
    pub color: String,
    pub blur: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TileMode {
    Tile,
    Single,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OutputFormat {
    Jpeg,
    Png,
    Heic,
    SameAsInput,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GridPosition {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}
