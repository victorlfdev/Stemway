use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum ModelType {
    Demucs,
    BsRoformer,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Demucs => write!(f, "demucs"),
            ModelType::BsRoformer => write!(f, "bs-roformer"),
        }
    }
}

pub struct OutputDir {
    pub path: String,
    pub model: ModelType,
}

pub fn resolve_output_dir(output_dir: &str, track_name: &str, model: &str) -> Result<OutputDir, String> {
    let dir = match model {
        "bs-roformer" => {
            let candidate = format!("{}/bs-roformer-infer/{}", output_dir, track_name);
            if Path::new(&candidate).exists() {
                candidate
            } else {
                format!("{}/outputs/{}", output_dir, track_name)
            }
        }
        _ => {
            format!("{}/htdemucs_6s/{}", output_dir, track_name)
        }
    };

    if !Path::new(&dir).exists() {
        return Err(format!("Output directory not found: {}", dir));
    }

    let model_type = if model == "bs-roformer" {
        ModelType::BsRoformer
    } else {
        ModelType::Demucs
    };

    Ok(OutputDir { path: dir, model: model_type })
}

pub fn parse_output(output_dir: &str, _model: &str) -> Result<Vec<(String, String)>, String> {
    let stems_dir = Path::new(output_dir);

    if !stems_dir.exists() {
        return Err(format!("Output directory not found: {}", output_dir));
    }

    let mut stems: Vec<(String, String)> = Vec::new();

    for entry in fs::read_dir(stems_dir)
        .map_err(|e| format!("Failed to read output directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("wav") {
            let stem_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("Invalid stem filename")?
                .to_string();

            let file_path = path
                .to_str()
                .ok_or("Invalid file path")?
                .to_string();

            stems.push((stem_name, file_path));
        }
    }

    stems.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    Ok(stems)
}
