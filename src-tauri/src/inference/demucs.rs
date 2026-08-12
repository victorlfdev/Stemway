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
const MODEL_NAME: &str = "htdemucs_2s.onnx";

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
        sample_rate: u32,
    ) -> Result<Vec<Vec<f32>>, String> {
        if samples.len() < 4 {
            return Ok(vec![vec![], vec![], vec![], vec![]]);
        }
        
        let target_sr = 16000;
        let ratio = if sample_rate as f64 > target_sr as f64 {
            target_sr as f64 / sample_rate as f64
        } else {
            1.0
        };
        
        let new_len = (samples.len() as f64 * ratio) as usize;
        let mut resampled = vec![0.0f32; new_len];
        
        for i in 0..samples.len() {
            let idx = (i as f64 * ratio) as usize;
            if idx < new_len {
                resampled[idx] = samples[i];
            }
        }
        
        let input_tensor = Tensor::<f32>::from_array(
            ([1usize, 2usize, new_len], resampled.into_boxed_slice())
        ).map_err(|e| format!("Tensor creation error: {}", e))?;
        
        let session = self.get_session()?;
        let outputs = session.run(ort::inputs![input_tensor])
            .map_err(|e| format!("Inference error: {}", e))?;
        
        let output_0 = &outputs[0];
        let output_1 = &outputs[1];
        let output_2 = &outputs[2];
        let output_3 = &outputs[3];
        
        let (_, data0) = output_0.try_extract_tensor::<f32>().map_err(|e| format!("Extract 0: {}", e))?;
        let (_, data1) = output_1.try_extract_tensor::<f32>().map_err(|e| format!("Extract 1: {}", e))?;
        let (_, data2) = output_2.try_extract_tensor::<f32>().map_err(|e| format!("Extract 2: {}", e))?;
        let (_, data3) = output_3.try_extract_tensor::<f32>().map_err(|e| format!("Extract 3: {}", e))?;
        
        Ok(vec![data0.to_vec(), data1.to_vec(), data2.to_vec(), data3.to_vec()])
    }
}
