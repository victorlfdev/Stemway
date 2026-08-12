import { useRef, useEffect, useCallback } from 'react'

export default function TimeRuler({ duration, zoom = 1 }) {
  const pixelsPerSecond = 50 * zoom

  const markers = []
  for (let i = 0; i <= Math.ceil(duration); i++) {
    markers.push({
      second: i,
      x: i * pixelsPerSecond,
      isMajor: i % 5 === 0
    })
  }

  return (
    <div
      className="flex items-end border-b border-[#333] bg-[#1a1a1a] text-[10px] font-mono select-none"
      style={{ height: '20px', minWidth: `${duration * pixelsPerSecond}px` }}
    >
      {markers.map((m) => (
        <div
          key={m.second}
          className="flex-shrink-0 flex items-center relative"
          style={{ width: `${pixelsPerSecond}px` }}
        >
          <div
            className={`absolute bottom-0 w-px ${m.isMajor ? 'bg-[#555] h-3' : 'bg-[#333] h-1.5'}`}
          />
          {m.isMajor && (
            <span className="text-gray-500 ml-1" style={{ fontSize: '9px' }}>
              {Math.floor(m.second / 60)}:{(m.second % 60).toString().padStart(2, '0')}
            </span>
          )}
        </div>
      ))}
    </div>
  )
}
