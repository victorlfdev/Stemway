import { useEffect, useRef, useCallback, useState } from 'react'
import TrackHeader from './TrackHeader'
import WaveformCanvas from './WaveformCanvas'
import PlaybackCursor from './PlaybackCursor'

export default function TrackRow({ name, color, stemBase64, volume, muted, solo, onVolumeChange, onMute, onSolo, currentTime, duration, zoom = 1, onSetAudioRef, audioLoaded = false }) {
  const audioRef = useRef(null)
  const registeredRef = useRef(false)
  const [blobUrl, setBlobUrl] = useState(null)
  const [loading, setLoading] = useState(false)
  const abortRef = useRef(false)

  useEffect(() => {
    if (!stemBase64 || !stemBase64.startsWith('data:')) {
      setBlobUrl(null)
      setLoading(false)
      return
    }

    const dataUriMatch = stemBase64.match(/^data:audio\/(wav|mp3|ogg|flac);base64,/)
    const mimeType = dataUriMatch ? `audio/${dataUriMatch[1]}` : 'audio/wav'
    const base64Data = stemBase64.split(',')[1] || ''
    let currentAbort = false
    abortRef.current = currentAbort

    setLoading(true)
    requestAnimationFrame(() => {
      if (currentAbort || !stemBase64.startsWith('data:')) {
        setLoading(false)
        return
      }
      try {
        const binaryString = atob(base64Data)
        const bytes = Uint8Array.from({ length: binaryString.length }, (_, i) =>
          binaryString.charCodeAt(i)
        )
        const blob = new Blob(bytes, { type: mimeType })
        const url = URL.createObjectURL(blob)
        setBlobUrl(url)
        setLoading(false)
      } catch (err) {
        console.error(`Failed to decode stem base64 for ${name}:`, err)
        setLoading(false)
      }
    })

    return () => {
      currentAbort = true
      abortRef.current = true
    }
  }, [stemBase64, name])

  useEffect(() => {
    if (!blobUrl) return
    if (audioRef.current) {
      audioRef.current.src = blobUrl
    }
  }, [blobUrl])

  useEffect(() => {
    if (audioRef.current && onSetAudioRef && !registeredRef.current) {
      registeredRef.current = true
      onSetAudioRef(audioRef.current)
    }
  }, [onSetAudioRef])

  const handleSeek = useCallback((e) => {
    const canvas = e.target
    if (canvas.tagName !== 'CANVAS') return
    const rect = canvas.getBoundingClientRect()
    const x = e.clientX - rect.left
    const clickedTime = (x / rect.width) * duration
    if (typeof window.__onSeek === 'function') {
      window.__onSeek(clickedTime)
    }
  }, [duration])

  return (
    <div className="flex border-b border-[#2a2a2a] hover:bg-[#222] transition-colors" style={{ minHeight: '64px' }}>
      <TrackHeader
        name={name}
        color={color}
        volume={volume}
        onVolumeChange={onVolumeChange}
        muted={muted}
        solo={solo}
        onMute={onMute}
        onSolo={onSolo}
        channel="L/R"
      />

      <div
        className="relative flex-1 overflow-hidden"
        style={{
          height: '64px',
          minWidth: '800px',
          overflowX: 'auto',
          overflowY: 'hidden',
          backgroundColor: '#1a1a1a'
        }}
      >
        <div style={{ position: 'relative', width: `${duration * 50 * zoom}px`, height: '64px' }}>
          {stemBase64 && (
            <>
              <WaveformCanvas
                src={stemBase64}
                duration={duration}
                color={color}
                onClick={handleSeek}
              />
              <PlaybackCursor currentTime={currentTime} duration={duration} pixelsPerSecond={50 * zoom} />
              {!loading && (
                <audio
                  ref={audioRef}
                  src={blobUrl}
                  preload="auto"
                  style={{ display: 'none' }}
                />
              )}
              {loading && (
                <div className="absolute inset-0 flex items-center justify-center bg-[#1a1a1a] bg-opacity-80">
                  <div className="text-xs text-gray-500">Loading...</div>
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  )
}
