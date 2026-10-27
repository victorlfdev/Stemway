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
    pub backend: Option<String>,
}

#[derive(serde::Serialize)]
pub struct FileProbeResult {
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub file_size_bytes: u64,
}

#[derive(serde::Serialize)]
pub struct FileSizeResult {
    pub size: u64,
}

#[tauri::command]
pub fn get_file_size(path: String) -> Result<FileSizeResult, String> {
    let metadata =
        std::fs::metadata(&path).map_err(|e| format!("Failed to get file metadata: {}", e))?;
    Ok(FileSizeResult {
        size: metadata.len(),
    })
}

#[derive(serde::Serialize)]
pub struct OpenFolderResult {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub async fn probe_file(path: String) -> Result<FileProbeResult, String> {
    let (duration_secs, sample_rate, channels, file_size_bytes) =
        tauri::async_runtime::spawn_blocking(move || crate::info::probe_wav(&path))
            .await
            .map_err(|e| e.to_string())??;
    Ok(FileProbeResult {
        duration_secs,
        sample_rate,
        channels,
        file_size_bytes,
    })
}

#[tauri::command]
pub async fn open_output_folder(path: String) -> Result<OpenFolderResult, String> {
    let path_ref = std::path::Path::new(&path);
    let folder_to_open = if path_ref.is_file() {
        path_ref
            .parent()
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

static PROCESSING: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tauri::command]
pub async fn process_file(
    path: String,
    model: String,
    app: tauri::AppHandle,
) -> Result<ProcessResult, String> {
    let _guard = PROCESSING
        .try_lock()
        .map_err(|_| "Another separation is already running")?;
    if !["demucs", "bs-roformer", "bs-roformer-cpp"].contains(&model.as_str()) {
        return Err(format!("Unknown model: {model}"));
    }
    let input = std::path::PathBuf::from(&path);
    if !input.is_file() {
        return Err(format!("Audio file not found: {path}"));
    }
    let track_name = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?
        .to_owned();
    let cache = directories::ProjectDirs::from("", "", "stemway")
        .map(|d| d.cache_dir().to_path_buf())
        .unwrap_or_else(std::env::temp_dir);
    // Each run has its own directory, so a failed run cannot return old stems.
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let output = cache
        .join("stem-output")
        .join(&model)
        .join(format!("{track_name}-{timestamp}"));
    std::fs::create_dir_all(&output).map_err(|e| e.to_string())?;
    let backend;
    let pairs = match model.as_str() {
        "bs-roformer" => {
            let input_dir = output.join("input");
            std::fs::create_dir_all(&input_dir).map_err(|e| e.to_string())?;
            let converted = input_dir.join("track.wav");
            let source = input.clone();
            tauri::async_runtime::spawn_blocking(move || {
                crate::info::convert_to_wav(&source, &converted)
            })
            .await
            .map_err(|e| e.to_string())??;
            backend = Some("CPU".to_owned());
            crate::bs_roformer_cli::run_bs_roformer_cli_with_progress(
                &input_dir,
                &output,
                move |progress| {
                    let _ = app.emit("bs_roformer_progress", progress);
                },
            )
            .await?
        }
        "bs-roformer-cpp" => {
            backend = Some(
                crate::bs_roformer_cpp_cli::get_backend_name(
                    &crate::bs_roformer_cpp_cli::detect_gpu_backend(),
                )
                .to_owned(),
            );
            crate::bs_roformer_cpp_cli::run_bs_roformer_cpp_with_progress(
                &input,
                &output,
                move |progress| {
                    let _ = app.emit("bs_roformer_cpp_progress", progress);
                },
            )
            .await?
        }
        _ => {
            backend = Some("CPU".to_owned());
            crate::demucs_cli::run_demucs_cli_with_progress(&input, &output, move |progress| {
                let _ = app.emit("demucs_progress", progress);
            })
            .await?;
            crate::demucs_output::parse_demucs_output(
                output.to_str().ok_or("Invalid output path")?,
                &track_name,
            )?
        }
    };
    if pairs.len() != 6
        || pairs
            .iter()
            .any(|(_, p)| !std::path::Path::new(p).is_file())
    {
        return Err(format!(
            "Separation did not produce all 6 stems (found {})",
            pairs.len()
        ));
    }
    let first = pairs[0].1.clone();
    let (duration_secs, sample_rate, channels, file_size_bytes) =
        tauri::async_runtime::spawn_blocking(move || crate::info::probe_wav(&first))
            .await
            .map_err(|e| e.to_string())??;
    Ok(ProcessResult {
        paths: pairs.into_iter().map(|(_, p)| p).collect(),
        name: track_name,
        duration_secs,
        sample_rate,
        channels,
        file_size_bytes,
        model,
        backend,
    })
}

#[derive(serde::Serialize)]
pub struct StemInfo {
    pub id: String,
    pub muted: bool,
    pub soloed: bool,
    pub volume: f32,
}

#[derive(serde::Serialize)]
pub struct AudioLoadResult {
    pub stem_count: usize,
}

#[tauri::command]
pub fn audio_get_playback_state(
    audio: tauri::State<'_, crate::audio::AudioContext>,
) -> Result<bool, String> {
    let tx = audio.tx.lock().map_err(|e| e.to_string())?;
    let sender = tx.as_ref().ok_or("Audio context not initialized")?;
    let (response, rx) = std::sync::mpsc::sync_channel(1);
    sender
        .send(crate::audio::AudioCommand::GetPlaybackState { response })
        .map_err(|e| e.to_string())?;
    drop(tx);
    rx.recv().map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn audio_load_stems(
    audio: tauri::State<'_, crate::audio::AudioContext>,
    stem_paths: Vec<(String, String)>,
) -> Result<AudioLoadResult, String> {
    let stem_count = stem_paths.len();
    eprintln!("[Audio IPC] load_stems → {} stems", stem_count);
    for (name, path) in &stem_paths {
        eprintln!("  [Audio IPC]   {} → {}", name, path);
    }
    let tx = audio
        .tx
        .lock()
        .map_err(|e| format!("Failed to lock tx: {}", e))?;
    let sender = tx.as_ref().ok_or("Audio context not initialized")?;

    let (response_tx, response_rx) = std::sync::mpsc::sync_channel(1);
    let cmd = crate::audio::AudioCommand::LoadStems {
        stem_paths,
        response: response_tx,
    };
    sender
        .send(cmd)
        .map_err(|e| format!("Failed to send load command: {}", e))?;
    drop(tx);
    let loaded_ids = response_rx
        .recv()
        .map_err(|e| format!("Load failed: {}", e))??;
    eprintln!(
        "[Audio IPC] load_stems → OK ({} stems confirmed)",
        loaded_ids.len()
    );

    Ok(AudioLoadResult {
        stem_count: loaded_ids.len(),
    })
}

#[tauri::command]
pub fn audio_toggle_playback(
    audio: tauri::State<'_, crate::audio::AudioContext>,
) -> Result<(), String> {
    eprintln!("[Audio IPC] toggle_playback");
    let tx = audio
        .tx
        .lock()
        .map_err(|e| format!("Failed to lock tx: {}", e))?;
    let sender = tx.as_ref().ok_or("Audio context not initialized")?;

    let (response_tx, response_rx) = std::sync::mpsc::sync_channel(1);
    let cmd = crate::audio::AudioCommand::TogglePlayback {
        response: response_tx,
    };
    sender
        .send(cmd)
        .map_err(|e| format!("Failed to send toggle command: {}", e))?;
    drop(tx);
    response_rx
        .recv()
        .map_err(|e| format!("Toggle failed: {}", e))??;
    eprintln!("[Audio IPC] toggle_playback → OK");
    Ok(())
}

#[tauri::command]
pub fn audio_set_volume(
    audio: tauri::State<'_, crate::audio::AudioContext>,
    id: String,
    volume: f32,
) -> Result<(), String> {
    eprintln!("[Audio IPC] set_volume → id={}, volume={:.2}", id, volume);
    let tx = audio
        .tx
        .lock()
        .map_err(|e| format!("Failed to lock tx: {}", e))?;
    let sender = tx.as_ref().ok_or("Audio context not initialized")?;

    let (response_tx, response_rx) = std::sync::mpsc::sync_channel(1);
    let cmd = crate::audio::AudioCommand::SetVolume {
        id,
        volume,
        response: response_tx,
    };
    sender
        .send(cmd)
        .map_err(|e| format!("Failed to send volume command: {}", e))?;
    drop(tx);
    response_rx
        .recv()
        .map_err(|e| format!("Volume command failed: {}", e))??;
    eprintln!("[Audio IPC] set_volume → OK");
    Ok(())
}

#[tauri::command]
pub fn audio_set_mute(
    audio: tauri::State<'_, crate::audio::AudioContext>,
    id: String,
    muted: bool,
) -> Result<(), String> {
    eprintln!("[Audio IPC] set_mute → id={}, muted={}", id, muted);
    let tx = audio
        .tx
        .lock()
        .map_err(|e| format!("Failed to lock tx: {}", e))?;
    let sender = tx.as_ref().ok_or("Audio context not initialized")?;

    let (response_tx, response_rx) = std::sync::mpsc::sync_channel(1);
    let cmd = crate::audio::AudioCommand::SetMute {
        id,
        muted,
        response: response_tx,
    };
    sender
        .send(cmd)
        .map_err(|e| format!("Failed to send mute command: {}", e))?;
    drop(tx);
    response_rx
        .recv()
        .map_err(|e| format!("Mute command failed: {}", e))??;
    eprintln!("[Audio IPC] set_mute → OK");
    Ok(())
}

#[tauri::command]
pub fn audio_set_solo(
    audio: tauri::State<'_, crate::audio::AudioContext>,
    id: String,
    solo: bool,
) -> Result<(), String> {
    eprintln!("[Audio IPC] set_solo → id={}, solo={}", id, solo);
    let tx = audio
        .tx
        .lock()
        .map_err(|e| format!("Failed to lock tx: {}", e))?;
    let sender = tx.as_ref().ok_or("Audio context not initialized")?;

    let (response_tx, response_rx) = std::sync::mpsc::sync_channel(1);
    let cmd = crate::audio::AudioCommand::SetSolo {
        id,
        solo,
        response: response_tx,
    };
    sender
        .send(cmd)
        .map_err(|e| format!("Failed to send solo command: {}", e))?;
    drop(tx);
    response_rx
        .recv()
        .map_err(|e| format!("Solo command failed: {}", e))??;
    eprintln!("[Audio IPC] set_solo → OK");
    Ok(())
}

#[tauri::command]
pub fn audio_get_stem_count(
    audio: tauri::State<'_, crate::audio::AudioContext>,
) -> Result<usize, String> {
    let tx = audio
        .tx
        .lock()
        .map_err(|e| format!("Failed to lock tx: {}", e))?;
    let sender = tx.as_ref().ok_or("Audio context not initialized")?;

    let (response_tx, response_rx) = std::sync::mpsc::sync_channel(1);
    let cmd = crate::audio::AudioCommand::GetStemCount {
        response: response_tx,
    };
    sender
        .send(cmd)
        .map_err(|e| format!("Failed to send count command: {}", e))?;
    drop(tx);
    response_rx
        .recv()
        .map_err(|e| format!("Count command failed: {}", e))?
}

#[tauri::command]
pub fn audio_get_stems(
    audio: tauri::State<'_, crate::audio::AudioContext>,
) -> Result<Vec<StemInfo>, String> {
    let tx = audio
        .tx
        .lock()
        .map_err(|e| format!("Failed to lock tx: {}", e))?;
    let sender = tx.as_ref().ok_or("Audio context not initialized")?;

    let (response_tx, response_rx) = std::sync::mpsc::sync_channel(1);
    let cmd = crate::audio::AudioCommand::GetStems {
        response: response_tx,
    };
    sender
        .send(cmd)
        .map_err(|e| format!("Failed to send stems command: {}", e))?;
    drop(tx);
    let stems = response_rx
        .recv()
        .map_err(|e| format!("Stems command failed: {}", e))??;
    Ok(stems
        .into_iter()
        .map(|(id, muted, soloed, volume)| StemInfo {
            id,
            muted,
            soloed,
            volume,
        })
        .collect())
}
