import { Stage, Layer, Image as KonvaImage, Text } from 'react-konva'
import { useEffect, useState, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useWatermarkStore } from '../store/useWatermarkStore'

interface PreviewCanvasProps {
  filePath: string | null
}

const MAX_STAGE_WIDTH = 800
const MAX_STAGE_HEIGHT = 600

interface LayoutItem {
  x: number
  y: number
  rotation: number
}

interface LayoutResult {
  items: LayoutItem[]
  font_size: number
  text_width: number
  text_height: number
  text: string
  lines: string[]
  font_auto_shrunk: boolean
  missing_glyphs: string[]
}

const LINE_HEIGHT_BASE = 1.0
const CJK_FONT_FAMILY = "'Scorpion Watermark SC', sans-serif"

export default function PreviewCanvas({ filePath }: PreviewCanvasProps) {
  const [image, setImage] = useState<HTMLImageElement | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [imageSize, setImageSize] = useState<{ width: number; height: number } | null>(null)
  const [layout, setLayout] = useState<LayoutResult | null>(null)
  const [stageSize, setStageSize] = useState({ width: MAX_STAGE_WIDTH, height: MAX_STAGE_HEIGHT })
  const [fontReady, setFontReady] = useState(false)
  const config = useWatermarkStore((s) => s.config)

  // Konva 的 Text 依赖 canvas measureText 排版，字体未加载完会用 fallback 字体
  // 测量并绘制，导致文字重叠/错位。必须等字体就绪后再排版。
  useEffect(() => {
    let cancelled = false

    const load = async () => {
      if (typeof document === 'undefined' || !document.fonts) {
        if (!cancelled) setFontReady(true)
        return
      }
      try {
        // 用当前水印文字触发具体字形的加载
        await document.fonts.load(`400 32px 'Scorpion Watermark SC'`, config.visible.text || '水印')
        await document.fonts.ready
      } catch (err) {
        console.error('水印字体加载失败，回退到系统字体:', err)
      }
      if (!cancelled) setFontReady(true)
    }

    load()
    return () => {
      cancelled = true
    }
  }, [config.visible.text])

  useEffect(() => {
    if (!filePath) {
      setImage(null)
      setError(null)
      setImageSize(null)
      setLayout(null)
      return
    }

    let cancelled = false
    setLoading(true)
    setError(null)

    invoke<number[]>('generate_preview', { path: filePath })
      .then((bytes) => {
        if (cancelled) return
        const blob = new Blob([new Uint8Array(bytes)], { type: 'image/jpeg' })
        const url = URL.createObjectURL(blob)
        const img = new window.Image()
        img.onload = () => {
          if (!cancelled) {
            setImage(img)
            setImageSize({ width: img.naturalWidth, height: img.naturalHeight })
            setLoading(false)
          }
          URL.revokeObjectURL(url)
        }
        img.onerror = () => {
          if (!cancelled) {
            setError('加载图片失败')
            setLoading(false)
          }
          URL.revokeObjectURL(url)
        }
        img.src = url
      })
      .catch((err) => {
        if (!cancelled) {
          setError(String(err))
          setLoading(false)
        }
      })

    return () => {
      cancelled = true
    }
  }, [filePath])

  useEffect(() => {
    if (!imageSize) return

    const scale = Math.min(MAX_STAGE_WIDTH / imageSize.width, MAX_STAGE_HEIGHT / imageSize.height, 1.0)
    const w = Math.round(imageSize.width * scale)
    const h = Math.round(imageSize.height * scale)
    setStageSize({ width: w, height: h })
  }, [imageSize])

  useEffect(() => {
    if (!imageSize || !fontReady) return
    if (!config.visible.enabled || !config.visible.text) {
      setLayout(null)
      return
    }

    let cancelled = false

    invoke<LayoutResult>('calculate_layout', {
      canvasWidth: imageSize.width,
      canvasHeight: imageSize.height,
      config: config.visible,
    })
      .then((result) => {
        if (cancelled) return
        const scale = Math.min(MAX_STAGE_WIDTH / imageSize.width, MAX_STAGE_HEIGHT / imageSize.height, 1.0)
        const scaledResult: LayoutResult = {
          ...result,
          font_size: result.font_size * scale,
          text_width: result.text_width * scale,
          text_height: result.text_height * scale,
          items: result.items.map((item) => ({
            ...item,
            x: item.x * scale,
            y: item.y * scale,
          })),
        }
        setLayout(scaledResult)
      })
      .catch((err) => {
        if (!cancelled) {
          console.error('calculate_layout 失败:', err)
        }
      })

    return () => {
      cancelled = true
    }
  }, [imageSize, config.visible, fontReady])

  const watermarkElements = useMemo(() => {
    if (!layout || !config.visible.enabled || !config.visible.text) return null

    const lineHeight = layout.font_size * (LINE_HEIGHT_BASE + Math.max(config.visible.lineSpacing, 0))

    return layout.items.map((item, index) => (
      <Text
        key={index}
        text={layout.text}
        x={item.x}
        y={item.y}
        fontSize={layout.font_size}
        fontFamily={CJK_FONT_FAMILY}
        lineHeight={lineHeight}
        fill={config.visible.color}
        opacity={config.visible.opacity}
        rotation={item.rotation}
        offsetX={layout.text_width / 2}
        offsetY={layout.text_height / 2}
      />
    ))
  }, [layout, config.visible])

  if (!filePath) {
    return (
      <div style={{ color: '#999', fontSize: '16px' }}>
        选择文件以预览
      </div>
    )
  }

  if (loading) {
    return (
      <div style={{ color: '#666', fontSize: '14px' }}>
        加载预览中...
      </div>
    )
  }

  if (error) {
    return (
      <div style={{ color: '#d32f2f', fontSize: '14px' }}>
        错误：{error}
      </div>
    )
  }

  return (
    <div style={{ maxWidth: '100%', maxHeight: '100%', overflow: 'auto' }}>
      <Stage width={stageSize.width} height={stageSize.height}>
        <Layer>
          {image && (
            <KonvaImage
              image={image}
              width={stageSize.width}
              height={stageSize.height}
              listening={false}
            />
          )}
          {watermarkElements}
        </Layer>
      </Stage>
      {layout && layout.missing_glyphs.length > 0 && (
        <div style={{ marginTop: '6px', fontSize: '12px', color: '#d32f2f', textAlign: 'center' }}>
          字体缺少这些字符，将显示为方块或回退字体：{layout.missing_glyphs.join(' ')}
        </div>
      )}
      {layout?.font_auto_shrunk && (
        <div style={{ marginTop: '8px', fontSize: '12px', color: '#e08600', textAlign: 'center' }}>
          文字较长，已自动缩小字号并换行以完整显示
        </div>
      )}
    </div>
  )
}
