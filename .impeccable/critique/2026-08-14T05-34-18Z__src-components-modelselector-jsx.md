---
timestamp: 2026-08-14T05-34-18Z
slug: src-components-modelselector-jsx
---
---
target: "ModelSelector component — dark radio-group for audio separation model selection"
total_score: 28
max_score: 32
na_heuristics: "9,10"
p0_count: 0
p1_count: 3
---

# Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 4 | Selected state, hover, glow — every interaction confirms visually |
| 2 | Match System / Real World | 3 | Good audio terminology and privacy copy; hardcoded model values weaken it |
| 3 | User Control and Freedom | 3 | Clear selection; no keyboard nav (arrow keys), no deselect |
| 4 | Consistency and Standards | 2 | Missing focus styles (DESIGN.md), wrong contrast values, no min touch targets |
| 5 | Error Prevention | 2 | No null/undefined guards on `models` prop; no empty state handling |
| 6 | Recognition Rather Than Recall | 4 | Cards are self-contained with visible name, description, and meta |
| 7 | Flexibility and Efficiency | 3 | Single selection path; no keyboard shortcuts or accelerators |
| 8 | Aesthetic and Minimalist Design | 4 | Dark studio aesthetic, green glow, clean hierarchy — excellent |
| 9 | Help Users Recognize, Diagnose, and Recover from Error | n/a | No error states in this component |
| 10 | Help and Documentation | n/a | Simple radio group, no help needed |
| | **Total** | **28/32** | **Good** |

# Design Specificity Verdict

**Moderately specific.** The component clearly belongs to Stem Separator — the "Local processing · No uploads" trust signal, the green glow, the BEST badge, and the model-specific copy ("Fastest with GPU") all anchor it to this product. However, the overall structure (section header + card row with name/description/meta) is generic enough that the component could be copy-pasted into any selection UI with minimal changes. The dead `formatDuration` function is actually a stronger signal of copy-paste debt than intentional design.

**Deterministic scan:** The automated detector returned zero findings — the code passes linting and basic static analysis. However, the detector does not check contrast ratios, focus styles, keyboard navigation, or accessibility compliance. Those require visual or manual review.

# Overall Impression

A well-structured, visually polished component that follows the DESIGN.md color and layout tokens faithfully. The dark studio aesthetic works beautifully — the green glow on the selected card is the star. But three P1 issues undermine it: missing keyboard focus styles violate the DESIGN.md accessibility spec, the documented contrast values are factually wrong, and hardcoded model values couple the UI tightly to data internals. Fix these and the component is ship-ready.

# What's Working

1. **Green glow on selected card** — The `shadow-[0_0_20px_rgba(22,163,74,0.1)]` is a great detail. It communicates "this is the powerful one" without adding text or breaking the minimalist aesthetic. Matches DESIGN.md exactly.

2. **Privacy trust signal** — "Local processing · No uploads" placed at the header is smart UX. It addresses the #1 concern for users dealing with AI models before they even make a choice.

3. **ARIA structure** — The radio group uses `role="radiogroup"`, `role="radio"`, `aria-checked`, and `aria-label` correctly. Screen readers will announce the group and each option's state.

# Priority Issues

## [P1] Missing keyboard focus styles
- **Why it matters:** DESIGN.md explicitly states "all interactive elements have visible focus styles via outline utilities." Without focus rings, keyboard-only users cannot see which card is focused. This makes the component unusable for ~15% of users who rely on keyboard navigation.
- **Fix:** Add `focus-visible:outline focus-visible:outline-2 focus-visible:outline-green-600/80 focus-visible:outline-offset-2` to the button's className. Consider adding `focus-visible:ring-2 focus-visible:ring-green-600/50 focus-visible:ring-offset-2 focus-visible:ring-offset-[#0f0f0f]` for a ring instead.
- **Suggested command:** `/impeccable harden`

## [P1] Color contrast fails WCAG AA
- **Why it matters:** DESIGN.md claims `#666` on `#0f0f0f` is 6.6:1 — but the actual contrast ratio is ~3.6:1. The secondary text (#666 on line 12, #555 on lines 13, 39) both fail WCAG AA for normal text (needs 4.5:1) and caption/large-text AA (needs 3:1 for #555). This is a factual documentation error AND an actual accessibility violation.
- **Fix:** Change `#666` to `#888` (ratio ~5.4:1 on #0f0f0f — passes AA for large text, close for normal). Change `#555` to `#777` (ratio ~4.6:1 — passes AA for large text). Update DESIGN.md to reflect correct ratios.
- **Suggested command:** `/impeccable harden`

## [P1] Dead `formatDuration` function
- **Why it matters:** Lines 2–7 define a function that is never called. Dead code signals copy-paste from another component, confuses developers reading the file, and wastes bytes in the bundle (even if minified). It's also defined inside the component body, re-creating the closure on every render.
- **Fix:** Remove the function. If duration formatting is needed elsewhere, hoist it to a shared utils file and import it.
- **Suggested command:** `/impeccable harden`

## [P2] Hardcoded model value check couples UI to data shape
- **Why it matters:** Line 41 uses `m.value === 'bs-roformer-cpp'` to determine the "Fastest with GPU" label. If model values change or new models are added, this logic breaks. The UI should not depend on internal enum strings.
- **Fix:** Move all display text into the `MODELS` data object as properties (e.g., `m.meta`, `m.subtitle`, `m.badge`). The component should render `m.meta` without any conditional logic.
- **Suggested command:** `/impeccable harden`

## [P2] No edge-case handling for null/empty models
- **Why it matters:** If `models` is undefined or null, `models.map()` throws a TypeError and crashes the component. If `models` is an empty array, the user sees only the section header with nothing to interact with and no guidance.
- **Fix:** Add a guard: `if (!models?.length) return null` or render an empty state message. Add PropTypes or TypeScript types to catch this at development time.
- **Suggested command:** `/impeccable harden`

## [P2] BEST badge clips outside card bounds
- **Why it matters:** The badge uses `absolute -top-2 left-3` which places it partially outside the button's visual boundary. Depending on the parent container's overflow handling, it may clip unexpectedly or create inconsistent spacing between cards.
- **Fix:** Use `top-[-8px]` with `overflow-visible` on the button, or add a bottom border-bottom on the header to give the badge breathing room. Alternatively, move the badge inside the card with `absolute top-2 left-3`.
- **Suggested command:** `/impeccable layout`

# Persona Red Flags

**Jordan (First-Timer):** No visible help or tooltips on hover explaining what each model does. The "BEST" badge creates pressure to always pick recommended without explaining when the alternatives are preferable. Would abandon at step 2 if the processing time estimates are confusing.

**Alex (Audio Engineer):** No model file sizes, VRAM requirements, or expected processing time on the cards. An engineer needs to know whether a model fits their DAW session's memory budget, not just quality tiers. Would want keyboard shortcuts to switch models quickly.

**Sam (Accessibility-Dependent User):** Zero focus ring on keyboard navigation. Radio group is theoretically navigable with ARIA but practically impossible without visible focus indicators. Color contrast on secondary text fails AA — Sam using a screen magnifier at 200% would struggle to read the description text.

# Minor Observations

- The section header "Separation model" correctly uses `text-xs text-[#666] font-medium uppercase tracking-wider` per DESIGN.md, but `text-xs` (12px) is at the boundary between body and caption size. Consider `text-[11px]` for tighter integration with the micro typography scale.
- The button is a `<button>` element (not `<input type="radio">`) which is correct for custom styling, but React's synthetic event system won't provide native radio-group keyboard behavior. Arrow keys, Enter, and Space need manual handling if keyboard navigation is added.
- `transition-all` on the button is broad but acceptable here since only border and background color change. For optimization, narrow to `transition-colors duration-200`.
- No animation when the BEST badge appears/disappears. If models are loaded dynamically (e.g., after model download), a fade-in would feel more polished.
- The `key={m.value}` is stable but fragile if model values change between re-renders.

# Questions to Consider

1. The BEST badge and "Recommended for best results" text both communicate the same hierarchy signal. Does the user need both, or does one create visual redundancy?
2. Should selecting an already-selected model deselect it (toggle-off), or lock the choice? The current behavior offers no way to change back once a model is picked.
3. If a user cares more about speed than quality, which model do they pick? The cards emphasize quality hierarchy (BEST) but provide zero performance data (time, VRAM, model size) to make an informed tradeoff decision.
