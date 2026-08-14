---
target: StemResults component
total_score: 28
max_score: 40
na_heuristics: 
p0_count: 2
p1_count: 1
timestamp: 2026-08-14T05-17-58Z
slug: src-components-stemresults-jsx
---
## Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 4 | Loading indicators, playing glow, progress fill — all clear |
| 2 | Match System / Real World | 4 | Mute/solo metaphors, emoji icons — DAW conventions |
| 3 | User Control and Freedom | 3 | No global stop; no way to stop-all playing stems |
| 4 | Consistency and Standards | 2 | Typography deviates from DESIGN.md (missing tracking-wide, inconsistent label sizes) |
| 5 | Error Prevention | 2 | Silent failures on stem load — no guardrails |
| 6 | Recognition Rather Than Recall | 3 | Color coding reduces recall; emoji icons work but platform-dependent |
| 7 | Flexibility and Efficiency | 2 | No keyboard shortcuts, no play-all/stop-all for power users |
| 8 | Aesthetic and Minimalist Design | 4 | Dark studio aesthetic, tasteful glow, clean cards |
| 9 | Error Recovery | 2 | Silent console.error on load failure — no recovery path for user |
| 10 | Help and Documentation | 3 | Loading indicator present; solo behavior unclear to non-audio users |
| **Total** | | **28/40** | **Acceptable — Significant improvements needed** |

## Design Specificity Verdict

**Purpose-built for stem separation.** The per-stem color-coded progress bars, mute/solo toggles, emoji stem icons, and `STEM_CONFIG` mapping with 6 instrument types are all domain-specific to audio stem separation. This is not interchangeable with a generic audio player.

**Deterministic scan:** The automated detector returned no findings (`[]`). This is expected — the detector focuses on structural/code issues (contrast ratios, markup accessibility), not UX workflow problems.

## Overall Impression

StemResults is a solid, functional component that nails the studio-quality aesthetic. The color-coded stems, mute/solo toggles, and green playing glow are beautiful touches that feel professional. Where it stumbles is in the details that separate a good tool from a great one: zero focus styles, silent failure modes, no keyboard shortcuts, and an emotionally flat completion state. For a desktop audio tool, it needs to feel more responsive and more forgiving.

## What's Working

1. **Per-stem color-coded progress fill** — Each stem card uses the stem-specific color for the seek bar gradient. This is a hallmark of professional DAWs (Pro Tools, Ableton) and extends that metaphor effectively to the web layer.
2. **Mute/solo state indicators** — Red background when muted (`bg-red-600/20`), yellow when solo (`bg-yellow-600/20`). Clear, consistent, and visually distinct without being loud.
3. **Green playing state glow** — The `border-green-600/40 shadow-[0_0_20px_rgba(22,163,74,0.08)]` on the active stem is tasteful studio-quality feedback. It doesn't overpower the dark theme.

## Priority Issues

### [P0] No visible focus styles — accessibility violation
**Why it matters:** DESIGN.md explicitly states "visible focus styles via outline utilities" (line 119). Zero `<button>` or range input in this component has `focus-visible:` styles. Keyboard-only and screen reader users cannot navigate this component.
**Fix:** Add `focus-visible:ring-2 focus-visible:ring-green-500 focus-visible:ring-offset-2 focus-visible:ring-offset-[#0f0f0f]` to all interactive elements.
**Suggested command:** `/impeccable harden`

### [P0] No error state for stem loading failures
**Why it matters:** If `read_stem_as_base4` fails (line 45-47), the user sees only a dead audio card. The "Loading..." text disappears after the attempt, leaving an unplayable, unexplained stem. Silent failures destroy trust.
**Fix:** Add an error state UI after the loading phase: a small error message with a retry button, similar to ErrorBanner but inline within the card.
**Suggested command:** `/impeccable harden`

### [P1] No global stop or keyboard shortcuts
**Why it matters:** Power users (Alex) can have 6 stems playing simultaneously with no escape hatch. Every DAW has spacebar for play/pause and keyboard shortcuts for mute/solo. Without these, this tool feels like a web prototype, not a desktop audio app.
**Fix:** Add a "Stop All" button at the StemResults level. Add keyboard handlers: space for play/pause, M for mute toggle, S for solo toggle.
**Suggested command:** `/impeccable harden`

### [P2] Typography inconsistency with DESIGN.md
**Why it matters:** DESIGN.md specifies micro text as `0.625rem (10px), tracking-wide, uppercase`. Timecodes at line 177 use `text-[10px]` but are missing `tracking-wide`. Stem header label uses `text-xs` instead of `text-sm` from body spec. This erodes design system integrity.
**Fix:** Standardize all micro text with `text-[10px] tracking-wide uppercase` as per DESIGN.md line 25.
**Suggested command:** `/impeccable polish`

### [P2] Race condition in animation frame loop
**Why it matters:** Clicking play 20 times rapidly schedules 20 `requestAnimationFrame` loops, each calling `setCurrentTime`. No cleanup on unmount, no deduplication. This is a memory leak waiting to happen.
**Fix:** Store the animation frame ID in a ref, cancel previous loop before starting new one. Add cleanup in useEffect dependency array.
**Suggested command:** `/impeccable harden`

## Persona Red Flags

**Alex (Power User):** No spacebar to play/pause currently focused stem. No `M` or `S` keyboard shortcuts. No "stop all" button. In a 6-stem session, Alex must use the mouse for every action. This is standard DAW behavior that's missing here.

**Sam (Accessibility-Dependent User):** Zero visible focus indicators. The seek range input has no `role="slider"`, no keyboard handlers, no tab stops. If a stem fails to load, there's no `aria-disabled` state or `aria-live` region notification. Sam assumes the app is broken.

**Riley (Stress Tester):** Clicking play on 6 stems in rapid succession triggers 6 concurrent `invoke('read_stem_as_base4')` calls, each loading a full WAV as base64 into memory. No cancellation, no memory budget. 20 animation frames from rapid clicks = 20 React state updates per click burst.

## Minor Observations

- Emoji icons (🎤, 🥁) render differently across platforms. A Linux user may see different emoji than macOS. Consider SVG fallback.
- `STEM_CONFIG` defines 6 stems but DESIGN.md only specifies 4 colors. `guitar` (#10b981) and `piano` (#06b6d4) use tones that may clash with the #16a34a accent.
- Path splitting with `'/'` on line 229 assumes Unix-style paths. Will break on Windows.
- Duplicate `formatTime` function defined in both `StemPlayer` (line 109) and `StemResults` (line 253). Risk of drift.
- Volume slider uses `onClick` but not `onKeyDown` — no keyboard volume adjustment.
