import { useRef, useEffect, useCallback, useState } from 'react'
import { motion } from 'motion/react'

function BlobCursor({
  color = 'rgba(22,163,74,0.3)',
  size = 24,
  className = '',
}) {
  const containerRef = useRef(null)
  const [mousePos, setMousePos] = useState({ x: -9999, y: -9999 })

  const handleMouseMove = useCallback((e) => {
    if (!containerRef.current) return
    const rect = containerRef.current.getBoundingClientRect()
    setMousePos({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    })
  }, [])

  const handleMouseLeave = useCallback(() => {
    setMousePos({ x: -9999, y: -9999 })
  }, [])

  useEffect(() => {
    const el = containerRef.current
    if (!el) return
    el.addEventListener('mousemove', handleMouseMove)
    el.addEventListener('mouseleave', handleMouseLeave)
    return () => {
      el.removeEventListener('mousemove', handleMouseMove)
      el.removeEventListener('mouseleave', handleMouseLeave)
    }
  }, [handleMouseMove, handleMouseLeave])

  return (
    <div
      ref={containerRef}
      className={`relative overflow-hidden ${className}`}
      style={{ position: 'relative', width: '100%', height: '100%' }}
    >
      <motion.div
        className="absolute rounded-full pointer-events-none"
        style={{
          backgroundColor: color,
          filter: 'blur(4px)',
          width: size,
          height: size,
          opacity: 0.3,
        }}
        animate={{
          x: mousePos.x - size / 2,
          y: mousePos.y - size / 2,
        }}
        transition={{ type: 'spring', stiffness: 400, damping: 25 }}
      />
      <motion.div
        className="absolute rounded-full pointer-events-none"
        style={{
          backgroundColor: color,
          filter: 'blur(8px)',
          width: size * 0.7,
          height: size * 0.7,
          opacity: 0.2,
        }}
        animate={{
          x: mousePos.x - (size * 0.7) / 2,
          y: mousePos.y - (size * 0.7) / 2,
        }}
        transition={{ type: 'spring', stiffness: 300, damping: 18 }}
      />
      <motion.div
        className="absolute rounded-full pointer-events-none"
        style={{
          backgroundColor: color,
          filter: 'blur(12px)',
          width: size * 0.5,
          height: size * 0.5,
          opacity: 0.15,
        }}
        animate={{
          x: mousePos.x - (size * 0.5) / 2,
          y: mousePos.y - (size * 0.5) / 2,
        }}
        transition={{ type: 'spring', stiffness: 200, damping: 15 }}
      />
    </div>
  )
}

export default BlobCursor
