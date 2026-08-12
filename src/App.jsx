import { useState, useRef, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { dirname, basename } from '@tauri-apps/api/path'

function formatDuration(seconds) {
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

function formatFileSize(bytes) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function DropZone({ onFileLoaded }) {
  const [isDragging, setIsDragging] = useState(false)
  const fileInputRef = useRef(null)

  const handleDrop = useCallback((e) => {
    e.preventDefault()
    setIsDragging(false)
    const file = e.dataTransfer?.files[0]
    if (file) {
      onFileLoaded(file)
    }
  }, [onFileLoaded])

  const handleDragOver = useCallback((e) => {
    e.preventDefault()
    setIsDragging(true)
  }, [])

  const handleDragLeave = useCallback(() => {
    setIsDragging(false)
  }, [])

  const handleClick = useCallback(() => {
    fileInputRef.current?.click()
  }, [])

  const handleChange = useCallback((e) => {
    const file = e.target.files[0]
    if (file) {
      onFileLoaded(file)
    }
  }, [onFileLoaded])

  return (
    <div
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onClick={handleClick}
      className={`border-2 border-dashed rounded-xl p-10 text-center cursor-pointer transition-all ${
        isDragging
          ? 'border-green-400 bg-green-400/10 scale-[1.02]'
          : 'border-[#1a1a1a] hover:border-[#2a2a2a] hover:bg-[#111]'
      }`}
    >
      <input
        ref={fileInputRef}
        type="file"
        accept=".wav,.mp3,.flac,.ogg"
        onChange={handleChange}
        className="hidden"
      />
      <div className="text-4xl mb-3">🎵</div>
      <p className="text-gray-300 text-lg mb-2">Drop audio file here</p>
      <p className="text-gray-500 text-sm">WAV, MP3, FLAC, OGG</p>
    </div>
  )
}

function FileInfo({ metadata }) {
  return (
    <div className="bg-[#111] rounded-xl p-5 border border-[#1a1a1a]">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">{metadata.name}</h3>
        <span className="text-gray-500 text-sm">{formatFileSize(metadata.file_size_bytes)}</span>
      </div>
      <div className="grid grid-cols-3 gap-4 text-sm">
        <div>
          <div className="text-gray-500 mb-1">Duration</div>
          <div className="text-white font-mono">{formatDuration(metadata.duration_secs)}</div>
        </div>
        <div>
          <div className="text-gray-500 mb-1">Sample Rate</div>
          <div className="text-white font-mono">{metadata.sample_rate} Hz</div>
        </div>
        <div>
          <div className="text-gray-500 mb-1">Channels</div>
          <div className="text-white font-mono">{metadata.channels === 1 ? 'Mono' : metadata.channels === 2 ? 'Stereo' : `${metadata.channels}`}</div>
        </div>
      </div>
    </div>
  )
}

function ProgressBar({ stage, progress, message }) {
  return (
    <div className="bg-[#111] rounded-xl p-5 border border-[#1a1a1a]">
      <div className="flex items-center justify-between mb-3">
        <span className="text-sm font-medium">{stage}</span>
        <span className="text-sm text-gray-400">{Math.round(progress * 100)}%</span>
      </div>
      <div className="w-full bg-[#1a1a1a] rounded-full h-2 overflow-hidden">
        <div
          className="bg-green-500 h-full rounded-full transition-all duration-300"
          style={{ width: `${progress * 100}%` }}
        />
      </div>
      {message && <p className="text-gray-500 text-xs mt-2">{message}</p>}
    </div>
  )
}

function StemPlayer({ stem, index, onLoaded }) {
  const audioRef = useRef(null)
  const [playing, setPlaying] = useState(false)
  const [playingIndex, setPlayingIndex] = useState(null)
  const [volume, setVolume] = useState(1)
  const [muted, setMuted] = useState(false)
  const [solo, setSolo] = useState(false)
  const [duration, setDuration] = useState(0)
  const [currentTime, setCurrentTime] = useState(0)

  useEffect(() => {
    if (audioRef.current) {
      audioRef.current.volume = volume * (muted ? 0 : 1)
    }
  }, [volume, muted])

  const handlePlay = useCallback(() => {
    if (audioRef.current) {
      if (playing && playingIndex === index) {
        audioRef.current.pause()
        setPlaying(false)
        setPlayingIndex(null)
      } else {
        audioRef.current.play()
        setPlaying(true)
        setPlayingIndex(index)
      }
    }
  }, [playing, playingIndex, index])

  const handleTimeUpdate = useCallback(() => {
    if (audioRef.current) {
      setCurrentTime(audioRef.current.currentTime)
    }
  }, [])

  const handleLoadedMetadata = useCallback(() => {
    if (audioRef.current) {
      setDuration(audioRef.current.duration)
    }
  }, [])

  const handleSeek = useCallback((e) => {
    if (audioRef.current) {
      audioRef.current.currentTime = parseFloat(e.target.value)
    }
  }, [])

  const stemColors = ['#f28c00', '#1db954', '#e91e63', '#9c27b0']
  const stemLabels = ['Vocals', 'Bass', 'Drums', 'Other']

  return (
    <div className="bg-[#111] rounded-xl p-4 border border-[#1a1a1a]">
      <div className="flex items-center gap-3 mb-3">
        <div className="w-10 h-10 rounded-full flex items-center justify-center" style={{ backgroundColor: `${stemColors[index]}22`, color: stemColors[index] }}>
          <span className="text-sm font-bold">{index + 1}</span>
        </div>
        <div className="flex-1">
          <div className="text-sm font-medium">{stemLabels[index]}</div>
          <div className="text-xs text-gray-500">{stem}</div>
        </div>
      </div>

      <audio
        ref={audioRef}
        src={`file://${stem}`}
        onTimeUpdate={handleTimeUpdate}
        onLoadedMetadata={handleLoadedMetadata}
        onEnded={() => { setPlaying(false); setPlayingIndex(null) }}
        className="hidden"
      />

      <div className="flex items-center gap-3">
        <button
          onClick={handlePlay}
          className="w-10 h-10 rounded-full flex items-center justify-center transition-colors"
          style={{ backgroundColor: playing && playingIndex === index ? stemColors[index] : '#1a1a1a', color: playing && playingIndex === index ? '#000' : '#fff' }}
        >
          {playing && playingIndex === index ? (
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><rect x="3" y="2" width="3.5" height="12" rx="1" /><rect x="9.5" y="2" width="3.5" height="12" rx="1" /></svg>
          ) : (
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><polygon points="4,2 14,8 4,14" /></svg>
          )}
        </button>

        <input
          type="range"
          min="0"
          max={duration || 100}
          value={currentTime}
          onChange={handleSeek}
          className="flex-1 accent-green-500"
          style={{ accentColor: stemColors[index] }}
        />

        <span className="text-xs font-mono text-gray-400 w-12 text-right">
          {formatDuration(currentTime)}
        </span>
      </div>

      <div className="flex items-center gap-2 mt-3">
        <button
          onClick={() => setMuted(!muted)}
          className={`text-xs px-2 py-1 rounded transition-colors ${muted ? 'bg-red-500/20 text-red-400' : 'bg-[#1a1a1a] text-gray-400 hover:bg-[#2a2a2a]'}`}
        >
          {muted ? '🔇' : '🔊'}
        </button>
        <button
          onClick={() => setSolo(!solo)}
          className={`text-xs px-2 py-1 rounded transition-colors ${solo ? 'bg-yellow-500/20 text-yellow-400' : 'bg-[#1a1a1a] text-gray-400 hover:bg-[#2a2a2a]'}`}
        >
          Solo
        </button>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={volume}
          onChange={(e) => setVolume(parseFloat(e.target.value))}
          className="flex-1 accent-green-500"
          style={{ accentColor: stemColors[index] }}
        />
      </div>
    </div>
  )
}

function ExportButton({ stemPaths, onExported }) {
  const [exporting, setExporting] = useState(false)
  const [exported, setExported] = useState(false)

  const handleExport = useCallback(async () => {
    setExporting(true)
    try {
      if (stemPaths && stemPaths.length > 0) {
        const dir = await dirname(stemPaths[0])
        onExported?.(stemPaths, dir)
        setExported(true)
        setTimeout(() => setExported(false), 3000)
      }
    } catch (err) {
      console.error('Export error:', err)
    }
    setExporting(false)
  }, [stemPaths, onExported])

  return (
    <button
      onClick={handleExport}
      disabled={exporting || !stemPaths || stemPaths.length === 0}
      className="bg-green-500 hover:bg-green-400 disabled:bg-gray-600 disabled:cursor-not-allowed text-black font-bold py-3 px-8 rounded-full transition-colors"
    >
      {exporting ? '⏳ Saving...' : exported ? '✅ Saved!' : '💾 Export Stems'}
    </button>
  )
}

function App() {
  const [fileMetadata, setFileMetadata] = useState(null)
  const [processing, setProcessing] = useState(false)
  const [progress, setProgress] = useState({ stage: '', progress: 0, message: '' })
  const [stemPaths, setStemPaths] = useState(null)
  const [error, setError] = useState(null)
  const [appReady, setAppReady] = useState(false)

  useEffect(() => {
    setAppReady(true)
  }, [])

  const onFileLoaded = useCallback(async (file) => {
    setError(null)
    setProcessing(false)
    setStemPaths(null)

    try {
      const path = file.path || file.webkitRelativePath || URL.createObjectURL(file)
      const meta = {
        name: file.name.replace(/\.[^/.]+$/, ''),
        path,
        file_size_bytes: file.size,
        duration_secs: 0,
        sample_rate: 0,
        channels: 0,
      }

      if (file.path) {
        try {
          const metadata = await invoke('analyze_file', { path: file.path })
          setFileMetadata(metadata)
        } catch (err) {
          console.warn('analyze_file failed, using basic info:', err)
          setFileMetadata(meta)
        }
      } else {
        setFileMetadata(meta)
      }

      const outputDir = (await dirname(file.path || '/tmp')).replace(/\\/g, '/')

      setProcessing(true)
      setProgress({ stage: 'Loading', progress: 0.1, message: 'Analyzing audio file...' })

      try {
        const paths = await invoke('process_audio_file', { path: file.path || '', output_dir: outputDir })
        setStemPaths(paths)
        setProgress({ stage: 'Complete', progress: 1, message: 'Stems separated successfully!' })
      } catch (err) {
        setError(`Processing failed: ${err}`)
        setProcessing(false)
      }
    } catch (err) {
      setError(`Error loading file: ${err}`)
    }
  }, [])

  const handleExported = useCallback((paths, dir) => {
    console.log('Stems exported to:', dir)
  }, [])

  const stemData = stemPaths ? stemPaths.map((p, i) => ({ path: p, index: i })) : []

  return (
    <div className="min-h-screen bg-[#0a0a0a] text-white">
      <header className="px-6 py-4 border-b border-[#1a1a1a] flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-xl">🎵</span>
          <h1 className="text-xl font-semibold">Stem Separator</h1>
        </div>
        <span className="text-xs text-gray-500">Powered by HTDemucs ONNX</span>
      </header>

      <main className="max-w-4xl mx-auto py-8 px-6">
        {!appReady && (
          <div className="text-center py-20 text-gray-500">Initializing...</div>
        )}

        {appReady && !fileMetadata && (
          <div className="mb-8">
            <DropZone onFileLoaded={onFileLoaded} />
          </div>
        )}

        {appReady && fileMetadata && (
          <>
            <div className="mb-6">
              <FileInfo metadata={fileMetadata} />
            </div>

            {error && (
              <div className="mb-6 bg-red-500/10 border border-red-500/20 rounded-xl p-4 text-red-400">
                {error}
              </div>
            )}

            {processing && (
              <div className="mb-6">
                <ProgressBar {...progress} />
              </div>
            )}

            {stemPaths && stemPaths.length > 0 && (
              <div className="space-y-3 mb-6">
                {stemData.map((stem) => (
                  <StemPlayer
                    key={stem.index}
                    stem={stem.path}
                    index={stem.index}
                    onLoaded={true}
                  />
                ))}
              </div>
            )}

            {stemPaths && stemPaths.length > 0 && (
              <div className="flex justify-center mt-6">
                <ExportButton stemPaths={stemPaths} onExported={handleExported} />
              </div>
            )}

            {appReady && !fileMetadata && processing && (
              <div className="text-center">
                <DropZone onFileLoaded={onFileLoaded} />
              </div>
            )}
          </>
        )}
      </main>

      <footer className="px-6 py-4 border-t border-[#1a1a1a] text-center text-xs text-gray-600">
        <p>Stem Separator v0.1.0 — HTDemucs ONNX model by StemSplitio</p>
      </footer>
    </div>
  )
}

export default App
