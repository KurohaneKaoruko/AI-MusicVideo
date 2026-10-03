#!/usr/bin/env python
"""Generic segment-parallel MV renderer for the terminal-MV projects.

Works with any renderer exe that supports:
    <exe> --geometry                     -> "cols rows cell_w cell_h W H"
    <exe> --video START:END:FPS --out F  -> render frames [START,END) to mp4
    <exe> --dev beats                    -> (optional) analysis self-test

Pipeline: split the timeline into N segments, render them in parallel
(each renderer exe finds its own ffmpeg via NTN_FFMPEG/MV_FFMPEG/PATH),
losslessly concatenate, then mux the song as AAC.

Usage:
  python tools/mv_render.py --exe <path> --audio <mp3> --out <mp4> \
      [--duration 183.2] [--fps 60] [--segments 6] [--crf 18]

The output is 1080p60 H.264 (resolution comes from the exe's geometry).
"""
import argparse
import os
import subprocess
import sys
import tempfile
import time


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
    raise SystemExit("ffmpeg not found: pip install imageio-ffmpeg or add ffmpeg to PATH")


def exe_geometry(exe):
    out = subprocess.run([exe, "--geometry"], capture_output=True, text=True, check=True).stdout
    cols, rows, cw, ch, w, h = (int(x) for x in out.split())
    return cols, rows, cw, ch, w, h


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--exe", required=True)
    ap.add_argument("--audio", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--duration", type=float, default=None, help="total video length in seconds")
    ap.add_argument("--frames", type=int, default=None,
                    help="total frame count; splits land on exact frame boundaries "
                         "(takes precedence over --duration)")
    ap.add_argument("--fps", type=int, default=60)
    ap.add_argument("--segments", type=int, default=6)
    ap.add_argument("--crf", type=int, default=18)
    ap.add_argument("--preset", default="faster")
    ap.add_argument("--keep-segments", action="store_true")
    args = ap.parse_args()

    exe = os.path.abspath(args.exe)
    audio = os.path.abspath(args.audio)
    out = os.path.abspath(args.out)
    ff = find_ffmpeg()

    _, _, _, _, W, H = exe_geometry(exe)
    print(f"renderer geometry: {W}x{H}")

    if args.duration is None:
        # probe the audio duration
        r = subprocess.run(
            [ff, "-i", audio, "-f", "null", "-"], capture_output=True, text=True)
        dur = 0.0
        for line in (r.stderr or "").splitlines():
            if "Duration:" in line:
                hh, mm, ss = line.split("Duration:")[1].split(",")[0].strip().split(":")
                dur = int(hh) * 3600 + int(mm) * 60 + float(ss)
        args.duration = dur + 1.0  # small tail for the final fade
    total = args.frames / args.fps if args.frames else args.duration
    print(f"total {total:.2f}s @ {args.fps}fps in {args.segments} segments -> {out}")

    os.makedirs(os.path.dirname(out) or ".", exist_ok=True)
    tmp = tempfile.mkdtemp(prefix="mvseg_")
    seg_paths = []
    procs = []
    t0 = time.time()

    # Segment boundaries. Frame-indexed when --frames is given: a boundary at
    # t = n/fps is not exactly representable in binary, so a duration-based split
    # can drift by a frame per segment and shift the tail of the film. Splitting
    # the integer frame range instead keeps the total frame count exact.
    if args.frames:
        total_frames = args.frames

        def bounds(i):
            f0 = total_frames * i // args.segments
            f1 = total_frames * (i + 1) // args.segments
            return f0 / args.fps, f1 / args.fps

    else:
        seg_len = total / args.segments

        def bounds(i):
            a = i * seg_len
            b = total if i == args.segments - 1 else (i + 1) * seg_len
            return a, b

    for i in range(args.segments):
        a, b = bounds(i)
        seg = os.path.join(tmp, f"seg_{i:02d}.mp4")
        seg_paths.append(seg)
        env = dict(os.environ, NTN_FFMPEG=ff, MV_FFMPEG=ff)
        # forward --audio so the exe does not depend on the caller's cwd to
        # locate the song (it falls back to searching ./ for any mp3)
        p = subprocess.Popen(
            [exe, "--audio", audio, "--video", f"{a:.6f}:{b:.6f}:{args.fps}", "--out", seg],
            env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        procs.append((i, p))

    fails = 0
    for i, p in procs:
        p.wait()
        if p.returncode != 0:
            print(f"segment {i} FAILED (rc={p.returncode})")
            fails += 1
    if fails:
        raise SystemExit(f"{fails} segment(s) failed")
    print(f"segments rendered in {time.time()-t0:.0f}s")

    # concat list (relative-safe quoting)
    list_file = os.path.join(tmp, "list.txt")
    with open(list_file, "w", encoding="utf-8") as f:
        for seg in seg_paths:
            f.write("file '" + seg.replace("'", "'\\''") + "'\n")

    print("concatenating + muxing audio…")
    subprocess.run([
        ff, "-y",
        "-f", "concat", "-safe", "0", "-i", list_file,
        "-i", audio,
        "-map", "0:v:0", "-map", "1:a:0",
        "-c:v", "copy",
        "-c:a", "aac", "-b:a", "192k",
        "-af", "apad",
        "-t", f"{total:.3f}",
        "-movflags", "+faststart",
        out,
    ], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    if not args.keep_segments:
        for seg in seg_paths:
            try:
                os.remove(seg)
            except OSError:
                pass
        try:
            os.remove(list_file)
            os.rmdir(tmp)
        except OSError:
            pass

    size = os.path.getsize(out) / 1e6
    print(f"done in {time.time()-t0:.0f}s -> {out} ({size:.1f} MB)")


if __name__ == "__main__":
    main()
