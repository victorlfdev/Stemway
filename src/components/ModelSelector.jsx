import { useRef } from 'react'
import SpotlightCard from './ui/spotlight-card'

const EMPTY_STATE = (
  <div className="text-center py-8">
    <div className="text-sm text-[#888] mb-1">No separation models available</div>
    <div className="text-sm text-[#777]">Models will appear when ready</div>
  </div>
)

function ModelSelector({ selectedModel, onSelect, models }) {
  const buttonRefs = useRef([])

  if (!models?.length) {
    return EMPTY_STATE
  }

  const handleKeyDown = (e) => {
    const { key, target } = e
    const currentIndex = models.findIndex((_, i) => buttonRefs.current[i] === target)

    if (key === 'ArrowRight') {
      e.preventDefault()
      const nextIndex = (currentIndex + 1) % models.length
      buttonRefs.current[nextIndex]?.focus()
      onSelect(models[nextIndex].value)
    } else if (key === 'ArrowLeft') {
      e.preventDefault()
      const prevIndex = (currentIndex - 1 + models.length) % models.length
      buttonRefs.current[prevIndex]?.focus()
      onSelect(models[prevIndex].value)
    } else if (key === 'Home') {
      e.preventDefault()
      buttonRefs.current[0]?.focus()
      onSelect(models[0].value)
    } else if (key === 'End') {
      e.preventDefault()
      buttonRefs.current[models.length - 1]?.focus()
      onSelect(models[models.length - 1].value)
    }
  }

  return (
    <div className="w-full">
      <div className="flex items-center justify-between mb-3">
        <div className="text-[11px] text-[#888] font-semibold uppercase tracking-[0.08em]">Separation model</div>
        <div className="text-[11px] text-[#777] tracking-[0.08em]">Local processing · No uploads</div>
      </div>
      <div className="flex gap-2" role="radiogroup" aria-label="Choose separation model">
        {models.map((m, index) => (
          <SpotlightCard
            key={m.value}
            spotlightColor="rgba(22,163,74,0.12)"
            borderColor={selectedModel === m.value ? '#2a3a2a' : '#2a2a2a'}
            borderColorHover="#3a3a3a"
            isSelected={selectedModel === m.value}
          >
            <button
              ref={(el) => { buttonRefs.current[index] = el }}
              onClick={() => onSelect(m.value)}
              onKeyDown={handleKeyDown}
              className={`flex-1 text-left px-4 py-4 rounded-xl relative w-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-green-600/50 focus-visible:ring-offset-2 focus-visible:ring-offset-[#0f0f0f] ${
                selectedModel === m.value
                  ? 'bg-green-600/10 shadow-[0_0_20px_rgba(22,163,74,0.1)]'
                  : 'bg-[#0f0f0f]'
              }`}
              role="radio"
              aria-checked={selectedModel === m.value}
              aria-label={`${m.label}: ${m.description}`}
            >
              <div className="font-bold text-sm mb-1.5 flex items-center gap-2">
                {m.label}
                {m.recommended && (
                  <span className="text-[10px] bg-green-600 text-white px-1.5 py-0.5 rounded-full font-bold tracking-[0.08em]">
                    BEST
                  </span>
                )}
              </div>
              <div className={`text-sm mb-3 leading-relaxed ${
                selectedModel === m.value ? 'text-green-200/70' : 'text-[#888]'
              }`}>{m.description}</div>
              <div className={`text-[11px] font-semibold tracking-[0.08em] ${
                selectedModel === m.value ? 'text-green-400' : 'text-[#777]'
              }`}>
                {m.meta}
              </div>
            </button>
          </SpotlightCard>
        ))}
      </div>
    </div>
  )
}

export default ModelSelector
