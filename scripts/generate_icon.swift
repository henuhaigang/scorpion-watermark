#!/usr/bin/env swift
//
// Scorpion Watermark 应用图标生成器
//
// 品牌一致性：沿用 Scorpion-Clipboard 的视觉语言 ——
//   · 蓝紫渐变圆角方形底（0.15,0.35,0.75 → 0.45,0.20,0.65，-45°）
//   · 22% 圆角 + 内侧高光描边
//   · 同一套蝎子造型与配色（品牌资产，不要随意改动）
//
// 产品差异：剪贴板换成图片卡片，卡片上叠加斜向水印纹理。
//
// 用法：
//   swift scripts/generate_icon.swift
//   # 产物写入 src-tauri/icons/
//

import AppKit
import Foundation

// MARK: - 品牌色（与 Scorpion-Clipboard 保持一致）

private let brandBlue = NSColor(red: 0.15, green: 0.35, blue: 0.75, alpha: 1)
private let brandPurple = NSColor(red: 0.45, green: 0.20, blue: 0.65, alpha: 1)
private let scorpionBlue = NSColor(red: 0.15, green: 0.35, blue: 0.75, alpha: 0.85)
private let stingerRed = NSColor(red: 0.85, green: 0.20, blue: 0.20, alpha: 1)
/// 水印纹理：比蝎子蓝更浅、更透明，只作为底纹不抢主体
private let watermarkTint = NSColor(red: 0.15, green: 0.35, blue: 0.75, alpha: 0.10)

private let canvas: CGFloat = 1024

// MARK: - 蝎子（品牌资产，造型与 Scorpion-Clipboard 一致）

/// 在指定矩形内绘制蝎子。`unit` 为 1024 基准下的缩放系数。
private func drawScorpion(in rect: NSRect, unit: CGFloat) {
    let cx = rect.midX
    let midY = rect.midY
    let s = unit

    // 身体
    let bodyW = 220 * s
    let bodyH = 120 * s
    let bodyX = cx - bodyW / 2
    let bodyY = midY - bodyH / 2 - 20 * s
    scorpionBlue.setFill()
    NSBezierPath(ovalIn: NSRect(x: bodyX, y: bodyY, width: bodyW, height: bodyH)).fill()

    // 尾巴
    let tail = NSBezierPath()
    tail.lineWidth = 28 * s
    tail.lineCapStyle = .round
    tail.lineJoinStyle = .round
    var tailX = cx
    let tailY = bodyY - 20 * s
    tail.move(to: NSPoint(x: tailX, y: tailY))
    tail.curve(to: NSPoint(x: tailX - 120 * s, y: tailY - 100 * s),
               controlPoint1: NSPoint(x: tailX - 30 * s, y: tailY - 40 * s),
               controlPoint2: NSPoint(x: tailX - 60 * s, y: tailY - 100 * s))
    tail.curve(to: NSPoint(x: tailX - 80 * s, y: tailY - 160 * s),
               controlPoint1: NSPoint(x: tailX - 160 * s, y: tailY - 100 * s),
               controlPoint2: NSPoint(x: tailX - 130 * s, y: tailY - 160 * s))
    tail.curve(to: NSPoint(x: tailX - 100 * s, y: tailY - 180 * s),
               controlPoint1: NSPoint(x: tailX - 70 * s, y: tailY - 150 * s),
               controlPoint2: NSPoint(x: tailX - 100 * s, y: tailY - 170 * s))
    scorpionBlue.setStroke()
    tail.stroke()

    stingerRed.setFill()
    NSBezierPath(ovalIn: NSRect(x: tailX - 105 * s, y: tailY - 190 * s,
                                width: 22 * s, height: 22 * s)).fill()

    // 双螯：从身体向外上方弯曲，末端分出两个钳齿
    for isLeft in [true, false] {
        let dir: CGFloat = isLeft ? -1 : 1
        let anchorX = isLeft ? bodyX + 10 * s : bodyX + bodyW - 10 * s
        let baseY = bodyY + bodyH / 2 + 10 * s
        let tipX = anchorX + dir * 80 * s
        let tipY = baseY + 70 * s

        let pincer = NSBezierPath()
        pincer.lineWidth = 22 * s
        pincer.lineCapStyle = .round
        pincer.move(to: NSPoint(x: anchorX, y: baseY))
        pincer.curve(to: NSPoint(x: tipX, y: tipY),
                     controlPoint1: NSPoint(x: anchorX + dir * 30 * s, y: baseY + 20 * s),
                     controlPoint2: NSPoint(x: anchorX + dir * 50 * s, y: baseY + 40 * s))
        scorpionBlue.setStroke()
        pincer.stroke()

        // 上钳齿
        let clawTop = NSBezierPath()
        clawTop.lineWidth = 16 * s
        clawTop.lineCapStyle = .round
        clawTop.move(to: NSPoint(x: tipX, y: tipY))
        clawTop.line(to: NSPoint(x: tipX + dir * 30 * s, y: tipY + 20 * s))
        scorpionBlue.setStroke()
        clawTop.stroke()

        // 下钳齿
        let clawBottom = NSBezierPath()
        clawBottom.lineWidth = 16 * s
        clawBottom.lineCapStyle = .round
        clawBottom.move(to: NSPoint(x: tipX, y: tipY))
        clawBottom.line(to: NSPoint(x: tipX + dir * 20 * s, y: tipY + 30 * s))
        scorpionBlue.setStroke()
        clawBottom.stroke()
    }

    // 八条腿
    let legSpecs: [(CGFloat, CGFloat)] = [(0.25, 0.4), (0.4, 0.3), (0.6, 0.3), (0.75, 0.4)]
    for (t, offset) in legSpecs {
        let legY = bodyY + bodyH * t
        for isLeft in [true, false] {
            let startX = isLeft ? bodyX + 5 * s : bodyX + bodyW - 5 * s
            let endX = isLeft ? bodyX - 40 * s : bodyX + bodyW + 40 * s
            let leg = NSBezierPath()
            leg.lineWidth = 12 * s
            leg.lineCapStyle = .round
            leg.move(to: NSPoint(x: startX, y: legY))
            leg.line(to: NSPoint(x: endX, y: legY - 30 * s * offset))
            scorpionBlue.setStroke()
            leg.stroke()
        }
    }

    // 眼睛
    let eyeR = 10 * s
    let eyeY = bodyY + bodyH * 0.35
    let eyeSpacing = 40 * s
    NSColor.white.setFill()
    for sign in [-1.0, 1.0] {
        NSBezierPath(ovalIn: NSRect(x: cx + CGFloat(sign) * eyeSpacing - eyeR,
                                    y: eyeY - eyeR,
                                    width: eyeR * 2, height: eyeR * 2)).fill()
    }
}

// MARK: - 画完整图标

/// 按指定像素尺寸绘制图标，返回 1:1 的位图（不做任何缩放）。
///
/// 注意：不能使用 `NSImage(size:)` + `lockFocus()`。在 Retina 屏上 `lockFocus()`
/// 会建立 2 倍后备缓冲，请求 32px 实际产出 64px，导致所有图标尺寸翻倍。
/// 这里直接创建与像素等大的位图上下文，并在此上下文中绘制。
private func drawIcon(size: CGFloat) -> NSBitmapImageRep {
    guard let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil,
        pixelsWide: Int(size), pixelsHigh: Int(size),
        bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true,
        isPlanar: false, colorSpaceName: .deviceRGB,
        bytesPerRow: 0, bitsPerPixel: 0
    ) else {
        fatalError("无法创建 \(size)x\(size) 位图")
    }

    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)

    let rect = NSRect(origin: .zero, size: NSSize(width: size, height: size))
    let unit = size / canvas
    let cornerRadius = size * 0.22

    // 圆角裁切
    let bg = NSBezierPath(roundedRect: rect, xRadius: cornerRadius, yRadius: cornerRadius)
    bg.addClip()

    // 品牌渐变底
    let gradient = NSGradient(starting: brandBlue, ending: brandPurple)
    gradient?.draw(in: rect, angle: -45)

    // 图片卡片（替代剪贴板）
    let cardW = 520 * unit
    let cardH = 620 * unit
    let cardRect = NSRect(x: rect.midX - cardW / 2,
                          y: rect.midY - cardH / 2,
                          width: cardW, height: cardH)
    let cardRadius = 54 * unit
    let cardPath = NSBezierPath(roundedRect: cardRect, xRadius: cardRadius, yRadius: cardRadius)
    NSColor(white: 1, alpha: 0.94).setFill()
    cardPath.fill()

    // 卡片内的斜向水印纹理 —— 裁切在卡片内，只作底纹
    cardPath.addClip()
    watermarkTint.setStroke()
    let stripe = NSBezierPath()
    stripe.lineWidth = 26 * unit
    stripe.lineCapStyle = .butt
    let step = 112 * unit
    var x = cardRect.minX - cardH
    while x < cardRect.maxX + cardH {
        stripe.move(to: NSPoint(x: x, y: cardRect.minY))
        stripe.line(to: NSPoint(x: x + cardH, y: cardRect.maxY))
        x += step
    }
    stripe.stroke()
    NSGraphicsContext.current?.cgContext.resetClip()

    // 蝎子主体
    drawScorpion(in: cardRect, unit: unit)

    // 内侧高光描边
    let inner = NSBezierPath(roundedRect: rect.insetBy(dx: 2, dy: 2),
                             xRadius: cornerRadius - 2, yRadius: cornerRadius - 2)
    NSColor(white: 1, alpha: 0.08).setStroke()
    inner.lineWidth = 1
    inner.stroke()

        NSGraphicsContext.restoreGraphicsState()
    return rep
}

// MARK: - 输出

private let repoRoot = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()   // scripts/
    .deletingLastPathComponent()   // 仓库根
private let iconsDir = repoRoot.appendingPathComponent("src-tauri/icons")
let iconsetDir = iconsDir.appendingPathComponent("AppIcon.iconset")

try? FileManager.default.createDirectory(at: iconsetDir, withIntermediateDirectories: true)

/// iconset 规格：文件名必须严格匹配，iconutil 才能正确打包
let iconsetSpecs: [(name: String, pixels: CGFloat)] = [
    ("icon_16x16.png", 16), ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32), ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128), ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256), ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512), ("icon_512x512@2x.png", 1024),
]

for spec in iconsetSpecs {
    let rep = drawIcon(size: spec.pixels)
    guard let data = rep.representation(using: .png, properties: [:]) else {
        print("✗ 编码失败 \(spec.name)")
        continue
    }
    try data.write(to: iconsetDir.appendingPathComponent(spec.name))
}

// Tauri 需要的固定文件名
let tauriSizes: [(name: String, pixels: CGFloat)] = [
    ("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256), ("icon.png", 1024),
]
for spec in tauriSizes {
    let rep = drawIcon(size: spec.pixels)
    if let data = rep.representation(using: .png, properties: [:]) {
        try data.write(to: iconsDir.appendingPathComponent(spec.name))
    }
}

print("✓ PNG 已写入 \(iconsDir.path)")
