# Stem Separator — Desktop App

App desktop standalone para separação de stems de áudio usando IA (HTDemucs ONNX).

## 📋 Requisitos

- **Estilo UI:** Moises.ai (drag-and-drop, dark theme, players sincronizados)
- **Backend:** Tauri (Rust) + ONNX Runtime
- **Frontend:** React + Vite + TailwindCSS
- **Modelo:** HTDemucs ONNX (316 MB) — download no primeiro uso
- **Entrada:** WAV + MP3
- **Saída:** 4 WAVs separados (Drums, Bass, Vocals, Other)
- **Builds:** `.exe` (Windows) + `.AppImage` (Linux)

---

## 🏗️ Estrutura do Projeto

```
stem-separator/
├── src-tauri/                          # Rust backend
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/
│       ├── main.rs                     # Entry point
│       ├── inference/
│       │   ├── mod.rs                  # ONNX session manager
│       │   └── runner.rs               # Inference pipeline
│       ├── audio/
│       │   ├── mod.rs                  # Audio I/O
│       │   ├── reader.rs               # WAV/MP3 decoder
│       │   └── writer.rs               # WAV writer
│       └── utils/
│           ├── mod.rs
│           └── chunking.rs             # Overlap-add chunking
├── src/                                 # React frontend
│   ├── package.json
│   ├── vite.config.js
│   ├── index.html
│   └── src/
│       ├── main.jsx
│       ├── App.jsx
│       ├── App.css
│       └── components/
│           ├── Dropzone.jsx             # Upload WAV/MP3
│           ├── AudioPlayer.jsx          # Stem player controls
│           ├── ProgressBar.jsx          # Processing progress
│           ├── StemControls.jsx         # Volume/mute/solo
│           ├── ExportButton.jsx         # Save WAVs
│           └── Layout.jsx               # Main layout
├── tauri.conf.json                      # App config
├── package.json                         # Node deps for frontend
└── plan.md                              # Este arquivo
```

---

## 🎨 UI/UX (Estilo Moises)

### Fluxo do Usuário

```
1. Drag-and-drop ou click para upload (WAV/MP3)
2. Preview: nome, duração, tamanho
3. Botão "Separate Stems" → processamento
4. 4 players aparecem com waveform
5. Controle: volume, mute, solo por stem
6. Export: salva 4 WAVs
```

### Layout

```
┌───────────────────────────────────────────────────┐
│  🎵 Stem Separator                                │
│                                                   │
│  ┌─────────────────────────────────────────────┐  │
│  │  [Drag & Drop Audio]                        │  │
│  │  or Click to Browse                         │  │
│  └─────────────────────────────────────────────┘  │
│                                                   │
│  [song.mp3 (3:45)]  [8.5 MB]                     │
│                                                   │
│  ┌─────────────────────────────────────────────┐  │
│  │  🚀 Separate Stems                           │  │
│  └─────────────────────────────────────────────┘  │
│                                                   │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐            │
│  │Drums │ │Bass  │ │Vocals│ │Other │            │
│  │▶ 🔇 │ │▶ 🔇 │ │▶ 🔇 │ │▶ 🔇 │            │
│  │━━━━━ │ │━━━━━ │ │━━━━━ │ │━━━━━ │            │
│  │Vol:▁▁▃▁│ │Vol:▁▁▃▁│ │Vol:▁▅▃▁│ │Vol:▁▂▃▁│            │
│  └──────┘ └──────┘ └──────┘ └──────┘            │
│                                                   │
│  ┌─────────────────────────────────────────────┐  │
│  │  💾 Export All as WAV                         │  │
│  └─────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────┘
```

---

## 🔄 Pipeline de Processamento

```
Entrada (WAV/MP3)
    │
    ▼
[1] AudioReader (symphonia)
    └─> Stereo PCM @ 44.1 kHz
    │
    ▼
[2] STFT
    └─> Spectrogram (1, 2, 343980)
    │
    ▼
[3] HTDemucs ONNX (ort)
    └─> 4 × Spectrograms (1, 4, 2, 343980)
         [drums, bass, other, vocals]
    │
    ▼
[4] iSTFT
    └─> 4 × Stereo PCM
    │
    ▼
[5] OutputWAVs
    └─> drums.wav, bass.wav, vocals.wav, other.wav
```

---

## 📦 Modelo HTDemucs

### Download (Primeiro Uso)
```
URL: https://huggingface.co/StemSplitio/htdemucs/resolve/main/htdemucs.onnx
Tamanho: 316 MB
Formato: ONNX (single-file, 4 stems)
```

### Cache Local
```
Linux: ~/.local/share/stem-separator/models/htdemucs.onnx
Windows: %APPDATA%\stem-separator\models\htdemucs.onnx
```

### Especificações
- **Input tensor:** `(1, 2, 343980)` — stereo, 44.1 kHz, 7.8s
- **Output tensor:** `(1, 4, 2, 343980)` — [drums, bass, other, vocals]
- **Latência CPU:** ~1.6s/segmento (single-threaded)
- **RTF:** 0.20x

---

## 🛠️ Dependências

### Backend (Rust)
```toml
[dependencies]
tauri = { version = "1.7", features = ["shell-open"] }
ort = "2.0"                  # ONNX Runtime bindings
symphonia = "0.5"            # Audio decoding (WAV/MP3)
symphonia-codec-pcm          # PCM codec
symphonia-format-wav         # WAV format
symphonia-format-mp3         # MP3 format
rodio = "0.19"               # Audio playback
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.12", features = ["blocking"] }
sha2 = "0.10"
tokio = { version = "1", features = ["full"] }
```

### Frontend (Node)
```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "vite": "^5.0.0",
    "@vitejs/plugin-react": "^4.0.0",
    "tailwindcss": "^3.4.0"
  }
}
```

---

## 📅 Implementação por Fases

### Fase 1: Tauri + React Setup (3-4 dias)
- [ ] `cargo tauri init stem-separator`
- [ ] Configurar React + Vite + TailwindCSS
- [ ] Implementar Dropzone + Layout
- [ ] File I/O: salvar arquivo em temp dir
- [ ] Build test: `.exe` + `.AppImage`

### Fase 2: Audio I/O (2-3 dias)
- [ ] Implementar `AudioReader`:
  - WAV decoder (symphonia-format-wav)
  - MP3 decoder (symphonia-format-mp3)
  - Normalizar para stereo @ 44.1 kHz
- [ ] Implementar `AudioWriter`:
  - WAV writer (16-bit PCM, stereo)
  - Multi-output (4 stems)
- [ ] Testar com arquivos reais

### Fase 3: ONNX Inference (4-5 dias)
- [ ] Configurar `ort` crate
- [ ] Implementar `SessionManager`:
  - Carregar modelo
  - Download automático (primeiro uso)
  - Verificar checksum
- [ ] Implementar `InferenceRunner`:
  - PCM → Float32 tensor
  - `session.run()`
  - Output → 4 × stereo Float32
- [ ] Testar com áudio

### Fase 4: Overlap-Add Chunking (2-3 dias)
- [ ] Implementar `ChunkProcessor`:
  - Dividir em chunks de 7.8s
  - 25% overlap
  - Hann window
- [ ] Implementar `Stitcher`:
  - Soma chunks
  - Normalização
- [ ] Testar músicas completas

### Fase 5: Integração + Polish (2-3 dias)
- [ ] Conectar UI + backend (`taipc`)
- [ ] Progress bar
- [ ] Player sincronizado
- [ ] Controles de stem (volume/mute/solo)
- [ ] Export WAVs
- [ ] Testes finais

---

## ⚡ Performance

| Hardware | Tempo/Segmento | RTF |
|----------|----------------|-----|
| CPU (1 thread) | 1.6s | 0.20x |
| CPU (8 threads) | 0.4s | 0.05x |
| **NVIDIA GPU (CUDA)** | **~0.15s** | **0.02x** |
| **AMD GPU (MIGraphX)** | **~0.2-0.3s** | **0.03x** |

**Uso de memória:** ~600 MB (modelo + buffer)

## 🎮 GPU Acceleration

### Execution Providers (`ort` crate)

| Provider | GPU | Feature Cargo | Status | Velocidade |
|----------|-----|---------------|--------|------------|
| **CUDA** | NVIDIA | `features = ["cuda"]` | ✅ Production | 5-10x CPU |
| **MIGraphX** | AMD | `features = ["migraphx"]` | ⚠️ Preview | 4-6x CPU |
| **WebGPU** | Any | `features = ["webgpu"]` | 🧪 Experimental | 3-5x CPU |

### Implementation

```rust
use ort::{ep, session::Session};

fn load_session() -> Result<Session> {
    Session::builder()?
        .with_execution_providers([
            #[cfg(feature = "cuda")]
            ep::CUDA::default().build(),
            #[cfg(feature = "migraphx")]
            ep::MIGraphX::default().build(),
            ep::CPU().default().build(), // fallback
        ])?
        .commit_from_file("htdemucs.onnx")
}
```

### Strategy

1. **Default:** CPU-only (funciona em tudo)
2. **Auto-detect:** Verificar GPU no boot
3. **Fallback:** Silencioso para CPU se GPU indisponível
4. **UI:** Mostrar status: "Processing: Using NVIDIA RTX 3060 (CUDA)"

### Cargo Features

```toml
[dependencies]
ort = "2.0"

[features]
default = ["cpu"]
cuda = ["ort/cuda"]
migraphx = ["ort/migraphx"]
webgpu = ["ort/webgpu"]
```

---

## 🐙 Git Workflow

### Inicialização
```bash
git init
git commit --allow-empty -m "feat: init project"
```

### Estrutura de Branches
```
main              ← sempre estável, tags de versão
feature/fase-1    ← Tauri + React setup
feature/fase-2    ← Audio I/O
feature/fase-3    ← ONNX Inference
feature/fase-4    ← Overlap-Add Chunking
feature/fase-5    ← Integração + Polish
```

### Commits Incrementais
- **Commit por função/módulo completo**
- **Commit ao final de cada fase** (merge squash da feature branch)
- **Prefixos convencionais:**
  - `feat:` nova funcionalidade
  - `fix:` correção de bug
  - `refactor:` reestruturação de código
  - `docs:` documentação
  - `chore:` config, deps, tooling
  - `test:` testes

### Exemplos
```bash
git checkout -b feature/fase-1
# ... desenvolvimento ...
git commit -m "feat: setup tauri + react + vite"
git commit -m "feat: implement dropzone + layout"
git commit -m "feat: file I/O + temp dir save"
git checkout main
git merge --squash feature/fase-1
git commit -m "feat: fase 1 - tauri + react setup completo"
```

---

## 🐛 Riscos e Mitigação

| Risco | Impacto | Mitigação |
|-------|---------|-----------|
| Download lento (316 MB) | Usuário espera | Progress bar, download async |
| MP3 não decodifica | Erro no player | Fallback FFmpeg ou aviso |
| Latência > 1.6s | Processamento lento | Thread dedicado, mostrar status |
| ONNX Runtime não encontrado | App não roda | Bundler baixa binários |

---

## 📝 Checklist de Entrega

- [x] Projeto limpo (removido JUCE/VST)
- [x] Git init + commit inicial
- [x] Git workflow definido (commits incrementais + branches)
- [ ] Tauri + React inicializado
- [ ] Dropzone funcional (WAV/MP3)
- [ ] AudioReader (WAV + MP3)
- [ ] ONNX Runtime integrado
- [ ] HTDemucs download (primeiro uso)
- [ ] Inference pipeline completo
- [ ] Overlap-add chunking
- [ ] WAV export (4 stems)
- [ ] Player sincronizado
- [ ] Controles de stem
- [ ] Progress bar
- [ ] Build Windows (.exe)
- [ ] Build Linux (.AppImage)
- [ ] Testes com arquivos reais
- [ ] README.md

---

## 🚀 Como Executar

### Desenvolvimento
```bash
# Instalar Rust + Tauri CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install tauri-cli

# Instalar Node.js + npm
# (já instalado via fnvm)

# Iniciar app
cd stem-separator
npm install
npm run tauri dev
```

### Build
```bash
# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc

# Linux
npm run tauri build -- --target x86_64-unknown-linux-gnu
```
