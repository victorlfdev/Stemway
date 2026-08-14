import { useState, useEffect, useRef, useCallback } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import Dropzone from './components/Dropzone'
import ModelSelector from './components/ModelSelector'
import FileInfo from './components/FileInfo'
import ProgressBar from './components/ProgressBar'
import StemResults from './components/StemResults'
import ErrorBanner from './components/ErrorBanner'

const EVENT_MAP = {
  'demucs': 'demucs_progress',
  'bs-roformer': 'bs_roformer_progress',
  'bs-roformer-cpp': 'bs_roformer_cpp_progress'
}

function App() {
  const [selectedModel, setSelectedModel] = useState('demucs')
  const [selectedFile, setSelectedFile] = useState(null)
  const [metadata, setMetadata] = useState(null)
  const [stemPaths, setStemPaths] = useState([])
  const [processing, setProcessing] = useState(false)
  const [progress, setProgress] = useState({ stage: '', percent: 0, message: '' })
  const [error, setError] = useState(null)
  const unlistenRef = useRef(null)

  const clearFile = useCallback(() => {
    setSelectedFile(null)
    setMetadata(null)
    setStemPaths([])
    setProcessing(false)
    setProgress({ stage: '', percent: 0, message: '' })
    setError(null)
    if (unlistenRef.current) {
      unlistenRef.current()
      unlistenRef.current = null
    }
  }, [])

  const dismissError = useCallback(() => {
    setError(null)
  }, [])

  const handleFileSelect = useCallback((path) => {
    clearFile()
    setSelectedFile(path)
  }, [clearFile])

  const setupProgressListener = useCallback(async () => {
    if (unlistenRef.current) {
      unlistenRef.current()
    }

    const eventName = EVENT_MAP[selectedModel] || 'demucs_progress'

    try {
      unlistenRef.current = await listen(eventName, (event) => {
        const data = event.payload
        if (data && typeof data.percent === 'number') {
          setProgress({
            stage: data.stage || data.message || 'Processing',
            percent: data.percent,
            message: data.message || ''
          })
        }
      })
    } catch (err) {
      console.error('Failed to listen for progress:', err)
    }
  }, [selectedModel])

  const handleSeparate = useCallback(async () => {
    if (!selectedFile) return

    setProcessing(true)
    setProgress({ stage: 'Starting', percent: 0, message: 'Initializing...' })

    await setupProgressListener()

    try {
      const result = await invoke('process_file', {
        path: selectedFile,
        model: selectedModel,
      })

      setMetadata(result)
      setStemPaths(result.paths)
      setProgress({ stage: 'Complete', percent: 100, message: 'Done!' })
      setProcessing(false)
    } catch (err) {
      setError(`Error: ${err}`)
      setProcessing(false)
    }
  }, [selectedFile, selectedModel, setupProgressListener])

  const handleOpenOutputFolder = useCallback(async (path) => {
    try {
      await invoke('open_output_folder', { path })
    } catch (err) {
      setError(`Failed to open folder: ${err}`)
    }
  }, [])

  const fileName = selectedFile ? selectedFile.split('/').pop() : null

  return (
    <div className="min-h-screen bg-[#0a0a0a] text-white flex flex-col">
      <header className="px-6 py-3 border-b border-[#1a1a1a] flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-lg">🎵</span>
          <h1 className="text-sm font-semibold">Stem Separator</h1>
          <span className="text-[10px] text-gray-600">v0.3.0</span>
        </div>
      </header>

      <div className="flex-1 flex flex-col items-center px-6 py-12">
        <div className="w-full max-w-lg">
          {!selectedFile && !processing && (
            <>
              <ModelSelector
                selectedModel={selectedModel}
                onSelect={setSelectedModel}
              />
              <div className="mt-6">
                <Dropzone onFileSelect={handleFileSelect} />
              </div>
            </>
          )}

          {processing && selectedFile && (
            <ProgressBar {...progress} />
          )}

          {selectedFile && !processing && metadata && (
            <>
              <FileInfo
                fileName={fileName}
                duration={metadata.duration_secs}
                sampleRate={metadata.sample_rate}
                channels={metadata.channels}
                model={metadata.model}
                backend={metadata.backend}
                onNewFile={clearFile}
              />
              <StemResults
                stemPaths={stemPaths}
                onOpenFolder={handleOpenOutputFolder}
                onNewFile={clearFile}
              />
            </>
          )}

          {selectedFile && !processing && !metadata && (
            <>
              <FileInfo
                fileName={fileName}
                duration={0}
                sampleRate={0}
                channels={0}
                model={selectedModel}
                onNewFile={clearFile}
              />
              <button
                onClick={handleSeparate}
                className="w-full bg-green-600 hover:bg-green-500 text-white font-semibold px-8 py-3.5 rounded-xl transition-colors text-sm shadow-lg shadow-green-600/20"
              >
                Separate Stems
              </button>
            </>
          )}
        </div>
      </div>

      {error && (
        <ErrorBanner message={error} onDismiss={dismissError} />
      )}

      <footer className="px-6 py-2 border-t border-[#1a1a1a] text-center text-[9px] text-gray-600">
        Stem Separator v0.3.0 — HTDemucs, BS-RoFormer, BS-RoFormer.cpp (Vulkan/CPU)
      </footer>
    </div>
  )
}

export default App
