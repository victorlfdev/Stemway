pub mod bs_roformer_cli;
pub mod bs_roformer_output;
pub mod bs_roformer_cpp_cli;
pub mod bs_roformer_cpp_output;
pub mod demucs_cli;
pub mod demucs_output;
pub mod commands;
pub mod info;

pub use commands::ProcessResult;
use tauri_plugin_fs::FsExt as _;

pub fn run() {
    println!("Stemway running");
}

pub fn tauri_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let scope = app.fs_scope();
            let cache_dir = directories::ProjectDirs::from("", "", "stemway")
                .map(|d| d.cache_dir().to_path_buf())
                .expect("failed to get cache dir");
            scope.allow_directory(&cache_dir, true)?;
            let temp_dir = std::env::temp_dir();
            scope.allow_directory(&temp_dir, true)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::process_file,
            commands::open_output_folder,
            commands::get_file_size,
            commands::probe_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running stemway tauri app");
}

#[cfg(test)]
mod tests {
    use crate::info;

    #[test]
    fn test_probe_test_wav() {
        let result = info::probe_wav("test_output_stem__bass.wav");
        assert!(result.is_ok());
        let (dur, sr, ch, size) = result.unwrap();
        assert!(dur > 90.0);
        assert_eq!(sr, 16000);
        assert_eq!(ch, 2);
        assert!(size > 0);
    }
}
