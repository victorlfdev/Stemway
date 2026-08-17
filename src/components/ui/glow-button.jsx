import { motion } from 'motion/react'

function GlowButton({
  children,
  className = '',
  glowColor = '#16a34a',
  intensity = 0.15,
  onClick,
  ...props
}) {
  const hexToRgb = (hex) => {
    const r = parseInt(hex.slice(1, 3), 16)
    const g = parseInt(hex.slice(3, 5), 16)
    const b = parseInt(hex.slice(5, 7), 16)
    return { r, g, b }
  }

  const { r, g, b } = hexToRgb(glowColor)

  return (
    <motion.button
      className={`relative inline-flex items-center justify-center font-semibold rounded-xl text-white px-8 py-4 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-green-500 focus-visible:ring-offset-2 focus-visible:ring-offset-[#0a0a0a] ${className}`}
      onClick={onClick}
      {...props}
      whileHover={{
        boxShadow: `0 0 30px rgba(${r},${g},${b},${intensity + 0.1}), 0 0 60px rgba(${r},${g},${b},${intensity * 0.5})`,
      }}
      whileTap={{ scale: 0.98 }}
      style={{
        boxShadow: `0 0 20px rgba(${r},${g},${b},${intensity})`,
        transition: 'box-shadow 0.3s ease',
        background: glowColor,
        border: `1px solid ${glowColor}`,
      }}
      animate={{
        boxShadow: [
          `0 0 20px rgba(${r},${g},${b},${intensity})`,
          `0 0 30px rgba(${r},${g},${b},${intensity + 0.05})`,
          `0 0 20px rgba(${r},${g},${b},${intensity})`,
        ],
      }}
      transition={{
        boxShadow: {
          duration: 2,
          repeat: Number.POSITIVE_INFINITY,
          ease: 'easeInOut',
        },
      }}
    >
      {children}
    </motion.button>
  )
}

export default GlowButton
