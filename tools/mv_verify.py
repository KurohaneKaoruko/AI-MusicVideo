# -*- coding: utf-8 -*-
"""Verify a rendered MV: probe streams + extract sample frames.

Usage:
    python mv_verify.py <video.mp4> [outdir]

Probe output goes to stdout; sample frames (10 evenly spaced) are written
to <outdir> as PNG. Defaults to .preview/ next to the video.
"""
import os
import subprocess
import sys

try:
    import imageio_ffmpeg

    ff = imageio_ffmpeg.get_ffmpeg_exe()
except Exception:
    ff = 'ffmpeg'


def main() -> None:
    if len(sys.argv) < 2:
        sys.exit('usage: python mv_verify.py <video.mp4> [outdir]')
    src = sys.argv[1]
    outdir = sys.argv[2] if len(sys.argv) > 2 else os.path.join(
        os.path.dirname(os.path.abspath(src)), '.preview')
    os.makedirs(outdir, exist_ok=True)

    r = subprocess.run([ff, '-i', src], capture_output=True, text=True)
    for line in r.stderr.splitlines():
        if any(k in line for k in ('Duration', 'Stream', 'bitrate')):
            print(line.strip())

    times = [5, 30, 62, 80, 99, 113, 125, 150, 166, 180]
    for i, t in enumerate(times):
        out = os.path.join(outdir, f'vrf_{i:02d}_{t}s.png')
        subprocess.run([ff, '-y', '-ss', str(t), '-i', src, '-frames:v', '1', out],
                       capture_output=True)
        print('frame', t, '->', os.path.basename(out))
    print('verify done')


if __name__ == '__main__':
    main()
