use ort::session::Session;
use ort::value::Tensor;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;

pub struct DemucsModel {
    session: Option<Session>,
    model_path: PathBuf,
}

const DEMUCS_URL: &str = "https://github.com/pykeio/ort/releases/download/model-onnx/demucs-2s.toml";
const MODEL_NAME: &str = "htdemucs.onnx";

impl DemucsModel {
    pub fn new() -> Result<Self, String> {
        let cache_dir = Self::get_cache_dir();
        let model_path = cache_dir.join(MODEL_NAME);
        
        if !model_path.exists() {
            Self::download_model(&model_path).map_err(|e| format!("Failed to download model: {}", e))?;
        }
        
        Ok(Self {
            session: None,
            model_path,
        })
    }
    
    fn get_cache_dir() -> PathBuf {
        if let Some(dir) = directories::ProjectDirs::from("", "", "stem-separator") {
            dir.cache_dir().to_path_buf()
        } else {
            PathBuf::from(".ort_cache")
        }
    }
    
    fn download_model(path: &Path) -> Result<(), String> {
        println!("Downloading HTDemucs model (~316 MB)...");
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create cache directory: {}", e))?;
        
        let response = ureq::get(DEMUCS_URL).call().map_err(|e| format!("HTTP error: {}", e))?;
        let mut file = fs::File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
        let mut body = Vec::new();
        let mut reader = response.into_reader();
        reader.read_to_end(&mut body).map_err(|e| format!("Failed to read response: {}", e))?;
        file.write_all(&body).map_err(|e| format!("Failed to write file: {}", e))?;
        
        Ok(())
    }
    
    fn get_session(&mut self) -> Result<&mut Session, String> {
        if self.session.is_some() {
            return Ok(self.session.as_mut().unwrap());
        }
        
        let session = Session::builder()
            .map_err(|e| format!("Session builder error: {}", e))?
            .commit_from_file(&self.model_path)
            .map_err(|e| format!("Failed to load ONNX model: {}", e))?;
        
        self.session = Some(session);
        Ok(self.session.as_mut().unwrap())
    }
    
    pub fn separate(
        &mut self,
        samples: &[f32],
        _sample_rate: u32,
    ) -> Result<Vec<Vec<f32>>, String> {
        if samples.len() < 2 {
            return Ok(vec![vec![], vec![], vec![], vec![]]);
        }
        
        let stereo_len = samples.len();
        
        let input_tensor = Tensor::<f32>::from_array(
            ([1usize, 2usize, stereo_len / 2], samples.to_vec())
        ).map_err(|e| format!("Tensor creation error: {}", e))?;
        
        let session = self.get_session()?;
        let outputs = session.run(ort::inputs![input_tensor])
            .map_err(|e| format!("Inference error: {}", e))?;
        
        let output = &outputs[0];
        let (_, data) = output.try_extract_tensor::<f32>().map_err(|e| format!("Extract: {}", e))?;
        let flat: Vec<f32> = data.to_vec();
        
        let out_samples = flat.len() / (4 * 2);
        
        let mut stems = Vec::new();
        for s in 0..4 {
            let mut stem = vec![0.0f32; out_samples];
            for t in 0..out_samples {
                let idx = s * 2 * out_samples + t;
                if idx < flat.len() {
                    stem[t] = flat[idx];
                }
            }
            stems.push(stem);
        }
        
        Ok(stems)
    }
}
