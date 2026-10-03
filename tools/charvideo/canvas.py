"""charvideo.canvas -- a W x H grid of characters with per-cell RGB color."""
import numpy as np


class Canvas:
    def __init__(self, w, h, bg=(0.008, 0.012, 0.03)):
        self.w = int(w)
        self.h = int(h)
        self.ch = np.full((self.h, self.w), ' ', dtype='<U1')
        self.fg = np.zeros((self.h, self.w, 3), dtype=np.float64)
        self.bg_row = np.tile(np.asarray(bg, np.float64), (self.h, 1))

    # ------------------------------------------------------------- basic --
    def clear(self):
        self.ch[:] = ' '
        self.fg[:] = 0.0

    def put(self, x, y, c, rgb):
        if 0 <= int(x) < self.w and 0 <= int(y) < self.h and c:
            self.ch[int(y), int(x)] = c
            self.fg[int(y), int(x)] = rgb

    def put_max(self, x, y, c, rgb):
        """Only overwrite if brighter (for additive-looking overlaps)."""
        x, y = int(x), int(y)
        if 0 <= x < self.w and 0 <= y < self.h and c:
            cur = self.fg[y, x].sum()
            if rgb_sum(rgb) >= cur or self.ch[y, x] == ' ':
                self.ch[y, x] = c
                self.fg[y, x] = rgb

    def fade(self, f):
        self.fg *= f

    def tint(self, rgb):
        self.fg *= np.asarray(rgb, np.float64)

    def add_color(self, rgb):
        """Add flat light to every non-space cell (flash)."""
        mask = self.ch != ' '
        self.fg[mask] = np.clip(self.fg[mask] + rgb, 0, 1)

    def fill_flash(self, rgb, strength):
        if strength <= 0:
            return
        m = min(strength, 1.0)
        if m > 0.85:
            self.ch[:, :] = '\u2593'
        self.fg[:] = np.clip(self.fg + np.asarray(rgb) * m, 0, 1)

    # -------------------------------------------------------------- text --
    def text(self, x, y, s, rgb, spacing=1):
        for i, c in enumerate(s):
            self.put(x + i * (1 + spacing), y, c, rgb)

    def text_c(self, y, s, rgb, spacing=1):
        """Horizontally centered text."""
        wpx = sum(1 + spacing for _ in s) - spacing
        self.text((self.w - wpx) // 2, y, s, rgb, spacing)

    def big_text(self, x, y, s, rgb, scale=1, char='#', split=0, alpha=1.0):
        """5x7 bitmap font, `scale` cells per pixel.  split>0 draws a
        chromatic-aberration ghost layer."""
        from . import font5x7
        rgb = np.asarray(rgb, np.float64) * alpha
        if split > 0:
            gl = font5x7.text_grid(s)
            for j, row in enumerate(gl):
                for i, px in enumerate(row):
                    if px == '1':
                        for dy in range(scale):
                            for dx in range(scale):
                                self.put_max(x + (i * scale + dx) - split,
                                             y + j * scale + dy, char,
                                             (0.9 * alpha, 0.15 * alpha, 0.35 * alpha))
                                self.put_max(x + (i * scale + dx) + split,
                                             y + j * scale + dy, char,
                                             (0.2 * alpha, 0.6 * alpha, 1.0 * alpha))
        g = font5x7.text_grid(s)
        for j, row in enumerate(g):
            for i, px in enumerate(row):
                if px == '1':
                    for dy in range(scale):
                        for dx in range(scale):
                            self.put_max(x + i * scale + dx, y + j * scale + dy,
                                         char, rgb)

    def big_text_c(self, y, s, rgb, scale=1, char='#', split=0, alpha=1.0):
        gw = (len(s) * 6 - 1) * scale
        self.big_text(max(0, (self.w - gw) // 2), y, s, rgb, scale, char, split, alpha)

    # ------------------------------------------------------------- lines --
    def hline(self, x0, x1, y, c='-', rgb=(0.4, 0.4, 0.5)):
        for x in range(int(x0), int(x1) + 1):
            self.put(x, y, c, rgb)

    def vline(self, y0, y1, x, c='|', rgb=(0.4, 0.4, 0.5)):
        for y in range(int(y0), int(y1) + 1):
            self.put(x, y, c, rgb)

    def box(self, x0, y0, x1, y1, rgb=(0.4, 0.4, 0.5)):
        self.hline(x0, x1, y0, '-', rgb)
        self.hline(x0, x1, y1, '-', rgb)
        self.vline(y0, y1, x0, '|', rgb)
        self.vline(y0, y1, x1, '|', rgb)

    def shift_row(self, y, x0, w, dx):
        """Glitch: roll a row segment horizontally by dx."""
        if not (0 <= y < self.h):
            return
        x0 = max(0, int(x0)); w = min(int(w), self.w - x0)
        if w <= 1:
            return
        seg_c = np.roll(self.ch[y, x0:x0 + w], int(dx))
        seg_f = np.roll(self.fg[y, x0:x0 + w], int(dx), axis=0)
        self.ch[y, x0:x0 + w] = seg_c
        self.fg[y, x0:x0 + w] = seg_f

    # ------------------------------------------------------------ output --
    def set_bg_gradient(self, top, bottom):
        top = np.asarray(top, np.float64); bottom = np.asarray(bottom, np.float64)
        f = np.linspace(0, 1, self.h)[:, None]
        self.bg_row = top * (1 - f) + bottom * f

    def to_pil(self, font, cell_w, cell_h):
        from PIL import Image, ImageDraw
        W, H = self.w * cell_w, self.h * cell_h
        img = Image.new('RGB', (W, H))
        dr = ImageDraw.Draw(img)
        for y in range(self.h):
            c = tuple(int(v * 255) for v in self.bg_row[y])
            dr.rectangle([0, y * cell_h, W, (y + 1) * cell_h - 1], fill=c)
        ys, xs = np.nonzero(self.ch != ' ')
        for y, x in zip(ys.tolist(), xs.tolist()):
            rgb = self.fg[y, x]
            dr.text((x * cell_w, y * cell_h), self.ch[y, x], font=font,
                    fill=(int(rgb[0] * 255), int(rgb[1] * 255), int(rgb[2] * 255)))
        return img

    def to_ansi(self):
        out = ['\x1b[0m\x1b[H']
        for y in range(self.h):
            bg = tuple(int(v * 255) for v in self.bg_row[y])
            out.append(f'\x1b[48;2;{bg[0]};{bg[1]};{bg[2]}m')
            last = None
            row = self.ch[y]; fgm = self.fg[y]
            for x in range(self.w):
                c = row[x]
                if c == ' ':
                    out.append(' ')
                    continue
                r, g, b = (int(v * 255) for v in fgm[x])
                key = (r, g, b)
                if key != last:
                    out.append(f'\x1b[38;2;{r};{g};{b}m')
                    last = key
                out.append(c)
            out.append('\n')
        out.append('\x1b[0m')
        return ''.join(out)


def rgb_sum(rgb):
    return float(np.asarray(rgb, np.float64).sum())
