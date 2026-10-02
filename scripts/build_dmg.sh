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
warn() { printf '\033[1;33m警告:\033[0m %s\n' "$*"; }
fail() { printf '\033[1;31m错误:\033[0m %s\n' "$*" >&2; exit 1; }

FAST=0
[[ "${1:-}" == "--fast" ]] && FAST=1

command -v cargo >/dev/null || fail "未找到 cargo，请先安装 Rust"

build_with_tauri() {
    # 必须走官方 `tauri build`。曾尝试用 `cargo build` + `lipo` + `tauri bundle`
    # 手工拼 Universal Binary，产物能通过签名与 CRC 校验、能启动，
    # 但窗口内容是白页 —— 资源嵌入环节绕过了 tauri 的构建流程。
    # 实测：官方流程产物 17.5MB 正常显示，手工流程产物 33MB 白页。
    local target_flag="$1"
    info "tauri build ${target_flag:-(本机架构)}"
    # shellcheck disable=SC2086
    npx tauri build ${target_flag:+$target_flag} --bundles "${BUNDLES:-dmg,app}"
}

# 优先尝试 Universal Binary；rustup 不认 x86_64 标准库时退回本机架构
try_universal() {
    info "尝试构建 Universal Binary（Intel + Apple Silicon）"
    if npx tauri build --target universal-apple-darwin --bundles "${BUNDLES:-dmg,app}" 2>/tmp/universal.err; then
        TARGET_DIR_BUNDLE="$REPO_ROOT/src-tauri/target/universal-apple-darwin/release/bundle"
        return 0
    fi
    if grep -q "is not installed" /tmp/universal.err 2>/dev/null; then
        warn "rustup 未安装 x86_64 标准库，退回单架构构建"
        warn "  修复方式：rustup target add x86_64-apple-darwin"
        warn "  若镜像源缺该版本，见 README「打包 DMG」的说明"
        return 1
    fi
    cat /tmp/universal.err >&2
    return 1
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

    local name
    name="$(basename "$dmg")"
    # 产物名统一为 Scorpion-Watermark-<版本>-<架构>.dmg
    if [[ "$name" == *universal* ]]; then
        name="Scorpion-Watermark-${ver}-universal.dmg"
    else
        name="Scorpion-Watermark-${ver}-$(uname -m).dmg"
    fi
    cp "$dmg" "$out/$name"
    info "产物：$out/$name"
}

# ── 主流程 ─────────────────────────────────────────────────
if [[ $FAST -eq 1 ]]; then
    build_with_tauri ""
    BUNDLE="$TARGET_DIR/release/bundle"
else
    if try_universal; then
        :
    else
        build_with_tauri ""
        BUNDLE="$TARGET_DIR/release/bundle"
    fi
fi

info "产物目录：$BUNDLE"
ls -lh "$BUNDLE/dmg"/*.dmg 2>/dev/null || true

# Tauri 打包时已自动 ad-hoc 签名；此处只校验，不重复签名
APP="$(find "$BUNDLE/macos" -maxdepth 1 -name '*.app' | head -1)"
if [[ -n "$APP" ]]; then
    info "校验签名"
    codesign --verify --deep --strict --verbose=1 "$APP" 2>&1 | tail -1 || true
    if security find-identity -v -p codesigning 2>/dev/null | grep -q "Developer ID"; then
        info "检测到 Developer ID 证书，建议改用正式签名 + 公证后分发"
    fi
fi

collect "$BUNDLE"

info "完成"
cat <<'TXT'

提示：当前为 ad-hoc 签名（无 Apple 证书）。
用户下载后双击会被 Gatekeeper 拦截，需右键 → 打开。
彻底解决需购买 Apple Developer Program（$99/年）做 Developer ID 签名 + 公证。
TXT
