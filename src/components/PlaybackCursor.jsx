export default function PlaybackCursor({ currentTime, duration, pixelsPerSecond = 50 }) {
  const x = (currentTime / 1) * pixelsPerSecond
  const width = Math.max(duration * pixelsPerSecond, 1)

  if (x > width) return null

  return (
    <div
      className="absolute top-0 bottom-0 z-10 pointer-events-none"
      style={{ left: `${x}px`, width: '2px' }}
    >
      <div className="w-2 h-2 bg-green-400 rotate-45 transform -translate-x-1 -translate-y-1 rounded-sm shadow" />
      <div
        className="absolute top-2 left-0 w-px bg-green-400"
        style={{ height: '100%' }}
      />
    </div>
  )
}
