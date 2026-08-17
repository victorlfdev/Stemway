import { useState, useRef, useEffect } from 'react'

function Ripple({ children, className = '', color = 'rgba(22,163,74,0.3)' }) {
  const [ripples, setRipples] = useState([])
  const containerRef = useRef(null)

  const createRipple = (e) => {
    const rect = containerRef.current?.getBoundingClientRect()
    if (!rect) return

    const x = e.clientX - rect.left
    const y = e.clientY - rect.top
    const size = Math.max(rect.width, rect.height)

    const newRipple = { id: Date.now() + Math.random(), x, y, size }
    setRipples((prev) => [...prev, newRipple])

    setTimeout(() => {
      setRipples((prev) => prev.filter((r) => r.id !== newRipple.id))
    }, 600)
  }

  return (
    <div
      ref={containerRef}
      className={`relative overflow-hidden ${className}`}
      onClick={createRipple}
    >
      <style>{`
        @keyframes ripple-anim {
          0% { transform: scale(0); opacity: 1; }
          100% { transform: scale(4); opacity: 0; }
        }
      `}</style>
      {ripples.map((ripple) => (
        <span
          key={ripple.id}
          className="absolute rounded-full pointer-events-none"
          style={{
            left: ripple.x - ripple.size / 2,
            top: ripple.y - ripple.size / 2,
            width: ripple.size,
            height: ripple.size,
            backgroundColor: color,
            animation: 'ripple-anim 0.6s ease-out forwards',
          }}
        />
      ))}
      {children}
    </div>
  )
}

export default Ripple
