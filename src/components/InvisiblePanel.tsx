import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useWatermarkStore } from '../store/useWatermarkStore'

interface InvisiblePanelProps {
  filePath: string | null
}

export default function InvisiblePanel({ filePath }: InvisiblePanelProps) {
  const config = useWatermarkStore((s) => s.config)
  const updateInvisible = useWatermarkStore((s) => s.updateInvisible)
  const [extracting, setExtracting] = useState(false)
  const [extractResult, setExtractResult] = useState<string | null>(null)
  const [extractError, setExtractError] = useState<string | null>(null)

  const handleExtract = async () => {
    if (!filePath || !config.invisible?.key) return

    setExtracting(true)
    setExtractResult(null)
    setExtractError(null)

    try {
      const result = await invoke<string>('extract_invisible_watermark', {
        path: filePath,
        key: config.invisible.key,
      })
      setExtractResult(result)
    } catch (err) {
      setExtractError(String(err))
    } finally {
      setExtracting(false)
    }
  }

  return (
    <div style={{ marginBottom: '16px' }}>
      <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '12px' }}>隐形水印</h3>

      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>载荷</label>
        <input
          type="text"
          value={config.invisible?.payload ?? ''}
          onChange={(e) => updateInvisible({ payload: e.target.value })}
          placeholder="用户ID / 订单号 / 时间戳"
          style={{ width: '100%', padding: '6px', border: '1px solid #ddd', borderRadius: '4px' }}
        />
      </div>

      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>密钥</label>
        <input
          type="text"
          value={config.invisible?.key ?? ''}
          onChange={(e) => updateInvisible({ key: e.target.value })}
          placeholder="密钥"
          style={{ width: '100%', padding: '6px', border: '1px solid #ddd', borderRadius: '4px' }}
        />
      </div>

      <button
        onClick={handleExtract}
        disabled={!filePath || !config.invisible?.key || extracting}
        style={{
          width: '100%',
          padding: '8px',
          border: 'none',
          borderRadius: '4px',
          background: !filePath || !config.invisible?.key || extracting ? '#ccc' : '#4a90d9',
          color: '#fff',
          fontSize: '13px',
          cursor: !filePath || !config.invisible?.key || extracting ? 'not-allowed' : 'pointer',
          marginBottom: '8px',
        }}
      >
        {extracting ? '提取中...' : '提取水印'}
      </button>

      {extractResult !== null && (
        <div style={{ padding: '8px', background: '#e8f5e9', borderRadius: '4px', fontSize: '12px', color: '#2e7d32', marginBottom: '8px' }}>
          提取结果：{extractResult}
        </div>
      )}

      {extractError && (
        <div style={{ padding: '8px', background: '#ffebee', borderRadius: '4px', fontSize: '12px', color: '#d32f2f', marginBottom: '8px' }}>
          {extractError}
        </div>
      )}

      <div style={{ fontSize: '11px', color: '#999', lineHeight: 1.4 }}>
        隐形水印在导出时自动嵌入，可抵抗裁剪和重压缩。
      </div>
    </div>
  )
}
