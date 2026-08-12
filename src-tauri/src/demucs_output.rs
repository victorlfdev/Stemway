use std::fs;
use std::path::Path;

pub fn parse_demucs_output(output_dir: &str, track_name: &str) -> Result<Vec<(String, String)>, String> {
    let demucs_dir = format!("{}/htdemucs_6s/{}", output_dir, track_name);
    let stems_dir = Path::new(&demucs_dir);
    
    if !stems_dir.exists() {
        return Err(format!("Demucs output directory not found: {}", demucs_dir));
    }
    
    let mut stems = Vec::new();
    
    for entry in fs::read_dir(stems_dir).map_err(|e| format!("Failed to read demucs output: {}", e))? {
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
    
    stems.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(stems)
}
