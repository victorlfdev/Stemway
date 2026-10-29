use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;

#[derive(Debug, Clone, PartialEq)]
pub enum BsRoformerBackend {
    Cpu,
    Vulkan,
    Cuda11,
    Cuda12,
    MacosArm,
    MacosX64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BsRoformerCppProgress {
    pub stage: String,
    pub percent: f64,
    pub message: String,
    pub stem_index: Option<u32>,
}

#[cfg(windows)]
const BSROFORMER_BINARY_NAME: &str = "bs_roformer-cli.exe";
#[cfg(not(windows))]
const BSROFORMER_BINARY_NAME: &str = "bs_roformer-cli";
const BSROFORMER_MODEL_URL: &str = "https://huggingface.co/victorlfdev/bs-roformer-multi-q8/resolve/main/bs-roformer-multi-q8.gguf";
const BSROFORMER_BINARY_URL_LINUX_VULKAN: &str = "https://github.com/victorlfdev/Fork-BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-linux-vulkan.tar.xz";
const BSROFORMER_BINARY_URL_LINUX_CPU: &str = "https://github.com/chenmozhijin/BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-linux-x64-cpu.tar.xz";
const BSROFORMER_BINARY_URL_LINUX_CUDA11: &str = "https://github.com/chenmozhijin/BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-linux-cuda-11.8.0.tar.xz";
const BSROFORMER_BINARY_URL_LINUX_CUDA12: &str = "https://github.com/chenmozhijin/BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-linux-cuda-12.9.1.tar.xz";
const BSROFORMER_BINARY_URL_WINDOWS_VULKAN: &str = "https://github.com/chenmozhijin/BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-windows-vulkan.zip";
const BSROFORMER_BINARY_URL_WINDOWS_CPU: &str = "https://github.com/chenmozhijin/BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-windows-x64-msvc.zip";
const BSROFORMER_MODEL_NAME: &str = "bs-roformer-multi-q8.gguf";

pub fn detect_gpu_backend() -> BsRoformerBackend {
    if cfg!(target_os = "macos") {
        #[cfg(target_arch = "aarch64")]
        {
            return BsRoformerBackend::MacosArm;
        }
        #[cfg(target_arch = "x86_64")]
        {
            return BsRoformerBackend::MacosX64;
        }
    }

    if cfg!(target_os = "linux") {
        let output = std::process::Command::new("sh")
            .args([
                "-c",
                "lspci 2>/dev/null | grep -i -E 'vga|3d|display' || true",
            ])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        if output.contains("NVIDIA")
            || output.contains("GeForce")
            || output.contains("RTX")
            || output.contains("Tesla")
        {
            let cuda12 = std::process::Command::new("sh")
                .args(["-c", "nvidia-smi 2>/dev/null | head -5 || true"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default();
            if !cuda12.is_empty() {
                return BsRoformerBackend::Cuda12;
            }
            return BsRoformerBackend::Cuda11;
        }

        if output.contains("AMD")
            || output.contains("ATi")
            || output.contains("Radeon")
            || output.contains("RX ")
        {
            let vulkan_lib = std::process::Command::new("sh")
                .args([
                    "-c",
                    "find /usr/lib /usr/lib64 -name 'libvulkan*' 2>/dev/null | head -1 || true",
                ])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default();
            if !vulkan_lib.trim().is_empty() {
                eprintln!("Detected AMD GPU with Vulkan support");
                return BsRoformerBackend::Vulkan;
            }
        }

        eprintln!("No compatible GPU detected, using CPU backend");
        return BsRoformerBackend::Cpu;
    }

    if cfg!(target_os = "windows") {
        let mut probe = std::process::Command::new("powershell.exe");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            probe.creation_flags(0x08000000);
        }
        let output = probe
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "(Get-CimInstance Win32_VideoController).Name",
            ])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        if output.contains("NVIDIA") || output.contains("GeForce") || output.contains("RTX") {
            return BsRoformerBackend::Vulkan;
        }

        if output.contains("AMD") || output.contains("Radeon") {
            return BsRoformerBackend::Vulkan;
        }

        eprintln!("No compatible GPU detected on Windows, using CPU backend");
        return BsRoformerBackend::Cpu;
    }

    BsRoformerBackend::Cpu
}

fn get_binary_url(backend: &BsRoformerBackend) -> &'static str {
    if cfg!(target_os = "linux") {
        match backend {
            BsRoformerBackend::Vulkan => BSROFORMER_BINARY_URL_LINUX_VULKAN,
            BsRoformerBackend::Cuda12 => BSROFORMER_BINARY_URL_LINUX_CUDA12,
            BsRoformerBackend::Cuda11 => BSROFORMER_BINARY_URL_LINUX_CUDA11,
            BsRoformerBackend::Cpu => BSROFORMER_BINARY_URL_LINUX_CPU,
            _ => BSROFORMER_BINARY_URL_LINUX_CPU,
        }
    } else if cfg!(target_os = "windows") {
        match backend {
            BsRoformerBackend::Vulkan | BsRoformerBackend::Cuda12 | BsRoformerBackend::Cuda11 => {
                BSROFORMER_BINARY_URL_WINDOWS_VULKAN
            }
            _ => BSROFORMER_BINARY_URL_WINDOWS_CPU,
        }
    } else if cfg!(target_os = "macos") {
        match backend {
            BsRoformerBackend::MacosArm => "https://github.com/chenmozhijin/BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-macos-arm64.tar.xz",
            BsRoformerBackend::MacosX64 => "https://github.com/chenmozhijin/BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-macos-x64.tar.xz",
            _ => "https://github.com/chenmozhijin/BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-macos-arm64.tar.xz",
        }
    } else {
        panic!("Unsupported platform");
    }
}

pub fn get_backend_name(backend: &BsRoformerBackend) -> &'static str {
    match backend {
        BsRoformerBackend::Cpu => "CPU",
        BsRoformerBackend::Vulkan => "Vulkan (GPU)",
        BsRoformerBackend::Cuda11 => "CUDA 11 (GPU)",
        BsRoformerBackend::Cuda12 => "CUDA 12 (GPU)",
        BsRoformerBackend::MacosArm => "macOS ARM",
        BsRoformerBackend::MacosX64 => "macOS x64",
    }
}

pub fn get_cache_dirs() -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let dirs = directories::ProjectDirs::from("", "", "stemway")
        .map(|d| d.cache_dir().to_path_buf())
        .unwrap_or_else(std::env::temp_dir);

    let binary_dir = dirs.join("binaries").join("bs-roformer");
    let model_dir = dirs.join("models").join("bs-roformer");

    fs::create_dir_all(&binary_dir).map_err(|e| format!("Failed to create binary dir: {}", e))?;
    fs::create_dir_all(&model_dir).map_err(|e| format!("Failed to create model dir: {}", e))?;

    Ok((dirs, binary_dir, model_dir))
}

pub async fn get_binary_path() -> Result<PathBuf, String> {
    let backend = detect_gpu_backend();
    get_binary_path_for(&backend).await
}

pub async fn get_binary_path_for(backend: &BsRoformerBackend) -> Result<PathBuf, String> {
    let (cache_dir, binary_root, _model_dir) = get_cache_dirs()?;
    let binary_dir = binary_root.join(format!("{backend:?}"));
    fs::create_dir_all(&binary_dir).map_err(|e| e.to_string())?;
    let binary_path = binary_dir.join(BSROFORMER_BINARY_NAME);

    eprintln!("BSRoformer backend: {}", get_backend_name(backend));

    // Redownload if binary exists but is empty or corrupted
    if binary_path.exists() {
        let metadata = std::fs::metadata(&binary_path).map_err(|e| e.to_string())?;
        if metadata.len() > 100_000 {
            return Ok(binary_path);
        }
        let _ = std::fs::remove_file(&binary_path);
    }

    let binary_url = get_binary_url(backend);

    eprintln!(
        "Downloading BSRoformer.cpp binary (backend: {})...",
        get_backend_name(backend)
    );
    download_and_extract_binary(binary_url, &binary_dir, &cache_dir, backend).await?;

    if !binary_path.exists() {
        return Err("Failed to extract BSRoformer binary".to_string());
    }

    #[cfg(unix)]
    {
        let _ = std::fs::set_permissions(&binary_path, std::fs::Permissions::from_mode(0o755));
    }

    Ok(binary_path)
}

async fn download_to_file(url: &str, dest: &Path) -> Result<(), String> {
    crate::jobs::check()?;
    let client = reqwest::Client::new();
    let response = tokio::select! {
        response = client.get(url).send() => response.map_err(|e|format!("Falha no download: {e}"))?,
        _ = crate::jobs::cancelled() => return Err("Operação cancelada".into()),
    };

    if !response.status().is_success() {
        return Err(format!("Download failed (status: {})", response.status()));
    }

    use std::io::Write;
    let temporary = dest.with_extension("download-part");
    let mut file = std::fs::File::create(&temporary).map_err(|e| e.to_string())?;
    let expected = response.content_length();
    let mut response = response;
    let mut downloaded = 0u64;
    let mut reported = std::time::Instant::now() - std::time::Duration::from_secs(1);
    loop {
        let chunk = tokio::select! {
            chunk = response.chunk() => chunk.map_err(|e|format!("Download interrompido: {e}"))?,
            _ = crate::jobs::cancelled() => return Err("Operação cancelada".into()),
        };
        let Some(chunk) = chunk else {
            break;
        };
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        crate::jobs::check()?;
        downloaded += chunk.len() as u64;
        if reported.elapsed().as_millis() >= 200 {
            crate::jobs::progress(
                "downloading",
                expected.map(|n| downloaded as f64 / n as f64 * 100.0),
                &format!(
                    "Download: {:.1} MB{}",
                    downloaded as f64 / 1_000_000.0,
                    expected
                        .map(|n| format!(" / {:.1} MB", n as f64 / 1_000_000.0))
                        .unwrap_or_default()
                ),
            );
            reported = std::time::Instant::now();
        }
    }
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    if expected.is_some_and(|size| size != downloaded) {
        return Err("Incomplete download; please retry".into());
    }
    std::fs::rename(&temporary, dest).map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn download_and_extract_binary(
    url: &str,
    dest_dir: &Path,
    cache_dir: &Path,
    backend: &BsRoformerBackend,
) -> Result<(), String> {
    if cfg!(target_os = "linux") {
        extract_tar_xz_all(url, dest_dir, cache_dir, backend).await?;
    } else if cfg!(target_os = "windows") {
        extract_zip_all(url, dest_dir).await?;
    } else if cfg!(target_os = "macos") {
        extract_tar_xz_all(url, dest_dir, cache_dir, backend).await?;
    }

    Ok(())
}

async fn extract_tar_xz_all(
    url: &str,
    dest_dir: &Path,
    _cache_dir: &Path,
    backend: &BsRoformerBackend,
) -> Result<(), String> {
    let temp_tar = dest_dir.join("temp_download.tar.xz");
    download_to_file(url, &temp_tar).await?;

    let compressed =
        std::fs::read(&temp_tar).map_err(|e| format!("Failed to read tar.xz file: {}", e))?;

    let decoder = xz2::read::XzDecoder::new_multi_decoder(std::io::Cursor::new(&compressed));
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(dest_dir)
        .map_err(|e| format!("Failed to extract tar archive: {}", e))?;

    let _ = std::fs::remove_file(&temp_tar);

    let binary_path = dest_dir.join(BSROFORMER_BINARY_NAME);
    if !binary_path.exists() {
        return Err("Could not find bs_roformer-cli in extracted archive".to_string());
    }

    let mut libs = vec![
        ("libggml.so.0", "libggml.so.0.19.0"),
        ("libggml-base.so.0", "libggml-base.so.0.19.0"),
        ("libggml-cpu.so.0", "libggml-cpu.so.0.19.0"),
    ];

    match backend {
        BsRoformerBackend::Vulkan => {
            libs.push(("libggml-vulkan.so.0", "libggml-vulkan.so.0.19.0"));
            eprintln!("Extracted Vulkan backend ({} libs)", libs.len());
        }
        _ => {
            eprintln!("Extracted {} shared libraries", libs.len());
        }
    }

    for (link_name, target_name) in &libs {
        let link_path = dest_dir.join(link_name);
        if !link_path.exists() {
            #[cfg(unix)]
            {
                let _ = std::os::unix::fs::symlink(target_name, &link_path);
            }
            #[cfg(not(unix))]
            {
                // Esta função só roda em Linux/macOS em tempo de execução,
                // mas ainda é compilada em todas as plataformas — sem symlink no Windows.
                let target_path = dest_dir.join(target_name);
                let _ = std::fs::copy(&target_path, &link_path);
            }
        }
    }

    Ok(())
}

async fn extract_zip_all(url: &str, dest_dir: &Path) -> Result<(), String> {
    let temp_zip = dest_dir.join("temp_download.zip");
    download_to_file(url, &temp_zip).await?;

    let file =
        std::fs::File::open(&temp_zip).map_err(|e| format!("Failed to open zip file: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to parse zip archive: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry {}: {}", i, e))?;
        let out_path = match entry.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }
            let mut outfile = std::fs::File::create(&out_path)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }

    let _ = std::fs::remove_file(&temp_zip);

    let binary_path = dest_dir.join(BSROFORMER_BINARY_NAME);
    if !binary_path.exists() {
        return Err("Could not find bs_roformer-cli in extracted archive".to_string());
    }

    Ok(())
}

pub async fn get_model_path() -> Result<PathBuf, String> {
    let (_cache_dir, _binary_dir, model_dir) = get_cache_dirs()?;
    let model_path = model_dir.join(BSROFORMER_MODEL_NAME);

    if model_path.exists() {
        return Ok(model_path);
    }

    eprintln!("Downloading BSRoformer model (~250 MB)...");
    download_model_async(&model_path).await?;
    eprintln!("Model downloaded successfully.");

    Ok(model_path)
}

async fn download_model_async(dest: &Path) -> Result<(), String> {
    let parent = dest.parent().ok_or("Invalid parent directory")?;
    fs::create_dir_all(parent).map_err(|e| format!("Failed to create model dir: {}", e))?;

    let temp_file = dest.with_extension(".download");
    download_to_file(BSROFORMER_MODEL_URL, &temp_file).await?;

    std::fs::rename(&temp_file, dest).map_err(|e| format!("Failed to rename model file: {}", e))?;

    Ok(())
}

pub fn convert_and_resample_if_needed(input_path: &Path) -> Result<PathBuf, String> {
    let (cache, _, _) = get_cache_dirs()?;
    let temp_dir = cache.join("converted-input");
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let temp_path = temp_dir.join(format!("input-{}-{timestamp}.wav", std::process::id()));
    // WAV files also need conversion: the CLI requires 44.1 kHz.
    if let Err(error) = crate::info::convert_to_wav(input_path, &temp_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(error);
    }
    Ok(temp_path)
}

pub async fn run_bs_roformer_cpp_with_progress<F>(
    input_path: &Path,
    output_dir: &Path,
    progress_callback: F,
) -> Result<Vec<(String, String)>, String>
where
    F: Fn(BsRoformerCppProgress) + Send + Sync + 'static,
{
    run_bs_roformer_cpp_backend(
        input_path,
        output_dir,
        detect_gpu_backend(),
        progress_callback,
    )
    .await
}

pub async fn run_bs_roformer_cpp_backend<F>(
    input_path: &Path,
    output_dir: &Path,
    backend: BsRoformerBackend,
    progress_callback: F,
) -> Result<Vec<(String, String)>, String>
where
    F: Fn(BsRoformerCppProgress) + Send + Sync + 'static,
{
    progress_callback(BsRoformerCppProgress {
        stage: "Preparing".into(),
        percent: 0.0,
        message: "Preparing the engine and model (first use downloads files)...".into(),
        stem_index: None,
    });
    let binary = get_binary_path_for(&backend).await?;
    let model = get_model_path().await?;
    let source = input_path.to_owned();
    crate::jobs::progress(
        "converting",
        None,
        "Convertendo áudio para 44,1 kHz estéreo…",
    );
    let converted =
        tauri::async_runtime::spawn_blocking(move || convert_and_resample_if_needed(&source))
            .await
            .map_err(|e| e.to_string())??;
    let result = run_inference(&binary, &model, &converted, output_dir, progress_callback).await;
    let _ = fs::remove_file(converted);
    result
}

async fn run_inference<F>(
    binary: &Path,
    model: &Path,
    input: &Path,
    output_dir: &Path,
    callback: F,
) -> Result<Vec<(String, String)>, String>
where
    F: Fn(BsRoformerCppProgress) + Send + Sync + 'static,
{
    let base = output_dir.join("output.wav");
    let mut cmd = TokioCommand::new(binary);
    cmd.arg(model)
        .arg(input)
        .arg(&base)
        .args(["--overlap", "2"])
        .current_dir(binary.parent().ok_or("Invalid binary path")?)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(target_os = "linux")]
    cmd.env(
        "LD_LIBRARY_PATH",
        binary.parent().ok_or("Invalid binary path")?,
    );
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    #[cfg(unix)]
    cmd.process_group(0);
    crate::jobs::check()?;
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Cannot start the inference engine: {e}"))?;
    let _guard = crate::jobs::register(child.id().ok_or("Processo indisponível")?)?;
    let stdout = child.stdout.take().ok_or("Missing stdout")?;
    let stderr = child.stderr.take().ok_or("Missing stderr")?;
    let callback = std::sync::Arc::new(callback);
    callback(BsRoformerCppProgress {
        stage: "Processing".into(),
        percent: 5.0,
        message: "Separating audio...".into(),
        stem_index: None,
    });
    let out_cb = callback.clone();
    let err_cb = callback.clone();
    let stdout_task = tokio::spawn(read_inference_output(stdout, out_cb));
    let stderr_task = tokio::spawn(read_inference_output(stderr, err_cb));
    let status = child.wait().await.map_err(|e| e.to_string())?;
    let stdout = stdout_task.await.map_err(|e| e.to_string())??;
    let stderr = stderr_task.await.map_err(|e| e.to_string())??;
    crate::jobs::check()?;
    if !status.success() {
        return Err(format!("Inference failed ({status}): {stderr}\n{stdout}"));
    }
    let pairs = parse_multi_stem_output(output_dir, &base)?;
    let mut stems = Vec::new();
    for (name, path) in pairs {
        let dest = output_dir.join(format!("{name}.wav"));
        fs::rename(&path, &dest).map_err(|e| format!("Failed to save {name}: {e}"))?;
        stems.push((name, dest.to_string_lossy().into_owned()));
    }
    callback(BsRoformerCppProgress {
        stage: "Complete".into(),
        percent: 100.0,
        message: "Separated 6 stems!".into(),
        stem_index: None,
    });
    Ok(stems)
}

async fn read_inference_output<R, F>(
    stream: R,
    callback: std::sync::Arc<F>,
) -> Result<String, String>
where
    R: tokio::io::AsyncRead + Unpin,
    F: Fn(BsRoformerCppProgress) + Send + Sync + 'static,
{
    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::new();
    let mut log = String::new();
    loop {
        buffer.clear();
        if reader
            .read_until(b'\r', &mut buffer)
            .await
            .map_err(|e| e.to_string())?
            == 0
        {
            break;
        }
        let text = String::from_utf8_lossy(&buffer);
        log.push_str(&text);
        if let Some(before) = text.split('%').next().filter(|_| text.contains('%')) {
            if let Some(percent) = before
                .split_whitespace()
                .last()
                .and_then(|s| s.parse::<f64>().ok())
            {
                callback(BsRoformerCppProgress {
                    stage: "Processing".into(),
                    percent: percent.min(99.0),
                    message: "Separating audio...".into(),
                    stem_index: None,
                });
            }
        }
    }
    Ok(log)
}

const STEM_NAMES: [&str; 6] = ["bass", "drums", "other", "vocals", "guitar", "piano"];

fn parse_multi_stem_output(
    output_dir: &Path,
    output_base: &Path,
) -> Result<Vec<(String, String)>, String> {
    let base_name = output_base
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid base output name")?;

    let mut stems: Vec<(u32, String, String)> = Vec::new();

    for entry in
        fs::read_dir(output_dir).map_err(|e| format!("Failed to read output directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("wav") {
            let file_stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("Invalid stem filename")?
                .to_string();

            let prefix = format!("{}_stem_", base_name);
            if file_stem.starts_with(&prefix) {
                let suffix = &file_stem[prefix.len()..];
                if let Ok(idx) = suffix.parse::<u32>() {
                    let name = if idx < STEM_NAMES.len() as u32 {
                        STEM_NAMES[idx as usize].to_string()
                    } else {
                        format!("stem_{}", idx)
                    };
                    let path = path.to_str().ok_or("Invalid path")?.to_string();
                    stems.push((idx, name, path));
                }
            }
        }
    }

    stems.sort_by_key(|a| a.0);

    let result: Vec<(String, String)> = stems
        .into_iter()
        .map(|(_, name, path)| (name, path))
        .collect();

    if result.len() != 6 {
        return Err(format!("Expected 6 stems, found {}", result.len()));
    }
    Ok(result)
}
