import { useState, useCallback, useRef, useEffect } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import Ripple from './ui/ripple'

const AUDIO_FILTERS = [{
  name: 'Audio',
  extensions: ['wav', 'mp3', 'flac', 'ogg', 'aiff', 'aif']
}]

function Dropzone({ onFileSelect }) {
  const [dragging, setDragging] = useState(false)
  const [blobPos, setBlobPos] = useState({ x: 0, y: 0 })
  const containerRef = useRef(null)

  const pickFile = useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: AUDIO_FILTERS
      })
      if (selected) {
        onFileSelect(selected)
      }
    } catch (err) {
      console.error('Failed to select file:', err)
    }
  }, [onFileSelect])

  const handleDrop = useCallback((e) => {
    e.preventDefault()
    setDragging(false)
    const file = e.dataTransfer.files[0]
    if (file) {
      const path = file.path || `/tmp/dropped-${file.name}`
      onFileSelect(path)
    }
  }, [onFileSelect])

  const handleDragOver = useCallback((e) => {
    e.preventDefault()
    setDragging(true)
  }, [])

  const handleDragLeave = useCallback((e) => {
    e.preventDefault()
    setDragging(false)
  }, [])

  const handleMouseMove = useCallback((e) => {
    if (!containerRef.current) return
    const rect = containerRef.current.getBoundingClientRect()
    setBlobPos({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    })
  }, [])

  return (
    <div
      ref={containerRef}
      className={`border-2 rounded-xl p-8 text-center transition-all cursor-pointer relative ${
        dragging
          ? 'border-green-500 bg-green-500/5'
          : 'border-[#2a2a2a] bg-[#0f0f0f] hover:border-[#3a3a3a] hover:bg-[#141414]'
      }`}
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onClick={pickFile}
      onMouseMove={handleMouseMove}
      role="button"
      tabIndex={0}
      aria-label="Drop audio file here or click to browse"
    >
      <div
        className="pointer-events-none absolute rounded-full transition-opacity"
        style={{
          left: blobPos.x,
          top: blobPos.y,
          width: 24,
          height: 24,
          transform: 'translate(-50%, -50%)',
          backgroundColor: 'rgba(22,163,74,0.3)',
          filter: 'blur(4px)',
          opacity: dragging ? 0.4 : 0.15,
          transition: 'opacity 0.3s ease',
        }}
      />
      <div
        className="pointer-events-none absolute rounded-full transition-opacity"
        style={{
          left: blobPos.x,
          top: blobPos.y,
          width: 14,
          height: 14,
          transform: 'translate(-50%, -50%)',
          backgroundColor: 'rgba(22,163,74,0.2)',
          filter: 'blur(8px)',
          opacity: dragging ? 0.3 : 0.08,
          transition: 'opacity 0.3s ease',
        }}
      />
        <Ripple color="rgba(22,163,74,0.25)">
          <div className="flex flex-col items-center gap-3">
            <div className="w-12 h-12 rounded-full flex items-center justify-center bg-[#1a1a1a] border border-[#2a2a2a]">
              <svg className="w-5 h-5 text-[#888]" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m0 0l-4-4m4 4l4-4" />
              </svg>
            </div>
            <div>
              <span className="text-base font-bold text-white">Drop audio file here</span>
              <span className="text-sm text-[#666]"> or </span>
              <span className="text-base font-semibold text-green-500 hover:text-green-400 underline underline-offset-2">browse files</span>
            </div>
            <div className="text-xs text-[#555]">WAV, MP3, FLAC, OGG, AIFF</div>
          </div>
        </Ripple>
    </div>
  )
}

export default Dropzone
