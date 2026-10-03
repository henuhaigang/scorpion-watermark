# Scorpion Watermark — OpenCode 项目规则

## 项目概述
macOS 桌面水印工具，项目代号 Scorpion Watermark。
Tauri v2 + Rust 后端 + React/TypeScript 前端。
详细需求见 SPEC.md，使用说明见 README.md。

## 授权
PolyForm Noncommercial License 1.0.0（见 LICENSE）：可自由使用/复制/修改/分发，禁止商用。
注意这是 **source-available（源码可见）而非开源** —— OSI 定义要求开源许可必然允许商用。
字体为 Noto Sans SC 子集（SIL OFL 1.1），**允许自由分发**，不构成分发障碍。

## 文档导航
- `README.md` —— 功能现状、使用说明、排版规则、已知限制
- `SPEC.md` —— 需求与技术方案
- `LICENSE` —— 授权条款
- `skills/` —— AI 助手工作流（详见下方 skills 章节）

## 技术栈
- 桌面框架：Tauri v2（Universal Binary，macOS 11.0+）
- 前端：React 18 + TypeScript + Vite + Zustand + react-konva
- 后端：Rust 2021 edition
- 图像处理：image, imageproc, ab_glyph
- PDF：lopdf
- 隐形水印：blind_watermark（DWT-DCT-SVD）—— **当前不可用**，依赖已移除
- 元数据清理：kamadak-exif, img-parts

## 代码规范
- Rust：遵循 clippy 建议，错误用 Result<T, String> 返回给前端
- TypeScript：strict 模式，不使用 any
- 前后端数据类型必须对齐（Rust serde camelCase ↔ TS interface）
- 所有 Tauri command 放在 src-tauri/src/commands/ 下，按功能分文件
- 核心处理逻辑放在 src-tauri/src/core/ 下，不依赖 Tauri 运行时

## 关键约束
- 必须本地离线处理，禁止任何网络请求
- 输出不覆盖原文件
- 默认剥离所有元数据
- 预览用前端 Canvas，导出用 Rust 全分辨率处理

## 水印字体
- 前后端必须使用**同一个字体文件**，否则预览与导出字形不一致
  - 后端：`src-tauri/assets/fonts/WatermarkSC-Regular.ttf`（`core/font.rs` 里 `include_bytes!`）
  - 前端：`public/fonts/WatermarkSC-Regular.ttf`（`index.css` 里 `@font-face`）
  - 有测试 `test_backend_and_frontend_font_identical` 校验两份文件字节相同
- CSS 里的族名 `Scorpion Watermark SC` 是自定义别名，与字体内部名称无关，换字体无需改代码
- 重新生成子集：`python3 scripts/build_font.py <源字体> [--index N]`
  - TTC 字体集合必须用 `--index` 指定字面
  - 字集 = GB2312 + ASCII + CJK 标点 + 全角，可用 `--include-range` 扩充
- 当前字体 **Noto Sans SC（SIL OFL 1.1，可自由分发）**，已随仓库提交
- 换任何开源字体：下载后执行 `python3 scripts/build_font.py <源字体>` 即可，代码零改动
- 项目禁止网络请求，**不要自动下载字体**，让用户提供文件
- 字体是裁剪过的子集，生僻字可能缺失；`calculate_layout` 会返回 `missing_glyphs` 供前端提示

## 排版规则
- 文字变多时**自动缩小字号**尽量保持单行；压成单行会低于可读阈值时才换行
- `MAX_LINE_WIDTH_RATIO` 0.35：单行文字最长占画布宽度
- `COMFORTABLE_FONT_RATIO` 0.02：可读阈值，低于此值宁可换行也不硬塞单行
- `MIN_FONT_RATIO` 0.015：字号绝对下限
- `MAX_LINES` 3：最多行数
- 行间距范围 **0~1.5**（默认 0.5），`行高 = 字号 × (LINE_HEIGHT_BASE + 行间距)`
- 水印间距 `block_gap_ratio` 范围 **0~3**（默认 1.0），控制**块与块**的距离，
  与行间距相互独立。**语义是「留多少空白」，0 = 紧贴，不设下限** —— 早期用
  `max(字号×滑块值, 行高)` 兜底，导致滑块拉到 0 仍残留一个行高，已移除
  - 语义是「额外行距」而非「行高倍数」：倍数写法下行间距 0 会让行高归零导致重叠
### 字号与行高的三套基准（最容易出错的地方）
同一个「字号」在三处语义不同，混用就会出现偏移，且**字号越大越严重**：

| 位置 | 语义 | 正确写法 |
|---|---|---|
| Rust `ab_glyph::PxScale` | **像素**行框高度 | 经 `font.rs` 的 `scale_for()` 转换 |
| Rust 内部 `line_height` | **像素** | `font_size * (LINE_HEIGHT_BASE + line_spacing)` |
| **Konva `lineHeight`** | **倍数**，内部再乘 fontSize | 只传 `LINE_HEIGHT_BASE + lineSpacing` |

- `scale_for()`：`PxScale` 是行框高度，界面字号是 em 字号，两者对多数中文字体
  并不相等（Noto Sans SC 行框是 1448/1000），直接 `PxScale::from()` 会偏小 30%
- Konva 见 `konva/lib/shapes/Text.js`：`lineHeightPx = lineHeight * fontSize`。
  **传像素值会被再乘一次 fontSize**，实际行高放大 fontSize 倍，表现为
  「画布一角有空白，只有缩小字体才正常」
- 预览与导出共用后端算出的 `items` 坐标，但渲染基准不同：Rust 侧按像素行高，
  Konva 按倍数行高。改行距公式时**两边都要改**，否则预览与导出不一致
- 竖直间距用**墨迹高度** `字号 + (行数-1)×行高`，不是行框高度 `行高×行数`。
  单行块若按行框算，gap=0 也会多出半个行高的空白
- 文字任何情况下都不截断，宁可换行或缩小字号
- 平铺网格在**旋转坐标系**下生成，列行数须按画布四角逆旋转后的包络计算，
  **不可用对角线长度 + 圆形半径裁剪** —— 矩形四角到中心距离恰等于半对角线，
  会被裁剪边界剔掉，导致画布角上留白（字号越大越明显）
- Rust 整数除法向零截断，`-n / 2` 会差一列，须显式算下界；负数取模用
  `rem_euclid` 保证错开方向一致
- `WIDTH_TOLERANCE` 1px：`fits_in_lines` 与 `split_by_width` 必须用同一容差，
  否则会出现「判定单行放得下、渲染时又被切开」的矛盾

## 开发命令
- 开发：npm run tauri dev
- 构建 Universal Binary：npm run tauri build -- --target universal-apple-darwin
- Rust 检查：cd src-tauri && cargo clippy
- 全量测试：cd src-tauri && cargo test --lib
- 重建字体子集：python3 scripts/build_font.py <源字体> --index <N>
- 重新生成图标：swift scripts/generate_icon.swift（加 --preview 只出预览图）
- 打包 DMG：./scripts/build_dmg.sh（产物在 dist-packages/）
  **禁止手工 `cargo build` + `lipo` + `tauri bundle`** —— 实测产物白页，
  所有校验都通过但内容区不渲染。详见 README「打包 DMG」

## skills（AI 助手工作流）
按任务类型选用，改动前先读对应 skill：
- `skills/debug-missing-watermark/SKILL.md` —— 水印不显示/被截断/重影/腰斩的排查流程
- `skills/add-watermark-parameter/SKILL.md` —— 新增或修改水印参数、排版常量
- `skills/rebuild-watermark-font/SKILL.md` —— 更换字体、扩充字集、修复前后端不一致

## 禁止事项
- 不要修改 src-tauri/src/main.rs（Tauri 移动端入口要求）
- 不要在 lib.rs 中写业务逻辑，只做插件注册和 command 注册
- 不要引入不必要的依赖，新增依赖前先说明理由
- 不要删除或替换字体文件除非用户明确要求
- 不要用 PDF 导出功能（`core/pdf_watermark.rs` 会覆盖原页面内容流）

## 构建产物
- `dist-packages/Scorpion-Watermark-<版本>-<架构>.dmg` —— 分发用 DMG（不入库）
- `docs/releases/<版本>.md` —— 发布说明，已是可粘贴格式，发版时直接把内容
  复制到 GitHub Release 页面即可，无需加工。注意其中的 SHA256 与体积对应
  **发布当时**的产物，重新构建后会变化，以 Release 页面上的为准
- ad-hoc 签名（无 Apple 证书），用户需右键打开绕过 Gatekeeper
- 详见 README「打包 DMG」，含三个已知构建坑（rustup 镜像 404、PATH 里的
  第三方 xattr、bash 变量后接中文字符）

## 已知缺陷（勿依赖，勿声称可用）
- `core/pdf_watermark.rs` 覆盖 `Contents` 会**破坏原 PDF 内容**，且字体/透明度/铺排均未实现
- 隐形水印与鉴定功能**已整体停用**：底层库 `blind_watermark` 0.1.3 提取功能失效
  （实测：嵌入生效但提取恒返回全 0，改变种子/强度/尺寸/载荷长度均无效，
  0.1.3 已是最新版）。UI 入口已移除，接口返回明确错误。
  载荷格式与感知哈希已就绪（`core/wm_payload.rs`），恢复时可直接复用
- 描边 `stroke_color`/`stroke_width` 与阴影 `shadow` 有配置字段但**渲染未实现**
- 单点模式仅接通「居中」，其余八宫格位置与 `custom_xy` 未实现
- RAW 格式未支持（文件选择器列了但加载会失败）
