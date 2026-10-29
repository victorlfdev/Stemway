# Stemway — Design System

<!-- impeccable:design-schema 1 -->

## Visual World

A dark, restrained desktop studio. Preserve the established near-black surface and green action accent. The interface serves musicians operating on local files: recognizable controls, honest feedback and compact audio rows take precedence over decorative effects.

## Palette

- Background: #0a0a0a; sidebar: #0e100f; surfaces: #111312 and #191d1b.
- Borders: #2a302d; text: #f0f3f1; secondary: #a2ada6.
- Accent: #75d39b; primary action background: #29844d with white text.
- Stem colors: voice #f18a91, drums #e8b66c, bass #87b0ef, guitar #74caa2, piano #8dd3da, other #b5a0e8.

## Typography

Use the local system UI font (Segoe UI on Windows). No remote font requests. Body 14px, headings 17–24px, secondary metadata 11–12px. Timecodes and percentages use tabular numerals. All factual product text is Portuguese.

## Layout and Components

- Header: Stemway identity and navigation between Estúdio and Modelos e hardware.
- Sidebar: searchable durable project library, selected item and local-storage explanation.
- Main workspace: model/backend selection, file import, queue, optional excerpt comparison and active project.
- Mixer: shared transport, timeline and six aligned waveform rows with gain, mute and solo. Waveforms represent decoded peaks; the playhead is a separate overlay.
- Advanced controls use inline disclosure sections for study/markers, comparison and export.
- Model management uses a list with storage/status and explicit actions. Removal asks inline before removing model assets; projects remain available.
- Progress is sticky within the workspace, with stage, message, elapsed time and cancellation. Unknown percentages use native indeterminate progress.
- Errors are readable, dismissible and announced through an alert region. Completion messages use status regions.

## Responsive Behavior

At 1024px, use a 190px project sidebar. Below 760px, stack the library above the workspace and arrange project buttons horizontally. Track volume moves to a second row on narrow windows. Toolbars wrap and file paths truncate with full paths available through titles.

## Accessibility and Motion

Use native buttons, selects, checkboxes and range controls. All sliders and toggle buttons have descriptive accessible names and visible keyboard focus. M/S expose pressed state. Avoid decorative entrance animations. Respect reduced motion. Ctrl+O adds files; Space toggles transport and arrows seek when focus is outside form controls.

## Content and Interaction Rules

No invented benchmarks, quality guarantees or subscription tiers. Show actual engine names and backend. Label local measurements and estimates. Maintain clear distinctions between original stems, audible mix and transformed study copies. Exports always write a new subfolder.
