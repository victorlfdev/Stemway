function ErrorBanner({ message, onDismiss }) {
  if (!message) return null

  return (
    <div className="fixed bottom-4 left-4 right-4 z-50">
      <div className="bg-red-900/90 border border-red-700 rounded-xl p-3 flex items-center justify-between gap-3">
        <span className="text-red-200 text-sm">{message}</span>
        <button
          onClick={onDismiss}
          className="text-red-400 hover:text-red-200 text-xs shrink-0"
        >
          ✕
        </button>
      </div>
    </div>
  )
}

export default ErrorBanner
