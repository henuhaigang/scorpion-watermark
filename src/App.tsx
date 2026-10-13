import { useState, useEffect } from 'react'
import FileDropZone from './components/FileDropZone'
import PreviewCanvas from './components/PreviewCanvas'
import VisiblePanel from './components/VisiblePanel'
import OutputPanel from './components/OutputPanel'
import PresetBar from './components/PresetBar'
import SettingsBottomSheet from './components/SettingsBottomSheet'
import { useWatermarkStore } from './store/useWatermarkStore'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

function App() {
  const [filePath, setFilePath] = useState<string | null>(null)
  const [isMobile, setIsMobile] = useState(false)
  const [showSettings, setShowSettings] = useState(false)
  const [activeTab, setActiveTab] = useState<'visible' | 'output'>('visible')
  const [showSourceMenu, setShowSourceMenu] = useState(false)

  useEffect(() => {
    const checkMobile = () => setIsMobile(window.innerWidth < 768)
    checkMobile()
    window.addEventListener('resize', checkMobile)
    return () => window.removeEventListener('resize', checkMobile)
  }, [])

  const config = useWatermarkStore((s) => s.config)

  const handleFilePick = async (source: 'gallery' | 'camera' | 'file') => {
    setShowSourceMenu(false)
    try {
      console.log('[handleFilePick] >>> Calling pick_image with source:', source)
      const result = await invoke('pick_image', {
        options: { source, maxWidth: 4096, maxHeight: 4096, quality: 90 }
      }) as { path: string }
      console.log('[handleFilePick] <<< pick_image returned:', JSON.stringify(result))
      if (result?.path) {
        console.log('[handleFilePick] Setting filePath:', result.path)
        setFilePath(result.path)
      } else {
        console.warn('[handleFilePick] No path in result:', JSON.stringify(result))
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      if (errorMsg.includes('cancelled') || errorMsg.includes('Cancelled') || errorMsg.includes('User cancelled')) {
        console.log('User cancelled image picker')
        return
      }
      console.warn('Native picker failed, falling back to file dialog:', err)
      const selected = await open({
        multiple: false,
        filters: [
          { name: '图片', extensions: ['jpg', 'jpeg', 'png', 'gif', 'webp', 'heic', 'raw'] },
          { name: 'PDF', extensions: ['pdf'] },
        ],
      })
      if (selected && typeof selected === 'string') {
        setFilePath(selected)
      }
    }
  }

const handleExport = async () => {
    if (!filePath) return
    try {
      await invoke('export_file', { 
        inputPath: filePath, 
        config: {
          visible: config.visible,
          invisible: config.invisible,
          output: config.output,
        }
      })
      alert('导出成功！')
    } catch (err) {
      alert('导出失败: ' + err)
    }
  }

  useEffect(() => {
    const testIPC = async () => {
      try {
        const result = await invoke('test_ipc') as string
        console.log('test_ipc result:', result)
        alert('test_ipc: ' + result)
      } catch (err) {
        console.error('test_ipc error:', err)
        alert('test_ipc error: ' + err)
      }
    }
    testIPC()
  }, [])

  if (isMobile) {
    return (
      <div style={{ display: 'flex', height: '100vh', flexDirection: 'column', background: '#fafafa' }}>
        {/* Fixed Header - stays at top */}
        <header style={{
          padding: '12px 16px',
          background: 'rgba(255,255,255,0.95)',
          backdropFilter: 'blur(10px)',
          webkitBackdropFilter: 'blur(10px)',
          borderBottom: '1px solid #e0e0e0',
          position: 'sticky',
          top: 0,
          zIndex: 20,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}>
          <h1 style={{ fontSize: '18px', fontWeight: 600, margin: 0, color: '#1a1a1a' }}>蝎子水印</h1>
        </header>

        {/* Main content area - scrollable between header and bottom bar */}
        <div style={{ 
          flex: 1, 
          position: 'relative', 
          background: '#f5f5f5', 
          display: 'flex', 
          flexDirection: 'column',
          overflow: 'hidden'
        }}>
          {/* Preview area - fills available space */}
          <div style={{ flex: 1, position: 'relative', minHeight: 0, overflow: 'hidden' }}>
            <PreviewCanvas filePath={filePath} />
            
            {/* Watermark status badge - positioned above bottom bar */}
            {filePath && (
              <div style={{
                position: 'absolute',
                bottom: 90,
                left: 16,
                right: 16,
                background: 'rgba(255,255,255,0.9)',
                backdropFilter: 'blur(10px)',
                webkitBackdropFilter: 'blur(10px)',
                borderRadius: '12px',
                padding: '10px 14px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                boxShadow: '0 2px 12px rgba(0,0,0,0.08)',
                fontSize: '13px',
                border: '1px solid rgba(0,0,0,0.05)'
              }}>
                <span style={{ color: '#333' }}>
                  {config.visible.enabled ? `水印: "${config.visible.text}"` : '水印已关闭'}
                </span>
                <button
                  onClick={() => setShowSourceMenu(true)}
                  style={{
                    padding: '6px 12px',
                    border: '1px solid #ddd',
                    borderRadius: '8px',
                    background: '#fff',
                    fontSize: '12px',
                    color: '#4a90d9',
                    cursor: 'pointer',
                  }}
                >
                  换图片
                </button>
              </div>
            )}

            {/* Empty state - centered */}
            {!filePath && (
              <div style={{
                position: 'absolute',
                inset: 0,
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                padding: '20px',
              }}>
                <div style={{ 
                  width: '80px', 
                  height: '80px', 
                  borderRadius: '20px', 
                  background: 'linear-gradient(135deg, #4a90d9 0%, #357abd 100%)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  marginBottom: '16px',
                  boxShadow: '0 8px 24px rgba(74,144,217,0.3)'
                }}>
                  <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2">
                    <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                    <circle cx="8.5" cy="8.5" r="1.5" />
                    <polyline points="21 15 16 10 5 21" />
                  </svg>
                </div>
                <p style={{ color: '#666', fontSize: '16px', marginTop: 0, marginBottom: '8px', fontWeight: 500 }}>
                  选择图片开始
                </p>
                <p style={{ color: '#999', fontSize: '13px', marginTop: 0, marginBottom: '24px', textAlign: 'center' }}>
                  支持相册、拍照、文件选择
                </p>
                <button
                  onClick={() => setShowSourceMenu(true)}
                  style={{
                    width: '100%',
                    maxWidth: 280,
                    padding: '16px',
                    border: 'none',
                    borderRadius: '14px',
                    background: 'linear-gradient(135deg, #4a90d9 0%, #357abd 100%)',
                    color: '#fff',
                    fontSize: '16px',
                    fontWeight: 600,
                    cursor: 'pointer',
                    boxShadow: '0 4px 16px rgba(74,144,217,0.4)',
                  }}
                >
                  选择图片
                </button>
              </div>
            )}
          </div>

          {/* Fixed bottom bar - always at bottom */}
          <div style={{
            position: 'sticky',
            bottom: 0,
            padding: '10px 16px',
            paddingBottom: 'calc(10px + env(safe-area-inset-bottom))',
            background: 'rgba(255,255,255,0.95)',
            backdropFilter: 'blur(10px)',
            webkitBackdropFilter: 'blur(10px)',
            borderTop: '1px solid #e0e0e0',
            display: 'flex',
            flexDirection: 'column',
            gap: '10px',
            zIndex: 15,
          }}>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button
                onClick={() => { setActiveTab('visible'); setShowSettings(true); }}
                style={{
                  flex: 1,
                  padding: '14px',
                  border: 'none',
                  borderRadius: '12px',
                  background: activeTab === 'visible' ? '#4a90d9' : 'rgba(74,144,217,0.1)',
                  color: activeTab === 'visible' ? '#fff' : '#4a90d9',
                  fontSize: '15px',
                  fontWeight: 600,
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                }}
              >
                可见水印
              </button>
              <button
                onClick={() => { setActiveTab('output'); setShowSettings(true); }}
                style={{
                  flex: 1,
                  padding: '14px',
                  border: 'none',
                  borderRadius: '12px',
                  background: activeTab === 'output' ? '#4a90d9' : 'rgba(74,144,217,0.1)',
                  color: activeTab === 'output' ? '#fff' : '#4a90d9',
                  fontSize: '15px',
                  fontWeight: 600,
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                }}
              >
                输出设置
              </button>
              {filePath && (
                <button
                  onClick={handleExport}
                  style={{
                    flex: 1,
                    padding: '14px',
                    border: 'none',
                    borderRadius: '12px',
                    background: 'linear-gradient(135deg, #4a90d9 0%, #357abd 100%)',
                    color: '#fff',
                    fontSize: '15px',
                    fontWeight: 600,
                    cursor: 'pointer',
                    boxShadow: '0 2px 8px rgba(74,144,217,0.3)',
                  }}
                >
                  导出
                </button>
              )}
            </div>
          </div>
        </div>

        <SettingsBottomSheet
          isOpen={showSettings}
          onClose={() => setShowSettings(false)}
          activeTab={activeTab}
          onTabChange={setActiveTab}
          filePath={filePath}
        />

        {/* Source picker menu - frosted glass */}
        {showSourceMenu && (
          <div
            style={{
              position: 'fixed',
              bottom: 0,
              left: 0,
              right: 0,
              background: 'rgba(255,255,255,0.9)',
              backdropFilter: 'blur(20px)',
              webkitBackdropFilter: 'blur(20px)',
              borderTopLeftRadius: '20px',
              borderTopRightRadius: '20px',
              padding: '16px',
              paddingBottom: 'calc(16px + env(safe-area-inset-bottom))',
              zIndex: 1000,
              boxShadow: '0 -4px 32px rgba(0,0,0,0.12)',
              borderTop: '1px solid rgba(255,255,255,0.3)',
            }}
            onClick={() => setShowSourceMenu(false)}
          >
            <div style={{ 
              width: '40px', 
              height: '5px', 
              borderRadius: '3px', 
              background: '#ddd', 
              margin: '0 auto 16px',
              opacity: 0.6
            }} />
            <div style={{ textAlign: 'center', marginBottom: '16px', fontSize: '16px', fontWeight: 600, color: '#1a1a1a' }}>
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
                borderRadius: '14px',
                background: '#f8f9fa',
                fontSize: '16px',
                color: '#1a1a1a',
                cursor: 'pointer',
                marginBottom: '10px',
                transition: 'background 0.15s',
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
                borderRadius: '14px',
                background: '#f8f9fa',
                fontSize: '16px',
                color: '#1a1a1a',
                cursor: 'pointer',
                marginBottom: '10px',
                transition: 'background 0.15s',
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
                borderRadius: '14px',
                background: '#f8f9fa',
                fontSize: '16px',
                color: '#1a1a1a',
                cursor: 'pointer',
                transition: 'background 0.15s',
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
          </div>
        )}
      </div>
    )
  }

  return (
    <div style={{ display: 'flex', height: '100vh', flexDirection: 'column', background: '#fafafa' }}>
      <header style={{ padding: '12px 24px', borderBottom: '1px solid #e0e0e0', background: '#fff' }}>
        <h1 style={{ fontSize: '20px', fontWeight: 600, margin: 0 }}>蝎子水印</h1>
      </header>

      <PresetBar />

      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        <div style={{ width: '340px', borderRight: '1px solid #e0e0e0', overflowY: 'auto', padding: '16px', background: '#fff' }}>
          <FileDropZone filePath={filePath} onFileSelect={setFilePath} />
          <VisiblePanel />
          <OutputPanel filePath={filePath} />
        </div>

        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', background: '#f5f5f5', minWidth: 0 }}>
          <PreviewCanvas filePath={filePath} />
        </div>
      </div>
    </div>
  )
}

export default App