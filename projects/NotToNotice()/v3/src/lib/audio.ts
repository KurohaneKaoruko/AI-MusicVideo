// ---------------------------------------------------------------------------
// audio.ts - precomputed audio features (from the shared analysis pipeline).
// every lookup is a pure function of the frame index -> deterministic renders.
// ---------------------------------------------------------------------------

import analysisJson from "../analysis.json";
import { clamp } from "./math";

const d = analysisJson as unknown as {
  fps: number;
  frames: number;
  duration: number;
  bpm: number;
  beat_t0: number;
  beat_period: number;
  beats: number[];
  hits: [number, number][];
  rms: number[];
  low: number[];
  mid: number[];
  high: number[];
  centroid: number[];
  onset: number[];
  kick: number[];
  hat: number[];
  pulse: number[];
};

export const AUDIO_FPS = d.fps;
export const AUDIO_FRAMES = d.frames;
export const BPM = d.bpm;
export const BEAT_T0 = d.beat_t0;
export const BEAT_PERIOD = d.beat_period;

const KEYS = ["rms", "low", "mid", "high", "centroid", "onset", "kick", "hat", "pulse"] as const;
export type FeatKey = (typeof KEYS)[number];

// one-pole IIR smoothed (causal) + fast-attack/slow-release follower
const smoothed: Record<string, Float32Array> = {};
const risen: Record<string, Float32Array> = {};
for (const k of KEYS) {
  const src = d[k] as number[];
  const sm = new Float32Array(src.length);
  let acc = 0;
  for (let i = 0; i < src.length; i++) {
    acc = acc * 0.55 + src[i] * 0.45;
    sm[i] = acc;
  }
  smoothed[k] = sm;
  const ri = new Float32Array(src.length);
  let acc2 = 0;
  for (let i = 0; i < src.length; i++) {
    acc2 = src[i] > acc2 ? acc2 + (src[i] - acc2) * 0.9 : acc2 + (src[i] - acc2) * 0.12;
    ri[i] = acc2;
  }
  risen[k] = ri;
}

// accent hits bucketed by frame
const hitsByFrame = new Map<number, number>();
for (const [t, s] of d.hits) {
  const f = Math.round(t * AUDIO_FPS);
  hitsByFrame.set(f, Math.max(hitsByFrame.get(f) ?? 0, s));
}
export const HITS: [number, number][] = d.hits;
export const BEATS: number[] = d.beats;

const ip = (arr: Float32Array | number[], frame: number): number => {
  const i = Math.max(0, Math.min(arr.length - 1, frame | 0));
  const f = frame - (frame | 0);
  const a = arr[i];
  const b = arr[Math.min(arr.length - 1, i + 1)];
  return a + (b - a) * f;
};

export type AudioFrame = {
  rms: number; low: number; mid: number; high: number; centroid: number;
  onset: number; kick: number; hat: number; pulse: number;
  rmsS: number; lowS: number; highS: number; onsetS: number; pulseS: number;
  rmsR: number; onsetR: number; kickR: number;
  hit: number;   // strong accent at this exact frame (0 if none)
  loud: number;  // ~1.2s window energy
  beat: number;  // 0..1 position within current beat
  beatPulse: number; // decaying envelope since last beat
};

/** windowed loudness */
const loudAt = (frame: number): number => {
  const i = Math.max(0, Math.min(d.frames - 1, frame | 0));
  const r = d.rms;
  let acc = 0;
  let n = 0;
  for (let k = i - 36; k <= i + 36; k += 3) {
    if (k < 0 || k >= r.length) continue;
    acc += r[k];
    n++;
  }
  return n ? acc / n : 0;
};

const hitAt = (frame: number): number => {
  let best = 0;
  for (let dd = -1; dd <= 1; dd++) {
    const v = hitsByFrame.get(frame + dd);
    if (v !== undefined) best = Math.max(best, v);
  }
  return best > 0.35 ? best : 0;
};

/** beat phase: 0 at beat, rises to 1 just before next */
const beatInfo = (t: number): { beat: number; beatPulse: number } => {
  const phase = ((t - BEAT_T0) / BEAT_PERIOD) % 1;
  const beat = phase < 0 ? phase + 1 : phase;
  return { beat, beatPulse: Math.exp(-beat * 5.5) };
};

export const audioAt = (frame: number): AudioFrame => {
  const t = frame / AUDIO_FPS;
  const { beat, beatPulse } = beatInfo(t);
  return {
    rms: ip(d.rms, frame), low: ip(d.low, frame), mid: ip(d.mid, frame),
    high: ip(d.high, frame), centroid: ip(d.centroid, frame),
    onset: ip(d.onset, frame), kick: ip(d.kick, frame), hat: ip(d.hat, frame),
    pulse: ip(d.pulse, frame),
    rmsS: ip(smoothed.rms, frame), lowS: ip(smoothed.low, frame),
    highS: ip(smoothed.high, frame), onsetS: ip(smoothed.onset, frame),
    pulseS: ip(smoothed.pulse, frame),
    rmsR: ip(risen.rms, frame), onsetR: ip(risen.onset, frame), kickR: ip(risen.kick, frame),
    hit: hitAt(frame), loud: loudAt(frame), beat, beatPulse,
  };
};

/** accent "shockwaves" still alive at this frame -> for shader rings */
export const recentRings = (frame: number, window = 1.1, max = 6): number[] => {
  const out: number[] = [];
  const fps = AUDIO_FPS;
  for (let k = 0; k < 90; k++) {
    const f = frame - k;
    const v = hitsByFrame.get(f);
    if (v !== undefined && v > 0.4) {
      const age = (frame - f) / fps;
      if (age <= window) out.push(age, v);
      if (out.length >= max * 2) break;
    }
  }
  while (out.length < max * 2) out.push(99, 0);
  return out.slice(0, max * 2);
};

export const norm = (v: number, lo: number, hi: number): number => clamp((v - lo) / (hi - lo || 1e-9));
