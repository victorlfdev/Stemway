use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use tauri::Emitter as _;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ProcessResult {
    pub paths: Vec<String>,
    pub name: String,
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub file_size_bytes: u64,
}

#[derive(serde::Serialize)]
pub struct StemFileData {
    pub data: String,
}

#[tauri::command]
pub async fn read_stem_as_base64(path: String) -> Result<StemFileData, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("Failed to read stem file: {}", e))?;
    let mime = "audio/wav";
    let encoded = BASE64.encode(&bytes);
    let data = format!("data:{mime};base64,{}", encoded);
    Ok(StemFileData { data })
}

#[tauri::command]
pub async fn process_file(
    path: String,
    app: tauri::AppHandle,
) -> Result<ProcessResult, String> {
    let temp_dir = directories::ProjectDirs::from("", "", "stem-separator")
        .map(|d| d.cache_dir().to_path_buf())
        .unwrap_or_else(|| std::env::temp_dir());
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let output_base = temp_dir.join("stem-output");
    std::fs::create_dir_all(&output_base).map_err(|e| format!("Failed to create output dir: {}", e))?;

    crate::demucs_cli::ensure_demucs_installed()?;

    let input_path = std::path::Path::new(&path);

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<crate::demucs_cli::DemucsProgress>();

    // Spawn task to forward progress events to frontend
    {
        let app_forward = app.clone();
        tokio::spawn(async move {
            let mut rx = rx;
            while let Some(progress) = rx.recv().await {
                let _ = app_forward.emit("demucs_progress", &progress);
            }
        });
    }

    // Run demucs with progress callback
    let result = crate::demucs_cli::run_demucs_cli_with_progress(
        input_path,
        &output_base,
        move |prog: crate::demucs_cli::DemucsProgress| {
            let _ = tx.send(prog);
        },
    )
    .await?;

    let _ = result;

    let track_name = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let stem_pairs = crate::demucs_output::parse_demucs_output(
        output_base.to_str().ok_or("Invalid output dir")?,
        &track_name,
    )?;

    let paths: Vec<String> = stem_pairs.iter().map(|(_, p)| p.clone()).collect();

    let (duration_secs, sample_rate, channels, file_size_bytes) =
        if let Some(first_stem) = stem_pairs.first() {
            crate::info::probe_wav(&first_stem.1)?
        } else {
            (0.0, 44100, 2, 0)
        };

    Ok(ProcessResult {
        paths,
        name: track_name,
        duration_secs,
        sample_rate,
        channels,
        file_size_bytes,
    })
}
