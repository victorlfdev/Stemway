use std::fs::File;
use std::io::{BufWriter, Write};
use crate::audio::AudioError;

pub struct AudioWriter;

#[repr(C)]
#[derive(Copy, Clone)]
struct WavHeader {
    chunk_id: [u8; 4],
    chunk_size: u32,
    format: [u8; 4],
    sub_fmt: [u8; 4],
    sub_size: u32,
    audio_format: u16,
    num_channels: u16,
    sample_rate: u32,
    bytes_per_sec: u32,
    block_align: u16,
    bits_per_sample: u16,
    data_chunk: [u8; 4],
    data_size: u32,
}

impl WavHeader {
    fn to_bytes(&self) -> [u8; 44] {
        let mut buf = [0u8; 44];
        unsafe {
            std::ptr::copy_nonoverlapping(
                self as *const WavHeader as *const u8,
                buf.as_mut_ptr(),
                44,
            );
        }
        buf
    }
}

impl AudioWriter {
    pub fn write_stereo_wav(path: &str, samples: &[f32], sample_rate: u32) -> Result<(), AudioError> {
        let file = File::create(path).map_err(|e| AudioError::Io(e))?;
        let mut writer = BufWriter::new(file);
        
        let num_samples = samples.len() / 2;
        let bits_per_sample = 16;
        let num_channels: u16 = 2;
        let block_align = num_channels * (bits_per_sample as u16 / 8);
        let data_size = (num_samples as u32) * (block_align as u32);
        let header_size = 44;
        let total_size = data_size + header_size as u32;
        
        let header = WavHeader {
            chunk_id: *b"RIFF",
            chunk_size: total_size - 8,
            format: *b"WAVE",
            sub_fmt: [0x01, 0x00, 0x00, 0x00],
            sub_size: 16,
            audio_format: 1,
            num_channels,
            sample_rate,
            bytes_per_sec: sample_rate * (num_channels as u32) * (bits_per_sample as u32 / 8),
            block_align,
            bits_per_sample,
            data_chunk: *b"data",
            data_size,
        };
        
        let bytes = header.to_bytes();
        writer.write_all(&bytes).map_err(|e| AudioError::Io(e))?;
        
        for i in (0..samples.len()).step_by(2) {
            let left = samples[i].clamp(-1.0, 1.0);
            let right = if i + 1 < samples.len() {
                samples[i + 1].clamp(-1.0, 1.0)
            } else {
                0.0
            };
            
            let left_i16 = (left * 32767.0) as i16;
            let right_i16 = (right * 32767.0) as i16;
            
            writer.write_all(&left_i16.to_le_bytes()).map_err(|e| AudioError::Io(e))?;
            writer.write_all(&right_i16.to_le_bytes()).map_err(|e| AudioError::Io(e))?;
        }
        
        writer.flush().map_err(|e| AudioError::Io(e))?;
        
        Ok(())
    }
    
    pub fn write_4_stems(
        output_dir: &str,
        stems: &[&[f32]],
        stem_names: &[&str],
        sample_rate: u32,
    ) -> Result<Vec<String>, AudioError> {
        let mut written_paths = Vec::new();
        
        for (stem, name) in stems.iter().zip(stem_names.iter()) {
            let path = format!("{}/{}.wav", output_dir, name);
            Self::write_stereo_wav(&path, stem, sample_rate)?;
            written_paths.push(path);
        }
        
        Ok(written_paths)
    }
}
