//! Real-device smoke test for seek, looping, study, export, persistence and cancellation.
//! cargo run --example studio_smoke
use app_lib::{
    audio::{AudioCommand, AudioContext},
    media::{self, ExportRequest, ExportTrack},
};
use std::sync::mpsc;
use std::time::Duration;

fn ask<T>(
    audio: &AudioContext,
    make: impl FnOnce(mpsc::SyncSender<Result<T, String>>) -> AudioCommand,
) -> Result<T, String> {
    let (response, rx) = mpsc::sync_channel(1);
    audio
        .tx
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .send(make(response))
        .map_err(|e| e.to_string())?;
    rx.recv_timeout(Duration::from_secs(10))
        .map_err(|e| e.to_string())?
}

fn main() -> Result<(), String> {
    app_lib::jobs::begin();
    let folder = std::env::temp_dir().join(format!(
        "stemway-studio-smoke-{}",
        app_lib::library::stamp()
    ));
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let mut stems = Vec::new();
    for (i, id) in ["vocals", "drums", "bass", "guitar", "piano", "other"]
        .into_iter()
        .enumerate()
    {
        let path = folder.join(format!("{id}.wav"));
        media::run(
            media::ffmpeg()
                .args([
                    "-f",
                    "lavfi",
                    "-i",
                    &format!("sine=frequency={}:duration=4", 220 + i * 110),
                    "-ar",
                    "44100",
                    "-ac",
                    "2",
                ])
                .arg(&path),
        )?;
        stems.push((id.to_string(), path.to_string_lossy().into_owned()));
    }
    let audio = AudioContext::new();
    let loaded = ask(&audio, |response| AudioCommand::LoadStems {
        stem_paths: stems.clone(),
        response,
    })?;
    assert_eq!(loaded.len(), 6);
    for (id, _) in &stems {
        ask(&audio, |response| AudioCommand::SetVolume {
            id: id.clone(),
            volume: 0.01,
            response,
        })?;
    }
    ask(&audio, |response| AudioCommand::Seek {
        seconds: 1.5,
        response,
    })?;
    let snapshot = ask(&audio, |response| AudioCommand::Transport { response })?;
    assert!((snapshot.position - 1.5).abs() < 0.1);
    ask(&audio, |response| AudioCommand::Loop {
        region: Some((1.0, 1.3)),
        response,
    })?;
    ask(&audio, |response| AudioCommand::Seek {
        seconds: 1.1,
        response,
    })?;
    ask(&audio, |response| AudioCommand::TogglePlayback { response })?;
    std::thread::sleep(Duration::from_millis(650));
    let snapshot = ask(&audio, |response| AudioCommand::Transport { response })?;
    assert!(
        snapshot.playing && snapshot.position >= 1.0 && snapshot.position < 1.4,
        "loop position: {}",
        snapshot.position
    );
    ask(&audio, |response| AudioCommand::TogglePlayback { response })?;
    ask(&audio, |response| AudioCommand::Loop {
        region: None,
        response,
    })?;
    ask(&audio, |response| AudioCommand::Seek {
        seconds: 3.9,
        response,
    })?;
    ask(&audio, |response| AudioCommand::TogglePlayback { response })?;
    std::thread::sleep(Duration::from_millis(350));
    let snapshot = ask(&audio, |response| AudioCommand::Transport { response })?;
    assert!(!snapshot.playing);
    // Seeking after reaching EOF must recreate exhausted sources.
    ask(&audio, |response| AudioCommand::Seek {
        seconds: 0.5,
        response,
    })?;
    ask(&audio, |response| AudioCommand::TogglePlayback { response })?;
    std::thread::sleep(Duration::from_millis(100));
    assert!(ask(&audio, |response| AudioCommand::Transport { response })?.playing);
    ask(&audio, |response| AudioCommand::LoadStems {
        stem_paths: Vec::new(),
        response,
    })?;
    println!("PASS: real-device seek, loop, EOF replay and unload");

    let paths: Vec<String> = stems.iter().map(|(_, p)| p.clone()).collect();
    let rendered =
        tauri::async_runtime::block_on(app_lib::media::render_study(paths.clone(), 0.75, -3))?;
    let (duration, _, _, _) = app_lib::info::probe_wav(&rendered[0])?;
    assert!((duration - 4.0 / 0.75).abs() < 0.2);
    for format in ["wav16", "wav24", "flac", "mp3"] {
        let outputs = media::export(ExportRequest {
            tracks: stems
                .iter()
                .map(|(id, path)| ExportTrack {
                    id: id.clone(),
                    path: path.clone(),
                    gain: 0.1,
                })
                .collect(),
            directory: folder.to_string_lossy().into_owned(),
            name: "mix".into(),
            format: format.into(),
            mix: true,
            speed: 1.0,
            semitones: 0,
            region: Some((1.0, 2.5)),
        })?;
        let (duration, rate, channels, _) = app_lib::info::probe_wav(&outputs[0])?;
        assert!((duration - 1.5).abs() < 0.2);
        assert_eq!((rate, channels), (44100, 2));
    }
    println!("PASS: independent study render and four export formats");

    let id = app_lib::library::stamp();
    let project_dir = app_lib::library::projects_dir()?.join(&id);
    std::fs::create_dir(&project_dir).map_err(|e| e.to_string())?;
    let result = app_lib::ProcessResult {
        project_id: id.clone(),
        source_path: paths[0].clone(),
        elapsed_secs: 1.0,
        clip_start: 0.0,
        paths,
        name: "studio smoke".into(),
        duration_secs: 4.0,
        sample_rate: 44100,
        channels: 2,
        file_size_bytes: 0,
        model: "demucs".into(),
        backend: Some("CPU".into()),
    };
    app_lib::library::persist_result(id.clone(), result)?;
    let state = serde_json::json!({"mix":{"vocals":{"volume":0.4,"muted":true}},"position":1.5,"markers":[{"name":"refrão","position":1.0}]});
    tauri::async_runtime::block_on(app_lib::library::save_project(id.clone(), state.clone()))?;
    let library = tauri::async_runtime::block_on(app_lib::library::list_projects())?;
    assert_eq!(library.iter().find(|p| p.id == id).unwrap().state, state);
    std::fs::remove_dir_all(&project_dir).map_err(|e| e.to_string())?;
    println!("PASS: durable project round trip with mixer and markers");

    let worker = std::thread::spawn(|| {
        let mut cmd = if cfg!(windows) {
            let mut c = std::process::Command::new("powershell.exe");
            c.args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 30",
            ]);
            c
        } else {
            let mut c = std::process::Command::new("sleep");
            c.arg("30");
            c
        };
        app_lib::jobs::sync_output(&mut cmd)
    });
    std::thread::sleep(Duration::from_millis(500));
    app_lib::jobs::cancel()?;
    assert!(worker.join().unwrap().unwrap_err().contains("cancelada"));
    app_lib::jobs::begin();
    println!("PASS: cancellation terminates the owned process");
    std::fs::remove_dir_all(&folder).map_err(|e| e.to_string())?;
    if let Some(path) = rendered
        .first()
        .and_then(|p| std::path::Path::new(p).parent())
    {
        std::fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
