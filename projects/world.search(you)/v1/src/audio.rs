//! MP3 decoding (symphonia) + playback (cpal) + the master clock.

use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------- decode ---

pub struct Decoded {
    pub samples: Arc<Vec<i16>>, // interleaved, stereo (mono duplicated)
    pub rate: u32,
    pub frames: usize,
    pub duration: f64,
    pub channels: usize,
}

pub fn decode(path: &Path, progress: impl Fn(f64)) -> Result<Decoded> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use symphonia::default::{get_codecs, get_probe};

    let src = std::fs::File::open(path)
        .with_context(|| format!("cannot open audio file {}", path.display()))?;
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let probed = get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| anyhow!("probe failed: {e}"))?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| anyhow!("no audio track"))?
        .clone();
    let track_id = track.id;
    let mut decoder = get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| anyhow!("decoder init failed: {e}"))?;

    let file_len = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let mut out: Vec<i16> = Vec::new();
    let mut sbuf: Option<SampleBuffer<i16>> = None;
    let mut cur_spec: Option<symphonia::core::audio::SignalSpec> = None;
    let mut frames = 0usize;
    let mut channels = 2usize;
    let mut rate = 48000u32;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => return Err(anyhow!("decode error: {e}")),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(anyhow!("decode error: {e}")),
        };
        let spec = *decoded.spec();
        rate = spec.rate;
        channels = spec.channels.count();
        let cap = decoded.capacity() as u64;
        let need_new = match cur_spec {
            Some(cs) => cs != spec,
            None => true,
        };
        if need_new {
            sbuf = Some(SampleBuffer::<i16>::new(cap, spec));
            cur_spec = Some(spec);
        }
        let sb = sbuf.as_mut().unwrap();
        sb.copy_interleaved_ref(decoded);
        frames += sb.len() / channels;
        out.extend_from_slice(sb.samples());
        if file_len > 0 {
            progress((frames as f64 * rate as f64 * channels as f64 * 2.0 / file_len as f64).min(1.0));
        }
    }

    // normalize channel layout to stereo
    let samples = if channels == 1 {
        let mut st = Vec::with_capacity(out.len() * 2);
        for s in &out {
            st.push(*s);
            st.push(*s);
        }
        channels = 2;
        st
    } else if channels > 2 {
        let mut st = Vec::with_capacity(frames * 2);
        for fr in out.chunks(channels) {
            st.push(fr[0]);
            st.push(fr[1]);
        }
        channels = 2;
        st
    } else {
        out
    };

    let duration = frames as f64 / rate as f64;
    progress(1.0);
    Ok(Decoded {
        samples: Arc::new(samples),
        rate,
        frames,
        duration,
        channels,
    })
}

// --------------------------------------------------------------- playback ---

struct Ctrl {
    pos_f: f64,      // file-frame cursor
    vol: f32,
    playing: bool,
    finished: bool,
}

pub struct AudioOut {
    ctrl: Arc<Mutex<Ctrl>>,
    pub rate: u32,
    pub frames: usize,
    pub duration: f64,
    _stream: Option<cpal::Stream>,
}

impl AudioOut {
    pub fn start(dec: &Decoded) -> Result<AudioOut> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("no audio output device"))?;
        let cfg: cpal::StreamConfig = device.default_output_config()?.into();
        let dev_rate = cfg.sample_rate.0 as f64;
        let out_ch = cfg.channels as usize;
        let step = dec.rate as f64 / dev_rate; // file frames per device frame
        let samples = dec.samples.clone();
        let n_frames = dec.frames;
        let file_ch = dec.channels;
        let ctrl = Arc::new(Mutex::new(Ctrl {
            pos_f: 0.0,
            vol: 0.85,
            playing: true,
            finished: false,
        }));
        let ctrl_cb = ctrl.clone();

        let stream = device
            .build_output_stream(
                &cfg,
                move |data: &mut [f32], _| {
                    let mut c = ctrl_cb.lock().unwrap();
                    if !c.playing {
                        for v in data.iter_mut() {
                            *v = 0.0;
                        }
                        return;
                    }
                    let vol = c.vol;
                    let mut pos = c.pos_f;
                    let total = data.len() / out_ch.max(1);
                    for f in 0..total {
                        let i = pos as usize;
                        if i + 1 >= n_frames {
                            for v in data[f * out_ch..(f + 1) * out_ch].iter_mut() {
                                *v = 0.0;
                            }
                            c.finished = true;
                            c.playing = false;
                            break;
                        }
                        let fr = (pos - i as f64) as f32;
                        for ch in 0..out_ch {
                            let sc = ch.min(file_ch - 1);
                            let a = samples[i * file_ch + sc] as f32 / 32768.0;
                            let b = samples[(i + 1) * file_ch + sc] as f32 / 32768.0;
                            data[f * out_ch + ch] = (a + (b - a) * fr) * vol;
                        }
                        pos += step;
                    }
                    c.pos_f = pos;
                },
                move |err| {
                    let _ = err;
                },
                None,
            )
            .map_err(|e| anyhow!("audio stream failed: {e}"))?;
        stream.play().map_err(|e| anyhow!("audio play failed: {e}"))?;

        Ok(AudioOut {
            ctrl,
            rate: dec.rate,
            frames: dec.frames,
            duration: dec.duration,
            _stream: Some(stream),
        })
    }

    pub fn time(&self) -> f64 {
        let c = self.ctrl.lock().unwrap();
        (c.pos_f / self.rate as f64).clamp(0.0, self.duration)
    }
    pub fn set_paused(&self, p: bool) {
        self.ctrl.lock().unwrap().playing = !p && !self.ctrl.lock().unwrap().finished;
    }
    pub fn is_paused(&self) -> bool {
        !self.ctrl.lock().unwrap().playing
    }
    pub fn seek(&self, t: f64) {
        let mut c = self.ctrl.lock().unwrap();
        c.pos_f = (t * self.rate as f64).clamp(0.0, (self.frames as f64) - 2.0);
        c.finished = false;
        c.playing = true;
    }
    pub fn set_volume(&self, v: f32) {
        self.ctrl.lock().unwrap().vol = v.clamp(0.0, 1.0);
    }
    #[allow(dead_code)]
    pub fn volume(&self) -> f32 {
        self.ctrl.lock().unwrap().vol
    }
    pub fn finished(&self) -> bool {
        self.ctrl.lock().unwrap().finished
    }
}

// ------------------------------------------------------------------ clock ---

/// Master clock: audio position, or wall time when running without audio.
pub enum Clock {
    Audio(Box<AudioOut>),
    Wall {
        start: std::time::Instant,
        offset: f64,
        paused_at: Option<std::time::Instant>,
    },
}

impl Clock {
    pub fn wall() -> Clock {
        Clock::Wall {
            start: std::time::Instant::now(),
            offset: 0.0,
            paused_at: None,
        }
    }
    pub fn time(&self) -> f64 {
        match self {
            Clock::Audio(a) => a.time(),
            Clock::Wall { start, offset, paused_at } => match paused_at {
                Some(p) => *offset + (p.duration_since(*start).as_secs_f64()),
                None => *offset + start.elapsed().as_secs_f64(),
            },
        }
    }
    pub fn set_paused(&mut self, p: bool) {
        match self {
            Clock::Audio(a) => a.set_paused(p),
            Clock::Wall { start, offset, paused_at } => {
                if p {
                    if paused_at.is_none() {
                        *paused_at = Some(std::time::Instant::now());
                    }
                } else if let Some(pt) = paused_at.take() {
                    // resume: fold pause duration into offset
                    *offset += pt.duration_since(*start).as_secs_f64();
                    *start = std::time::Instant::now();
                }
            }
        }
    }
    pub fn is_paused(&self) -> bool {
        match self {
            Clock::Audio(a) => a.is_paused(),
            Clock::Wall { paused_at, .. } => paused_at.is_some(),
        }
    }
    pub fn seek(&mut self, t: f64, dur: f64) {
        let t = t.clamp(0.0, dur);
        match self {
            Clock::Audio(a) => a.seek(t),
            Clock::Wall { start, offset, .. } => {
                *offset = t;
                *start = std::time::Instant::now();
            }
        }
    }
}

