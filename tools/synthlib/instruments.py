"""synthlib.instruments -- a library of fully synthesized instruments.

Every function returns a (L, R) float ndarray.  No samples anywhere:
kicks are pitch-swept sines, snares are shaped noise + tone, pads are
detuned supersaws, bells are 2-operator FM, basses are filtered
detuned-saw stacks.
"""
import numpy as np

from . import dsp
from .dsp import (SR, saw, square, sine, noise, t_arr, exp_env, adsr,
                  lp, hp, bp, chunk_lp, pan2, soft_clip)

# ------------------------------------------------------------------ drums --
def kick(vel=1.0, tune=48.0, sr=SR):
    n = int(0.38 * sr)
    t = t_arr(n, sr)
    f = tune + 130.0 * np.exp(-t / 0.030)          # pitch drop
    body = sine(f, n, sr=sr) * np.exp(-t / 0.13)
    punch = hp(noise(n, 1) * np.exp(-t / 0.006), 3200.0, sr=sr) * 0.6
    knock = bp(noise(n, 2), 1100.0, 1.2, sr=sr) * np.exp(-t / 0.012) * 0.35
    x = soft_clip(body * 1.5 + punch + knock, 1.8)
    x *= exp_env(n, 0.36, a=0.0008, sr=sr)
    l, r = pan2(x)
    return l * vel, r * vel

def sub(f, dur, vel=1.0, sr=SR):
    n = int(dur * sr) + int(0.25 * sr)
    x = sine(f, n, sr=sr) + 0.12 * sine(f * 2, n, sr=sr)
    x = soft_clip(x * 1.2, 1.3) * exp_env(n, dur * 0.85, a=0.004, sr=sr)
    l, r = pan2(x)
    return l * vel, r * vel

def snare(vel=1.0, sr=SR):
    n = int(0.30 * sr)
    t = t_arr(n, sr)
    nz = noise(n, 11)
    crack = bp(nz, 1900.0, 0.8, sr=sr) * np.exp(-t / 0.075)
    body_n = lp(nz, 8000.0, sr=sr) * np.exp(-t / 0.028) * 0.7
    f = 200.0 * np.exp(-t / 0.030) + 165.0
    tone = sine(f, n, sr=sr) * np.exp(-t / 0.045) * 0.9
    x = soft_clip(crack * 1.6 + body_n + tone, 1.9)
    x *= exp_env(n, 0.28, a=0.0006, sr=sr)
    l, r = pan2(x)
    return l * vel, r * vel

def clap(vel=1.0, sr=SR):
    n = int(0.42 * sr)
    t = t_arr(n, sr)
    x = np.zeros(n)
    for k, dt in enumerate((0.0, 0.011, 0.023, 0.036)):
        i0 = int(dt * sr)
        m = n - i0
        tt = t_arr(m, sr)
        x[i0:] += bp(noise(m, 21 + k), 1150.0, 1.0, sr=sr) * np.exp(-tt / 0.012) * 0.8
    x += bp(noise(n, 25), 1300.0, 0.9, sr=sr) * np.exp(-t / 0.11) * 0.55
    x = soft_clip(x * 1.4, 1.6) * exp_env(n, 0.4, a=0.0006, sr=sr)
    l, r = pan2(x)
    return l * vel, r * vel

def hatc(vel=1.0, sr=SR):
    n = int(0.07 * sr)
    t = t_arr(n, sr)
    x = hp(noise(n, 31), 8200.0, sr=sr) * np.exp(-t / 0.014)
    x += bp(noise(n, 32), 10400.0, 2.0, sr=sr) * np.exp(-t / 0.008) * 0.5
    x *= exp_env(n, 0.05, a=0.0002, sr=sr)
    l, r = pan2(x, 0.12)
    return l * vel, r * vel

def hato(vel=1.0, sr=SR):
    n = int(0.45 * sr)
    t = t_arr(n, sr)
    x = hp(noise(n, 41), 7200.0, sr=sr) * np.exp(-t / 0.13)
    x *= exp_env(n, 0.4, a=0.0004, sr=sr)
    l, r = pan2(x, 0.15)
    return l * vel, r * vel

def ride(vel=1.0, sr=SR):
    n = int(0.55 * sr)
    t = t_arr(n, sr)
    x = bp(noise(n, 51), 6200.0, 3.0, sr=sr) * np.exp(-t / 0.22)
    x += bp(noise(n, 52), 9100.0, 4.0, sr=sr) * np.exp(-t / 0.10) * 0.6
    x *= exp_env(n, 0.5, a=0.0005, sr=sr)
    l, r = pan2(x, -0.2)
    return l * 0.7 * vel, r * 0.7 * vel

# ------------------------------------------------------------------ bass --
def reese(f, dur, vel=1.0, wob=2.9, seed=0, sr=SR):
    """Growling detuned-saw bass with an LFO-swept low-pass."""
    n = int(dur * sr) + int(0.12 * sr)
    t = t_arr(n, sr)
    x = np.zeros(n)
    for k, det in enumerate((0.986, 1.0, 1.014)):
        x += saw(f * det, n, phase=k * 0.33 + seed * 0.17, sr=sr)
    lfo = 0.5 + 0.5 * np.sin(2 * np.pi * wob * t + seed * 1.7)
    x = chunk_lp(x, 180.0 + lfo * 950.0, sr=sr)
    x = soft_clip(x * 1.9, 2.0)
    x = hp(x, 55.0, sr=sr)
    env = adsr(n, a=0.004, d=dur * 0.9, s=0.72, r=0.09, sr=sr)
    l, r = pan2(x * env)
    return l * vel, r * vel

# ------------------------------------------------------------------ pads --
def pad(freqs, dur, vel=1.0, cutoff=1500.0, attack=0.35, seed=0, sr=SR):
    """Wide supersaw chord pad."""
    n = int(dur * sr) + int(0.5 * sr)
    t = t_arr(n, sr)
    L = np.zeros(n); R = np.zeros(n)
    pans = (-0.85, -0.45, 0.0, 0.45, 0.85)
    det_c = (-14.0, -7.0, 0.0, 8.0, 15.0)
    for fn in freqs:
        for k in range(5):
            det = (det_c[k] + 2.5 * np.sin(2 * np.pi * (0.21 + 0.05 * k) * t + seed + k)) / 1200.0
            fa = fn * (1.0 + det)
            v = saw(fa, n, phase=0.13 * k + seed + fn, sr=sr) * 0.16
            l, r = pan2(v, pans[k])
            L += l; R += r
    x_l = lp(L, cutoff, sr=sr); x_r = lp(R, cutoff, sr=sr)
    env = adsr(n, a=attack, d=dur, s=0.85, r=0.42, sr=sr)
    return x_l * env * vel, x_r * env * vel

# ------------------------------------------------------------ keys/lead --
def pluck(f, vel=1.0, cutoff=4500.0, dur=0.32, sr=SR):
    """Short filtered saw for arps."""
    n = int(dur * sr) + int(0.10 * sr)
    x = saw(f, n, phase=f % 1.0, sr=sr)
    t = t_arr(n, sr)
    co = 480.0 + (cutoff - 480.0) * np.exp(-t / 0.055)
    x = chunk_lp(x, co, sr=sr)
    x = hp(x, 240.0, sr=sr)
    x *= exp_env(n, 0.105, a=0.001, sr=sr)
    l, r = pan2(x)
    return l * vel, r * vel

def lead(f, dur, vel=1.0, bright=1.0, octlayer=False, seed=0, sr=SR):
    """Warm supersaw lead: 5 detuned saws, gentle drive, soft vibrato."""
    n = int(dur * sr) + int(0.18 * sr)
    t = t_arr(n, sr)
    vib = 0.0035 * np.sin(2 * np.pi * 5.2 * t) * np.clip((t - 0.12) / 0.2, 0, 1)
    L = np.zeros(n); R = np.zeros(n)
    det_c = (-12.0, -5.0, 0.0, 6.0, 13.0)
    pans = (-0.7, -0.3, 0.0, 0.3, 0.7)
    for k in range(5):
        fa = f * (1.0 + det_c[k] / 1200.0 + vib)
        v = saw(fa, n, phase=0.17 * k + seed, sr=sr) * 0.2
        l, r = pan2(v, pans[k])
        L += l; R += r
    if octlayer:
        v = saw(f * 0.5, n, phase=0.4, sr=sr) * 0.26
        L += v * 0.7; R += v * 0.7
    x_l = lp(L, 3800.0 * bright, sr=sr)
    x_r = lp(R, 3800.0 * bright, sr=sr)
    x_l = soft_clip(x_l * 1.35, 1.3); x_r = soft_clip(x_r * 1.35, 1.3)
    env = adsr(n, a=0.006, d=dur * 0.75, s=0.8, r=0.12, sr=sr)
    return x_l * env * vel, x_r * env * vel

def bass_pluck(f, dur, vel=1.0, cutoff=950.0, sr=SR):
    """Clean punchy bass: saw+square through a closing low-pass."""
    n = int(dur * sr) + int(0.08 * sr)
    t = t_arr(n, sr)
    x = saw(f, n, sr=sr) * 0.7 + square(f, n, 0.5, sr=sr) * 0.3
    co = 150.0 + (cutoff - 150.0) * np.exp(-t / max(dur * 0.45, 1e-3))
    x = chunk_lp(x, co, sr=sr)
    x = hp(x, 45.0, sr=sr)
    x = soft_clip(x * 1.4, 1.4)
    x *= adsr(n, a=0.003, d=dur * 0.7, s=0.55, r=0.05, sr=sr)
    l, r = pan2(x)
    return l * vel, r * vel

def bell(f, vel=1.0, dur=1.8, sr=SR):
    """2-op FM glass bell (slightly detuned across the stereo field)."""
    n = int(dur * sr)
    t = t_arr(n, sr)
    idx = 3.4 * np.exp(-t / 0.22)
    l = n // 2
    fm = np.cumsum(np.full(n, f * 2.76)) / sr
    out = np.empty(n)
    out[:l] = np.sin(2 * np.pi * np.cumsum(np.full(l, f)) / sr +
                     idx[:l] * np.sin(2 * np.pi * fm[:l]))
    out[l:] = np.sin(2 * np.pi * np.cumsum(np.full(n - l, f * 1.0015)) / sr +
                     idx[l:] * np.sin(2 * np.pi * fm[l:]))
    out += 0.25 * np.sin(2 * np.pi * np.cumsum(np.full(n, f * 2.01)) / sr) * np.exp(-t / 0.45)
    out *= np.exp(-t / 0.85) * exp_env(n, dur, a=0.001, sr=sr)
    return out * 0.8 * vel, out * 0.8 * vel

# -------------------------------------------------------------------- fx --
def riser(dur, vel=1.0, seed=0, sr=SR):
    """Noise sweep + rising saw cluster with speeding tremolo."""
    n = int(dur * sr)
    t = t_arr(n, sr)
    prog = t / dur
    nz = noise(n, 61 + seed)
    co = 350.0 * (16.0 ** prog)
    x = chunk_lp(nz, co, sr=sr) * (prog ** 1.6) * 0.8
    cluster = np.zeros(n)
    for k in range(5):
        fr = 220.0 * (2.0 ** (prog * 2.0)) * (1.0 + 0.01 * k)
        cluster += saw(fr, n, phase=0.15 * k, sr=sr)
    trem_rate = 6.0 + 26.0 * prog
    trem = 0.55 + 0.45 * np.sin(2 * np.pi * np.cumsum(np.full(n, trem_rate)) / sr)
    x += cluster * trem * (prog ** 2.2) * 0.22
    x = soft_clip(x * 1.3, 1.5)
    x *= exp_env(n, dur * 1.4, a=0.05, sr=sr)
    l, r = pan2(x)
    return l * vel, r * vel

def impact(vel=1.0, sr=SR):
    """Sub drop + noise splash for section slams."""
    n = int(1.6 * sr)
    t = t_arr(n, sr)
    f = 30.0 + 52.0 * np.exp(-t / 0.20)
    x = sine(f, n, sr=sr) * np.exp(-t / 0.55)
    splash = lp(noise(n, 71), 8500.0, sr=sr) * np.exp(-t / 0.22) * 0.5
    l, r = pan2(x, 0.0)
    sl, srn = pan2(splash, 0.0)
    L, R = soft_clip(l + sl, 1.4), soft_clip(r + srn, 1.4)
    return L * vel, R * vel

def revcym(dur=1.1, vel=1.0, sr=SR):
    """Reverse cymbal swoosh (riser into a hit)."""
    n = int(dur * sr)
    t = t_arr(n, sr)
    x = bp(noise(n, 81), 6200.0, 0.8, sr=sr) + 0.5 * hp(noise(n, 82), 2500.0, sr=sr)
    x *= (t / dur) ** 2.4
    x *= exp_env(n, dur * 3.0, a=0.02, sr=sr)
    l, r = pan2(x)
    return l * vel, r * vel

def downlift(dur=0.9, vel=1.0, sr=SR):
    n = int(dur * sr)
    t = t_arr(n, sr)
    fr = 1800.0 * np.exp(-t / 0.28) + 90.0
    x = saw(fr, n, sr=sr) * 0.5 + \
        chunk_lp(noise(n, 91), 300 + 2600 * np.exp(-t / 0.3), sr=sr) * 0.5
    x *= exp_env(n, dur * 0.9, a=0.004, sr=sr)
    l, r = pan2(x)
    return l * vel, r * vel

def blip(f=1200.0, dur=0.05, vel=1.0, pan=0.0, seed=0, sr=SR):
    """Bit-crushed square blip (boot sounds / glitches)."""
    n = max(int(dur * sr), 64)
    x = square(f, n, 0.5, seed * 0.37, sr=sr)
    x = np.round(x * 3.0) / 3.0                       # crush
    x = hp(x, 280.0, sr=sr) * exp_env(n, dur * 0.7, a=0.001, sr=sr)
    l, r = pan2(x, pan)
    return l * 0.8 * vel, r * 0.8 * vel

def zap(f1=2200.0, f2=180.0, dur=0.5, vel=1.0, sr=SR):
    """Falling chirp -- 'system halt' sound."""
    n = int(dur * sr)
    t = t_arr(n, sr)
    fr = f2 + (f1 - f2) * np.exp(-t / (dur * 0.24))
    x = square(fr, n, 0.5, sr=sr) * 0.6 + saw(fr * 0.501, n, sr=sr) * 0.4
    x = lp(x, 3600.0, sr=sr) * exp_env(n, dur * 0.9, a=0.002, sr=sr)
    l, r = pan2(x)
    return l * vel, r * vel

INSTRUMENTS = {
    'kick': kick, 'sub': sub, 'snare': snare, 'clap': clap,
    'hatc': hatc, 'hato': hato, 'ride': ride,
    'reese': reese, 'bass_pluck': bass_pluck, 'pad': pad, 'pluck': pluck,
    'lead': lead, 'bell': bell,
    'riser': riser, 'impact': impact, 'revcym': revcym, 'downlift': downlift,
    'blip': blip, 'zap': zap,
}

# instrument -> suggested mix bus
BUS_OF = {
    'kick': 'DRUMS', 'snare': 'DRUMS', 'clap': 'DRUMS',
    'hatc': 'DRUMS', 'hato': 'DRUMS', 'ride': 'DRUMS',
    'reese': 'BASS', 'bass_pluck': 'BASS', 'sub': 'SUB',
    'pad': 'PADS', 'pluck': 'ARP', 'lead': 'LEAD', 'bell': 'LEAD',
    'riser': 'FX', 'impact': 'FX', 'revcym': 'FX', 'downlift': 'FX',
    'blip': 'FX', 'zap': 'FX',
}
