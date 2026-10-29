//! Durable projects and small waveform envelopes; audio never leaves this machine.
use crate::commands::ProcessResult;
use rodio::Source;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static WRITES: Mutex<()> = Mutex::new(());

pub fn data_dir() -> Result<PathBuf, String> {
    directories::ProjectDirs::from("", "", "stemway")
        .map(|d| d.data_local_dir().to_path_buf())
        .ok_or("Pasta de dados indisponível".into())
}

pub fn projects_dir() -> Result<PathBuf, String> {
    let path = data_dir()?.join("projects");
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn stamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string()
}

fn project_dir(id: &str) -> Result<PathBuf, String> {
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Projeto inválido".into());
    }
    Ok(projects_dir()?.join(id))
}

pub fn atomic_json(path: &Path, value: &impl serde::Serialize) -> Result<(), String> {
    let _guard = WRITES.lock().map_err(|e| e.to_string())?;
    let temporary = path.with_extension("json.tmp");
    let mut file = std::fs::File::create(&temporary).map_err(|e| e.to_string())?;
    use std::io::Write;
    file.write_all(&serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    std::fs::rename(temporary, path).map_err(|e| e.to_string())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub id: String,
    pub result: ProcessResult,
    #[serde(default)]
    pub state: serde_json::Value,
}

pub fn persist_result(id: String, result: ProcessResult) -> Result<(), String> {
    atomic_json(
        &project_dir(&id)?.join("project.json"),
        &Project {
            id,
            result,
            state: serde_json::json!({}),
        },
    )
}

#[tauri::command]
pub async fn list_projects() -> Result<Vec<Project>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut projects = Vec::new();
        for entry in std::fs::read_dir(projects_dir()?).map_err(|e| e.to_string())? {
            let path = entry
                .map_err(|e| e.to_string())?
                .path()
                .join("project.json");
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(project) = serde_json::from_slice::<Project>(&bytes) {
                    projects.push(project);
                }
            }
        }
        projects.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(projects)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn save_project(id: String, state: serde_json::Value) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = project_dir(&id)?.join("project.json");
        let mut project: Project =
            serde_json::from_slice(&std::fs::read(&path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        project.state = state;
        atomic_json(&path, &project)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub fn envelope(path: &str, bins: usize) -> Result<Vec<f32>, String> {
    let source = rodio::Decoder::new(std::io::BufReader::new(
        std::fs::File::open(path).map_err(|e| e.to_string())?,
    ))
    .map_err(|e| e.to_string())?;
    let count = (source
        .total_duration()
        .ok_or("Duração desconhecida")?
        .as_secs_f64()
        * source.sample_rate() as f64
        * source.channels() as f64)
        .max(1.0);
    let mut peaks = vec![0.0f32; bins];
    for (i, sample) in source.convert_samples::<f32>().enumerate() {
        let bucket = ((i as f64 / count) * bins as f64) as usize;
        if bucket < bins {
            peaks[bucket] = peaks[bucket].max(sample.abs());
        }
    }
    Ok(peaks)
}

#[tauri::command]
pub async fn audio_waveforms(paths: Vec<String>) -> Result<Vec<Vec<f32>>, String> {
    if paths.len() > 12 {
        return Err("Muitas faixas".into());
    }
    tauri::async_runtime::spawn_blocking(move || paths.iter().map(|p| envelope(p, 700)).collect())
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn audio_transport(
    audio: tauri::State<'_, crate::audio::AudioContext>,
) -> Result<crate::audio::Transport, String> {
    let (response, rx) = std::sync::mpsc::sync_channel(1);
    audio
        .tx
        .lock()
        .map_err(|e| e.to_string())?
        .as_ref()
        .ok_or("Áudio indisponível")?
        .send(crate::audio::AudioCommand::Transport { response })
        .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn audio_seek(
    seconds: f64,
    audio: tauri::State<'_, crate::audio::AudioContext>,
) -> Result<(), String> {
    let (response, rx) = std::sync::mpsc::sync_channel(1);
    audio
        .tx
        .lock()
        .map_err(|e| e.to_string())?
        .as_ref()
        .ok_or("Áudio indisponível")?
        .send(crate::audio::AudioCommand::Seek { seconds, response })
        .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn audio_set_loop(
    region: Option<(f64, f64)>,
    audio: tauri::State<'_, crate::audio::AudioContext>,
) -> Result<(), String> {
    let (response, rx) = std::sync::mpsc::sync_channel(1);
    audio
        .tx
        .lock()
        .map_err(|e| e.to_string())?
        .as_ref()
        .ok_or("Áudio indisponível")?
        .send(crate::audio::AudioCommand::Loop { region, response })
        .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejects_project_traversal() {
        for id in ["", "../x", "123/x", "C:\\x"] {
            assert!(super::project_dir(id).is_err());
        }
    }
}
