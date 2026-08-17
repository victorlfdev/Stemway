import { useInView, useMotionValue, useSpring } from 'motion/react'
import { useCallback, useEffect, useRef } from 'react'

function CountUp({
  value = 0,
  from = 0,
  direction = 'up',
  delay = 0,
  duration = 2,
  decimals = 0,
  separator = '',
  className = '',
}) {
  const ref = useRef(null)
  const motionValue = useMotionValue(direction === 'down' ? value : from)

  const damping = 20 + 40 * (1 / duration)
  const stiffness = 100 * (1 / duration)

  const springValue = useSpring(motionValue, { damping, stiffness })

  const getDecimalPlaces = (num) => {
    const str = num.toString()
    if (str.includes('.')) {
      const parts = str.split('.')
      if (parts[1] && parseInt(parts[1]) !== 0) {
        return Math.max(parts[1].length, decimals)
      }
    }
    return decimals
  }

  const maxDecimals = Math.max(getDecimalPlaces(from), getDecimalPlaces(value))

  const formatValue = useCallback(
    (latest) => {
      const fixed = decimals > 0
        ? Number(latest).toFixed(decimals)
        : Math.round(latest).toString()
      if (!separator) return fixed
      const parts = fixed.split('.')
      const intPart = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, separator)
      return parts[1] ? `${intPart}.${parts[1]}` : intPart
    },
    [separator, decimals]
  )

  useEffect(() => {
    if (ref.current) {
      ref.current.textContent = formatValue(from)
    }
  }, [from, formatValue])

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      motionValue.set(value)
    }, delay * 1000)
    return () => clearTimeout(timeoutId)
  }, [value, delay, motionValue])

  useEffect(() => {
    const unsubscribe = springValue.on('change', (latest) => {
      if (ref.current) {
        ref.current.textContent = formatValue(latest)
      }
    })
    return () => unsubscribe()
  }, [springValue, formatValue])

  return <span className={className} ref={ref} />
}

export default CountUp
