import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useWatermarkStore } from '../store/useWatermarkStore'
import type { OutputFormat } from '../types/watermark'

interface OutputPanelProps {
  filePath: string | null
}

export default function OutputPanel({ filePath }: OutputPanelProps) {
  const config = useWatermarkStore((s) => s.config)
  const updateOutput = useWatermarkStore((s) => s.updateOutput)
  const [exporting, setExporting] = useState(false)
  const [exportError, setExportError] = useState<string | null>(null)
  const [exportSuccess, setExportSuccess] = useState(false)

  const handleExport = async () => {
    if (!filePath) return

    setExporting(true)
    setExportError(null)
    setExportSuccess(false)

    try {
      await invoke('export_file', {
        inputPath: filePath,
        config,
      })
      setExportSuccess(true)
    } catch (err) {
      setExportError(String(err))
    } finally {
      setExporting(false)
    }
  }

  return (
    <div style={{ marginBottom: '16px' }}>
      <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '12px' }}>输出</h3>

      <div style={{ marginBottom: '8px' }}>
        <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '4px' }}>格式</label>
        <select
          value={config.output.format}
          onChange={(e) => updateOutput({ format: e.target.value as OutputFormat })}
          style={{ width: '100%', padding: '6px', border: '1px solid #ddd', borderRadius: '4px' }}
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
          checked={config.output.stripMetadata}
          onChange={(e) => updateOutput({ stripMetadata: e.target.checked })}
        />
        去除元数据（EXIF/GPS/IPTC/XMP）
      </label>

      <button
        onClick={handleExport}
        disabled={!filePath || exporting}
        style={{
          width: '100%',
          padding: '10px',
          border: 'none',
          borderRadius: '4px',
          background: !filePath || exporting ? '#ccc' : '#4a90d9',
          color: '#fff',
          fontSize: '14px',
          fontWeight: 500,
          cursor: !filePath || exporting ? 'not-allowed' : 'pointer',
        }}
      >
        {exporting ? '导出中...' : '导出'}
      </button>

      {exportError && (
        <div style={{ marginTop: '8px', padding: '8px', background: '#ffebee', borderRadius: '4px', fontSize: '12px', color: '#d32f2f' }}>
          {exportError}
        </div>
      )}

      {exportSuccess && (
        <div style={{ marginTop: '8px', padding: '8px', background: '#e8f5e9', borderRadius: '4px', fontSize: '12px', color: '#2e7d32' }}>
          导出成功！
        </div>
      )}
    </div>
  )
}
