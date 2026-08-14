import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'

const STEM_ICONS = {
  vocals: '🎤',
  drums: '🥁',
  bass: '🎸',
  guitar: '🎸',
  piano: '🎹',
  other: '🎵',
}

function formatFileSize(bytes) {
  if (bytes === 0) return ''
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

function StemResults({ stemPaths, onOpenFolder, onNewFile }) {
  const [stemSizes, setStemSizes] = useState({})
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (!stemPaths.length) return

    setLoading(true)
    const sizes = {}
    const loadSizes = async () => {
      for (const path of stemPaths) {
        try {
          const stats = await invoke('get_file_size', { path })
          sizes[path] = stats.size
        } catch {
          sizes[path] = 0
        }
      }
      setStemSizes(sizes)
      setLoading(false)
    }
    loadSizes()
  }, [stemPaths])

  const stemNames = stemPaths.map(path => {
    const name = path.split('/').pop().replace('.wav', '')
    const icon = STEM_ICONS[name] || '🎵'
    return { name, path, icon }
  })

  const folder = stemPaths[0]?.split('/').slice(0, -1).join('/') || ''

  return (
    <div className="w-full">
      <div className="mb-6">
        <div className="text-xs text-gray-500 mb-2">Results</div>
        <div className="space-y-1">
          {stemNames.map(({ name, path, icon }) => (
            <div
              key={path}
              className="flex items-center justify-between bg-[#1a1a1a] border border-[#333] px-4 py-3 rounded-lg"
            >
              <div className="flex items-center gap-3">
                <span className="text-sm">{icon}</span>
                <span className="text-sm capitalize">{name}</span>
              </div>
              <span className="text-xs text-gray-500">
                {stemSizes[path] ? formatFileSize(stemSizes[path]) : loading ? '...' : ''}
              </span>
            </div>
          ))}
        </div>
      </div>

      <button
        onClick={() => onOpenFolder(folder)}
        className="w-full bg-[#1a1a1a] hover:bg-[#2a2a2a] border border-[#333] hover:border-[#444] text-gray-300 font-medium px-6 py-3 rounded-xl transition-colors text-sm flex items-center justify-center gap-2"
      >
        <span className="text-base">📂</span>
        <span>Open Output Folder</span>
      </button>
    </div>
  )
}

export default StemResults
