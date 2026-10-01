#!/usr/bin/env python3
"""从任意源字体生成水印字体子集，并同时安装到后端与前端。

用法:
    python3 scripts/build_font.py <源字体路径> [--index N] [--family NAME]

当前仓库使用的字体（Noto Sans SC，SIL OFL 1.1，可自由分发）:
    python3 scripts/build_font.py NotoSansSC-Regular.otf

换成别的开源字体:
    python3 scripts/build_font.py /path/to/SourceHanSansSC-Regular.otf

含多个字面的 TTC 字体集合必须用 --index 指定字面序号。

设计要点:
  * 输出字体被重命名为固定别名（默认 "Scorpion Watermark SC"）。
    CSS 里的 @font-face family 是我们自己起的别名，与字体内部名称无关，
    因此换字体不需要改任何前端/后端代码。
  * 同一个文件同时写入后端 include_bytes! 路径和 public/fonts/，
    保证预览与导出字形绝对一致（配套测试会校验两份文件字节相同）。
  * 字集 = GB2312 全部汉字 + ASCII + CJK 标点 + 全角 + 常用补充字。
    如需更完整，可追加 --include-range，例如 --include-range 0x3400-0x4DBF（扩展A）。

依赖: fonttools（构建期工具，不是应用运行时依赖）
"""

import argparse
import hashlib
import io
import os
import shutil
import sys

try:
    from fontTools import subset
    from fontTools.ttLib import TTCollection, TTFont
except ImportError:
    sys.exit("缺少 fonttools，请先安装: pip3 install fonttools")

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BACKEND_FONT = os.path.join(REPO, "src-tauri/assets/fonts/WatermarkSC-Regular.ttf")
FRONTEND_FONT = os.path.join(REPO, "public/fonts/WatermarkSC-Regular.ttf")
DEFAULT_FAMILY = "Scorpion Watermark SC"
DEFAULT_INDEX = 0

# 运行时用于校验的字集，必须与 generate_charset() 保持一致
SMOKE_TEST_TEXT = "仅供用来办理车险使用"


def generate_charset(extra_ranges):
    chars = set()
    chars.update(range(0x20, 0x7F))      # ASCII
    chars.update(range(0x3000, 0x3040))  # CJK 标点 「」、。《》
    chars.update(range(0xFF00, 0xFFF0))  # 全角字符
    for ch in "‘’“”…—–·°±×÷№€™":
        chars.add(ord(ch))
    for i in range(94 * 94):             # GB2312 全集
        try:
            chars.add(ord(bytes([0xA1 + i // 94, 0xA1 + i % 94]).decode("gb2312")))
        except Exception:
            pass
    for spec in extra_ranges:
        start, end = (int(x, 0) for x in spec.split("-", 1))
        chars.update(range(start, end + 1))
    return sorted(c for c in chars if c <= 0xFFFF)


def load_source_font(path, index):
    """ TTC 字体集合需要指定 --index。"""
    with open(path, "rb") as fh:
        magic = fh.read(4)
    if magic == b"ttcf":
        collection = TTCollection(path, lazy=False)
        if not 0 <= index < len(collection.fonts):
            sys.exit(
                f"{path} 是含 {len(collection.fonts)} 个字面的字体集合，"
                f"请用 --index 指定（0..{len(collection.fonts) - 1}）"
            )
        print(f"源字体: {os.path.basename(path)} 字面 #{index} "
              f"({collection.fonts[index]['name'].getDebugName(4)})")
        return collection.fonts[index]
    print(f"源字体: {os.path.basename(path)}")
    return TTFont(path)


def rename(font, family):
    postscript = family.replace(" ", "") + "-Regular"
    for rec in font["name"].names:
        if rec.nameID == 1:
            rec.string = family
        elif rec.nameID == 2:
            rec.string = "Regular"
        elif rec.nameID == 4:
            rec.string = family
        elif rec.nameID == 6:
            rec.string = postscript
        elif rec.nameID == 16:
            rec.string = family
        elif rec.nameID == 17:
            rec.string = "Regular"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("source", help="源字体文件路径（.ttf/.otf/.ttc）")
    ap.add_argument("--index", type=int, default=DEFAULT_INDEX,
                    help="TTC 字体集合中要取的字面序号")
    ap.add_argument("--family", default=DEFAULT_FAMILY,
                    help=f"输出字体的族名别名（默认 {DEFAULT_FAMILY}）")
    ap.add_argument("--include-range", action="append", default=[],
                    help="额外包含的码位区间，如 0x3400-0x4DBF，可重复")
    args = ap.parse_args()

    if not os.path.isfile(args.source):
        sys.exit(f"找不到源字体: {args.source}")

    font = load_source_font(args.source, args.index)
    unicodes = generate_charset(args.include_range)
    print(f"字集: {len(unicodes)} 个码位")

    options = subset.Options()
    options.layout_features = ["*"]
    options.name_IDs = ["*"]
    options.notdef_outline = True
    options.drop_tables += ["DSIG"]

    subsetter = subset.Subsetter(options=options)
    subsetter.populate(unicodes=unicodes)
    subsetter.subset(font)

    rename(font, args.family)

    # 只序列化一次，再把同一份字节写到两个路径。
    # fontTools 每次 save() 都会刷新 head.modified 时间戳，
    # 分别保存两次会产生不同字节，导致前后端字体不一致。
    font.recalcTimestamp = False
    buffer = io.BytesIO()
    font.save(buffer)
    data = buffer.getvalue()

    for path in (BACKEND_FONT, FRONTEND_FONT):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "wb") as fh:
            fh.write(data)

    digest = hashlib.sha256(data).hexdigest()
    for path in (BACKEND_FONT, FRONTEND_FONT):
        with open(path, "rb") as fh:
            if hashlib.sha256(fh.read()).hexdigest() != digest:
                sys.exit("后端与前端字体文件不一致")

    size_kb = len(data) // 1024
    print(f"已写入后端: {BACKEND_FONT}")
    print(f"已写入前端: {FRONTEND_FONT}")
    print(f"大小: {size_kb} KB  sha256: {digest[:16]}…")

    # 校验常用文案字形齐全
    cmap = TTFont(BACKEND_FONT).getBestCmap()
    missing = [c for c in SMOKE_TEST_TEXT if ord(c) not in cmap]
    if missing:
        sys.exit(f"字集中缺少字形: {''.join(missing)}")
    print("常用文案字形校验通过")


if __name__ == "__main__":
    main()
