//! Offline beat detection: spectral flux -> autocorrelation tempo -> phase alignment.
//! The result is a uniform beat grid, cross-checked against lyric anchor times.

use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub struct BeatGrid {
    /// time of beat 0
    pub t0: f64,
    /// seconds per beat
    pub period: f64,
    pub bpm: f64,
}

impl BeatGrid {
    pub fn beat_index(&self, t: f64) -> i64 {
        ((t - self.t0) / self.period).floor() as i64
    }
    pub fn beat_phase(&self, t: f64) -> f64 {
        let k = (t - self.t0) / self.period;
        k - k.floor()
    }
    #[allow(dead_code)]
    pub fn bar_phase(&self, t: f64) -> f64 {
        let k = (t - self.t0) / (self.period * 4.0);
        k - k.floor()
    }
    pub fn downbeat(&self, t: f64) -> bool {
        self.beat_index(t).rem_euclid(4) == 0
    }
    /// percussive envelope 0..1 (fast attack, decay within the beat)
    pub fn pulse(&self, t: f64) -> f64 {
        let p = self.beat_phase(t);
        (1.0 - p).powf(2.4)
    }
    pub fn beat_time(&self, k: i64) -> f64 {
        self.t0 + k as f64 * self.period
    }
    #[allow(dead_code)]
    /// time of next beat at or after `t`
    pub fn next_beat(&self, t: f64) -> f64 {
        self.beat_time(self.beat_index(t) + if self.beat_phase(t) < 1e-6 { 0 } else { 1 })
    }
}

const FRAME: usize = 2048;
const HOP: usize = 512;

fn onset_envelope(mono: &[f32], rate: u32) -> (Vec<f32>, f64) {
    let n_bins = FRAME / 2;
    let env_rate = rate as f64 / HOP as f64;
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FRAME);

    // hann window
    let win: Vec<f32> = (0..FRAME)
        .map(|i| {
            0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / FRAME as f64).cos() as f32
        })
        .collect();

    let max_bin = ((8000.0 / (rate as f64 / 2.0)) * n_bins as f64) as usize;
    let max_bin = max_bin.clamp(16, n_bins);

    let mut prev = vec![0f32; max_bin];
    let mut flux: Vec<f32> = Vec::with_capacity(mono.len() / HOP + 2);
    let mut buf = vec![Complex::new(0.0, 0.0); FRAME];

    let mut pos = 0usize;
    while pos + FRAME <= mono.len() {
        for i in 0..FRAME {
            buf[i] = Complex::new(mono[pos + i] * win[i], 0.0);
        }
        fft.process(&mut buf);
        let mut f = 0f32;
        for b in 1..max_bin {
            let m = (buf[b].re * buf[b].re + buf[b].im * buf[b].im).sqrt().ln_1p();
            let d = m - prev[b];
            if d > 0.0 {
                f += d;
            }
            prev[b] = m;
        }
        flux.push(f);
        pos += HOP;
    }

    // normalize: subtract a moving average (~0.8 s), rectify
    let w = (env_rate * 0.8) as usize;
    let mut norm = vec![0f32; flux.len()];
    let mut acc = 0f64;
    for i in 0..flux.len() {
        acc += flux[i] as f64;
        if i >= w {
            acc -= flux[i - w] as f64;
        }
        let base = acc / w.min(i + 1) as f64;
        norm[i] = (flux[i] as f64 - base).max(0.0) as f32;
    }
    let max = norm.iter().cloned().fold(0.0f32, f32::max).max(1e-6);
    for v in norm.iter_mut() {
        *v /= max;
    }
    (norm, env_rate)
}

fn autocorr_peak(env: &[f32], env_rate: f64) -> Vec<(f64, f64)> {
    // returns (bpm, strength) candidates in 55..190 BPM
    let min_lag = ((60.0 / 190.0) * env_rate) as usize;
    let max_lag = ((60.0 / 55.0) * env_rate) as usize;
    let n = env.len();
    let mut scores: Vec<(f64, f64)> = Vec::new();
    for lag in min_lag..=max_lag.min(n / 2) {
        let mut s = 0f64;
        let mut i = 0;
        while i + lag < n {
            s += (env[i] * env[i + lag]) as f64;
            i += 1;
        }
        scores.push((60.0 * env_rate / lag as f64, s / (n - lag) as f64));
    }
    // local maxima only
    let mut peaks: Vec<(f64, f64)> = Vec::new();
    for i in 1..scores.len() - 1 {
        if scores[i].1 > scores[i - 1].1 && scores[i].1 >= scores[i + 1].1 {
            peaks.push(scores[i]);
        }
    }
    peaks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    peaks.truncate(8);
    peaks
}

fn best_phase(env: &[f32], env_rate: f64, period: f64) -> f64 {
    // scan offset in [0, period), score = envelope energy at beat points
    let steps = 96;
    let mut best = (0.0, -1.0f64);
    for s in 0..steps {
        let off = period * s as f64 / steps as f64;
        let mut score = 0f64;
        let mut tt = off;
        while (tt * env_rate) < env.len() as f64 {
            let i = (tt * env_rate) as usize;
            if i < env.len() {
                score += env[i] as f64;
            }
            tt += period;
        }
        if score > best.1 {
            best = (off, score);
        }
    }
    best.0
}

fn anchor_penalty(g: BeatGrid, anchors: &[f64]) -> f64 {
    if anchors.is_empty() {
        return 0.0;
    }
    let mut dists: Vec<f64> = anchors
        .iter()
        .map(|&a| {
            let k = (a - g.t0) / g.period;
            let near = g.beat_time(k.round() as i64);
            (a - near).abs()
        })
        .collect();
    dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = dists[dists.len() / 2];
    (mid * 1000.0).min(400.0) // ms, capped
}

pub fn analyze(mono: &[f32], rate: u32, _dur: f64, anchors: &[f64]) -> BeatGrid {
    let (env, env_rate) = onset_envelope(mono, rate);
    let peaks = autocorr_peak(&env, env_rate);

    // candidate tempi incl. half/double
    let mut cands: Vec<f64> = Vec::new();
    for &(bpm, _) in &peaks {
        for m in [1.0, 0.5, 2.0] {
            let b = bpm * m;
            if (55.0..=190.0).contains(&b) {
                cands.push(b);
            }
        }
    }
    cands.sort_by(|a, b| a.partial_cmp(b).unwrap());
    cands.dedup_by(|a, b| (*a - *b).abs() < 0.1);

    let mut best = BeatGrid { t0: 0.0, period: 60.0 / 120.0, bpm: 120.0 };
    let mut best_score = f64::INFINITY;
    for &bpm in &cands {
        let period = 60.0 / bpm;
        let off = best_phase(&env, env_rate, period);
        let mut g = BeatGrid { t0: off, period, bpm };
        // refine the grid against lyric anchors (they sit on bar lines)
        if anchors.len() >= 4 {
            for _ in 0..3 {
                let mut pts: Vec<(f64, f64)> = Vec::new(); // (k, t)
                for &a in anchors {
                    let k = ((a - g.t0) / g.period).round();
                    pts.push((k, a));
                }
                // drop the worst 25% residuals
                let mut res: Vec<f64> = pts.iter().map(|(k, t)| *t - (g.t0 + k * g.period)).collect();
                res.sort_by(|a, b| a.abs().partial_cmp(&b.abs()).unwrap());
                let cut = res[res.len() * 3 / 4].abs();
                let pts: Vec<(f64, f64)> = pts
                    .into_iter()
                    .filter(|(k, t)| (*t - (g.t0 + k * g.period)).abs() <= cut + 1e-9)
                    .collect();
                if pts.len() < 4 {
                    break;
                }
                // least squares t = t0 + k*period
                let n = pts.len() as f64;
                let sk: f64 = pts.iter().map(|(k, _)| k).sum();
                let st: f64 = pts.iter().map(|(_, t)| t).sum();
                let skk: f64 = pts.iter().map(|(k, _)| k * k).sum();
                let skt: f64 = pts.iter().map(|(k, t)| k * t).sum();
                let denom = n * skk - sk * sk;
                if denom.abs() < 1e-6 {
                    break;
                }
                let new_period = (n * skt - sk * st) / denom;
                let new_t0 = (st - new_period * sk) / n;
                if !(0.2..1.2).contains(&new_period) {
                    break;
                }
                g.period = new_period;
                g.bpm = 60.0 / new_period;
                g.t0 = new_t0 % new_period;
                if g.t0 < 0.0 {
                    g.t0 += new_period;
                }
            }
        }
        let pen = anchor_penalty(g, anchors);
        let strength = peaks
            .iter()
            .map(|&(b, s)| {
                let r = (b / bpm).round();
                if (b / bpm / r - 1.0).abs() < 0.02 { s } else { 0.0 }
            })
            .fold(0.0f64, f64::max);
        // listeners settle into the slower pulse when both fit; nudge away from
        // hyper-grid tempi
        let tempo_pen = if bpm > 150.0 { 45.0 } else { 0.0 };
        let score = pen + tempo_pen - strength * 40.0;
        if score < best_score {
            best_score = score;
            best = g;
        }
    }
    let mut t0 = best.t0;
    let mut period = best.period;
    // this song breathes slowly: settle the pulse into ~60..100 BPM
    while period < 60.0 / 100.0 {
        period *= 2.0;
    }
    while period > 60.0 / 60.0 {
        period /= 2.0;
    }
    while t0 >= period {
        t0 -= period;
    }
    while t0 < 0.0 {
        t0 += period;
    }
    BeatGrid { t0, period, bpm: 60.0 / period }
}

// ------------------------------------------------------------------ cache ---

fn cache_path(audio: &Path) -> PathBuf {
    audio.with_extension("beatcache")
}

pub fn analyze_cached(
    audio: &Path,
    mono: &[f32],
    rate: u32,
    dur: f64,
    anchors: &[f64],
) -> std::io::Result<BeatGrid> {
    let cp = cache_path(audio);
    let sig = format!(
        "wsb2|{}|{}|{:.3}|{}",
        rate,
        mono.len(),
        dur,
        anchors.len()
    );
    if let Ok(s) = std::fs::read_to_string(&cp) {
        if let Some(rest) = s.strip_prefix(&sig) {
            if let Some((bpm, t0)) = rest.strip_prefix('|').and_then(|r| {
                let mut it = r.split('|');
                Some((
                    it.next()?.parse::<f64>().ok()?,
                    it.next()?.parse::<f64>().ok()?,
                ))
            }) {
                let period = 60.0 / bpm;
                if period > 0.2 && period < 1.2 {
                    return Ok(BeatGrid { t0, period, bpm });
                }
            }
        }
    }
    let g = analyze(mono, rate, dur, anchors);
    std::fs::write(&cp, format!("{}|{}|{}", sig, g.bpm, g.t0))?;
    Ok(g)
}
