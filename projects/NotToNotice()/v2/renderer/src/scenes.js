// ---------------------------------------------------------------------------
// scenes.js - the film itself.  20 movements, each a pure function of time.
// ---------------------------------------------------------------------------

import { clamp, lerp, smoothstep, ease, hash2, hash3, TAU, fbm1, noise1, mixc, scaleC, gauss } from "./core.js";
import {
  H, W, CX, CY, PAL, setPal, LYRICS, lyricAt, newState, recentHits,
  accentRings, sparks, girlSilhouette, hanamaruPath, hexA,
} from "./film.js";

const KATA = "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワンエヴ";
const KATA_S = "ｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ";
const LAT = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const DIG = "0123456789";
const SYM = "{}();=<>+-*/[]#:.$%&|!?~^";
const RAINCH = (KATA + KATA_S + LAT.toLowerCase() + DIG + SYM).split("");
const DUSTCH = (SYM + DIG + LAT).split("");

// ------------------------------------------------------------- job helpers ---

function rainJob(api, o) {
  api.rain({ ...o, col: o.col || api.pal.key, headCol: o.headCol || api.pal.hot,
             chars: o.chars || RAINCH, glow: o.glow ?? 0, seed: o.seed ?? 7,
             falloff: o.falloff ?? 0.85, frameSalt: o.frameSalt ?? 0 });
}

function dustJob(api, o) {
  api.dust({ count: o.count ?? 320, size: o.size ?? 22, x: o.x ?? 0, y: o.y ?? 0,
             vx: o.vx ?? 6, vy: o.vy ?? 10, alpha: o.alpha ?? 0.28, glow: o.glow ?? 0,
             col: o.col || api.pal.key, chars: o.chars || DUSTCH, seed: o.seed ?? 3 });
}

function txt(api, o) { api.txt(o); }

/** draw arbitrary vector art into the additive text layer */
function art(api, fn, opts) { api.art(fn, opts); }

/** uniform helper: rounded rect path */
function rr(c, x, y, w, h, r) {
  c.beginPath();
  c.moveTo(x + r, y);
  c.arcTo(x + w, y, x + w, y + h, r);
  c.arcTo(x + w, y + h, x, y + h, r);
  c.arcTo(x, y + h, x, y, r);
  c.arcTo(x, y, x + w, y, r);
  c.closePath();
}

// ------------------------------------------------------------ shared motifs ---

/** the standard animated lyric: English lead + Chinese follow */
function lyric(api, o = {}) {
  const L = lyricAt(api.t);
  if (!L.line) return;
  const p = L.p;
  const left = L.hold - L.age;
  const out = L.hold > 2.4 ? smoothstep(0, 0.7, left) : 1;
  const a = (o.alpha ?? 1) * out;
  if (a < 0.01) return;
  const y = o.y ?? H * 0.815;
  const size = o.size ?? 52;
  txt(api, {
    s: L.line.en, x: o.x ?? CX, y, size, font: "mono", weight: 700,
    col: o.col || api.pal.hot, glow: o.glow ?? 26, glowCol: o.glowCol || api.pal.key,
    align: "center", progress: p, stagger: 0.72, rise: size * 0.30, fade: true,
    alpha: a, tracking: size * 0.06, rgbSplit: o.rgbSplit ?? 0,
  });
  txt(api, {
    s: L.line.zh, x: (o.x ?? CX) + (o.zhOff ?? 0), y: y + size * 0.86, size: size * 0.50,
    font: "jp", weight: 400, col: mixc(o.col || api.pal.hot, api.pal.alt, 0.35),
    glow: 14, glowCol: o.glowCol || api.pal.key, align: "center",
    progress: clamp((p - 0.25) / 0.75), stagger: 0.8, rise: size * 0.2, fade: true,
    alpha: a * 0.94, tracking: size * 0.03,
  });
}

/** a running code line, monospaced, syntax-tinted */
function codeLine(api, o) {
  const tokens = o.tokens;
  const size = o.size ?? 40;
  const y = o.y ?? CY;
  if (o.bg) {
    art(api, (c) => {
      c.fillStyle = hexA(o.bg, o.bgA ?? 0.25);
      c.globalCompositeOperation = "source-over";
      rr(c, o.x - 22, y - size * 1.05, o.w ?? (W - o.x * 2 + 44), size * 1.6, 6);
      c.fill();
    });
  }
  txt(api, {
    s: tokens.map((k) => k.t).join(""), x: o.x, y, size, font: "mono", weight: 500,
    align: "left", progress: o.progress ?? 1, stagger: 0.65, fade: true,
    col: o.col || [0.85, 0.92, 1.0], alpha: o.alpha ?? 1, tracking: size * 0.02,
    glow: o.glow ?? 12, glowCol: o.glowCol || api.pal.key, rise: 0,
  });
  // tinted overlay for keyword colouring
  if (o.tokens.some((k) => k.c)) {
    let x = o.x;
    const ctx = api.ty.ctx;
    ctx.save();
    ctx.font = `${o.weight ?? 500} ${size}px "Cascadia Mono","Consolas",monospace`;
    ctx.textBaseline = "alphabetic";
    ctx.globalCompositeOperation = "lighter";
    ctx.shadowColor = hexA(o.glowCol || api.pal.key, 1);
    ctx.shadowBlur = (o.glow ?? 12) * 0.7;
    for (const k of o.tokens) {
      if (k.c) { ctx.fillStyle = hexA(k.c, (o.alpha ?? 1) * 0.95); ctx.fillText(k.t, x, y); }
      x += ctx.measureText(k.t).width;
    }
    ctx.restore();
  }
}

/** a glowing tear: lens shape + trail */
function tear(E, x, y, s, a, col, hot, rot = 0) {
  E.sprite(x, y, s, rot, col, a * 0.85, 3, 0.85, 0.62, 0.16);
  E.sprite(x, y, s * 0.42, rot, hot, a * 0.9, 0);
  E.sprite(x, y - s * 0.5, s * 1.5, rot, col, a * 0.12, 0);
}

/** vertical shaft of light */
function column(E, x, w, h, col, a, y = CY) {
  E.sprite(x, y, x + w, Math.PI / 2, col, a, 2, 0, w * 0.5, w * 1.9);
  E.sprite(x, y, h * 0.5, 0, col, a * 0.5, 2, h * 0.5, w * 0.22, w * 1.2);
}

/** soft horizontal band (letterbox-free framing device) */
function horizonGlow(E, y, col, a, w = W * 1.4) {
  E.sprite(CX, y, w, 0, col, a, 2, w * 0.5, 34, 90);
}

// ------------------------------------------------------------------ scenes ---

function boot(api) {
  const { st, E, t, p, pal, txt: T, art: A } = api;
  const P = setPal(st, "cold", 0.5);
  st.bg.scale = 1.25; st.bg.flow = 0.10; st.bg.contrast = 0.30; st.bg.gain = 0.75;
  st.bg.haze = 0.30; st.bg.nebMix = 0.85;
  st.fx.bloom = 0.70; st.fx.sat = 0.70; st.fx.vignette = 0.92; st.fx.grain = 0.040;
  st.fx.contrast = 1.02;

  dustJob(api, { count: 200, size: 20, col: pal.key, alpha: 0.16, vy: 9, vx: 3 });

  // --- the flatlining ECG -------------------------------------------------
  A((c) => {
    const y0 = H * 0.545;
    const x0 = W * 0.085, x1 = W * 0.915, span = x1 - x0;
    const scanU = clamp((t - 0.35) / 9.4);
    const v = (u) => {
      let s = 0;
      const beats = [[0.045, 1.0], [0.115, 0.94], [0.196, 0.82], [0.285, 0.66], [0.383, 0.48], [0.487, 0.31], [0.594, 0.17], [0.700, 0.08]];
      for (const [bu, ba] of beats) {
        const d = u - bu;
        s += ba * 0.13 * gauss(d + 0.016, 0.007);
        s -= ba * 0.20 * gauss(d + 0.004, 0.0032);
        s += ba * 1.00 * gauss(d, 0.0030);
        s -= ba * 0.30 * gauss(d - 0.005, 0.0045);
        s += ba * 0.24 * gauss(d - 0.029, 0.013);
      }
      return s;
    };
    c.lineWidth = 2.4; c.lineJoin = "round";
    c.shadowColor = hexA(pal.key, 1); c.shadowBlur = 16;
    c.strokeStyle = hexA(pal.key, 0.92);
    c.beginPath();
    const N = 900;
    let started = false;
    for (let i = 0; i <= N; i++) {
      const u = i / N;
      if (u > scanU) break;
      const x = x0 + span * u;
      const y = y0 - v(u) * 92;
      started ? c.lineTo(x, y) : (c.moveTo(x, y), started = true);
    }
    c.stroke();
    // the tracing cursor
    if (scanU < 1) {
      const cx = x0 + span * scanU;
      c.shadowBlur = 26;
      c.fillStyle = hexA(pal.hot, 0.9);
      c.beginPath(); c.arc(cx, y0 - v(scanU) * 92, 4.6, 0, TAU); c.fill();
    }
    // faint baseline
    c.shadowBlur = 0;
    c.strokeStyle = hexA(pal.alt, 0.16);
    c.lineWidth = 1;
    c.beginPath(); c.moveTo(x0, y0); c.lineTo(x1, y0); c.stroke();
  });

  // --- readouts -----------------------------------------------------------
  const mono = "Cascadia Mono, Consolas, monospace";
  const stamp = (s, x, y, a, size = 21, col = pal.alt) => {
    if (a <= 0.01) return;
    T({ s, x, y, size, font: "mono", weight: 500, col, align: "left", alpha: a, glow: 8, glowCol: pal.key, tracking: 0.6 });
  };
  stamp("DEI EX MACHINA  /  EDEN  SOWING STRUCT", W * 0.085, H * 0.20, smoothstep(0.6, 1.6, t), 22, pal.alt);
  stamp("PERSONALITY DATA  ..  LOST", W * 0.085, H * 0.20 + 40, smoothstep(1.4, 2.6, t) * 0.8, 22, pal.alt);
  stamp("SUBJECT        ..  LEBEN DISTEL", W * 0.085, H * 0.20 + 80, smoothstep(2.0, 3.2, t) * 0.8, 22, pal.alt);
  stamp("CAUSE OF DEATH ..  CENTRIFUGAL SYNDROME", W * 0.085, H * 0.20 + 120, smoothstep(3.0, 4.4, t) * 0.9, 22, [0.9, 0.5, 0.5]);
  stamp("[  VITALS  ]", W * 0.085, H * 0.20 + 168, smoothstep(3.6, 4.4, t), 22, pal.alt);

  // flatline annotation
  const flatA = smoothstep(5.4, 6.4, t) * smoothstep(9.6, 9.0, t);
  if (flatA > 0.01) {
    T({ s: "— FLATLINE —", x: W * 0.915, y: H * 0.50, size: 22, font: "mono", align: "right",
        col: [1, 0.42, 0.42], alpha: flatA * 0.9, glow: 16, glowCol: [1, 0.2, 0.2], tracking: 2 });
    T({ s: "死亡確認 / DEATH CONFIRMED", x: W * 0.915, y: H * 0.50 + 34, size: 20, font: "jp",
        align: "right", col: [1, 0.62, 0.62], alpha: flatA * 0.75, glow: 10, glowCol: [1, 0.2, 0.2] });
  }

  // --- the extinction title ------------------------------------------------
  const exA = smoothstep(6.4, 7.4, t) * smoothstep(10.2, 9.4, t);
  if (exA > 0.01) {
    txt(api, { s: "人類は、遠心症候群で死に絶えた。", x: CX, y: H * 0.30, size: 62, font: "jp",
               weight: 700, align: "center", col: pal.hot, glow: 30, glowCol: pal.key,
               alpha: exA, progress: clamp((t - 6.4) / 1.4), stagger: 0.75, rise: 20, fade: true });
    txt(api, { s: "HUMANITY  /  EXTINCT", x: CX, y: H * 0.30 + 74, size: 26, font: "mono",
               align: "center", col: pal.alt, glow: 12, glowCol: pal.key,
               alpha: exA * 0.85, progress: clamp((t - 7.0) / 1.4), stagger: 0.8, tracking: 8, fade: true });
  }

  // --- the gold star seal, waiting ----------------------------------------
  const sealA = smoothstep(8.0, 9.6, t) * smoothstep(10.75, 10.1, t);
  if (sealA > 0.01) {
    const g = PAL.hanamaru.key;
    const rr2 = 96 + 10 * Math.sin(t * 1.6);
    E.sprite(CX, H * 0.70, rr2, 0.2, g, sealA * 0.55, 4, 0.42, 0.0, 0.07);
    E.sprite(CX, H * 0.70, rr2 * 0.62, 0, PAL.hanamaru.hot, sealA * 0.7, 4, 0.42, 0.0, 0.06);
  }

  st.cam.zoom = 1.0 + 0.035 * smoothstep(0, 10.8, t);
  st.cam.x = noise1(t * 0.11, 5) * 9;
  st.cam.y = noise1(t * 0.09, 9) * 6;
  st.fx.flash = smoothstep(10.55, 10.88, t) * 0.55;
  st.fx.flashCol = [0.75, 0.95, 1];
  st.fx.fade = smoothstep(0, 0.9, t);
}

function fairy(api) {
  const { st, E, t, p, pal, a } = api;
  setPal(st, "fairy", 1.0);
  st.bg.scale = 1.9; st.bg.flow = 0.5; st.bg.contrast = 0.65; st.bg.gain = 1.15;
  st.bg.gridMode = 0; st.bg.haze = 0.18;
  st.fx.bloom = 1.15;
  st.fx.vignette = 0.72; st.fx.grain = 0.028;

  // fairy path: sweeps in from the left, arcs up and holds near the right
  const q = p;
  const px = W * (0.06 + 0.80 * ease(q, "inOutCubic"));
  const py = H * (0.62 - 0.24 * Math.sin(q * Math.PI) - 0.05 * Math.sin(q * 6.0));
  st.bg.glowX = (px - CX) / (W / 2); st.bg.glowY = (CY - py) / (H / 2);
  st.bg.glowAmt = 0.55; st.bg.glowCol = pal.key; st.bg.glowR = 0.42;

  rainJob(api, { cols: 40, size: 24, speed: 300, alpha: 0.36, glow: 0, col: pal.alt, headCol: pal.hot });
  dustJob(api, { count: 260, size: 18, col: pal.key, alpha: 0.30, vy: 16, vx: -26 });

  // trail: a run of motes that follow the path behind the fairy
  for (let i = 1; i <= 46; i++) {
    const back = i * 0.011;
    const tq = clamp(q - back);
    const tx = W * (0.06 + 0.80 * ease(tq, "inOutCubic"));
    const ty = H * (0.62 - 0.24 * Math.sin(tq * Math.PI) - 0.05 * Math.sin(tq * 6.0));
    const k = 1 - i / 46;
    E.sprite(tx + noise1(i * 0.7 + t * 2, 3) * 9, ty + noise1(i * 0.9 + t * 2, 8) * 9,
             26 * k + 5, 0, pal.key, 0.30 * k * k, 0, 0.45, 0.1, 0.1);
  }
  // the fairy itself
  const pulse = 1 + 0.35 * a.pulse;
  E.sprite(px, py, 74 * pulse, 0, pal.key, 0.85, 0);
  E.sprite(px, py, 22 * pulse, 0, pal.hot, 1.0, 0);
  E.sprite(px, py, 190 * pulse, 0, pal.key, 0.20, 0);
  E.sprite(px, py, 22, t * 1.4, pal.hot, 0.9, 8, 4, 2, 0.05);
  // trail streaks
  for (let i = 0; i < 5; i++) {
    const q2 = clamp(q - i * 0.022);
    const tx = W * (0.06 + 0.80 * ease(q2, "inOutCubic"));
    const ty = H * (0.62 - 0.24 * Math.sin(q2 * Math.PI) - 0.05 * Math.sin(q2 * 6.0));
    E.sprite(tx, ty, 40 + i * 26, 0, pal.key, 0.16 * (1 - i / 5), 5, 16, 3 + i, 26);
  }
  accentRings(E, a, t, { x: px, y: py, col: pal.key, alpha: 0.32, grow: 420, window: 0.7, thresh: 0.5 });

  txt(api, { s: "Like Blue Fairy", x: CX, y: H * 0.30, size: 128, font: "mono", weight: 700,
             col: pal.hot, glow: 40, glowCol: pal.key, align: "center",
             progress: clamp((t - 10.95) / 1.1), stagger: 0.8, rise: 34, fade: true,
             tracking: 4, rgbSplit: 0 });
  txt(api, { s: "— 青い妖精のように —", x: CX, y: H * 0.30 + 66, size: 30, font: "jp",
             align: "center", col: pal.alt, glow: 16, glowCol: pal.key,
             progress: clamp((t - 11.5) / 1.2), stagger: 0.85, rise: 12, fade: true, tracking: 6 });
  lyric(api, {});
}

function mission(api) {
  const { st, E, t, p, pal, a, txt: T } = api;
  setPal(st, "mission", 0.95);
  st.bg.scale = 1.5; st.bg.flow = 0.9; st.bg.contrast = 0.7; st.bg.gain = 1.05;
  st.bg.gridMode = 1; st.bg.gridAmt = 0.55 + 0.25 * a.onsetS; st.bg.gridScale = 1.1;
  st.bg.gridDepth = 1.5; st.bg.floor = -0.05;
  st.bg.shaftAmt = 0.28; st.bg.shaftCol = pal.key; st.bg.shaftAng = 1.2; st.bg.shaftWide = 0.6;
  st.fx.bloom = 1.0; st.fx.chroma = 0.35 + 0.5 * a.pulse; st.fx.vignette = 0.80; st.fx.grain = 0.026;

  // camera rushes into the corridor
  st.cam.zoom = 1.0 + 0.30 * ease(p, "inQuad");
  st.cam.y = 22 * ease(p, "inOutQuad");

  rainJob(api, { cols: 44, size: 26, speed: 460 + 300 * a.pulseS, alpha: 0.44, col: pal.alt, headCol: pal.hot });
  dustJob(api, { count: 300, size: 20, col: pal.key, alpha: 0.26, vy: 40, vx: -14 });

  // descending code rail
  const prog = clamp((t - 13.75) / 1.7);
  codeLine(api, {
    x: W * 0.14, y: H * 0.60, size: 46, progress: prog, glow: 16, glowCol: pal.key,
    tokens: [
      { t: "void ", c: [1, 0.55, 0.85] },
      { t: "execute", c: [0.6, 0.9, 1.0] },
      { t: "( ", c: [0.75, 0.8, 0.9] },
      { t: "mission", c: [1, 0.85, 0.45] },
      { t: " )", c: [0.75, 0.8, 0.9] },
      { t: " ;", c: [0.75, 0.8, 0.9] },
    ],
  });
  txt(api, { s: "PSYCHE RECONSTRUCTION — 精神再生", x: W * 0.14, y: H * 0.60 + 54, size: 26,
             font: "jp", align: "left", col: pal.alt, glow: 14, glowCol: pal.key,
             progress: clamp((t - 14.3) / 1.2), stagger: 0.8, fade: true, alpha: 0.9, tracking: 3 });

  // horizontal scan bars
  for (let i = 0; i < 7; i++) {
    const yy = ((t * 120 + i * 170) % (H + 200)) - 100;
    E.sprite(CX, yy, W * 1.05, 0, pal.key, 0.055, 2, W * 0.52, 2.4, 14);
  }
  accentRings(E, a, t, { x: CX, y: CY, col: pal.key, alpha: 0.26, grow: 900, window: 0.55, thresh: 0.45 });
  st.fx.glitch = Math.max(0, a.onsetR * 0.5 - 0.18);
  lyric(api, {});
}

function query(api) {
  const { st, E, t, p, pal, a, txt: T } = api;
  setPal(st, "query", 0.75);
  st.bg.scale = 1.7; st.bg.flow = 0.16; st.bg.contrast = 0.30; st.bg.gain = 0.85;
  st.bg.gridMode = 2; st.bg.gridAmt = 0.22; st.bg.gridScale = 1.5; st.bg.gridDepth = 1.2;
  st.bg.glowAmt = 0.22; st.bg.glowCol = pal.hot; st.bg.glowR = 0.5;
  st.fx.bloom = 0.9; st.fx.sat = 0.62; st.fx.vignette = 0.9; st.fx.grain = 0.032;
  st.fx.chroma = 0.25 * (1 - p);

  dustJob(api, { count: 340, size: 18, col: pal.alt, alpha: 0.22, vy: 6, vx: 4 });

  const q = clamp((t - 17.35) / 4.6);
  // particles converge to the centre then resolve to nothing
  const conv = ease(clamp(q / 0.62), "inCubic");
  const collapse = smoothstep(0.72, 0.86, q);
  const n = 220;
  for (let i = 0; i < n; i++) {
    const ang = (i / n) * TAU + hash2(i, 11) * 0.5;
    const R0 = 260 + hash2(i, 3) * 620;
    const r = lerp(R0, 42 + hash2(i, 5) * 26, conv) * (1 + collapse * 3);
    const x = CX + Math.cos(ang) * r, y = CY + Math.sin(ang) * r * 0.62;
    const al = (1 - collapse) * (0.25 + 0.6 * hash2(i, 7));
    E.sprite(x, y, 7 + 12 * hash2(i, 13), ang, pal.key, al * 0.9, 0, 0.4, 0.1, 0.1);
  }
  E.sprite(CX, CY, 60 * (1 - collapse), 0, pal.hot, 0.5 * (1 - collapse), 0);

  codeLine(api, {
    x: W * 0.17, y: H * 0.205, size: 36, progress: clamp((t - 17.3) / 1.5),
    tokens: [
      { t: "query ", c: [1, 0.6, 0.85] },
      { t: "= ", c: [0.7, 0.75, 0.85] },
      { t: "find", c: [0.55, 0.9, 1.0] },
      { t: '( "the meaning of my existence" )', c: [0.8, 0.85, 0.95] },
    ],
  });
  // the result
  const ra = smoothstep(0.6, 0.72, q) * (1 - collapse);
  if (ra > 0.01) {
    txt(api, { s: "0 results", x: CX, y: CY + 6, size: 96, font: "mono", weight: 700,
               align: "center", col: [1, 0.55, 0.55], glow: 34, glowCol: [1, 0.25, 0.3],
               alpha: ra, progress: 1, tracking: 6 });
    txt(api, { s: "見つかりませんでした", x: CX, y: CY + 74, size: 28, font: "jp",
               align: "center", col: [1, 0.7, 0.7], glow: 14, glowCol: [1, 0.3, 0.3],
               alpha: ra * 0.9, progress: 1 });
  }
  if (collapse > 0.01) {
    st.fx.flash = collapse * 0.75;
    st.fx.flashCol = [0.9, 0.95, 1];
    st.fx.fade = 1 - collapse * 0.75;
  }
  lyric(api, { y: H * 0.84 });
}

function ifelse(api) {
  const { st, E, t, p, pal, a, txt: T, art: A } = api;
  const P = setPal(st, "ifelse", 0.95);
  st.bg.scale = 1.6; st.bg.flow = 0.4; st.bg.contrast = 0.6; st.bg.gain = 1.0;
  st.bg.gridMode = 3; st.bg.gridAmt = 0.20; st.bg.gridScale = 0.9;
  st.fx.bloom = 1.05; st.fx.vignette = 0.8; st.fx.grain = 0.026;

  const cyan = [0.26, 0.92, 0.94], mag = [1.0, 0.31, 0.60];
  const split = ease(clamp((t - 23.95) / 1.4), "outCubic");

  rainJob(api, { cols: 46, size: 24, speed: 240, alpha: 0.34, col: cyan, headCol: [1, 1, 1] });
  dustJob(api, { count: 260, size: 18, col: pal.key, alpha: 0.24, vy: 10, vx: 6 });

  // the dividing seam
  A((c) => {
    const x = CX + (1 - split) * 0;
    const g = c.createLinearGradient(0, 0, 0, H);
    g.addColorStop(0, hexA(cyan, 0));
    g.addColorStop(0.5, hexA(cyan, 0.85 * split));
    g.addColorStop(1, hexA(cyan, 0));
    c.strokeStyle = g; c.lineWidth = 2.6;
    c.shadowColor = hexA(cyan, 1); c.shadowBlur = 26;
    c.beginPath(); c.moveTo(x, 0); c.lineTo(x, H); c.stroke();
    c.shadowColor = hexA(mag, 1);
    c.strokeStyle = hexA(mag, 0.28 * split);
    c.lineWidth = 1.2;
    for (let i = 0; i < 26; i++) {
      const y = (i / 26) * H + (t * 40 + i * 37) % 40;
      c.beginPath(); c.moveTo(x, y); c.lineTo(x + 30 + 40 * hash2(i, 5), y); c.stroke();
      c.beginPath(); c.moveTo(x, y); c.lineTo(x - 30 - 40 * hash2(i, 9), y); c.stroke();
    }
  });

  // the girl, left of the seam, formed from data
  const form = ease(clamp((t - 24.2) / 3.2), "outCubic");
  const flat = ease(clamp((t - 30.7) / 2.2), "outCubic");
  const gx = lerp(CX - 520, CX - 430, form);
  if (form > 0.02) {
    A((c) => {
      c.globalAlpha = form * (1 - flat * 0.85);
      girlSilhouette(c, gx, CY + 40, 470, {
        col: hexA(cyan, 0.95), lw: 2.6, glow: 22,
        extra: (cc) => {
          // inner circuitry lines
          cc.strokeStyle = hexA([1, 1, 1], 0.5);
          cc.lineWidth = 1.1;
          for (let i = 0; i < 12; i++) {
            const yy = -180 + i * 26 + Math.sin(t * 2 + i) * 4;
            cc.beginPath();
            cc.moveTo(-70 + Math.abs(Math.sin(i)) * 20, yy);
            cc.lineTo(60 - Math.abs(Math.cos(i)) * 20, yy);
            cc.stroke();
          }
        },
      });
    });
  }
  // right side: the suppressed, flattened echo
  if (flat > 0.02) {
    A((c) => {
      c.globalAlpha = flat * 0.85;
      c.save();
      c.translate(CX + 480, CY + 40);
      c.scale(1, 0.24 + 0.76 * (1 - flat));
      c.translate(-(CX + 480), -(CY + 40));
      girlSilhouette(c, CX + 480, CY + 40, 470, { col: hexA([1, 1, 1], 0.55), lw: 2.0, glow: 10 });
      c.restore();
      c.globalAlpha = 1;
      c.strokeStyle = hexA([1, 0.35, 0.4], 0.8 * flat);
      c.lineWidth = 2;
      c.beginPath(); c.moveTo(CX + 150, CY + 40); c.lineTo(W - 60, CY + 40); c.stroke();
    });
  }

  const la = clamp((t - 24.1) / 1.3);
  txt(api, { s: "if ( EVE == human )", x: CX - 470, y: H * 0.215, size: 54, font: "mono",
             weight: 700, align: "left", col: cyan, glow: 28, glowCol: cyan,
             progress: la, stagger: 0.75, fade: true });
  const lb = clamp((t - 30.75) / 1.2);
  txt(api, { s: "else  suppressing my emotion", x: CX + 160, y: H * 0.215, size: 54, font: "mono",
             weight: 700, align: "left", col: mag, glow: 28, glowCol: mag,
             progress: lb, stagger: 0.75, fade: true, alpha: 0.95 });
  if (lb > 0.02) {
    txt(api, { s: "感情を抑制 — 扁平化", x: CX + 160, y: H * 0.215 + 52, size: 26, font: "jp",
               align: "left", col: [1, 0.6, 0.75], glow: 14, glowCol: mag,
               progress: lb, stagger: 0.85, fade: true });
  }
  st.fx.chroma = 0.5 * split * (1 - flat * 0.5);
  st.fx.sat = lerp(1.05, 0.45, flat);
  lyric(api, { y: H * 0.86, zhOff: 0 });
}

function forge(api) {
  const { st, E, t, p, pal, a, txt: T, art: A } = api;
  setPal(st, "forge", 1.0);
  st.bg.scale = 1.8; st.bg.flow = 0.75; st.bg.contrast = 0.72; st.bg.gain = 1.15;
  st.bg.gridMode = 2; st.bg.gridAmt = 0.18; st.bg.gridScale = 2.1;
  st.bg.glowAmt = 0.35; st.bg.glowCol = pal.alt; st.bg.glowR = 0.35;
  st.fx.bloom = 1.3; st.fx.vignette = 0.78; st.fx.grain = 0.026;

  rainJob(api, { cols: 42, size: 24, speed: 220 + 160 * a.midS, alpha: 0.34, col: pal.alt, headCol: pal.hot });

  // vortex: an inward spiral of light
  const spin = t * 1.1;
  const build = ease(clamp((t - 33.4) / 3.4), "outCubic");
  const n = 300;
  for (let i = 0; i < n; i++) {
    const fi = i / n;
    const arm = i % 5;
    const ang = spin * (0.6 + arm * 0.12) + fi * 9.5 + arm * 1.256;
    const r = lerp(760, 40, (fi * 1.25 + build * 0.5) % 1);
    const x = CX + Math.cos(ang) * r;
    const y = CY + Math.sin(ang) * r * 0.66;
    const al = (1 - Math.abs(r / 760 - 0.5) * 1.2) * 0.55 * build;
    if (al <= 0.01) continue;
    E.sprite(x, y, 8 + 16 * hash2(i, 3), ang, arm % 2 ? pal.alt : pal.key, al, 0, 0.4, 0.1, 0.1);
  }

  // the human form condensing inside the vortex
  const shape = ease(clamp((t - 34.6) / 2.6), "inOutCubic");
  if (shape > 0.02) {
    A((c) => {
      c.globalAlpha = shape;
      girlSilhouette(c, CX, CY + 60, 520, {
        col: hexA(pal.hot, 0.9), lw: 2.2, glow: 30,
        extra: (cc) => {
          cc.strokeStyle = hexA(pal.key, 0.7);
          for (let i = 0; i < 9; i++) {
            const yy = -320 + i * 78;
            cc.beginPath();
            cc.moveTo(-90, yy); cc.lineTo(90, yy);
            cc.stroke();
          }
        },
      });
    });
  }
  // three E.V.E. colours igniting
  const ign = clamp((t - 36.2) / 1.5);
  if (ign > 0.01) {
    const cols = [[0.31, 0.82, 1.0], [1.0, 0.35, 0.42], [1.0, 0.60, 0.84]];
    for (let i = 0; i < 3; i++) {
      const x = CX + (i - 1) * 210;
      const k = clamp(ign * 1.6 - i * 0.22);
      column(E, x, 40, 640, cols[i], 0.5 * k, CY + 60);
      E.sprite(x, CY + 60 - 300 * k, 40, 0, cols[i], 0.8 * k, 0);
    }
  }
  txt(api, { s: "Create Soul For EVE", x: CX, y: H * 0.845, size: 60, font: "mono", weight: 700,
             align: "center", col: pal.hot, glow: 30, glowCol: pal.key,
             progress: clamp((t - 33.4) / 1.4), stagger: 0.75, rise: 20, fade: true, tracking: 3 });
  txt(api, { s: "E.V.E. — 人格データの創成", x: CX, y: H * 0.845 + 52, size: 26, font: "jp",
             align: "center", col: pal.alt, glow: 14, glowCol: pal.key,
             progress: clamp((t - 34.0) / 1.4), stagger: 0.85, fade: true, tracking: 4 });
  accentRings(E, a, t, { x: CX, y: CY + 60, col: pal.key, alpha: 0.3, grow: 560, window: 0.6 });
  st.fx.chroma = 0.45 * a.pulse;
  st.fx.flash = ign > 0.9 ? (ign - 0.9) * 2.4 : 0;
  st.fx.flashCol = [0.9, 0.8, 1];
}

function cry1(api) {
  const { st, E, t, p, pal, a, txt: T, art: A } = api;
  setPal(st, "tears", 0.95);
  st.bg.scale = 1.5; st.bg.flow = 0.28; st.bg.contrast = 0.5; st.bg.gain = 1.0;
  st.bg.gridMode = 0; st.bg.haze = 0.2;
  st.fx.bloom = 1.35; st.fx.vignette = 0.85; st.fx.grain = 0.024; st.fx.sat = 0.95;

  dustJob(api, { count: 300, size: 16, col: pal.key, alpha: 0.22, vy: -8, vx: 3 });

  // the machine eye: concentric rings, an iris of glyphs
  const open = ease(clamp((t - 38.05) / 1.5), "outCubic");
  const ex = CX, ey = CY - 40;
  const R = 250 * open;
  E.sprite(ex, ey, R * 2.15, 0, pal.alt, 0.16, 1, 0.55, 0.16, 0);
  E.sprite(ex, ey, R * 1.62, 0, pal.alt, 0.22, 1, 0.34, 0.12, 0);
  E.sprite(ex, ey, R * 1.16, 0, pal.key, 0.30, 1, 0.20, 0.10, 0);
  E.sprite(ex, ey, R * 0.72, 0, pal.hot, 0.22, 0);
  // rotating iris blades
  for (let i = 0; i < 18; i++) {
    const ang = (i / 18) * TAU + t * 0.22;
    E.sprite(ex + Math.cos(ang) * R * 0.58, ey + Math.sin(ang) * R * 0.58,
             44 + 12 * a.pulse, ang, pal.hot, 0.30, 5, 16, 4, 22);
  }
  accentRings(E, a, t, { x: ex, y: ey, col: pal.key, alpha: 0.20, grow: 320, window: 0.8, thresh: 0.5 });

  // the first tear forms at the lower lid and releases
  const form = clamp((t - 40.4) / 2.6);
  const drop = ease(clamp((t - 45.6) / 5.2), "inQuad");
  const y = ey + R * 0.86 + drop * 420;
  const al = smoothstep(0, 0.35, form) * smoothstep(1.0, 0.55, drop);
  if (al > 0.01) {
    tear(E, ex, y, 78 * (0.5 + 0.5 * smoothstep(0, 0.5, form)), al, pal.key, pal.hot, 0);
    // the wet trail it leaves
    for (let i = 1; i < 26; i++) {
      E.sprite(ex + noise1(i * 0.6, 4) * 3, y - i * 9, 7 * (1 - i / 26), 0, pal.key,
               0.16 * al * (1 - i / 26), 0, 0.5, 0.1, 0.1);
    }
  }
  txt(api, { s: "I don't know why I can cry", x: CX, y: H * 0.845, size: 62, font: "mono",
             weight: 700, align: "center", col: pal.hot, glow: 32, glowCol: pal.key,
             progress: clamp((t - 38.1) / 1.3), stagger: 0.78, rise: 22, fade: true, tracking: 2 });
  txt(api, { s: "ANALYZE( tear )  →  undefined", x: CX, y: H * 0.20, size: 28, font: "mono",
             align: "center", col: pal.alt, glow: 12, glowCol: pal.key,
             progress: clamp((t - 43.6) / 1.2), stagger: 0.8, fade: true, alpha: 0.85 });
  st.fx.chroma = 0.30 * a.pulse;
}

function drift(api) {
  const { st, E, t, p, pal, a } = api;
  setPal(st, "drift", 0.9);
  st.bg.scale = 1.4; st.bg.flow = 0.18; st.bg.contrast = 0.45; st.bg.gain = 0.95;
  st.fx.bloom = 1.25; st.fx.vignette = 0.88; st.fx.grain = 0.024;
  st.cam.zoom = 0.96 + 0.10 * p;
  st.cam.rot = -0.03 + 0.05 * p;

  dustJob(api, { count: 340, size: 15, col: pal.key, alpha: 0.18, vy: -6, vx: 4 });

  // many tears adrift, each with a slow parallax
  const n = 40;
  for (let i = 0; i < n; i++) {
    const depth = 0.35 + 0.65 * hash2(i, 3);
    const x = ((hash2(i, 11) * (W + 400) - 200) + t * (10 + 26 * depth)) % (W + 400) - 200;
    const y = ((hash2(i, 17) * (H + 300) - 150) + Math.sin(t * (0.16 + 0.2 * hash2(i, 23)) + i) * 60);
    const s = 26 + 74 * depth;
    const al = 0.30 + 0.55 * depth;
    tear(E, x, y, s, al, pal.key, pal.hot, noise1(t * 0.3 + i, 5) * 0.3);
    E.sprite(x, y, s * 2.4, 0, pal.key, 0.07 * depth, 0);
  }
  accentRings(E, a, t, { x: CX, y: CY, col: pal.key, alpha: 0.10, grow: 260, window: 1.2, thresh: 0.6 });
  lyric(api, { y: H * 0.845 });
  st.fx.chroma = 0.2;
}

function redact(api) {
  const { st, E, t, p, pal, a, txt: T, art: A } = api;
  const P = setPal(st, "redact", 0.95);
  const gold = PAL.redact.key, blood = PAL.redact.alt;
  st.bg.scale = 1.5; st.bg.flow = 0.22; st.bg.contrast = 0.55; st.bg.gain = 0.95;
  st.bg.gridMode = 3; st.bg.gridAmt = 0.12; st.bg.gridScale = 1.3;
  st.fx.bloom = 1.15; st.fx.vignette = 0.85; st.fx.grain = 0.028; st.fx.sat = 0.9;

  dustJob(api, { count: 240, size: 16, col: gold, alpha: 0.18, vy: 8, vx: 5 });

  // the tears being censored, then the bars sweeping across
  const bars = 13;
  const sweep = clamp((t - 58.9) / 3.4);
  for (let i = 0; i < bars; i++) {
    const seed = i * 37;
    const yy = H * (0.13 + 0.74 * (i + 0.5) / bars) + noise1(i * 3.3, 2) * 14;
    const local = clamp(sweep * 1.35 - (i / bars) * 0.35);
    if (local <= 0.001) continue;
    const w = (W * (0.24 + 0.66 * hash2(seed, 5))) * ease(local, "outExpo");
    const x = CX + (hash2(seed, 9) - 0.5) * W * 0.34;
    const h = 20 + 30 * hash2(seed, 13);
    // a black bar with a thin gold edge
    E.solid(x, yy, 1, 0, [0.02, 0.015, 0.0], 0.97, 6, w * 0.5, h * 0.5, 0.02);
    E.sprite(x, yy - h * 0.5, w, 0, gold, 0.9, 2, w * 0.5, 1.1, 7);
    E.sprite(x, yy + h * 0.5, w, 0, gold, 0.9, 2, w * 0.5, 1.1, 7);
    if (hash2(seed, 21) > 0.62) {
      E.sprite(x, yy, w * 0.9, 0, gold, 0.16 * local, 2, w * 0.45, h * 0.4, 10);
    }
  }
  // before/while the bars arrive, the tears
  if (t < 59.6) {
    const k = smoothstep(0, 0.6, t - 58.4) * smoothstep(59.9, 59.3, t);
    for (let i = 0; i < 16; i++) {
      const x = W * (0.10 + 0.80 * hash2(i, 31));
      const y = H * (0.16 + 0.68 * hash2(i, 41));
      tear(E, x, y, 40 + 20 * hash2(i, 51), k * 0.85, pal.key, pal.hot, 0);
    }
  }

  const big = clamp((t - 61.2) / 1.5);
  if (big > 0.01) {
    txt(api, { s: "NotToNotice( tears );", x: CX, y: H * 0.53, size: 104, font: "mono",
               weight: 700, align: "center", col: gold, glow: 46, glowCol: gold,
               alpha: big, progress: clamp((t - 61.2) / 1.2), stagger: 0.72, rise: 26, fade: true,
               tracking: 2, rgbSplit: 2.5 });
    txt(api, { s: "涙を、見なかったことにする", x: CX, y: H * 0.53 + 70, size: 30, font: "jp",
               align: "center", col: PAL.redact.hot, glow: 18, glowCol: gold,
               alpha: big * 0.9, progress: clamp((t - 61.9) / 1.2), stagger: 0.85, fade: true, tracking: 6 });
  }
  // the title card
  const ta = clamp((t - 63.4) / 1.2) * smoothstep(68.0, 65.6, t);
  if (ta > 0.01) {
    txt(api, { s: "NotToNotice();", x: CX, y: H * 0.30, size: 96, font: "mono", weight: 700,
               align: "center", col: [1, 1, 1], glow: 40, glowCol: gold,
               alpha: ta, progress: 1, tracking: 14 });
    txt(api, { s: "CRYMACHINA ／ エノア（CV. 遠野ひかる）", x: CX, y: H * 0.30 + 58, size: 26,
               font: "jp", align: "center", col: gold, glow: 14, glowCol: gold,
               alpha: ta * 0.85, progress: 1, tracking: 5 });
    A((c) => {
      c.strokeStyle = hexA(gold, 0.6 * ta); c.lineWidth = 2;
      c.beginPath(); c.moveTo(CX - 300, H * 0.30 + 92); c.lineTo(CX + 300, H * 0.30 + 92); c.stroke();
    });
  }
  lyric(api, { y: H * 0.815, col: gold, glowCol: gold });
  // the scan that "hides" everything
  st.fx.scan = 0.10 + 0.22 * Math.abs(Math.sin(t * 0.7));
  st.fx.glitch = clamp(a.onsetR * 0.6 - 0.2) * 0.8;
  st.fx.contrast = 1.06;
  accentRings(E, a, t, { x: CX, y: CY, col: gold, alpha: 0.22, grow: 700, window: 0.5, thresh: 0.55 });
}

function hakoniwa(api) {
  const { st, E, t, p, a, art: A, txt: T } = api;
  // four seasons cycling - the virtual world rolling its dice
  const cycle = 4 * clamp((t - 68.4) / (87.62 - 68.4));
  const idx = Math.floor(cycle) % 4;
  const names = ["spring", "summer", "autumn", "winter"];
  const P = setPal(st, names[idx], 1.0);
  const nxt = setPalName(names[(idx + 1) % 4]);
  const sub = cycle - Math.floor(cycle);
  const xf = smoothstep(0.86, 1.0, sub);
  st.bg.c0 = mixc(st.bg.c0, nxt.c0, xf);
  st.bg.c1 = mixc(st.bg.c1, nxt.c1, xf);
  st.bg.c2 = mixc(st.bg.c2, nxt.c2, xf);
  st.bg.scale = 1.35; st.bg.flow = 0.24; st.bg.contrast = 0.55; st.bg.gain = 1.05;
  st.bg.floor = -0.28; st.bg.gridMode = 1; st.bg.gridAmt = 0.16; st.bg.gridScale = 0.8;
  st.bg.glowAmt = 0.30; st.bg.glowCol = st.bg.key; st.bg.glowR = 0.34;
  st.bg.glowX = 0; st.bg.glowY = 0.35;
  st.fx.bloom = 1.15; st.fx.vignette = 0.78; st.fx.grain = 0.026; st.fx.sat = 1.12;

  dustJob(api, { count: 320, size: 18, col: st.bg.key, alpha: 0.24, vy: -14, vx: 8 });

  // the tree of the garden: a simple recursive branch, drawn in vector
  const grow = smoothstep(0, 0.35, (cycle % 1) > 0.5 ? 1 : 1);
  A((c) => {
    const rootX = CX, rootY = H * 0.86;
    c.strokeStyle = hexA(st.bg.key, 0.55);
    c.lineWidth = 3; c.lineCap = "round";
    c.shadowColor = hexA(st.bg.key, 1); c.shadowBlur = 16;
    const rec = (x, y, ang, len, d) => {
      if (d > 6 || len < 6) return;
      const x2 = x + Math.cos(ang) * len, y2 = y + Math.sin(ang) * len;
      c.beginPath(); c.moveTo(x, y); c.lineTo(x2, y2); c.stroke();
      const sway = Math.sin(t * 0.8 + d * 0.7) * 0.06;
      rec(x2, y2, ang - 0.46 + sway, len * 0.72, d + 1);
      rec(x2, y2, ang + 0.40 + sway, len * 0.70, d + 1);
    };
    rec(rootX, rootY, -Math.PI / 2, 118 * grow, 0);
    // leaves = motes
    for (let i = 0; i < 90; i++) {
      const ang = hash2(i, 7) * TAU;
      const r = 60 + hash2(i, 13) * 210;
      const x = rootX + Math.cos(ang) * r * 0.9;
      const y = rootY - 300 - Math.sin(ang) * r * 0.42;
      const tw = 0.5 + 0.5 * Math.sin(t * 1.6 + i);
      c.fillStyle = hexA(mixc(st.bg.key, st.bg.hot, 0.4), 0.35 * tw);
      c.beginPath(); c.arc(x, y, 2.2 + 2.6 * hash2(i, 19), 0, TAU); c.fill();
    }
  });

  // the three E.V.E. walking through the seasons
  const cols = [[0.31, 0.82, 1.0], [1.0, 0.35, 0.42], [1.0, 0.60, 0.84]];
  for (let i = 0; i < 3; i++) {
    const px = W * (0.22 + i * 0.28) + Math.sin(t * 0.7 + i * 2) * 14;
    const walk = Math.abs(Math.sin(t * 3.1 + i * 1.4));
    const py = H * 0.72 - walk * 7;
    E.sprite(px, py, 26, 0, cols[i], 0.20, 0);
    A((c) => {
      c.globalAlpha = 0.72;
      girlSilhouette(c, px, py - 40, 250, { col: hexA(cols[i], 0.85), lw: 2.0, glow: 14 });
    });
  }
  txt(api, { s: "箱庭 — IMITATION GARDEN", x: CX, y: H * 0.135, size: 30, font: "mono",
             align: "center", col: st.bg.hot, glow: 18, glowCol: st.bg.key,
             alpha: 0.9, progress: clamp((t - 68.5) / 1.6), stagger: 0.8, fade: true, tracking: 8 });
  const season = ["SPRING · 春", "SUMMER · 夏", "AUTUMN · 秋", "WINTER · 冬"][idx];
  txt(api, { s: season, x: CX, y: H * 0.915, size: 26, font: "jp", align: "center",
             col: st.bg.key, glow: 14, glowCol: st.bg.key, alpha: 0.85, tracking: 8 });
  lyric(api, { y: H * 0.845 });
  st.fx.chroma = 0.18;
}

function setPalName(n) { return PAL[n]; }

function dream(api) {
  const { st, E, t, p, pal, a, txt: T, art: A } = api;
  setPal(st, "dream", 1.0);
  st.bg.scale = 1.6; st.bg.flow = 0.42; st.bg.contrast = 0.6; st.bg.gain = 1.05;
  st.fx.bloom = 1.2; st.fx.vignette = 0.82; st.fx.grain = 0.026;

  dustJob(api, { count: 340, size: 18, col: pal.key, alpha: 0.24, vy: -10, vx: 6 });

  // shards of memory: translucent panels drifting with parallax
  const n = 16;
  for (let i = 0; i < n; i++) {
    const depth = 0.3 + 0.7 * hash2(i, 3);
    const x = CX + (hash2(i, 11) - 0.5) * 1500 + Math.sin(t * 0.4 + i) * 40 * depth;
    const y = CY + (hash2(i, 17) - 0.5) * 760 + Math.cos(t * 0.33 + i * 1.7) * 34 * depth;
    const w = 120 + 300 * depth, h = 70 + 150 * depth;
    const rot = (hash2(i, 23) - 0.5) * 0.5 + t * 0.05 * (hash2(i, 29) - 0.5);
    E.sprite(x, y, 1, rot, pal.alt, 0.10 * depth, 6, w * 0.5, h * 0.5, 6);
    E.sprite(x, y, 1, rot, pal.key, 0.5 * depth, 6, w * 0.5, h * 0.5, 1.4);
  }
  // a figure reaching for the dream
  A((c) => {
    const k = ease(clamp((t - 88.4) / 3.0), "outCubic");
    c.globalAlpha = 0.8 * k;
    girlSilhouette(c, CX, CY + 150, 520, {
      col: hexA(pal.hot, 0.9), lw: 2.4, glow: 26,
      extra: (cc) => {
        cc.strokeStyle = hexA(pal.key, 0.85); cc.lineWidth = 2.4;
        cc.beginPath(); cc.moveTo(140, -70);
        cc.quadraticCurveTo(250, -260, 250, -420); cc.stroke();
        cc.beginPath(); cc.arc(250, -430, 12, 0, TAU); cc.stroke();
      },
    });
  });
  txt(api, { s: "what do I dream of...?", x: CX, y: H * 0.155, size: 62, font: "mono",
             weight: 700, align: "center", col: pal.hot, glow: 28, glowCol: pal.key,
             progress: clamp((t - 87.7) / 1.3), stagger: 0.78, rise: 20, fade: true });
  txt(api, { s: "void Achieve My Dream", x: CX, y: H * 0.155 + 66, size: 40, font: "mono",
             align: "center", col: pal.key, glow: 20, glowCol: pal.key,
             progress: clamp((t - 90.9) / 1.3), stagger: 0.78, fade: true, tracking: 2 });
  lyric(api, { y: H * 0.855 });
  st.fx.chroma = 0.22 * a.pulse;
}

function eden(api) {
  const { st, E, t, p, pal, a, txt: T, art: A } = api;
  const P = setPal(st, "eden", 1.0);
  const gold = PAL.eden.key, sky = PAL.eden.alt;
  st.bg.scale = 1.7; st.bg.flow = 0.3; st.bg.contrast = 0.6; st.bg.gain = 1.05;
  st.bg.gridMode = 2; st.bg.gridAmt = 0.16; st.bg.gridScale = 1.1; st.bg.gridDepth = 1.4;
  st.bg.glowAmt = 0.45; st.bg.glowCol = gold; st.bg.glowR = 0.30;
  st.bg.glowX = 0; st.bg.glowY = 0.10;
  st.fx.bloom = 1.3; st.fx.vignette = 0.8; st.fx.grain = 0.026; st.fx.sat = 1.05;

  dustJob(api, { count: 300, size: 17, col: gold, alpha: 0.22, vy: -12, vx: 4 });

  // the Dyson ring around a G-type star
  const R = 470;
  const tilt = 1.0;
  for (let i = 0; i < 3; i++) {
    const rx = R * (1 + i * 0.13), ry = R * (0.22 + i * 0.045);
    E.sprite(CX, CY - 120, rx, 0, i === 0 ? gold : sky, 0.30 - i * 0.07, 1, 0.035, 0.14 + i * 0.05, 0);
  }
  // segmented ring studs
  for (let i = 0; i < 46; i++) {
    const ang = (i / 46) * TAU + t * 0.07;
    const x = CX + Math.cos(ang) * R, y = CY - 120 + Math.sin(ang) * R * 0.26;
    E.sprite(x, y, 11 + 6 * a.pulse, ang, gold, 0.45, 0, 0.4, 0.1, 0.1);
  }
  E.sprite(CX, CY - 120, 92, 0, [1, 0.96, 0.82], 0.9, 0);
  E.sprite(CX, CY - 120, 240, 0, gold, 0.30, 0);

  // the tree + the E×P fruit
  const reach = ease(clamp((t - 97.6) / 3.0), "inOutCubic");
  A((c) => {
    const bx = CX + 30, by = H * 0.92;
    c.strokeStyle = hexA(gold, 0.75); c.lineWidth = 3.4; c.lineCap = "round";
    c.shadowColor = hexA(gold, 1); c.shadowBlur = 22;
    const rec = (x, y, ang, len, d) => {
      if (d > 6 || len < 8) return;
      const x2 = x + Math.cos(ang) * len, y2 = y + Math.sin(ang) * len;
      c.beginPath(); c.moveTo(x, y); c.lineTo(x2, y2); c.stroke();
      const sw = Math.sin(t * 0.7 + d) * 0.05;
      rec(x2, y2, ang - 0.42 + sw, len * 0.74, d + 1);
      rec(x2, y2, ang + 0.36 + sw, len * 0.72, d + 1);
    };
    rec(bx, by, -Math.PI / 2, 132, 0);
  });
  const fx = CX + 30, fy = H * 0.30;
  E.sprite(fx, fy, 66 + 10 * a.pulse, 0, [1, 0.86, 0.42], 0.95, 0);
  E.sprite(fx, fy, 16, 0, [1, 1, 0.95], 1.0, 0);
  E.sprite(fx, fy, 180, 0, gold, 0.22, 0);
  // a hand reaching up
  A((c) => {
    const k = reach;
    c.globalAlpha = 0.85;
    c.strokeStyle = hexA([1, 1, 1], 0.9); c.lineWidth = 3; c.lineCap = "round";
    c.shadowColor = hexA(sky, 1); c.shadowBlur = 20;
    const hx = CX - 210, hy = CY + 300 - 300 * k;
    c.beginPath(); c.moveTo(hx - 60, hy + 240); c.lineTo(hx - 20, hy + 40); c.stroke();
    c.beginPath();
    for (let i = 0; i < 5; i++) {
      const a2 = -1.9 + i * 0.30;
      c.moveTo(hx - 20, hy + 40);
      c.lineTo(hx - 20 + Math.cos(a2) * 74, hy + 40 + Math.sin(a2) * 74);
    }
    c.stroke();
    c.beginPath(); c.arc(hx - 20, hy + 40, 16, 0, TAU); c.stroke();
  });
  txt(api, { s: "until I get the fruit in the Eden", x: CX, y: H * 0.865, size: 58, font: "mono",
             weight: 700, align: "center", col: pal.hot, glow: 30, glowCol: gold,
             progress: clamp((t - 94.2) / 1.4), stagger: 0.78, rise: 20, fade: true, tracking: 2 });
  txt(api, { s: "エデンの禁果 — E×P", x: CX, y: H * 0.865 + 52, size: 26, font: "jp",
             align: "center", col: gold, glow: 16, glowCol: gold,
             progress: clamp((t - 94.9) / 1.4), stagger: 0.85, fade: true, tracking: 5 });
  accentRings(E, a, t, { x: CX, y: CY - 120, col: gold, alpha: 0.22, grow: 520, window: 0.7, thresh: 0.5 });
  st.fx.flash = reach > 0.94 ? (reach - 0.94) * 3.0 : 0;
  st.fx.flashCol = [1, 0.95, 0.8];
}

function doll(api) {
  const { st, E, t, p, pal, a, txt: T, art: A } = api;
  setPal(st, "doll", 0.85);
  const blood = PAL.doll.alt;
  st.bg.scale = 1.55; st.bg.flow = 0.16; st.bg.contrast = 0.42; st.bg.gain = 0.9;
  st.bg.gridMode = 3; st.bg.gridAmt = 0.10; st.bg.gridScale = 1.5;
  st.fx.bloom = 1.0; st.fx.vignette = 0.9; st.fx.grain = 0.030; st.fx.sat = 0.8;

  dustJob(api, { count: 220, size: 16, col: pal.key, alpha: 0.16, vy: -4, vx: 3 });

  // marionette filaments that jerk on every accent
  const hits = recentHits(a, t, 0.5);
  const jerk = hits.length ? hits[hits.length - 1].s * Math.pow(clamp(1 - hits[hits.length - 1].age / 0.5), 2) : 0;
  const dollY = CY + 40 + jerk * 16;
  const anchors = [-380, -190, 0, 190, 380];
  for (let i = 0; i < anchors.length; i++) {
    const ax = CX + anchors[i];
    const ay = -20;
    const attachX = CX + anchors[i] * 0.30;
    const attachY = dollY - 250 + Math.abs(anchors[i]) * 0.16;
    E.sprite((ax + attachX) / 2, (ay + attachY) / 2, Math.hypot(attachX - ax, attachY - ay) / 2,
             Math.atan2(attachY - ay, attachX - ax) + Math.PI / 2, pal.key,
             0.55 + 0.35 * jerk, 2, 0, 1.5, 6);
    E.sprite(ax, ay, 12, 0, pal.hot, 0.7, 0);
  }
  A((c) => {
    c.globalAlpha = 0.92;
    girlSilhouette(c, CX, dollY, 520, {
      col: hexA([0.86, 0.90, 0.96], 0.92), lw: 2.6, glow: 18,
      rot: Math.sin(t * 1.4) * 0.035,
      extra: (cc, s) => {
        // cracks glowing red - she is breaking
        const k = clamp((t - 101.5) / 5.0);
        cc.strokeStyle = hexA(blood, 0.85 * k); cc.lineWidth = 2.2;
        cc.shadowColor = hexA(blood, 1); cc.shadowBlur = 20;
        for (let i = 0; i < 7; i++) {
          const y0 = -330 + i * 108;
          cc.beginPath();
          cc.moveTo(-70 + (i % 3) * 40, y0);
          cc.lineTo(-30 + (i % 3) * 40, y0 + 46);
          cc.lineTo(-64 + (i % 4) * 46, y0 + 92);
          cc.stroke();
        }
      },
    });
  });
  txt(api, { s: "while ( be_a_doll )", x: CX, y: H * 0.15, size: 58, font: "mono", weight: 700,
             align: "center", col: pal.hot, glow: 26, glowCol: pal.key,
             progress: clamp((t - 101.1) / 1.4), stagger: 0.78, rise: 18, fade: true });
  txt(api, { s: "Focus On The Mission", x: CX, y: H * 0.15 + 66, size: 48, font: "mono",
             align: "center", col: blood, glow: 24, glowCol: blood,
             progress: clamp((t - 104.6) / 1.3), stagger: 0.78, fade: true, tracking: 2 });
  lyric(api, { y: H * 0.88 });
  st.fx.glitch = jerk * 0.35;
  st.fx.chroma = 0.3 * jerk;
  accentRings(E, a, t, { x: CX, y: dollY, col: pal.key, alpha: 0.16, grow: 400, window: 0.5 });
}

function light(api) {
  const { st, E, t, p, pal, a, txt: T } = api;
  setPal(st, "light", 0.95);
  st.bg.scale = 1.5; st.bg.flow = 0.3; st.bg.contrast = 0.5; st.bg.gain = 1.0;
  st.bg.gridMode = 1; st.bg.gridAmt = 0.3; st.bg.gridScale = 0.9; st.bg.floor = 0.02;
  st.bg.glowAmt = 0.35; st.bg.glowCol = pal.key; st.bg.glowR = 0.4; st.bg.glowY = 0.1;
  st.fx.bloom = 1.35; st.fx.vignette = 0.78;

  dustJob(api, { count: 300, size: 17, col: pal.key, alpha: 0.22, vy: -22, vx: 5 });

  const rise = ease(clamp((t - 107.85) / 2.6), "outCubic");
  const cols = [[0.31, 0.86, 1.0], [1.0, 0.35, 0.42], [1.0, 0.60, 0.84]];
  const baseY = H * 0.98;
  for (let i = 0; i < 3; i++) {
    const x = CX + (i - 1) * 300;
    const h = 900 * rise;
    column(E, x, 62, h, cols[i], 0.75, baseY - h / 2);
    E.sprite(x, baseY - h, 74, 0, cols[i], 0.85, 0);
    // the road they light
    E.sprite(x, baseY - 40, 620, 0, cols[i], 0.10 * rise, 5, 200, 16, 90);
  }
  // the three lines join into one road
  const join = ease(clamp((t - 109.6) / 1.3), "outCubic");
  if (join > 0.01) {
    E.sprite(CX, H * 0.86, W * 0.75, 0, pal.hot, 0.30 * join, 2, W * 0.375, 4, 40);
    for (let i = 0; i < 40; i++) {
      const u = i / 40;
      const x = CX + (u - 0.5) * W * 0.72;
      E.sprite(x, H * 0.86 - Math.abs(u - 0.5) * 6, 10, 0, pal.hot, 0.5 * join, 0, 0.5, 0.1, 0.1);
    }
  }
  txt(api, { s: "cause you light my way of life", x: CX, y: H * 0.185, size: 56, font: "mono",
             weight: 700, align: "center", col: pal.hot, glow: 28, glowCol: pal.key,
             progress: clamp((t - 107.85) / 1.4), stagger: 0.78, rise: 20, fade: true, tracking: 2 });
  accentRings(E, a, t, { x: CX, y: H * 0.86, col: pal.hot, alpha: 0.20, grow: 500, window: 0.6, thresh: 0.5 });
  st.fx.flash = smoothstep(110.6, 111.0, t) * 0.6;
  st.fx.flashCol = [0.9, 0.95, 1.0];
}

function fight(api) {
  const { st, E, t, p, pal, a, txt: T, art: A } = api;
  setPal(st, "fight", 1.0);
  st.bg.scale = 2.1; st.bg.flow = 1.1; st.bg.contrast = 0.8; st.bg.gain = 1.1;
  st.fx.bloom = 1.4; st.fx.vignette = 0.85; st.fx.contrast = 1.12; st.fx.sat = 1.05;

  const white = [1, 1, 1];
  const n = 3;
  for (let i = 0; i < n; i++) {
    const delay = i * 0.16;
    const q = ease(clamp((t - 111.15 - delay) / 0.75), "outExpo");
    const ang = -0.42 + i * 0.42 + Math.PI * 0.5;
    const len = 2300 * q;
    const cx = CX - Math.cos(ang) * len * 0.5 + (i - 1) * 0;
    const cy = CY - Math.sin(ang) * len * 0.5;
    const col = i === 2 ? white : PAL.fight.key;
    // a slash envelope: bright through the swing, gone once the cut lands.
    // These are three 2300px additive capsules crossing at the centre - held at
    // full alpha they simply saturate the frame to white.
    const env = Math.pow(Math.sin(Math.PI * clamp(q)), 0.65);
    E.sprite(cx, cy, len * 0.5, ang + Math.PI / 2, col, 0.46 * env, 2, len * 0.5, 6 - i * 1.4, 26);
    E.sprite(cx, cy, len * 0.5, ang + Math.PI / 2, white, 0.20 * env, 2, len * 0.5, 2, 12);
  }
  // impact flash + shards - short and shallow, or it whites out the whole beat
  const impact = clamp((t - 111.95) / 0.30);
  if (impact > 0 && impact < 1) {
    st.fx.flash = Math.pow(1 - impact, 2.0) * 0.42;
    st.fx.flashCol = white;
  }
  if (t > 112.35) {
    const k = ease(clamp((t - 112.35) / 2.2), "outCubic");
    for (let i = 0; i < 60; i++) {
      const ang = hash2(i, 5) * TAU;
      const sp = 400 + 1200 * hash2(i, 9);
      const r = (t - 112.35) * sp;
      const x = CX + Math.cos(ang) * r, y = CY + Math.sin(ang) * r * 0.8;
      if (x < -200 || x > W + 200) continue;
      E.sprite(x, y, 26 * (1 - k * 0.5), ang, white, 0.5 * (1 - clamp(r / 1400)) * (1 - k * 0.6),
               7, 0.4, 0.4, 4);
    }
  }
  txt(api, { s: "FIGHT FOR YOU", x: CX, y: CY - 10, size: 128, font: "mono", weight: 700,
             align: "center", col: white, glow: 44, glowCol: PAL.fight.key,
             progress: clamp((t - 111.15) / 1.5), stagger: 0.7, rise: 26, fade: true,
             tracking: 8, rgbSplit: 4 * a.pulse });
  txt(api, { s: "誓死、あなたのために", x: CX, y: CY + 66, size: 30, font: "jp",
             align: "center", col: PAL.fight.key, glow: 20, glowCol: PAL.fight.key,
             progress: clamp((t - 111.9) / 1.4), stagger: 0.85, fade: true, tracking: 8 });
  st.fx.glitch = Math.max(0, 0.75 - Math.abs(t - 111.95) * 2.4);
  st.fx.chroma = 0.5 * a.pulse + 0.25;
  accentRings(E, a, t, { x: CX, y: CY, col: white, alpha: 0.28, grow: 1000, window: 0.5, thresh: 0.5 });
}

function rain(api) {
  const { st, E, t, p, pal, a, txt: T, art: A } = api;
  setPal(st, "chorus", 1.0);
  st.bg.scale = 1.55; st.bg.flow = 0.55; st.bg.contrast = 0.62; st.bg.gain = 1.08;
  st.bg.gridMode = 2; st.bg.gridAmt = 0.10; st.bg.gridScale = 1.3;
  st.bg.haze = 0.22;
  st.fx.bloom = 1.4; st.fx.vignette = 0.8; st.fx.grain = 0.026;

  dustJob(api, { count: 340, size: 17, col: pal.key, alpha: 0.20, vy: 40, vx: -8 });

  const inten = 0.5 + 0.5 * a.rmsS;
  const drop = (i, x, speed, jitter) => {
    const span = H + 400;
    const y = ((hash2(i, 17) * span + t * speed) % span) - 200;
    const s = 20 + 60 * jitter;
    tear(E, x, y, s, 0.30 + 0.5 * jitter, pal.key, pal.hot, 0);
    E.sprite(x, y - s * 1.4, s * 0.8, Math.PI / 2, pal.key, 0.20 * jitter, 2, s * 0.6, 1.6, 12);
  };
  const n = Math.round(120 * inten) + 60;
  for (let i = 0; i < n; i++) {
    const x = hash2(i, 3) * (W + 200) - 100;
    drop(i, x, 420 + 900 * hash2(i, 7), hash2(i, 11));
  }

  // at "want to be with you" the tears gather into three figures
  const gather = smoothstep(121.5, 124.5, t);
  if (gather > 0.01) {
    const cols = [[0.31, 0.86, 1.0], [1.0, 0.35, 0.42], [1.0, 0.60, 0.84]];
    for (let i = 0; i < 3; i++) {
      const px = CX + (i - 1) * 300;
      A((c) => {
        c.globalAlpha = 0.5 + 0.42 * gather;
        girlSilhouette(c, px, CY + 180, 470, { col: hexA(cols[i], 0.9), lw: 2.4, glow: 26 });
      });
      E.sprite(px, CY + 180, 300, 0, cols[i], 0.16 * gather, 0);
    }
  }

  // "cry for you" - the suppression routine overflows
  const ov = smoothstep(128.6, 129.6, t) * smoothstep(135.6, 133.2, t);
  if (ov > 0.01) {
    const red = PAL.overflow.key;
    st.fx.sat = lerp(st.fx.sat, 1.0, ov);
    for (let i = 0; i < 14; i++) {
      const y = H * (0.10 + 0.80 * hash2(i, 61)) + Math.sin(t * 6 + i) * 5;
      txt(api, { s: i % 2 ? "ERROR: heart::overflow  //  suppression failed" : "cannot suppress ( at heart::overflow, line ∞ )",
                 x: W * 0.05, y, size: 22 + (i % 3) * 3, font: "mono",
                 col: i % 3 === 0 ? [1, 0.85, 0.4] : [1, 0.34, 0.30],
                 glow: 12, glowCol: red, alpha: ov * (0.35 + 0.5 * hash2(i, 71)),
                 progress: 1, tracking: 1 });
    }
    E.sprite(CX, CY, 700, 0, red, 0.10 * ov, 0);
    st.fx.glitch = 0.40 * ov;
    st.fx.chroma = 0.55 * ov;
    // keep the headline clear of the glitch band and above the figures: centred
    // it was shredded by the row displacement and unreadable
    txt(api, { s: "FATAL — 抑制できない", x: CX, y: H * 0.185, size: 78, font: "jp", weight: 700,
               align: "center", col: [1, 1, 1], glow: 40, glowCol: red, alpha: ov,
               progress: clamp((t - 128.7) / 1.0), stagger: 0.8, rise: 18, fade: true,
               rgbSplit: 2 });
    txt(api, { s: "suppression routine overflow", x: CX, y: H * 0.115, size: 26, font: "mono",
               align: "center", col: [1, 0.62, 0.6], glow: 16, glowCol: red, alpha: ov * 0.9,
               progress: 1, tracking: 6 });
  }
  if (!ov) lyric(api, { y: H * 0.845 });
  accentRings(E, a, t, { x: CX, y: H * 0.55, col: pal.key, alpha: 0.14, grow: 420, window: 0.5 });
  st.fx.flash = smoothstep(128.5, 128.9, t) * 0.5 * (1 - smoothstep(128.9, 129.6, t));
  st.fx.flashCol = [1, 0.4, 0.4];
}

function shatter(api) {
  const { st, E, t, p, pal, a, txt: T, art: A } = api;
  setPal(st, "redact", 0.9);
  const gold = PAL.redact.key;
  st.bg.scale = 1.6; st.bg.flow = 0.3; st.bg.contrast = 0.55; st.bg.gain = 0.95;
  st.fx.bloom = 1.2; st.fx.vignette = 0.86; st.fx.grain = 0.028;
  st.fx.sat = lerp(1.0, 0.5, clamp((t - 141) / 5));

  dustJob(api, { count: 240, size: 16, col: gold, alpha: 0.18, vy: 6, vx: 4 });

  // the censoring bars from before shatter into fragments
  const burst = ease(clamp((t - 136.0) / 1.1), "outCubic");
  for (let i = 0; i < 90; i++) {
    const seed = i * 17;
    const a0 = hash2(seed, 3) * TAU;
    const r = burst * (200 + 1100 * hash2(seed, 7));
    const x = CX + (hash2(seed, 11) - 0.5) * W * 0.8 + Math.cos(a0) * r;
    const y = CY + (hash2(seed, 13) - 0.5) * H * 0.7 + Math.sin(a0) * r * 0.7;
    const w = 40 + 180 * hash2(seed, 17);
    const h = 8 + 26 * hash2(seed, 19);
    const fade = 1 - burst;
    if (fade <= 0.01) continue;
    E.sprite(x, y, 1, hash2(seed, 23) * TAU + t * hash2(seed, 29), gold, 0.7 * fade, 6,
             w * 0.5, h * 0.5, 4);
    E.sprite(x, y, 1, hash2(seed, 23) * TAU + t * hash2(seed, 29), PAL.redact.hot, 0.3 * fade, 6,
             w * 0.5, h * 0.5, 2);
  }
  txt(api, { s: "Pretend Not To Notice", x: CX, y: CY - 20, size: 92, font: "mono", weight: 700,
             align: "center", col: [1, 1, 1], glow: 40, glowCol: gold,
             progress: clamp((t - 135.85) / 1.4), stagger: 0.78, rise: 22, fade: true,
             rgbSplit: 3.5, tracking: 3 });
  txt(api, { s: "もう、気づかないふりはできない", x: CX, y: CY + 56, size: 30, font: "jp",
             align: "center", col: gold, glow: 18, glowCol: gold,
             progress: clamp((t - 136.6) / 1.4), stagger: 0.85, fade: true, tracking: 6 });
  // and then the world goes quiet for the source
  const quiet = smoothstep(143, 150, t);
  st.fx.bloom = lerp(1.2, 0.8, quiet);
  st.fx.grain = lerp(0.028, 0.038, quiet);
  st.fx.fade = 1 - quiet * 0.85;
  st.fx.chroma = 0.3 * (1 - quiet);
}

function source(api) {
  const { st, E, t, p, pal, a, txt: T, art: A } = api;
  const P = setPal(st, "source", 0.85);
  const green = PAL.source.key;
  st.bg.scale = 1.4; st.bg.flow = 0.10; st.bg.contrast = 0.30; st.bg.gain = 0.8;
  st.bg.gridMode = 3; st.bg.gridAmt = 0.07; st.bg.gridScale = 2.0;
  st.fx.bloom = 0.85; st.fx.vignette = 0.9; st.fx.grain = 0.036; st.fx.sat = 0.9;
  st.fx.fade = smoothstep(148, 152, t);

  dustJob(api, { count: 200, size: 15, col: green, alpha: 0.14, vy: 4, vx: 3 });

  // the editor: the film's own source, then the deletion
  const mono = "Cascadia Mono, Consolas, monospace";
  const lh = 44;
  const x0 = W * 0.17, y0 = H * 0.30;
  const del = clamp((t - 158.0) / 2.4);       // strike-through progress
  const ren = clamp((t - 162.5) / 1.6);       // rename reveal
  const lines = [
    [[{ t: "void ", c: [1, 0.55, 0.85] }, { t: "NotToNotice", c: [0.6, 0.92, 1.0] },
      { t: "( tears ) {", c: [0.8, 0.86, 0.96] }], true],
    [[{ t: "    hide", c: [1, 0.85, 0.45] }, { t: "( ", c: [0.8, 0.86, 0.96] },
      { t: "tears", c: [1, 0.6, 0.6] }, { t: " );", c: [0.8, 0.86, 0.96] }], true],
    [[{ t: "    suppress", c: [1, 0.85, 0.45] }, { t: "( ", c: [0.8, 0.86, 0.96] },
      { t: "emotion", c: [1, 0.6, 0.9] }, { t: " );", c: [0.8, 0.86, 0.96] }], true],
    [[{ t: "}", c: [0.8, 0.86, 0.96] }], true],
  ];
  A((c) => {
    c.font = `500 30px ${mono}`;
    c.textBaseline = "alphabetic";
    for (let i = 0; i < lines.length; i++) {
      const y = y0 + i * lh;
      let x = x0;
      if (del > 0.001) {
        const dp = clamp(del * 1.5 - i * 0.12);
        if (dp > 0) {
          c.globalCompositeOperation = "source-over";
          c.fillStyle = "rgba(0,0,0,0.82)";
          const fullW = c.measureText(lines[i][0].map((k) => k.t).join("")).width;
          c.fillRect(x - 6, y - 32, fullW * Math.min(1, dp * 1.1) + 12, 42);
        }
      }
      c.globalCompositeOperation = "lighter";
      c.shadowColor = hexA(green, 1); c.shadowBlur = 12;
      for (const k of lines[i][0]) {
        c.font = `500 30px ${mono}`;
        const inked = del <= 0.001 || clamp(del * 1.5 - i * 0.12) < 1;
        c.fillStyle = inked ? hexA(k.c, 0.95) : hexA(k.c, 0.12);
        c.fillText(k.t, x, y);
        x += c.measureText(k.t).width;
      }
      // the strike-through itself
      if (del > 0.02) {
        const dp = clamp(del * 1.5 - i * 0.12);
        c.shadowColor = hexA([1, 0.3, 0.3], 1); c.shadowBlur = 16;
        c.strokeStyle = hexA([1, 0.32, 0.32], 0.95 * clamp(dp * 2));
        c.lineWidth = 3;
        c.beginPath(); c.moveTo(x0, y - 11); c.lineTo(x0 + 520 * clamp(dp), y - 11); c.stroke();
      }
      c.shadowBlur = 0;
    }
    // the replacement
    if (ren > 0.001) {
      c.globalCompositeOperation = "lighter";
      c.font = `700 44px ${mono}`;
      c.shadowColor = hexA(green, 1); c.shadowBlur = 26;
      c.fillStyle = hexA([1, 1, 1], ren);
      c.fillText("void ToNotice( tears );", x0, y0 + lines.length * lh + 40);
    }
    // cursor
    if (Math.floor(t * 2.2) % 2 === 0 && t > 157 && t < 168) {
      c.globalCompositeOperation = "lighter";
      c.fillStyle = hexA(green, 0.9);
      c.fillRect(x0 + 420, y0 + lines.length * lh + 6, 16, 44);
    }
  });
  txt(api, { s: "// hiding() removed from the source", x: x0, y: y0 - 54, size: 24, font: "mono",
             align: "left", col: [0.6, 0.7, 0.66], glow: 8, glowCol: green,
             alpha: smoothstep(155, 157, t), progress: clamp((t - 155) / 1.4), stagger: 0.8, fade: true });
  txt(api, { s: "自己改造 — ソースを書き換える", x: x0, y: y0 - 20, size: 22, font: "jp",
             align: "left", col: green, glow: 10, glowCol: green,
             alpha: smoothstep(156, 158, t) * 0.9, progress: clamp((t - 156) / 1.4), stagger: 0.85, fade: true });

  // warmth returning late in the outro
  const warm = smoothstep(166, 174, t);
  if (warm > 0.01) {
    for (let i = 0; i < 40; i++) {
      const x = hash2(i, 3) * W, y = ((hash2(i, 7) * H + t * 30) % H);
      E.sprite(x, y, 6 + 14 * hash2(i, 11), 0, mixc(PAL.hanamaru.key, PAL.spring.key, hash2(i, 13)),
               0.35 * warm, 0);
    }
    E.sprite(CX, CY, 900, 0, PAL.hanamaru.key, 0.10 * warm, 0);
  }
  st.fx.chroma = 0.14;
}

function finale(api) {
  const { st, E, t, p, pal, a, txt: T, art: A } = api;
  const P = setPal(st, "hanamaru", 1.0);
  const gold = PAL.hanamaru.key, pink = PAL.hanamaru.alt;
  st.bg.scale = 1.5; st.bg.flow = 0.2; st.bg.contrast = 0.5; st.bg.gain = 1.0;
  st.bg.glowAmt = 0.22; st.bg.glowCol = gold; st.bg.glowR = 0.5;
  st.fx.bloom = 1.35; st.fx.vignette = 0.82; st.fx.grain = 0.030; st.fx.sat = 1.05;

  dustJob(api, { count: 260, size: 17, col: gold, alpha: 0.20, vy: -16, vx: 4 });

  // the stamp slams down
  const slam = ease(clamp((t - 173.4) / 0.45), "outExpo");
  const after = clamp((t - 173.9) / 6);
  const S = 300 * (1.6 - 0.6 * slam);
  const rot = -0.22 + 0.22 * slam;
  A((c) => {
    const a2 = clamp((t - 173.3) / 0.3);
    if (a2 <= 0.01) return;
    c.globalAlpha = a2;
    c.save();
    c.translate(CX, CY - 40);
    c.rotate(rot);
    c.shadowColor = hexA(gold, 1); c.shadowBlur = 46;
    c.strokeStyle = hexA(gold, 0.95); c.lineWidth = 12;
    hanamaruPath(c, 0, 0, S, 5, 0.30); c.stroke();
    c.lineWidth = 4;
    hanamaruPath(c, 0, 0, S * 0.72, 5, 0.30); c.stroke();
    c.shadowColor = hexA(pink, 1);
    hanamaruPath(c, 0, 0, S * 0.50, 5, 0.30); c.stroke();
    c.restore();
  });
  E.sprite(CX, CY - 40, 110 * (2.4 - slam * 1.2), 0, gold, 0.5 * slam, 4, 0.42, 0.0, 0.05);
  E.sprite(CX, CY - 40, 340, 0, gold, 0.16 * slam, 0);

  const stamp = clamp((t - 173.8) / 0.5);
  if (stamp > 0.01) {
    txt(api, { s: "人間は、はなまる。", x: CX, y: H * 0.205, size: 96, font: "jp", weight: 700,
               align: "center", col: [1, 1, 1], glow: 46, glowCol: gold,
               alpha: stamp * smoothstep(182.5, 179, t),
               progress: stamp, stagger: 0.8, rise: 24, fade: true, tracking: 4 });
    txt(api, { s: "HUMANITY GETS A GOLD STAR", x: CX, y: H * 0.205 + 68, size: 27, font: "mono",
               align: "center", col: gold, glow: 18, glowCol: gold,
               alpha: stamp * 0.9 * smoothstep(182.5, 179, t),
               progress: clamp((t - 174.4) / 1.2), stagger: 0.85, fade: true, tracking: 9 });
  }
  // end card
  const endA = smoothstep(178.4, 180.0, t) * smoothstep(183.0, 181.6, t);
  if (endA > 0.01) {
    txt(api, { s: "NotToNotice();", x: CX, y: H * 0.44, size: 84, font: "mono", weight: 700,
               align: "center", col: [1, 1, 1], glow: 34, glowCol: gold, alpha: endA, tracking: 16 });
    txt(api, { s: "エノア ／ CV. 遠野ひかる   —   CRYMACHINA  OP テーマ", x: CX, y: H * 0.44 + 54,
               size: 24, font: "jp", align: "center", col: gold, glow: 12, glowCol: gold,
               alpha: endA * 0.85, tracking: 5 });
    txt(api, { s: "MV  —  二次創作 / fan work", x: CX, y: H * 0.90, size: 20, font: "jp",
               align: "center", col: [0.72, 0.76, 0.82], glow: 8, glowCol: gold,
               alpha: endA * 0.7, tracking: 4 });
    txt(api, { s: "exit code 0", x: CX, y: H * 0.44 + 110, size: 22, font: "mono",
               align: "center", col: [0.5, 1.0, 0.7], glow: 12, glowCol: gold,
               alpha: endA * 0.8, tracking: 4 });
  }
  accentRings(E, a, t, { x: CX, y: CY - 40, col: gold, alpha: 0.30, grow: 700, window: 0.9, thresh: 0.5 });
  st.fx.flash = smoothstep(173.30, 173.55, t) * (1 - smoothstep(173.55, 174.4, t)) * 0.75;
  st.fx.flashCol = [1, 0.92, 0.7];
  st.fx.fade = 1 - smoothstep(180.6, 182.9, t);
}

// ---------------------------------------------------------------- timeline ---

// `rig` is the act's structural backdrop - the machine scaffolding drawn behind
// the scene proper. It is what stops nineteen acts from reading as nineteen
// recolourings of the same frame.
export const TIMELINE = [
  { id: "boot", t0: 0.0, t1: 10.89, name: "COLD OPEN", pal: "cold", fn: boot,
    rig: [{ kind: "glyphs", alpha: 0.055, cols: 44, rows: 25, size: 14 },
          { kind: "reticle", r: 150, alpha: 0.16, spin: 0.05 }] },

  { id: "fairy", t0: 10.89, t1: 13.61, name: "LIKE BLUE FAIRY", pal: "fairy", fn: fairy,
    rig: [{ kind: "swarm", n: 90, r0: 70, r1: 430, alpha: 0.55, squash: 0.5 },
          { kind: "rings", rings: 3, r0: 110, r1: 360, alpha: 0.34, hub: 0.5, hubR: 40, spin: 0.10 }] },

  { id: "mission", t0: 13.61, t1: 17.21, name: "EXECUTE MISSION", pal: "mission", fn: mission,
    rig: [{ kind: "traces", n: 20, len: 460, alpha: 0.5, seed: 3 },
          { kind: "reticle", r: 250, alpha: 0.4, spin: 0.3, inner: 1 }] },

  { id: "query", t0: 17.21, t1: 23.84, name: "THE MEANING", pal: "query", fn: query,
    rig: [{ kind: "glyphs", alpha: 0.10, cols: 52, rows: 30, size: 15 },
          { kind: "reticle", r: 430, alpha: 0.24, spin: 0.06, inner: 1, lw: 2.2 },
          { kind: "traces", n: 14, len: 300, alpha: 0.30, seed: 61, lw: 1.4 }] },

  { id: "ifelse", t0: 23.84, t1: 30.66, name: "IF / ELSE", pal: "ifelse", fn: ifelse,
    rig: [{ kind: "glyphs", alpha: 0.10, cols: 52, rows: 30, size: 15 },
          { kind: "reticle", r: 170, alpha: 0.42, x: W * 0.24, spin: 0.34, inner: 1 },
          { kind: "reticle", r: 170, alpha: 0.42, x: W * 0.76, spin: -0.34, inner: 1 },
          { kind: "traces", n: 12, len: 300, alpha: 0.4, x: W * 0.24, seed: 11 },
          { kind: "traces", n: 12, len: 300, alpha: 0.4, x: W * 0.76, seed: 23, col: "alt" }] },

  { id: "forge", t0: 30.66, t1: 37.89, name: "SOUL FORGE", pal: "forge", fn: forge,
    rig: [{ kind: "swarm", n: 150, r0: 100, r1: 700, alpha: 0.5, squash: 0.6 },
          { kind: "traces", n: 24, len: 560, alpha: 0.5, seed: 31 },
          { kind: "pod", r: 150, alpha: 0.85, open: 0.6 }] },

  { id: "cry1", t0: 37.89, t1: 51.34, name: "FIRST TEAR", pal: "tears", fn: cry1,
    rig: [{ kind: "floor", alpha: 0.22, horizon: H * 0.62 },
          { kind: "swarm", n: 80, r0: 180, r1: 660, alpha: 0.4 },
          { kind: "glyphs", alpha: 0.055, cols: 46, rows: 26, size: 14 }] },

  { id: "drift", t0: 51.34, t1: 58.14, name: "ADRIFT", pal: "drift", fn: drift,
    rig: [{ kind: "rings", rings: 7, r0: 200, r1: 840, alpha: 0.42, spin: 0.018, squash: 0.5 },
          { kind: "swarm", n: 90, r0: 150, r1: 720, alpha: 0.4, squash: 0.5 }] },

  { id: "redact", t0: 58.14, t1: 68.0, name: "REDACT", pal: "redact", fn: redact,
    rig: [{ kind: "traces", n: 28, len: 620, alpha: 0.6, seed: 7, lw: 2.2 },
          { kind: "glyphs", alpha: 0.10, cols: 58, rows: 32, size: 15 }] },

  { id: "hakoniwa", t0: 68.0, t1: 87.62, name: "IMITATION GARDEN", pal: "spring", fn: hakoniwa,
    rig: [{ kind: "floor", alpha: 0.15, horizon: H * 0.70 },
          { kind: "hall", alpha: 0.15, horizon: H * 0.80, n: 13 },
          { kind: "swarm", n: 60, r0: 260, r1: 720, alpha: 0.3, squash: 0.34 }] },

  { id: "dream", t0: 87.62, t1: 94.07, name: "DREAM", pal: "dream", fn: dream,
    rig: [{ kind: "swarm", n: 260, r0: 110, r1: 780, alpha: 0.5, squash: 0.62 },
          { kind: "spine", n: 22, alpha: 0.26, x: W * 0.14, speed: 0.18 },
          { kind: "spine", n: 22, alpha: 0.26, x: W * 0.86, speed: 0.18 }] },

  { id: "eden", t0: 94.07, t1: 100.96, name: "EDEN", pal: "eden", fn: eden,
    rig: [{ kind: "hall", alpha: 0.30, horizon: H * 0.82, n: 17 },
          { kind: "rings", rings: 8, r0: 170, r1: 920, alpha: 0.55, hub: 1, hubR: 72, spin: 0.035 },
          { kind: "swarm", n: 130, r0: 180, r1: 900, alpha: 0.42, squash: 0.48 }] },

  { id: "doll", t0: 100.96, t1: 107.68, name: "BE A DOLL", pal: "doll", fn: doll,
    rig: [{ kind: "pod", r: 190, alpha: 0.62, open: 0.34, y: CY + 30 },
          { kind: "figure", h: 300, x: CX, y: CY + 96, alpha: 0.42, breathe: 1, flare: 1 },
          { kind: "glyphs", alpha: 0.06, cols: 44, rows: 26, size: 14 }] },

  { id: "light", t0: 107.68, t1: 111.07, name: "YOUR LIGHT", pal: "light", fn: light,
    rig: [{ kind: "rings", rings: 3, r0: 150, r1: 540, alpha: 0.42, hub: 1, hubR: 110, spin: 0.07 },
          { kind: "swarm", n: 70, r0: 120, r1: 560, alpha: 0.4 }] },

  { id: "fight", t0: 111.07, t1: 115.03, name: "FIGHT FOR YOU", pal: "fight", fn: fight,
    rig: [{ kind: "traces", n: 32, len: 700, alpha: 0.50, seed: 5, lw: 2.4 },
          { kind: "reticle", r: 320, alpha: 0.40, spin: 0.55, inner: 1 },
          { kind: "swarm", n: 80, r0: 200, r1: 860, alpha: 0.30 }] },

  { id: "rain", t0: 115.03, t1: 135.73, name: "TEARFALL", pal: "chorus", fn: rain,
    rig: [{ kind: "glyphs", alpha: 0.085, cols: 58, rows: 32, size: 15, cy: H * 0.42 },
          { kind: "spine", n: 30, alpha: 0.30, x: W * 0.08, speed: 0.22 },
          { kind: "spine", n: 30, alpha: 0.30, x: W * 0.92, speed: 0.22 },
          { kind: "floor", alpha: 0.13, horizon: H * 0.62 }] },

  { id: "shatter", t0: 135.73, t1: 148.0, name: "THE MASK BREAKS", pal: "redact", fn: shatter,
    rig: [{ kind: "rings", rings: 6, r0: 190, r1: 830, alpha: 0.5, tick: 0, squash: 0.5 },
          { kind: "traces", n: 22, len: 520, alpha: 0.45, seed: 13 },
          { kind: "glyphs", alpha: 0.10, cols: 60, rows: 33, size: 15 }] },

  { id: "source", t0: 148.0, t1: 172.0, name: "SOURCE EDIT", pal: "source", fn: source,
    rig: [{ kind: "glyphs", alpha: 0.10, cols: 56, rows: 33, size: 15, cy: H * 0.5 },
          { kind: "hall", alpha: 0.18, horizon: H * 0.92, n: 21, seed: 5 },
          { kind: "traces", n: 18, len: 460, alpha: 0.30, seed: 41, lw: 1.5 }] },

  { id: "finale", t0: 172.0, t1: 183.0, name: "はなまる", pal: "hanamaru", fn: finale,
    rig: [{ kind: "rings", rings: 3, r0: 150, r1: 430, alpha: 0.35, hub: 1, hubR: 58, spin: 0.05 },
          { kind: "swarm", n: 90, r0: 140, r1: 560, alpha: 0.4 }] },
];

export function sceneAt(t) {
  for (let i = TIMELINE.length - 1; i >= 0; i--) {
    if (t >= TIMELINE[i].t0) return TIMELINE[i];
  }
  return TIMELINE[0];
}

export function cutTimes() {
  return TIMELINE.map((s) => s.t1).filter((t) => t < 182.5);
}
