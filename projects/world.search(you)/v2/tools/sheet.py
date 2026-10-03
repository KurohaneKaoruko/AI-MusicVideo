#!/usr/bin/env python
"""Build a contact sheet from .preview/*.png (design aid for the v2 MV)."""
import glob
import os
import sys

from PIL import Image

src = sys.argv[1] if len(sys.argv) > 1 else ".preview"
out = sys.argv[2] if len(sys.argv) > 2 else ".preview/sheet.png"
cols = int(sys.argv[3]) if len(sys.argv) > 3 else 4

files = sorted(glob.glob(os.path.join(src, "s*.png")))
if not files:
    raise SystemExit(f"no frames in {src}")

tw, th = 470, 264
pad = 6
rows = (len(files) + cols - 1) // cols
sheet = Image.new("RGB", (cols * (tw + pad) + pad, rows * (th + pad) + pad), (18, 16, 20))

for i, f in enumerate(files):
    im = Image.open(f).convert("RGB").resize((tw, th), Image.LANCZOS)
    r, c = divmod(i, cols)
    sheet.paste(im, (pad + c * (tw + pad), pad + r * (th + pad)))

sheet.save(out)
print(f"{len(files)} frames -> {out} ({sheet.size[0]}x{sheet.size[1]})")
