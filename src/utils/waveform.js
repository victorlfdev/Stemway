export function parseWavHeader(buffer) {
  const view = new DataView(buffer)
  const numChannels = view.getUint16(22, true)
  const sampleRate = view.getUint32(24, true)
  const dataOffset = view.getUint32(20, true)
  const dataSize = view.getUint32(40, true) || (buffer.byteLength - dataOffset)

  const samples = []
  const bytesPerSample = 2
  const channelsPerSample = numChannels
  const totalSamples = Math.min(dataSize / (bytesPerSample * channelsPerSample), 44100 * 60)

  const step = Math.max(1, Math.floor(totalSamples / 200))

  for (let i = 0; i < 200; i++) {
    const idx = Math.floor((i / 200) * totalSamples)
    const pos = dataOffset + idx * bytesPerSample * channelsPerSample
    let sum = 0
    for (let c = 0; c < channelsPerSample; c++) {
      const samplePos = pos + c * bytesPerSample
      if (samplePos + bytesPerSample <= buffer.byteLength) {
        sum += view.getInt16(samplePos, true)
      }
    }
    const avg = channelsPerSample > 0 ? sum / channelsPerSample : 0
    const normalized = avg / 32768
    samples.push(normalized)
  }

  return { samples, numChannels, sampleRate }
}

export function generateWaveformData(base64Data, width, height) {
  const cleanBase64 = base64Data.split(',')[1] || base64Data
  const binary = atob(cleanBase64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }

  const { samples, numChannels, sampleRate } = parseWavHeader(bytes.buffer)

  if (samples.length === 0) {
    return { paths: [], sampleRate, numChannels }
  }

  const paths = []
  const gap = 1 / (samples.length - 1 || 1)

  for (let i = 0; i < samples.length; i++) {
    const x = i * gap
    const y = (1 - samples[i]) / 2
    if (i === 0) {
      paths.push(`M ${x} ${y}`)
    } else {
      paths.push(`L ${x} ${y}`)
    }
  }

  return { paths, sampleRate, numChannels }
}
