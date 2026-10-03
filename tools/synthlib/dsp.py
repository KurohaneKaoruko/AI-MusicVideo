"""synthlib.dsp -- low-level DSP toolbox (project-independent).

Oscillators, envelopes, RBJ biquad filters, time-varying filtering,
FFT convolution reverb, ping-pong delay, sidechain ducking, master
helpers.  Pure numpy/scipy.
"""
import numpy as np
from scipy.signal import lfilter, lfilter_zi

SR = 44100                     # library default sample rate

# ------------------------------------------------------------------ notes --
_SEMI = {'C': 0, 'D': 2, 'E': 4, 'F': 5, 'G': 7, 'A': 9, 'B': 11}

def note_freq(name):
    """'F5' / 'Db4' / 'C#3' -> frequency in Hz."""
    semi = _SEMI[name[0].upper()]
    i = 1
    if len(name) > 1 and name[1] in 'b#':
        semi += -1 if name[1] == 'b' else 1
        i = 2
    octv = int(name[i:])
    midi = (octv + 1) * 12 + semi
    return 440.0 * 2.0 ** ((midi - 69) / 12.0)

def midi_freq(m):
    return 440.0 * 2.0 ** ((m - 69) / 12.0)

# --------------------------------------------------------------- helpers --
def t_arr(n, sr=SR):
    return np.arange(n, dtype=np.float64) / sr

def osc_phase(f, n, phase=0.0, sr=SR):
    """Phase 0..1 for scalar or per-sample frequency array."""
    if np.isscalar(f):
        return (phase + f * np.arange(n, dtype=np.float64) / sr) % 1.0
    return (phase + np.cumsum(np.asarray(f, np.float64)) / sr) % 1.0

def saw(f, n, phase=0.0, sr=SR):
    return 2.0 * osc_phase(f, n, phase, sr) - 1.0

def square(f, n, duty=0.5, phase=0.0, sr=SR):
    return np.where(osc_phase(f, n, phase, sr) < duty, 1.0, -1.0)

def sine(f, n, phase=0.0, sr=SR):
    return np.sin(2.0 * np.pi * osc_phase(f, n, phase, sr))

def tri(f, n, phase=0.0, sr=SR):
    p = osc_phase(f, n, phase, sr)
    return 4.0 * np.abs(p - 0.5) - 1.0

def noise(n, seed=None):
    return np.random.default_rng(seed).standard_normal(n)

# -------------------------------------------------------------- envelopes --
def exp_env(n, tau, a=0.002, sout=0.010, sr=SR):
    """Exponential decay with a tiny attack and click-free tail fade."""
    e = np.exp(-t_arr(n, sr) / max(tau, 1e-4))
    na = max(int(a * sr), 1)
    e[:na] *= np.linspace(0.0, 1.0, na)
    ns = max(int(sout * sr), 1)
    if n > ns:
        e[-ns:] *= np.linspace(1.0, 0.0, ns)
    return e

def adsr(n, a=0.01, d=0.1, s=0.8, r=0.05, sr=SR):
    na, nd, nr = max(int(a * sr), 1), max(int(d * sr), 1), max(int(r * sr), 1)
    ns = max(n - na - nd - nr, 0)
    env = np.concatenate([
        np.linspace(0, 1, na),
        np.linspace(1, s, nd),
        np.full(ns, s),
        np.linspace(s, 0, nr),
    ])
    return env[:n] if len(env) >= n else np.pad(env, (0, n - len(env)))

def fade_edges(x, fin=0.003, fout=0.010, sr=SR):
    na = max(int(fin * sr), 1)
    x[:na] *= np.linspace(0, 1, na)
    nb = max(int(fout * sr), 1)
    if len(x) > nb:
        x[-nb:] *= np.linspace(1, 0, nb)
    return x

# ---------------------------------------------------------------- filters --
def _coef(kind, fc, q=0.7071, sr=SR):
    fc = min(max(float(fc), 18.0), sr * 0.475)
    w0 = 2.0 * np.pi * fc / sr
    cw, sw = np.cos(w0), np.sin(w0)
    al = sw / (2.0 * q)
    if kind == 'lp':
        b = np.array([(1 - cw) / 2, 1 - cw, (1 - cw) / 2])
    elif kind == 'hp':
        b = np.array([(1 + cw) / 2, -(1 + cw), (1 + cw) / 2])
    else:                                  # band-pass, unity peak
        b = np.array([al, 0.0, -al])
    a = np.array([1 + al, -2 * cw, 1 - al])
    return b / a[0], a / a[0]

def filt(x, kind, fc, q=0.7071, sr=SR):
    b, a = _coef(kind, fc, q, sr)
    return lfilter(b, a, x)

def lp(x, fc, q=0.7071, sr=SR): return filt(x, 'lp', fc, q, sr)
def hp(x, fc, q=0.7071, sr=SR): return filt(x, 'hp', fc, q, sr)
def bp(x, fc, q=0.7071, sr=SR): return filt(x, 'bp', fc, q, sr)

def chunk_lp(x, cutoffs, sr=SR):
    """Time-varying low-pass. `cutoffs` is a per-sample cutoff curve (or
    scalar).  Filter state carries across chunks -> no zipper clicks."""
    b, a = _coef('lp', 1000.0, sr=sr)
    zi = lfilter_zi(b, a)
    y = np.empty_like(x, dtype=np.float64)
    ch = int(0.020 * sr)
    n = len(x)
    if np.isscalar(cutoffs):
        cutoffs = np.full(n, float(cutoffs))
    for i in range(0, n, ch):
        j = min(i + ch, n)
        fc = float(cutoffs[min(i + ch // 2, n - 1)])
        bb, aa = _coef('lp', fc, sr=sr)
        seg, zi = lfilter(bb, aa, x[i:j], zi=zi * x[i])
        y[i:j] = seg
    return y

# ------------------------------------------------------------------ mix ----
def pan2(m, p=0.0):
    """Mono -> (L, R) constant-power pan, p in [-1, 1]."""
    th = (np.clip(p, -1, 1) + 1.0) * np.pi / 4.0
    return m * np.cos(th), m * np.sin(th)

def place(busL, busR, start, sig, gain=1.0, sr=SR):
    """Add a mono or stereo signal into a stereo bus at `start` seconds."""
    n = len(busL)
    i0 = int(round(start * sr))
    if i0 >= n:
        return
    if np.ndim(sig) == 1:
        l = r = np.asarray(sig, np.float64)
    else:
        l, r = sig
    if gain != 1.0:
        l, r = l * gain, r * gain
    j = min(n, i0 + len(l))
    busL[i0:j] += l[:j - i0]
    busR[i0:j] += r[:j - i0]

# -------------------------------------------------------------- effects ----
def fftconv(x, ir):
    """Full linear convolution via FFT (returns len(x)+len(ir)-1)."""
    n = len(x) + len(ir) - 1
    n2 = 1 << int(n - 1).bit_length()
    y = np.fft.irfft(np.fft.rfft(x, n2) * np.fft.rfft(ir, n2), n2)
    return y[:n]

def make_ir(dur, decay, damp=5200.0, predelay=0.0, seed=0, sr=SR):
    """Synthetic reverb impulse response: sparse early reflections +
    dense exponentially-decaying stereo noise, gently low-passed."""
    n = int(dur * sr)
    t = t_arr(n, sr)
    env = np.exp(-t * decay)
    irs = []
    for ch in range(2):
        rng = np.random.default_rng(seed + ch * 7)
        x = rng.standard_normal(n) * env
        ne = int(0.08 * sr)
        er = np.zeros(n)
        for k in range(10):
            i = int(rng.uniform(0.004, 0.075) * sr)
            er[i] += rng.uniform(-1, 1) * (1.0 - k / 12.0)
        x += er * 0.5
        if predelay > 0:
            pd = int(predelay * sr)
            x = np.concatenate([np.zeros(pd), x[:-pd]])
        x = lp(fade_edges(x, 0.001, 0.05, sr), damp, sr=sr)
        irs.append(x)
    m = max(np.max(np.abs(irs[0])), np.max(np.abs(irs[1])), 1e-9)
    return np.stack([irs[0] / m, irs[1] / m], axis=1)

def pingpong(mono, dt, fb=0.42, taps=6, damp=3800.0, sr=SR):
    """Ping-pong delay: echoes alternate L/R. Returns (L, R)."""
    n = len(mono)
    out = np.zeros((2, n))
    cur = mono.copy()
    d = max(int(dt * sr), 1)
    for i in range(1, taps + 1):
        cur = filt(cur, 'lp', damp, sr=sr) * fb
        i0 = i * d
        if i0 >= n:
            break
        m = min(n - i0, len(cur))
        out[(i - 1) % 2, i0:i0 + m] += cur[:m]
    return out[0], out[1]

def duck_env(n, times, depth=0.55, release=0.15, sr=SR):
    """Gain envelope that dips at every time in `times` (sidechain pump)."""
    env = np.ones(n)
    tail = int(release * 4.0 * sr)
    rel = np.arange(tail) / sr
    g = 1.0 - depth * np.exp(-rel / release)
    for t in sorted(times):
        i0 = int(t * sr)
        if i0 >= n:
            break
        j = min(n, i0 + tail)
        env[i0:j] = np.minimum(env[i0:j], g[: j - i0])
    return env

# --------------------------------------------------------------- master ----
def soft_clip(x, drive=1.2):
    return np.tanh(x * drive)

def normalize(x, peak=0.97):
    m = np.max(np.abs(x))
    return x * (peak / m) if m > 1e-9 else x

def db(x):
    return 20.0 * np.log10(np.maximum(np.abs(x), 1e-10))
