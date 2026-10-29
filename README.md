# Stemway

A local-first, open-source desktop application that separates music audio tracks into individual stems using AI models — all processing happens on your machine.

## Run on Windows (validated setup)

After a local build, open `src-tauri/target/release/stemway.exe` directly, or install
`src-tauri/target/release/bundle/nsis/Stemway_0.3.0_x64-setup.exe`. The release app
includes the frontend and does not need Vite or a terminal running.

The default **BS-RoFormer nativo** model runs on the Radeon RX 9060 XT through
Vulkan. It downloads the official Windows inference engine and GGUF model on
first use. A compatible graphics driver is sufficient; the Vulkan SDK is not
required to run the prebuilt engine.

To develop locally, install current stable Rust, Visual Studio C++ tools, Node.js
18/20/22+ and FFmpeg (including ffprobe), then run `npm ci` and `npm run tauri dev`.
Restart your terminal after installing Rust so Cargo is on PATH. Visual Studio
Insiders is supported by the current Rust toolchain.

The Python engines use a private environment under
`%LOCALAPPDATA%/stemway/data/python-env`; packages are installed there on first
use, without modifying global Python packages. For an explicit setup with Python
3.12, run in PowerShell:

```powershell
python -m venv "$env:LOCALAPPDATA/stemway/data/python-env"
& "$env:LOCALAPPDATA/stemway/data/python-env/Scripts/python.exe" -m pip install -r requirements-audio.txt
```

For development, a project `.venv` or the active `VIRTUAL_ENV` is also detected.
Python subprocesses run in UTF-8 on Windows. Torch and torchaudio are kept at the
tested matching version 2.5.1.

## Features

- **Multiple AI models:** HTDemucs, BS-RoFormer, BS-RoFormer.cpp with different quality/speed trade-offs
- **6 stems:** Drums, Bass, Vocals, Other, Guitar and Piano
- **Local processing:** No data leaves your computer, no subscriptions required
- **GPU acceleration:** Vulkan and CUDA support via BS-RoFormer.cpp
- **Cross-platform:** Linux, Windows, macOS
- **Open source:** MIT licensed
- **Persistent library:** Reopen projects with mixer levels, mute/solo, markers, loop and study settings
- **Waveform player:** Shared timeline, seeking, A–B loop and keyboard shortcuts
- **Batch queue:** Multiple files, reordering, retries, pause after current task and real cancellation
- **Study tools:** Independent speed (0.5–2x) and pitch (-12 to +12 semitones), named markers and backing-track presets
- **Custom export:** Selected stems or the audible mix, WAV 16/24-bit, FLAC and MP3, optional excerpts and study settings
- **Model comparison:** Process the same excerpt (up to 30 seconds) with two models, then switch A/B at the same position
- **Offline model manager:** Prepare, inspect storage, reinstall or remove model assets; CPU/GPU preference and actual local timing samples
- **Portuguese interface:** Accessible controls, library search and an offline system font

## Getting Started

### Prerequisites

**For Linux:**
- Current stable Rust (tested with 1.99.1) — `rustup`
- System dependencies:
  ```bash
  # Ubuntu/Debian
  sudo apt install libwebkit2gtk-4.1-dev build-essential xlib-util-dev

  # Fedora
  sudo dnf install webkit2gtk4.1-devel openssl-devel

  # Arch
  sudo pacman -S webkit2gtk-4.1 openssl
  ```

**For macOS:**
- Current stable Rust (tested with 1.99.1) — `rustup`
- Node.js 18+ — [nvm](https://github.com/nvm-sh/nvm) or [brew](https://brew.sh)

**For Windows:**
- Current stable Rust (tested with 1.99.1) — `winget install Rustlang.Rustup`
- Visual Studio C++ Build Tools — [Download](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
  - During installation, select the **"C++ build tools"** workload
  - Ensure **"Windows 10 SDK"** and **"C++ CMake tools"** are included
- Node.js 18+ — [nvm-windows](https://github.com/coreybutler/nvm-windows) or [installer](https://nodejs.org)
- FFmpeg + ffprobe — `winget install ffmpeg` or `scoop install ffmpeg`

**For all platforms:**
- Python 3.10–3.12 — required for HTDemucs and BS-RoFormer; installed into a private environment

**For model support:**
| Model | Dependency | Required? |
|-------|-----------|-----------|
| HTDemucs | `demucs==4.0.1` (private Python environment) | Optional — installed automatically on first use |
| BS-RoFormer | `bs-roformer-infer==0.1.5` (private Python environment) | Optional — installed automatically on first use |
| BS-RoFormer.cpp | None (self-contained binary) | Optional — downloaded automatically on first use |

### Installation

```bash
# Clone the repository
git clone https://github.com/stemway/stemway.git
cd stemway

# Install frontend dependencies
npm install

# Run the app in development mode
npm run tauri dev
```

### Building for Production

```bash
# Build for your current OS
npm run tauri build

# Or build for a specific target:
npm run tauri build -- --target x86_64-unknown-linux-gnu  # Linux AppImage
npm run tauri build -- --target x86_64-pc-windows-msvc    # Windows .exe
npm run tauri build -- --target aarch64-apple-darwin      # macOS universal
```

Binaries are output to `src-tauri/target/release/bundle`:
- **Linux:** `.AppImage`
- **Windows:** `.exe` (NSIS installer)
- **macOS:** `.dmg` + `.app`

## Models Compared

### HTDemucs
- **Speed:** CPU processing; see measured timings on your computer in the app
- **Stems:** Vocals, Drums, Bass, Other, Guitar, Piano
- **GPU:** No
- **Best for:** Quick separation, reliable results

### BS-RoFormer Python
- **Speed:** CPU processing; runtime depends on track length and hardware
- **Stems:** Vocals, Drums, Bass, Other, Guitar, Piano
- **GPU:** No
- **Best for:** Alternative model comparison

### BS-RoFormer nativo (BS-RoFormer.cpp)
- **Speed:** GPU or CPU processing; the app records actual local timings
- **Stems:** Vocals, Drums, Bass, Other, Guitar, Piano
- **GPU:** Vulkan (NVIDIA/AMD), CUDA 11/12 (NVIDIA), macOS ARM/x64
- **Best for:** Native multi-instrument separation with a compatible GPU
- Hardware guidance and short-excerpt A/B comparison help choose the appropriate engine

## Usage

1. **Choose a model** and CPU/GPU preference (the native engine supports explicit backend selection).
2. **Add music** using the file picker, Ctrl+O or drag-and-drop. WAV, MP3, FLAC, OGG, AIFF and M4A are accepted.
3. **Process the queue** with “Processar fila”. Reorder pending tasks, retry failures, pause after the current track or cancel the current operation.
4. **Listen and study** using the shared timeline, six real waveforms, volume/mute/solo and presets. Space toggles playback; arrows seek five seconds when focus is outside form controls.
5. **Use study tools** to mark A/B, loop an excerpt, add named markers, and apply speed/pitch independently. Applying settings prepares cached local copies.
6. **Export** a selected mix or individual stems, choosing format, destination, excerpt and study settings. A fresh subfolder preserves existing files.
7. **Return later** through the searchable project library. The mixer, markers, study settings and listening position are saved automatically.

To compare models, click a source file in the queue and expand “Comparar modelos em um trecho”. Select two different engines and an excerpt up to 30 seconds. Completed A/B buttons preserve musical position and listening settings while reloading the selected result; switching is not gapless.

“Modelos e hardware” displays the detected adapter, backend, owned model storage and offline preparation actions. It reports actual local timing samples rather than universal performance guarantees. The Python environments remain installed when model weights are removed.

## Output Format

All stems are saved as:
- **Format:** WAV (16-bit PCM for HTDemucs; floating-point WAV for RoFormer)
- **Channels:** Stereo
- **Sample rate:** 44100 Hz; input files are converted when needed

New projects are stored permanently in the `projects/` subdirectory of the local application data directory, including a `project.json` manifest:
- **Linux:** `~/.local/share/stemway/projects/`
- **Windows:** `%LOCALAPPDATA%\stemway\data\projects\`
- **macOS:** `~/Library/Application Support/stemway/projects/`

Model engines/weights remain in application-owned cache directories. Waveform envelopes and reusable study copies live under application data. Legacy separation outputs in the old cache location are left untouched. An interrupted queue is restored in a paused state and requires a manual restart.

Custom exports support WAV PCM 16/24-bit, FLAC and MP3 320 kbps at 44.1 kHz stereo. Individual exports preserve stem levels; mix exports apply volume, mute and solo plus a clipping limiter. Original stem WAVs are preserved in every case.

## Technology Stack

- **Desktop shell:** Tauri 2 (Rust)
- **Frontend:** React 18 + Vite 6 + Tailwind CSS 3
- **Audio separation:** HTDemucs, BS-RoFormer, BS-RoFormer.cpp (Python/C++ CLI tools)
- **External tools:** FFmpeg (audio format conversion), ffprobe (metadata extraction)
- **Model files:** ONNX / GGUF format, cached locally on first use

## Project Structure

```
stemway/
├── src-tauri/           # Rust backend (Tauri 2)
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── lib.rs       # Tauri app setup, command handlers
│   │   ├── commands.rs  # Tauri invoke handlers
│   │   ├── demucs_cli.rs           # HTDemucs Python CLI wrapper
│   │   ├── bs_roformer_cli.rs      # BS-RoFormer Python CLI wrapper
│   │   ├── bs_roformer_cpp_cli.rs  # BS-RoFormer.cpp C++ CLI wrapper
│   │   ├── *_output.rs              # Output file parsers
│   │   └── info.rs                  # Audio probing (ffprobe)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                 # React frontend
│   ├── components/      # UI components
│   ├── App.jsx          # Main app logic
│   └── main.jsx         # Entry point
├── package.json
└── tailwind.config.js
```

## Troubleshooting

### "demucs not found" / "bs-roformer-infer not found"
These are installed automatically into Stemway's private Python environment.
If setup fails, run the PowerShell commands in **Run on Windows** above. Avoid
installing arbitrary newer Torch/torchaudio versions; use `requirements-audio.txt`.

### BS-RoFormer.cpp won't launch (missing DLLs on Windows)
If you see errors like `api-ms-win-core...` or `Vulkan.dll not found`:
```bash
# Install Vulkan runtime
winget install LunarG.VulkanSDK

# For CUDA support, install CUDA Toolkit from NVIDIA:
# https://developer.nvidia.com/cuda-toolkit-archive
```

### BS-RoFormer.cpp won't launch (missing libraries on Linux)
Check with `ldd src-tauri/*/binaries/bs-roformer/bs_roformer-cli`. Common fixes:
```bash
# Vulkan (AMD/NVIDIA)
sudo apt install libvulkan1 vulkan-tools
sudo apt install vulkan-amdgpu-driver  # AMD
sudo apt install nvidia-vulkan-icd     # NVIDIA

# CUDA (NVIDIA)
sudo apt install cuda-toolkit
```

### macOS code signing errors
```bash
codesign --remove-signature src-tauri/target/*/stemway.app
codesign --force --deep --sign - src-tauri/target/*/stemway.app
```

### ffprobe not found
Install FFmpeg:
```bash
sudo apt install ffmpeg          # Ubuntu/Debian
sudo dnf install ffmpeg          # Fedora
brew install ffmpeg              # macOS
winget install ffmpeg            # Windows (winget, recommended)
choco install ffmpeg             # Windows (Chocolatey)
scoop install ffmpeg             # Windows (scoop)
```

## License

[MIT License](LICENSE)

## Acknowledgements

This project relies on the following third-party tools and models:

- **[HTDemucs](https://github.com/facebookresearch/demucs)** — *Demucs: Neural Music Separation* by François Rigollet, et al. (Facebook Research). Licensed under MIT.
- **[BS-RoFormer](https://github.com/lucidrains/BS-RoFormer)** — *BS-RoFormer: Better Symmetric Spectrogram Decomposition* by Phil Wang (lucidrains). Licensed under MIT.
- **[BS-RoFormer.cpp](https://github.com/chenmozhijin/BSRoformer.cpp)** — *BS-RoFormer C++ inference port* by chenmozhijin. Licensed under MIT.
- **[Tauri](https://tauri.app/)** — Cross-platform desktop framework. Licensed under MIT / Apache-2.0.

## Validation

FFmpeg must be on PATH for backend audio tests.

```powershell
npm run build
npm test
npm audit
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings
# Real-device transport, study render, four export formats, persistence and cancellation:
cargo run --manifest-path src-tauri/Cargo.toml --example studio_smoke --locked
# The exact command used by the interface; saves a real project in the library:
cargo run --manifest-path src-tauri/Cargo.toml --example workflow_smoke --locked -- "C:/path/to/audio.wav" bs-roformer-cpp auto
# Real pipeline test; downloads the selected model on first use:
cargo run --manifest-path src-tauri/Cargo.toml --example smoke -- "C:/path/to/audio.wav" bs-roformer-cpp --audio
```

The smoke example accepts `demucs` or `bs-roformer` as well. It validates all six
output files, duration, stereo channels and sample rate. `--audio` also checks
loading, playback, volume, mute, solo and unloading against the real audio device.
Every separation receives a new output directory to avoid returning stale stems.
