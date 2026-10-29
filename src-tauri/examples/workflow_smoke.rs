//! Run the same separation command as the UI; produces a reusable library project.
//! cargo run --example workflow_smoke -- <file> [model] [backend]
fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).ok_or("Provide an audio path")?.clone();
    let model = args.get(2).cloned().unwrap_or("bs-roformer-cpp".into());
    let backend = args.get(3).cloned().unwrap_or("auto".into());
    let clip = if let (Some(start), Some(end)) = (args.get(4), args.get(5)) {
        Some((
            start.parse::<f64>().map_err(|e| e.to_string())?,
            end.parse::<f64>().map_err(|e| e.to_string())?,
        ))
    } else {
        None
    };
    let result = tauri::async_runtime::block_on(app_lib::commands::process_file(
        path,
        model,
        Some(backend),
        clip,
    ))?;
    assert_eq!(result.paths.len(), 6);
    for path in &result.paths {
        let (duration, rate, channels, _) = app_lib::info::probe_wav(path)?;
        assert!(duration > 0.0);
        assert_eq!((rate, channels), (44100, 2));
    }
    let peaks =
        tauri::async_runtime::block_on(app_lib::library::audio_waveforms(result.paths.clone()))?;
    assert_eq!(peaks.len(), 6);
    assert!(peaks.iter().any(|p| p.iter().any(|v| *v > 0.0001)));
    println!("PASS: separation, six valid stems, persistent project and waveforms");
    println!("Project: {}", result.project_id);
    println!(
        "Backend: {:?}; inference: {:.2}s",
        result.backend, result.elapsed_secs
    );
    Ok(())
}
