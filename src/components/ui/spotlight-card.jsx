import './spotlight-card.css'

function SpotlightCard({
  children,
  className = '',
  spotlightColor = 'rgba(22,163,74,0.12)',
  borderColor = '#2a2a2a',
  borderColorHover = '#3a3a3a',
  activeBorderColor = '#2a3a2a',
  isSelected = false,
}) {
  const handleMouseMove = (e) => {
    const rect = e.currentTarget.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top
    e.currentTarget.style.setProperty('--mouse-x', `${x}px`)
    e.currentTarget.style.setProperty('--mouse-y', `${y}px`)
    e.currentTarget.style.setProperty('--spotlight-color', spotlightColor)
  }

  const handleMouseEnter = (e) => {
    e.currentTarget.style.setProperty('--border-color', isSelected ? activeBorderColor : borderColorHover)
  }

  const handleMouseLeave = (e) => {
    e.currentTarget.style.setProperty('--border-color', isSelected ? activeBorderColor : borderColor)
  }

  return (
    <div
      className={`spotlight-card ${className}`}
      onMouseMove={handleMouseMove}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {children}
    </div>
  )
}

export default SpotlightCard
