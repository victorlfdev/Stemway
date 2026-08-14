<!-- impeccable:product-schema 1 -->

# Product

## Platform

web

## Users

Musicians, producers, DJs, and audio engineers who need to isolate individual instrument stems from full music tracks for purposes including remixing, sampling, mixing analysis, backing track creation, and live performance. Users operate locally on their own machines with personal audio files they own or have rights to use.

## Product Purpose

Stemway is a desktop application that uses AI models to separate a full music audio track (WAV or MP3) into four individual stems: drums, bass, vocals, and other. The product lets users choose between multiple separation models, process their files locally, preview the resulting stems, and export them as WAV files. Success means a user obtains clean, usable stems from their music in minutes, not hours.

## Positioning

Stemway offers three separation models (HTDemucs, BS-RoFormer, BS-RoFormer.cpp) with GPU acceleration options, all running locally on the user's machine. No data leaves the computer, no subscriptions are required, and the source code is open. This is a local-first, open-source alternative to web-only stem separation services.

## Operating Context

Users run the application as a standalone desktop app on Linux, Windows, or macOS. They drag-and-drop or browse for audio files (WAV or MP3), select a separation model, and wait for processing. Output is four WAV files per stem, saved to a local cache directory. The app uses `ffprobe` to probe audio metadata and manages model installation automatically on first use.

## Capabilities and Constraints

- **Input formats:** WAV, MP3 (stereo)
- **Output format:** 16-bit PCM WAV, stereo, 4 stems (drums, bass, vocals, other)
- **Models:** HTDemucs (Python-based CLI), BS-RoFormer (Python-based CLI), BS-RoFormer.cpp (Vulkan/CPU via native binary)
- **GPU acceleration:** BS-RoFormer.cpp supports Vulkan (NVIDIA/AMD) and CPU fallback
- **File management:** Processed stems stored in OS cache directory (~/.local/share/stemway/ on Linux, %APPDATA% on Windows)
- **Dependency management:** External models installed automatically on first use
- **Technical constraints:** Large model downloads (~316 MB for HTDemucs); processing time depends on file length and model complexity; CPU fallback for BS-RoFormer.cpp is significantly slower than GPU

## Brand Commitments

None confirmed at this time.

## Evidence on Hand

- Test audio files in `src-tauri/`: `test_stereo.wav`, `test-music.mp3`, `test_empty.wav`
- Test output WAVs in `src-tauri/`: `test_output_stem__bass.wav`, `test_output_stem__drums.wav`, `test_output_stem__other.wav`, `test_output_stem__vocals.wav`
- Plan documentation in `plan.md`

Future work must not fabricate testimonials, benchmark numbers, or user claims.

## Product Principles

1. **Local-first:** All processing happens on the user's machine. No network requests for audio data; models downloaded once and cached.
2. **Model choice:** Offer users a selection of AI models with different quality/speed trade-offs, not a single opaque default.
3. **Transparent output:** Clearly show processing progress, model being used, and the resulting stem files. No hidden steps.
4. **Simple workflow:** Drag a file, choose a model, get stems. The interface should not require audio engineering knowledge.

## Accessibility & Inclusion

No product-specific accessibility requirements established at this time. The app should be usable with standard desktop accessibility tools (screen readers, keyboard navigation).
