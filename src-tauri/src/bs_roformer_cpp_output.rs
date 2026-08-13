use std::fs;
use std::path::Path;

pub fn parse_output(output_dir: &str) -> Result<Vec<(String, String)>, String> {
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
