#!/usr/bin/env swift
//
// Scorpion Watermark 应用图标生成器
//
// 品牌：沿用 Scorpion-Clipboard 的蓝紫渐变与蝎子造型。
// 设计取向：现代 macOS 应用图标 ——
//   · squircle（连续曲率方形，超椭圆 n=5）而非普通圆角矩形
//   · 单一高对比主体，去掉底纹噪点，16px 下仍可辨识
//   · 保留红色尾针作为品牌记忆点
//
// 用法：
//   swift scripts/generate_icon.swift
//   swift scripts/generate_icon.swift --preview   # 只导出 1024 预览图
//

import AppKit
import Foundation

// MARK: - 画布

private let canvas: CGFloat = 1024
private let strokeWhite = NSColor.white

private func sign(_ v: CGFloat) -> CGFloat { v < 0 ? -1 : 1 }

/// 连续曲率方形（squircle）。macOS Big Sur 起的应用图标都采用这个轮廓，
/// 与普通 roundedRect 的关键差别是转角曲率连续，用超椭圆近似。
private func squircle(size: CGFloat, exponent: CGFloat = 5.0, samples: Int = 480) -> NSBezierPath {
    let a = size / 2
    let path = NSBezierPath()
    for i in 0..<samples {
        let t = CGFloat(i) / CGFloat(samples) * 2 * .pi
        let ct = cos(t), st = sin(t)
        let x = a + a * pow(abs(ct), 2 / exponent) * sign(ct)
        let y = a + a * pow(abs(st), 2 / exponent) * sign(st)
        if i == 0 { path.move(to: NSPoint(x: x, y: y)) }
        else { path.line(to: NSPoint(x: x, y: y)) }
    }
    path.close()
    return path
}

// MARK: - 蝎子剪影


/// 绘制一只张开的钳子。
///
/// 造型是「右窄左宽的圆角楔形 + 挖空的钳口」。钳口用 even-odd 填充规则
/// 挖成孔洞，因此不需要用背景色去覆盖，渐变背景上也能正确显示。
/// 这是让「钳子」在图标尺寸下读得出来的关键 —— 实心圆头只会像棒棒糖，
/// 细笔画会糊成一团。
private func drawPincer(at tip: NSPoint, scale: CGFloat, flip: CGFloat) {
    let s = scale
    let path = NSBezierPath()

    // 外形：上缘
    path.move(to: NSPoint(x: tip.x + 26 * s * flip, y: tip.y + 16 * s))
    path.curve(to: NSPoint(x: tip.x - 48 * s * flip, y: tip.y + 46 * s),
               controlPoint1: NSPoint(x: tip.x - 4 * s * flip, y: tip.y + 34 * s),
               controlPoint2: NSPoint(x: tip.x - 32 * s * flip, y: tip.y + 48 * s))
    // 钳尖外缘
    path.curve(to: NSPoint(x: tip.x - 56 * s * flip, y: tip.y + 2 * s),
               controlPoint1: NSPoint(x: tip.x - 64 * s * flip, y: tip.y + 28 * s),
               controlPoint2: NSPoint(x: tip.x - 64 * s * flip, y: tip.y + 14 * s))
    // 下缘回到根部
    path.curve(to: NSPoint(x: tip.x - 42 * s * flip, y: tip.y - 44 * s),
               controlPoint1: NSPoint(x: tip.x - 58 * s * flip, y: tip.y - 14 * s),
               controlPoint2: NSPoint(x: tip.x - 54 * s * flip, y: tip.y - 32 * s))
    path.curve(to: NSPoint(x: tip.x + 26 * s * flip, y: tip.y - 16 * s),
               controlPoint1: NSPoint(x: tip.x - 22 * s * flip, y: tip.y - 50 * s),
               controlPoint2: NSPoint(x: tip.x + 4 * s * flip, y: tip.y - 34 * s))
    path.close()

    // 钳口：挖成孔洞
    let notch = NSBezierPath()
    notch.move(to: NSPoint(x: tip.x - 52 * s * flip, y: tip.y + 8 * s))
    notch.line(to: NSPoint(x: tip.x - 10 * s * flip, y: tip.y + 2 * s))
    notch.line(to: NSPoint(x: tip.x - 50 * s * flip, y: tip.y - 10 * s))
    notch.close()
    path.append(notch)

    // NSBezierPath 没有 even-odd 属性，改用 CGContext 的填充规则
    strokeWhite.setFill()
    let ctx = NSGraphicsContext.current!.cgContext
    ctx.saveGState()
    ctx.addPath(path.cgPath)
    ctx.fillPath(using: .evenOdd)
    ctx.restoreGState()
}

/// 在 `unit` = 1（1024 基准）的坐标系里绘制蝎子，调用方负责缩放。
///
/// 布局要点：蝎子朝左。尾巴从体右升起、绕右侧成 C 形向下收，尾尖落在身体
/// 正下方的空白区；双螯向左张开；步足只向下。这样各部件互不穿插，
/// 小尺寸下也读得出轮廓。
private func drawScorpion(unit: CGFloat, zoom: CGFloat = 1.14) {
    // 以画布中心为原点缩放，使主体占据更接近主流图标的比例
    func sc(_ v: CGFloat) -> CGFloat { (v - canvas / 2) * zoom + canvas / 2 }
    func p(_ x: CGFloat, _ y: CGFloat) -> NSPoint {
        NSPoint(x: sc(x) * unit, y: sc(y) * unit)
    }
    func ell(_ cx: CGFloat, _ cy: CGFloat, _ w: CGFloat, _ h: CGFloat) -> NSBezierPath {
        NSBezierPath(ovalIn: NSRect(x: sc(cx - w / 2) * unit, y: sc(cy - h / 2) * unit,
                                    width: w * zoom * unit, height: h * zoom * unit))
    }
    func lw(_ v: CGFloat) -> CGFloat { v * zoom * unit }

    // ── 躯干：三段椭圆叠出分节体形 ──
    strokeWhite.setFill()
    ell(555, 460, 250, 150).fill()   // 腹部
    ell(425, 465, 185, 155).fill()   // 头胸部
    ell(665, 470, 130, 135).fill()   // 尾基

    // ── 步足：每侧三条，斜向下并略向后 ──
    let legs: [(CGFloat, CGFloat, CGFloat, CGFloat)] = [
        (420, 420, 322, 306), (468, 400, 396, 280), (508, 390, 466, 262),
        (602, 395, 642, 276), (648, 405, 716, 300),
    ]
    for (x0, y0, x1, y1) in legs {
        let leg = NSBezierPath()
        leg.lineWidth = lw(36)
        leg.lineCapStyle = .round
        leg.move(to: p(x0, y0))
        leg.curve(to: p(x1, y1),
                  controlPoint1: p(x0 - 8, y0 - (y0 - y1) * 0.4),
                  controlPoint2: p(x1 + 6, y1 + (y0 - y1) * 0.3))
        strokeWhite.setStroke()
        leg.stroke()
    }

    // ── 双螯：向左张开，末端是紧凑的 V 形钳口 ──
    let pincers: [(CGFloat, CGFloat, CGFloat, CGFloat, CGFloat)] = [
        // (起点x, 起点y, 尖端x, 尖端y, 线宽)
        (352, 496, 250, 566, 54),
        (348, 440, 258, 352, 50),
    ]
    for (bx, by, tx, ty, w) in pincers {
        let arm = NSBezierPath()
        arm.lineWidth = lw(w)
        arm.lineCapStyle = .round
        arm.move(to: p(bx, by))
        arm.curve(to: p(tx, ty),
                  controlPoint1: p(bx - 46, by + 34),
                  controlPoint2: p(tx + 54, ty - 44))
        strokeWhite.setStroke()
        arm.stroke()

        drawPincer(at: p(tx, ty), scale: unit * zoom * 1.14, flip: 1)
    }

    // ── 尾部：从尾基升起，绕右侧成 C 形，尾尖收向身体正下方 ──
    let tail = NSBezierPath()
    tail.lineWidth = lw(56)
    tail.lineCapStyle = .round
    tail.lineJoinStyle = .round
    tail.move(to: p(700, 490))
    tail.curve(to: p(840, 720), controlPoint1: p(805, 552), controlPoint2: p(838, 636))
    tail.curve(to: p(580, 870), controlPoint1: p(842, 812), controlPoint2: p(720, 872))
    tail.curve(to: p(505, 826), controlPoint1: p(560, 870), controlPoint2: p(524, 856))
    strokeWhite.setStroke()
    tail.stroke()

    // ── 毒刺：红色水滴，品牌的记忆点 ──
    let sting = NSBezierPath()
    sting.move(to: p(505, 826))
    sting.curve(to: p(468, 762), controlPoint1: p(488, 804), controlPoint2: p(476, 784))
    sting.curve(to: p(520, 772), controlPoint1: p(482, 754), controlPoint2: p(506, 762))
    sting.close()
    NSColor(srgbRed: 0.95, green: 0.29, blue: 0.33, alpha: 1).setFill()
    sting.fill()
}

// MARK: - 整图

private func drawIcon(size: CGFloat) -> NSBitmapImageRep {
    guard let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil,
        pixelsWide: Int(size), pixelsHigh: Int(size),
        bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true,
        isPlanar: false, colorSpaceName: .deviceRGB,
        bytesPerRow: 0, bitsPerPixel: 0
    ) else { fatalError("无法创建位图") }

    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)

    let rect = NSRect(origin: .zero, size: NSSize(width: size, height: size))

    // squircle 轮廓
    let mask = squircle(size: size)
    mask.addClip()

    // 品牌渐变：靛蓝 → 紫罗兰，对角方向
    let gradient = NSGradient(colors: [
        NSColor(srgbRed: 0.16, green: 0.29, blue: 0.66, alpha: 1),
        NSColor(srgbRed: 0.29, green: 0.20, blue: 0.60, alpha: 1),
        NSColor(srgbRed: 0.45, green: 0.24, blue: 0.66, alpha: 1),
    ])!
    gradient.draw(in: rect, angle: -60)

    // 左上柔光，增加体积感
    if let glow = NSGradient(
        starting: NSColor(white: 1, alpha: 0.20),
        ending: NSColor(white: 1, alpha: 0)
    ) {
        glow.draw(fromCenter: NSPoint(x: size * 0.30, y: size * 0.76),
                  radius: 0,
                  toCenter: NSPoint(x: size * 0.30, y: size * 0.76),
                  radius: size * 0.62,
                  options: [])
    }

    drawScorpion(unit: size / canvas)

    NSGraphicsContext.restoreGraphicsState()
    return rep
}

// MARK: - 输出

private let repoRoot = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()
    .deletingLastPathComponent()
private let iconsDir = repoRoot.appendingPathComponent("src-tauri/icons")
private let iconsetDir = iconsDir.appendingPathComponent("AppIcon.iconset")

func write(_ rep: NSBitmapImageRep, to url: URL) {
    guard let data = rep.representation(using: .png, properties: [:]) else {
        print("✗ 编码失败 \(url.lastPathComponent)")
        return
    }
    try? data.write(to: url)
}

if CommandLine.arguments.contains("--preview") {
    write(drawIcon(size: 512), to: URL(fileURLWithPath: "/tmp/icon_preview.png"))
    print("✓ 预览图 /tmp/icon_preview.png")
    exit(0)
}

try? FileManager.default.createDirectory(at: iconsetDir, withIntermediateDirectories: true)

let iconsetSpecs: [(String, CGFloat)] = [
    ("icon_16x16.png", 16), ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32), ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128), ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256), ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512), ("icon_512x512@2x.png", 1024),
]
for (name, px) in iconsetSpecs {
    write(drawIcon(size: px), to: iconsetDir.appendingPathComponent(name))
}

let tauriSizes: [(String, CGFloat)] = [
    ("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256), ("icon.png", 1024),
]
for (name, px) in tauriSizes {
    write(drawIcon(size: px), to: iconsDir.appendingPathComponent(name))
}

print("✓ 已写入 \(iconsDir.path)")