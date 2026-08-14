# Stemway

<div align="center">

</div>

A local-first, open-source desktop application that separates music audio tracks into four stems using AI models — all processing happens on your machine.

## Features

- **Four stems:** Drums, Bass, Vocals, and Other
- **Multiple AI models:** HTDemucs, BS-RoFormer, BS-RoFormer.cpp with different quality/speed trade-offs
- **Local processing:** No data leaves your computer
- **Preview & export:** Play individual stems and save as 16-bit PCM WAV files
- **GPU acceleration:** Vulkan support via BS-RoFormer.cpp for fast separation
- **Cross-platform:** Linux, Windows, macOS

## Getting Started

### Prerequisites

- Rust 1.77.2 or later
- Node.js 18+
- Python 3.8+ (for HTDemucs and BS-RoFormer models)

### Installation

```bash
# Clone the repository
git clone https://github.com/stemway/stemway.git
cd stemway

# Install dependencies
npm install

# Build and run the app
npm run tauri dev
```

### Building for Production

```bash
npm run tauri build
```

Binaries are output to `src-tauri/target/release/bundle`.

## Technology Stack

- **Desktop shell:** Tauri 2 (Rust)
- **Frontend:** React + Vite + Tailwind CSS
- **Audio separation:** HTDemucs, BS-RoFormer, BS-RoFormer.cpp (Python/C++ CLI tools)
- **Output format:** 16-bit PCM WAV, stereo

## Project Structure

```
stemway/
├── src-tauri/           # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── commands.rs  # Tauri command handlers
│   │   ├── demucs_cli.rs      # HTDemucs integration
│   │   ├── bs_roformer_cli.rs # BS-RoFormer integration
│   │   ├── bs_roformer_cpp_cli.rs # BS-RoFormer.cpp integration
│   │   └── *_output.rs # Output parsers
│   └── tauri.conf.json
├── src/                 # React frontend
├── package.json
└── tailwind.config.js
```

## License

[License pending]

## Acknowledgements

- [HTDemucs](https://github.com/facebookresearch/ambigene) - Facebook Research
- [BS-RoFormer](https://github.com/trichetmatthewp/BS-RoFormer) - Music and Audio Processing Lab
- [BS-RoFormer.cpp](https://github.com/NatanGazzah/BS-RoFormer.cpp) - GPU-accelerated C++ implementation
- [Tauri](https://tauri.app/) - Cross-platform desktop framework
