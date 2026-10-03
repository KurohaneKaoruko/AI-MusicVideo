#!/usr/bin/env python
"""Audio feature extraction for the NotToNotice(); v2 MV.

Decodes the mp3 with ffmpeg, then produces one feature vector per video frame
(60 fps): band energies, spectral centroid, onset strength.  Detects musical
accents (hits) and fits a tempo grid to the lyric anchors.

Output: analysis/out/analysis.json
"""
import json
import math
import os
import subprocess

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
AUDIO = os.path.join(ROOT, "NotToNotice.mp3")
OUT = os.path.join(HERE, "out", "analysis.json")

SR = 44100
FPS = 60
HOP = SR // FPS            # 735 samples == exactly one video frame
WIN = 2048                 # analysis window for the per-frame features
TOTAL = 183.0
NFRAMES = int(math.ceil(TOTAL * FPS))

# lyric anchors (seconds) from the official LRC, with the offset pairing fixed
ANCHORS = [
    10.89, 13.61, 17.21, 23.84, 30.66, 33.18, 37.89, 44.36, 51.34, 58.14,
    87.62, 90.76, 94.07, 100.96, 104.47, 107.68, 111.07, 115.03, 121.55,
    128.56, 135.73,
]


def find_ffmpeg():
    try:
        import imageio_ffmpeg
        return imageio_ffmpeg.get_ffmpeg_exe()
    except Exception:
        pass
    from shutil import which
    p = which("ffmpeg")
    if p:
        return p
    raise SystemExit("ffmpeg not found")


def decode(ff):
    cmd = [ff, "-v", "quiet", "-i", AUDIO, "-f", "f32le", "-ac", "1", "-ar", str(SR), "-"]
    raw = subprocess.run(cmd, capture_output=True, check=True).stdout
    return np.frombuffer(raw, dtype="<f4").astype(np.float32)


def stft_mag(x, hop, win):
    n = 1 + (len(x) - win) // hop
    idx = np.arange(win)[None, :] + hop * np.arange(n)[:, None]
    w = np.hanning(win).astype(np.float32)
    return np.abs(np.fft.rfft(x[idx] * w[None, :], axis=1)).astype(np.float32)


def band_energy(mag, lo, hi):
    n = mag.shape[1]
    ia = max(0, min(int(lo / (SR / 2) * n), n - 1))
    ib = max(ia + 1, min(int(hi / (SR / 2) * n), n))
    return np.sqrt((mag[:, ia:ib] ** 2).mean(axis=1))


def norm_pct(v, lo_pct=3.0, hi_pct=98.0, gamma=1.0):
    db = 20.0 * np.log10(np.maximum(v, 1e-9))
    lo, hi = float(np.percentile(db, lo_pct)), float(np.percentile(db, hi_pct))
    if hi - lo < 1e-6:
        hi = lo + 1.0
    return np.clip(np.power(np.clip((db - lo) / (hi - lo), 0, 1), gamma), 0, 1).astype(np.float32)


def resample_to_frames(v, src_rate, n):
    """resample a per-hop series onto video-frame centres"""
    pos = np.clip((np.arange(n) + 0.5) * FPS / src_rate - 0.5, 0, len(v) - 1)
    i0 = np.floor(pos).astype(np.int64)
    i1 = np.minimum(i0 + 1, len(v) - 1)
    f = (pos - i0).astype(np.float32)
    return (v[i0] * (1 - f) + v[i1] * f).astype(np.float32)


# ------------------------------------------------------------- onset / beat ---

def onset_envelope(x, hop=256, win=1024):
    """log-magnitude spectral flux, detrended by a ~0.7 s moving average"""
    mag = stft_mag(x, hop, win)
    n = mag.shape[1]
    hi = max(16, min(int(9000.0 / (SR / 2) * n), n))
    logm = np.log1p(mag[:, :hi] * 90.0)
    flux = np.maximum(0.0, np.diff(logm, axis=0)).sum(axis=1)
    flux = np.concatenate([[0.0], flux])
    w = max(3, int(0.7 * SR / hop))
    base = np.convolve(flux, np.ones(w) / w, mode="same")
    env = np.maximum(flux - base, 0.0)
    env /= max(float(np.percentile(env, 99.5)), 1e-9)
    return np.clip(env, 0, 1.2).astype(np.float32), SR / hop


def pick_hits(env, rate, min_gap=0.08, mult=1.5):
    w = max(3, int(0.32 * rate))
    base = np.convolve(env, np.ones(w) / w, mode="same")
    thr = base * mult + 0.015
    cand = np.where((env > thr) & (env > np.roll(env, 1)) & (env >= np.roll(env, -1)))[0]
    hits, last = [], -1e9
    for i in cand:
        t = i / rate
        if t - last < min_gap:
            continue
        hits.append((round(float(t), 4), float(env[i])))
        last = t
    return hits


def fit_tempo(hits, lo_bpm=55.0, hi_bpm=200.0, step=0.02):
    """Phase-independent tempo coherence.

    z(period) = | sum_i  s_i * exp(2*pi*i * t_i / period) | / sum_i s_i
    Peaks of z mean the hits share a common periodicity; the argument of z
    gives the phase directly, so no phase search is needed.
    """
    if len(hits) < 20:
        return [(120.0, 0.0, 0.0)]
    ht = np.array([h[0] for h in hits])
    hs = np.array([h[1] for h in hits])
    hs = hs / max(hs.max(), 1e-9)
    bpms = np.arange(lo_bpm, hi_bpm, step)
    periods = 60.0 / bpms
    z = (hs[:, None] * np.exp(2j * np.pi * ht[:, None] / periods[None, :])).sum(axis=0)
    z /= hs.sum()
    score = np.abs(z)
    offs = (np.angle(z) / (2 * np.pi)) % 1.0 * periods

    cand = []
    for i in range(1, len(score) - 1):
        if score[i] > score[i - 1] and score[i] >= score[i + 1]:
            cand.append((float(bpms[i]), float(score[i]), float(offs[i])))
    cand.sort(key=lambda r: -r[1])
    peaks, seen = [], []
    for bpm, sc, off in cand:
        if any(abs(bpm - b) < 3.0 for b in seen):
            continue
        seen.append(bpm)
        peaks.append((bpm, sc, off))
        if len(peaks) >= 10:
            break
    return peaks or [(120.0, 0.0, 0.0)]


def band_onset(x, lo, hi, hop=256, win=1024):
    """spectral flux restricted to one band - kick (low) / hats (high)"""
    mag = stft_mag(x, hop, win)
    n = mag.shape[1]
    ia = max(0, min(int(lo / (SR / 2) * n), n - 2))
    ib = max(ia + 1, min(int(hi / (SR / 2) * n), n))
    logm = np.log1p(mag[:, ia:ib] * 90.0)
    flux = np.maximum(0.0, np.diff(logm, axis=0)).sum(axis=1)
    flux = np.concatenate([[0.0], flux])
    w = max(3, int(0.5 * SR / hop))
    base = np.convolve(flux, np.ones(w) / w, mode="same")
    env = np.maximum(flux - base, 0.0)
    env /= max(float(np.percentile(env, 99.5)), 1e-9)
    return np.clip(env, 0, 2.0).astype(np.float32), SR / hop


def lock_phase(env, rate, period):
    """pick the grid phase that collects the most energy of a given onset band"""
    best, second = (0.0, -1.0), (0.0, -1.0)
    for off in np.linspace(0.0, period, 480, endpoint=False):
        idx = np.round((np.arange(off, len(env) / rate, period)) * rate).astype(np.int64)
        idx = idx[(idx >= 0) & (idx < len(env))]
        if len(idx) < 10:
            continue
        sc = float(env[idx].mean())
        if sc > best[1]:
            second, best = best, (float(off), sc)
        elif sc > second[1] and abs(off - best[0]) > period * 0.25:
            second = (float(off), sc)
    return best, second


def best_phase_scan(hits, period, tol=0.045):
    """The coherence phase averages beats and off-beats together, so scan for
    the phase that actually collects the most accent energy instead."""
    ht = np.array([h[0] for h in hits])
    hs = np.array([h[1] for h in hits])
    hs = hs / max(hs.max(), 1e-9)
    m = hs >= np.percentile(hs, 50)
    ht, hs = ht[m], hs[m]
    best, second = (0.0, -1.0), (0.0, -1.0)
    for off in np.linspace(0.0, period, 720, endpoint=False):
        d = np.abs(ht - (off + np.round((ht - off) / period) * period))
        sc = float((hs * np.exp(-(d / tol) ** 2)).sum())
        if sc > best[1]:
            second, best = best, (float(off), sc)
        elif sc > second[1] and abs(off - best[0]) > period * 0.25:
            second = (float(off), sc)
    return best, second


def refine(hits, period, phase, tol=0.055):
    """least-squares refine, locked to the detected beat family (off-beats are
    excluded by the tolerance instead of dragging the fit into the middle)"""
    ht = np.array([h[0] for h in hits])
    hs = np.array([h[1] for h in hits])
    ht = ht[hs >= np.percentile(hs, 45)]
    t0, per = phase, period
    for _ in range(6):
        k = np.round((ht - t0) / per)
        keep = np.abs(ht - (t0 + k * per)) <= tol
        if keep.sum() < 8:
            break
        A = np.vstack([k[keep], np.ones(int(keep.sum()))]).T
        sol, *_ = np.linalg.lstsq(A, ht[keep], rcond=None)
        per, t0 = float(sol[0]), float(sol[1])
        if not (0.25 < per < 1.1):
            break
    return per, t0


# ------------------------------------------------------------------- main -----

def main():
    ff = find_ffmpeg()
    x = decode(ff)
    print(f"decoded {len(x)/SR:.3f}s ({len(x)} samples)")

    # ---- per-frame features -------------------------------------------------
    mag = stft_mag(x, HOP, WIN)
    n = mag.shape[0]
    idx = np.arange(n)[:, None] * HOP + np.arange(WIN)[None, :]
    rms = np.sqrt((x[idx] ** 2).mean(axis=1))
    low = band_energy(mag, 30, 160)
    mid = band_energy(mag, 160, 2000)
    high = band_energy(mag, 2000, 14000)
    freqs = np.fft.rfftfreq(WIN, 1.0 / SR).astype(np.float32)
    centroid = (mag * freqs[None, :]).sum(axis=1) / np.maximum(mag.sum(axis=1), 1e-9)

    def pad(v):
        v = v.astype(np.float32)
        if len(v) >= NFRAMES:
            return v[:NFRAMES]
        return np.concatenate([v, np.full(NFRAMES - len(v), v[-1] if len(v) else 0.0, np.float32)])

    f_rms = pad(norm_pct(rms, 3, 99, 0.85))
    f_low = pad(norm_pct(low, 3, 98, 0.9))
    f_mid = pad(norm_pct(mid, 3, 98, 0.9))
    f_high = pad(norm_pct(high, 3, 98, 0.9))
    f_cent = pad(np.clip((centroid - 200.0) / 4200.0, 0, 1))

    # ---- onset / tempo ------------------------------------------------------
    env, rate = onset_envelope(x)
    hits = pick_hits(env, rate)
    print(f"onset hits: {len(hits)}  ({len(hits)/ (len(x)/SR):.2f}/s)")

    peaks = fit_tempo(hits)
    print("tempo candidates  (bpm, hit-alignment, phase):")
    for bpm, sc, off in peaks[:8]:
        print(f"    {bpm:7.2f}   {sc:.3f}   {off:.4f}")

    # prefer a musical range for the visual pulse, but report the raw winner
    musical = [p for p in peaks if 122.0 <= p[0] <= 178.0]
    chosen = max(musical or peaks, key=lambda p: p[1])
    per = 60.0 / chosen[0]
    print(f"coherence winner: bpm={chosen[0]:.3f} align={chosen[1]:.3f} "
          f"phase={chosen[2]:.4f}")

    (p1, s1), (p2, s2) = best_phase_scan(hits, per)
    low_env, low_rate = band_onset(x, 28, 190)
    high_env, high_rate = band_onset(x, 2500, 11000)
    (lp, ls), (lp2, ls2) = lock_phase(low_env, low_rate, per)
    (hp, hs_), (hp2, hs2) = lock_phase(high_env, high_rate, per)
    print(f"phase scan (broadband hits): best={p1:.4f} ({s1:.2f})  runner-up={p2:.4f} ({s2:.2f})")
    print(f"phase lock (kick  28-190Hz): best={lp:.4f} ({ls:.3f})  runner-up={lp2:.4f} ({ls2:.3f})")
    print(f"phase lock (hats 2.5-11kHz): best={hp:.4f} ({hs_:.3f})  runner-up={hp2:.4f} ({hs2:.3f})")

    per, t0 = refine(hits, per, lp)
    bpm = 60.0 / per
    print(f"chosen grid: bpm={bpm:.4f} period={per:.5f} t0={t0:.4f}")
    print(f"  kick coverage on grid = {lock_phase(low_env, low_rate, per)[0][1]:.3f} "
          f"(phase rerun)")
    d = np.array([h[0] for h in hits if h[1] >= 0.35 * max(x[1] for x in hits)])
    d = d - (t0 + np.round((d - t0) / per) * per)
    print(f"strong-hit offset: median={np.median(np.abs(d))*1000:.1f}ms  "
          f"within 50ms={np.mean(np.abs(d) < 0.05)*100:.0f}%")

    # the LRC timings are vocal syllable entries, not beat positions: report
    # them only so it is clear they are NOT used to correct the grid.
    a = np.array(ANCHORS)
    k = np.round((a - t0) / per)
    resid = a - (t0 + k * per)
    print(f"lyric anchors (informational, not used for fitting): "
          f"median offset={np.median(np.abs(resid))*1000:.1f}ms "
          f"(spread {np.abs(resid).min()*1000:.0f}..{np.abs(resid).max()*1000:.0f}ms)")

    beats = [round(float(t), 5) for t in np.arange(t0, TOTAL, per) if t >= 0]
    f_onset = pad(resample_to_frames(env, rate, NFRAMES))
    f_kick = pad(resample_to_frames(low_env, low_rate, NFRAMES))
    f_hat = pad(resample_to_frames(high_env, high_rate, NFRAMES))

    # Beat pulse WITHOUT a rigid grid.
    #
    # Diagnostic finding: the detected kick / accent peaks are distributed
    # almost uniformly against the best-fit constant grid (median distance
    # ~= period/4 at every phase), i.e. this track's groove does not lock to
    # one global BPM.  A drift-free alternative is a fast-attack / slow-decay
    # envelope follower, which is beat-synchronous by construction.
    raw = np.maximum(f_onset, 0.85 * f_kick)
    pub = np.zeros(NFRAMES, dtype=np.float32)
    decay = float(np.exp(-1.0 / (0.17 * FPS)))
    acc = 0.0
    for i in range(NFRAMES):
        acc = max(float(raw[i]), acc * decay)
        pub[i] = acc
    pub /= max(pub.max(), 1e-9)
    f_pulse = pub.astype(np.float32)

    def q(a):
        return [round(float(v), 4) for v in a]

    data = {
        "src": os.path.basename(AUDIO),
        "duration": round(len(x) / SR, 4),
        "total": TOTAL,
        "fps": FPS,
        "frames": NFRAMES,
        "bpm": round(bpm, 4),
        "beat_t0": round(float(t0), 5),
        "beat_period": round(float(per), 5),
        "beats": beats,
        "hits": [[round(t, 4), round(s, 4)] for t, s in hits],
        "anchors": ANCHORS,
        "rms": q(f_rms),
        "low": q(f_low),
        "mid": q(f_mid),
        "high": q(f_high),
        "centroid": q(f_cent),
        "onset": q(f_onset),
        "kick": q(f_kick),
        "hat": q(f_hat),
        "pulse": q(f_pulse),
    }
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as fh:
        json.dump(data, fh, ensure_ascii=False, separators=(",", ":"))
    print(f"wrote {OUT} ({os.path.getsize(OUT)/1024:.0f} KB)")

    # ---- structure report ---------------------------------------------------
    smooth = np.convolve(f_rms, np.ones(FPS) / FPS, mode="same")
    sm_low = np.convolve(f_low, np.ones(FPS) / FPS, mode="same")
    sm_mid = np.convolve(f_mid, np.ones(FPS) / FPS, mode="same")
    sm_hi = np.convolve(f_high, np.ones(FPS) / FPS, mode="same")
    print("\n   t   |  rms            | low  | mid  | high |  oc  |lyric")
    for i in range(0, NFRAMES, FPS):
        t = i / FPS

        def bar(v, w=16):
            kk = int(round(float(v) * w))
            return "#" * kk + "." * (w - kk)

        mark = "  <<<" if any(abs(a2 - t) < 0.5 for a2 in ANCHORS) else ""
        print(f"{t:6.1f} |{bar(smooth[i])}|{bar(sm_low[i],5)}|{bar(sm_mid[i],5)}|"
              f"{bar(sm_hi[i],5)}|{f_onset[i]:.2f} |{mark}")


if __name__ == "__main__":
    main()
