<!-- impeccable:product-schema 1 -->

# Product

## Platform

web

## Users

Musicians, producers, DJs and audio engineers using the Stemway desktop app on their own computer. Core tasks are extracting stems, creating backing tracks, comparing separation models and practicing instrument parts.

## Product Purpose

Separate local audio into six stems: vocals, drums, bass, guitar, piano and other. Let users listen, navigate, mix, study and export those results, then reopen their work without repeating inference.

## Positioning

An open-source, local-first desktop studio built with Tauri 2, Rust and React 18. Audio is never uploaded. Model engines and weights are downloaded when requested or on first use; prepared models can run offline. No paid tiers: model names identify their engine rather than a subscription level.

## Operating Context

Windows, Linux and macOS desktop app. The latest integration checks run on Windows with Vulkan and a real output device. Other operating systems require platform validation before claiming equivalent runtime coverage.

## Capabilities and Constraints

- Inputs: WAV, MP3, FLAC, OGG, AIFF/AIF and M4A through FFmpeg.
- Six-stem models: HTDemucs 4.0.1, BS-RoFormer-Infer 0.1.5 and BSRoformer.cpp 0.1.0.
- Native model: automatic backend selection or explicit CPU/GPU choice. Python engines currently use CPU.
- Sequential file queue with reordering, removal, retries, pause after the current task and cancellation of owned subprocesses. Restored interrupted tasks wait for a manual restart.
- Durable local project library with saved gains, mute/solo, study settings, loop, position and markers.
- Mixer with real waveform envelopes, shared transport, seeking, presets and keyboard shortcuts.
- Study copies with independent speed (0.5–2x) and pitch (-12 to +12 semitones), A–B loop and named markers. Study copies and waveform envelopes are cached locally.
- Export selected individual stems or the audible mix as WAV 16/24-bit, FLAC or MP3 320 kbps, including an optional excerpt and study settings. Export creates a new subfolder to avoid overwriting sources.
- Model comparison processes up to 30 seconds through two models, then switches A/B at the same musical position using the same mix and study settings. Switching reloads audio; it is not a gapless crossfade.
- Model management shows owned weight storage, downloads for offline use, reinstalls and removes model assets while preserving projects and the private Python environment.
- Hardware-based model suggestion plus empirical processing ratios from this computer, filtered to the selected backend. These are observations, not quality rankings or universal benchmarks.
- Progress distinguishes setup, install, download, conversion, inference and saving. Unknown percentages stay indeterminate. ETA appears only with useful inference progress.
- FFmpeg/ffprobe must be available. Python 3.10–3.12 is needed for the Python models.

## Brand Commitments

Simple task flow, honest progress, no invented quality guarantees, benchmark claims, testimonials or subscription implications.

## Evidence on Hand

Rust unit tests, frontend behavior tests, real-device `studio_smoke`, and real-model `workflow_smoke` examples. The repository contains a music test file for manual integration validation.

## Product Principles

1. Keep music and projects local.
2. Preserve originals; write derived audio to separate files.
3. Show actual model/backend and distinguish measured values from estimates.
4. Keep advanced study, comparison and export controls discoverable without obscuring the basic separation workflow.
5. Persist completed work and let a failed queue task leave other tasks usable.

## Accessibility & Inclusion

Portuguese interface, descriptive labels for all sliders and mute/solo buttons, keyboard focus, native form controls, screen-reader status messages and reduced-motion support. Shortcuts: Ctrl+O to add files, Space to toggle playback and left/right arrows to seek five seconds when a form control is not focused.
