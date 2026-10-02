# Scorpion Watermark 蝎子水印

macOS 桌面水印工具。为图片添加斜向平铺的可见文字水印，导出时自动剥离 EXIF/GPS 等隐私元数据。

**完全本地离线处理，不发起任何网络请求。**

> ## 水印字体
>
> 内置字体为 **Noto Sans SC** 裁剪子集，**SIL Open Font License 1.1**，
> 允许自由使用、修改与再分发，**可随本软件一同分发**。
>
> 仓库已包含该子集，克隆后无需任何额外步骤即可编译运行。
>
> 想换成别的字体见下方「水印字体」章节，**零代码改动**。

---

## 授权

本项目采用 [PolyForm Noncommercial License 1.0.0](LICENSE)（源码可见，非商用）。

- ✅ 任何人可以自由使用、复制、修改、创建衍生作品
- ✅ 任何人可以自由分发本软件的副本
- ❌ 不得用于任何商业目的

> **关于「开源」的准确说法**：按 OSI（开源促进会）的定义，允许商业使用是开源许可证的必要条件之一。因此本许可证**不属于严格意义上的开源许可证**，而是 *source-available（源码可见）*。使用 `source-available` / `非商用` 描述比 `开源` 更准确。

依赖的第三方 crate / npm 包各自保留其原有开源许可证（MIT、Apache-2.0 等）。

### ✅ 字体可自由分发

内置的 `WatermarkSC-Regular.ttf` 是从 **Noto Sans SC** 裁剪出的子集。

| 项目 | 内容 |
|---|---|
| 字体 | Noto Sans SC（思源黑体系列的 Google 版） |
| 版权 | © 2014-2021 Adobe |
| 授权 | **SIL Open Font License 1.1** —— 允许自由使用、修改、再分发与商业使用 |
| 字形数 | 约 7700（GB2312 全集 + ASCII + CJK 标点 + 全角） |
| 体积 | 约 1.9 MB |

OFL 1.1 允许字体随软件一同分发，**因此本仓库的字体不构成分发障碍**，
克隆后即可直接编译运行，无需任何额外步骤。

想换成别的开源字体（思源黑体 Source Han Sans、阿里巴巴普惠体等），
执行下面一条命令即可，**零代码改动**：

```bash
python3 scripts/build_font.py /path/to/你的开源中文字体.otf
```

## 功能

| 功能 | 状态 | 说明 |
|---|---|---|
| 可见文字水印（斜向平铺） | ✅ 可用 | 实时预览、导出全分辨率渲染 |
| 长文本自动折行 | ✅ 可用 | 单行优先，放不下时自动缩字号，仍放不下才折行 |
| 自动剥离元数据 | ✅ 可用 | 输出不含 EXIF/GPS/IPTC/XMP |
| 预设模板保存/加载/删除 | ✅ 可用 | 参数以 JSON 持久化 |
| JPG / PNG / WebP / GIF | ✅ 可用 | |
| HEIC | ⚠️ 仅 macOS | 走系统 ImageIO，其他平台会明确报错 |
| 单点模式 + 九宫格定位 | ⚠️ 部分可用 | UI 只有「居中」，其余位置与自定义坐标未接入 |
| 鉴定 / 追责功能 | ❌ 未实现 | 依赖隐形水印，一并不可用 |
| 透明 PNG 导出为 JPEG | ✅ 可用 | 自动合成到不透明底图 |
| 隐形水印（嵌入 / 提取） | ❌ 不可用 | 底层库提取功能失效，UI 已移除入口，见「已知限制」 |
| PDF 水印 | ❌ 存在严重缺陷 | 会**覆盖原页面内容**，见「已知限制」 |
| 描边 / 阴影 | ❌ 未实现 | 配置结构里有字段，但渲染未实现 |

---

## 使用说明

### 安装与运行

需要 macOS 11.0+，以及 [Rust](https://rustup.rs/) 与 [Node.js](https://nodejs.org/)。

水印字体已随仓库提供（OFL 许可），**无需额外下载**。

```bash
git clone git@github.com:henuhaigang/scorpion-watermark.git
cd scorpion-watermark
npm install
npm run tauri dev
```

### 基本流程

1. **拖入或点击选择文件**（支持 JPG / PNG / WebP / GIF / HEIC）
2. 在右侧面板填写水印文字，调整角度、透明度、字号比例、颜色
3. 画布实时显示预览效果
4. 选择输出格式，点击**导出**，指定保存位置
5. 导出会另存为新文件，**永不覆盖原图**

### 界面说明

- **预设栏**（顶部）：保存当前参数组合、一键复用。加载旧预设时超出范围的参数会被自动钳位
- **可见水印**：启用开关、文字、模式（平铺/单点）、角度、透明度、字号比例、行间距、颜色
- **输出**：格式（与原图相同 / JPEG / PNG / HEIC）、去元数据开关、导出按钮

---

## 水印字体

> 🚫 仓库内字体的**分发限制**见上方
> [「字体不可随本项目分发」](#-字体不可随本项目分发使用者必须自行替换)。要分发请先替换。

### 前后端必须使用同一个字体文件

否则预览与导出的字形会不一致。

| 位置 | 路径 | 用途 |
|---|---|---|
| 后端 | `src-tauri/assets/fonts/WatermarkSC-Regular.ttf` | `include_bytes!` 编译进二进制 |
| 前端 | `public/fonts/WatermarkSC-Regular.ttf` | `@font-face` |

有测试 `test_backend_and_frontend_font_identical` 校验两份文件**字节完全相同**。

### 为什么换字体不用改代码

`src/index.css` 里的族名 `Scorpion Watermark SC` 是**我们自己起的别名**，通过 `@font-face` 绑定到具体文件，与字体内部叫什么名字无关。所以：

```bash
# 换成任意开源字体，只需替换文件
python3 scripts/build_font.py /path/to/NotoSansSC-Regular.otf
```

脚本会：裁剪字集 → 重命名为固定别名 → **同时写入前后端两个路径** → 校验字节一致 → 校验常用文案字形齐全。

字集 = GB2312 全部汉字 + ASCII + CJK 标点 + 全角 + 常用补充字（约 7700 字形，6.2MB）。需要更完整可扩充：

```bash
python3 scripts/build_font.py NotoSansSC-Regular.otf --include-range 0x3400-0x4DBF
```

TTC 字体集合（含多个字面，如 macOS 系统字体）必须用 `--index` 指定字面序号，否则脚本会报错提示。

### 缺失字形提示

字体是裁剪过的子集，生僻字可能不在其中。`calculate_layout` 会返回 `missing_glyphs`，预览区以红色提示：

> 字体缺少这些字符，将显示为方块或回退字体：𠀀

缺字形时后端画成豆腐块、前端逐字回退系统字体，两边表现不一致，所以必须提示。

---

## 排版规则

排版逻辑集中在 `src-tauri/src/core/font.rs` 的常量与 `core/layout.rs`：

| 常量 | 值 | 含义 |
|---|---|---|
| `MAX_LINE_WIDTH_RATIO` | `0.35` | 单行文字最长占画布宽度 |
| `COMFORTABLE_FONT_RATIO` | `0.02` | 字号低于此值就改用多行，避免水印小到看不见 |
| `MIN_FONT_RATIO` | `0.015` | 字号绝对下限 |
| `MAX_FONT_RATIO` | `0.08` | 字号上限 |
| `MAX_LINES` | `3` | 最多行数 |
| `LINE_HEIGHT_RATIO` | `1.3` | 行高相对字号的倍数 |
| `BLOCK_GAP_RATIO` | `2.0` | 文字块之间的间隙相对字号的倍数 |

### 行数与字号的选择顺序

1. 设定字号下**单行放得下 → 单行**，哪怕设定字号偏小也不折行
2. 否则逐行增加，取**行数最少且字号 ≥ 2% 画布宽**的方案
3. 兜底：维持设定字号，取能放下全文的最少行数

实测行为（画布宽 1200px）：

| 字数 | 字号比例 3.0 | 字号比例 5.0 |
|---|---|---|
| 2 ~ 10 字 | 1 行 36px | 1 行 42~60px |
| 13 字 | 1 行 32px | 1 行 32px |
| 26 字 | 2 行 36px | 2 行 36px |

**硬性保证**：文字永不截断、永不加省略号；字号永不低于画布宽度的 1.5%。

### 为什么 `BLOCK_GAP_RATIO` 必须大于行距

如果文字块之间的间隙比块内行距还小，相邻块的文字会比同一块的行更密，
看起来就像文字被拦腰截断——这是本项目实际踩过的坑。

---

## 已知限制

### PDF 水印会破坏原文件内容

`core/pdf_watermark.rs` 用 `page_dict.set("Contents", ...)` **覆盖**了页面原有的内容流，
导出的 PDF 会丢失原始页面内容。此外还存在：未注册所用字体资源、没有透明度处理、
不铺排不旋转、用的是绝对字号而非比例字号、中文未设置编码。

**请勿对 PDF 使用本工具的导出功能。**

### 隐形水印与鉴定功能不可用

底层库 `blind_watermark` 0.1.3 的**提取功能失效**，该功能已整体停用，
UI 入口已移除。

实测结论（隔离测试，直接调用库自身）：

| 变量 | 取值范围 | 结果 |
|---|---|---|
| 种子模式 | `None` / `Some(seed)` | 差异 31/32 字节 |
| 嵌入强度 | (36,20) → (220,140) | 差异 31/32 字节 |
| 图像尺寸 | 256 / 512 / 1024 | 差异 31/32 字节 |
| 载荷长度 | 8 / 32 字节 | 差异 7/8、31/32 字节 |

嵌入确实会修改像素（实测 22 万像素变化、最大差值 14），但提取恒返回全 0。
所有参数维度均无法改变结果。crates.io 上 0.1.3 已是最新版本，无修复版本可升级。

接口返回明确错误而非崩溃或静默失败 —— 鉴证功能一旦给出错误结论，
危害远大于功能缺失。

**已就绪、待复用的部分**（`core/wm_payload.rs`，均通过测试）：

- 定长 32 字节载荷格式：魔数 + 版本 + 感知哈希 + 归属者哈希 + 时间戳 + 随机数 + crc32
- 感知哈希（8×8 中位数阈值）：JPEG q=95/75/50 压缩后相似度均为 1.000，
  构图级改动（如上下颠倒）会降到 0.250
- 密钥派生改为 `SHA-256(key)` 前 8 字节。原实现 `key.bytes().sum()`
  几乎没有密钥强度，`"ab"` 与 `"ba"` 会得到相同种子

恢复该功能需先解决底层问题：vendor 该库并定位修复，或自行实现一套嵌入/提取方案。
载荷格式与感知哈希可直接复用。

### 其他

- RAW 格式未支持（`FileDropZone` 的过滤器里列了 `raw`，但加载会失败）
- 描边（`strokeColor` / `strokeWidth`）与阴影（`shadow`）配置字段存在但渲染未实现
- 单点模式仅接通「居中」，其余八宫格位置与自定义坐标未实现
- 应用无自动更新、无代码签名，macOS 首次运行需在「系统设置 → 隐私与安全性」中放行

---

## 开发

### 目录结构

```
scorpion-watermark/
├── SPEC.md                    需求与技术方案
├── AGENTS.md                  项目规则（给 AI 助手看）
├── LICENSE                    PolyForm Noncommercial 1.0.0
├── scripts/
│   ├── build_font.py         字体子集生成与安装
│   ├── build_dmg.sh          打包 Universal Binary DMG
│   └── generate_icon.swift   应用图标生成
├── skills/                    AI 助手工作流
├── src-tauri/src/
│   ├── lib.rs                 仅插件与 command 注册
│   ├── commands/              Tauri command 层
│   └── core/                  核心逻辑，不依赖 Tauri 运行时
│       ├── config.rs          前后端共享的数据结构
│       ├── font.rs            字体缓存、排版常量、字形测量
│       ├── layout.rs          折行与平铺布局
│       ├── text_renderer.rs   字形光栅化、旋转、alpha 合成
│       └── pipeline.rs        处理管线编排
├── docs/releases/            各版本发布说明归档
├── dist-packages/            打包产物（DMG，不入库）
└── src/
    ├── components/            PreviewCanvas / VisiblePanel / ...
    ├── store/                 zustand 状态与参数归一化
    └── types/                 与 Rust 对齐的 TS 类型
```

### 常用命令

| 命令 | 说明 |
|---|---|
| `npm run tauri dev` | 开发模式 |
| `./scripts/build_dmg.sh` | **打包 Universal Binary DMG**（推荐，见下） |
| `./scripts/build_dmg.sh --fast` | 只打包本机架构，快约 5 倍 |
| `swift scripts/generate_icon.swift` | 重新生成应用图标 |
| `python3 scripts/build_font.py <源字体>` | 重建字体子集 |
| `cd src-tauri && cargo test --lib` | 全量测试 |
| `cd src-tauri && cargo clippy --all-targets` | Rust 静态检查 |
| `npx tsc -b --force` | 前端类型检查 |

---

## 打包 DMG

### 产物位置

```
dist-packages/Scorpion-Watermark-0.1.0-universal.dmg
```

各版本的发布说明归档在 `docs/releases/`，当前：[v0.1.0](docs/releases/v0.1.0.md)

Universal Binary，同时支持 Intel 与 Apple Silicon。原始产物在
`src-tauri/target/universal-apple-darwin/release/bundle/dmg/`。

### 一条命令

```bash
./scripts/build_dmg.sh
```

脚本会依次完成：构建前端 → 构建 arm64 → 确保 x86_64 标准库可用 →
构建 x86_64 → `lipo` 合并为 Universal Binary → `tauri bundle` 打包 DMG →
校验签名 → 复制到 `dist-packages/`。

### 为什么不直接用 `tauri build --target universal-apple-darwin`

该命令会先调 `rustup target list --installed` 做前置校验。若 x86_64 标准库是
**手动安装**的（见下一节），rustup 可能不认得，校验就会失败并中止。
脚本改为用 `cargo` 分别构建两个架构、`lipo` 合并，再交给 `tauri bundle` 打包，
绕开这个校验。

### 三个会踩的坑

脚本已经处理了以下问题，手动构建时同样会遇到：

1. **rustup 镜像源缺 x86_64 标准库**
   某些镜像（如清华）同步滞后，`rustup target add x86_64-apple-darwin` 会返回
   404。脚本会自动改为从官方源下载 `rust-std` 组件、解压后手动装入工具链，
   并补上 rustup 的组件登记（`manifest.in` 需重命名为
   `manifest-rust-std-x86_64-apple-darwin`，并向 `components` 文件**追加**条目 ——
   覆盖会丢掉 aarch64 等原有条目）。

2. **PATH 里的 `xattr` 不是系统命令**
   Tauri 打包时会执行 `xattr -cr` 清理 app bundle 的扩展属性。若 PATH 中存在
   同名的第三方 CLI（例如某些 Python 包提供的 `xattr` 命令），它不支持 `-r`
   参数，打包会直接失败。脚本开头会把 `/usr/bin` 前置到 PATH。

3. **bash 变量名后紧跟中文字符**
   `echo "$target）"` 会被 bash 解析成变量名 `target）`，在 `set -u` 下报
   `unbound variable`。中文字符紧跟变量引用时必须写成 `${target}`。

### 二进制体积

`Cargo.toml` 的 release profile 开启了 `opt-level = "s"` + thin LTO + `strip`，
二进制从 **99MB 降到 18MB**。用 thin LTO 而非完整 LTO 是因为体积收益接近，
但构建时间短得多。

### ⚠️ 代码签名与 Gatekeeper

当前构建产物是 **ad-hoc 签名**（`tauri.conf.json` 里 `signingIdentity: "-"`），
因为没有 Apple 开发者证书。

**用户下载后双击会被 Gatekeeper 拦截**，提示「无法验证开发者」，需要：

- 右键点击 app → 选择「打开」→ 确认打开，或
- 终端执行：`xattr -cr /Applications/Scorpion\ Watermark.app`

要彻底解决必须购买 **Apple Developer Program（$99/年）**，然后：

1. 用 Developer ID 证书签名（把 `signingIdentity` 改为证书名）
2. 提交 Apple 公证（notarization）：`xcrun notarytool submit ... --wait`
3.  staple 公证凭据：`xcrun stapler staple`

公证后用户双击即可安装，无需任何绕过操作。

### 前后端类型约定

Rust 用 `#[serde(rename_all = "camelCase")]`，TS 侧字段用 camelCase。
**Tauri v2 的 command 参数名总是 camelCase** —— 例如 Rust 的 `canvas_width`
在前端必须传 `canvasWidth`，传 snake_case 会静默反序列化失败。

### 容易踩的坑

1. **ab_glyph 的缩放因子是 `scale / height_unscaled`**，不是 `scale / units_per_em`，
   更不是直接 `h_advance_unscaled * scale`。用错会让文字缓冲区放大上千倍，
   旋转后被粘贴到画面外，表现为「完全没有水印」。
2. **Konva 的 `Text` 依赖 canvas `measureText`**，字体没加载完会用 fallback 字体
   排版，导致文字重叠错位。必须等 `document.fonts.load()` 完成再排版。
3. **fontTools 每次 `save()` 都会刷新时间戳**，分别保存两次会产生不同字节，
   前后端字体就不一致了。要只序列化一次、把同一份字节写到两个路径。

---

## 致谢

- 桌面框架：[Tauri](https://tauri.app)
- 图像处理：[image](https://github.com/image-rs/image)、[imageproc](https://github.com/image-rs/imageproc)、[ab_glyph](https://github.com/alexheretic/ab_glyph)
- 前端画布：[Konva](https://konvajs.org)、[react-konva](https://github.com/konvajs/react-konva)
- 状态管理：[Zustand](https://github.com/pmndrs/zustand)
- PDF：[lopdf](https://github.com/J-F-Liu/lopdf)
- 元数据：[kamadak-exif](https://github.com/kamadak-exif)、[img-parts](https://github.com//image-rs/img-parts)
- 字体工具：[fonttools](https://github.com/fonttools/fonttools)（构建期脚本依赖）