use std::fs;
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncBufReadExt, BufReader};

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

const BSROFORMER_BINARY_NAME: &str = "bs_roformer-cli";
const BSROFORMER_MODEL_URL: &str = "https://huggingface.co/victorlfdev/bs-roformer-multi-q8/resolve/main/bs-roformer-multi-q8.gguf";
const BSROFORMER_BINARY_URL_LINUX_VULKAN: &str = "https://github.com/victorlfdev/Fork-BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-linux-vulkan.tar.xz";
const BSROFORMER_BINARY_URL_LINUX_CPU: &str = "https://github.com/chenmozhijin/BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-linux-x64-cpu.tar.xz";
const BSROFORMER_BINARY_URL_LINUX_CUDA11: &str = "https://github.com/chenmozhijin/BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-linux-cuda-11.8.0.tar.xz";
const BSROFORMER_BINARY_URL_LINUX_CUDA12: &str = "https://github.com/chenmozhijin/BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-linux-cuda-12.9.1.tar.xz";
const BSROFORMER_BINARY_URL_WINDOWS_VULKAN: &str = "https://github.com/victorlfdev/Fork-BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-windows-vulkan.zip";
const BSROFORMER_BINARY_URL_WINDOWS_CPU: &str = "https://github.com/chenmozhijin/BSRoformer.cpp/releases/download/v0.1.0/BSRoformer-windows-cpu.zip";
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
            .args(["-c", "lspci 2>/dev/null | grep -i -E 'vga|3d|display' || true"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        if output.contains("NVIDIA") || output.contains("GeForce") || output.contains("RTX") || output.contains("Tesla") {
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

        if output.contains("AMD") || output.contains("ATi") || output.contains("Radeon") || output.contains("RX ") {
            let vulkan_lib = std::process::Command::new("sh")
                .args(["-c", "find /usr/lib /usr/lib64 -name 'libvulkan*' 2>/dev/null | head -1 || true"])
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
        let output = std::process::Command::new("wmic")
            .args(["path", "win32_videocontroller", "get", "name"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        if output.contains("NVIDIA") || output.contains("GeForce") || output.contains("RTX") {
            return BsRoformerBackend::Cuda12;
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
    let dirs = directories::ProjectDirs::from("", "", "stem-separator")
        .map(|d| d.cache_dir().to_path_buf())
        .unwrap_or_else(|| std::env::temp_dir());

    let binary_dir = dirs.join("binaries").join("bs-roformer");
    let model_dir = dirs.join("models").join("bs-roformer");

    fs::create_dir_all(&binary_dir).map_err(|e| format!("Failed to create binary dir: {}", e))?;
    fs::create_dir_all(&model_dir).map_err(|e| format!("Failed to create model dir: {}", e))?;

    Ok((dirs, binary_dir, model_dir))
}

pub async fn get_binary_path() -> Result<PathBuf, String> {
    let (cache_dir, binary_dir, _model_dir) = get_cache_dirs()?;
    let backend = detect_gpu_backend();
    let binary_path = binary_dir.join(BSROFORMER_BINARY_NAME);

    eprintln!("BSRoformer backend: {}", get_backend_name(&backend));

    // Redownload if binary exists but is empty or corrupted
    if binary_path.exists() {
        let metadata = std::fs::metadata(&binary_path).map_err(|e| e.to_string())?;
        if metadata.len() > 100_000 {
            return Ok(binary_path);
        }
        let _ = std::fs::remove_file(&binary_path);
    }

    let binary_url = get_binary_url(&backend);

    eprintln!("Downloading BSRoformer.cpp binary (backend: {})...", get_backend_name(&backend));
    download_and_extract_binary(binary_url, &binary_dir, &cache_dir, &backend).await?;

    if !binary_path.exists() {
        return Err("Failed to extract BSRoformer binary".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::fs::set_permissions(
            &binary_path,
            std::fs::Permissions::from_mode(0o755),
        );
    }

    Ok(binary_path)
}

async fn download_to_file(url: &str, dest: &Path) -> Result<(), String> {
    let output = std::process::Command::new("curl")
        .args(["-sL", "--output", dest.to_str().ok_or("Invalid dest path")?, url])
        .output()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        return Err(format!(
            "curl failed (status: {}): stderr={}, stdout={}",
            output.status, stderr, stdout
        ));
    }

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

    let output = std::process::Command::new("tar")
        .args(["-xJf", "temp_download.tar.xz"])
        .current_dir(dest_dir)
        .output()
        .map_err(|e| format!("Failed to run tar: {}", e))?;

    let _ = std::fs::remove_file(&temp_tar);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("tar extract failed: {}", stderr));
    }

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
            let _ = std::os::unix::fs::symlink(target_name, &link_path);
        }
    }

    Ok(())
}

async fn extract_zip_all(url: &str, dest_dir: &Path) -> Result<(), String> {
    let temp_zip = dest_dir.join("temp_download.zip");
    download_to_file(url, &temp_zip).await?;

    let output = std::process::Command::new("unzip")
        .args(["-o", "temp_download.zip", "-d", "."])
        .current_dir(dest_dir)
        .output()
        .map_err(|e| format!("Failed to run unzip: {}", e))?;

    let _ = std::fs::remove_file(&temp_zip);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("unzip extract failed: {}", stderr));
    }

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

    std::fs::rename(&temp_file, dest)
        .map_err(|e| format!("Failed to rename model file: {}", e))?;

    Ok(())
}

pub fn check_and_resample_to_44100(input_path: &Path) -> Result<PathBuf, String> {
    let probe = std::process::Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-show_entries", "stream=sample_rate",
            "-of", "csv=p=0",
            input_path.to_str().ok_or("Invalid path")?,
        ])
        .output()
        .map_err(|e| format!("Failed to probe audio file (is ffprobe installed?). Error: {}", e))?;

    if !probe.status.success() {
        return Err("Failed to probe audio file. Is ffmpeg/ffprobe installed?".to_string());
    }

    let sample_rate = String::from_utf8(probe.stdout)
        .map_err(|e| format!("Invalid ffprobe output: {}", e))?
        .trim()
        .parse::<u32>()
        .unwrap_or(0);

    if sample_rate == 44100 {
        return Ok(input_path.to_path_buf());
    }

    eprintln!(
        "Input sample rate {} Hz != 44100 Hz required by BSRoformer. Resampling to 44100 Hz...",
        sample_rate
    );

    let temp_path = input_path
        .parent()
        .map(|p| p.join(format!("temp_44100_{}.wav", std::process::id())))
        .ok_or("Cannot create temp file")?;

    let resample = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i", input_path.to_str().ok_or("Invalid input path")?,
            "-ar", "44100",
            temp_path.to_str().ok_or("Invalid temp path")?,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffmpeg (is ffmpeg installed?). Error: {}", e))?;

    if !resample.status.success() {
        let stderr = String::from_utf8_lossy(&resample.stderr);
        return Err(format!("ffmpeg resample failed: {}", stderr));
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
    let binary = get_binary_path().await?;
    let model = get_model_path().await?;

    let output_dir_clone = output_dir.to_path_buf();
    let output_file = output_dir_clone.join("output.wav");

    let input_for_binary = check_and_resample_to_44100(input_path)?;
    let is_temp_file = input_for_binary != input_path;

    let binary_dir = binary
        .parent()
        .ok_or("Cannot determine binary directory")?;

    eprintln!("BSRoformer binary: {}", binary.display());
    eprintln!("BSRoformer model: {}", model.display());
    eprintln!("BSRoformer output base: {}", output_file.display());
    eprintln!("BSRoformer input file (after resample check): {}", input_for_binary.display());
    eprintln!("Binary dir: {}", binary_dir.display());

    let binary_dir_clone = binary_dir.to_path_buf();
    let binary_clone = binary.clone();

    let mut cmd = TokioCommand::new(&binary_clone);
    let cmd_args: Vec<String> = vec![
        model.to_str().ok_or("Invalid model path")?.to_string(),
        input_for_binary.to_str().ok_or("Invalid input path")?.to_string(),
        output_file.to_str().ok_or("Invalid output path")?.to_string(),
        "--overlap".to_string(),
        "2".to_string(),
    ];

    cmd.args(&cmd_args)
    .current_dir(&binary_dir_clone)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .kill_on_drop(true);

    #[cfg(target_os = "linux")]
    cmd.env("LD_LIBRARY_PATH", &binary_dir_clone);

    #[cfg(not(target_os = "linux"))]
    {
        cmd.env("PATH", format!("{}:{}", binary_dir_clone.display(), std::env::var("PATH").unwrap_or_default()));
        if cfg!(target_os = "windows") {
            cmd.env("SYSTEMROOT", std::env::var("SYSTEMROOT").unwrap_or_default());
        }
    }

    let mut process = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn BSRoformer binary: {}. binary={}, binary_dir={}, LD_LIBRARY_PATH={}, stderr_hint=Run 'ldd {}' to see which libraries are missing", e, binary.display(), binary_dir.display(), binary_dir.display(), binary.display()))?;

    let stderr_handle = process
        .stderr
        .take()
        .ok_or("Failed to take stderr handle")?;

    let stdout_handle = process
        .stdout
        .take()
        .ok_or("Failed to take stdout handle")?;

    let overall_progress = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0));
    let cb = std::sync::Arc::new(progress_callback);
    let stderr_shared = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let stderr_shared_clone = std::sync::Arc::clone(&stderr_shared);
    let overall_clone = std::sync::Arc::clone(&overall_progress);
    let cb_clone = std::sync::Arc::clone(&cb);
    let stderr_shared_err = std::sync::Arc::clone(&stderr_shared);
    let cb_err = std::sync::Arc::clone(&cb);

    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr_handle);
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader
                .read_line(&mut line)
                .await
                .map_err(|e| format!("Failed to read stderr: {}", e));

            match bytes_read {
                Ok(0) => break,
                Ok(_) => {
                    let stripped: String = line
                        .chars()
                        .filter(|c| !c.is_ascii_control())
                        .collect();
                    let mut full_stderr = stderr_shared_clone.lock().unwrap();
                    *full_stderr += &stripped;

                    let stage_clone = std::sync::Arc::clone(&cb_clone);
                    if stripped.contains("Processing") || stripped.contains("processing") {
                        let _ = (stage_clone)(BsRoformerCppProgress {
                            stage: "Processing".to_string(),
                            percent: 50.0,
                            message: "Running inference...".to_string(),
                            stem_index: None,
                        });
                    }
                    if stripped.contains("Saving") || stripped.contains("saving") {
                        let stem_idx = extract_stem_index_from_line(&stripped);
                        let stem_msg = if let Some(idx) = stem_idx {
                            format!("Saving stem {}", idx)
                        } else {
                            "Saving output...".to_string()
                        };
                        let _ = (stage_clone)(BsRoformerCppProgress {
                            stage: "Saving".to_string(),
                            percent: 90.0,
                            message: stem_msg,
                            stem_index: stem_idx,
                        });
                    }
                    if stripped.contains("Error") || stripped.contains("error") {
                        let _ = (stage_clone)(BsRoformerCppProgress {
                            stage: "Error".to_string(),
                            percent: 0.0,
                            message: format!("BSRoformer error: {}", stripped.trim()),
                            stem_index: None,
                        });
                    }
                }
                Err(e) => {
                    eprintln!("BSRoformer stderr read error: {}", e);
                    break;
                }
            }
        }
    });

    let stdout_shared = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let stdout_shared_clone = std::sync::Arc::clone(&stdout_shared);
    let stdout_err = std::sync::Arc::clone(&stdout_shared);

    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout_handle);
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader
                .read_line(&mut line)
                .await
                .map_err(|e| format!("Failed to read stdout: {}", e));

            match bytes_read {
                Ok(0) => break,
                Ok(_) => {
                    let stripped: String = line
                        .chars()
                        .filter(|c| !c.is_ascii_control())
                        .collect();
                    let mut full_stdout = stdout_shared_clone.lock().unwrap();
                    *full_stdout += &stripped;
                    eprintln!("BSRoformer stdout: {}", stripped.trim());
                }
                Err(e) => {
                    eprintln!("BSRoformer stdout read error: {}", e);
                    break;
                }
            }
        }
    });

    let wait_task = tokio::spawn(async move {
        let result = process
            .wait_with_output()
            .await
            .map_err(|e| format!("Failed to wait for BSRoformer: {}", e));

        let total_time = overall_clone.load(std::sync::atomic::Ordering::SeqCst);
        if total_time == 1 {
            return Ok::<Vec<(String, String)>, String>(Vec::new());
        }

        match result {
            Ok(output) => {
                let status = output.status;
                let stderr_str = stderr_shared_err.lock().unwrap().clone();
                let _stdout_str = stdout_err.lock().unwrap().clone();

                eprintln!("BSRoformer exited with status: {}", status);
                eprintln!("BSRoformer full stderr: {}", stderr_str);

                if status.success() {
                    let stem_pairs = parse_multi_stem_output(&output_dir_clone, &output_file);
                    if let Ok(ref pairs) = stem_pairs {
                        let total = pairs.len();
                        if total > 0 {
                            (cb_err)(BsRoformerCppProgress {
                                stage: "Complete".to_string(),
                                percent: 100.0,
                                message: format!("Separated {} stems!", total),
                                stem_index: Some((total - 1) as u32),
                            });
                        }
                        let pairs_to_rename: Vec<(String, String)> = pairs.iter().map(|(name, path)| (name.clone(), path.clone())).collect();
                        let mut new_pairs: Vec<(String, String)> = Vec::new();
                        for (name, path) in pairs_to_rename {
                            let new_path = output_dir_clone.join(format!("{}.wav", name));
                            if Path::new(&path).exists() && !Path::new(&new_path).exists() {
                                let _ = fs::rename(path, &new_path);
                            }
                            new_pairs.push((name.clone(), new_path.to_str().ok_or("Invalid path")?.to_string()));
                        }
                        return Ok(new_pairs);
                    }
                    overall_clone.store(1, std::sync::atomic::Ordering::SeqCst);
                    return stem_pairs;
                } else {
                    (cb_err)(BsRoformerCppProgress {
                        stage: "Error".to_string(),
                        percent: 0.0,
                        message: format!("BSRoformer exited with status: {}. stderr: {}", status, stderr_str),
                        stem_index: None,
                    });
                }
            }
            Err(e) => {
                (cb_err)(BsRoformerCppProgress {
                    stage: "Error".to_string(),
                    percent: 0.0,
                    message: e.to_string(),
                    stem_index: None,
                });
            }
        }
        overall_clone.store(1, std::sync::atomic::Ordering::SeqCst);
        Err("BSRoformer process did not complete successfully".to_string())
    });

    let wait_result = tokio::join!(stderr_task, stdout_task, wait_task);

    if is_temp_file {
        let _ = fs::remove_file(&input_for_binary);
    }

    match wait_result.2 {
        Ok(stems) => stems,
        Err(e) => Err(e.to_string()),
    }
}

fn extract_stem_index_from_line(line: &str) -> Option<u32> {
    let lower = line.to_lowercase();
    if let Some(idx) = lower.find("stem ") {
        let after = &lower[idx + 5..];
        let after = after.trim_start();
        if let Some(num) = after.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().ok() {
            return Some(num);
        }
    }
    None
}

const STEM_NAMES: [&str; 6] = [
    "bass",
    "drums",
    "other",
    "vocals",
    "guitar",
    "piano",
];

fn parse_multi_stem_output(
    output_dir: &Path,
    output_base: &Path,
) -> Result<Vec<(String, String)>, String> {
    let base_name = output_base
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid base output name")?;

    let mut stems: Vec<(u32, String, String)> = Vec::new();

    for entry in fs::read_dir(output_dir)
        .map_err(|e| format!("Failed to read output directory: {}", e))?
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

    stems.sort_by(|a, b| a.0.cmp(&b.0));

    let result: Vec<(String, String)> = stems.into_iter()
        .map(|(_, name, path)| (name, path))
        .collect();

    Ok(result)
}
