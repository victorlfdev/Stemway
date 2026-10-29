pub mod audio;
pub mod bs_roformer_cli;
pub mod bs_roformer_cpp_cli;
pub mod bs_roformer_cpp_output;
pub mod bs_roformer_output;
pub mod commands;
pub mod demucs_cli;
pub mod demucs_output;
pub mod info;
pub mod jobs;
pub mod library;
pub mod media;
pub mod models;
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
            jobs::init(app.handle().clone());
            let scope = app.fs_scope();
            let cache_dir = directories::ProjectDirs::from("", "", "stemway")
                .map(|d| d.cache_dir().to_path_buf())
                .expect("failed to get cache dir");
            scope.allow_directory(&cache_dir, true)?;
            scope.allow_directory(crate::library::data_dir()?, true)?;
            let temp_dir = std::env::temp_dir();
            scope.allow_directory(&temp_dir, true)?;
            Ok(())
        })
        .manage(audio)
        .on_window_event(|_, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                let _ = jobs::cancel();
            }
        })
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
            library::list_projects,
            library::save_project,
            library::read_workspace,
            library::write_workspace,
            library::audio_waveforms,
            library::audio_transport,
            library::audio_seek,
            library::audio_set_loop,
            media::render_study,
            media::export_audio,
            jobs::cancel_operation,
            models::model_status,
            models::hardware_info,
            models::prepare_model,
            models::remove_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running stemway tauri app");
}
