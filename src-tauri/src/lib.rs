pub mod audio;
pub mod inference;
pub mod utils;

pub use inference::demucs::DemucsModel;
pub use inference::chunk::ChunkProcessor;
pub use inference::get_env;

pub fn run() {
    println!("Stem Separator running");
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
    fn test_chunk_processor_empty_samples() {
        let processor = ChunkProcessor::new();
        let result = processor.process_sequential(&[], &mut |_samples| Ok(vec![vec![]; 4]));
        assert!(result.is_ok());
        let stems = result.unwrap();
        assert_eq!(stems.len(), 4);
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
}
