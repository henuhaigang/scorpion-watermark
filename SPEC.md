# Scorpion Watermark — 需求与技术方案

## 1. 项目概述

**项目名称**：Scorpion Watermark
**项目代号**：scorpion-watermark
**目标**：macOS 桌面端水印工具，支持图片和 PDF 的可见水印与隐形水印，要求本地离线处理，跨平台架构可扩展。

**平台**：macOS 11.0+（Intel + Apple Silicon Universal Binary）
**技术栈**：Tauri v2 + Rust（后端）+ React + TypeScript + Vite（前端）

## 2. 功能需求

### 2.1 一期必须功能（MVP）

| 编号 | 功能 | 验收标准 |
|---|---|---|
| F1 | 文字水印自定义 + 斜向铺满 + 实时预览 | 用户输入文字后，预览画布立即显示斜向平铺效果 |
| F2 | 隐形水印（防裁剪/防PS/溯源） | 裁剪图片后仍能提取水印；JPEG 重压缩后仍能提取 |
| F3 | 另存新文件 + 去除隐私元数据 | 输出文件不含 EXIF/GPS/IPTC/XMP |
| F4 | 支持 JPG/PNG/HEIC/RAW/GIF + PDF | 所有格式可加载、预览、导出 |
| F5 | 预设模板保存/加载/删除 | 参数配置可持久化为 JSON，一键复用 |

### 2.2 一期不做的

批量处理、视频水印、Word 水印、iOS/Windows 端、动态水印。

### 2.3 关键交互细节

- 水印模式：**铺满模式**（斜向平铺，角度可调，默认 -45°）和**单点模式**（九宫格定位 + 自定义坐标）
- 可调参数：透明度、字号、旋转角度、颜色、描边颜色/宽度、阴影
- 输出：不覆盖原文件，用户指定输出路径，格式可选 JPG/PNG/HEIC/与原格式相同

## 3. 技术架构

### 3.1 目录结构

```
scorpion-watermark/
├── SPEC.md                    # 本文件
├── AGENTS.md                  # OpenCode 项目规则
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── file.rs        # 打开/读取文件
│       │   ├── preview.rs     # 生成预览缩略图
│       │   ├── export.rs      # 导出处理后的文件
│       │   └── preset.rs      # 预设模板 CRUD
│       └── core/
│           ├── mod.rs
│           ├── config.rs      # WatermarkConfig 数据结构
│           ├── image_loader.rs
│           ├── visible_watermark.rs
│           ├── invisible_watermark.rs
│           ├── pdf_watermark.rs
│           ├── metadata_cleaner.rs
│           └── pipeline.rs    # 处理管线编排
├── src/
│   ├── App.tsx
│   ├── store/useWatermarkStore.ts
│   ├── components/
│   │   ├── FileDropZone.tsx
│   │   ├── PreviewCanvas.tsx
│   │   ├── VisiblePanel.tsx
│   │   ├── InvisiblePanel.tsx
│   │   ├── OutputPanel.tsx
│   │   └── PresetBar.tsx
│   └── types/watermark.ts
└── package.json
```

### 3.2 关键数据结构

```rust
// config.rs — 前后端共享，serde 序列化，camelCase
pub struct WatermarkConfig {
    pub visible: VisibleWatermark,
    pub invisible: Option<InvisibleWatermark>,
    pub output: OutputConfig,
}

pub struct VisibleWatermark {
    pub enabled: bool,
    pub text: String,
    pub mode: TileMode,          // Tile | Single
    pub angle: f32,              // 默认 -45
    pub opacity: f32,
    pub font_size: f32,
    pub color: String,
    pub stroke_color: Option<String>,
    pub stroke_width: f32,
    pub shadow: Option<ShadowConfig>,
    pub position: Option<GridPosition>,   // 单点模式
    pub custom_xy: Option<[f32; 2]>,
}

pub struct InvisibleWatermark {
    pub payload: String,   // 溯源信息（用户ID/订单号/时间戳）
    pub key: String,       // 密钥
}

pub struct OutputConfig {
    pub format: OutputFormat,    // Jpeg | Png | Heic | SameAsInput
    pub strip_metadata: bool,    // 默认 true
}
```

### 3.3 依赖清单

```toml
[dependencies]
tauri = { version = "2", features = ["protocol-asset"] }
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
image = { version = "0.25", features = ["jpeg", "png", "gif", "webp"] }
imageproc = "0.25"
ab_glyph = "0.2"
lopdf = "0.34"
blind_watermark = "0.2"
kamadak-exif = "0.6"
img-parts = "0.3"

[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
core-foundation = "0.10"
core-graphics = "0.24"
```

### 3.4 处理管线

```
输入文件 → image_loader 加载为 RGBA 位图
         → visible_watermark 叠加可见水印（文字斜铺/单点）
         → invisible_watermark 嵌入隐形水印（DWT-DCT-SVD）
         → metadata_cleaner 剥离 EXIF/GPS/IPTC/XMP
         → 编码写出到用户指定路径
```

PDF 路径类似，但使用 `lopdf` 加载，为每页追加内容流绘制旋转文字。

### 3.5 关键设计决策

- **预览与导出分离**：预览用前端 Canvas 渲染低分辨率缩略图，导出用 Rust 全分辨率处理
- **水印参数统一结构**：前后端共享 `WatermarkConfig`，预设模板就是该结构体的 JSON 持久化
- **隐形水印独立开关**：可见/隐形可分别启用，隐形水印仅在导出时嵌入，预览不显示
- **HEIC/RAW 走 macOS 原生 ImageIO**：通过 `objc2` 调用系统框架，后续 Windows 端再补 `libraw`

## 4. 里程碑

| 里程碑 | 内容 | 验收 |
|---|---|---|
| M0 | 项目骨架，Tauri v2 + React 跑通，Universal Binary 编译成功 | `npm run tauri dev` 能打开窗口 |
| M1 | 文件加载 + 预览（JPG/PNG/HEIC/RAW/GIF） | 拖入文件能看到预览画布 |
| M2 | 可见水印全部参数 + 实时预览 | 调参数立即反映到画布 |
| M3 | 导出 + 去元数据 + 另存新文件 | 输出文件干净且无水印残留问题 |
| M4 | PDF 支持 | PDF 逐页加水印并可导出 |
| M5 | 隐形水印嵌入 + 提取验证 | 裁剪/重压缩后仍可提取 |
| M6 | 预设模板 CRUD | 保存后重启应用仍可加载 |
| M7 | 打磨、签名、公证 | 可分发 DMG |
