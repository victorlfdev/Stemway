function ProgressBar({ stage, percent, message }) {
  const displayPercent = Math.min(Math.round(percent), 100)

  return (
    <div className="w-full">
      <div className="bg-[#1a1a1a] rounded-xl p-6 border border-[#333]">
        <div className="text-sm font-medium mb-1">{stage || 'Processing'}</div>
        <div className="text-xs text-gray-400 mb-4">{message || 'Please wait...'}</div>
        <div className="w-full bg-[#2a2a2a] rounded-full h-2 overflow-hidden mb-2">
          <div
            className="bg-green-600 h-full rounded-full transition-all duration-500"
            style={{ width: `${displayPercent}%` }}
          />
        </div>
        <div className="text-xs text-gray-500 text-right">{displayPercent}%</div>
      </div>
    </div>
  )
}

export default ProgressBar
