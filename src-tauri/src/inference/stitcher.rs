use std::vec::Vec;

pub struct Stitcher;

impl Stitcher {
    pub fn new() -> Self {
        Self
    }
    
    /// Merge overlapping chunks using Hann window overlap-add.
    /// Each chunk contains 4 stems. Merges each stem independently.
    pub fn merge_stems(
        &self,
        chunks: &[(usize, Vec<Vec<f32>>)]
    ) -> Vec<Vec<f32>> {
        if chunks.is_empty() {
            return vec![vec![], vec![], vec![], vec![]];
        }
        
        let chunk_size = chunks[0].1[0].len();
        let total_len = chunk_size * chunks.len();
        
        // Each stem gets its own output and weight arrays
        let mut stem_outputs: Vec<Vec<f32>> = (0..4).map(|_| vec![0.0f32; total_len]).collect();
        let mut stem_weights: Vec<Vec<f32>> = (0..4).map(|_| vec![0.0f32; total_len]).collect();
        
        let win = Self::generate_hann_window(chunk_size);
        
        let mut idx = 0;
        for (_offset, stems) in chunks {
            let stem_len = chunk_size.min(total_len.saturating_sub(idx));
            
            for i in 0..4 {
                for j in 0..stem_len {
                    let w = win[j];
                    let pos = idx + j;
                    if pos < total_len {
                        stem_outputs[i][pos] += stems[i][j] * w * w;
                        stem_weights[i][pos] += w * w;
                    }
                }
            }
            
            idx += chunk_size;
        }
        
        // Normalize each stem by its weight
        let mut output = Vec::with_capacity(4);
        for i in 0..4 {
            let mut stem = stem_outputs[i].clone();
            for j in 0..total_len {
                if stem_weights[i][j] > 1e-10 {
                    stem[j] /= stem_weights[i][j];
                }
            }
            output.push(stem);
        }
        
        output
    }
    
    /// Normalize each stem to have max absolute value <= target
    pub fn normalize(&self, stems: &mut Vec<Vec<f32>>, target: f32) -> Vec<Vec<f32>> {
        let mut output = Vec::with_capacity(4);
        
        for stem in stems.iter() {
            if stem.is_empty() {
                output.push(stem.clone());
                continue;
            }
            
            let max_val = stem.iter().fold(0.0f32, |acc, &x| {
                let abs = x.abs();
                if abs > acc { abs } else { acc }
            });
            
            let scale = if max_val > target {
                target / max_val
            } else {
                1.0
            };
            
            output.push(stem.iter().map(|&x| x * scale).collect());
        }
        
        output
    }
    
    fn generate_hann_window(size: usize) -> Vec<f32> {
        let mut window = vec![0.0f32; size];
        for i in 0..size {
            let pi = std::f32::consts::PI;
            window[i] = 0.5 * (1.0 - (2.0 * pi * i as f32 / (size as f32 - 1.0)).cos());
        }
        window
    }
}
