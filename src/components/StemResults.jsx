import { useState, useEffect, useRef, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'


function formatFileSize(bytes) {
  if (!bytes || bytes === 0) return ''
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

function formatTime(seconds) {
  if (!seconds) return '0:00'
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${m}:${s.toString().padStart(2, '0')}`
}

function extractStemName(filePath) {
  if (!filePath) return ''
  const separator = filePath.includes('\\') ? '\\' : '/'
  const parts = filePath.split(separator)
  const filename = parts.pop()
  return filename ? filename.replace('.wav', '') : ''
}

function extractFolder(filePath) {
  if (!filePath) return ''
  const separator = filePath.includes('\\') ? '\\' : '/'
  const parts = filePath.split(separator)
  parts.pop()
  return parts.join(separator) || ''
}

const STEM_CONFIG = {
  'vocals': { label: 'Vocals', icon: '🎤', color: '#ef4444' },
  'drums': { label: 'Drums', icon: '🥁', color: '#f59e0b' },
  'bass': { label: 'Bass', icon: '🎸', color: '#3b82f6' },
  'other': { label: 'Other', icon: '🎵', color: '#8b5cf6' },
  'guitar': { label: 'Guitar', icon: '🎸', color: '#10b981' },
  'piano': { label: 'Piano', icon: '🎹', color: '#06b6d4' }
}

const FOCUS_RING = 'focus-visible:ring-2 focus-visible:ring-green-500 focus-visible:ring-offset-2 focus-visible:ring-offset-[#0f0f0f]'

function StemPlayer({ stemName, stemPath, onMuteToggle, onSoloToggle, onPlayToggle, onStop, isCurrentlyMuted, isCurrentlySolo, isCurrentlyPlaying }) {
  const [isPlaying, setIsPlaying] = useState(false)
  const [volume, setVolume] = useState(100)
  const [isMuted, setIsMuted] = useState(false)
  const [isSolo, setIsSolo] = useState(false)
  const [currentTime, setCurrentTime] = useState(0)
  const [duration, setDuration] = useState(0)
  const [audioData, setAudioData] = useState(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(false)
  const audioRef = useRef(null)
  const animFrameRef = useRef(null)
  const loadedRef = useRef(false)

  const config = STEM_CONFIG[stemName] || STEM_CONFIG['other']

  const loadAudio = useCallback(async () => {
    if (loadedRef.current && stemPath) {
      return
    }
    setLoading(true)
    setError(false)
    try {
      const result = await invoke('read_stem_as_base64', { path: stemPath })
      if (result && result.data) {
        setAudioData(result.data)
        loadedRef.current = true
      } else {
        setError(true)
      }
    } catch (err) {
      setError(true)
      console.error('Failed to load stem:', stemPath, err)
    }
    setLoading(false)
  }, [stemPath])

  useEffect(() => {
    loadAudio()
  }, [loadAudio])

  useEffect(() => {
    if (!audioRef.current || !audioData) return

    const audio = audioRef.current
    let shouldStop = false

    const update = () => {
      if (shouldStop) return
      if (!audio.paused) {
        setCurrentTime(audio.currentTime)
      }
      animFrameRef.current = requestAnimationFrame(update)
    }
    animFrameRef.current = requestAnimationFrame(update)

    return () => {
      shouldStop = true
      if (animFrameRef.current) cancelAnimationFrame(animFrameRef.current)
    }
  }, [audioData])

  const togglePlay = useCallback(() => {
    if (!audioRef.current || !audioData) return

    if (isPlaying) {
      audioRef.current.pause()
      setIsPlaying(false)
    } else {
      audioRef.current.src = audioData
      audioRef.current.volume = isMuted ? 0 : volume / 100
      audioRef.current.play()
      setIsPlaying(true)
    }
  }, [isPlaying, audioData, volume, isMuted])

  const stop = useCallback(() => {
    if (!audioRef.current) return
    audioRef.current.pause()
    audioRef.current.currentTime = 0
    setIsPlaying(false)
    setCurrentTime(0)
  }, [])

  const seek = useCallback((e) => {
    if (!audioRef.current || !duration) return
    const rect = e.target.getBoundingClientRect()
    const x = e.clientX - rect.left
    const pct = Math.max(0, Math.min(1, x / rect.width))
    audioRef.current.currentTime = pct * duration
    setCurrentTime(pct * duration)
  }, [duration])

  const handleVolume = useCallback((e) => {
    e.stopPropagation()
    const rect = e.target.getBoundingClientRect()
    const x = e.clientX - rect.left
    const pct = Math.max(0, Math.min(1, x / rect.width))
    setVolume(Math.round(pct * 100))
    if (audioRef.current) {
      audioRef.current.volume = pct
    }
  }, [])

  const handleVolumeKeyDown = useCallback((e) => {
    e.stopPropagation()
    if (e.key === 'ArrowRight' || e.key === 'ArrowUp') {
      e.preventDefault()
      setVolume(prev => {
        const next = Math.min(100, prev + 5)
        if (audioRef.current) audioRef.current.volume = next / 100
        return next
      })
    } else if (e.key === 'ArrowLeft' || e.key === 'ArrowDown') {
      e.preventDefault()
      setVolume(prev => {
        const next = Math.max(0, prev - 5)
        if (audioRef.current) audioRef.current.volume = next / 100
        return next
      })
    }
  }, [])

  const toggleMute = useCallback(() => {
    setIsMuted(prev => {
      const next = !prev
      if (onMuteToggle) onMuteToggle(next)
      return next
    })
  }, [onMuteToggle])

  const toggleSolo = useCallback(() => {
    setIsSolo(prev => {
      const next = !prev
      if (onSoloToggle) onSoloToggle(next)
      return next
    })
  }, [onSoloToggle])

  const retry = useCallback(() => {
    loadedRef.current = false
    loadAudio()
  }, [loadAudio])

  useEffect(() => {
    if (audioRef.current) {
      audioRef.current.volume = isMuted ? 0 : volume / 100
    }
  }, [volume, isMuted])

  const progressPct = duration > 0 ? (currentTime / duration) * 100 : 0

  return (
    <div
      className={`bg-[#0f0f0f] rounded-xl border p-4 transition-all ${
        isPlaying ? 'border-green-600/40 shadow-[0_0_20px_rgba(22,163,74,0.08)]' : 'border-[#1a1a1a]'
      }`}
      role="region"
      aria-label={`${config.label} stem`}
    >
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <span className="text-base" role="img" aria-hidden="true">{config.icon}</span>
          <span className="text-sm font-medium">{config.label}</span>
          {loading && !error && <span className="text-[10px] text-[#555] tracking-wide uppercase">Loading...</span>}
          {error && !loading && (
            <span className="text-[10px] text-red-400 tracking-wide uppercase">Failed to load</span>
          )}
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={toggleMute}
            className={`w-8 h-8 rounded-lg flex items-center justify-center transition-all text-xs ${
              isMuted
                ? 'bg-red-600/20 text-red-400 hover:bg-red-600/30'
                : 'bg-[#1a1a1a] text-[#666] hover:text-white'
            } ${FOCUS_RING} focus-visible:rounded-lg`}
            aria-label={isMuted ? 'Unmute' : 'Mute'}
            title={isMuted ? 'Unmute' : 'Mute'}
          >
            {isMuted ? '🔇' : '🔊'}
          </button>
          <button
            onClick={toggleSolo}
            className={`w-8 h-8 rounded-lg flex items-center justify-center transition-all text-xs ${
              isSolo
                ? 'bg-yellow-600/20 text-yellow-400 hover:bg-yellow-600/30'
                : 'bg-[#1a1a1a] text-[#666] hover:text-white'
            } ${FOCUS_RING} focus-visible:rounded-lg`}
            aria-label={isSolo ? 'Unsolo' : 'Solo'}
            title={isSolo ? 'Unsolo' : 'Solo'}
          >
            S
          </button>
        </div>
      </div>

      <div className="relative">
        <input
          type="range"
          min="0"
          max={duration || 100}
          value={currentTime}
          onChange={seek}
          onKeyDown={(e) => {
            if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
              e.preventDefault()
              const step = duration / 100
              audioRef.current.currentTime = Math.max(0, Math.min(duration, audioRef.current.currentTime + (e.key === 'ArrowRight' ? step : -step)))
            }
          }}
          className={`w-full h-8 appearance-none bg-transparent cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2 [&::-webkit-slider-thumb]:h-2 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:shadow-md [&::-webkit-slider-thumb]:-mt-1.5 ${FOCUS_RING}`}
          style={{
            backgroundImage: `linear-gradient(to right, ${config.color} 0%, ${config.color} ${progressPct}%, transparent ${progressPct}%, transparent 100%)`,
            backgroundSize: '100% 3px',
            backgroundPosition: '0 calc(50% - 1.5px)',
            backgroundRepeat: 'no-repeat',
          }}
          aria-label="Seek"
        />
        <div className="flex items-center justify-between mt-1">
          <span className="text-[10px] text-[#555] font-mono tracking-wide">{formatTime(currentTime)}</span>
          <span className="text-[10px] text-[#555] font-mono tracking-wide">{formatTime(duration)}</span>
        </div>
      </div>

      <div className="flex items-center gap-2 mt-3">
        <button
          onClick={togglePlay}
          disabled={loading || !audioData || error}
          className={`w-10 h-10 rounded-full bg-green-600 hover:bg-green-500 disabled:bg-[#2a2a2a] flex items-center justify-center transition-all shrink-0 ${FOCUS_RING}`}
          aria-label={isPlaying ? 'Pause' : 'Play'}
        >
          {isPlaying ? (
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <rect x="6" y="4" width="4" height="16" rx="1" />
              <rect x="14" y="4" width="4" height="16" rx="1" />
            </svg>
          ) : (
            <svg className="w-4 h-4 ml-0.5" fill="currentColor" viewBox="0 0 24 24">
              <path d="M8 5v14l11-7z" />
            </svg>
          )}
        </button>

        <div className="flex-1 relative">
          <label className="text-[10px] text-[#555] block mb-1 tracking-wide uppercase">Volume</label>
          <div
            className={`h-2 bg-[#1a1a1a] rounded-full relative cursor-pointer ${FOCUS_RING} focus-visible:rounded-full`}
            onClick={handleVolume}
            onKeyDown={handleVolumeKeyDown}
            tabIndex={0}
            role="slider"
            aria-label="Volume"
            aria-valuenow={volume}
            aria-valuemin={0}
            aria-valuemax={100}
          >
            <div
              className="h-full bg-[#666] rounded-full"
              style={{ width: `${volume}%` }}
            />
          </div>
        </div>
      </div>

      <div className="flex items-center gap-2 mt-2">
        {error && !loading && (
          <button
            onClick={retry}
            className={`text-xs text-green-500 hover:text-green-400 transition-colors px-2 py-1 rounded ${FOCUS_RING}`}
          >
            Retry
          </button>
        )}
        {!error && isPlaying && (
          <button
            onClick={stop}
            className={`text-xs text-[#666] hover:text-white transition-colors px-2 py-1 rounded ${FOCUS_RING}`}
          >
            Stop
          </button>
        )}
      </div>

      <audio ref={audioRef} onPause={() => setIsPlaying(false)} onLoadedMetadata={(e) => setDuration(e.currentTarget.duration)} />
    </div>
  )
}

function StemResults({ stemPaths, onOpenFolder, onNewFile }) {
  const stemNames = stemPaths.map(extractStemName)
  const folder = extractFolder(stemPaths[0] || '')
  const playingStemsRef = useRef(new Set())

  const handleMuteToggle = useCallback((stemName, isMuted) => {
    // Solo behavior: when a stem is soloed, mute all others
    if (isMuted) {
      playingStemsRef.current.delete(stemName)
    }
  }, [])

  const handleSoloToggle = useCallback((stemName, isSolo) => {
    if (isSolo) {
      playingStemsRef.current.add(stemName)
    } else {
      playingStemsRef.current.delete(stemName)
    }
  }, [])

  const handlePlayToggle = useCallback((stemName) => {
    if (playingStemsRef.current.has(stemName)) {
      playingStemsRef.current.delete(stemName)
    } else {
      playingStemsRef.current.add(stemName)
    }
  }, [])

  const handleStop = useCallback(() => {
    playingStemsRef.current.clear()
  }, [])

  const handleStopAll = useCallback(() => {
    playingStemsRef.current.clear()
  }, [])

  const playingCount = playingStemsRef.current.size

  return (
    <div className="w-full">
      <div className="flex items-center justify-between mb-3">
        <div>
          <div className="text-xs text-[#666] font-medium uppercase tracking-wider">Stems</div>
          <div className="text-xs text-[#555]">{stemPaths.length} stems ready</div>
        </div>
        <div className="flex items-center gap-2">
          {playingCount > 0 && (
            <button
              onClick={handleStopAll}
              className={`text-xs text-red-400 hover:text-red-300 transition-colors flex items-center gap-1 px-3 py-1.5 rounded-lg hover:bg-red-600/10 ${FOCUS_RING}`}
            >
              <span>⏹</span>
              Stop all ({playingCount})
            </button>
          )}
          <button
            onClick={() => onOpenFolder(folder)}
            className={`text-xs text-green-500 hover:text-green-400 transition-colors flex items-center gap-1 px-3 py-1.5 rounded-lg hover:bg-green-600/10 ${FOCUS_RING}`}
          >
            <span>📂</span>
            Open folder
          </button>
        </div>
      </div>

      <div className="space-y-2">
        {stemNames.map((name, idx) => (
          <StemPlayer
            key={stemPaths[idx]}
            stemName={name}
            stemPath={stemPaths[idx]}
            onMuteToggle={(isMuted) => handleMuteToggle(name, isMuted)}
            onSoloToggle={(isSolo) => handleSoloToggle(name, isSolo)}
            onPlayToggle={() => handlePlayToggle(name)}
            onStop={() => handleStop()}
            isCurrentlyMuted={false}
            isCurrentlySolo={false}
            isCurrentlyPlaying={playingStemsRef.current.has(name)}
            formatDuration={(seconds) => formatTime(seconds)}
          />
        ))}
      </div>
    </div>
  )
}

export default StemResults
