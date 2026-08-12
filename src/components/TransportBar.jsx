import { formatDuration } from '../App'

export default function TransportBar({ isPlaying, isStopped, currentTime, totalDuration, onPlay, onPause, onStop, onRewind }) {
  const minutes = Math.floor(currentTime / 60)
  const seconds = Math.floor(currentTime % 60)
  const ms = Math.floor((currentTime * 1000) % 1000)
  const timeStr = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}.${ms.toString().padStart(3, '0')}`

  const progress = totalDuration > 0 ? (currentTime / totalDuration) * 100 : 0

  return (
    <div className="flex items-center gap-4 px-4 py-2 border-b border-[#333] bg-[#252525]" style={{ height: '48px' }}>
      <div className="flex items-center gap-2">
        <button
          onClick={onRewind}
          className="w-9 h-7 bg-[#333] hover:bg-[#444] rounded flex items-center justify-center transition-colors"
          title="Rewind"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><rect x="3" y="2" width="3.5" height="12" rx="1" /><rect x="9.5" y="2" width="3.5" height="12" rx="1" /></svg>
        </button>

        <button
          onClick={onPlay}
          className={`w-10 h-8 rounded flex items-center justify-center transition-colors ${
            isPlaying ? 'bg-green-600 hover:bg-green-500' : 'bg-[#333] hover:bg-[#444]'
          }`}
          title={isPlaying ? 'Pause' : 'Play'}
        >
          {isPlaying ? (
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><rect x="3" y="2" width="4" height="12" rx="1" /><rect x="9" y="2" width="4" height="12" rx="1" /></svg>
          ) : (
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><polygon points="4,2 14,8 4,14" /></svg>
          )}
        </button>

        <button
          onClick={onStop}
          className={`w-10 h-8 rounded flex items-center justify-center transition-colors ${
            isStopped ? 'bg-red-700 hover:bg-red-600' : 'bg-[#333] hover:bg-[#444]'
          }`}
          title="Stop"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><rect x="3" y="3" width="10" height="10" rx="1" /></svg>
        </button>
      </div>

      <div className="flex-1 flex items-center gap-3">
        <div className="font-mono text-sm text-green-400 w-24 text-center">{timeStr}</div>

        <div className="flex-1 h-1.5 bg-[#1a1a1a] rounded cursor-pointer relative" title={`${currentTime.toFixed(1)} / ${totalDuration.toFixed(1)}`}>
          <div className="h-full rounded bg-green-600 transition-all" style={{ width: `${progress}%` }} />
        </div>

        <div className="text-xs text-gray-500 font-mono">{formatDuration(totalDuration)}</div>
      </div>
    </div>
  )
}
