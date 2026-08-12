pub mod audio;
pub mod inference;
pub mod utils;
pub mod commands;

pub use inference::demucs::DemucsModel;
pub use inference::chunk::ChunkProcessor;
pub use inference::stitcher::Stitcher;
pub use inference::get_env;

pub fn run() {
    println!("Stem Separator running");
}

pub fn tauri_app() {
    use commands::AppState;
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::analyze_file,
            commands::process_audio_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running stem separator tauri app");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::writer::AudioWriter;

    #[test]
    fn test_chunk_processor_creation() {
        let processor = ChunkProcessor::new();
        assert!(processor.chunk_size > 0);
        assert!(processor.hop_size > 0);
        assert!(processor.hop_size < processor.chunk_size);
    }

    #[test]
    fn test_hann_window_values() {
        let processor = ChunkProcessor::new();
        let window = &processor.hann_window;
        
        assert!(!window.is_empty());
        assert!((window[0] - 0.0).abs() < 0.01);
        assert!((window[window.len() - 1] - 0.0).abs() < 0.01);
        
        let mid = window.len() / 2;
        assert!(window[mid] > 0.9);
    }

    #[test]
    fn test_chunk_processor_chunk_size() {
        let processor = ChunkProcessor::new();
        
        assert_eq!(processor.chunk_size, 343980);
        assert_eq!(processor.hop_size, 257985);
        assert_eq!(processor.overlap, 85995);
    }

    #[test]
    fn test_stitcher_creation() {
        let stitcher = Stitcher::new();
        let chunks = vec![(0, vec![vec![1.0f32; 100]; 4])];
        let result = stitcher.merge_stems(&chunks);
        assert!(result[0].len() > 0);
    }

    #[test]
    fn test_stitcher_merge_simple() {
        let stitcher = Stitcher::new();
        let chunk_size = 100;
        let chunks = vec![(0, vec![vec![0.5f32; chunk_size]; 4])];
        let result = stitcher.merge_stems(&chunks);
        
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].len(), chunk_size);
        assert!((result[0][50] - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_stitcher_merge_two_chunks() {
        let stitcher = Stitcher::new();
        let chunk_size = 100;
        let chunks = vec![
            (0, vec![vec![0.4f32; chunk_size]; 4]),
            (chunk_size, vec![vec![0.6f32; chunk_size]; 4]),
        ];
        let result = stitcher.merge_stems(&chunks);
        
        assert_eq!(result[0].len(), 200);
        assert!((result[0][50] - 0.4).abs() < 0.05);
        assert!((result[0][150] - 0.6).abs() < 0.05);
    }

    #[test]
    fn test_stitcher_normalize() {
        let stitcher = Stitcher::new();
        let mut stems = vec![vec![1.0f32, 2.0, 3.0, -4.0]];
        let result = stitcher.normalize(&mut stems, 0.99);
        
        assert_eq!(result[0].len(), 4);
        assert!(result[0].iter().all(|&x| x.abs() <= 1.0));
        assert!(result[0].iter().any(|&x| x.abs() >= 0.9));
    }

    #[test]
    fn test_write_empty_wav() {
        let output_path = "test_empty.wav";
        let samples: Vec<f32> = vec![];
        
        let result = AudioWriter::write_stereo_wav(output_path, &samples, 44100);
        
        assert!(result.is_ok());
        
        let file = std::fs::metadata(output_path);
        assert!(file.is_ok());
    }

    #[test]
    fn test_write_stereo_wav() {
        let output_path = "test_stereo.wav";
        let samples: Vec<f32> = vec![
            0.5, -0.5,  0.3, -0.3,  0.0, 0.0,
            0.1, -0.1,  0.9, -0.9,  0.7, -0.7
        ];
        
        let result = AudioWriter::write_stereo_wav(output_path, &samples, 44100);
        assert!(result.is_ok());
        
        let metadata = std::fs::metadata(output_path);
        assert!(metadata.is_ok());
        let size = metadata.unwrap().len();
        assert!(size > 0);
    }

    #[test]
    fn test_end_to_end_pipeline() {
        use crate::audio::AudioReader;
        
        let input_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-music.mp3");
        
        let reader = AudioReader::new(&input_path)
            .expect("Failed to read test-music.mp3");
        
        let duration = reader.duration_secs;
        let sample_rate = reader.sample_rate;
        let samples = reader.get_samples();
        
        assert!(!samples.is_empty());
        assert!(duration > 0.0);
        
        let target_sr = 16000;
        let ratio = if sample_rate as f64 > target_sr as f64 {
            target_sr as f64 / sample_rate as f64
        } else {
            1.0
        };
        let stereo_len = samples.len() / 2;
        let new_len = (stereo_len as f64 * ratio) as usize;
        let mut resampled = vec![0.0f32; new_len * 2];
        for i in (0..samples.len()).step_by(2) {
            let idx = (i as f64 / 2.0 * ratio) as usize;
            if idx < new_len {
                resampled[idx * 2] = samples[i];
                if i + 1 < samples.len() {
                    resampled[idx * 2 + 1] = samples[i + 1];
                }
            }
        }
        
        let processor = ChunkProcessor::new();
        let stems = processor.process_sequential_stereo(&resampled, &mut |chunk_data| {
            let mut model = DemucsModel::new()?;
            let result = model.separate(chunk_data, target_sr)?;
            Ok(result)
        }).expect("Inference failed");
        
        assert_eq!(stems.len(), 4);
        assert!(!stems[0].is_empty());
        
        let normalized = {
            let mut stitcher = Stitcher::new();
            stitcher.normalize(&mut stems.clone(), 0.99)
        };
        
        let output_base = format!(
            "{}/test_output_stem",
            env!("CARGO_MANIFEST_DIR")
        );
        
        for (i, stem) in normalized.iter().enumerate() {
            let stem_name = match i {
                0 => "_vocals",
                1 => "_bass",
                2 => "_drums",
                3 => "_other",
                _ => "",
            };
            let path = format!("{}_{}.wav", output_base, stem_name);
            let result = AudioWriter::write_stereo_wav(&path, stem, target_sr);
            assert!(result.is_ok(), "Failed to write stem {}: {}", i, stem_name);
            
            let metadata = std::fs::metadata(&path);
            assert!(metadata.is_ok(), "File not created: {}", path);
            let size = metadata.unwrap().len();
            assert!(size > 0, "File is empty: {}", path);
        }
    }
}
