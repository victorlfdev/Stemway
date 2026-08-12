import { useEffect, useRef, useCallback, useState } from 'react'

export default function WaveformCanvas({ src, duration, zoom = 1, color = '#4ade80', onClick }) {
  const canvasRef = useRef(null)
  const [paths, setPaths] = useState([])
  const [isLoaded, setIsLoaded] = useState(false)

  useEffect(() => {
    if (!src) return

    const cleanBase64 = src.split(',')[1] || src
    try {
      const binary = atob(cleanBase64)
      const bytes = new Uint8Array(binary.length)
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i)
      }

      const view = new DataView(bytes.buffer)
      const numChannels = view.getUint16(22, true)
      const sampleRate = view.getUint32(24, true)
      let dataOffset = view.getUint32(20, true)
      let dataSize = view.getUint32(40, true) || (bytes.byteLength - dataOffset)

      const samples = []
      const bytesPerSample = 2
      const channelsPerSample = numChannels
      const totalSamples = Math.min(dataSize / (bytesPerSample * channelsPerSample), sampleRate * 60)
      const step = Math.max(1, Math.floor(totalSamples / 400))

      for (let i = 0; i < 400; i++) {
        const idx = Math.floor((i / 400) * totalSamples)
        const pos = dataOffset + idx * bytesPerSample * channelsPerSample
        let sum = 0
        for (let c = 0; c < channelsPerSample; c++) {
          const samplePos = pos + c * bytesPerSample
          if (samplePos + bytesPerSample <= bytes.byteLength) {
            sum += view.getInt16(samplePos, true)
          }
        }
        const avg = channelsPerSample > 0 ? sum / channelsPerSample : 0
        const normalized = avg / 32768
        samples.push(normalized)
      }

      setPaths(samples)
      setIsLoaded(true)
    } catch (e) {
      console.error('Failed to parse WAV:', e)
    }
  }, [src])

  const drawWaveform = useCallback(() => {
    const canvas = canvasRef.current
    if (!canvas || canvas.tagName !== 'CANVAS' || !paths.length) return

    if (!(canvas instanceof HTMLCanvasElement)) return
    const ctx = canvas.getContext('2d')
    const dpr = window.devicePixelRatio || 1
    const rect = canvas.getBoundingClientRect()

    canvas.width = rect.width * dpr
    canvas.height = rect.height * dpr
    ctx.scale(dpr, dpr)
    ctx.clearRect(0, 0, rect.width, rect.height)

    const w = rect.width
    const h = rect.height
    const step = Math.max(1, Math.floor(400 / w))
    const mid = h / 2

    // Draw filled waveform
    ctx.beginPath()
    ctx.moveTo(0, mid)

    for (let i = 0; i < Math.min(paths.length, 400); i++) {
      const x = (i / 400) * w
      const y = mid - paths[i] * mid * 0.8
      ctx.lineTo(x, y)
    }

    ctx.lineTo(w, mid)
    ctx.lineTo(0, mid)
    ctx.fillStyle = color + '33'
    ctx.fill()

    // Draw center line
    ctx.beginPath()
    ctx.moveTo(0, mid)
    ctx.lineTo(w, mid)
    ctx.strokeStyle = color + '66'
    ctx.lineWidth = 0.5
    ctx.stroke()

    // Draw waveform outline
    ctx.beginPath()
    ctx.moveTo(0, mid)

    for (let i = 0; i < Math.min(paths.length, 400); i++) {
      const x = (i / 400) * w
      const y = mid - paths[i] * mid * 0.8
      ctx.lineTo(x, y)
    }

    ctx.strokeStyle = color + 'aa'
    ctx.lineWidth = 1
    ctx.stroke()
  }, [paths, color])

  useEffect(() => {
    drawWaveform()
  }, [drawWaveform])

  if (!isLoaded) {
    return (
      <div
        ref={canvasRef}
        className="w-full"
        style={{ height: '60px', backgroundColor: '#222', borderRadius: '4px' }}
      />
    )
  }

  return (
    <canvas
      ref={canvasRef}
      className="w-full rounded"
      style={{ height: '60px', cursor: 'pointer', backgroundColor: '#222' }}
      onClick={onClick}
    />
  )
}
