use std::vec::Vec;

pub struct ChunkProcessor {
    pub chunk_size: usize,
    pub hop_size: usize,
}

const DEFAULT_SR: u32 = 16000;
const CHUNK_DURATION: f64 = 10.0;
const OVERLAP: f64 = 0.25;

impl ChunkProcessor {
    pub fn new() -> Self {
        Self {
            chunk_size: (DEFAULT_SR as f64 * CHUNK_DURATION) as usize,
            hop_size: (DEFAULT_SR as f64 * CHUNK_DURATION * (1.0 - OVERLAP)) as usize,
        }
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
        
        let mut results = Vec::new();
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
                Ok(stems) => results.push((offset, stems)),
                Err(e) => return Err(e),
            }
            
            offset += hop;
        }
        
        self.merge_results(results)
    }
    
    fn merge_results(&self, results: Vec<(usize, Vec<Vec<f32>>)>) -> Result<Vec<Vec<f32>>, String> {
        if results.is_empty() {
            return Ok(vec![vec![], vec![], vec![], vec![]]);
        }
        
        let max_len = results.iter().map(|(_, s)| s[0].len()).max().unwrap_or(0);
        let output_size = self.hop_size.min(max_len);
        
        let mut output = vec![vec![0.0f32; output_size]; 4];
        
        let mut pos = 0;
        for (_, stems) in &results {
            if pos >= output_size {
                break;
            }
            
            let take = (output_size - pos).min(stems[0].len());
            
            for i in 0..4 {
                output[i][pos..pos + take].copy_from_slice(&stems[i][..take]);
            }
            
            pos += take;
        }
        
        Ok(output)
    }
}
