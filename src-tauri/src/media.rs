//! Offline study renders and exports share exactly the same filters and gains.
use crate::library::{data_dir, stamp};
use std::path::{Path, PathBuf};

pub fn ffmpeg() -> std::process::Command {
    let mut cmd = std::process::Command::new("ffmpeg");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd.args(["-v", "error", "-nostdin", "-n"]);
    cmd
}

pub fn run(cmd: &mut std::process::Command) -> Result<(), String> {
    let output = cmd
        .output()
        .map_err(|e| format!("FFmpeg indisponível: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(())
}

pub fn study_filter(speed: f64, semitones: i32) -> Result<String, String> {
    if !speed.is_finite() || !(0.5..=2.0).contains(&speed) || !(-12..=12).contains(&semitones) {
        return Err("Velocidade deve estar entre 0,5 e 2; tom entre -12 e +12".into());
    }
    let pitch = 2.0f64.powf(semitones as f64 / 12.0);
    let mut tempo = speed / pitch;
    let mut filters = vec![format!(
        "aresample=44100,asetrate={:.6},aresample=44100",
        44100.0 * pitch
    )];
    while tempo < 0.5 {
        filters.push("atempo=0.5".into());
        tempo /= 0.5;
    }
    while tempo > 2.0 {
        filters.push("atempo=2".into());
        tempo /= 2.0;
    }
    filters.push(format!("atempo={tempo:.8}"));
    Ok(filters.join(","))
}

#[tauri::command]
pub async fn render_study(
    paths: Vec<String>,
    speed: f64,
    semitones: i32,
) -> Result<Vec<String>, String> {
    let filter = study_filter(speed, semitones)?;
    if paths.is_empty() || paths.len() > 12 {
        return Err("Selecione as faixas".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        static RENDERS: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = RENDERS.lock().map_err(|e| e.to_string())?;
        let key = crate::library::source_key(&paths, &format!("study-v1-{speed}-{semitones}"))?;
        let root = data_dir()?.join("study");
        let completed = root.join(key);
        let cached: Vec<String> = (0..paths.len())
            .map(|i| {
                completed
                    .join(format!("{i}.wav"))
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        if cached.iter().all(|p| Path::new(p).is_file()) {
            return Ok(cached);
        }
        let dir = root.join(stamp());
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let result = (|| {
            let mut outputs = Vec::new();
            for (i, path) in paths.iter().enumerate() {
                let dest = dir.join(format!("{i}.wav"));
                run(ffmpeg()
                    .arg("-i")
                    .arg(path)
                    .args(["-af", &filter, "-ac", "2", "-c:a", "pcm_f32le"])
                    .arg(&dest))?;
                outputs.push(dest.to_string_lossy().into_owned());
            }
            std::fs::rename(&dir, &completed).map_err(|e| e.to_string())?;
            Ok(cached)
        })();
        if result.is_err() {
            let _ = std::fs::remove_dir_all(&dir);
        }
        result
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(serde::Deserialize, Clone)]
pub struct ExportTrack {
    pub path: String,
    pub id: String,
    pub gain: f64,
}

#[derive(serde::Deserialize)]
pub struct ExportRequest {
    pub tracks: Vec<ExportTrack>,
    pub directory: String,
    pub name: String,
    pub format: String,
    pub mix: bool,
    pub speed: f64,
    pub semitones: i32,
    pub region: Option<(f64, f64)>,
}

fn safe_name(name: &str) -> String {
    let value: String = name
        .chars()
        .map(|c| {
            if c.is_control() || "<>:\"/\\|?*".contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let value = value.trim().trim_matches('.');
    if value.is_empty() {
        "Stemway".into()
    } else {
        value.chars().take(100).collect()
    }
}

fn encode(cmd: &mut std::process::Command, format: &str) -> Result<(), String> {
    match format {
        "wav16" => {
            cmd.args(["-c:a", "pcm_s16le"]);
        }
        "wav24" => {
            cmd.args(["-c:a", "pcm_s24le"]);
        }
        "flac" => {
            cmd.args(["-c:a", "flac"]);
        }
        "mp3" => {
            cmd.args(["-c:a", "libmp3lame", "-b:a", "320k"]);
        }
        _ => return Err("Formato inválido".into()),
    }
    cmd.args(["-ar", "44100", "-ac", "2"]);
    Ok(())
}

pub fn export(request: ExportRequest) -> Result<Vec<String>, String> {
    let filter = study_filter(request.speed, request.semitones)?;
    if request.tracks.is_empty()
        || request.tracks.len() > 12
        || request.tracks.iter().any(|t| {
            !t.gain.is_finite() || !(0.0..=1.0).contains(&t.gain) || !Path::new(&t.path).is_file()
        })
    {
        return Err("Faixas de exportação inválidas".into());
    }
    if let Some((a, b)) = request.region {
        if !a.is_finite() || !b.is_finite() || a < 0.0 || b <= a {
            return Err("Trecho inválido".into());
        }
    }
    let extension = match request.format.as_str() {
        "wav16" | "wav24" => "wav",
        "flac" => "flac",
        "mp3" => "mp3",
        _ => return Err("Formato inválido".into()),
    };
    if !Path::new(&request.directory).is_dir() {
        return Err("Selecione uma pasta existente".into());
    }
    // A new subfolder avoids overwriting user files, including the source stems.
    let folder =
        PathBuf::from(&request.directory).join(format!("{}-{}", safe_name(&request.name), stamp()));
    std::fs::create_dir(&folder).map_err(|e| e.to_string())?;
    let result = (|| {
        let mut outputs = Vec::new();
        if request.mix {
            let mut cmd = ffmpeg();
            let mut graph = Vec::new();
            for (i, track) in request.tracks.iter().enumerate() {
                cmd.arg("-i").arg(&track.path);
                let trim = request
                    .region
                    .map(|(a, b)| format!("atrim=start={a}:end={b},asetpts=PTS-STARTPTS,"))
                    .unwrap_or_default();
                graph.push(format!("[{i}:a]{trim}{filter},volume={}[s{i}]", track.gain));
            }
            let inputs: String = (0..request.tracks.len())
                .map(|i| format!("[s{i}]"))
                .collect();
            graph.push(format!(
                "{inputs}amix=inputs={}:normalize=0,alimiter=limit=0.98:level=0:latency=1[out]",
                request.tracks.len()
            ));
            cmd.args(["-filter_complex", &graph.join(";"), "-map", "[out]"]);
            encode(&mut cmd, &request.format)?;
            let output = folder.join(format!("mix.{extension}"));
            run(cmd.arg(&output))?;
            outputs.push(output.to_string_lossy().into_owned());
        } else {
            for (i, track) in request.tracks.iter().enumerate() {
                let trim = request
                    .region
                    .map(|(a, b)| format!("atrim=start={a}:end={b},asetpts=PTS-STARTPTS,"))
                    .unwrap_or_default();
                let output = folder.join(format!("{}-{}.{extension}", i + 1, safe_name(&track.id)));
                let mut cmd = ffmpeg();
                cmd.arg("-i")
                    .arg(&track.path)
                    .args(["-af", &format!("{trim}{filter}")]);
                encode(&mut cmd, &request.format)?;
                run(cmd.arg(&output))?;
                outputs.push(output.to_string_lossy().into_owned());
            }
        }
        Ok(outputs)
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&folder);
    }
    result
}

#[tauri::command]
pub async fn export_audio(request: ExportRequest) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || export(request))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn study_pitch_and_speed_have_independent_compensation() {
        let f = study_filter(0.5, 12).unwrap();
        assert!(f.contains("asetrate=88200"));
        assert!(f.contains("atempo=0.5,atempo=0.50000000"));
        assert!(study_filter(f64::NAN, 0).is_err());
    }

    #[test]
    fn real_export_and_waveform() {
        let folder = std::env::temp_dir().join(format!("stemway-export-{}", stamp()));
        std::fs::create_dir(&folder).unwrap();
        let input = folder.join("input.wav");
        run(ffmpeg()
            .args([
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=1",
                "-ac",
                "2",
            ])
            .arg(&input))
        .unwrap();
        let peaks = crate::library::envelope(input.to_str().unwrap(), 100).unwrap();
        assert!(peaks.iter().all(|p| *p > 0.0));
        let outputs = export(ExportRequest {
            tracks: vec![ExportTrack {
                path: input.to_string_lossy().into_owned(),
                id: "voz".into(),
                gain: 0.5,
            }],
            directory: folder.to_string_lossy().into_owned(),
            name: "test".into(),
            format: "wav24".into(),
            mix: true,
            speed: 0.5,
            semitones: 3,
            region: None,
        })
        .unwrap();
        let (duration, rate, channels, _) = crate::info::probe_wav(&outputs[0]).unwrap();
        assert!((duration - 2.0).abs() < 0.2, "duration: {duration}");
        assert_eq!((rate, channels), (44100, 2));
        assert!(input.exists());
        std::fs::remove_dir_all(folder).unwrap();
    }
}
