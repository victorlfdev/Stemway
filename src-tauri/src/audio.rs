use rodio::{Decoder, Sink};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;

pub struct AudioContext {
    pub tx: std::sync::Mutex<Option<mpsc::SyncSender<AudioCommand>>>,
}

pub enum AudioCommand {
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
        response: mpsc::SyncSender<Result<Vec<(String, bool, bool, f32)>, String>>,
    },
}

pub struct AudioManager {
    stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    stems: HashMap<String, StemTrack>,
    is_playing: bool,
    solo_stems: HashSet<String>,
    pre_solo_states: HashMap<String, (f32, bool)>,
}

struct StemTrack {
    sink: Sink,
    id: String,
    base_volume: f32,
    is_muted: bool,
    is_soloed: bool,
}

impl AudioManager {
    fn new(stream: rodio::OutputStream, stream_handle: rodio::OutputStreamHandle) -> Self {
        Self {
            stream,
            stream_handle,
            stems: HashMap::new(),
            is_playing: false,
            solo_stems: HashSet::new(),
            pre_solo_states: HashMap::new(),
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
            for (_id, track) in &new_stems {
                track.sink.pause();
            }
            self.stems = new_stems;
            self.is_playing = false;
            self.solo_stems.clear();
            self.pre_solo_states.clear();
            Ok(loaded_ids)
        } else {
            Err("Failed to load some stems".into())
        }
    }

    fn toggle_playback(&mut self) -> Result<(), String> {
        if self.is_playing {
            for track in self.stems.values_mut() {
                track.sink.pause();
            }
        } else {
            for track in self.stems.values_mut() {
                if !track.is_muted
                    && (self.solo_stems.is_empty() || self.solo_stems.contains(&track.id))
                {
                    track.sink.play();
                }
            }
        }
        self.is_playing = !self.is_playing;
        Ok(())
    }

    fn set_volume(&mut self, id: &str, volume: f32) -> Result<(), String> {
        if let Some(track) = self.stems.get_mut(id) {
            let was_soloed = self.solo_stems.contains(id);
            track.base_volume = volume;
            track.is_muted = false;
            track.is_soloed = was_soloed;
            let effective_vol = if self.solo_stems.is_empty() || self.solo_stems.contains(id) {
                volume
            } else {
                0.0
            };
            track.sink.set_volume(effective_vol);
            if self.is_playing && !track.is_muted {
                track.sink.play();
            }
            Ok(())
        } else {
            Err(format!("Stem not found: {}", id))
        }
    }

    fn set_mute(&mut self, id: &str, muted: bool) -> Result<(), String> {
        if let Some(track) = self.stems.get_mut(id) {
            let was_soloed = self.solo_stems.contains(id);
            track.is_muted = muted;
            track.is_soloed = was_soloed;

            if muted {
                track.sink.set_volume(0.0);
            } else {
                let vol = if self.solo_stems.is_empty() || self.solo_stems.contains(id) {
                    track.base_volume
                } else {
                    0.0
                };
                track.sink.set_volume(vol);
                if self.is_playing {
                    track.sink.play();
                }
            }
            Ok(())
        } else {
            Err(format!("Stem not found: {}", id))
        }
    }

    fn set_solo(&mut self, id: &str, solo: bool) -> Result<(), String> {
        if solo {
            if self.solo_stems.is_empty() {
                for (sid, track) in self.stems.iter() {
                    self.pre_solo_states.insert(sid.clone(), (track.base_volume, track.is_muted));
                }
            }
            self.solo_stems.insert(id.to_string());
            for (sid, track) in self.stems.iter_mut() {
                let vol = if self.solo_stems.contains(sid) {
                    track.base_volume
                } else {
                    0.0
                };
                track.sink.set_volume(vol);
            }
            if self.is_playing {
                for track in self.stems.values_mut() {
                    if self.solo_stems.contains(&track.id) {
                        track.sink.play();
                    } else {
                        track.sink.pause();
                    }
                }
            }
            Ok(())
        } else {
            self.solo_stems.remove(id);
            for (sid, track) in self.stems.iter_mut() {
                track.is_soloed = self.solo_stems.contains(sid);
            }
            if self.solo_stems.is_empty() {
                for (sid, track) in self.stems.iter_mut() {
                    if let Some(&(saved_vol, saved_muted)) = self.pre_solo_states.get(sid) {
                        track.base_volume = saved_vol;
                        track.is_muted = saved_muted;
                        track.sink.set_volume(if saved_muted { 0.0 } else { saved_vol });
                    }
                }
                if self.is_playing {
                    for track in self.stems.values_mut() {
                        if !track.is_muted {
                            track.sink.play();
                        }
                    }
                }
            } else {
                for (sid, track) in self.stems.iter_mut() {
                    let vol = if self.solo_stems.contains(sid) {
                        track.base_volume
                    } else {
                        0.0
                    };
                    track.sink.set_volume(vol);
                }
                if self.is_playing {
                    for track in self.stems.values_mut() {
                        if self.solo_stems.contains(&track.id) && !track.is_muted {
                            track.sink.play();
                        } else {
                            track.sink.pause();
                        }
                    }
                }
            }
            Ok(())
        }
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
    id: &str,
    path: &str,
) -> Result<StemTrack, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    let source = Decoder::new(reader).map_err(|e| format!("Failed to decode audio: {}", e))?;

    let sink = Sink::try_new(stream_handle)
        .map_err(|e| format!("Failed to create sink: {}", e))?;
    sink.append(source);

    Ok(StemTrack {
        sink,
        id: id.to_string(),
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
                    AudioCommand::LoadStems {
                        stem_paths,
                        response,
                    } => {
                        eprintln!("[Audio Thread] LoadStems → {} stems", stem_paths.len());
                        let result = manager.load_stems(stem_paths);
                        match &result {
                            Ok(ids) => eprintln!("[Audio Thread] LoadStems → OK ({} stems)", ids.len()),
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
                    AudioCommand::SetSolo {
                        id,
                        solo,
                        response,
                    } => {
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
