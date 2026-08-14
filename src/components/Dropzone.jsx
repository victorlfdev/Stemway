import { useState, useCallback } from 'react'
import { open } from '@tauri-apps/plugin-dialog'

const AUDIO_FILTERS = [{
  name: 'Audio',
  extensions: ['wav', 'mp3', 'flac', 'ogg', 'aiff', 'aif']
}]

function Dropzone({ onFileSelect }) {
  const [dragging, setDragging] = useState(false)

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

  return (
    <div
      className={`border-2 rounded-xl p-8 text-center transition-all cursor-pointer ${
        dragging
          ? 'border-green-500 bg-green-500/5'
          : 'border-[#2a2a2a] bg-[#0f0f0f] hover:border-[#3a3a3a] hover:bg-[#141414]'
      }`}
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onClick={pickFile}
      role="button"
      tabIndex={0}
      aria-label="Drop audio file here or click to browse"
    >
      <div className="flex flex-col items-center gap-3">
        <div className="w-12 h-12 rounded-full flex items-center justify-center bg-[#1a1a1a] border border-[#2a2a2a]">
          <svg className="w-5 h-5 text-[#888]" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m0 0l-4-4m4 4l4-4" />
          </svg>
        </div>
        <div>
          <span className="text-sm font-medium text-white">Drop audio file here</span>
          <span className="text-sm text-[#666]"> or </span>
          <span className="text-sm text-green-500 hover:text-green-400 underline underline-offset-2">browse files</span>
        </div>
        <div className="text-[11px] text-[#555]">WAV, MP3, FLAC, OGG, AIFF</div>
      </div>
    </div>
  )
}

export default Dropzone
