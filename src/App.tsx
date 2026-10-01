import { useState } from 'react'
import FileDropZone from './components/FileDropZone'
import PreviewCanvas from './components/PreviewCanvas'
import VisiblePanel from './components/VisiblePanel'
import InvisiblePanel from './components/InvisiblePanel'
import OutputPanel from './components/OutputPanel'
import PresetBar from './components/PresetBar'

function App() {
  const [filePath, setFilePath] = useState<string | null>(null)

  return (
    <div style={{ display: 'flex', height: '100vh', flexDirection: 'column' }}>
      <header style={{ padding: '12px 24px', borderBottom: '1px solid #e0e0e0' }}>
        <h1 style={{ fontSize: '20px', fontWeight: 600 }}>蝎子水印</h1>
      </header>

      <PresetBar />

      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        <div style={{ width: '320px', borderRight: '1px solid #e0e0e0', overflowY: 'auto', padding: '16px' }}>
          <FileDropZone filePath={filePath} onFileSelect={setFilePath} />
          <VisiblePanel />
          <InvisiblePanel filePath={filePath} />
          <OutputPanel filePath={filePath} />
        </div>

        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', background: '#f5f5f5' }}>
          <PreviewCanvas filePath={filePath} />
        </div>
      </div>
    </div>
  )
}

export default App
