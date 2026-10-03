import glob
import sys
from PIL import Image

src = sys.argv[1] if len(sys.argv) > 1 else ".preview"
pattern = sys.argv[2] if len(sys.argv) > 2 else "s*.png"
out = sys.argv[3] if len(sys.argv) > 3 else ".preview/row.png"

files = sorted(glob.glob(f"{src}/{pattern}"))
if not files:
    raise SystemExit(f"no frames for {pattern}")
tw, th = 560, 315
cols = len(files)
sheet = Image.new("RGB", (tw * cols + (cols + 1) * 4, th + 8), (18, 16, 20))
for i, f in enumerate(files):
    sheet.paste(Image.open(f).convert("RGB").resize((tw, th), Image.Resampling.LANCZOS), (4 + i * (tw + 4), 4))
sheet.save(out)
print(f"{len(files)} -> {out} {sheet.size}")
