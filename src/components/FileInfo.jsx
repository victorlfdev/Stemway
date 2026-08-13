function formatDuration(seconds) {
  if (seconds <= 0) return ''
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

function formatModelName(model) {
  switch (model) {
    case 'bs-roformer': return 'BS-RoFormer'
    case 'bs-roformer-cpp': return 'BS-RoFormer.cpp'
    default: return 'HTDemucs'
  }
}

function FileInfo({ fileName, duration, sampleRate, channels, model, onNewFile }) {
  const infoLines = []
  if (duration > 0) infoLines.push(formatDuration(duration))
  infoLines.push(`${sampleRate} Hz`)
  if (channels === 2) infoLines.push('Stereo')
  else if (channels === 1) infoLines.push('Mono')
  else infoLines.push(`${channels}ch`)
  infoLines.push(formatModelName(model))

  return (
    <div className="bg-[#1a1a1a] rounded-xl border border-[#333] p-6 mb-6">
      <div className="flex items-center justify-between mb-1">
        <div className="text-sm font-medium truncate mr-4">{fileName}</div>
        <button
          onClick={onNewFile}
          className="text-xs text-gray-500 hover:text-gray-300 transition-colors shrink-0"
        >
          Change file
        </button>
      </div>
      <div className="text-xs text-gray-500">{infoLines.join(' • ')}</div>
    </div>
  )
}

export default FileInfo
