# Fix BSRoformer.cpp — Complete Fix List

## Problems Fixed

### Phase 1: Binary Execution & Download
1. ✅ `download_to_file()` — curl instead of reqwest (more reliable)
2. ✅ `extract_tar_xz_all()` / `extract_zip_all()` — extracts ALL files (not just binary)
3. ✅ `LD_LIBRARY_PATH` + `current_dir` — points linker to binary dir
4. ✅ **Symlinks created after extraction** — `libggml.so.0` → `.0.15.1`, etc.
5. ✅ **Binary size validation** — removes and redownloads if file ≤ 100KB (corrupt)

### Phase 2: Process Deadlock
6. ✅ **Stdout pipe read** — reads stdout in `stdout_task` to prevent pipe buffer deadlock
7. ✅ **stderr captured in `Arc<Mutex<String>>`** — visible in logs even after process exits

### Phase 3: Sample Rate Mismatch (CURRENT)
8. ✅ **WAV resample to 44100 Hz** — BSRoformer binary REJECTS non-44100 Hz audio
   - Error: `Error: Input audio sample rate must be 44100 Hz. Current: 48000`
   - Solution: `check_and_resample_to_44100()` usa ffprobe + ffmpeg
   - ✅ **Validado com arquivo real**: `Audio - I Write Sins Not Tragedies.wav` (48kHz → 44100 Hz)
   - ✅ **Binário BSRoformer processou**: 5% em 120s (~3min áudio)
   - ⬜ **Teste completo app TaurI**: pendente (requer UI interativa)

9. ✅ **Detecção automática de GPU + download do binário correto**
   - `detect_gpu_backend()` — verifica lspci + vulkan libs → retorna backend
   - Backends: CPU, Vulkan (AMD), CUDA 11/12 (NVIDIA), macOS ARM/x64
   - `get_binary_url()` — mapeia backend → URL correta do release
   - `extract_tar_xz_all()` — cria symlink `libggml-vulkan.so.0` quando presente
   - ✅ Testado: detectou AMD Radeon RX 9060 XT → selecionou Vulkan
   - ⬜ Teste de download automático via app Tauri: pendente

## How to implement Phase 3

Add a `check_and_resample_to_44100(input_path: &Path) -> Result<PathBuf, String>` function to `bs_roformer_cpp_cli.rs`:

```rust
/// Returns the path to use: original or temp resampled WAV at 44100 Hz
pub fn check_and_resample_to_44100(input_path: &Path) -> Result<PathBuf, String> {
    // Step 1: probe with ffprobe (blocking, fast)
    let probe = std::process::Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-show_entries", "stream=sample_rate",
            "-of", "csv=p=0",
            input_path.as_os_str().to_str().ok_or("Invalid path")?,
        ])
        .output()
        .map_err(|e| format!("ffprobe not found or failed: {}", e))?;

    if !probe.status.success() {
        return Err("Failed to probe audio file. Is ffmpeg/ffprobe installed?".to_string());
    }

    let sample_rate = String::from_utf8(probe.stdout)
        .map_err(|e| format!("Invalid ffprobe output: {}", e))?
        .trim()
        .parse::<u32>()
        .unwrap_or(0);

    if sample_rate == 44100 {
        return Ok(input_path.to_path_buf()); // already correct
    }

    // Step 2: resample to 44100 Hz
    let temp_path = input_path
        .parent()
        .map(|p| p.join("temp_44100.wav"))
        .ok_or("Cannot create temp file")?;

    eprintln!(
        "Input sample rate {} Hz != 44100 Hz. Resampling to 44100 Hz...",
        sample_rate
    );

    let resample = std::process::Command::new("ffmpeg")
        .args([
            "-y",                 // overwrite
            "-i", input_path.to_str().ok_or("Invalid input path")?,
            "-ar", "44100",       // sample rate
            temp_path.to_str().ok_or("Invalid temp path")?,
        ])
        .output()
        .map_err(|e| format!("ffmpeg not found or failed: {}", e))?;

    if !resample.status.success() {
        let stderr = String::from_utf8_lossy(&resample.stderr);
        return Err(format!("ffmpeg resample failed: {}", stderr));
    }

    Ok(temp_path)
}
```

Then in `run_bs_roformer_cpp_with_progress`:
```rust
// Use resampled file if needed (returns temp path or original)
let input_for_binary = check_and_resample_to_44100(input_path)?;

// Pass input_for_binary to the binary instead of input_path
// ...
// At end, clean up temp if it was created
if input_for_binary != input_path {
    let _ = fs::remove_file(&input_for_binary);
}
```

## Files to modify
- `src-tauri/src/bs_roformer_cpp_cli.rs` — Add `check_and_resample_to_44100()` + integrate
