import { useEffect, useRef, useState } from 'react'
import { useWatermarkStore } from '../store/useWatermarkStore'
import type { GridPosition, TileMode, OutputFormat } from '../types/watermark'
import CloseIcon from './CloseIcon'

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

interface SettingsBottomSheetProps {
  isOpen: boolean
  onClose: () => void
  activeTab: 'visible' | 'output'
  onTabChange: (tab: 'visible' | 'output') => void
  filePath: string | null
}

export default function SettingsBottomSheet({
  isOpen,
  onClose,
  activeTab,
  onTabChange,
  filePath,
}: SettingsBottomSheetProps) {
  const config = useWatermarkStore((s) => s.config)
  const updateVisible = useWatermarkStore((s) => s.updateVisible)
  const updateOutput = useWatermarkStore((s) => s.updateOutput)
  const overlayRef = useRef<HTMLDivElement>(null)
  const sheetRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    if (isOpen) {
      document.addEventListener('keydown', handleKeyDown)
      document.body.style.overflow = 'hidden'
    }
    return () => {
      document.removeEventListener('keydown', handleKeyDown)
      document.body.style.overflow = ''
    }
  }, [isOpen, onClose])

  if (!isOpen) return null

  return (
    <>
      <div
        ref={overlayRef}
        onClick={onClose}
        style={{
          position: 'fixed',
          inset: 0,
          background: 'rgba(0,0,0,0.45)',
          zIndex: 100,
          opacity: 1,
          transition: 'opacity 0.2s',
        }}
      />
      <div
        ref={sheetRef}
        style={{
          position: 'fixed',
          bottom: 0,
          left: 0,
          right: 0,
          height: '65vh',
          maxHeight: '65vh',
          background: 'rgba(255,255,255,0.9)',
          backdropFilter: 'blur(24px)',
          WebkitBackdropFilter: 'blur(24px)',
          borderTopLeftRadius: '24px',
          borderTopRightRadius: '24px',
          zIndex: 101,
          display: 'flex',
          flexDirection: 'column',
          boxShadow: '0 -8px 40px rgba(0,0,0,0.15)',
          borderTop: '1px solid rgba(255,255,255,0.4)',
          transform: 'translateY(0)',
          transition: 'transform 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94)',
        }}
      >
        <div style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '12px 16px',
          borderBottom: '1px solid #e0e0e0',
          position: 'sticky',
          top: 0,
          background: '#fff',
          borderTopLeftRadius: '16px',
          borderTopRightRadius: '16px',
          zIndex: 1,
        }}>
          <div style={{ width: 24 }} />
          <h3 style={{ fontSize: '16px', fontWeight: 600, margin: 0, flex: 1, textAlign: 'center' }}>
            {activeTab === 'visible' ? '可见水印' : '输出设置'}
          </h3>
          <button
            onClick={onClose}
            style={{
              width: 24,
              height: 24,
              border: 'none',
              background: 'transparent',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              borderRadius: '50%',
              color: '#666',
            }}
          >
            <CloseIcon />
          </button>
        </div>

        <div style={{
          display: 'flex',
          padding: '8px 16px 0',
          borderBottom: '1px solid #e0e0e0',
          background: '#fff',
          position: 'sticky',
          top: 52,
          zIndex: 1,
        }}>
          <button
            onClick={() => onTabChange('visible')}
            style={{
              flex: 1,
              padding: '10px 16px',
              border: 'none',
              background: activeTab === 'visible' ? '#4a90d9' : 'transparent',
              color: activeTab === 'visible' ? '#fff' : '#333',
              fontSize: '14px',
              fontWeight: 500,
              cursor: 'pointer',
              borderRadius: '8px 8px 0 0',
              marginBottom: '-1px',
              borderBottom: activeTab === 'visible' ? '2px solid #4a90d9' : '2px solid transparent',
            }}
          >
            可见水印
          </button>
          <button
            onClick={() => onTabChange('output')}
            style={{
              flex: 1,
              padding: '10px 16px',
              border: 'none',
              background: activeTab === 'output' ? '#4a90d9' : 'transparent',
              color: activeTab === 'output' ? '#fff' : '#333',
              fontSize: '14px',
              fontWeight: 500,
              cursor: 'pointer',
              borderRadius: '8px 8px 0 0',
              marginBottom: '-1px',
              borderBottom: activeTab === 'output' ? '2px solid #4a90d9' : '2px solid transparent',
            }}
          >
            输出设置
          </button>
        </div>

        <div style={{
          flex: 1,
          overflowY: 'auto',
          padding: '16px',
          paddingBottom: '100px',
        }}>
          {activeTab === 'visible' && (
            <VisibleSettingsPanel config={config.visible} updateVisible={updateVisible} />
          )}
          {activeTab === 'output' && (
            <OutputSettingsPanel
              config={config.output}
              updateOutput={updateOutput}
              filePath={filePath}
            />
          )}
        </div>

        <div style={{
          position: 'sticky',
          bottom: 0,
          padding: '12px 16px',
          paddingBottom: 'calc(12px + env(safe-area-inset-bottom))',
          borderTop: '1px solid rgba(0,0,0,0.08)',
          background: 'rgba(255,255,255,0.9)',
          backdropFilter: 'blur(10px)',
          WebkitBackdropFilter: 'blur(10px)',
          borderBottomLeftRadius: '20px',
          borderBottomRightRadius: '20px',
          display: 'flex',
          gap: '12px',
        }}>
          <button
            onClick={onClose}
            style={{
              flex: 1,
              padding: '12px',
              border: '1px solid #ddd',
              borderRadius: '8px',
              background: '#fff',
              fontSize: '14px',
              fontWeight: 500,
              color: '#333',
              cursor: 'pointer',
            }}
          >
            取消
          </button>
          <button
            onClick={onClose}
            style={{
              flex: 1,
              padding: '12px',
              border: 'none',
              borderRadius: '8px',
              background: '#4a90d9',
              fontSize: '14px',
              fontWeight: 500,
              color: '#fff',
              cursor: 'pointer',
            }}
          >
            完成
          </button>
        </div>
      </div>
    </>
  )
}

function VisibleSettingsPanel({
  config,
  updateVisible,
}: {
  config: any
  updateVisible: (updates: any) => void
}) {
  const v = config

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <div>
        <label style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px', fontSize: '13px' }}>
          <input
            type="checkbox"
            checked={v.enabled}
            onChange={(e) => updateVisible({ enabled: e.target.checked })}
          />
          启用可见水印
        </label>
      </div>

      <div>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>水印文字</label>
        <input
          type="text"
          value={v.text}
          onChange={(e) => updateVisible({ text: e.target.value })}
          style={{
            width: '100%',
            padding: '12px',
            border: '1px solid #ddd',
            borderRadius: '8px',
            fontSize: '16px',
          }}
        />
      </div>

      <div>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>模式</label>
        <select
          value={v.mode}
          onChange={(e) => updateVisible({ mode: e.target.value as TileMode })}
          style={{ width: '100%', padding: '12px', border: '1px solid #ddd', borderRadius: '8px', fontSize: '16px' }}
        >
          <option value="tile">平铺</option>
          <option value="single">单点</option>
        </select>
      </div>

      {v.mode === 'single' && (
        <div>
          <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>位置</label>
          <select
            value={v.position ?? 'center'}
            onChange={(e) => updateVisible({ position: e.target.value as GridPosition })}
            style={{ width: '100%', padding: '12px', border: '1px solid #ddd', borderRadius: '8px', fontSize: '16px' }}
          >
            {GRID_POSITIONS.map((p) => (
              <option key={p.value} value={p.value}>{p.label}</option>
            ))}
          </select>
        </div>
      )}

      <div>
        <label style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', color: '#666', marginBottom: '4px' }}>
          <span>角度</span>
          <span>{v.angle}°</span>
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

      <div>
        <label style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', color: '#666', marginBottom: '4px' }}>
          <span>透明度</span>
          <span>{Math.round(v.opacity * 100)}%</span>
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

      <div>
        <label style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', color: '#666', marginBottom: '4px' }}>
          <span>字号比例</span>
          <span>{v.fontSizeRatio}%</span>
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

      <div>
        <label style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', color: '#666', marginBottom: '4px' }}>
          <span>行间距</span>
          <span>{v.lineSpacing}</span>
        </label>
        <input
          type="range"
          min={0}
          max={1.5}
          step={0.05}
          value={v.lineSpacing}
          onChange={(e) => updateVisible({ lineSpacing: Number(e.target.value) })}
          style={{ width: '100%' }}
        />
        <div style={{ fontSize: '11px', color: '#999', marginTop: '2px' }}>
          同一块水印内多行文字之间的距离
        </div>
      </div>

      <div>
        <label style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', color: '#666', marginBottom: '4px' }}>
          <span>水印间距</span>
          <span>{v.blockGapRatio}</span>
        </label>
        <input
          type="range"
          min={0}
          max={3}
          step={0.05}
          value={v.blockGapRatio}
          onChange={(e) => updateVisible({ blockGapRatio: Number(e.target.value) })}
          style={{ width: '100%' }}
        />
        <div style={{ fontSize: '11px', color: '#999', marginTop: '2px' }}>
          相邻水印块之间的距离（平铺时的疏密）
        </div>
      </div>

      <div>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>颜色</label>
        <input
          type="color"
          value={v.color}
          onChange={(e) => updateVisible({ color: e.target.value })}
          style={{ width: '100%', height: '44px', border: '1px solid #ddd', borderRadius: '8px' }}
        />
      </div>
    </div>
  )
}

function OutputSettingsPanel({
  config,
  updateOutput,
  filePath,
}: {
  config: any
  updateOutput: (updates: any) => void
  filePath: string | null
}) {
  const fullConfig = useWatermarkStore((s) => s.config)
  const [exporting, setExporting] = useState(false)
  const [exportError, setExportError] = useState<string | null>(null)
  const [exportSuccess, setExportSuccess] = useState(false)

  const handleExport = async () => {
    console.log('[handleExport] >>> Clicked, filePath:', filePath)
    if (!filePath) {
      console.log('[handleExport] No filePath, returning')
      return
    }

    setExporting(true)
    setExportError(null)
    setExportSuccess(false)

    try {
      console.log('[handleExport] Importing invoke...')
      const { invoke } = await import('@tauri-apps/api/core')
      console.log('[handleExport] Invoking export_file with:', { inputPath: filePath, visible: fullConfig.visible, invisible: fullConfig.invisible, output: fullConfig.output })
      await invoke('export_file', {
        inputPath: filePath,
        config: {
          visible: fullConfig.visible,
          invisible: fullConfig.invisible,
          output: fullConfig.output,
        },
      })
      console.log('[handleExport] export_file succeeded')
      setExportSuccess(true)
    } catch (err) {
      console.error('[handleExport] export_file error:', err)
      setExportError(String(err))
    } finally {
      setExporting(false)
    }
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <div>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>输出格式</label>
        <select
          value={config.format}
          onChange={(e) => updateOutput({ format: e.target.value as OutputFormat })}
          style={{ width: '100%', padding: '12px', border: '1px solid #ddd', borderRadius: '8px', fontSize: '16px' }}
        >
          <option value="sameAsInput">与原图相同</option>
          <option value="jpeg">JPEG</option>
          <option value="png">PNG</option>
          <option value="heic">HEIC</option>
        </select>
      </div>

      <label style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '13px', marginBottom: '12px' }}>
        <input
          type="checkbox"
          checked={config.stripMetadata}
          onChange={(e) => updateOutput({ stripMetadata: e.target.checked })}
        />
        去除元数据（EXIF/GPS/IPTC/XMP）
      </label>

      <button
        onClick={handleExport}
        disabled={!filePath || exporting}
        style={{
          width: '100%',
          padding: '14px',
          border: 'none',
          borderRadius: '8px',
          background: !filePath || exporting ? '#ccc' : '#4a90d9',
          color: '#fff',
          fontSize: '16px',
          fontWeight: 500,
          cursor: !filePath || exporting ? 'not-allowed' : 'pointer',
        }}
      >
        {exporting ? '导出中...' : '导出图片'}
      </button>

      {exportError && (
        <div style={{ marginTop: '8px', padding: '12px', background: '#ffebee', borderRadius: '8px', fontSize: '13px', color: '#d32f2f' }}>
          {exportError}
        </div>
      )}

      {exportSuccess && (
        <div style={{ marginTop: '8px', padding: '12px', background: '#e8f5e9', borderRadius: '8px', fontSize: '13px', color: '#2e7d32' }}>
          导出成功！
        </div>
      )}
    </div>
  )
}