# Stemway — Design System

<!-- impeccable:design-schema 1 -->

## Visual World

Dark, studio-quality interface for a local-first AI audio separation tool. The aesthetic draws from professional audio software and Moises.ai: near-black backgrounds, green accent, clean separation of workflow stages. The visual language communicates precision, reliability, and technical competence without feeling clinical.

## Palette

- **Background:** `#0a0a0a` (near-black, the canvas)
- **Surface:** `#0f0f0f` (elevated cards, slightly lighter)
- **Border:** `#1a1a1a` (subtle separators), `#2a2a2a` (interactive borders)
- **Text primary:** `#ffffff` (body, headings)
- **Text secondary:** `#888` (descriptions), `#888` (labels, muted), `#777` (captions)
- **Accent:** `#16a34a` green — primary actions, active states
- **Stem colors:** `#ef4444` (vocals), `#f59e0b` (drums), `#3b82f6` (bass), `#8b5cf6` (other)

## Typography

- **Font family:** Inter (Google Fonts, SIL Open Font License) — loaded via `@import` in `index.css`
- **Heading:** 1rem (16px), bold (700), `tracking-tight`
- **Body:** 0.875rem (14px), medium weight (500)
- **Labels:** 0.75rem (12px), semibold (600), `tracking-[0.08em]`, uppercase
- **Caption:** 0.6875rem (11px), regular (400), `tracking-[0.08em]`
- **Micro:** 0.625rem (10px), bold (700), `tracking-[0.08em]`
- **Monospace:** Used for timecodes and percentage values

## Components

### App Shell

- Full-height dark flex column
- Header: `px-6 py-4`, bottom border `#1a1a1a`, app logo + title on left, "New file" button on right
- Main: centered max-width 2xl, `px-6 py-8`
  - Footer: `px-6 py-3`, top border `#1a1a1a`, two-column text at `text-[10px]`

### Dropzone

- Rounded-xl, `border-2 border-[#2a2a2a]` (default), `border-green-500 bg-green-500/5` (dragging)
- Background: `#0f0f0f` (default), `hover:bg-[#141414]` (hover)
- Icon circle: `w-12 h-12`, rounded-full, `bg-[#1a1a1a] border-[#2a2a2a]`
- Text: `text-base font-bold` (action label), `text-xs` (supported formats)
- Click target: full area, `cursor-pointer`, `role="button"`

### ModelSelector

- Radio-group layout with three model cards
- Selected card: `bg-green-600/10 border-green-600/60`, subtle green glow `shadow-[0_0_20px_rgba(22,163,74,0.1)]`
- Unselected: `bg-[#0f0f0f] border-[#2a2a2a]`, hover `border-[#3a3a3a]`
- Focus ring: `focus-visible:ring-2 focus-visible:ring-green-600/50 focus-visible:ring-offset-2 focus-visible:ring-offset-[#0f0f0f]`
- "BEST" badge: inline with model title, `text-[10px] font-bold bg-green-600 text-white px-1.5 py-0.5 rounded-full tracking-[0.08em]`, in a `flex items-center gap-2` container
- Header: `text-[11px] text-[#888] font-semibold uppercase tracking-[0.08em]`
- Description: `text-sm leading-relaxed`, secondary text `text-[#888]` (selected: `text-green-200/70`)
- Meta text: `text-[11px] font-semibold tracking-[0.08em]`, `text-[#777]` (selected: `text-green-400`)
- Keyboard navigation: ArrowLeft/ArrowRight moves focus and selects; Home/End jumps to first/last; full keyboard accessibility
- Empty state: displays "No separation models available" message instead of hiding the section

### FileInfo

- Background: `#0f0f0f`, border `#1a1a1a`, rounded-xl, `p-5`
- File icon: `w-8 h-8 rounded-lg bg-[#1a1a1a]` with SVG icon
- File name: `text-sm font-bold truncate`
- Metadata line: `text-[11px] text-[#888]`, separator `·`
- "Change file" button: `text-sm px-3 py-1.5 rounded-lg hover:bg-[#1a1a1a]`

### ProgressBar

- Background: `#0f0f0f`, border `#1a1a1a`, rounded-xl, `p-6`
- Stage label: `text-xl font-bold`
- Progress track: `h-2.5 bg-[#1a1a1a] rounded-full`
- Progress fill: `bg-green-600 rounded-full transition-all duration-500 ease-out`
- Percentage: `text-sm font-bold text-[#555] font-mono`
- Model label: `text-[11px] text-[#555] font-semibold tracking-[0.08em] uppercase`

### StemResults

- Header row: "Stems" label (`text-[11px] text-[#666] font-semibold uppercase tracking-[0.08em]`) + "Open folder" link (`text-sm`)
- Each stem card: `bg-[#0f0f0f] rounded-xl border border-[#1a1a1a] p-4`
- Active stem (playing): `border-green-600/40 shadow-[0_0_20px_rgba(22,163,74,0.08)]`
- Stem header: icon + label on left, mute/solo toggles on right
- Mute toggle: red when muted, gray otherwise
- Solo toggle: yellow when active
- Progress bar: custom range input with stem-color fill, `h-2`
- Controls: play/pause button (green circle, `w-10 h-10`), volume slider
- Timecodes: `text-[10px] text-[#555] font-mono`
- Stem name: `text-sm font-semibold`

### ErrorBanner

- Position: `fixed bottom-4 left-4 right-4 z-50`
- Background: `bg-red-950/95` with `backdrop-blur-sm`
- Border: `border-red-900/50`
- Text: `text-red-200 text-sm`
- Dismiss button: `text-red-500 hover:text-white text-sm`

## States

### Default
- Dark, clean, all elements at rest
- Hover states on interactive elements: subtle border brightening, background shifts

### Active / Hover
- Dropzone: green border and tint when dragging
- Model cards: border brightens to `#3a3a3a`
- Buttons: background darkens slightly, text brightens

### Loading / Processing
- Spinner: green border, animate-spin
- Progress bar: smooth transition `duration-500 ease-out`
- Percentage counter: monospace, right-aligned

### Error
- Non-blocking banner at bottom, red background with blur backdrop
- Dismissable with button on right

## Responsive

- Desktop-first layout (`max-w-2xl` centered)
- Works at `1024px+` window size
- Scales down to ~`768px` with comfortable padding
- Model selector cards wrap gracefully on smaller screens
- Model selector returns `null` if no models available (defensive guard)

## Accessibility

- Keyboard focus: all interactive elements have visible focus styles via outline utilities
- ARIA: radio groups for model selector, sliders for volume/seek, regions for stem players
- Contrast: body text `#ffffff` on `#0a0a0a` (21:1), secondary text `#888` on `#0f0f0f` (5.4:1), captions `#777` on `#0f0f0f` (4.6:1)
- Touch targets: minimum `w-8 h-8` for toggles, `w-10 h-10` for primary buttons

## Anti-patterns

- No gradient text, no glass/blur as decoration, no progress rings
- No colored borders above 1px on cards or list items
- No monospace as costume for non-technical content
- No section numbering unless sequence carries information
- No modal for tasks that don't need interruption or protected focus
