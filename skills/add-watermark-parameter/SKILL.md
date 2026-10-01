---
name: add-watermark-parameter
description: 新增或修改水印参数。当需要增加一个可见水印属性（如描边、阴影、单点位置）、调整排版常量、或改动前后端共享的数据结构时使用。
---

# 新增 / 修改水印参数

## 参数要改的地方（一个都别漏）

| # | 文件 | 改什么 |
|---|---|---|
| 1 | `src-tauri/src/core/config.rs` | Rust 结构体字段 |
| 2 | `src/types/watermark.ts` | TS interface 字段 |
| 3 | `src/store/useWatermarkStore.ts` | `defaultConfig` 默认值 + `normalizeConfig` 钳位 |
| 4 | `src/components/VisiblePanel.tsx` | UI 控件 |
| 5 | `src-tauri/src/core/layout.rs` | 布局计算（如影响排版） |
| 6 | `src-tauri/src/core/text_renderer.rs` | 渲染（如影响绘制） |
| 7 | `src-tauri/src/core/visible_watermark.rs` | 传给渲染层的 `RenderParams` |

Rust 侧结构体已有 `#[serde(rename_all = "camelCase")]`，TS 侧字段用 camelCase 即可对齐。

## 已实现 vs 未实现

`config.rs` 里有些字段**存在但渲染未实现**，加 UI 控件前先确认真的有渲染逻辑：

- `stroke_color` / `stroke_width` —— ❌ 未实现
- `shadow` —— ❌ 未实现
- `position` / `custom_xy` —— ⚠️ 只接通了「居中」，其余未实现

## 排版常量

集中在 `src-tauri/src/core/font.rs`，改常量前先读 `README.md` 的「排版规则」：

| 常量 | 当前值 | 含义 |
|---|---|---|
| `MAX_LINE_WIDTH_RATIO` | 0.35 | 单行最长占画布宽度 |
| `COMFORTABLE_FONT_RATIO` | 0.02 | 低于此字号就改用多行 |
| `MIN_FONT_RATIO` | 0.015 | 字号绝对下限 |
| `MAX_FONT_RATIO` | 0.08 | 字号上限 |
| `MAX_LINES` | 3 | 最多行数 |
| `LINE_HEIGHT_RATIO` | 1.3 | 行高 / 字号 |
| `BLOCK_GAP_RATIO` | 2.0 | 块间隙 / 字号 |

`BLOCK_GAP_RATIO` 必须明显大于块内行距，否则相邻块的文字比同一块的行更密，
看起来像被截断——这是实际踩过的坑。

## 验证

```bash
cd src-tauri && cargo test --lib && cargo clippy --all-targets
npx tsc -b --force
```

必须成立的**硬性不变量**（`visible_watermark.rs` 的测试已覆盖）：

- 文字完整，任何情况下不截断、不加省略号
- 字号 ≥ 画布宽度 × `MIN_FONT_RATIO`
- 行数 ≤ `MAX_LINES`
- 字号 ≥ `COMFORTABLE_FONT_RATIO` 时必须走单行
- 不透明底图叠加水印后 alpha 仍为 255

改动排版后跑 `cargo run --example test_watermark`，然后**肉眼检查**
`/tmp/test_watermark.png` —— 像素断言看不出「像被截断」这类视觉问题。

## 容易踩的坑

### ab_glyph 的缩放因子

缩放因子是 `scale / height_unscaled`（`height_unscaled = ascent - descent`），
**不是** `scale / units_per_em`，更不是直接 `h_advance_unscaled * scale`。

用错会让文字缓冲区放大上千倍，旋转后被粘贴到画面外，
表现为**完全没有水印**（`apply_visible_watermark` 返回 Ok 但 0 像素变化）。

正确写法：用 `font.as_scaled(scale)` 后调 `scaled.h_advance()` / `scaled.ascent()` / `scaled.height()`，
封装在 `core/font.rs` 的 `measure_text_width` / `char_width`。

### 缓冲区高度

用 imageproc 的 `text_size(scale, font, line)` 测量真实墨迹范围，
并额外留一行余量。估算高度会导致字形被裁掉（表现为「腰斩」）。

### Tauri v2 参数名总是 camelCase

Rust 的 `canvas_width`，前端必须传 `canvasWidth`。
传 snake_case 会静默反序列化失败，错误被 `catch` 吞掉后没有任何提示。

### alpha 合成用 source-over

不要写成 `overlay_a * alpha + base_a * (1 - alpha)`（alpha 被重复相乘）。
正确做法是标准 source-over，并算出 `oa = sa + da * (1 - sa)`。

### Konva 多行需要 lineHeight

Rust 用 `字号 × LINE_HEIGHT_RATIO × lineSpacing`，Konva 必须传同一个值，
否则换行后的预览与导出对不上。

### 字体加载完成前不要排版

Konva 的 `Text` 靠 canvas `measureText`，字体没加载完会用 fallback 字体排版，
导致文字重叠错位。必须 `await document.fonts.load()` 后再计算布局
（`PreviewCanvas.tsx` 里的 `fontReady` 门控）。