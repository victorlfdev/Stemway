use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug, Clone, serde::Serialize)]
pub struct DemucsProgress {
    pub stage: String,
    pub percent: f64,
    pub message: String,
}

pub fn is_demucs_installed() -> bool {
    matches!(
        std::process::Command::new("demucs")
            .args(["--help"])
            .output()
            .map(|o| o.status.success()),
        Ok(true)
    )
}

fn install_via_pipx() -> Result<(), String> {
    let output = std::process::Command::new("pipx")
        .args(["install", "demucs"])
        .output()
        .map_err(|e| format!("pipx install failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pipx install demucs failed: {}", stderr));
    }

    Ok(())
}

fn install_via_pip() -> Result<(), String> {
    let output = std::process::Command::new("python3")
        .args(["-m", "pip", "install", "--user", "demucs"])
        .output()
        .map_err(|e| format!("pip install failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pip install demucs failed: {}", stderr));
    }

    Ok(())
}

pub fn ensure_demucs_installed() -> Result<PathBuf, String> {
    if is_demucs_installed() {
        return Ok(PathBuf::from("demucs"));
    }

    eprintln!("demucs not found. Installing via pipx...");
    if let Err(e) = install_via_pipx() {
        eprintln!("pipx install failed: {}. Trying pip...", e);
        if let Err(e2) = install_via_pip() {
            return Err(format!("Both pipx and pip failed: {}", e2));
        }
    }

    Ok(PathBuf::from("demucs"))
}

pub fn run_demucs_cli(input_path: &Path, output_dir: &Path) -> Result<String, String> {
    let output = std::process::Command::new("demucs")
        .args([
            "-n", "htdemucs_6s",
            "-o", output_dir.to_str().ok_or("Invalid output dir")?,
            input_path.to_str().ok_or("Invalid input path")?,
        ])
        .output()
        .map_err(|e| format!("Failed to run demucs: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return Err(format!("demucs failed: {}", stderr));
    }

    Ok(stdout.to_string())
}

pub async fn run_demucs_cli_with_progress<F>(
    input_path: &Path,
    output_dir: &Path,
    progress_callback: F,
) -> Result<String, String>
where
    F: Fn(DemucsProgress) + Send + Sync + 'static,
{
    let stdout_pipe = Stdio::piped();

    let mut cmd = TokioCommand::new("demucs");
    cmd.args([
        "-n", "htdemucs_6s",
        "-o", output_dir.to_str().ok_or("Invalid output dir")?,
        input_path.to_str().ok_or("Invalid input path")?,
    ])
    .stdout(stdout_pipe)
    .stderr(Stdio::piped())
    .kill_on_drop(true);

    let mut process = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn demucs process: {}", e))?;

    let stderr = process
        .stderr
        .take()
        .ok_or("Failed to take stderr handle")?;

    let progress_callback = std::sync::Arc::new(progress_callback);
    let overall_progress = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0));

    let overall = std::sync::Arc::clone(&overall_progress);
    let cb = std::sync::Arc::clone(&progress_callback);
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Ok(Some(line_result)) = lines.next_line().await {
            let stripped: String = line_result
                .chars()
                .filter(|c| !c.is_ascii_control())
                .collect();

                    if stripped.contains("Separating track") || stripped.contains("separating") {
                        (cb)(DemucsProgress {
                            stage: "Separating".to_string(),
                            percent: 0.0,
                            message: "Separating audio...".to_string(),
                        });
                    }

                    if stripped.contains('%') && stripped.contains('|') {
                        if let Some(pct_str) = stripped.split('%').next().and_then(|s| {
                            s.trim().split(' ').last().and_then(|s| s.trim().parse::<f64>().ok())
                        }) {
                            let pct = (pct_str / 100.0 * 90.0).min(89.0);
                            (cb)(DemucsProgress {
                                stage: "Processing".to_string(),
                                percent: pct,
                                message: format!("Processing: {:.0}%", pct_str),
                            });
                        }
                    }
                }
    });

    let overall_clone = std::sync::Arc::clone(&overall);
    let cb_clone = std::sync::Arc::clone(&progress_callback);
    let (stdout_tx, mut stdout_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    {
        let stdout_tx_clone = stdout_tx;
        tokio::spawn(async move {
            let result = process
                .wait_with_output()
                .await
                .map_err(|e| format!("Failed to wait for demucs: {}", e));

            match result {
                Ok(output) => {
                    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
                    let status = output.status;
                    if status.success() {
                        (cb_clone)(DemucsProgress {
                            stage: "Complete".to_string(),
                            percent: 100.0,
                            message: "Stems separated!".to_string(),
                        });
                    } else {
                        (cb_clone)(DemucsProgress {
                            stage: "Error".to_string(),
                            percent: 0.0,
                            message: format!("demucs exited with status: {}", status),
                        });
                    }
                    let _ = stdout_tx_clone.send(stdout_str);
                }
                Err(e) => {
                    (cb_clone)(DemucsProgress {
                        stage: "Error".to_string(),
                        percent: 0.0,
                        message: e.to_string(),
                    });
                }
            }
            overall_clone.store(1, std::sync::atomic::Ordering::SeqCst);
        });
    }

    let stdout = match stdout_rx.recv().await {
        Some(s) if !s.is_empty() => s,
        _ => String::new(),
    };

    if !stdout.is_empty() {
        eprintln!("{}", stdout);
    }

    let total_time = overall.load(std::sync::atomic::Ordering::SeqCst);
    if total_time == 0 {
        return Err("demucs process did not complete".to_string());
    }

    Ok(stdout)
}
