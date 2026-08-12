use std::path::{Path, PathBuf};
use std::process::Command;

pub fn is_demucs_installed() -> bool {
    matches!(
        Command::new("demucs")
            .args(["--help"])
            .output()
            .map(|o| o.status.success()),
        Ok(true)
    )
}

fn install_via_pipx() -> Result<(), String> {
    let output = Command::new("pipx")
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
    let output = Command::new("python3")
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
    let output = Command::new("demucs")
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
