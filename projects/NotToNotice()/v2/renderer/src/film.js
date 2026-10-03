// ---------------------------------------------------------------------------
// film.js - palettes, lyric table, default frame state, shared drawing helpers
// ---------------------------------------------------------------------------

import { clamp, lerp, smoothstep, ease, rgb, mixc, scaleC, hash2, hash3, TAU, fbm1, noise1 } from "./core.js";
import { mechFigure } from "./structure.js";

export const W = 1920, H = 1080;
export const CX = W / 2, CY = H / 2;

// --------------------------------------------------------------- palettes ---

const raw = {
  cold:   { nb: ["#01030a", "#05182c", "#0e4468"], key: "#63d6ff", alt: "#1a5680", hot: "#d6f6ff", grid: "#2ea9e0" },
  fairy:  { nb: ["#010616", "#072a55", "#1f7fd0"], key: "#4fd3ff", alt: "#1e6fc0", hot: "#eafcff", grid: "#39b6ff" },
  mission:{ nb: ["#01050f", "#0a1c33", "#16507e"], key: "#7fc4ff", alt: "#2b6ea8", hot: "#ffffff", grid: "#4f9ede" },
  query:  { nb: ["#04060c", "#111a26", "#2c3f52"], key: "#a8c4dc", alt: "#4a5c70", hot: "#ffffff", grid: "#7c93a8" },
  ifelse: { nb: ["#05040f", "#141033", "#3a1a5e"], key: "#43e6e6", alt: "#ff4f9a", hot: "#ffffff", grid: "#6a5fd8" },
  forge:  { nb: ["#0a0316", "#2a0b4a", "#6a1f9e"], key: "#b47cff", alt: "#ff5fd2", hot: "#ffe9ff", grid: "#8f4fd8" },
  tears:  { nb: ["#010610", "#0a2540", "#1d6b9e"], key: "#bde9ff", alt: "#4aa6e0", hot: "#ffffff", grid: "#5fc2f0" },
  drift:  { nb: ["#02040c", "#0a1a3c", "#1d4f8c"], key: "#7fb8ff", alt: "#2c6cb8", hot: "#ffffff", grid: "#3f7fd0" },
  redact: { nb: ["#0a0700", "#231700", "#5c3d05"], key: "#ffc637", alt: "#ff3b3b", hot: "#fff3c4", grid: "#c99400" },
  spring: { nb: ["#0d0410", "#3c0f2e", "#b03a6e"], key: "#ff9ec4", alt: "#ff5f9a", hot: "#fff0f6", grid: "#e0609a" },
  summer: { nb: ["#02100a", "#07361f", "#1d8a4e"], key: "#5ce08a", alt: "#2fae62", hot: "#e9fff2", grid: "#33c46e" },
  autumn: { nb: ["#100602", "#3a1503", "#a8480a"], key: "#ff9a3c", alt: "#e0621a", hot: "#fff0d8", grid: "#d0761e" },
  winter: { nb: ["#03060f", "#12203a", "#4a6a9c"], key: "#dfe9ff", alt: "#8aa8d8", hot: "#ffffff", grid: "#9fb8e0" },
  dream:  { nb: ["#04041a", "#171048", "#3d2f96"], key: "#9aa8ff", alt: "#5f6fe0", hot: "#ffffff", grid: "#6a72e0" },
  ascend: { nb: ["#02060f", "#0d2440", "#2f6fa8"], key: "#bcdcff", alt: "#4f8fd0", hot: "#ffffff", grid: "#5f9fdf" },
  eden:   { nb: ["#070400", "#2e1e05", "#8a6a10"], key: "#ffd25e", alt: "#6ec8ff", hot: "#fffbe8", grid: "#d0a83c" },
  doll:   { nb: ["#04050a", "#161a22", "#3c4552"], key: "#98a4b4", alt: "#e0223c", hot: "#ffffff", grid: "#5a6472" },
  light:  { nb: ["#02060f", "#0a2038", "#2a6a9e"], key: "#4fe6ff", alt: "#ff5a6e", hot: "#ffffff", grid: "#4fa8d8" },
  fight:  { nb: ["#0a0004", "#33000f", "#8a0a2a"], key: "#ff2d4a", alt: "#ffffff", hot: "#ffffff", grid: "#e0203c" },
  chorus: { nb: ["#010610", "#06284a", "#1a6aa8"], key: "#59c2ff", alt: "#a8e4ff", hot: "#ffffff", grid: "#3f9fd8" },
  overflow:{nb: ["#0a0202", "#3a0808", "#a02018"], key: "#ff3b30", alt: "#ffd166", hot: "#ffffff", grid: "#e03828" },
  source: { nb: ["#01060a", "#062018", "#0e4636"], key: "#7dff9a", alt: "#d8ffe0", hot: "#ffffff", grid: "#2fbf6a" },
  hanamaru:{nb: ["#070500", "#332302", "#a07a10"], key: "#ffcf3f", alt: "#ffa8d0", hot: "#ffffff", grid: "#e0b02c" },
  end:    { nb: ["#000000", "#02040a", "#0a1424"], key: "#cfe4ff", alt: "#7f9fd0", hot: "#ffffff", grid: "#4f6f9f" },
};

export const PAL = {};
for (const [k, v] of Object.entries(raw)) {
  PAL[k] = {
    c0: rgb(v.nb[0]), c1: rgb(v.nb[1]), c2: rgb(v.nb[2]),
    key: rgb(v.key), alt: rgb(v.alt), hot: rgb(v.hot), grid: rgb(v.grid),
    nb: v.nb, name: k,
  };
}

// ------------------------------------------------------------------ lyrics ---
// Timings from the official LRC.  NOTE: the published LRC has the Chinese
// translation offset by one line (its zh entry at time T actually belongs to
// the English line at the previous timestamp) - the pairing below is corrected.

export const LYRICS = [
  { t: 10.89, en: "Like Blue Fairy", zh: "如若蔚蓝色的妖姬啊" },
  { t: 13.61, en: "void Execute The Mission", zh: "推诿这无上的职责" },
  { t: 17.21, en: "the meaning of my existence", zh: "我存在的意义为何？" },
  { t: 23.84, en: "If EVE Become Human", zh: "于新夜化作人形" },
  { t: 30.66, en: "else suppressing my emotion", zh: "寒冰平息了" },
  { t: 33.18, en: "Create Soul For EVE", zh: "我于白夜躁动的创造之灵" },
  { t: 37.89, en: "I don't know why I can cry", zh: "我不知道，我为什么会哭泣" },
  { t: 44.36, en: "I don't know why I wish you happiness", zh: "我不知为何，要庇佑你幸福" },
  { t: 51.34, en: "I don't know why I can cry", zh: "我无从知晓啊，我为何要哭泣" },
  { t: 58.14, en: "Pretend Not To Notice", zh: "伪装作，毫不在意的样子" },
  { t: 87.62, en: "what do I dream of...?", zh: "我所向往之物为何？" },
  { t: 90.76, en: "void Achieve My Dream", zh: "缺乏逐梦的理想" },
  { t: 94.07, en: "until I get the fruit in the Eden", zh: "直到我取下，伊甸的禁果" },
  { t: 100.96, en: "while Be A Doll", zh: "哪怕做一具人偶也好" },
  { t: 104.47, en: "Focus On The Mission", zh: "坚守职责" },
  { t: 107.68, en: "cause you light my way of life", zh: "因为你照亮了我永生的路" },
  { t: 111.07, en: "Fight For You", zh: "誓死为你而战" },
  { t: 115.03, en: "I don't know why I can cry", zh: "我不知为何，我能够哭泣" },
  { t: 121.55, en: "I don't know why I want to be with you", zh: "我不知道为什么，我想伴你左右" },
  { t: 128.56, en: "I don't know why I can cry for you", zh: "我情不自禁地为你而恸哭" },
  { t: 135.73, en: "Pretend Not To Notice", zh: "却伪装作，毫不在意的模样" },
];

/** the lyric line that owns time t, and its local progress */
export function lyricAt(t) {
  let idx = -1;
  for (let i = 0; i < LYRICS.length; i++) if (t >= LYRICS[i].t) idx = i; else break;
  if (idx < 0) return { idx: -1, line: null, age: 0, hold: 0 };
  const line = LYRICS[idx];
  const next = LYRICS[idx + 1];
  const end = next ? next.t : 140.07;
  return {
    idx, line, end,
    age: t - line.t,
    hold: end - line.t,
    p: clamp((t - line.t) / Math.min(2.2, (end - line.t) * 0.5)),
  };
}

// ------------------------------------------------------------------- state ---

export function newState(t, frame) {
  return {
    t, frame, aspect: W / H,
    cam: { x: 0, y: 0, zoom: 1, rot: 0 },
    bg: {
      t, c0: [0, 0, 0], c1: [0, 0, 0], c2: [0, 0, 0],
      key: [1, 1, 1], alt: [1, 1, 1], hot: [1, 1, 1],
      scale: 1.6, warp: 1.1, flow: 0.35, contrast: 0.5, gain: 1.0, offx: 0, offy: 0,
      nebMix: 1.0, haze: 0.0, floor: -0.6,
      gridMode: 0, gridCol: [1, 1, 1], gridAmt: 0, gridScale: 1, gridDepth: 1,
      shaftAmt: 0, shaftCol: [1, 1, 1], shaftAng: 1.57, shaftWide: 0.5,
      glowAmt: 0, glowCol: [1, 1, 1], glowR: 0.6, glowX: 0, glowY: 0,
    },
    _pal: null,
    fx: {
      bloom: 0.85, bloomThresh: 0.42,
      chroma: 0, chromaR: 0, glitch: 0, glitchSeed: frame, rows: 42,
      scan: 0, grain: 0.030, vignette: 0.75, flash: 0, flashCol: [1, 1, 1],
      fade: 1, sat: 1.0, contrast: 1.0, lift: 0, liftCol: [0, 0, 0],
      smear: 0, smearY: 0.5, exposure: 1.0, tint: [1, 1, 1],
    },
    textActive: false, textGain: 1.0, textTint: [1, 1, 1],
    texts: [], jobs: [],
  };
}

export function setPal(st, key, k = 1) {
  const p = PAL[key] || PAL.cold;
  st.bg.c0 = scaleC(p.c0, k);
  st.bg.c1 = scaleC(p.c1, k);
  st.bg.c2 = scaleC(p.c2, k);
  st.bg.gridCol = p.grid;
  st.bg.shaftCol = p.alt;
  st.bg.glowCol = p.key;
  st.bg.key = p.key;
  st.bg.alt = p.alt;
  st.bg.hot = p.hot;
  st.bg.gridCol = p.grid;
  st._pal = p;
  return p;
}

// ---------------------------------------------------------------- helpers ----

/** upcoming/active accent hits within a time window, newest last */
export function recentHits(audio, t, win = 1.4) {
  const out = [];
  const hits = audio.hits;
  if (!hits || !hits.length) return out;
  // hits are sorted; binary search the start
  let lo = 0, hi = hits.length - 1, start = hits.length;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (hits[mid][0] >= t - win) { start = mid; hi = mid - 1; } else lo = mid + 1;
  }
  for (let i = start; i < hits.length; i++) {
    const [ht, hs] = hits[i];
    if (ht > t) break;
    out.push({ age: t - ht, s: hs, t: ht });
  }
  return out;
}

/** the classic "impact ring" burst on every accent */
export function accentRings(E, audio, t, o = {}) {
  const win = o.window ?? 0.9;
  const decay = o.decay ?? 2.7;
  const grow = o.grow ?? 640;
  const R0 = o.r0 ?? 30;
  const thr = o.thresh ?? 0.4;
  const col = o.col || [1, 1, 1];
  const alphaK = o.alpha ?? 0.5;
  for (const h of recentHits(audio, t, win)) {
    if (h.s < thr) continue;
    const k = Math.pow(clamp(1 - h.age / win), decay) * h.s;
    if (k < 0.01) continue;
    const r = R0 + h.age * grow;
    const a = k * alphaK;
    E.sprite(o.x ?? 0, o.y ?? 0, r, 0, col, a, 1, 0.10, 0.16 + 0.06 * h.s, 0);
    if (o.double) E.sprite(o.x ?? 0, o.y ?? 0, r * 0.62, 0, col, a * 0.6, 1, 0.06, 0.22, 0);
  }
}

/** radial spark burst helper */
export function sparks(E, x, y, n, seed, t, col, o = {}) {
  const r0 = o.r0 ?? 10, r1 = o.r1 ?? 300;
  const spd = o.speed ?? 260;
  for (let i = 0; i < n; i++) {
    const a = (i / n) * TAU + hash2(seed + i, 3) * 0.4;
    const s = 0.4 + 0.6 * hash2(seed + i, 7);
    const life = (t * s * spd) % r1;
    const rr = (o.inward ? (1 - life / r1) : (life / r1));
    const d = lerp(r0, r1, rr);
    E.sprite(x + Math.cos(a) * d, y + Math.sin(a) * d,
             o.size ?? 9, a, col, (1 - life / r1) * (o.alpha ?? 0.7) * s,
             o.shape ?? 0, 0.42, 0.1, 0.1);
  }
}

/** a stylised mechanical figure - see structure.js for the drawing itself */
export function girlSilhouette(c, x, y, h, o = {}) {
  mechFigure(c, x, y, h, o.t ?? 0, o.col || "#ffffff", {
    alpha: o.alpha ?? 1,
    lw: o.lw ?? 2.0,
    rot: o.rot,
    flare: o.flare,
    breathe: o.breathe,
    reach: o.reach,
  });
}

/** the five-petal "hanamaru" stamp as vector art (used for the end card) */
export function hanamaruPath(c, x, y, r, petals = 5, k = 0.34) {
  c.beginPath();
  for (let i = 0; i <= 360; i += 2) {
    const th = (i * Math.PI) / 180;
    const rr = r * (1 - k + k * Math.abs(Math.cos((petals / 2) * th)));
    const px = x + Math.cos(th) * rr, py = y + Math.sin(th) * rr;
    i === 0 ? c.moveTo(px, py) : c.lineTo(px, py);
  }
  c.closePath();
}

export function hexA(c, a) { return `rgba(${Math.round(c[0] * 255)},${Math.round(c[1] * 255)},${Math.round(c[2] * 255)},${a.toFixed(3)})`; }
