// ---------------------------------------------------------------------------
// type.js - the typographic layer.  Everything is drawn into one 2D canvas
// which the engine composites into the scene *before* bloom, so type glows
// exactly like every other light source in the film.
// ---------------------------------------------------------------------------

import { clamp, lerp, smoothstep, ease, hash2, css } from "./core.js";

// Skia falls off a cliff when a large canvas mixes `lighter` compositing with
// shadowBlur / ctx.filter: rasterisation is deferred, so the bill lands on the
// texture upload instead of the JS. These knobs let us measure and trim it.
export const TYPE_FLAGS = {
  glow: true,    // ctx.shadowBlur halo on glyphs
  filter: true,  // ctx.filter blur during the per-character stagger
  scale: 1,      // text-layer resolution relative to the film raster
};

export const FONTS = {
  mono: '"Cascadia Mono","Consolas","DejaVu Sans Mono",monospace',
  monoB: '"Cascadia Mono","Consolas",monospace',
  jp: '"Yu Gothic","Meiryo","MS Gothic",sans-serif',
  jpS: '"Yu Mincho","MS PMincho","MS Gothic",serif',
  sans: '"Segoe UI","Yu Gothic UI",Arial,sans-serif',
};

export function fontStr(kind, size, weight = 400) {
  return `${weight} ${size}px ${FONTS[kind] || kind}`;
}

export class TypeLayer {
  constructor(ctx, W, H) {
    this.ctx = ctx;
    this.W = W;
    this.H = H;
  }

  clear() {
    const c = this.ctx;
    c.setTransform(1, 0, 0, 1, 0, 0);
    c.globalAlpha = 1;
    c.globalCompositeOperation = "source-over";
    c.clearRect(0, 0, this.W, this.H);
  }

  /** draw a run of text with optional per-character animation */
  run(o) {
    const c = this.ctx;
    const size = o.size;
    const weight = o.weight ?? 700;
    const font = fontStr(o.font || "mono", size, weight);
    const str = o.s ?? "";
    const alpha = o.alpha ?? 1;
    if (alpha <= 0.002 || !str.length) return;

    c.save();
    c.font = font;
    c.textBaseline = o.baseline || "alphabetic";
    c.textAlign = "left";
    const tracking = o.tracking ?? 0;
    const scaleX = o.scaleX ?? 1;
    const scaleY = o.scaleY ?? 1;
    const rot = o.rot ?? 0;
    const cx = o.x, cy = o.y;
    c.translate(cx, cy);
    if (rot) c.rotate(rot);
    c.scale(scaleX, scaleY);

    const glow = o.glow ?? 0;
    const glowCol = o.glowCol || o.col;
    const split = o.rgbSplit ?? 0;

    // per-character layout
    const chars = [...str];
    let total = 0;
    const widths = [];
    for (const ch of chars) { const w = c.measureText(ch).width + tracking; widths.push(w); total += w; }
    total -= tracking;
    const align = o.align || "left";
    let x = align === "center" ? -total / 2 : align === "right" ? -total : 0;

    const prog = o.progress ?? 1;
    const stagger = o.stagger ?? 0.55;
    const rise = o.rise ?? 0;
    const charEase = o.charEase || "outCubic";
    const blur = o.blur ?? 0;

    c.globalCompositeOperation = o.composite || "lighter";

    for (let i = 0; i < chars.length; i++) {
      const n = chars.length;
      // stagger window: last character finishes at progress = 1
      const t0 = (i / Math.max(n - 1, 1)) * stagger;
      const local = clamp((prog - t0) / Math.max(1 - stagger, 0.001));
      const e = ease(local, charEase);
      const a = o.fade ? e : (local > 0 ? 1 : 0);
      if (a > 0.002) {
        const dy = rise * (1 - e);
        const sc = o.pop ? lerp(o.pop, 1, e) : 1;

        if (blur > 0 && TYPE_FLAGS.filter) c.filter = `blur(${(blur * (1 - e)).toFixed(2)}px)`;
        if (glow > 0 && TYPE_FLAGS.glow) {
          c.shadowColor = css(glowCol, 1);
          c.shadowBlur = glow * (o.glowPulse ?? 1);
        } else {
          c.shadowBlur = 0;
        }

        if (split > 0) {
          c.fillStyle = css([o.col[0], 0, 0], a);
          c.fillText(chars[i], x + split, dy);
          c.fillStyle = css([0, o.col[1], o.col[2]], a);
          c.fillText(chars[i], x - split, dy);
          c.fillStyle = css([o.col[0] * 0.6, o.col[1] * 0.6, o.col[2] * 0.6], a);
          c.fillText(chars[i], x, dy);
        } else {
          c.fillStyle = css(o.col, a);
          if (sc !== 1) {
            c.save();
            const w = widths[i];
            c.translate(x + w / 2, dy);
            c.scale(sc, sc);
            c.fillText(chars[i], -w / 2, 0);
            c.restore();
          } else {
            c.fillText(chars[i], x, dy);
          }
        }
        c.shadowBlur = 0;
        c.filter = "none";
      }
      x += widths[i];
    }
    c.restore();
  }

  /** escape hatch: arbitrary vector art drawn into the same additive layer */
  custom(fn, { composite = "lighter", alpha = 1 } = {}) {
    const c = this.ctx;
    c.save();
    c.globalCompositeOperation = composite;
    c.globalAlpha = alpha;
    fn(c, this.W, this.H);
    c.restore();
  }

  /** a block of monospaced code with a fake syntax-highlight palette */
  code(o) {
    const c = this.ctx;
    const lines = o.lines;
    const size = o.size;
    const lh = o.lh ?? size * 1.32;
    c.save();
    c.font = fontStr("mono", size, o.weight ?? 500);
    c.textBaseline = "top";
    c.textAlign = "left";
    c.translate(o.x, o.y);
    if (o.rot) c.rotate(o.rot);
    c.globalCompositeOperation = "lighter";
    const prog = o.progress ?? 1;
    const total = lines.length;
    for (let i = 0; i < total; i++) {
      const lp = clamp((prog * (total + 2) - i) / 2);
      if (lp <= 0) continue;
      const a = (o.alpha ?? 1) * ease(lp, "outCubic");
      const y = i * lh;
      let x = 0;
      const toks = lines[i];
      for (const tk of toks) {
        c.font = fontStr("mono", size, tk.b ? 700 : (o.weight ?? 500));
        const col = tk.c ?? o.col;
        if (o.glow) { c.shadowColor = css(col, 1); c.shadowBlur = o.glow; }
        c.fillStyle = css(col, a);
        c.fillText(tk.t, x, y);
        c.shadowBlur = 0;
        x += c.measureText(tk.t).width;
      }
      if (o.cursor && i === Math.floor(prog * (total + 2)) - 1) {
        c.fillStyle = css(o.col, a * 0.9);
        c.fillRect(x + 4, y + 1, size * 0.52, size * 0.92);
      }
    }
    c.restore();
  }

  /** floating glyph columns - "personality data" rain */
  rain(o) {
    const c = this.ctx;
    const cols = o.cols;
    const size = o.size;
    const t = o.t;
    c.save();
    c.font = fontStr(o.font || "jp", size, o.weight ?? 500);
    c.textBaseline = "top";
    c.textAlign = "center";
    c.globalCompositeOperation = "lighter";
    const CH = o.chars;
    for (let i = 0; i < cols; i++) {
      const seed = o.seed + i;
      const x = o.x + (i + 0.5) * o.dx - o.dx / 2;
      const speed = o.speed * (0.55 + 0.9 * hash2(i, 11));
      const phase = hash2(i, 29);
      const len = Math.floor(lerp(o.minLen, o.maxLen, hash2(i, 43)));
      const span = this.H + size * 4;
      let head = ((t * speed + phase * span * 2) % (span + len * size * 1.05)) - len * size;
      const col = o.col;
      const headCol = o.headCol || [1, 1, 1];
      for (let k = 0; k < len; k++) {
        const y = head - k * size * 0.98;
        if (y < -size || y > this.H + size) continue;
        const idx = Math.floor(hash2(i * 131 + k, o.frameSalt || 0) * CH.length);
        const ch = CH[idx % CH.length];
        const fade = k === 0 ? 1 : Math.pow(1 - k / len, o.falloff ?? 1.0);
        const a = (o.alpha ?? 0.55) * fade;
        if (a < 0.01) continue;
        const cc = k === 0 ? headCol : col;
        c.fillStyle = css(cc, a * (0.7 + 0.3 * hash2(i, k * 7 + 5)));
        if (o.glow) { c.shadowColor = css(cc, 1); c.shadowBlur = o.glow; }
        c.fillText(ch, x, y);
        c.shadowBlur = 0;
      }
    }
    c.restore();
  }

  /** scattered glyph dust with parallax - the "data" ambience */
  dust(o) {
    const c = this.ctx;
    c.save();
    c.font = fontStr(o.font || "mono", o.size, o.weight ?? 400);
    c.textBaseline = "middle";
    c.textAlign = "center";
    c.globalCompositeOperation = "lighter";
    const CH = o.chars;
    const n = o.count;
    const t = o.t;
    const W = this.W, H = this.H;
    for (let i = 0; i < n; i++) {
      const depth = 0.25 + 0.75 * hash2(i, 3);
      const seed = o.seed + i;
      let x = (hash2(seed, 17) * (W + 240) - 120) + o.vx * depth * t;
      x = ((x % (W + 240)) + (W + 240)) % (W + 240) - 120;
      let y = ((hash2(seed, 31) * H + o.vy * depth * t) % H + H) % H;
      const tw = 0.5 + 0.5 * Math.sin(t * (0.6 + hash2(i, 7) * 1.6) + i);
      const a = (o.alpha ?? 0.4) * depth * (0.35 + 0.65 * tw);
      if (a < 0.01) continue;
      const idx = Math.floor(hash2(seed, 53) * CH.length);
      c.fillStyle = css(o.col, a);
      if (o.glow) { c.shadowColor = css(o.col, 1); c.shadowBlur = o.glow * depth; }
      c.fillText(CH[idx % CH.length], x + o.x, y + o.y);
      c.shadowBlur = 0;
    }
    c.restore();
  }
}

export const LATIN = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
export const CODE_CHARS = "{}();=<>+-*/[]#:.$%&|!?~^";
export const KATA = "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワンエヴ";
export const KATA2 = "ｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ";
export const ALLGLYPH = (KATA + LATIN.toLowerCase() + "0123456789" + CODE_CHARS).split("");
