pub struct ChunkProcessor;

impl ChunkProcessor {
    pub fn new() -> Self {
        Self
    }
    
    pub fn split(&self, _samples: &[f32]) -> Vec<Vec<f32>> {
        vec![_samples.to_vec()]
    }
}
