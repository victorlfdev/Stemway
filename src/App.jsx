import { useEffect } from 'react'

function App() {
  useEffect(() => {
    const style = document.createElement('style')
    style.textContent = `
      @tailwind base;
      @tailwind components;
      @tailwind utilities;
      
      * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
      }
      
      body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
        background: #0a0a0a;
        color: #ffffff;
      }
      
      #root {
        min-height: 100vh;
      }
    `
    document.head.appendChild(style)
    return () => document.head.removeChild(style)
  }, [])

  return (
    <div className="min-h-screen bg-[#0a0a0a] text-white">
      <header className="px-6 py-4 border-b border-[#1a1a1a]">
        <h1 className="text-xl font-semibold">🎵 Stem Separator</h1>
      </header>
      
      <main className="max-w-4xl mx-auto py-8 px-6">
        <div className="mb-8">
          <div className="border-2 border-dashed border-[#1a1a1a] rounded-lg p-12 text-center hover:border-[#2a2a2a] transition-colors cursor-pointer">
            <p className="text-[#666] text-lg mb-2">Drag & drop audio here</p>
            <p className="text-[#444] text-sm">or click to browse (WAV, MP3)</p>
          </div>
        </div>
        
        <div className="text-center">
          <button className="bg-[#1db954] hover:bg-[#1ed760] text-black font-bold py-3 px-8 rounded-full text-lg transition-colors">
            🚀 Separate Stems
          </button>
        </div>
      </main>
    </div>
  )
}

export default App
