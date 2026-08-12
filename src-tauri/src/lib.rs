pub mod demucs_cli;
pub mod demucs_output;
pub mod commands;
pub mod info;

pub use commands::ProcessResult;

pub fn run() {
    println!("Stem Separator running");
}

pub fn tauri_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::process_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running stem separator tauri app");
}

#[cfg(test)]
mod tests {
    use crate::info;

    #[test]
    fn test_probe_test_wav() {
        let result = info::probe_wav("/tmp/demucs_final_test/htdemucs_6s/test_final/bass.wav");
        assert!(result.is_ok());
        let (dur, sr, ch, size) = result.unwrap();
        assert!(dur > 9.0 && dur < 11.0);
        assert_eq!(sr, 44100);
        assert_eq!(ch, 2);
        assert!(size > 0);
    }
}
