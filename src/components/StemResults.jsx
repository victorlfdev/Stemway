import { useRef, useEffect } from 'react'
import ShimmerCard from './ui/shimmer-card'

const STEM_CONFIG = {
  'vocals': { label: 'Vocals', icon: '🎤', color: '#ef4444' },
  'drums': { label: 'Drums', icon: '🥁', color: '#f59e0b' },
  'bass': { label: 'Bass', icon: '🎸', color: '#3b82f6' },
  'other': { label: 'Other', icon: '🎵', color: '#8b5cf6' },
  'guitar': { label: 'Guitar', icon: '🎸', color: '#10b981' },
  'piano': { label: 'Piano', icon: '🎹', color: '#06b6d4' }
}

function extractStemName(filePath) {
  if (!filePath) return ''
  const separator = filePath.includes('\\') ? '\\' : '/'
  const parts = filePath.split(separator)
  const filename = parts.pop()
  return filename ? filename.replace('.wav', '') : ''
}

function extractFolder(filePath) {
  if (!filePath) return ''
  const separator = filePath.includes('\\') ? '\\' : '/'
  const parts = filePath.split(separator)
  parts.pop()
  return parts.join(separator) || ''
}

function StemResults({ stemPaths, onOpenFolder }) {
  const folder = extractFolder(stemPaths[0] || '')
  const containerRef = useRef(null)

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const cards = entry.target.querySelectorAll('[data-stem-card]')
            cards.forEach((card, i) => {
              card.style.animation = `fadeSlideIn 0.4s ease-out ${i * 0.08}s both`
            })
          }
        })
      },
      { threshold: 0.1 }
    )
    if (containerRef.current) observer.observe(containerRef.current)
    return () => observer.disconnect()
  }, [stemPaths])

  return (
    <div className="w-full max-w-2xl lg:max-w-4xl">
      <div className="flex items-center justify-between mb-3">
        <div>
          <div className="text-[11px] text-[#666] font-semibold uppercase tracking-[0.08em]">Stems</div>
          <div className="text-[11px] text-[#555]">{stemPaths.length} stems ready</div>
        </div>
        <button
          onClick={() => onOpenFolder(folder)}
          className="text-sm text-green-500 hover:text-green-400 transition-colors flex items-center gap-1 px-3 py-1.5 rounded-lg hover:bg-green-600/10 focus-visible:ring-2 focus-visible:ring-green-500 focus-visible:ring-offset-2 focus-visible:ring-offset-[#0f0f0f] focus-visible:rounded-lg"
        >
          <span>📂</span>
          Open folder
        </button>
      </div>

      <div className="bg-[#0f0f0f] rounded-xl border border-[#1a1a1a] p-4">
        <div ref={containerRef} className="flex flex-wrap gap-2">
          {stemPaths.map((path, idx) => {
            const stemName = extractStemName(path)
            const config = STEM_CONFIG[stemName] || STEM_CONFIG['other']
            return (
              <ShimmerCard
                key={path}
                shimmerColor="rgba(22,163,74,0.06)"
                duration={1.5}
                className="flex items-center gap-2 px-3 py-2 rounded-lg border border-[#1a1a1a] bg-[#141414] hover:border-[#2a2a2a] transition-colors"
                data-stem-card
              >
                <span className="text-base">{config.icon}</span>
                <span className="text-sm font-semibold" style={{ color: config.color }}>{config.label}</span>
              </ShimmerCard>
            )
          })}
        </div>
      </div>

      <style>{`
        @keyframes fadeSlideIn {
          from { opacity: 0; transform: translateY(8px); }
          to { opacity: 1; transform: translateY(0); }
        }
      `}</style>
    </div>
  )
}

export default StemResults
