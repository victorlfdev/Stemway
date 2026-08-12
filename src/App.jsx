import { useState, useCallback, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import TransportBar from './components/TransportBar'
import TrackRow from './components/TrackRow'
import TimeRuler from './components/TimeRuler'

export function formatDuration(seconds) {
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

const STEMS = [
  { name: 'Bass', color: '#f97316' },
  { name: 'Drums', color: '#4ade80' },
  { name: 'Guitar', color: '#eab308' },
  { name: 'Other', color: '#a855f7' },
  { name: 'Piano', color: '#e11d48' },
  { name: 'Vocals', color: '#3b82f6' },
]

function App() {
  const [metadata, setMetadata] = useState(null)
  const [stemPaths, setStemPaths] = useState([])
  const [stemBase64, setStemBase64] = useState({})
  const [processing, setProcessing] = useState(false)
  const [progress, setProgress] = useState({ stage: '', percent: 0, message: '' })
  const [error, setError] = useState(null)

  const [isPlaying, setIsPlaying] = useState(false)
  const [isStopped, setIsStopped] = useState(true)
  const [currentTime, setCurrentTime] = useState(0)
  const [globalVolume, setGlobalVolume] = useState(0.8)
  const [zoom, setZoom] = useState(1)
  const [trackStates, setTrackStates] = useState(
    STEMS.map(() => ({ muted: false, solo: false, volume: 1 }))
  )

  const audioRefs = useRef({})
  const animFrameRef = useRef(null)

  const allMuted = trackStates.every((t) => t.muted)
  const anySolo = trackStates.some((t) => t.solo)

  useEffect(() => {
    window.__onSeek = (time) => {
      Object.values(audioRefs.current).forEach((audio) => {
        if (audio) audio.currentTime = time
      })
      setCurrentTime(time)
      if (!isPlaying) {
        Object.values(audioRefs.current).forEach((audio) => {
          if (audio) audio.pause()
        })
      }
    }
  }, [isPlaying])

  const loadStemBase64 = useCallback(async (stemIndex) => {
    if (stemPaths[stemIndex] && !stemBase64[stemIndex]) {
      try {
        const result = await invoke('read_stem_as_base64', { path: stemPaths[stemIndex] })
        setStemBase64((prev) => ({ ...prev, [stemIndex]: result.data }))
      } catch (e) {
        console.error(`Failed to load stem ${stemIndex}:`, e)
      }
    }
  }, [stemPaths, stemBase64])

  useEffect(() => {
    if (stemPaths.length > 0) {
      stemPaths.forEach((_, i) => loadStemBase64(i))
    }
  }, [stemPaths, loadStemBase64])

  const updatePlayback = useCallback(() => {
    let earliest = Infinity
    const duration = metadata?.duration_secs || 0

    Object.entries(audioRefs.current).forEach(([idx, audio]) => {
      if (audio && !audio.paused) {
        if (audio.currentTime < earliest) earliest = audio.currentTime
        if (audio.currentTime >= duration) {
          audio.pause()
          audio.currentTime = 0
          setIsPlaying(false)
          setIsStopped(true)
          if (animFrameRef.current) cancelAnimationFrame(animFrameRef.current)
          return
        }
      }
    })

    if (!isPlaying) {
      if (animFrameRef.current) cancelAnimationFrame(animFrameRef.current)
      return
    }

    setCurrentTime(earliest === Infinity ? 0 : earliest)
    animFrameRef.current = requestAnimationFrame(updatePlayback)
  }, [isPlaying, metadata])

  useEffect(() => {
    if (isPlaying) {
      animFrameRef.current = requestAnimationFrame(updatePlayback)
    }
    return () => {
      if (animFrameRef.current) cancelAnimationFrame(animFrameRef.current)
    }
  }, [isPlaying, updatePlayback])

  useEffect(() => {
    if (!isPlaying) {
      Object.values(audioRefs.current).forEach((audio) => {
        if (audio) {
          audio.volume = 0
          audio.pause()
        }
      })
    }
  }, [allMuted, anySolo, isPlaying])

  useEffect(() => {
    Object.entries(audioRefs.current).forEach(([idx, audio]) => {
      if (!audio) return
      const state = trackStates[parseInt(idx)] || { muted: false, solo: false, volume: 1 }
      const trackMuted = state.muted || (anySolo && !state.solo) || allMuted
      const trackVolume = (trackMuted ? 0 : state.volume) * globalVolume
      audio.volume = trackVolume
    })
  }, [trackStates, globalVolume, allMuted, anySolo])

  const playAll = useCallback(() => {
    const duration = metadata?.duration_secs || 0
    Object.entries(audioRefs.current).forEach(([idx, audio]) => {
      const state = trackStates[parseInt(idx)] || { muted: false, solo: false, volume: 1 }
      const trackMuted = state.muted || (anySolo && !state.solo) || allMuted
      if (audio && !trackMuted) {
        audio.currentTime = currentTime
        audio.play().catch(() => {})
      }
    })
    setIsPlaying(true)
    setIsStopped(false)
  }, [metadata, trackStates, anySolo, allMuted, currentTime])

  const pauseAll = useCallback(() => {
    Object.values(audioRefs.current).forEach((audio) => {
      if (audio) audio.pause()
    })
    setIsPlaying(false)
  }, [])

  const stopAll = useCallback(() => {
    Object.values(audioRefs.current).forEach((audio) => {
      if (audio) {
        audio.pause()
        audio.currentTime = 0
      }
    })
    setCurrentTime(0)
    setIsPlaying(false)
    setIsStopped(true)
  }, [])

  const rewind = useCallback(() => {
    stopAll()
  }, [stopAll])

  const handleSelectFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{
          name: 'Audio',
          extensions: ['wav', 'mp3', 'flac', 'ogg', 'oga', 'opus', 'wma', 'aiff', 'aif']
        }]
      })

      if (selected) {
        setError(null)
        setProcessing(true)
        setProgress({ stage: 'Loading', percent: 0, message: 'Analyzing...' })

        const result = await invoke('process_file', { path: selected })
        setMetadata(result)
        setStemPaths(result.paths)
        setCurrentTime(0)
        setIsPlaying(false)
        setIsStopped(true)
        setTrackStates(STEMS.map(() => ({ muted: false, solo: false, volume: 1 })))
        setProcessing(false)
        setProgress({ stage: 'Complete', percent: 100, message: 'Done!' })
      }
    } catch (err) {
      setError(`Error: ${err}`)
      setProcessing(false)
    }
  }

  useEffect(() => {
    let unlisten = null
    listen('demucs_progress', (e) => {
      setProgress((prev) => ({ ...prev, ...e.payload }))
      if (e.payload.stage === 'Complete') {
        setProcessing(false)
      }
    }).then((fn) => { unlisten = fn })
    return () => { if (unlisten) unlisten() }
  }, [])

  const handleVolumeChange = useCallback((index, volume) => {
    setTrackStates((prev) => {
      const next = [...prev]
      next[index] = { ...next[index], volume }
      return next
    })
  }, [])

  const toggleMute = useCallback((index) => {
    setTrackStates((prev) => {
      const next = [...prev]
      next[index] = { ...next[index], muted: !next[index].muted }
      return next
    })
  }, [])

  const toggleSolo = useCallback((index) => {
    setTrackStates((prev) => {
      const next = [...prev]
      next[index] = { ...next[index], solo: !next[index].solo }
      return next
    })
  }, [])

  const handleZoomIn = () => setZoom((z) => Math.min(z * 1.5, 8))
  const handleZoomOut = () => setZoom((z) => Math.max(z / 1.5, 0.5))

  return (
    <div className="min-h-screen bg-[#0a0a0a] text-white flex flex-col">
      <header className="px-4 py-2 border-b border-[#1a1a1a] flex items-center justify-between" style={{ height: '36px' }}>
        <div className="flex items-center gap-2">
          <span className="text-sm">🎵</span>
          <h1 className="text-sm font-semibold">Stem Separator</h1>
          <span className="text-[10px] text-gray-600">v0.1.0</span>
        </div>
        <div className="flex items-center gap-3">
          <span className="text-[10px] text-gray-500">HTDemucs 6-s</span>
          {!metadata && (
            <button
              onClick={handleSelectFile}
              className="text-xs bg-green-600 hover:bg-green-500 px-3 py-1 rounded transition-colors"
            >
              Open Audio File
            </button>
          )}
        </div>
      </header>

      {metadata && (
        <>
          <TransportBar
            isPlaying={isPlaying}
            isStopped={isStopped}
            currentTime={currentTime}
            totalDuration={metadata.duration_secs}
            onPlay={playAll}
            onPause={pauseAll}
            onStop={stopAll}
            onRewind={rewind}
          />

          <div className="flex-1 flex overflow-hidden">
            <div className="flex-1 flex flex-col overflow-hidden">
              <div className="overflow-x-auto">
                <TimeRuler duration={metadata.duration_secs} zoom={zoom} />
                <div className="relative">
                  {STEMS.map((stem, i) => (
                    <TrackRow
                      key={stem.name}
                      name={stem.name}
                      color={stem.color}
                      stemPath={stemBase64[i] || null}
                      volume={trackStates[i]?.volume || 1}
                      muted={trackStates[i]?.muted || false}
                      solo={trackStates[i]?.solo || false}
                      onVolumeChange={(v) => handleVolumeChange(i, v)}
                      onMute={() => toggleMute(i)}
                      onSolo={() => toggleSolo(i)}
                      currentTime={currentTime}
                      duration={metadata.duration_secs}
                      zoom={zoom}
                    />
                  ))}

                  {stemPaths.length === 0 && !processing && (
                    <div className="flex items-center justify-center h-64 text-gray-600">
                      <div className="text-center">
                        <div className="text-2xl mb-2">🎵</div>
                        <p className="text-sm">Open an audio file to get started</p>
                        <button
                          onClick={handleSelectFile}
                          className="mt-3 text-xs bg-green-600 hover:bg-green-500 px-4 py-2 rounded transition-colors"
                        >
                          Select File
                        </button>
                      </div>
                    </div>
                  )}

                  {processing && (
                    <div className="flex items-center justify-center h-64">
                      <div className="text-center">
                        <div className="text-sm font-medium mb-2">{progress.stage}</div>
                        <div className="text-xs text-gray-400 mb-3">{progress.message}</div>
                        <div className="w-48 bg-[#1a1a1a] rounded-full h-2 overflow-hidden mx-auto">
                          <div
                            className="bg-green-600 h-full rounded-full transition-all duration-300"
                            style={{ width: `${progress.percent}%` }}
                          />
                        </div>
                        <div className="text-xs text-gray-500 mt-1">{Math.round(progress.percent)}%</div>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>

            <div className="w-40 border-l border-[#1a1a1a] bg-[#111] flex flex-col">
              <div className="p-3 border-b border-[#1a1a1a]">
                <div className="text-xs font-medium mb-2">Master</div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => setGlobalVolume((v) => Math.max(0, v - 0.1))}
                    className="w-6 h-5 bg-[#333] hover:bg-[#444] rounded text-[10px] flex items-center justify-center"
                  >-</button>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    value={globalVolume}
                    onChange={(e) => setGlobalVolume(parseFloat(e.target.value))}
                    className="flex-1 accent-green-500 h-1"
                  />
                  <button
                    onClick={() => setGlobalVolume((v) => Math.min(1, v + 0.1))}
                    className="w-6 h-5 bg-[#333] hover:bg-[#444] rounded text-[10px] flex items-center justify-center"
                  >+</button>
                </div>
                <div className="text-xs font-mono text-gray-400 mt-1 text-center">{Math.round(globalVolume * 100)}%</div>
              </div>

              <div className="p-3 border-b border-[#1a1a1a]">
                <div className="text-xs font-medium mb-2">Zoom</div>
                <div className="flex gap-1">
                  <button
                    onClick={handleZoomOut}
                    className="flex-1 bg-[#333] hover:bg-[#444] rounded text-[10px] py-1 transition-colors"
                  >-</button>
                  <span className="flex-1 text-center text-xs text-gray-400 py-1">{Math.round(zoom * 100)}%</span>
                  <button
                    onClick={handleZoomIn}
                    className="flex-1 bg-[#333] hover:bg-[#444] rounded text-[10px] py-1 transition-colors"
                  >+</button>
                </div>
              </div>

              <div className="p-3">
                <div className="text-xs font-medium mb-2">Info</div>
                <div className="text-[10px] text-gray-500 space-y-1">
                  <div>Duration: {formatDuration(metadata.duration_secs)}</div>
                  <div>Sample Rate: {metadata.sample_rate} Hz</div>
                  <div>Channels: {metadata.channels === 1 ? 'Mono' : metadata.channels === 2 ? 'Stereo' : metadata.channels}</div>
                  <div>Stems: {stemPaths.length}</div>
                </div>
              </div>
            </div>
          </div>
        </>
      )}

      {!metadata && (
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center max-w-md">
            <div className="text-5xl mb-4">🎵</div>
            <h2 className="text-xl font-semibold mb-2">Stem Separator</h2>
            <p className="text-gray-500 mb-6 text-sm">
              Separate audio into stems using AI. <br />
              Supports WAV, MP3, FLAC, OGG.
            </p>
            <button
              onClick={handleSelectFile}
              className="bg-green-600 hover:bg-green-500 text-white font-semibold px-8 py-3 rounded-xl transition-colors text-sm"
            >
              Open Audio File
            </button>
            <p className="text-gray-600 text-xs mt-4">Powered by HTDemucs (demucs CLI)</p>
          </div>
        </div>
      )}

      {error && (
        <div className="fixed bottom-4 left-4 right-4 bg-red-900/90 border border-red-700 rounded-xl p-3 text-red-200 text-sm">
          {error}
        </div>
      )}

      <footer className="px-4 py-1.5 border-t border-[#1a1a1a] text-center text-[9px] text-gray-600">
        Stem Separator v0.1.0 — HTDemucs 6-stem model (demucs CLI)
      </footer>
    </div>
  )
}

export default App
