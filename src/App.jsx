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

const MODELS = [
  { value: 'bs-roformer-cpp', label: '6-Track Premium', description: 'Best quality (BS-RoFormer.cpp). 6 instruments. GPU accelerated (~30sec) or CPU (~25min)', recommended: true },
  { value: 'demucs', label: '4-Track Standard', description: 'Good quality (HTDemucs). 4 instruments. Fast CPU processing', recommended: false },
  { value: 'bs-roformer', label: '4-Track Essential', description: 'Alternative (BS-RoFormer). 4 instruments. Slower, CPU only', recommended: false }
]

function App() {
  const [selectedModel, setSelectedModel] = useState('bs-roformer-cpp')
  const [selectedFile, setSelectedFile] = useState(null)
  const [metadata, setMetadata] = useState(null)
  const [stemPaths, setStemPaths] = useState([])
  const [processing, setProcessing] = useState(false)
  const [progress, setProgress] = useState({ stage: '', percent: 0, message: '' })
  const [error, setError] = useState(null)
  const [downloading, setDownloading] = useState(false)
  const unlistenRef = useRef(null)

  const clearFile = useCallback(() => {
    setSelectedFile(null)
    setMetadata(null)
    setStemPaths([])
    setProcessing(false)
    setDownloading(false)
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
        } else if (data && data.stage === 'downloading') {
          setDownloading(true)
          setProgress({
            stage: 'Downloading model',
            percent: data.percent || 0,
            message: data.message || 'This may take a moment...'
          })
        } else if (data && data.message && !data.percent) {
          setProgress({
            stage: data.message,
            percent: progress.percent,
            message: ''
          })
        }
      })
    } catch (err) {
      console.error('Failed to listen for progress:', err)
    }
  }, [selectedModel, progress.percent])

  const handleSeparate = useCallback(async () => {
    if (!selectedFile) return

    setProcessing(true)
    setDownloading(false)
    setProgress({ stage: 'Initializing', percent: 0, message: 'Analyzing file...' })

    await setupProgressListener()

    try {
      const result = await invoke('process_file', {
        path: selectedFile,
        model: selectedModel,
      })

      setMetadata(result)
      setStemPaths(result.paths)
      setProgress({ stage: 'Complete', percent: 100, message: 'Stems ready' })
      setProcessing(false)
      setDownloading(false)
    } catch (err) {
      setError(String(err))
      setProcessing(false)
      setDownloading(false)
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

  const selectedModelInfo = MODELS.find(m => m.value === selectedModel)

  return (
    <div className="min-h-screen bg-[#0a0a0a] text-white flex flex-col antialiased">
      <header className="px-6 py-4 border-b border-[#1a1a1a] flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-green-600/20 flex items-center justify-center">
            <svg className="w-4 h-4 text-green-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 19V6l12-3v13M9 19c0 1.126-0.895 2-2 2s-2-0.874-2-2 0.895-2 2-2 2 0.874 2 2zm12-13c0 1.126-0.895 2-2 2s-2-0.874-2-2 0.895-2 2-2 2 0.874 2 2z" />
            </svg>
          </div>
          <div>
            <h1 className="text-sm font-semibold tracking-tight">Stem Separator</h1>
            <span className="text-[10px] text-[#666]">v0.3.0</span>
          </div>
        </div>
        {metadata && (
          <button
            onClick={clearFile}
            className="text-xs text-[#666] hover:text-white transition-colors px-3 py-1.5 rounded-lg hover:bg-[#1a1a1a]"
          >
            New file
          </button>
        )}
      </header>

      <main className="flex-1 flex flex-col items-center px-6 py-8">
        <div className="w-full max-w-2xl">
          {!selectedFile && !processing && !downloading && (
            <div className="space-y-8">
              <div className="text-center mb-4">
                <h2 className="text-xl font-semibold tracking-tight mb-2">Separate any track into stems</h2>
                <p className="text-sm text-[#888]">Choose a model, drop your file, and get isolated drums, bass, vocals, and more.</p>
              </div>

              <ModelSelector
                selectedModel={selectedModel}
                onSelect={setSelectedModel}
                models={MODELS}
              />

              <Dropzone onFileSelect={handleFileSelect} />
            </div>
          )}

          {downloading && selectedFile && (
            <div className="space-y-4">
              <FileInfo
                fileName={fileName}
                duration={0}
                sampleRate={0}
                channels={0}
                model={selectedModel}
                backend={null}
                onNewFile={clearFile}
              />
              <div className="bg-[#1a1a1a] border border-[#2a2a2a] rounded-xl p-6">
                <div className="flex items-center gap-3 mb-3">
                  <div className="w-5 h-5 border-2 border-green-500 border-t-transparent rounded-full animate-spin" />
                  <div>
                    <div className="text-sm font-medium">Downloading model</div>
                    <div className="text-xs text-[#888]">{progress.message || 'First-time setup. This may take a moment.'}</div>
                  </div>
                </div>
                <div className="w-full bg-[#0f0f0f] rounded-full h-2 overflow-hidden">
                  <div
                    className="bg-green-600 h-full rounded-full transition-all duration-300"
                    style={{ width: `${progress.percent}%` }}
                  />
                </div>
                <div className="text-xs text-[#666] text-right mt-2">{Math.round(progress.percent)}%</div>
              </div>
            </div>
          )}

          {processing && selectedFile && (
            <div className="space-y-4">
              <FileInfo
                fileName={fileName}
                duration={metadata?.duration_secs || 0}
                sampleRate={metadata?.sample_rate || 0}
                channels={metadata?.channels || 0}
                model={selectedModel}
                backend={metadata?.backend || null}
                onNewFile={clearFile}
              />
              <ProgressBar
                stage={progress.stage}
                percent={progress.percent}
                message={progress.message}
                model={selectedModel}
              />
            </div>
          )}

          {selectedFile && !processing && !downloading && metadata && stemPaths.length > 0 && (
            <div className="space-y-6">
              <FileInfo
                fileName={fileName}
                duration={metadata.duration_secs}
                sampleRate={metadata.sample_rate}
                channels={metadata.channels}
                model={metadata.model}
                backend={metadata.backend || null}
                onNewFile={clearFile}
              />
              <StemResults
                stemPaths={stemPaths}
                onOpenFolder={handleOpenOutputFolder}
                onNewFile={clearFile}
              />
            </div>
          )}

          {selectedFile && !processing && !downloading && !metadata && (
            <div className="space-y-4">
              <FileInfo
                fileName={fileName}
                duration={0}
                sampleRate={0}
                channels={0}
                model={selectedModel}
                backend={null}
                onNewFile={clearFile}
              />
              <button
                onClick={handleSeparate}
                className="w-full bg-green-600 hover:bg-green-500 text-white font-semibold px-8 py-4 rounded-xl transition-all text-sm shadow-lg shadow-green-600/20 hover:shadow-green-500/30 active:scale-[0.99]"
              >
                Separate Stems
              </button>
            </div>
          )}
        </div>
      </main>

      {error && (
        <ErrorBanner message={error} onDismiss={dismissError} />
      )}

      <footer className="px-6 py-3 border-t border-[#1a1a1a] flex items-center justify-between">
        <span className="text-[9px] text-[#444] tracking-wide">
          Stem Separator — Local AI audio separation
        </span>
        <span className="text-[9px] text-[#444] tracking-wide">
          {selectedModelInfo?.label || MODELS[0].label} · 4-Track Standard · 4-Track Essential · 6-Track Premium
        </span>
      </footer>
    </div>
  )
}

export default App
