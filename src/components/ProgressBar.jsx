import { useState, useEffect } from 'react'

const CYCLES = [
  'Analyzing audio',
  'Processing stems',
  'Separating tracks',
  'Rendering output'
]

const MODEL_LABELS = {
  'demucs': '4-Track Standard',
  'bs-roformer': '4-Track Essential',
  'bs-roformer-cpp': '6-Track Premium'
}

function ProgressBar({ stage, percent, message, model }) {
  const [cycleIndex, setCycleIndex] = useState(0)

  useEffect(() => {
    const interval = setInterval(() => {
      setCycleIndex((prev) => (prev + 1) % CYCLES.length)
    }, 1800)
    return () => clearInterval(interval)
  }, [])

  const displayModel = MODEL_LABELS[model] || 'Processing'

  return (
    <div className="w-full">
      <div className="bg-[#0f0f0f] rounded-xl border border-[#1a1a1a] p-6">
        <div className="flex flex-col items-center justify-center py-2">
          <div className="relative w-16 h-16 mb-6">
            <div
              className="w-16 h-16 rounded-full border-2 border-[#1a1a1a]"
            />
            <div
              className="w-16 h-16 rounded-full border-2 border-transparent animate-spin"
              style={{
                borderColor: '#22c55e transparent #8b5cf6 transparent',
                animationDuration: '1.4s',
                animationTimingFunction: 'linear',
                animationIterationCount: 'infinite',
              }}
            />
            <div className="absolute inset-[6px] rounded-full bg-[#0f0f0f] flex items-center justify-center">
              <div
                className="w-3 h-3 rounded-full bg-green-500"
                style={{
                  boxShadow: '0 0 8px rgba(34,197,94,0.6)',
                  animation: 'pulse 1.4s ease-in-out infinite',
                }}
              />
            </div>
          </div>

          <div className="text-lg font-semibold mb-1 text-center min-h-[1.75rem]">
            <span className="text-white">{CYCLES[cycleIndex]}</span>
          </div>

          <div className="text-sm text-[#666] mb-3">
            {message || 'This may take a moment...'}
          </div>

          <div className="text-[10px] text-[#555] font-medium tracking-wide uppercase">
            {displayModel}
          </div>

          {stage && (
            <div className="text-xs text-[#444] mt-2 font-mono">
              {stage}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default ProgressBar
