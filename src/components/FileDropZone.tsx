import { useCallback, useEffect, useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'

interface FileDropZoneProps {
  filePath: string | null
  onFileSelect: (path: string) => void
  style?: React.CSSProperties
}

type ImageSource = 'gallery' | 'camera' | 'file'

export default function FileDropZone({ filePath, onFileSelect, style }: FileDropZoneProps) {
  const [isDragging, setIsDragging] = useState(false)
  const [showSourceMenu, setShowSourceMenu] = useState(false)
  const [isMobile, setIsMobile] = useState(false)

  useEffect(() => {
    const checkMobile = () => setIsMobile(window.innerWidth < 768)
    checkMobile()
    window.addEventListener('resize', checkMobile)
    return () => window.removeEventListener('resize', checkMobile)
  }, [])

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

  const handleFilePick = useCallback(async (source: ImageSource) => {
    setShowSourceMenu(false)
    try {
      const result = await invoke('pick_image', {
        options: {
          source,
          maxWidth: 4096,
          maxHeight: 4096,
          quality: 90,
        }
      }) as { path: string }
      if (result?.path) {
        console.log('Selected file path:', result.path)
        onFileSelect(result.path)
      }
    } catch (err) {
      // Don't fallback if user cancelled the picker
      const errorMsg = err instanceof Error ? err.message : String(err)
      if (errorMsg.includes('cancelled') || errorMsg.includes('Cancelled') || errorMsg.includes('User cancelled')) {
        console.log('User cancelled image picker')
        return
      }
      // Fallback to file dialog if native picker fails
      console.warn('Native picker failed, falling back to file dialog:', err)
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
    }
  }, [onFileSelect])

  const handleClick = useCallback(async () => {
    if (isMobile) {
      setShowSourceMenu(true)
    } else {
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
    }
  }, [onFileSelect, isMobile])

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
    <>
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
          ...style,
        }}
      >
        <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>
          {filePath ? '已选择文件' : isMobile ? '选择图片来源' : '点击或拖放文件'}
        </div>
        {filePath && (
          <div style={{ fontSize: '12px', color: '#333', wordBreak: 'break-all' }}>
            {filePath.split('/').pop()}
          </div>
        )}
      </div>

      {showSourceMenu && (
        <div
          style={{
            position: 'fixed',
            bottom: 0,
            left: 0,
            right: 0,
            background: '#fff',
            borderTopLeftRadius: '16px',
            borderTopRightRadius: '16px',
            padding: '16px',
            paddingBottom: '32px',
            zIndex: 1000,
            boxShadow: '0 -4px 24px rgba(0,0,0,0.15)',
          }}
          onClick={() => setShowSourceMenu(false)}
        >
          <div style={{ textAlign: 'center', marginBottom: '16px', fontSize: '14px', color: '#666' }}>
            选择图片来源
          </div>
          <button
            onClick={(e) => { e.stopPropagation(); handleFilePick('gallery'); }}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '12px',
              width: '100%',
              padding: '16px',
              border: 'none',
              borderRadius: '12px',
              background: '#f5f5f5',
              fontSize: '16px',
              color: '#333',
              cursor: 'pointer',
              marginBottom: '12px',
            }}
          >
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
              <circle cx="8.5" cy="8.5" r="1.5" />
              <polyline points="21 15 16 10 5 21" />
            </svg>
            从相册选择
          </button>
          <button
            onClick={(e) => { e.stopPropagation(); handleFilePick('camera'); }}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '12px',
              width: '100%',
              padding: '16px',
              border: 'none',
              borderRadius: '12px',
              background: '#f5f5f5',
              fontSize: '16px',
              color: '#333',
              cursor: 'pointer',
              marginBottom: '12px',
            }}
          >
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z" />
              <circle cx="12" cy="13" r="4" />
            </svg>
            拍照
          </button>
          <button
            onClick={(e) => { e.stopPropagation(); handleFilePick('file'); }}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '12px',
              width: '100%',
              padding: '16px',
              border: 'none',
              borderRadius: '12px',
              background: '#f5f5f5',
              fontSize: '16px',
              color: '#333',
              cursor: 'pointer',
              marginBottom: '12px',
            }}
          >
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="16" y1="13" x2="8" y2="13" />
              <line x1="16" y1="17" x2="8" y2="17" />
              <polyline points="10 9 9 9 8 9" />
            </svg>
            从文件选择
          </button>
          <button
            onClick={async (e) => {
              e.stopPropagation();
              try {
                const files = await invoke('debug_list_temp_dir') as string[]
                console.log('Temp dir files:', files)
                alert('Temp dir files: ' + JSON.stringify(files))
              } catch (err) {
                console.error('Debug failed:', err)
                alert('Debug failed: ' + err)
              }
            }}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '12px',
              width: '100%',
              padding: '16px',
              border: 'none',
              borderRadius: '12px',
              background: '#e0e0e0',
              fontSize: '16px',
              color: '#333',
              cursor: 'pointer',
            }}
          >
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
            </svg>
            调试: 列出临时目录
          </button>
        </div>
      )}
    </>
  )
}
