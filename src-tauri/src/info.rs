use std::process::Command;
use serde_json::Value as JsonValue;

fn parse_u16_from_json(val: &JsonValue) -> Option<u16> {
    match val {
        JsonValue::Number(n) => n.as_u64().map(|v| v as u16),
        JsonValue::String(s) => s.parse::<u16>().ok(),
        _ => None,
    }
}

pub fn probe_wav(path: &str) -> Result<(f64, u32, u16, u64), String> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-show_entries", "format=duration,size",
            "-show_entries", "stream=sample_rate,channels",
            "-of", "json",
            path,
        ])
        .output()
        .map_err(|e| format!("ffprobe failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let probe: JsonValue = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    let format = probe
        .get("format")
        .ok_or("No format in ffprobe output")?;

    let duration = format
        .get("duration")
        .and_then(|v| v.as_str())
        .and_then(|d| d.parse::<f64>().ok())
        .ok_or("Could not parse duration from ffprobe")?;

    let size = format
        .get("size")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let streams = probe
        .get("streams")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .ok_or("No streams in ffprobe output")?;

    let sample_rate = streams
        .get("sample_rate")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or("Could not parse sample_rate from ffprobe")?;

    let channels = streams
        .get("channels")
        .and_then(|v| parse_u16_from_json(v))
        .ok_or("Could not parse channels from ffprobe")?;

    Ok((duration, sample_rate, channels, size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_test_wav() {
        let result = probe_wav("/tmp/demucs_final_test/htdemucs_6s/test_final/bass.wav");
        assert!(result.is_ok());
        let (dur, sr, ch, size) = result.unwrap();
        assert!(dur > 9.0);
        assert!(dur < 11.0);
        assert_eq!(sr, 44100);
        assert_eq!(ch, 2);
        assert!(size > 0);
    }
}
