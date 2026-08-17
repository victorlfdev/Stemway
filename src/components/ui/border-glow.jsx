import { useRef, useCallback, useEffect } from 'react'
import './border-glow.css'

function parseHSL(hslStr) {
  const match = hslStr.match(/([\d.]+)\s*([\d.]+)%?\s*([\d.]+)%?/)
  if (!match) return { h: 142, s: 75, l: 39 }
  return { h: parseFloat(match[1]), s: parseFloat(match[2]), l: parseFloat(match[3]) }
}

function buildGlowVars(glowColor, intensity) {
  const { h, s, l } = parseHSL(glowColor)
  const base = `${h}deg ${s}% ${l}%`
  const opacities = [100, 60, 50, 40, 30, 20, 10]
  const keys = ['', '-60', '-50', '-40', '-30', '-20', '-10']
  const vars = {}
  for (let i = 0; i < opacities.length; i++) {
    vars[`--glow-color${keys[i]}`] = `hsl(${base} / ${Math.min(opacities[i] * intensity, 100)}%)`
  }
  return vars
}

const GRADIENT_POSITIONS = ['80% 55%', '69% 34%', '8% 6%', '41% 38%', '86% 85%', '82% 18%', '51% 4%']
const GRADIENT_KEYS = ['--gradient-one', '--gradient-two', '--gradient-three', '--gradient-four', '--gradient-five', '--gradient-six', '--gradient-seven']
const COLOR_MAP = [0, 1, 2, 0, 1, 2, 1]

function buildGradientVars(colors) {
  const vars = {}
  for (let i = 0; i < 7; i++) {
    const c = colors[Math.min(COLOR_MAP[i], colors.length - 1)]
    vars[GRADIENT_KEYS[i]] = `radial-gradient(at ${GRADIENT_POSITIONS[i]}, ${c} 0px, transparent 50%)`
  }
  vars['--gradient-base'] = `linear-gradient(${colors[0]} 0 100%)`
  return vars
}

function BorderGlow({
  children,
  className = '',
  edgeSensitivity = 30,
  glowColor = '142 75 39',
  backgroundColor = '#0f0f0f',
  borderRadius = 16,
  glowRadius = 40,
  glowIntensity = 1.0,
  coneSpread = 25,
  colors = ['#22c55e', '#16a34a', '#15803d'],
  fillOpacity = 0.5,
}) {
  const cardRef = useRef(null)

  const getCenterOfElement = useCallback((el) => {
    const { width, height } = el.getBoundingClientRect()
    return [width / 2, height / 2]
  }, [])

  const getEdgeProximity = useCallback((el, x, y) => {
    const [cx, cy] = getCenterOfElement(el)
    const dx = x - cx
    const dy = y - cy
    let kx = Infinity
    let ky = Infinity
    if (dx !== 0) kx = cx / Math.abs(dx)
    if (dy !== 0) ky = cy / Math.abs(dy)
    const edge = Math.min(Math.max(1 / Math.min(kx, ky), 0), 1)
    return edge
  }, [getCenterOfElement])

  const getCursorAngle = useCallback((el, x, y) => {
    const [cx, cy] = getCenterOfElement(el)
    const dx = x - cx
    const dy = y - cy
    if (dx === 0 && dy === 0) return 0
    const radians = Math.atan2(dy, dx)
    let degrees = radians * (180 / Math.PI) + 90
    if (degrees < 0) degrees += 360
    return degrees
  }, [getCenterOfElement])

  const handlePointerMove = useCallback((e) => {
    const card = cardRef.current
    if (!card) return
    const rect = card.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top
    const edge = getEdgeProximity(card, x, y)
    const angle = getCursorAngle(card, x, y)
    card.style.setProperty('--edge-proximity', `${(edge * 100).toFixed(3)}`)
    card.style.setProperty('--cursor-angle', `${angle.toFixed(3)}deg`)
  }, [getEdgeProximity, getCursorAngle])

  const glowVars = buildGlowVars(glowColor, glowIntensity)

  return (
    <div
      ref={cardRef}
      onPointerMove={handlePointerMove}
      className={`border-glow-card ${className}`}
      style={{
        '--card-bg': backgroundColor,
        '--edge-sensitivity': edgeSensitivity,
        '--border-radius': `${borderRadius}px`,
        '--glow-padding': `${glowRadius}px`,
        '--cone-spread': coneSpread,
        '--fill-opacity': fillOpacity,
        ...glowVars,
        ...buildGradientVars(colors),
      }}
    >
      <span className="edge-light" />
      <div className="border-glow-inner">
        {children}
      </div>
    </div>
  )
}

export default BorderGlow
