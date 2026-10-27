use rodio::{Decoder, Sink};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;

pub struct AudioContext {
    pub tx: std::sync::Mutex<Option<mpsc::SyncSender<AudioCommand>>>,
}

type StemState = (String, bool, bool, f32);

pub enum AudioCommand {
    GetPlaybackState {
        response: mpsc::SyncSender<Result<bool, String>>,
    },
    LoadStems {
        stem_paths: Vec<(String, String)>,
        response: mpsc::SyncSender<Result<Vec<String>, String>>,
    },
    TogglePlayback {
        response: mpsc::SyncSender<Result<(), String>>,
    },
    SetVolume {
        id: String,
        volume: f32,
        response: mpsc::SyncSender<Result<(), String>>,
    },
    SetMute {
        id: String,
        muted: bool,
        response: mpsc::SyncSender<Result<(), String>>,
    },
    SetSolo {
        id: String,
        solo: bool,
        response: mpsc::SyncSender<Result<(), String>>,
    },
    GetStemCount {
        response: mpsc::SyncSender<Result<usize, String>>,
    },
    GetStems {
        response: mpsc::SyncSender<Result<Vec<StemState>, String>>,
    },
}

pub struct AudioManager {
    _stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    stems: HashMap<String, StemTrack>,
    is_playing: bool,
    solo_stems: HashSet<String>,
}

struct StemTrack {
    sink: Sink,
    path: String,
    base_volume: f32,
    is_muted: bool,
    is_soloed: bool,
}

impl AudioManager {
    fn new(stream: rodio::OutputStream, stream_handle: rodio::OutputStreamHandle) -> Self {
        Self {
            _stream: stream,
            stream_handle,
            stems: HashMap::new(),
            is_playing: false,
            solo_stems: HashSet::new(),
        }
    }

    fn load_stems(&mut self, stem_paths: Vec<(String, String)>) -> Result<Vec<String>, String> {
        let mut new_stems: HashMap<String, StemTrack> = HashMap::new();
        let mut loaded_ids = Vec::new();
        let path_count = stem_paths.len();

        for (id, path) in &stem_paths {
            match load_stem_track(&self.stream_handle, id, path) {
                Ok(track) => {
                    loaded_ids.push(id.clone());
                    new_stems.insert(id.clone(), track);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        if loaded_ids.len() == path_count {
            for track in new_stems.values() {
                track.sink.pause();
            }
            self.stems = new_stems;
            self.is_playing = false;
            self.solo_stems.clear();
            Ok(loaded_ids)
        } else {
            Err("Failed to load some stems".into())
        }
    }

    fn toggle_playback(&mut self) -> Result<(), String> {
        if self.stems.is_empty() {
            return Err("No stems loaded".into());
        }
        if self.stems.values().all(|track| track.sink.empty()) {
            for track in self.stems.values_mut() {
                let file = File::open(&track.path).map_err(|e| e.to_string())?;
                let decoder = Decoder::new(BufReader::new(file)).map_err(|e| e.to_string())?;
                track.sink.pause();
                track.sink.append(decoder);
            }
            self.is_playing = false;
        }
        self.is_playing = !self.is_playing;
        // Muted and non-soloed tracks must keep advancing to stay in sync.
        for track in self.stems.values() {
            if self.is_playing {
                track.sink.play();
            } else {
                track.sink.pause();
            }
        }
        Ok(())
    }

    fn update_gains(&self) {
        for (id, track) in &self.stems {
            track.sink.set_volume(effective_volume(
                track.base_volume,
                track.is_muted,
                !self.solo_stems.is_empty(),
                self.solo_stems.contains(id),
            ));
        }
    }

    fn set_volume(&mut self, id: &str, volume: f32) -> Result<(), String> {
        if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
            return Err("Volume must be between 0 and 1".into());
        }
        self.stems
            .get_mut(id)
            .ok_or_else(|| format!("Stem not found: {id}"))?
            .base_volume = volume;
        self.update_gains();
        Ok(())
    }

    fn set_mute(&mut self, id: &str, muted: bool) -> Result<(), String> {
        self.stems
            .get_mut(id)
            .ok_or_else(|| format!("Stem not found: {id}"))?
            .is_muted = muted;
        self.update_gains();
        Ok(())
    }

    fn set_solo(&mut self, id: &str, solo: bool) -> Result<(), String> {
        self.stems
            .get_mut(id)
            .ok_or_else(|| format!("Stem not found: {id}"))?
            .is_soloed = solo;
        if solo {
            self.solo_stems.insert(id.to_owned());
        } else {
            self.solo_stems.remove(id);
        }
        self.update_gains();
        Ok(())
    }

    fn get_stem_count(&self) -> usize {
        self.stems.len()
    }

    fn get_stems(&self) -> Vec<(String, bool, bool, f32)> {
        self.stems
            .iter()
            .map(|(id, track)| {
                (
                    id.clone(),
                    track.is_muted,
                    track.is_soloed,
                    track.base_volume,
                )
            })
            .collect()
    }
}

fn load_stem_track(
    stream_handle: &rodio::OutputStreamHandle,
    _id: &str,
    path: &str,
) -> Result<StemTrack, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    let source = Decoder::new(reader).map_err(|e| format!("Failed to decode audio: {}", e))?;

    let sink = Sink::try_new(stream_handle).map_err(|e| format!("Failed to create sink: {}", e))?;
    sink.pause();
    sink.append(source);

    Ok(StemTrack {
        sink,
        path: path.to_string(),
        base_volume: 1.0,
        is_muted: false,
        is_soloed: false,
    })
}

impl AudioContext {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::sync_channel(32);

        std::thread::spawn(move || {
            let (stream, stream_handle) = match rodio::OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to open audio output: {}", e);
                    return;
                }
            };

            let mut manager = AudioManager::new(stream, stream_handle);

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    AudioCommand::GetPlaybackState { response } => {
                        if manager.stems.values().all(|track| track.sink.empty()) {
                            manager.is_playing = false;
                        }
                        let _ = response.send(Ok(manager.is_playing));
                    }
                    AudioCommand::LoadStems {
                        stem_paths,
                        response,
                    } => {
                        eprintln!("[Audio Thread] LoadStems → {} stems", stem_paths.len());
                        let result = manager.load_stems(stem_paths);
                        match &result {
                            Ok(ids) => {
                                eprintln!("[Audio Thread] LoadStems → OK ({} stems)", ids.len())
                            }
                            Err(e) => eprintln!("[Audio Thread] LoadStems → ERR: {}", e),
                        }
                        let _ = response.send(result);
                    }
                    AudioCommand::TogglePlayback { response } => {
                        eprintln!("[Audio Thread] TogglePlayback");
                        let result = manager.toggle_playback();
                        match &result {
                            Ok(()) => eprintln!("[Audio Thread] TogglePlayback → OK"),
                            Err(e) => eprintln!("[Audio Thread] TogglePlayback → ERR: {}", e),
                        }
                        let _ = response.send(result);
                    }
                    AudioCommand::SetVolume {
                        id,
                        volume,
                        response,
                    } => {
                        eprintln!("[Audio Thread] SetVolume → id={}, vol={:.2}", id, volume);
                        let result = manager.set_volume(&id, volume);
                        match &result {
                            Ok(()) => eprintln!("[Audio Thread] SetVolume → OK"),
                            Err(e) => eprintln!("[Audio Thread] SetVolume → ERR: {}", e),
                        }
                        let _ = response.send(result);
                    }
                    AudioCommand::SetMute {
                        id,
                        muted,
                        response,
                    } => {
                        eprintln!("[Audio Thread] SetMute → id={}, muted={}", id, muted);
                        let result = manager.set_mute(&id, muted);
                        match &result {
                            Ok(()) => eprintln!("[Audio Thread] SetMute → OK"),
                            Err(e) => eprintln!("[Audio Thread] SetMute → ERR: {}", e),
                        }
                        let _ = response.send(result);
                    }
                    AudioCommand::SetSolo { id, solo, response } => {
                        eprintln!("[Audio Thread] SetSolo → id={}, solo={}", id, solo);
                        let result = manager.set_solo(&id, solo);
                        match &result {
                            Ok(()) => eprintln!("[Audio Thread] SetSolo → OK"),
                            Err(e) => eprintln!("[Audio Thread] SetSolo → ERR: {}", e),
                        }
                        let _ = response.send(result);
                    }
                    AudioCommand::GetStemCount { response } => {
                        let count = manager.get_stem_count();
                        let _ = response.send(Ok(count));
                    }
                    AudioCommand::GetStems { response } => {
                        let stems = manager.get_stems();
                        let _ = response.send(Ok(stems));
                    }
                }
            }
        });

        AudioContext {
            tx: std::sync::Mutex::new(Some(tx)),
        }
    }
}

fn effective_volume(volume: f32, muted: bool, any_solo: bool, soloed: bool) -> f32 {
    if muted || (any_solo && !soloed) {
        0.0
    } else {
        volume
    }
}

impl Default for AudioContext {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mute_and_solo_preserve_user_volume() {
        assert_eq!(super::effective_volume(0.4, false, false, false), 0.4);
        assert_eq!(super::effective_volume(0.4, true, false, false), 0.0);
        assert_eq!(super::effective_volume(0.4, false, true, false), 0.0);
        assert_eq!(super::effective_volume(0.4, false, true, true), 0.4);
        assert_eq!(super::effective_volume(0.4, true, true, true), 0.0);
    }
}
