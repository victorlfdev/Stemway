import { MorphingSquare } from "@/components/ui/morphing-square"

function LoadingScreen({ stage, message, percent }) {
  return (
    <div className="w-full">
      <div className="bg-[#0f0f0f] rounded-xl border border-[#1a1a1a] p-6">
        <div className="loader-wrapper">
          <MorphingSquare
            message={stage || "Generating"}
            className="bg-white"
          />
        </div>

        {percent > 0 && (
          <div className="w-full h-1.5 bg-[#1a1a1a] rounded-full overflow-hidden mt-4">
            <div
              className="h-full bg-gradient-to-r from-purple-600 to-purple-400 rounded-full transition-all duration-300 ease-out"
              style={{ width: `${percent}%` }}
            />
          </div>
        )}

        {message && (
          <div className="text-xs text-[#888] mt-3 text-center">
            {message}
          </div>
        )}
      </div>
    </div>
  )
}

export default LoadingScreen
