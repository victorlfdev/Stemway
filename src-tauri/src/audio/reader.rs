use std::path::PathBuf;
use std::fs::File;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Debug)]
pub enum AudioError {
    Io(std::io::Error),
    Symphonia(Error),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::Io(e) => write!(f, "IO error: {}", e),
            AudioError::Symphonia(e) => write!(f, "Symphonia error: {}", e),
        }
    }
}

impl std::error::Error for AudioError {}

pub struct AudioReader {
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_secs: f64,
    pub buffer_f32: Vec<f32>,
}

impl AudioReader {
    pub fn new(path: &PathBuf) -> Result<Self, AudioError> {
        let file = Box::new(File::open(path).map_err(|e| AudioError::Io(e))?);
        let mss = MediaSourceStream::new(file, Default::default());
        
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();
        let decoder_opts = DecoderOptions::default();
        
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| AudioError::Symphonia(e))?;
        
        let mut format = probed.format;
        let track = format
            .default_track()
            .ok_or_else(|| AudioError::Symphonia(Error::Unsupported("no audio track")))?;
        
        let codec_params = &track.codec_params;
        let sample_rate = codec_params.sample_rate.ok_or_else(||
            AudioError::Symphonia(Error::Unsupported("no sample rate"))
        )?;
        let n_channels = codec_params.channels.map(|c| c.count()).unwrap_or(1);
        
        let mut decoder = symphonia::default::get_codecs()
            .make(&codec_params, &decoder_opts)
            .map_err(|e| AudioError::Symphonia(e))?;
        
        let track_id = track.id;
        let mut sample_count = 0;
        let mut sample_buf = None;
        
        loop {
            let packet = match format.next_packet() {
                Ok(p) => p,
                Err(_) => break,
            };
            
            if packet.track_id() != track_id {
                continue;
            }
            
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    if sample_buf.is_none() {
                        let spec = *audio_buf.spec();
                        let duration = audio_buf.capacity() as u64;
                        sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                    }
                    
                    if let Some(buf) = &mut sample_buf {
                        buf.copy_interleaved_ref(audio_buf);
                        sample_count += buf.samples().len();
                    }
                }
                Err(Error::DecodeError(_)) => continue,
                Err(_) => break,
            }
        }
        
        let samples: Vec<f32> = sample_buf.map(|b| b.samples().to_vec()).unwrap_or_default();
        let duration_secs = sample_count as f64 / sample_rate as f64;
        
        let mut buffer_f32 = Vec::new();
        if n_channels == 1 {
            for &sample in &samples {
                buffer_f32.push(sample);
                buffer_f32.push(0.0);
            }
        } else {
            let step = if n_channels > 2 { 2 } else { n_channels };
            for i in (0..samples.len()).step_by(step) {
                buffer_f32.push(samples[i]);
                if i + 1 < samples.len() {
                    buffer_f32.push(samples[i + 1]);
                } else {
                    buffer_f32.push(0.0);
                }
            }
        }
        
        Ok(Self {
            sample_rate,
            channels: 2,
            duration_secs,
            buffer_f32,
        })
    }
    
    pub fn get_samples(&self) -> &Vec<f32> {
        &self.buffer_f32
    }
}
