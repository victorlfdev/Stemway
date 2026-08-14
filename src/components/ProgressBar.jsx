function ProgressBar({ stage, percent, message, model }) {
  const displayPercent = Math.min(Math.round(percent), 100)

  const modelLabels = {
    'demucs': '4-Track Standard',
    'bs-roformer': '4-Track Essential',
    'bs-roformer-cpp': '6-Track Premium'
  }

  return (
    <div className="w-full">
      <div className="bg-[#0f0f0f] rounded-xl border border-[#1a1a1a] p-6">
        <div className="flex items-center justify-between mb-1">
          <div className="text-sm font-medium">{stage || 'Processing'}</div>
          <span className="text-[10px] text-[#555] font-medium tracking-wide uppercase">{modelLabels[model] || 'Processing'}</span>
        </div>
        <div className="text-xs text-[#666] mb-4">{message || 'This may take a moment...'}</div>
        <div className="relative">
          <div className="w-full bg-[#1a1a1a] rounded-full h-2.5 overflow-hidden">
            <div
              className="bg-green-600 h-full rounded-full transition-all duration-500 ease-out"
              style={{ width: `${displayPercent}%` }}
            />
          </div>
        </div>
        <div className="text-xs text-[#555] text-right mt-2 font-mono">{displayPercent}%</div>
      </div>
    </div>
  )
}

export default ProgressBar
