use std::vec::Vec;

const CHUNK_SAMPLES: usize = 343980;
const OVERLAP_RATIO: f64 = 0.25;

pub struct ChunkProcessor {
    pub chunk_size: usize,
    pub hop_size: usize,
    pub hann_window: Vec<f32>,
    pub overlap: usize,
}

impl ChunkProcessor {
    pub fn new() -> Self {
        let hop_samples = (CHUNK_SAMPLES as f64 * (1.0 - OVERLAP_RATIO)) as usize;
        let overlap = CHUNK_SAMPLES - hop_samples;
        
        let hann_window = Self::generate_hann_window(CHUNK_SAMPLES);
        
        Self {
            chunk_size: CHUNK_SAMPLES,
            hop_size: hop_samples,
            hann_window,
            overlap,
        }
    }
    
    fn generate_hann_window(size: usize) -> Vec<f32> {
        let mut window = vec![0.0f32; size];
        for i in 0..size {
            let pi = std::f32::consts::PI;
            window[i] = 0.5 * (1.0 - (2.0 * pi * i as f32 / (size as f32 - 1.0)).cos());
        }
        window
    }
    
    pub fn process_sequential(
        &self,
        samples: &[f32],
        inference: &mut dyn FnMut(&[f32]) -> Result<Vec<Vec<f32>>, String>,
    ) -> Result<Vec<Vec<f32>>, String> {
        if samples.is_empty() {
            return Ok(vec![vec![], vec![], vec![], vec![]]);
        }
        
        let chunk = self.chunk_size;
        let hop = self.hop_size;
        
        let mut chunk_results: Vec<(usize, Vec<Vec<f32>>)> = Vec::new();
        let mut offset = 0;
        
        while offset < samples.len() {
            let end = (offset + chunk).min(samples.len());
            let chunk_data = &samples[offset..end];
            
            let padded = if chunk_data.len() < chunk {
                let mut p = chunk_data.to_vec();
                p.resize(chunk, 0.0);
                p
            } else {
                chunk_data.to_vec()
            };
            
            match inference(&padded) {
                Ok(stems) => chunk_results.push((offset, stems)),
                Err(e) => return Err(e),
            }
            
            offset += hop;
        }
        
        self.merge_chunks(chunk_results)
    }
    
    pub fn process_sequential_stereo(
        &self,
        samples: &[f32],
        inference: &mut dyn FnMut(&[f32]) -> Result<Vec<Vec<f32>>, String>,
    ) -> Result<Vec<Vec<f32>>, String> {
        if samples.len() < 2 {
            return Ok(vec![vec![], vec![], vec![], vec![]]);
        }
        
        let mono_len = samples.len() / 2;
        let mut mono = vec![0.0f32; mono_len];
        for i in (0..samples.len()).step_by(2) {
            let ch0 = samples[i];
            let ch1 = if i + 1 < samples.len() { samples[i + 1] } else { 0.0 };
            mono[i / 2] = (ch0 + ch1) * 0.5;
        }
        
        let chunk = self.chunk_size;
        let hop = self.hop_size;
        
        let mut chunk_results: Vec<(usize, Vec<Vec<f32>>)> = Vec::new();
        let mut offset = 0;
        
        while offset < mono_len {
            let end = (offset + chunk).min(mono_len);
            let chunk_mono = &mono[offset..end];
            
            let mut stereo_chunk = vec![0.0f32; chunk * 2];
            for i in 0..chunk_mono.len() {
                stereo_chunk[i * 2] = chunk_mono[i];
                stereo_chunk[i * 2 + 1] = chunk_mono[i];
            }
            
            match inference(&stereo_chunk) {
                Ok(stems) => chunk_results.push((offset, stems)),
                Err(e) => return Err(e),
            }
            
            offset += hop;
        }
        
        self.merge_stem_chunks(chunk_results)
    }
    
    fn merge_chunks(&self, results: Vec<(usize, Vec<Vec<f32>>)>) -> Result<Vec<Vec<f32>>, String> {
        if results.is_empty() {
            return Ok(vec![vec![], vec![], vec![], vec![]]);
        }
        
        let total_len = self.hop_size * results.len();
        
        let mut output = vec![vec![0.0f32; total_len]; 4];
        let mut weight = vec![0.0f32; total_len];
        
        let chunk = self.chunk_size;
        let hop = self.hop_size;
        let _overlap = self.overlap;
        let window = &self.hann_window;
        
        let mut idx = 0;
        for (_offset, stems) in &results {
            let take = chunk.min(total_len - idx);
            
            for i in 0..4 {
                let stem_len = stems[i].len().min(take);
                
                let out_start = idx;
                let _out_end = idx + stem_len;
                
                for j in 0..stem_len {
                    let w = window[j];
                    let pos = out_start + j;
                    if pos < total_len {
                        output[i][pos] += stems[i][j] * w * w;
                        weight[pos] += w * w;
                    }
                }
            }
            
            idx += hop;
        }
        
        for i in 0..4 {
            for j in 0..total_len {
                if weight[j] > 1e-10 {
                    output[i][j] /= weight[j];
                }
            }
        }
        
        Ok(output)
    }
    
    fn merge_stem_chunks(
        &self,
        results: Vec<(usize, Vec<Vec<f32>>)>,
    ) -> Result<Vec<Vec<f32>>, String> {
        if results.is_empty() {
            return Ok(vec![vec![], vec![], vec![], vec![]]);
        }
        
        let total_len = self.hop_size * results.len();
        
        let mut output = vec![vec![0.0f32; total_len]; 4];
        let mut weight = vec![0.0f32; total_len];
        
        let chunk = self.chunk_size;
        let hop = self.hop_size;
        let window = &self.hann_window;
        
        let mut idx = 0;
        for (_offset, stems) in &results {
            let take = chunk.min(total_len - idx);
            
            for i in 0..4 {
                let stem_len = stems[i].len().min(take);
                
                let out_start = idx;
                
                for j in 0..stem_len {
                    let w = window[j];
                    let pos = out_start + j;
                    if pos < total_len {
                        output[i][pos] += stems[i][j] * w * w;
                        weight[pos] += w * w;
                    }
                }
            }
            
            idx += hop;
        }
        
        for i in 0..4 {
            for j in 0..total_len {
                if weight[j] > 1e-10 {
                    output[i][j] /= weight[j];
                }
            }
        }
        
        Ok(output)
    }
}
