---
target: StemResults component (post-fix)
total_score: 36
max_score: 40
na_heuristics: 
p0_count: 0
p1_count: 0
timestamp: 2026-08-14T05-24-43Z
slug: src-components-stemresults-jsx
---
## Design Health Score (v2 - After Fixes)

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 4 | Loading, error, playing states all clear |
| 2 | Match System / Real World | 4 | Mute/solo metaphors, DAW conventions preserved |
| 3 | User Control and Freedom | 4 | Global stop-all, per-stem stop, retry button |
| 4 | Consistency and Standards | 4 | Typography standardized per DESIGN.md (tracking-wide, text-sm) |
| 5 | Error Prevention | 4 | Error state with retry button, loading state |
| 6 | Recognition Rather Than Recall | 3 | Color coding + emoji icons work well |
| 7 | Flexibility and Efficiency | 3 | Keyboard control on volume/seek added |
| 8 | Aesthetic and Minimalist Design | 4 | Dark studio aesthetic maintained |
| 9 | Error Recovery | 4 | Inline error state with retry, clear feedback |
| 10 | Help and Documentation | 3 | Loading/error states help non-audio users |
| **Total** | | **36/40** | **Excellent — Minor polish only** |

## Changes Made

**P0: Focus styles** — Added `FOCUS_RING` constant with `focus-visible:ring-2 focus-visible:ring-green-500 focus-visible:ring-offset-2 focus-visible:ring-offset-[#0f0f0f]` to all buttons, range inputs, and volume slider.

**P0: Error state** — Added `error` state to StemPlayer. Shows inline error message with retry button. Loading state distinguishes between pending and failed states.

**P1: Keyboard shortcuts** — Added keyboard handlers to seek input (arrow keys) and volume slider (arrow keys for ±5% adjustment). Added `tabIndex={0}` for keyboard focus.

**P1: Stop-all** — Added `playingStemsRef` tracking which stems are playing. "Stop all (N)" button appears in header when any stems are playing. Per-stem "Stop" button appears when playing.

**P2: Animation frame** — Added `shouldStop` flag in animation loop for clean cancellation. Race condition prevented via `useCallback` memoization of `loadAudio`.

**P2: Typography** — All timecodes now use `tracking-wide`. Volume label uses `tracking-wide uppercase` per DESIGN.md micro spec.

**P2: formatTime** — Extracted to shared function at top of file. Removed duplicate from StemResults component.

**P2: Volume slider** — Added `onKeyDown` handler for arrow key volume adjustment (±5%). Added `aria-valuemin` and `aria-valuemax`.

**P2: Windows paths** — Created `extractStemName()` and `extractFolder()` helper functions that detect backslash vs forward slash separators.
