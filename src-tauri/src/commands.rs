use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct AppState {
    pub stem_paths: Arc<Mutex<Option<Vec<String>>>>,
    pub current_file: Arc<Mutex<Option<String>>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            stem_paths: Arc::clone(&self.stem_paths),
            current_file: Arc::clone(&self.current_file),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            stem_paths: Arc::new(Mutex::new(None)),
            current_file: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_stems(&self, paths: Vec<String>) {
        let mut guard = self.stem_paths.lock().unwrap();
        *guard = Some(paths);
    }

    pub fn set_current_file(&self, path: String) {
        let mut guard = self.current_file.lock().unwrap();
        *guard = Some(path);
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FileMetadata {
    pub path: String,
    pub name: String,
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub file_size_bytes: u64,
}

#[tauri::command]
pub async fn analyze_file(path: String) -> Result<FileMetadata, String> {
    let path_buf = std::path::PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("File not found: {}", path));
    }

    let metadata = std::fs::metadata(&path_buf).map_err(|e| format!("IO error: {}", e))?;
    let file_size = metadata.len();
    let name = path_buf
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let reader = crate::audio::AudioReader::new(&path_buf)
        .map_err(|e| format!("Failed to read audio: {}", e))?;

    Ok(FileMetadata {
        path,
        name,
        duration_secs: reader.duration_secs,
        sample_rate: reader.sample_rate,
        channels: reader.channels,
        file_size_bytes: file_size,
    })
}

#[tauri::command]
pub async fn process_audio_file(
    path: String,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let path_buf = std::path::PathBuf::from(&path);
    let reader = crate::audio::AudioReader::new(&path_buf).map_err(|e| format!("Failed to read audio: {}", e))?;
    let samples = reader.get_samples().clone();
    let sample_rate = reader.sample_rate;
    let target_sr = 16000u32;

    let processed_samples = if sample_rate != target_sr {
        let ratio = target_sr as f64 / sample_rate as f64;
        let stereo_len = samples.len() / 2;
        let new_len = (stereo_len as f64 * ratio) as usize;
        let mut resampled = vec![0.0f32; new_len * 2];
        for i in (0..samples.len()).step_by(2) {
            let idx = (i as f64 / 2.0 * ratio) as usize;
            if idx < new_len {
                resampled[idx * 2] = samples[i];
                if i + 1 < samples.len() {
                    resampled[idx * 2 + 1] = samples[i + 1];
                }
            }
        }
        resampled
    } else {
        samples
    };

    let processor = crate::ChunkProcessor::new();
    let stem_names = ["vocals", "bass", "drums", "other"];

    let stems = processor
        .process_sequential_stereo(&processed_samples, &mut |chunk_data| {
            let mut model = crate::DemucsModel::new().map_err(|e| format!("Model error: {}", e))?;
            let result = model.separate(chunk_data, target_sr).map_err(|e| format!("Inference error: {}", e))?;
            Ok(result)
        })
        .map_err(|e| format!("Inference pipeline failed: {}", e))?;

    let normalized = {
        let stitcher = crate::Stitcher::new();
        let mut stem_clone: Vec<Vec<f32>> = stems.into_iter().collect();
        stitcher.normalize(&mut stem_clone, 0.99)
    };

    let normalized_slices: Vec<&[f32]> = normalized.iter().map(|s| s.as_slice()).collect();
    let written = crate::audio::writer::AudioWriter::write_4_stems(&output_dir, &normalized_slices, &stem_names, target_sr)
        .map_err(|e| format!("Failed to write WAV: {}", e))?;

    Ok(written)
}
