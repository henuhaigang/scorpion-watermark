import { useWatermarkStore } from '../store/useWatermarkStore'
import type { GridPosition, TileMode } from '../types/watermark'

const GRID_POSITIONS: { value: GridPosition; label: string }[] = [
  { value: 'topLeft', label: '左上' },
  { value: 'topCenter', label: '中上' },
  { value: 'topRight', label: '右上' },
  { value: 'middleLeft', label: '左中' },
  { value: 'center', label: '居中' },
  { value: 'middleRight', label: '右中' },
  { value: 'bottomLeft', label: '左下' },
  { value: 'bottomCenter', label: '中下' },
  { value: 'bottomRight', label: '右下' },
]

const SUPPORTED_CHARS = /^[a-zA-Z0-9\u4e00-\u9fff\u3000-\u303f\uff00-\uffef\s\p{P}]*$/u

export default function VisiblePanel() {
  const config = useWatermarkStore((s) => s.config)
  const updateVisible = useWatermarkStore((s) => s.updateVisible)
  const v = config.visible

  const isTextSupported = SUPPORTED_CHARS.test(v.text)

  return (
    <div style={{ marginBottom: '16px' }}>
      <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '12px' }}>可见水印</h3>

      <label style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px', fontSize: '13px' }}>
        <input
          type="checkbox"
          checked={v.enabled}
          onChange={(e) => updateVisible({ enabled: e.target.checked })}
        />
        启用
      </label>

      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>文字</label>
        <input
          type="text"
          value={v.text}
          onChange={(e) => updateVisible({ text: e.target.value })}
          style={{
            width: '100%',
            padding: '6px',
            border: isTextSupported ? '1px solid #ddd' : '1px solid #d32f2f',
            borderRadius: '4px',
          }}
        />
        {!isTextSupported && (
          <div style={{ fontSize: '11px', color: '#d32f2f', marginTop: '4px' }}>
            包含不支持的字符，部分字符可能无法显示
          </div>
        )}
      </div>

      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>模式</label>
        <select
          value={v.mode}
          onChange={(e) => updateVisible({ mode: e.target.value as TileMode })}
          style={{ width: '100%', padding: '6px', border: '1px solid #ddd', borderRadius: '4px' }}
        >
          <option value="tile">平铺</option>
          <option value="single">单点</option>
        </select>
      </div>

      {v.mode === 'single' && (
        <div style={{ marginBottom: '8px' }}>
          <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>位置</label>
          <select
            value={v.position ?? 'center'}
            onChange={(e) => updateVisible({ position: e.target.value as GridPosition })}
            style={{ width: '100%', padding: '6px', border: '1px solid #ddd', borderRadius: '4px' }}
          >
            {GRID_POSITIONS.map((p) => (
              <option key={p.value} value={p.value}>{p.label}</option>
            ))}
          </select>
        </div>
      )}

      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>
          角度：{v.angle}°
        </label>
        <input
          type="range"
          min={-90}
          max={90}
          value={v.angle}
          onChange={(e) => updateVisible({ angle: Number(e.target.value) })}
          style={{ width: '100%' }}
        />
      </div>

      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>
          透明度：{Math.round(v.opacity * 100)}%
        </label>
        <input
          type="range"
          min={0}
          max={100}
          value={v.opacity * 100}
          onChange={(e) => updateVisible({ opacity: Number(e.target.value) / 100 })}
          style={{ width: '100%' }}
        />
      </div>

      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>
          字号比例：{v.fontSizeRatio}%
        </label>
        <input
          type="range"
          min={0.5}
          max={5}
          step={0.1}
          value={v.fontSizeRatio}
          onChange={(e) => updateVisible({ fontSizeRatio: Number(e.target.value) })}
          style={{ width: '100%' }}
        />
      </div>

      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>
          行间距：{v.lineSpacing}
        </label>
        <input
          type="range"
          min={1}
          max={2}
          step={0.05}
          value={v.lineSpacing}
          onChange={(e) => updateVisible({ lineSpacing: Number(e.target.value) })}
          style={{ width: '100%' }}
        />
      </div>

      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>颜色</label>
        <input
          type="color"
          value={v.color}
          onChange={(e) => updateVisible({ color: e.target.value })}
          style={{ width: '100%', height: '32px', border: '1px solid #ddd', borderRadius: '4px' }}
        />
      </div>
    </div>
  )
}
