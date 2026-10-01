---
name: rebuild-watermark-font
description: 重建或替换水印字体子集。当需要更换字体（尤其是换成 Noto Sans SC 等开源字体以解决授权问题）、扩充或缩减字符集、或修复前后端字体不一致时使用。
---

# 重建水印字体

## 为什么必须用脚本

前后端必须使用**字节完全相同**的字体文件，否则预览与导出的字形会不一致。

| 位置 | 路径 | 用途 |
|---|---|---|
| 后端 | `src-tauri/assets/fonts/WatermarkSC-Regular.ttf` | `core/font.rs` 的 `include_bytes!` |
| 前端 | `public/fonts/WatermarkSC-Regular.ttf` | `index.css` 的 `@font-face` |

`npm run tauri build` 的链路是 Vite 把 `public/` 复制进 `dist/`，Tauri 再打包 `dist/`。
所以前端字体**必须放 `public/fonts/`**，放 `src/assets/` 不会被 Tauri 打进 bundle。

## 换字体不需要改代码

`src/index.css` 里的族名 `Scorpion Watermark SC` 是**自定义别名**，通过 `@font-face`
绑定到文件，与字体内部名称无关。换字体只需替换文件本身。

不要试图把族名改成字体内部名称（如 `Noto Sans SC`）—— 那会让浏览器优先使用系统已装的
同名字体，一致性就失效了。

## 操作步骤

```bash
# 1. 从任意源字体生成子集，同时写入前后端两处
python3 scripts/build_font.py /path/to/NotoSansSC-Regular.otf

# 2. TTC 字体集合必须指定字面序号
python3 scripts/build_font.py src-tauri/assets/fonts/NotoSansCJKsc-Regular.otf --index 1
#    原始文件：--index 0 = Heiti TC（繁体），--index 1 = Heiti SC（简体）

# 3. 需要更完整字集时扩充码位区间（可重复）
python3 scripts/build_font.py NotoSansSC-Regular.otf --include-range 0x3400-0x4DBF
```

脚本会自动完成：裁剪字集 → 重命名为固定别名 → 写入两处 → 校验字节一致 → 校验常用文案字形齐全。

## 验证

```bash
cd src-tauri && cargo test --lib font::
```

必须通过的三个测试：

- `test_backend_and_frontend_font_identical` —— 两份文件字节相同
- `test_common_watermark_text_has_no_missing_glyphs` —— 常用文案字形齐全
- `test_reports_missing_glyphs` —— 缺失字形能被检测到

若新增了常用文案测试样例，记得同时更新 `scripts/build_font.py` 里的 `SMOKE_TEST_TEXT`。

## 两个必须知道的坑

### fontTools 每次 save 都会刷新时间戳

分别 `save()` 两次会产生**不同字节**，前后端字体就不一致了。
`build_font.py` 的做法是只序列化一次（`font.recalcTimestamp = False` + `BytesIO`），
再把同一份字节写到两个路径。**不要改成分别保存。**

### 授权

- 当前字体源自 **Apple Heiti SC，专有授权**，仅适合自用，分发即可能侵权
- 分发前必须换成开源字体：**Noto Sans SC / 思源黑体（OFL 协议）**
- 项目规则禁止网络请求，所以**不要自动下载字体**，让用户提供文件

## 字集说明

默认字集 = GB2312 全部汉字 + ASCII + CJK 标点（U+3000–303F）+ 全角（U+FF00–FFEF）
+ 常用补充字，约 7700 字形 / 6.2MB。

GB2312 之外的生僻字会缺失。缺字形时后端画豆腐块、前端逐字回退系统字体，
两边表现不一致，所以 `core/font.rs` 的 `missing_glyphs()` 会把它们报出来，
由预览区红色提示。扩充字集时优先考虑 `--include-range` 补 CJK 扩展A/B。