use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug, Clone, serde::Serialize)]
pub struct BsRoformerProgress {
    pub stage: String,
    pub percent: f64,
    pub message: String,
}

pub fn is_bs_roformer_installed() -> bool {
    matches!(
        std::process::Command::new("bs-roformer-infer")
            .args(["--help"])
            .output()
            .map(|o| o.status.success()),
        Ok(true)
    )
}

fn install_via_pipx() -> Result<(), String> {
    let output = std::process::Command::new("pipx")
        .args(["install", "bs-roformer-infer"])
        .output()
        .map_err(|e| format!("pipx install failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pipx install bs-roformer-infer failed: {}", stderr));
    }

    Ok(())
}

fn install_via_pip() -> Result<(), String> {
    let output = std::process::Command::new("pip3")
        .args(["install", "--user", "bs-roformer-infer"])
        .output()
        .map_err(|e| format!("pip install failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pip install bs-roformer-infer failed: {}", stderr));
    }

    Ok(())
}

pub fn ensure_bs_roformer_installed() -> Result<PathBuf, String> {
    if is_bs_roformer_installed() {
        return Ok(PathBuf::from("bs-roformer-infer"));
    }

    eprintln!("bs-roformer-infer not found. Installing via pipx...");
    if let Err(e) = install_via_pipx() {
        eprintln!("pipx install failed: {}. Trying pip...", e);
        if let Err(e2) = install_via_pip() {
            return Err(format!("Both pipx and pip failed: {}", e2));
        }
    }

    Ok(PathBuf::from("bs-roformer-infer"))
}

fn check_and_resample_to_44100(input_file: &Path) -> Result<PathBuf, String> {
    let output = std::process::Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "stream=sample_rate", "-of", "default=noprint_wrappers=1:nokey=1", input_file.to_str().ok_or("Invalid input path")?])
        .output()
        .map_err(|e| format!("Failed to run ffprobe: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[resample] ffprobe failed: {}", stderr);
        return Ok(input_file.to_path_buf());
    }

    let sample_rate = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u32>()
        .unwrap_or(0);

    if sample_rate == 44100 {
        eprintln!("[resample] Sample rate is already 44100 Hz");
        return Ok(input_file.to_path_buf());
    }

    eprintln!("[resample] Sample rate {} Hz != 44100 Hz. Resampling...", sample_rate);

    let temp_file = input_file.with_file_name("temp_resampled.wav");
    
    let output = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i", input_file.to_str().ok_or("Invalid input path")?,
            "-ar", "44100",
            temp_file.to_str().ok_or("Invalid temp path")?,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg resample failed: {}", stderr));
    }

    std::fs::remove_file(input_file).ok();
    std::fs::rename(&temp_file, input_file).ok();
    eprintln!("[resample] Resampled and replaced: {}", input_file.display());
    Ok(input_file.to_path_buf())
}

pub async fn run_bs_roformer_cli_with_progress<F>(
    input_dir: &Path,
    output_dir: &Path,
    progress_callback: F,
) -> Result<Vec<(String, String)>, String>
where
    F: Fn(BsRoformerProgress) + Send + Sync + 'static,
{
    let input_dir_str = input_dir.to_str().ok_or("Invalid input dir")?;
    let output_dir_str = output_dir.to_str().ok_or("Invalid output dir")?;

    eprintln!("=== BS-RoFormer Python CLI ===");
    eprintln!("Input dir: {}", input_dir_str);
    eprintln!("Output dir: {}", output_dir_str);

    let wav_files: Vec<_> = std::fs::read_dir(input_dir)
        .map_err(|e| format!("Failed to read input dir: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|s| s.to_str()) == Some("wav")
        })
        .collect();

    eprintln!("[bs-roformer] Found {} .wav files in input dir", wav_files.len());
    for entry in &wav_files {
        eprintln!("[bs-roformer]   {}", entry.path().display());
    }

    if wav_files.is_empty() {
        return Err("No .wav files found in input directory".to_string());
    }

    let first_wav = &wav_files[0];
    let input_file = &first_wav.path();
    let resampled_file = check_and_resample_to_44100(input_file)?;

    let input_to_use = if resampled_file != *input_file {
        resampled_file.to_str().ok_or("Invalid resampled path")?
    } else {
        input_file.to_str().ok_or("Invalid input path")?
    };

    eprintln!("[bs-roformer] Using input file: {}", input_to_use);

    let mut cmd = TokioCommand::new("bs-roformer-infer");
    cmd.args([
        "--input_folder", input_dir_str,
        "--store_dir", output_dir_str,
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .kill_on_drop(true);

    eprintln!("[bs-roformer] Command: bs-roformer-infer --input_folder {} --store_dir {}", input_dir_str, output_dir_str);

    let mut process = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn bs-roformer-infer process: {}", e))?;

    let stderr = process
        .stderr
        .take()
        .ok_or("Failed to take stderr handle")?;

    let progress_callback = std::sync::Arc::new(progress_callback);
    let overall_progress = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0));

    let cb = std::sync::Arc::clone(&progress_callback);
    let stderr_task = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        loop {
            match lines.next_line().await {
                Ok(Some(line_result)) => {
                    let stripped: String = line_result
                        .chars()
                        .filter(|c| !c.is_ascii_control())
                        .collect();

                    eprintln!("[bs-roformer] {}", stripped);

                    if stripped.contains("downloading") || stripped.contains("Downloading") {
                        let _ = (cb)(BsRoformerProgress {
                            stage: "Downloading".to_string(),
                            percent: 10.0,
                            message: "Downloading model checkpoint...".to_string(),
                        });
                    }

                    if stripped.contains("Processing") || stripped.contains("processing") {
                        let _ = (cb)(BsRoformerProgress {
                            stage: "Processing".to_string(),
                            percent: 30.0,
                            message: "Processing audio...".to_string(),
                        });
                    }

                    if stripped.contains("Saving") || stripped.contains("saving") {
                        let _ = (cb)(BsRoformerProgress {
                            stage: "Saving".to_string(),
                            percent: 85.0,
                            message: "Saving stems...".to_string(),
                        });
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    eprintln!("stderr read error: {}", e);
                    break;
                }
            }
        }
    });

    let overall_clone = std::sync::Arc::clone(&overall_progress);
    let cb_clone = std::sync::Arc::clone(&progress_callback);
    let wait_task = tokio::spawn(async move {
        let result = process
            .wait_with_output()
            .await
            .map_err(|e| format!("Failed to wait for bs-roformer-infer: {}", e));

        match result {
            Ok(output) => {
                let status = output.status;
                let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

                if !stdout_str.is_empty() {
                    eprintln!("[bs-roformer] stdout:\n{}", stdout_str);
                }
                if !stderr_str.is_empty() {
                    eprintln!("[bs-roformer] stderr:\n{}", stderr_str);
                }

                if status.success() {
                    (cb_clone)(BsRoformerProgress {
                        stage: "Complete".to_string(),
                        percent: 100.0,
                        message: "Stems separated!".to_string(),
                    });
                } else {
                    (cb_clone)(BsRoformerProgress {
                        stage: "Error".to_string(),
                        percent: 0.0,
                        message: format!("bs-roformer-infer exited with status {}: {}", status, stderr_str),
                    });
                }
            }
            Err(e) => {
                (cb_clone)(BsRoformerProgress {
                    stage: "Error".to_string(),
                    percent: 0.0,
                    message: e.to_string(),
                });
            }
        }
        overall_clone.store(1, std::sync::atomic::Ordering::SeqCst);
    });

    let _ = tokio::join!(stderr_task, wait_task);

    let total_time = overall_progress.load(std::sync::atomic::Ordering::SeqCst);
    if total_time == 0 {
        return Err("bs-roformer-infer process did not complete".to_string());
    }

    let output_files: Vec<_> = std::fs::read_dir(output_dir)
        .map_err(|e| format!("Failed to read output dir: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|s| s.to_str()) == Some("wav")
        })
        .collect();

    eprintln!("[bs-roformer] Found {} .wav files in output dir: {}", output_files.len(), output_dir_str);
    for entry in &output_files {
        eprintln!("[bs-roformer]   {} (is_dir: {})", entry.path().display(), entry.path().is_dir());
    }

    let mut stems: Vec<(String, String)> = Vec::new();

    for entry in output_files {
        let path = entry.path();
        let stem_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid stem filename")?
            .to_string();

        let file_path = path
            .to_str()
            .ok_or("Invalid file path")?
            .to_string();

        if stem_name.contains('_') {
            let parts: Vec<&str> = stem_name.split('_').collect();
            if parts.len() >= 2 {
                let original_prefix = parts[0];
                let stem = parts[1];
                let clean_name = if original_prefix.contains("vocals") || original_prefix.contains("drums") || original_prefix.contains("bass") || original_prefix.contains("other") || original_prefix.contains("guitar") || original_prefix.contains("piano") {
                    stem
                } else {
                    &stem_name
                };
                eprintln!("[bs-roformer] Parsed stem: {} -> {} ({})", stem_name, clean_name, path.display());
                stems.push((clean_name.to_string(), file_path));
            }
        } else {
            stems.push((stem_name, file_path));
        }
    }

    stems.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    eprintln!("[bs-roformer] Returning {} stems", stems.len());
    for (name, path) in &stems {
        eprintln!("[bs-roformer]   {} -> {}", name, path);
    }

    Ok(stems)
}
