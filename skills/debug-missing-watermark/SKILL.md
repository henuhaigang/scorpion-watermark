---
name: debug-missing-watermark
description: 排查水印不显示或显示异常。当出现「预览没有水印」「导出没有水印」「文字被截断」「文字重影/重叠」「字被腰斩」等问题时使用。
---

# 排查水印显示问题

**先定位是哪一侧的问题，再动手改。** 预览走前端 Konva + 后端布局，导出走后端 Rust 渲染，
两者共用 `calculate_layout`，所以症状组合能直接指向根因。

## 症状对照表

| 症状 | 最可能的原因 |
|---|---|
| 预览、导出**都**没有水印 | Rust 缩放因子算错，缓冲区巨大被贴到画面外 |
| **只有**预览异常，导出正常 | 前端 Konva：字体未加载、`lineHeight` 缺失、参数名写错 |
| **只有**导出异常，预览正常 | Rust 渲染：缓冲区太小（腰斩）、alpha 合成错误 |
| 看起来**像被截断** | `BLOCK_GAP_RATIO` 小于块内行距，块间距比行距还密 |
| 提示「缺少字形」 | 字体子集不含该字，用 `--include-range` 扩充 |
| 导出会崩溃（panic） | 依赖库内部 `assert!`，需要预校验或 `catch_unwind` |

## 第 1 步：写像素断言，不要靠肉眼

`visible_watermark.rs` 已有现成模式：

```rust
let before: Vec<u8> = img.to_rgba8().as_raw().clone();
apply_visible_watermark(&mut img, &config).unwrap();
let after_img = img.to_rgba8();
let changed = before.chunks_exact(4)
    .zip(after_img.as_raw().chunks_exact(4))
    .filter(|(a, b)| a != b).count();
assert!(changed > 100, "只有 {} 个像素变化", changed);
```

`changed == 0` 而函数返回 `Ok` → 几乎必然是缩放因子问题（见下）。

## 第 2 步：验证字体与字形

```bash
cargo test --lib font:: -- --nocapture
```

`missing_glyphs()` 能直接告诉你哪些字不在子集里。

## 第 3 步：肉眼验证排版

```bash
cargo run --example test_watermark   # 输出到 /tmp/test_watermark.png
```

**像素断言看不出「像被截断」这类视觉问题**，必须看图。
需要试不同长度时改 `examples/test_watermark.rs` 里的 `text` 和 `font_size_ratio`。

## 根因速查

### 完全没有水印（0 像素变化）

ab_glyph 的缩放因子是 `scale / height_unscaled`，不是 `scale`：

```rust
// 错：缓冲区被放大 height_unscaled 倍（本字体是 1000 倍）
let advance = font.h_advance_unscaled(gid) * scale.x;   // 36 → 36000

// 对：用 ScaleFont
let advance = font.as_scaled(scale).h_advance(gid);     // 36
```

同理 `ascent_unscaled * scale.y` 也是错的，要用 `scaled.ascent()`。
`core/font.rs` 已封装好 `measure_text_width` / `char_width`，别直接调未缩放的接口。

### 腰斩（字被切掉一半）

缓冲区高度不足。字形墨迹范围约 `1.09F ~ 2.03F`，而缓冲区只有 `1.4F`。

```rust
// 用 imageproc 测真实墨迹，并留一行余量
let (w, h) = text_size(scale, font, line);
let buf_height = (block_height + padding * 2.0 + h as f32).ceil() as u32;
```

### 像被截断（多行块读起来像断了）

```
块内行距 = 字号 × 1.3 × lineSpacing     ← 默认 2.0 时是 2.6 倍
块间间隙 = 字号 × BLOCK_GAP_RATIO       ← 曾是 1.0 倍
```

块间距比行距还小 → 分不清哪几行属于同一个块。
把 `BLOCK_GAP_RATIO` 提到 2.0，并把 `lineSpacing` 默认降到 1.2、滑块范围收到 1~2。

### 预览重影 / 重叠

Konva 的 `Text` 依赖 canvas `measureText`，字体没加载完会用 fallback 字体排版。
必须等字体就绪：

```ts
await document.fonts.load(`400 32px 'Scorpion Watermark SC'`, config.visible.text)
await document.fonts.ready
```

并用 `fontReady` 门控住计算布局的 `useEffect`。

另外检查旧版那个吞更新的节流逻辑 —— `if (now - last < 33) return` 提前 return
且不补调用，会**丢掉最后一次调参**。字体已缓存后不需要节流，直接删掉。

### 换行决策错乱

`choose_font_and_lines` 的顺序必须是「先看单行放不放得下，再看字号够不够大」。
写反了会出现「机密文件」被拆成 3 行这种荒谬结果 —— 现有测试
`test_layout_prefers_single_line_when_readable` 就是守这个的。

## 反馈回路

改完必须跑全量校验：

```bash
cd src-tauri && cargo test --lib && cargo clippy --all-targets
npx tsc -b --force
```

前端改动还要重启 dev server 并**实际点一遍**：dev server 是后台进程，
关掉窗口或终端会话结束就会退出，需要重跑 `npm run tauri dev`。