import { useCallback, useEffect, useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow } from '@tauri-apps/api/window'

interface FileDropZoneProps {
  filePath: string | null
  onFileSelect: (path: string) => void
}

export default function FileDropZone({ filePath, onFileSelect }: FileDropZoneProps) {
  const [isDragging, setIsDragging] = useState(false)

  useEffect(() => {
    const unlisten = getCurrentWindow().onDragDropEvent((event) => {
      if (event.payload.type === 'drop') {
        const paths = event.payload.paths
        if (paths && paths.length > 0) {
          onFileSelect(paths[0])
        }
      }
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [onFileSelect])

  const handleClick = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [
        { name: '图片', extensions: ['jpg', 'jpeg', 'png', 'gif', 'webp', 'heic', 'raw'] },
        { name: 'PDF', extensions: ['pdf'] },
      ],
    })
    if (selected && typeof selected === 'string') {
      onFileSelect(selected)
    }
  }, [onFileSelect])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)
  }, [])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)
  }, [])

  return (
    <div
      onClick={handleClick}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      style={{
        border: `2px dashed ${isDragging ? '#4a90d9' : '#ccc'}`,
        borderRadius: '8px',
        padding: '24px',
        textAlign: 'center',
        cursor: 'pointer',
        marginBottom: '16px',
        background: isDragging ? '#e6f3ff' : filePath ? '#f0f8ff' : '#fafafa',
        transition: 'all 0.2s',
      }}
    >
      <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>
        {filePath ? '已选择文件' : '点击或拖放文件'}
      </div>
      {filePath && (
        <div style={{ fontSize: '12px', color: '#333', wordBreak: 'break-all' }}>
          {filePath.split('/').pop()}
        </div>
      )}
    </div>
  )
}
