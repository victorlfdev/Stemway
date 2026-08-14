function ErrorBanner({ message, onDismiss }) {
  if (!message) return null

  return (
    <div className="fixed bottom-4 left-4 right-4 z-50">
      <div className="bg-red-950/95 border border-red-900/50 rounded-xl px-4 py-3 flex items-center justify-between gap-3 backdrop-blur-sm">
        <span className="text-red-200 text-sm">{message}</span>
        <button
          onClick={onDismiss}
          className="text-red-500 hover:text-white text-xs transition-colors shrink-0 px-2 py-1 rounded hover:bg-red-900/50"
          aria-label="Dismiss error"
        >
          ✕
        </button>
      </div>
    </div>
  )
}

export default ErrorBanner
