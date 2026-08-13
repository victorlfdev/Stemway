import { useState, useCallback } from 'react'
import { open } from '@tauri-apps/plugin-dialog'

const AUDIO_FILTERS = [{
  name: 'Audio',
  extensions: ['wav', 'mp3', 'flac', 'ogg', 'oga', 'opus', 'wma', 'aiff', 'aif']
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
      className={`border-2 border-dashed rounded-2xl p-12 text-center transition-all cursor-pointer ${
        dragging
          ? 'border-green-500 bg-green-500/10'
          : 'border-[#333] bg-[#1a1a1a] hover:border-[#444]'
      }`}
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onClick={pickFile}
    >
      <div className="flex flex-col items-center gap-3">
        <div className="w-16 h-16 rounded-full bg-green-600/20 flex items-center justify-center">
          <svg className="w-8 h-8 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m0 0l-4-4m4 4l4-4" />
          </svg>
        </div>
        <div className="text-white font-semibold">Drop audio file here</div>
        <div className="text-sm text-gray-500">or click to browse</div>
        <div className="text-xs text-gray-600 mt-2">Supports WAV, MP3, FLAC, OGG</div>
      </div>
    </div>
  )
}

export default Dropzone
