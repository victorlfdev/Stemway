//! Run the real separation pipeline, including conversion, output parsing and playback.
//! cargo run --example smoke -- <audio-file> <bs-roformer-cpp|demucs|bs-roformer> [--audio]
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let input = PathBuf::from(args.get(1).ok_or("Provide an audio file")?);
    let model = args.get(2).map(String::as_str).unwrap_or("bs-roformer-cpp");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output = std::env::temp_dir().join(format!("stemway-smoke-{model}-{stamp}"));
    std::fs::create_dir_all(&output).map_err(|e| e.to_string())?;
    let stems = tauri::async_runtime::block_on(async {
        match model {
            "bs-roformer-cpp" => {
                app_lib::bs_roformer_cpp_cli::run_bs_roformer_cpp_with_progress(
                    &input,
                    &output,
                    |p| println!("{}: {}%", p.stage, p.percent),
                )
                .await
            }
            "demucs" => {
                app_lib::demucs_cli::run_demucs_cli_with_progress(&input, &output, |p| {
                    println!("{}: {}%", p.stage, p.percent)
                })
                .await?;
                app_lib::demucs_output::parse_demucs_output(
                    output.to_str().unwrap(),
                    input.file_stem().unwrap().to_str().unwrap(),
                )
            }
            "bs-roformer" => {
                let input_dir = output.join("input");
                std::fs::create_dir_all(&input_dir).map_err(|e| e.to_string())?;
                app_lib::info::convert_to_wav(&input, &input_dir.join("track.wav"))?;
                app_lib::bs_roformer_cli::run_bs_roformer_cli_with_progress(
                    &input_dir,
                    &output,
                    |p| println!("{}: {}%", p.stage, p.percent),
                )
                .await
            }
            _ => Err("Unknown model".into()),
        }
    })?;
    if stems.len() != 6 {
        return Err(format!("Expected 6 stems, got {}", stems.len()));
    }
    for (name, path) in &stems {
        let (duration, rate, channels, bytes) = app_lib::info::probe_wav(path)?;
        if duration <= 0.0 || rate != 44100 || channels != 2 || bytes <= 44 {
            return Err(format!("Invalid stem: {name}"));
        }
        println!("{name}: {duration:.2}s, {rate} Hz, {channels} channels, {bytes} bytes");
    }
    if args.iter().any(|arg| arg == "--audio") {
        use app_lib::audio::{AudioCommand, AudioContext};
        let audio = AudioContext::new();
        let tx = audio
            .tx
            .lock()
            .map_err(|e| e.to_string())?
            .as_ref()
            .unwrap()
            .clone();
        let (response, rx) = std::sync::mpsc::sync_channel(1);
        tx.send(AudioCommand::LoadStems {
            stem_paths: stems.clone(),
            response,
        })
        .map_err(|e| e.to_string())?;
        let loaded = rx.recv().map_err(|e| e.to_string())??;
        assert_eq!(loaded.len(), 6);
        for (id, _) in &stems {
            let (response, rx) = std::sync::mpsc::sync_channel(1);
            tx.send(AudioCommand::SetVolume {
                id: id.clone(),
                volume: 0.05,
                response,
            })
            .map_err(|e| e.to_string())?;
            rx.recv().map_err(|e| e.to_string())??;
        }
        for solo in [true, false] {
            let (response, rx) = std::sync::mpsc::sync_channel(1);
            tx.send(AudioCommand::SetSolo {
                id: stems[0].0.clone(),
                solo,
                response,
            })
            .map_err(|e| e.to_string())?;
            rx.recv().map_err(|e| e.to_string())??;
        }
        for muted in [true, false] {
            let (response, rx) = std::sync::mpsc::sync_channel(1);
            tx.send(AudioCommand::SetMute {
                id: stems[0].0.clone(),
                muted,
                response,
            })
            .map_err(|e| e.to_string())?;
            rx.recv().map_err(|e| e.to_string())??;
        }
        let (response, rx) = std::sync::mpsc::sync_channel(1);
        tx.send(AudioCommand::TogglePlayback { response })
            .map_err(|e| e.to_string())?;
        rx.recv().map_err(|e| e.to_string())??;
        let (response, rx) = std::sync::mpsc::sync_channel(1);
        tx.send(AudioCommand::LoadStems {
            stem_paths: Vec::new(),
            response,
        })
        .map_err(|e| e.to_string())?;
        rx.recv().map_err(|e| e.to_string())??;
        println!("Playback, volume, mute, solo and unload: OK");
    }
    println!("PASS: {}", output.display());
    Ok(())
}
