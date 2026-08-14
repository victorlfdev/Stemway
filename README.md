# Stemway

A local-first, open-source desktop application that separates music audio tracks into individual stems using AI models — all processing happens on your machine.

## Features

- **Multiple AI models:** HTDemucs, BS-RoFormer, BS-RoFormer.cpp with different quality/speed trade-offs
- **4 or 6 stems:** Drums, Bass, Vocals, Other (and Guitar, Piano with BS-RoFormer.cpp)
- **Local processing:** No data leaves your computer, no subscriptions required
- **GPU acceleration:** Vulkan and CUDA support via BS-RoFormer.cpp
- **Cross-platform:** Linux, Windows, macOS
- **Open source:** MIT licensed

## Getting Started

### Prerequisites

**For all platforms:**
- Rust 1.77.2 or later — [rustup](https://rustup.rs)
- Node.js 18+ — [nvm](https://github.com/nvm-sh/nvm) or package manager
- FFmpeg + ffprobe — [installation guide](https://ffmpeg.org/download.html)
- System dependencies:
  ```bash
  # Ubuntu/Debian
  sudo apt install libwebkit2gtk-4.1-dev build-essential xlib-util-dev

  # Fedora
  sudo dnf install webkit2gtk4.1-devel openssl-devel

  # Arch
  sudo pacman -S webkit2gtk-4.1 openssl

  # Windows
  # Install via vcpkg: vcpkg install openssl
  # Or download binaries from openssl.org

  # macOS
  # webkit2gtk is not needed; use `brew install webkitgtk` if prompted
  ```

**For model support:**
| Model | Dependency | Required? |
|-------|-----------|-----------|
| HTDemucs | `pipx install demucs` (Python) | Optional — installed automatically on first use |
| BS-RoFormer | `pipx install bs-roformer-infer` (Python) | Optional — installed automatically on first use |
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

### HTDemucs (4-Track Standard)
- **Quality:** Good
- **Speed:** Fast CPU processing (~3-5 min for a 3-minute song)
- **Stems:** Vocals, Drums, Bass, Other
- **GPU:** No
- **Best for:** Quick separation, reliable results

### BS-RoFormer (4-Track Essential)
- **Quality:** Good
- **Speed:** Slower, CPU only (~10-15 min)
- **Stems:** Vocals, Drums, Bass, Other
- **GPU:** No
- **Best for:** Alternative model comparison

### BS-RoFormer.cpp (6-Track Premium)
- **Quality:** Best
- **Speed:** Fast with GPU (~30 sec), slow with CPU (~25 min)
- **Stems:** Vocals, Drums, Bass, Other, Guitar, Piano
- **GPU:** Vulkan (NVIDIA/AMD), CUDA 11/12 (NVIDIA), macOS ARM/x64
- **Best for:** Highest quality, multi-instrument separation
- **Recommended** for most users

## Usage

1. **Choose a model** — Select from the three options based on quality/speed needs
2. **Drop your file** — Drag and drop a WAV or MP3 file, or click to browse
3. **Separate** — Click "Separate Stems" and wait for processing
4. **Preview** — Listen to individual stems in the results panel
5. **Export** — Open the output folder to access your stem WAV files

## Output Format

All stems are saved as:
- **Format:** 16-bit PCM WAV
- **Channels:** Stereo
- **Sample rate:** Matches input (HTDemucs/BS-RoFormer) or 44100 Hz (BS-RoFormer.cpp)

Processed files are stored in:
- **Linux:** `~/.local/share/stemway/stem-output/`
- **Windows:** `%APPDATA%\stemway\stem-output\`
- **macOS:** `~/Library/Caches/com.stemway.app/stem-output/`

## Technology Stack

- **Desktop shell:** Tauri 2 (Rust)
- **Frontend:** React 18 + Vite 5 + Tailwind CSS 3
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
These are installed automatically on first use via pipx or pip. If installation fails:
```bash
pipx install demucs
pipx install bs-roformer-infer
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
choco install ffmpeg             # Windows (Chocolatey)
```

## License

[MIT License](LICENSE)

## Acknowledgements

This project relies on the following third-party tools and models:

- **[HTDemucs](https://github.com/facebookresearch/demucs)** — *Demucs: Neural Music Separation* by François Rigollet, et al. (Facebook Research). Licensed under MIT.
- **[BS-RoFormer](https://github.com/lucidrains/BS-RoFormer)** — *BS-RoFormer: Better Symmetric Spectrogram Decomposition* by Phil Wang (lucidrains). Licensed under MIT.
- **[BS-RoFormer.cpp](https://github.com/chenmozhijin/BSRoformer.cpp)** — *BS-RoFormer C++ inference port* by chenmozhijin. Licensed under MIT.
- **[Tauri](https://tauri.app/)** — Cross-platform desktop framework. Licensed under MIT / Apache-2.0.
