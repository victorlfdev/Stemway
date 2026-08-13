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
    pub model: String,
}

#[derive(serde::Serialize)]
pub struct FileSizeResult {
    pub size: u64,
}

#[tauri::command]
pub fn get_file_size(path: String) -> Result<FileSizeResult, String> {
    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    Ok(FileSizeResult { size: metadata.len() })
}

#[derive(serde::Serialize)]
pub struct StemFileData {
    pub data: String,
}

#[derive(serde::Serialize)]
pub struct OpenFolderResult {
    pub success: bool,
    pub message: String,
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
pub async fn open_output_folder(path: String) -> Result<OpenFolderResult, String> {
    let path_ref = std::path::Path::new(&path);
    let folder_to_open = if path_ref.is_file() {
        path_ref.parent()
            .ok_or("Cannot determine parent folder")?
            .to_str()
            .ok_or("Invalid path")?
            .to_string()
    } else {
        path
    };

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("explorer")
        .arg(&folder_to_open)
        .spawn();

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open")
        .arg(&folder_to_open)
        .spawn();

    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open")
        .arg(&folder_to_open)
        .spawn();

    match result {
        Ok(_) => Ok(OpenFolderResult {
            success: true,
            message: folder_to_open,
        }),
        Err(e) => Err(format!("Failed to open folder: {}", e)),
    }
}

#[tauri::command]
pub async fn process_file(
    path: String,
    model: String,
    app: tauri::AppHandle,
) -> Result<ProcessResult, String> {
    let temp_dir = directories::ProjectDirs::from("", "", "stem-separator")
        .map(|d| d.cache_dir().to_path_buf())
        .unwrap_or_else(|| std::env::temp_dir());
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let output_base = temp_dir.join("stem-output");
    std::fs::create_dir_all(&output_base).map_err(|e| format!("Failed to create output dir: {}", e))?;

    let input_path = std::path::Path::new(&path);
    let track_name = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    match model.as_str() {
        "bs-roformer" => {
            crate::bs_roformer_cli::ensure_bs_roformer_installed()?;

            let bs_output_dir = output_base.join("bs-roformer-infer").join(&track_name);
            std::fs::create_dir_all(&bs_output_dir)
                .map_err(|e| format!("Failed to create bs-roformer output dir: {}", e))?;

            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<crate::bs_roformer_cli::BsRoformerProgress>();

            {
                let app_forward = app.clone();
                tokio::spawn(async move {
                    let mut rx = rx;
                    while let Some(progress) = rx.recv().await {
                        let _ = app_forward.emit("bs_roformer_progress", &progress);
                    }
                });
            }

            crate::bs_roformer_cli::run_bs_roformer_cli_with_progress(
                input_path,
                &bs_output_dir,
                move |prog: crate::bs_roformer_cli::BsRoformerProgress| {
                    let _ = tx.send(prog);
                },
            )
            .await?;

            let stem_pairs = crate::bs_roformer_output::parse_output(
                bs_output_dir.to_str().ok_or("Invalid bs-roformer output dir")?,
                "bs-roformer",
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
                model: "bs-roformer".to_string(),
            })
        }
        "bs-roformer-cpp" => {
            let cpp_output_dir = output_base.join("bs-roformer-cpp").join(&track_name);
            std::fs::create_dir_all(&cpp_output_dir)
                .map_err(|e| format!("Failed to create bs-roformer-cpp output dir: {}", e))?;

            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<crate::bs_roformer_cpp_cli::BsRoformerCppProgress>();

            {
                let app_forward = app.clone();
                tokio::spawn(async move {
                    let mut rx = rx;
                    while let Some(progress) = rx.recv().await {
                        let _ = app_forward.emit("bs_roformer_cpp_progress", &progress);
                    }
                });
            }

            let output_path = crate::bs_roformer_cpp_cli::run_bs_roformer_cpp_with_progress(
                input_path,
                &cpp_output_dir,
                move |prog: crate::bs_roformer_cpp_cli::BsRoformerCppProgress| {
                    let _ = tx.send(prog);
                },
            )
            .await?;

            let stem_pairs = vec![("vocals".to_string(), output_path.to_str().unwrap().to_string())];

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
                model: "bs-roformer-cpp".to_string(),
            })
        }
        _ => {
            crate::demucs_cli::ensure_demucs_installed()?;

            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<crate::demucs_cli::DemucsProgress>();

            {
                let app_forward = app.clone();
                tokio::spawn(async move {
                    let mut rx = rx;
                    while let Some(progress) = rx.recv().await {
                        let _ = app_forward.emit("demucs_progress", &progress);
                    }
                });
            }

            let _result = crate::demucs_cli::run_demucs_cli_with_progress(
                input_path,
                &output_base,
                move |prog: crate::demucs_cli::DemucsProgress| {
                    let _ = tx.send(prog);
                },
            )
            .await?;

            let stem_pairs = crate::bs_roformer_output::parse_output(
                output_base.join("htdemucs_6s").join(&track_name)
                    .to_str().ok_or("Invalid demucs output dir")?,
                "demucs",
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
                model: "demucs".to_string(),
            })
        }
    }
}
