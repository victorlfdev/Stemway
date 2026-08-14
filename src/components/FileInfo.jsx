function formatDuration(seconds) {
  if (seconds <= 0) return null
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

function formatSampleRate(hz) {
  if (hz <= 0) return '0 Hz'
  if (hz >= 1000) {
    return `${hz / 1000} kHz`
  }
  return `${hz} Hz`
}

function formatModelName(model) {
  switch (model) {
    case 'bs-roformer': return '4-Track Essential'
    case 'bs-roformer-cpp': return '6-Track Premium'
    default: return '4-Track Standard'
  }
}

function FileInfo({ fileName, duration, sampleRate, channels, model, backend, onNewFile }) {
  const durationStr = formatDuration(duration)
  const srStr = formatSampleRate(sampleRate)

  const infoParts = []
  if (durationStr) infoParts.push(durationStr)
  infoParts.push(srStr)
  if (channels > 0) {
    if (channels === 2) infoParts.push('Stereo')
    else if (channels === 1) infoParts.push('Mono')
    else infoParts.push(`${channels}ch`)
  } else {
    infoParts.push('--')
  }
  infoParts.push(formatModelName(model))

  if (backend) {
    infoParts.push(backend)
  }

  return (
    <div className="bg-[#0f0f0f] rounded-xl border border-[#1a1a1a] p-5 mb-6">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-[#1a1a1a] flex items-center justify-center">
            <svg className="w-4 h-4 text-[#666]" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 19V6l12-3v13M9 19c0 1.126-2 2-2s-2-0.874-2-2 0.895-2 2-2 2 0.874 2 2zm12-13c0 1.126-2-2-2-2s-2-0.874-2-2 0.895-2 2-2 2 0.874 2 2z" />
            </svg>
          </div>
          <div className="text-sm font-medium truncate">{fileName}</div>
        </div>
        <button
          onClick={onNewFile}
          className="text-xs text-[#aaa] hover:text-white transition-colors px-3 py-1.5 rounded-lg border border-[#2a2a2a] hover:border-[#3a3a3a] hover:bg-[#1a1a1a]"
        >
          Change file
        </button>
      </div>
      <div className="text-xs text-[#888]">{infoParts.join(' · ')}</div>
    </div>
  )
}

export default FileInfo
