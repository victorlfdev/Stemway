const MODELS = [
  { value: 'demucs', label: 'HTDemucs', info: '4 stems, fast', recommended: true },
  { value: 'bs-roformer', label: 'BS-RoFormer', info: '4 stems, slow', recommended: false },
  { value: 'bs-roformer-cpp', label: 'BS-RoFormer.cpp', info: 'Vocals, fast', recommended: false },
]

function ModelSelector({ selectedModel, onSelect }) {
  return (
    <div className="w-full">
      <div className="text-xs text-gray-500 mb-3 font-medium uppercase tracking-wide">Choose model</div>
      <div className="flex gap-3">
        {MODELS.map((m) => (
          <button
            key={m.value}
            onClick={() => onSelect(m.value)}
            className={`flex-1 text-left px-4 py-4 rounded-xl border transition-all ${
              selectedModel === m.value
                ? 'bg-green-600 border-green-600 shadow-lg shadow-green-600/20'
                : 'bg-[#1a1a1a] border-[#333] hover:border-[#444]'
            }`}
          >
            <div className="font-semibold text-sm mb-1">{m.label}</div>
            <div className={`text-xs mb-2 ${
              selectedModel === m.value ? 'text-white/80' : 'text-gray-500'
            }`}>{m.info}</div>
            {m.recommended && (
              <span className="text-[10px] bg-green-600/20 text-green-400 px-2 py-0.5 rounded-full">
                Recommended
              </span>
            )}
          </button>
        ))}
      </div>
      {MODELS.find(m => m.value === selectedModel)?.recommended === false && (
        <div className="text-xs text-yellow-500 mt-3">
          ⚠️ First run downloads model (~250 MB)
        </div>
      )}
    </div>
  )
}

export default ModelSelector
