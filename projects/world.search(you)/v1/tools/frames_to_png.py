#!/usr/bin/env python
"""Render a .preview/<name>.txt frame dump (from `world-search-you --capture`) into PNGs.

Usage:  python tools/frames_to_png.py frames [outdir] [frames_per_image]
Dump format: "#FRAME <t> <title>" then one line per row; each cell is
"r,g,b,bold,br,bg,bb<US>char" joined by <US>.
"""
import sys, os
from PIL import Image, ImageDraw, ImageFont

CW, CH = 9, 19
PAD = 16


def load_font(path, size):
    try:
        return ImageFont.truetype(path, size)
    except Exception:
        return ImageFont.load_default()


def parse(data):
    frames, cur = [], None
    for line in data.split("\n"):
        if line.startswith("#FRAME "):
            parts = line.split(" ", 2)
            cur = {"t": parts[1], "title": parts[2] if len(parts) > 2 else "", "rows": []}
            frames.append(cur)
        elif cur is not None and line:
            cells = []
            for meta, ch in zip(line.split("\u0001")[0::2], line.split("\u0001")[1::2]):
                f = meta.split(",")
                if len(f) < 7:
                    continue
                cells.append((int(f[0]), int(f[1]), int(f[2]), f[3] == "1",
                              int(f[4]), int(f[5]), int(f[6]), ch))
            cur["rows"].append(cells)
    return frames


def main():
    name = sys.argv[1] if len(sys.argv) > 1 else "frames"
    outdir = sys.argv[2] if len(sys.argv) > 2 else ".preview/png"
    per = int(sys.argv[3]) if len(sys.argv) > 3 else 4
    os.makedirs(outdir, exist_ok=True)
    with open(os.path.join(".preview", name + ".txt"), encoding="utf-8") as f:
        frames = parse(f.read())

    font = load_font(r"C:\Windows\Fonts\consola.ttf", 16)
    font_cjk = load_font(r"C:\Windows\Fonts\msyh.ttc", 16)
    font_ttl = load_font(r"C:\Windows\Fonts\consola.ttf", 15)

    made = []
    for start in range(0, len(frames), per):
        chunk = frames[start:start + per]
        gw = max((len(r) for fr in chunk for r in fr["rows"]), default=80)
        gh = max((len(fr["rows"]) for fr in chunk), default=24)
        W = PAD * 2 + gw * CW
        H = PAD * 2 + len(chunk) * (gh * CH + 30)
        img = Image.new("RGB", (W, H), (5, 7, 12))
        d = ImageDraw.Draw(img)
        y = PAD
        for fr in chunk:
            d.text((PAD, y), f"t={fr['t']}s   {fr['title']}", fill=(255, 109, 138), font=font_ttl)
            y += 22
            for row in fr["rows"]:
                for i, (r, g, b, bold, br, bg, bb, ch) in enumerate(row):
                    x = PAD + i * CW
                    if (br, bg, bb) != (10, 13, 19):
                        d.rectangle([x, y, x + CW, y + CH], fill=(br, bg, bb))
                    if ch != " ":
                        f = font_cjk if ord(ch[0]) > 0x2E80 else font
                        d.text((x, y), ch, fill=(r, g, b), font=f)
                y += CH
            y += 8
            d.line([(PAD, y - 4), (W - PAD, y - 4)], fill=(26, 32, 44))
        p = os.path.join(outdir, f"{name}_{start:03d}.png")
        img.save(p)
        made.append(p)
    print("\n".join(made))


if __name__ == "__main__":
    main()
