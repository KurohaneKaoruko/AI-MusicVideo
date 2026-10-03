"""Grab verification frames from the finished MV (ffmpeg seek + PIL tile)."""
import os
import subprocess
import sys

from PIL import Image

FF = r"C:\Users\Kurohanekaoruko\AppData\Roaming\Python\Python39\site-packages\imageio_ffmpeg\binaries\ffmpeg-win-x86_64-v7.1.exe"

src = sys.argv[1]
out = sys.argv[2]
times = [float(x) for x in sys.argv[3].split(",")]
cols = int(sys.argv[4]) if len(sys.argv) > 4 else 4

scratch = os.path.join(os.path.dirname(out) or ".", "_grab")
os.makedirs(scratch, exist_ok=True)

tw, th = 560, 315
tiles = []
for t in times:
    p = os.path.join(scratch, f"fr_{t}.png")
    subprocess.run(
        [FF, "-y", "-ss", str(t), "-i", src, "-frames:v", "1", p,
         "-hide_banner", "-loglevel", "error"],
        check=True,
    )
    tiles.append(Image.open(p).convert("RGB").resize((tw, th), Image.Resampling.LANCZOS))
    os.remove(p)

rows = (len(tiles) + cols - 1) // cols
sheet = Image.new("RGB", (cols * (tw + 6) + 6, rows * (th + 6) + 6), (16, 14, 18))
for i, im in enumerate(tiles):
    r, c = divmod(i, cols)
    sheet.paste(im, (6 + c * (tw + 6), 6 + r * (th + 6)))
sheet.save(out)
print(f"{len(tiles)} frames -> {out} {sheet.size}")
