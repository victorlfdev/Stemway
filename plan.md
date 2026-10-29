# Stemway — Implemented evolution

The original four-stem ONNX proposal has been superseded by the working six-stem CLI engines. Current behavior is described in PRODUCT.md and README.md.

## Completed phases

- [x] Shared transport, seek and A–B loop in the Rodio 0.18.1 audio engine.
- [x] Waveform envelopes generated from decoded audio and cached on disk.
- [x] Durable project directories and atomic JSON persistence of mixer, markers and study state.
- [x] Independent speed/pitch study rendering with FFmpeg, with reusable local copies.
- [x] Individual-stem and mix export, selectable tracks/destination/formats/excerpt.
- [x] Cancellation of owned inference, setup and conversion subprocesses and native downloads.
- [x] Sequential queue, reordering, retry, failure isolation and recovery after app restart.
- [x] CPU/GPU selection, backend-specific native binaries and hardware inspection.
- [x] Offline model preparation, storage inspection, reinstall and removal.
- [x] Short-excerpt model comparison and A/B playback at the same musical position.
- [x] Portuguese studio interface, library search, presets, markers and keyboard shortcuts.
- [x] Replace static speed claims with hardware guidance and actual local timing samples.

## Validation commands

```powershell
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings
cargo run --manifest-path src-tauri/Cargo.toml --example studio_smoke --locked
cargo run --manifest-path src-tauri/Cargo.toml --example workflow_smoke --locked -- C:/path/to/song.wav bs-roformer-cpp auto
# Optional start/end in seconds exercise the exact comparison excerpt pipeline:
cargo run --manifest-path src-tauri/Cargo.toml --example workflow_smoke --locked -- C:/path/to/song.wav demucs cpu 1 5
npm run tauri build -- --bundles nsis
```

`studio_smoke` checks real-device seek, loop, end-of-file replay, four export formats, independent study transformations, project persistence and owned-process cancellation. `workflow_smoke` runs the same command as the interface and validates six stems, audio metadata, durable storage and waveform extraction. It keeps the resulting project in the library for inspection.

## Practical boundaries

- Speed/pitch changes prepare local copies; they do not apply instantaneously during inference.
- A/B switching preserves musical position but reloads the selected result.
- Model downloads need internet; offline processing needs prepared assets.
- Runtime validation on Windows does not substitute for Linux/macOS testing.
- Playback is synchronized using paused Rodio sinks and a common transport; the app does not claim sample-accurate DAW transport.
