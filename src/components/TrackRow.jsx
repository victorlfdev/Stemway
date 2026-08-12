import { useState, useCallback } from 'react'
import TrackHeader from './TrackHeader'
import WaveformCanvas from './WaveformCanvas'
import PlaybackCursor from './PlaybackCursor'

export default function TrackRow({ name, color, stemPath, volume, muted, solo, onVolumeChange, onMute, onSolo, currentTime, duration, zoom = 1 }) {
  const handleCanvasClick = useCallback((e) => {
    if (!e || !e.target) return
    const canvas = e.target
    const rect = canvas.getBoundingClientRect()
    const x = e.clientX - rect.left
    const pixelsPerSecond = 50 * zoom
    const clickedTime = (x / rect.width) * duration
    if (typeof onVolumeChange === 'function') {
      const dummy = { clientX: e.clientX }
      const handleVolumeDrag = (ev) => {
        if (!ev.target) return
        const volRect = ev.target.closest('[ref]')?.parentElement?.getBoundingClientRect()
      }
    }
  }, [zoom, duration])

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
          {stemPath && (
            <WaveformCanvas
              src={stemPath}
              duration={duration}
              color={color}
              onClick={(e) => {
                const rect = e.target.getBoundingClientRect()
                const x = e.clientX - rect.left
                const clickedTime = (x / rect.width) * duration
                if (typeof window.__onSeek === 'function') {
                  window.__onSeek(clickedTime)
                }
              }}
            />
          )}
          <PlaybackCursor currentTime={currentTime} duration={duration} pixelsPerSecond={50 * zoom} />
        </div>
      </div>
    </div>
  )
}
