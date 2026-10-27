use serde_json::Value as JsonValue;
use std::process::Command;

fn command(name: &str) -> Command {
    let mut cmd = Command::new(name);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd
}

fn parse_u16_from_json(val: &JsonValue) -> Option<u16> {
    match val {
        JsonValue::Number(n) => n.as_u64().map(|v| v as u16),
        JsonValue::String(s) => s.parse::<u16>().ok(),
        _ => None,
    }
}

pub fn probe_wav(path: &str) -> Result<(f64, u32, u16, u64), String> {
    let output = command("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "format=duration,size",
            "-show_entries",
            "stream=sample_rate,channels",
            "-of",
            "json",
            path,
        ])
        .output()
        .map_err(|e| format!("ffprobe failed: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Cannot read audio: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let probe: JsonValue = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    let format = probe.get("format").ok_or("No format in ffprobe output")?;

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
        .and_then(parse_u16_from_json)
        .ok_or("Could not parse channels from ffprobe")?;

    Ok((duration, sample_rate, channels, size))
}

pub fn convert_to_wav(input: &std::path::Path, output: &std::path::Path) -> Result<(), String> {
    let result = command("ffmpeg")
        .args(["-v", "error", "-nostdin", "-y", "-i"])
        .arg(input)
        .args([
            "-map",
            "0:a:0",
            "-ar",
            "44100",
            "-ac",
            "2",
            "-c:a",
            "pcm_s16le",
        ])
        .arg(output)
        .output()
        .map_err(|e| format!("FFmpeg is required: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "Audio conversion failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_test_wav() {
        let path = std::env::temp_dir().join(format!("stemway-probe-{}.wav", std::process::id()));
        let output = command("ffmpeg")
            .args([
                "-v",
                "error",
                "-y",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=0.1",
                "-ar",
                "16000",
                "-ac",
                "2",
            ])
            .arg(&path)
            .output()
            .unwrap();
        assert!(output.status.success());
        let (dur, sr, ch, size) = probe_wav(path.to_str().unwrap()).unwrap();
        assert!((dur - 0.1).abs() < 0.01);
        assert_eq!(sr, 16000);
        assert_eq!(ch, 2);
        assert!(size > 0);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn missing_input_reports_the_real_error() {
        let error = probe_wav("stemway-file-that-does-not-exist.wav").unwrap_err();
        assert!(error.contains("Cannot read audio"));
    }
}
