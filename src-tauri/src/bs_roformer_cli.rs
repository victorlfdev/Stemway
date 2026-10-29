use std::path::Path;
use std::process::Stdio;

#[derive(Debug, Clone, serde::Serialize)]
pub struct BsRoformerProgress {
    pub stage: String,
    pub percent: f64,
    pub message: String,
}

pub async fn run_bs_roformer_cli_with_progress<F>(
    input_dir: &Path,
    output_dir: &Path,
    progress_callback: F,
) -> Result<Vec<(String, String)>, String>
where
    F: Fn(BsRoformerProgress) + Send + Sync + 'static,
{
    progress_callback(BsRoformerProgress {
        stage: "Preparing".into(),
        percent: 0.0,
        message:
            "Preparing BS-RoFormer (first use installs dependencies and downloads the model)..."
                .into(),
    });
    let executable = tauri::async_runtime::spawn_blocking(|| {
        crate::python_tools::ensure_tool("bs-roformer-infer", "bs-roformer-infer==0.1.5")
    })
    .await
    .map_err(|e| e.to_string())??;
    progress_callback(BsRoformerProgress {
        stage: "Processing".into(),
        percent: 5.0,
        message: "Separating audio. The first run may download the model (~700 MB)...".into(),
    });
    let mut cmd = tokio::process::Command::new(executable);
    cmd.env(
        "BS_ROFORMER_MODELS_PATH",
        crate::models::model_dir("bs-roformer")?,
    );
    cmd.arg("--input_folder")
        .arg(input_dir)
        .arg("--store_dir")
        .arg(output_dir)
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
    let process = cmd
        .spawn()
        .map_err(|e| format!("Failed to start BS-RoFormer: {e}"))?;
    let _guard = crate::jobs::register(process.id().ok_or("Processo indisponível")?)?;
    let output = process
        .wait_with_output()
        .await
        .map_err(|e| e.to_string())?;
    crate::jobs::check()?;
    if !output.status.success() {
        return Err(format!(
            "BS-RoFormer failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stems = normalize_output(output_dir)?;
    progress_callback(BsRoformerProgress {
        stage: "Complete".into(),
        percent: 100.0,
        message: "Stems separated!".into(),
    });
    Ok(stems)
}

fn normalize_output(output_dir: &Path) -> Result<Vec<(String, String)>, String> {
    let mut stems = Vec::new();
    for entry in std::fs::read_dir(output_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("wav") {
            continue;
        }
        let filename = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
        // Match the suffix: track names can themselves contain underscores or stem names.
        if let Some(name) = ["vocals", "drums", "bass", "guitar", "piano", "other"]
            .into_iter()
            .find(|name| filename == *name || filename.ends_with(&format!("_{name}")))
        {
            let dest = output_dir.join(format!("{name}.wav"));
            if path != dest {
                std::fs::rename(&path, &dest).map_err(|e| e.to_string())?;
            }
            stems.push((name.to_owned(), dest.to_string_lossy().into_owned()));
        }
    }
    if stems.len() != 6 {
        return Err(format!(
            "BS-RoFormer produced {} of 6 expected stems",
            stems.len()
        ));
    }
    stems.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(stems)
}

#[cfg(test)]
mod tests {
    #[test]
    fn recognizes_suffixes_and_excludes_instrumental_mix() {
        let dir = std::env::temp_dir().join(format!("stemway-bs-output-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for stem in [
            "vocals",
            "drums",
            "bass",
            "guitar",
            "piano",
            "other",
            "instrumental",
        ] {
            std::fs::write(dir.join(format!("my_vocals_song_{stem}.wav")), b"test").unwrap();
        }
        let result = super::normalize_output(&dir).unwrap();
        assert_eq!(result.len(), 6);
        assert!(result
            .iter()
            .all(|(name, path)| Path::new(path).file_stem().unwrap() == name.as_str()));
        std::fs::remove_dir_all(dir).unwrap();
    }
    use std::path::Path;
}
