function ModelSelector({ selectedModel, onSelect, models }) {
  const formatDuration = (seconds) => {
    if (!seconds) return ''
    const m = Math.floor(seconds / 60)
    const s = Math.floor(seconds % 60)
    return `${m}:${s.toString().padStart(2, '0')}`
  }

  return (
    <div className="w-full">
      <div className="flex items-center justify-between mb-3">
        <div className="text-xs text-[#666] font-medium uppercase tracking-wider">Separation model</div>
        <div className="text-[10px] text-[#555]">Local processing · No uploads</div>
      </div>
      <div className="flex gap-2" role="radiogroup" aria-label="Choose separation model">
        {models.map((m) => (
          <button
            key={m.value}
            onClick={() => onSelect(m.value)}
            className={`flex-1 text-left px-4 py-4 rounded-xl border transition-all relative ${
              selectedModel === m.value
                ? 'bg-green-600/10 border-green-600/60 shadow-[0_0_20px_rgba(22,163,74,0.1)]'
                : 'bg-[#0f0f0f] border-[#2a2a2a] hover:border-[#3a3a3a]'
            }`}
            role="radio"
            aria-checked={selectedModel === m.value}
            aria-label={`${m.label}: ${m.description}`}
          >
            {m.recommended && (
              <span className="absolute -top-2 left-3 text-[9px] bg-green-600 text-white px-2 py-0.5 rounded-full font-medium tracking-wide">
                BEST
              </span>
            )}
            <div className="font-semibold text-sm mb-1.5">{m.label}</div>
            <div className={`text-xs mb-3 leading-relaxed ${
              selectedModel === m.value ? 'text-green-200/70' : 'text-[#666]'
            }`}>{m.description}</div>
            <div className={`text-[10px] font-medium tracking-wide ${
              selectedModel === m.value ? 'text-green-400' : 'text-[#555]'
            }`}>
              {m.recommended ? 'Recommended for best results' : m.value === 'bs-roformer-cpp' ? 'Fastest with GPU' : 'Good quality alternative'}
            </div>
          </button>
        ))}
      </div>
    </div>
  )
}

export default ModelSelector
