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
    /// 相邻水印块之间的间隙，相对字号的倍数。
    ///
    /// 与 `line_spacing` 是两件事：后者控制**块内**行距，前者控制**块与块**的
    /// 距离。平铺时眼睛看到的疏密主要由它决定，因此需要暴露给用户。
    #[serde(default = "default_block_gap_ratio")]
    pub block_gap_ratio: f32,
    pub color: String,
    pub stroke_color: Option<String>,
    pub stroke_width: f32,
    pub shadow: Option<ShadowConfig>,
    pub position: Option<GridPosition>,
    pub custom_xy: Option<[f32; 2]>,
}

/// 旧预设没有 `block_gap_ratio` 字段，反序列化时用这个值兜底。
/// 取 1.0 而非原先硬编码的 2.0：2.0 的块间距是行高的两倍，平铺时过于稀疏，
/// 这是用户反馈「行间距调到 0 仍然很松」的直接原因。
fn default_block_gap_ratio() -> f32 {
    1.0
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
