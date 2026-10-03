#!/usr/bin/env python
"""Render the packed frame stream from `world-search-you --video` into an MP4.

Usage:
  python tools/render_video.py <output.mp4> [cols] [rows] [fps]

Reads the FRV1 binary stream on stdin (pipe it from the exe), draws each frame
at CELL_W x CELL_H per terminal cell, and feeds rawvideo to ffmpeg (bundled
with imageio-ffmpeg) together with the song from reference/*.mp3.
"""
import io
import os
import struct
import subprocess
import sys
import time

from PIL import Image, ImageDraw, ImageFont
import imageio_ffmpeg

CELL_W, CELL_H = 16, 30
FONT_SIZE = 24
BG = (10, 13, 19)

here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXE = os.path.join(here, "target", "release", "world-search-you.exe")


def find_audio():
    for d in ("reference", "assets", "."):
        p = os.path.join(here, d)
        if os.path.isdir(p):
            for f in sorted(os.listdir(p)):
                if f.lower().endswith(".mp3"):
                    return os.path.join(p, f)
    raise SystemExit("no mp3 found in reference/")


def load_fonts():
    reg = r"C:\Windows\Fonts\consola.ttf"
    bold = r"C:\Windows\Fonts\consolab.ttf"
    cjk = r"C:\Windows\Fonts\msyh.ttc"
    for p in (reg, bold, cjk):
        if not os.path.exists(p):
            raise SystemExit(f"missing font {p}")
    return (
        ImageFont.truetype(reg, FONT_SIZE),
        ImageFont.truetype(bold, FONT_SIZE),
        ImageFont.truetype(cjk, FONT_SIZE),
    )


def is_cjk(ch):
    return ord(ch) >= 0x2E80


class GlyphCache:
    """(ch, fg, bold) -> RGBA tile sized (cell_w * wide, CELL_H)."""

    def __init__(self, fonts):
        self.f_reg, self.f_bold, self.f_cjk = fonts
        self.cache = {}

    def tile(self, ch, fg, bold):
        key = (ch, fg, bold)
        t = self.cache.get(key)
        if t is not None:
            return t
        wide = is_cjk(ch)
        w = CELL_W * 2 if wide else CELL_W
        tile = Image.new("RGBA", (w, CELL_H), (0, 0, 0, 0))
        d = ImageDraw.Draw(tile)
        if wide:
            font = self.f_cjk
            pos = ((w - FONT_SIZE) // 2, (CELL_H - FONT_SIZE) // 2 + 1)
        else:
            font = self.f_bold if bold else self.f_reg
            pos = (2, (CELL_H - FONT_SIZE) // 2 + 1)
        d.text(pos, ch, font=font, fill=fg + (255,))
        if len(self.cache) > 20000:
            self.cache.clear()
        self.cache[key] = tile
        return tile


def read_exact(stream, n):
    buf = bytearray()
    while len(buf) < n:
        chunk = stream.read(n - len(buf))
        if not chunk:
            return None
        buf.extend(chunk)
    return bytes(buf)


def read_frame(stream):
    head = read_exact(stream, 20)
    if head is None:
        return None
    magic = head[:4]
    if magic != b"FRV1":
        raise SystemExit(f"bad stream magic {magic!r}")
    cols, rows = struct.unpack_from("<II", head, 4)
    (t,) = struct.unpack_from("<d", head, 12)
    rows_data = []
    for _y in range(rows):
        mode = stream.read(1)[0]
        row_bg = None
        if mode == 1:
            row_bg = tuple(stream.read(3))
        (n,) = struct.unpack_from("<H", stream.read(2))
        cells = []
        for _k in range(n):
            x, bold, fr, fg_, fb = struct.unpack("<HBBBB", read_exact(stream, 6))
            bg = None
            if mode == 0:
                bg = tuple(stream.read(3))
            ln = stream.read(1)[0]
            ch = read_exact(stream, ln).decode("utf-8")
            cells.append((x, bold, (fr, fg_, fb), bg, ch))
        rows_data.append((mode, row_bg, cells))
    return cols, rows, t, rows_data


def main():
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    out_path = sys.argv[1]
    cols = int(sys.argv[2]) if len(sys.argv) > 2 else 120
    rows = int(sys.argv[3]) if len(sys.argv) > 3 else 36
    fps = int(sys.argv[4]) if len(sys.argv) > 4 else 60

    W, H = cols * CELL_W, rows * CELL_H
    fonts = load_fonts()
    cache = GlyphCache(fonts)
    audio = find_audio()

    exe = EXE
    spec = os.environ.get("WSY_VIDEO_SPEC", "0:296:60")
    proc = subprocess.Popen(
        [exe, "--video", spec, "--cols", str(cols), "--rows", str(rows)],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        stdin=subprocess.DEVNULL,
    )
    stream = proc.stdout

    ff = imageio_ffmpeg.get_ffmpeg_exe()
    end_t = float(spec.split(":")[1])
    noaudio = os.environ.get("WSY_VIDEO_NOAUDIO") == "1"
    cmd = [
        ff, "-y",
        "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", str(fps), "-i", "-",
    ]
    if noaudio:
        # segment mode: video only, container duration == frames / fps
        cmd += ["-c:v", "libx264", "-preset", "faster", "-crf", "18",
                "-pix_fmt", "yuv420p", out_path]
    else:
        # standalone mode: mux the song, padded to the full video length
        cmd += [
            "-i", audio,
            "-c:v", "libx264", "-preset", "faster", "-crf", "18", "-pix_fmt", "yuv420p",
            "-c:a", "aac", "-b:a", "192k",
            "-af", "apad", "-t", f"{end_t:.3f}",
            "-movflags", "+faststart",
            out_path,
        ]
    ffmpeg = subprocess.Popen(cmd, stdin=subprocess.PIPE,
                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    t0 = time.time()
    n = 0
    # persistent framebuffer + dirty-row tracking: most rows are identical
    # between frames, so only changed rows are repainted
    img = Image.new("RGB", (W, H), BG)
    prev_rows = [None] * rows
    d = ImageDraw.Draw(img)
    while True:
        fr = read_frame(stream, )
        if fr is None:
            break
        cols_f, rows_f, t, rows_data = fr
        for y in range(rows):
            mode, row_bg, cells = rows_data[y]
            key = (mode, row_bg, cells)
            if key == prev_rows[y]:
                continue
            y0, y1 = y * CELL_H, (y + 1) * CELL_H
            if mode == 1:
                d.rectangle([0, y0, W - 1, y1 - 1], fill=row_bg if row_bg is not None else BG)
                for (x, bold, fg, _bg, ch) in cells:
                    tile = cache.tile(ch, fg, bool(bold))
                    img.paste(tile, (x * CELL_W, y0), tile)
            else:
                d.rectangle([0, y0, W - 1, y1 - 1], fill=BG)
                for (x, bold, fg, bg, ch) in cells:
                    if bg is not None and bg != BG:
                        d.rectangle([x * CELL_W, y0, (x + 1) * CELL_W - 1, y1 - 1], fill=bg)
                    tile = cache.tile(ch, fg, bool(bold))
                    img.paste(tile, (x * CELL_W, y0), tile)
            prev_rows[y] = key
        ffmpeg.stdin.write(img.tobytes())
        n += 1
        if n % 300 == 0:
            el = time.time() - t0
            print(f"\r{n} frames | {el:.0f}s | {n/el:.1f} fps | t={t:.1f}s", end="", flush=True)

    proc.wait()
    ffmpeg.stdin.close()
    ffmpeg.wait()
    print(f"\ndone: {n} frames -> {out_path}")


if __name__ == "__main__":
    main()
