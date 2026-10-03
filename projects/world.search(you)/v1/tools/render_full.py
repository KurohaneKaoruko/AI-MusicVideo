#!/usr/bin/env python
"""Render the full MV in parallel segments, then concat + mux the song.

Usage:  python tools/render_full.py [output.mp4] [fps]
Splits 0..296 s into N segments (N = 6), renders each with its own
world-search-you + ffmpeg pair (one process per logical CPU half), then
concatenates losslessly and muxes the reference mp3 as AAC.
"""
import os
import subprocess
import sys
import time

import imageio_ffmpeg

here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXE = os.path.join(here, "target", "release", "world-search-you.exe")
PY = sys.executable
FF = imageio_ffmpeg.get_ffmpeg_exe()
RENDERER = os.path.join(here, "tools", "render_video.py")

TOTAL = 296.0
FPS = 60
COLS, ROWS = 120, 36
SEGMENTS = 6
TMP = os.path.join(here, ".preview", "segs")


def find_audio():
    for d in ("reference", "assets", "."):
        p = os.path.join(here, d)
        if os.path.isdir(p):
            for f in sorted(os.listdir(p)):
                if f.lower().endswith(".mp3"):
                    return os.path.join(p, f)
    raise SystemExit("no mp3 found")


def render_segment(idx, t0, t1):
    env = dict(os.environ, WSY_VIDEO_SPEC=f"{t0}:{t1}:{FPS}")
    seg = os.path.join(TMP, f"seg_{idx:02d}.mp4")
    with open(seg, "wb") as out:
        pass
    cmd = [PY, RENDERER, seg, str(COLS), str(ROWS), str(FPS)]
    r = subprocess.run(cmd, env=env, cwd=here,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return seg, r.returncode


def main():
    out_path = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
        here, "dist", "world.search (you) ; - Mili [terminal MV 1080p60].mp4")
    global FPS
    if len(sys.argv) > 2:
        FPS = int(sys.argv[2])
    os.makedirs(TMP, exist_ok=True)
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    audio = find_audio()

    seg_len = TOTAL / SEGMENTS
    jobs = []
    t0 = time.time()
    for i in range(SEGMENTS):
        a = i * seg_len
        b = TOTAL if i == SEGMENTS - 1 else (i + 1) * seg_len
        jobs.append((i, a, b))

    # run all segments in parallel (video-only; audio is muxed at the concat step)
    procs = []
    for (i, a, b) in jobs:
        env = dict(os.environ,
                   WSY_VIDEO_SPEC=f"{a:.3f}:{b:.3f}:{FPS}",
                   WSY_VIDEO_NOAUDIO="1")
        seg = os.path.join(TMP, f"seg_{i:02d}.mp4")
        p = subprocess.Popen(
            [PY, RENDERER, seg, str(COLS), str(ROWS), str(FPS)],
            env=env, cwd=here, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        procs.append((i, seg, p))

    fails = 0
    for i, seg, p in procs:
        p.wait()
        if p.returncode != 0:
            print(f"segment {i} FAILED")
            fails += 1
    if fails:
        raise SystemExit(f"{fails} segment(s) failed")

    # concat list
    list_file = os.path.join(TMP, "list.txt")
    with open(list_file, "w", encoding="utf-8") as f:
        for i in range(SEGMENTS):
            seg = os.path.join(TMP, f"seg_{i:02d}.mp4")
            f.write("file '" + seg.replace("'", "'\\''") + "'\n")

    # concat (lossless) + mux audio, padding it to the full length
    subprocess.run([
        FF, "-y",
        "-f", "concat", "-safe", "0", "-i", list_file,
        "-i", audio,
        "-map", "0:v:0", "-map", "1:a:0",
        "-c:v", "copy",
        "-c:a", "aac", "-b:a", "192k",
        "-af", "apad",
        "-t", f"{TOTAL:.3f}",
        "-movflags", "+faststart",
        out_path,
    ], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    # cleanup
    for i in range(SEGMENTS):
        try:
            os.remove(os.path.join(TMP, f"seg_{i:02d}.mp4"))
        except OSError:
            pass
    try:
        os.remove(list_file)
    except OSError:
        pass

    el = time.time() - t0
    size = os.path.getsize(out_path) / 1e6
    print(f"done in {el:.0f}s -> {out_path} ({size:.1f} MB)")


if __name__ == "__main__":
    main()
