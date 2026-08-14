use std::fs;
use std::path::Path;

const STEM_NAMES: [&str; 6] = [
    "bass",
    "drums",
    "other",
    "vocals",
    "guitar",
    "piano",
];

pub fn parse_multi_stem_output(output_dir: &str) -> Result<Vec<(String, String)>, String> {
    let stems_dir = Path::new(output_dir);

    if !stems_dir.exists() {
        return Err(format!("Output directory not found: {}", output_dir));
    }

    let mut stems: Vec<(String, String)> = Vec::new();
    let mut stem_index_map: std::collections::HashMap<u32, String> = std::collections::HashMap::new();

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

            if let Some(stem_idx) = extract_stem_index(&stem_name) {
                let friendly_name = match stem_idx {
                    0 => "bass".to_string(),
                    1 => "drums".to_string(),
                    2 => "other".to_string(),
                    3 => "vocals".to_string(),
                    4 => "guitar".to_string(),
                    5 => "piano".to_string(),
                    _ => format!("stem_{}", stem_idx),
                };
                let file_path = path
                    .to_str()
                    .ok_or("Invalid file path")?
                    .to_string();
                stem_index_map.insert(stem_idx, friendly_name.clone());
                stems.push((friendly_name, file_path));
            }
        }
    }

    stems.sort_by(|a, b| {
        let a_idx = get_stem_order(&a.0);
        let b_idx = get_stem_order(&b.0);
        a_idx.cmp(&b_idx)
    });

    Ok(stems)
}

fn extract_stem_index(name: &str) -> Option<u32> {
    let parts: Vec<&str> = name.rsplit('_').collect();
    if parts.len() >= 2 {
        parts[0].parse::<u32>().ok()
    } else {
        None
    }
}

fn get_stem_order(name: &str) -> u32 {
    for (i, stem) in STEM_NAMES.iter().enumerate() {
        if *stem == name {
            return i as u32;
        }
    }
    99
}
