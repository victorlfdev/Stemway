import { useState, useRef } from 'react'

export default function TrackHeader({ name, color, volume, onVolumeChange, muted, solo, onMute, onSolo, channel = 'L/R' }) {
  const volumeRef = useRef(null)

  const handleVolumeDrag = (e) => {
    if (!volumeRef.current) return
    const rect = volumeRef.current.getBoundingClientRect()
    const y = e.clientY - rect.top
    const percent = 1 - (y / rect.height)
    const newVolume = Math.max(0, Math.min(1, percent))
    onVolumeChange(newVolume)
  }

  const handleMouseDown = (e) => {
    e.preventDefault()
    const onMouseMove = (ev) => handleVolumeDrag(ev)
    const onMouseUp = () => {
      document.removeEventListener('mousemove', onMouseMove)
      document.removeEventListener('mouseup', onMouseUp)
    }
    document.addEventListener('mousemove', onMouseMove)
    document.addEventListener('mouseup', onMouseUp)
  }

  return (
    <div className="flex-shrink-0 flex flex-col items-center justify-between py-2" style={{ width: '100px', minWidth: '100px' }}>
      <div className="text-center">
        <div className="text-xs font-medium mb-1" style={{ color }}>{name}</div>
        <div className="text-[9px] text-gray-500 mb-2">{channel}</div>
        <div className="flex gap-1 justify-center">
          <button
            onClick={onMute}
            className={`w-6 h-5 text-[9px] font-bold rounded flex items-center justify-center transition-colors ${
              muted ? 'bg-red-600 text-white' : 'bg-[#333] text-gray-400 hover:bg-[#444]'
            }`}
            title="Mute"
          >
            M
          </button>
          <button
            onClick={onSolo}
            className={`w-6 h-5 text-[9px] font-bold rounded flex items-center justify-center transition-colors ${
              solo ? 'bg-yellow-500 text-black' : 'bg-[#333] text-gray-400 hover:bg-[#444]'
            }`}
            title="Solo"
          >
            S
          </button>
        </div>
      </div>

      <div className="relative h-32 w-4 bg-[#1a1a1a] rounded cursor-pointer" ref={volumeRef} onMouseDown={handleMouseDown}>
        <div
          className="absolute bottom-0 left-0 right-0 rounded-sm transition-all"
          style={{ height: `${volume * 100}%`, backgroundColor: color }}
        />
        <div
          className="absolute left-1/2 w-5 h-1.5 bg-white rounded shadow border border-[#333] -translate-x-1/2 pointer-events-none transition-all"
          style={{ bottom: `calc(${volume * 100}% - 3px)` }}
        />
      </div>

      <div className="text-[9px] font-mono text-gray-400 mt-1">{Math.round(volume * 100)}</div>
    </div>
  )
}
