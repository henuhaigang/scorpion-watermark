import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useWatermarkStore } from '../store/useWatermarkStore'

export default function PresetBar() {
  const config = useWatermarkStore((s) => s.config)
  const setConfig = useWatermarkStore((s) => s.setConfig)
  const [presets, setPresets] = useState<string[]>([])
  const [selectedPreset, setSelectedPreset] = useState<string>('')
  const [newPresetName, setNewPresetName] = useState('')
  const [message, setMessage] = useState<string | null>(null)

  useEffect(() => {
    loadPresets()
  }, [])

  const loadPresets = async () => {
    try {
      const list = await invoke<string[]>('list_presets')
      setPresets(list)
    } catch (err) {
      console.error('加载预设失败:', err)
    }
  }

  const handleSave = async () => {
    if (!newPresetName.trim()) {
      setMessage('请输入预设名称')
      return
    }

    try {
      await invoke('save_preset', { name: newPresetName.trim(), config })
      setMessage(`预设"${newPresetName}"已保存`)
      setNewPresetName('')
      await loadPresets()
    } catch (err) {
      setMessage(`保存失败：${err}`)
    }
  }

  const handleLoad = async () => {
    if (!selectedPreset) {
      setMessage('请选择预设')
      return
    }

    try {
      const preset = await invoke('load_preset', { name: selectedPreset })
      setConfig(preset as typeof config)
      setMessage(`预设"${selectedPreset}"已加载`)
    } catch (err) {
      setMessage(`加载失败：${err}`)
    }
  }

  const handleDelete = async () => {
    if (!selectedPreset) {
      setMessage('请选择预设')
      return
    }

    try {
      await invoke('delete_preset', { name: selectedPreset })
      setMessage(`预设"${selectedPreset}"已删除`)
      setSelectedPreset('')
      await loadPresets()
    } catch (err) {
      setMessage(`删除失败：${err}`)
    }
  }

  return (
    <div style={{ padding: '8px 24px', borderBottom: '1px solid #e0e0e0', display: 'flex', gap: '8px', alignItems: 'center', flexWrap: 'wrap' }}>
      <input
        type="text"
        value={newPresetName}
        onChange={(e) => setNewPresetName(e.target.value)}
        placeholder="新预设名称"
        style={{ padding: '6px', border: '1px solid #ddd', borderRadius: '4px', width: '140px' }}
      />
      <button
        onClick={handleSave}
        style={{ padding: '6px 12px', border: '1px solid #ddd', borderRadius: '4px', background: '#fff', cursor: 'pointer' }}
      >
        保存
      </button>

      <select
        value={selectedPreset}
        onChange={(e) => setSelectedPreset(e.target.value)}
        style={{ padding: '6px', border: '1px solid #ddd', borderRadius: '4px', minWidth: '120px' }}
      >
        <option value="">选择预设</option>
        {presets.map((name) => (
          <option key={name} value={name}>{name}</option>
        ))}
      </select>

      <button
        onClick={handleLoad}
        disabled={!selectedPreset}
        style={{ padding: '6px 12px', border: '1px solid #ddd', borderRadius: '4px', background: '#fff', cursor: selectedPreset ? 'pointer' : 'not-allowed' }}
      >
        加载
      </button>

      <button
        onClick={handleDelete}
        disabled={!selectedPreset}
        style={{ padding: '6px 12px', border: '1px solid #ddd', borderRadius: '4px', background: '#fff', cursor: selectedPreset ? 'pointer' : 'not-allowed' }}
      >
        删除
      </button>

      {message && (
        <span style={{ fontSize: '12px', color: '#666' }}>{message}</span>
      )}
    </div>
  )
}
