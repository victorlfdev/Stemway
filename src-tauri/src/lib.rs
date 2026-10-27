pub mod audio;
pub mod bs_roformer_cli;
pub mod bs_roformer_cpp_cli;
pub mod bs_roformer_cpp_output;
pub mod bs_roformer_output;
pub mod commands;
pub mod demucs_cli;
pub mod demucs_output;
pub mod info;
pub mod python_tools;

pub use commands::ProcessResult;
use tauri_plugin_fs::FsExt as _;

pub fn run() {
    println!("Stemway running");
}

pub fn tauri_app() {
    let audio = audio::AudioContext::new();

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
        .manage(audio)
        .invoke_handler(tauri::generate_handler![
            commands::process_file,
            commands::open_output_folder,
            commands::get_file_size,
            commands::probe_file,
            commands::audio_load_stems,
            commands::audio_toggle_playback,
            commands::audio_set_volume,
            commands::audio_set_mute,
            commands::audio_set_solo,
            commands::audio_get_stem_count,
            commands::audio_get_stems,
            commands::audio_get_playback_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running stemway tauri app");
}
