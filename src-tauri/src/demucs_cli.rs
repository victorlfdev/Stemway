use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug, Clone, serde::Serialize)]
pub struct DemucsProgress {
    pub stage: String,
    pub percent: f64,
    pub message: String,
}

pub fn ensure_demucs_installed() -> Result<PathBuf, String> {
    crate::python_tools::ensure_tool("demucs", "demucs==4.0.1")
}

pub async fn run_demucs_cli_with_progress<F>(
    input_path: &Path,
    output_dir: &Path,
    progress_callback: F,
) -> Result<String, String>
where
    F: Fn(DemucsProgress) + Send + Sync + 'static,
{
    progress_callback(DemucsProgress {
        stage: "Preparing".into(),
        percent: 0.0,
        message: "Preparing HTDemucs (first use may install Python dependencies)...".into(),
    });
    let executable = tauri::async_runtime::spawn_blocking(ensure_demucs_installed)
        .await
        .map_err(|e| e.to_string())??;
    let mut cmd = tokio::process::Command::new(executable);
    cmd.env("TORCH_HOME", crate::models::model_dir("demucs")?);
    cmd.args(["-n", "htdemucs_6s", "--shifts", "1", "-d", "cpu", "-o"])
        .arg(output_dir)
        .arg(input_path)
        .env("PYTHONUNBUFFERED", "1")
        .env("PYTHONUTF8", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    #[cfg(unix)]
    cmd.process_group(0);
    crate::jobs::check()?;
    let mut process = cmd
        .spawn()
        .map_err(|e| format!("Failed to start HTDemucs: {e}"))?;
    let _guard = crate::jobs::register(process.id().ok_or("Processo indisponível")?)?;
    let stderr = process.stderr.take().ok_or("Missing HTDemucs stderr")?;
    let callback = std::sync::Arc::new(progress_callback);
    let cb = callback.clone();
    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        let mut errors = String::new();
        while let Some(line) = lines.next_line().await.map_err(|e| e.to_string())? {
            errors.push_str(&line);
            errors.push('\n');
            if let Some(before) = line.split('%').next().filter(|_| line.contains('%')) {
                if let Some(percent) = before
                    .split_whitespace()
                    .last()
                    .and_then(|p| p.parse::<f64>().ok())
                {
                    cb(DemucsProgress {
                        stage: "Processing".into(),
                        percent: percent.min(99.0),
                        message: "Separating audio...".into(),
                    });
                }
            }
        }
        Ok::<_, String>(errors)
    });
    let output = process
        .wait_with_output()
        .await
        .map_err(|e| e.to_string())?;
    let stderr = stderr_task.await.map_err(|e| e.to_string())??;
    crate::jobs::check()?;
    if !output.status.success() {
        return Err(format!("HTDemucs failed ({}): {stderr}", output.status));
    }
    callback(DemucsProgress {
        stage: "Complete".into(),
        percent: 100.0,
        message: "Stems separated!".into(),
    });
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
