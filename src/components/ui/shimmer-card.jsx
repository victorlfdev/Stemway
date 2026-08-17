import { useRef, useEffect } from 'react'

function ShimmerCard({
  children,
  className = '',
  shimmerColor = 'rgba(22,163,74,0.08)',
  duration = 2,
}) {
  const ref = useRef(null)
  const shimmerRef = useRef(null)

  useEffect(() => {
    const el = ref.current
    if (!el) return

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting && shimmerRef.current) {
            shimmerRef.current.style.animation = `shimmer-${duration}s linear infinite`
          } else if (shimmerRef.current) {
            shimmerRef.current.style.animation = 'none'
          }
        })
      },
      { threshold: 0.1 }
    )

    observer.observe(el)
    return () => observer.disconnect()
  }, [duration])

  return (
    <div ref={ref} className={`relative overflow-hidden rounded-xl ${className}`}>
      <style>{`
        @keyframes shimmer-${duration} {
          0% { transform: translateX(-100%) rotate(4deg); }
          100% { transform: translateX(200%) rotate(4deg); }
        }
      `}</style>
      <div
        ref={shimmerRef}
        className="absolute inset-0 pointer-events-none"
        style={{
          background: `linear-gradient(90deg, transparent, ${shimmerColor}, transparent)`,
          transform: 'rotate(4deg)',
        }}
      />
      {children}
    </div>
  )
}

export default ShimmerCard
