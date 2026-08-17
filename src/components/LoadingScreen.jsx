import { MorphingSquare } from "@/components/ui/morphing-square"
import CountUp from "@/components/ui/count-up"

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
          <div>
            <div className="w-full h-1.5 bg-[#1a1a1a] rounded-full overflow-hidden mt-4">
              <div
                className="h-full bg-gradient-to-r from-purple-600 to-purple-400 rounded-full transition-all duration-300 ease-out"
                style={{ width: `${percent}%` }}
              />
            </div>
            <div className="text-sm font-bold text-[#555] font-mono mt-2 text-center">
              <CountUp value={Math.round(percent)} duration={300} />%
            </div>
          </div>
        )}

        {message && (
          <div className="text-sm text-[#888] mt-3 text-center">
            {message}
          </div>
        )}
      </div>
    </div>
  )
}

export default LoadingScreen
