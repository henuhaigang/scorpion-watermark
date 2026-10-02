#!/usr/bin/env bash
#
# 构建 Universal Binary DMG（x86_64 + arm64）
#
# 为什么不用 `npm run tauri build -- --target universal-apple-darwin`：
# 它会先调 `rustup target list --installed` 做前置检查，若 x86_64 标准库是
# 手动安装的（见下），rustup 未必认得，校验就会失败。
# 因此这里改为：cargo 分别构建两个架构 → lipo 合并 → 交给 `tauri bundle` 打包。
#
# 用法：
#   scripts/build_dmg.sh              # 正常构建
#   scripts/build_dmg.sh --fast       # 单架构 arm64，跳过 lipo（快 5 倍，仅本机可用）
#
set -euo pipefail

# Tauri 打包时会调用 `xattr -cr` 清理 app bundle 的扩展属性。
# 若 PATH 里存在同名但不支持 -r 的第三方 CLI（例如某些 Python 包的 xattr 命令），
# 打包会直接失败。这里确保系统原生 /usr/bin/xattr 优先。
# node / npm 位于 /usr/local/bin，不受影响。
export PATH="/usr/bin:$PATH"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

BIN_NAME="scorpion-watermark"
DIST_DIR="$REPO_ROOT/dist-packages"
TARGET_DIR="$REPO_ROOT/src-tauri/target"

info() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
fail() { printf '\033[1;31m错误:\033[0m %s\n' "$*" >&2; exit 1; }

FAST=0
[[ "${1:-}" == "--fast" ]] && FAST=1

command -v cargo >/dev/null || fail "未找到 cargo，请先安装 Rust"

# ── 1. 确认 x86_64 标准库可用 ────────────────────────────────
ensure_x86_64_target() {
    if rustc --print sysroot >/dev/null 2>&1; then
        SYSROOT="$(rustc --print sysroot)"
        # 直接看标准库是否就位，比问 rustup 可靠（手动安装的情况 rustup 可能不认）
        if [[ -d "$SYSROOT/lib/rustlib/x86_64-apple-darwin/lib" ]]; then
            info "x86_64 标准库已就位"
            return
        fi
    fi

    info "安装 x86_64 标准库"
    if rustup target add x86_64-apple-darwin 2>/dev/null; then
        info "rustup 安装成功"
        return
    fi

    # rustup 失败：常见原因是镜像源缺该版本（返回 404）。
    # 绕过方式：直接从官方源下载 rust-std 组件手动装进工具链。
    local ver url tmp sysroot
    ver="$(rustc --version | awk '{print $2}')"
    url="https://static.rust-lang.org/dist/$(curl -s --max-time 30 https://static.rust-lang.org/dist/channel-rust-stable.toml | grep -m1 '^version' | cut -d'"' -f2)/rust-std-${ver}-x86_64-apple-darwin.tar.xz"
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    info "rustup 不可用，改为从官方源手动安装 rust-std ${ver}"
    curl -fsSL --max-time 600 -o "$tmp/std.tar.xz" "$url" || fail "下载失败：$url"
    tar xf "$tmp/std.tar.xz" -C "$tmp"

    sysroot="$(rustc --print sysroot)"
    local src="$tmp/rust-std-${ver}-x86_64-apple-darwin"
    [[ -d "$src/rust-std-x86_64-apple-darwin" ]] || fail "解压结果结构异常：$src"

    cp -R "$src/rust-std-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin" \
          "$sysroot/lib/rustlib/"
    # 官方 install.sh 会把 manifest.in 重命名为 manifest-<component>，
    # rustup 靠这个 + components 文件识别已安装组件
    cp "$src/rust-std-x86_64-apple-darwin/manifest.in" \
       "$sysroot/lib/rustlib/manifest-rust-std-x86_64-apple-darwin"

    # 追加而非覆盖 components —— 覆盖会丢掉 aarch64 等原有条目
    local components="$sysroot/lib/rustlib/components"
    grep -qx 'rust-std-x86_64-apple-darwin' "$components" 2>/dev/null || \
        printf 'rust-std-x86_64-apple-darwin\n' >> "$components"

    info "手动安装完成"
}

# ── 2. 构建前端 ─────────────────────────────────────────────
build_frontend() {
    info "构建前端"
    npm run build
}

# ── 3. 构建二进制 ───────────────────────────────────────────
build_arm64() {
    info "构建 arm64"
    (cd src-tauri && cargo build --release)
}

build_x86_64() {
    info "构建 x86_64"
    (cd src-tauri && cargo build --release --target x86_64-apple-darwin)
}

# ── 4. 合并 ────────────────────────────────────────────────
lipo_universal() {
    info "合并为 Universal Binary"
    mkdir -p "$TARGET_DIR/universal-apple-darwin/release"
    lipo -create \
        "$TARGET_DIR/release/$BIN_NAME" \
        "$TARGET_DIR/x86_64-apple-darwin/release/$BIN_NAME" \
        -output "$TARGET_DIR/universal-apple-darwin/release/$BIN_NAME"
    lipo -info "$TARGET_DIR/universal-apple-darwin/release/$BIN_NAME"
}

# ── 5. 打包 ────────────────────────────────────────────────
bundle_dmg() {
    local target="${1:-}"
    info "打包 DMG（target=${target}）"
    rm -rf "$TARGET_DIR/$target/release/bundle"
    npx tauri bundle --target "$target"
}

collect() {
    local src="$1"
    local out="$DIST_DIR"
    mkdir -p "$out"
    # DMG 在 bundle/dmg/ 子目录下；-maxdepth 必须放在 -name 之前，
    # 否则 BSD find 会直接失败
    local dmg ver search="$src"
    [[ -d "$src/dmg" ]] && search="$src/dmg"
    dmg="$(find "$search" -maxdepth 1 -name '*.dmg' | head -1)"
    [[ -f "$dmg" ]] || fail "未找到 DMG：$src"
    ver="$(python3 -c "import json;print(json.load(open('src-tauri/tauri.conf.json'))['version'])")"
    cp "$dmg" "$out/Scorpion-Watermark-${ver}-universal.dmg"
    info "产物：$out/Scorpion-Watermark-${ver}-universal.dmg"
}

# ── 主流程 ─────────────────────────────────────────────────
build_frontend
build_arm64

if [[ $FAST -eq 1 ]]; then
    BUNDLE_TARGET=""; BUNDLE="$TARGET_DIR/release/bundle"
    info "单架构模式，跳过 x86_64"
else
    ensure_x86_64_target
    build_x86_64
    lipo_universal
    BUNDLE_TARGET="universal-apple-darwin"
    BUNDLE="$TARGET_DIR/universal-apple-darwin/release/bundle"
fi

bundle_dmg "$BUNDLE_TARGET"

info "产物目录：$BUNDLE"
ls -lh "$BUNDLE/dmg"/*.dmg 2>/dev/null || true

# Tauri 打包时已自动 ad-hoc 签名；此处只校验，不重复签名
APP="$(find "$BUNDLE/macos" -maxdepth 1 -name '*.app' | head -1)"
if [[ -n "$APP" ]]; then
    info "校验签名"
    codesign --verify --deep --strict --verbose=1 "$APP" 2>&1 | tail -1 || true
    security find-identity -v -p codesigning >/dev/null 2>&1 || true
    if security find-identity -v -p codesigning 2>/dev/null | grep -q "Developer ID"; then
        info "检测到 Developer ID 证书，建议改用正式签名 + 公证后分发"
    fi
fi

collect "$BUNDLE"

info "完成"
cat <<'EOF'

提示：当前为 ad-hoc 签名（无 Apple Developer 证书）。
用户下载后双击会被 Gatekeeper 拦截，需右键 → 打开。
彻底解决需购买 Apple Developer Program（$99/年）做 Developer ID 签名 + 公证。
EOF