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

pub async fn run_bs_roformer_cli_with_progress<F>(
    input_path: &Path,
    output_dir: &Path,
    progress_callback: F,
) -> Result<String, String>
where
    F: Fn(BsRoformerProgress) + Send + Sync + 'static,
{
    let stdout_pipe = Stdio::piped();

    let mut cmd = TokioCommand::new("bs-roformer-infer");
    cmd.args([
        "--input_folder", input_path.to_str().ok_or("Invalid input path")?,
        "--store_dir", output_dir.to_str().ok_or("Invalid output dir")?,
    ])
    .stdout(stdout_pipe)
    .stderr(Stdio::piped())
    .kill_on_drop(true);

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
                if status.success() {
                    (cb_clone)(BsRoformerProgress {
                        stage: "Complete".to_string(),
                        percent: 100.0,
                        message: "Stems separated!".to_string(),
                    });
                } else {
                    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
                    (cb_clone)(BsRoformerProgress {
                        stage: "Error".to_string(),
                        percent: 0.0,
                        message: format!("bs-roformer-infer exited: {}", stderr_str),
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

    Ok(String::new())
}
