import { useRef, useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

const STEM_CONFIG = {
  'vocals': { label: 'Vocals', icon: '🎤', color: '#ef4444' },
  'drums': { label: 'Drums', icon: '🥁', color: '#f59e0b' },
  'bass': { label: 'Bass', icon: '🎸', color: '#3b82f6' },
  'other': { label: 'Other', icon: '🎵', color: '#8b5cf6' },
  'guitar': { label: 'Guitar', icon: '🎸', color: '#10b981' },
  'piano': { label: 'Piano', icon: '🎹', color: '#06b6d4' }
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

function StemResults({ stemPaths, onOpenFolder }) {
  const folder = extractFolder(stemPaths[0] || '')
  const containerRef = useRef(null)
  const [isPlaying, setIsPlaying] = useState(false)
  const [stems, setStems] = useState({})
  const [loaded, setLoaded] = useState(false)
  const [audioError, setAudioError] = useState(null)

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const cards = entry.target.querySelectorAll('[data-stem-card]')
            cards.forEach((card, i) => {
              card.style.animation = `fadeSlideIn 0.4s ease-out ${i * 0.08}s both`
            })
          }
        })
      },
      { threshold: 0.1 }
    )
    if (containerRef.current) observer.observe(containerRef.current)
    return () => observer.disconnect()
  }, [stemPaths])

  useEffect(() => {
    let disposed = false
    setLoaded(false)
    setIsPlaying(false)
    setAudioError(null)
    if (stemPaths.length === 0) return

    const stemMap = stemPaths.reduce((acc, path) => {
      const name = extractStemName(path)
      const id = name.replace(/\s+/g, '_').toLowerCase()
      acc[id] = path
      return acc
    }, {})

    invoke('audio_load_stems', { stemPaths: Object.entries(stemMap) })
      .then(() => {
        if (disposed) return
        const initial = {}
        stemPaths.forEach((path) => {
          const name = extractStemName(path)
          const id = name.replace(/\s+/g, '_').toLowerCase()
          initial[id] = { muted: false, soloed: false, volume: 1.0 }
        })
        setStems(initial)
        setLoaded(true)
        console.log(`[Audio] Loaded ${stemPaths.length} stems:`, stemPaths)
        console.log('[Audio] Initial stem state:', initial)
      })
      .catch((err) => {
        if (!disposed) setAudioError(`Failed to load audio: ${err}`)
      })
    return () => {
      disposed = true
      invoke('audio_load_stems', { stemPaths: [] }).catch(() => {})
    }
  }, [stemPaths])

  useEffect(() => {
    if (!loaded || !isPlaying) return
    const timer = setInterval(() => {
      invoke('audio_get_playback_state').then(setIsPlaying)
        .catch((err) => setAudioError(String(err)))
    }, 500)
    return () => clearInterval(timer)
  }, [loaded, isPlaying])

  const handlePlayPause = () => {
    if (!loaded) return
    const newPlaying = !isPlaying
    console.log(`[Audio] Toggle playback → ${newPlaying ? 'PLAY' : 'PAUSE'}`)
    invoke('audio_toggle_playback')
      .then(() => setIsPlaying(newPlaying))
      .catch((err) => {
        setAudioError(`Failed to play audio: ${err}`)
      })
  }

  const handleVolumeChange = (id, value) => {
    if (!loaded || !stems[id]) return
    const v = parseFloat(value)
    const current = stems[id]
    setStems(prev => ({ ...prev, [id]: { ...current, volume: v } }))
    invoke('audio_set_volume', { id, volume: v })
      .catch((err) => setAudioError(`Failed to set volume: ${err}`))
  }

  const handleMuteToggle = (id) => {
    if (!loaded || !stems[id]) return
    const current = stems[id]
    const muted = !current.muted
    console.log(`[Audio] Mute ${id} → ${muted}`)
    setStems(prev => ({ ...prev, [id]: { ...current, muted } }))
    invoke('audio_set_mute', { id, muted })
      .catch((err) => setAudioError(`Failed to set mute: ${err}`))
  }

  const handleSoloToggle = (id) => {
    if (!loaded || !stems[id]) return
    const current = stems[id]
    const solo = !current.soloed
    const updated = { ...stems, [id]: { ...current, soloed: solo } }
    console.log(`[Audio] Solo ${id} → ${solo}`)
    setStems(updated)
    invoke('audio_set_solo', { id, solo })
      .catch((err) => setAudioError(`Failed to set solo: ${err}`))
  }

  const stemKeys = stemPaths.map((path) => {
    const name = extractStemName(path)
    const id = name.replace(/\s+/g, '_').toLowerCase()
    return { name, id }
  })

  return (
    <div className="w-full max-w-2xl lg:max-w-4xl">
      <div className="flex items-center justify-between mb-3">
        <div>
          <div className="text-[11px] text-[#666] font-semibold uppercase tracking-[0.08em]">Stems</div>
          <div className="text-[11px] text-[#555]">{stemPaths.length} stems ready</div>
        </div>
        <button
          onClick={() => onOpenFolder(folder)}
          className="text-sm text-green-500 hover:text-green-400 transition-colors flex items-center gap-1 px-3 py-1.5 rounded-lg hover:bg-green-600/10 focus-visible:ring-2 focus-visible:ring-green-500 focus-visible:ring-offset-2 focus-visible:ring-offset-[#0f0f0f] focus-visible:rounded-lg"
        >
          <span>📂</span>
          Open folder
        </button>
      </div>

      <div className="bg-[#0f0f0f] rounded-xl border border-[#1a1a1a] p-4">
        {audioError && <p role="alert" className="text-sm text-red-400 mb-3">{audioError}</p>}
        <div className="flex items-center justify-between mb-3 pb-3 border-b border-[#1a1a1a]">
          <div className="text-sm font-semibold">Stem Mixer</div>
          <button
            onClick={handlePlayPause}
            disabled={!loaded}
            className="px-4 py-1.5 text-sm font-semibold rounded-lg bg-green-600 hover:bg-green-500 transition-colors"
          >
            {!loaded ? 'Loading audio…' : isPlaying ? '⏸ Pause' : '▶ Play All'}
          </button>
        </div>

        <div ref={containerRef} className="flex flex-col gap-2">
          {stemKeys.map(({ name, id }) => {
            const config = STEM_CONFIG[name] || STEM_CONFIG['other']
            const stem = stems[id] || { muted: false, soloed: false, volume: 1.0 }
            return (
              <div
                key={id}
                className="flex items-center gap-3 px-3 py-2 rounded-lg border border-[#1a1a1a] bg-[#141414] hover:border-[#2a2a2a] transition-colors"
                data-stem-card
              >
                <span className="text-base w-6 text-center">{config.icon}</span>
                <span className="text-sm font-medium w-20 truncate" style={{ color: config.color }}>{config.label}</span>

                <div className="flex items-center gap-2 flex-1">
                  <span className="text-xs text-[#666] w-6">Vol</span>
                  <input
                    type="range"
                    disabled={!loaded}
                    min="0"
                    max="1"
                    step="0.05"
                    value={stem.volume}
                    onChange={(e) => handleVolumeChange(id, e.target.value)}
                    className="flex-1 h-1 accent-green-500 cursor-pointer"
                  />
                </div>

                <div className="flex gap-1">
                  <button
                    onClick={() => handleMuteToggle(id)}
                    disabled={!loaded}
                    className={`px-2 py-1 text-xs font-bold rounded transition ${
                      stem.muted
                        ? 'bg-red-600 text-white'
                        : 'bg-[#1a1a1a] text-[#888] hover:bg-[#2a2a2a]'
                    }`}
                  >
                    M
                  </button>
                  <button
                    onClick={() => handleSoloToggle(id)}
                    disabled={!loaded}
                    className={`px-2 py-1 text-xs font-bold rounded transition ${
                      stem.soloed
                        ? 'bg-yellow-500 text-black'
                        : 'bg-[#1a1a1a] text-[#888] hover:bg-[#2a2a2a]'
                    }`}
                  >
                    S
                  </button>
                </div>
              </div>
            )
          })}
        </div>
      </div>

      <style>{`
        @keyframes fadeSlideIn {
          from { opacity: 0; transform: translateY(8px); }
          to { opacity: 1; transform: translateY(0); }
        }
      `}</style>
    </div>
  )
}

export default StemResults
