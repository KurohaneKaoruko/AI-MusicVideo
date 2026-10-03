// ---------------------------------------------------------------------------
// palette.ts - the colour score. each movement owns a key; transitions cross-
// fade on the CPU per frame so the shader just receives final colours.
// ---------------------------------------------------------------------------

import { rgb, mixc, type RGB } from "./math";

export type Pal = {
  c0: string; c1: string; c2: string;   // nebula: deep / mid / hot
  key: string; alt: string; hot: string; // accents: primary / secondary / highlight
};

export const PALS: Record<string, Pal> = {
  null:     { c0: "#010209", c1: "#040f1e", c2: "#0b2644", key: "#6fc8ff", alt: "#23486e", hot: "#eaffff" },
  boot:     { c0: "#010208", c1: "#04101f", c2: "#0c2a4c", key: "#63d6ff", alt: "#2a5f8a", hot: "#f2ffff" },
  fairy:    { c0: "#010616", c1: "#062a52", c2: "#1470c4", key: "#55d6ff", alt: "#1e6fc0", hot: "#f0fdff" },
  query:    { c0: "#04060a", c1: "#0f1822", c2: "#2c4055", key: "#a8c4dc", alt: "#4a5c70", hot: "#ffffff" },
  forge:    { c0: "#070214", c1: "#260a40", c2: "#5e1c88", key: "#b47cff", alt: "#ff5fd2", hot: "#ffeaff" },
  admit:    { c0: "#010510", c1: "#072040", c2: "#155082", key: "#9ad6ff", alt: "#3f7fb8", hot: "#ffffff" },
  redact:   { c0: "#060300", c1: "#201402", c2: "#5c420b", key: "#ffc637", alt: "#d08a1a", hot: "#fff7dc" },
  spring:   { c0: "#0d0410", c1: "#3c0f2e", c2: "#b03a6e", key: "#ff9ec4", alt: "#ff5f9a", hot: "#fff0f6" },
  summer:   { c0: "#02100a", c1: "#073620", c2: "#1d8a4e", key: "#5ce08a", alt: "#2fae62", hot: "#e9fff2" },
  autumn:   { c0: "#100602", c1: "#3a1503", c2: "#a8480a", key: "#ff9a3c", alt: "#e0621a", hot: "#fff0d8" },
  winter:   { c0: "#03060f", c1: "#12203c", c2: "#4a6aa0", key: "#dfe9ff", alt: "#8aa8d8", hot: "#ffffff" },
  ascent:   { c0: "#050314", c1: "#1c1048", c2: "#5c3aa8", key: "#b9a4ff", alt: "#6ec8ff", hot: "#ffffff" },
  eden:     { c0: "#070400", c1: "#2e1e05", c2: "#8a6a10", key: "#ffd25e", alt: "#6ec8ff", hot: "#fffbe8" },
  doll:     { c0: "#030407", c1: "#14181f", c2: "#3c4652", key: "#a8b2c0", alt: "#ff2d46", hot: "#ffffff" },
  fight:    { c0: "#070001", c1: "#2c020c", c2: "#7c0a22", key: "#ff3050", alt: "#ffd9e0", hot: "#ffffff" },
  tearfall: { c0: "#010510", c1: "#062242", c2: "#0e4e80", key: "#5fc4ff", alt: "#a8e4ff", hot: "#ffffff" },
  overflow: { c0: "#0a0202", c1: "#3a0808", c2: "#a02018", key: "#ff3b30", alt: "#ffd166", hot: "#ffffff" },
  break:    { c0: "#050203", c1: "#210609", c2: "#701418", key: "#ff5a5a", alt: "#ffe8e8", hot: "#ffffff" },
  source:   { c0: "#010704", c1: "#052318", c2: "#0e5238", key: "#64ff9e", alt: "#b8ffd4", hot: "#ffffff" },
  verdict:  { c0: "#050300", c1: "#2a1c04", c2: "#8c6414", key: "#ffd25e", alt: "#ffb0d8", hot: "#fffbe8" },
  end:      { c0: "#000000", c1: "#010204", c2: "#060b12", key: "#9fb8d8", alt: "#3a4a66", hot: "#ffffff" },
};

type PalLive = {
  c0: RGB; c1: RGB; c2: RGB; key: RGB; alt: RGB; hot: RGB;
  keyHex: string; altHex: string; hotHex: string;
  name: string;
};

const cache = new Map<string, PalLive>();
const live = (name: string): PalLive => {
  let p = cache.get(name);
  if (!p) {
    const r = PALS[name];
    p = {
      c0: rgb(r.c0), c1: rgb(r.c1), c2: rgb(r.c2),
      key: rgb(r.key), alt: rgb(r.alt), hot: rgb(r.hot),
      keyHex: r.key, altHex: r.alt, hotHex: r.hot, name,
    };
    cache.set(name, p);
  }
  return p;
};

/** cross-fade two palettes; t=0 -> a, t=1 -> b */
export const mixPal = (a: string, b: string, t: number): PalLive => {
  const A = live(a);
  const B = live(b);
  if (t <= 0) return A;
  if (t >= 1) return B;
  return {
    c0: mixc(A.c0, B.c0, t), c1: mixc(A.c1, B.c1, t), c2: mixc(A.c2, B.c2, t),
    key: mixc(A.key, B.key, t), alt: mixc(A.alt, B.alt, t), hot: mixc(A.hot, B.hot, t),
    keyHex: t < 0.5 ? A.keyHex : B.keyHex,
    altHex: t < 0.5 ? A.altHex : B.altHex,
    hotHex: t < 0.5 ? A.hotHex : B.hotHex,
    name: t < 0.5 ? A.name : B.name,
  };
};

export { live as pal, type PalLive };
