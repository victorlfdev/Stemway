pub mod demucs_cli;
pub mod demucs_output;
pub mod commands;
pub mod info;

pub use commands::ProcessResult;
use tauri_plugin_fs::FsExt as _;

pub fn run() {
    println!("Stem Separator running");
}

pub fn tauri_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let scope = app.fs_scope();
            let cache_dir = directories::ProjectDirs::from("", "", "stem-separator")
                .map(|d| d.cache_dir().to_path_buf())
                .expect("failed to get cache dir");
            scope.allow_directory(&cache_dir, true)?;
            let temp_dir = std::env::temp_dir();
            scope.allow_directory(&temp_dir, true)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::process_file,
            commands::read_stem_as_base64,
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
