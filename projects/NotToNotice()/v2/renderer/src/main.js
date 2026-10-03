// ---------------------------------------------------------------------------
// main.js - frame pipeline:  scene -> global layers -> type layer -> GL
// ---------------------------------------------------------------------------

import { Audio, clamp, lerp, smoothstep, noise1, srand } from "./core.js";
import { Engine } from "./engine.js";
import { TypeLayer, TYPE_FLAGS } from "./type.js";
import { TIMELINE, sceneAt, cutTimes } from "./scenes.js";
import { drawStructure } from "./structure.js";
import { newState, W, H, CX, CY, PAL } from "./film.js";

const FPS = 60;
const TOTAL = 183.0;
const FRAMES = Math.ceil(TOTAL * FPS);

// --- global bloom trim ------------------------------------------------------
// Each act authors its own bloom amount (0.70 .. 1.40) but the threshold was
// left at its 0.42 default, so raising the amount mostly bought mid-tone haze
// instead of making highlights flare. Every glyph was composited before bloom,
// so it also wore a blurred copy of itself: measured on the FATAL frame, that
// veil ate 35% of the edge energy in the text region. Threshold up, amount
// down, in one place, rather than re-tuning nineteen scenes.
const BLOOM_SCALE = 0.30;    // 0.70..1.40  ->  0.21..0.42
const BLOOM_MAX = 0.45;
const BLOOM_THRESH = 0.70;

const canvas = document.getElementById("c");
canvas.width = W;
canvas.height = H;
canvas.style.width = "960px";
canvas.style.height = "540px";

const E = new Engine(canvas);
const ty = new TypeLayer(E.ctx, W, H);
const CUTS = cutTimes();

let AUDIO = null;
let ready = false;
let LAST = null;

// --------------------------------------------------------------- painting ---

function pushText(st, o) { st.texts.push(o); st.textActive = true; }

function flushJobs(st) {
  for (const j of st.jobs) {
    if (j.kind === "rain") ty.rain(j.o);
    else if (j.kind === "dust") ty.dust(j.o);
    else if (j.kind === "art") ty.custom(j.fn, j.opts);
  }
  for (const o of st.texts) ty.run(o);
}

/** global camera life: drift, breathing, accent punches, cut transitions */
function globalLayer(st, t, frame, a) {
  const cam = st.cam;
  cam.x += noise1(t * 0.067, 21) * 16 + noise1(t * 0.231, 31) * 5;
  cam.y += noise1(t * 0.058, 41) * 11 + noise1(t * 0.192, 51) * 4;
  cam.zoom *= 1 + 0.013 * noise1(t * 0.131, 61) + 0.016 * a.pulse;

  const sh = Math.max(0, a.onsetR - 0.5) * 14;
  cam.x += srand(frame, 7) * sh;
  cam.y += srand(frame, 9) * sh;
  cam.rot += noise1(t * 0.047, 71) * 0.007 + srand(frame, 11) * sh * 0.0009;

  for (const c of CUTS) {
    const d = t - c;
    if (d > 0 && d < 0.24) {
      const k = 1 - d / 0.24;
      st.fx.chroma = Math.max(st.fx.chroma, 0.55 * k);
      st.fx.glitch = Math.max(st.fx.glitch, 0.30 * k * k);
      st.fx.flash = Math.max(st.fx.flash, 0.20 * k * k);
      st.fx.exposure *= 1 + 0.12 * k;
    } else if (d > -0.12 && d < 0) {
      cam.zoom *= 1 + 0.02 * (-d / 0.12);   // pre-roll tightening
    }
  }
  st.fx.fade *= smoothstep(0.0, 1.1, t) * (1 - smoothstep(181.0, 182.9, t));
}

/** the act's structural backdrop, declared in the timeline */
function structureLayer(st, t, frame, a, sc) {
  if (!sc.rig) return;
  const pal = st._pal || PAL.cold;
  const punch = 1 + 0.028 * a.pulse;         // the whole rig breathes on accents
  st.textActive = true;
  st.jobs.push({
    kind: "art",
    opts: { composite: "lighter" },
    fn: (c) => {
      c.save();
      c.translate(CX, CY); c.scale(punch, punch); c.translate(-CX, -CY);
      drawStructure(c, W, H, t, pal, sc.rig, CX, CY);
      c.restore();
    },
  });
}

/** the machine's own telemetry: a light strand, 8 nodes, an ExP counter */
function telemetry(st, t, frame, a, sc) {
  const gate = smoothstep(10.4, 12.4, t) * (1 - smoothstep(170.5, 174.0, t));
  if (gate < 0.01) return;
  const pal = st._pal || PAL.cold;
  const x0 = W * 0.06, x1 = W * 0.94, y = H - 34;
  const prog = clamp(t / 182.0);

  E.sprite((x0 + x1) / 2, y, (x1 - x0) / 2, 0, pal.alt, 0.20 * gate, 2, (x1 - x0) / 2, 0.8, 3);
  const px = lerp(x0, x1, prog);
  E.sprite((x0 + px) / 2, y, (px - x0) / 2, 0, pal.key, 0.40 * gate, 2, (px - x0) / 2, 1.4, 6);
  E.sprite((x0 + px) / 2, y, (px - x0) / 2, 0, pal.hot, 0.18 * gate * (0.6 + 0.6 * a.pulse),
           2, (px - x0) / 2, 3, 14);

  const names = ["PROPATOR", "ECCLESIA", "NOEIN", "ANTHROPOS", "LETHEIA", "LOGOS", "ZOE", "ENOA"];
  for (let i = 0; i < 8; i++) {
    const nx = lerp(x0, x1, i / 7);
    const near = Math.exp(-Math.pow((prog - i / 7) * 22, 2));
    E.sprite(nx, y, 7 + 5 * near, Math.PI / 4, prog >= i / 7 ? pal.hot : pal.alt,
             0.5 * gate + 0.4 * near * gate, 7, 1, 1, 0.8);
    if (near > 0.06) {
      pushText(st, { s: names[i], x: nx, y: y - 34, size: 15, font: "mono", align: "center",
                     col: pal.hot, glow: 10, glowCol: pal.key, alpha: near * gate,
                     tracking: 2.4, progress: 1 });
    }
  }
  E.sprite(px, y, 30, 0, pal.hot, 0.55 * gate * (0.7 + 0.5 * a.pulse), 0);
  E.sprite(px, y, 60, 0, pal.key, 0.16 * gate, 0);

  const exp = (prog * prog * 100).toFixed(1);
  pushText(st, { s: `E×P  ${exp.padStart(5)}%`, x: x1, y: y - 30, size: 19, font: "mono",
                 align: "right", col: pal.hot, glow: 12, glowCol: pal.key,
                 alpha: 0.80 * gate, tracking: 2.4, progress: 1 });
  const age = t - sc.t0;
  const ca = gate * (0.32 + 0.68 * Math.exp(-Math.pow((age - 0.7) / 1.1, 2)));
  pushText(st, { s: sc.name, x: x0, y: y - 30, size: 19, font: "mono", align: "left",
                 col: pal.hot, glow: 12, glowCol: pal.key, alpha: ca, tracking: 4.2, progress: 1 });
  pushText(st, { s: `${String(TIMELINE.indexOf(sc) + 1).padStart(2, "0")} / ${String(TIMELINE.length).padStart(2, "0")}`,
                 x: x0, y: y + 22, size: 15, font: "mono", align: "left",
                 col: pal.alt, glow: 8, glowCol: pal.key, alpha: 0.6 * gate, tracking: 3, progress: 1 });
}

// ----------------------------------------------------------------- frame ----

function renderFrame(frame) {
  const t = frame / FPS;
  const a = AUDIO.at(frame);

  E.addCount = 0;
  E.solidCount = 0;
  const st = newState(t, frame);
  st._pal = PAL.cold;
  st.bg.c0 = PAL.cold.c0; st.bg.c1 = PAL.cold.c1; st.bg.c2 = PAL.cold.c2;
  st.bg.key = PAL.cold.key; st.bg.alt = PAL.cold.alt; st.bg.hot = PAL.cold.hot;

  const sc = sceneAt(t);
  const dur = sc.t1 - sc.t0;
  const p = clamp((t - sc.t0) / dur);

  const api = {
    st, E, ty, W, H, CX, CY, t, frame, p, dur, a, scene: sc,
    get pal() { return st._pal || PAL.cold; },
    txt(o) { pushText(st, o); },
    art(fn, opts) { st.jobs.push({ kind: "art", fn, opts }); st.textActive = true; },
    rain(o) {
      st.jobs.push({ kind: "rain", o: { t: o.t === undefined ? t : o.t, ...o } });
      st.textActive = true;
    },
    dust(o) {
      st.jobs.push({ kind: "dust", o: { t: o.t === undefined ? t : o.t, ...o } });
      st.textActive = true;
    },
  };

  ty.clear();
  sc.fn(api);

  structureLayer(st, t, frame, a, sc);
  globalLayer(st, t, frame, a);
  telemetry(st, t, frame, a, sc);
  flushJobs(st);

  st.textGain = 1.0;
  st.textTint = [1, 1, 1];

  if (!DBG.noTrim) {
    const s = DBG.trimScale ?? BLOOM_SCALE;
    st.fx.bloom = Math.min(st.fx.bloom * s, DBG.trimMax ?? BLOOM_MAX);
    st.fx.bloomThresh = Math.max(st.fx.bloomThresh, DBG.trimThresh ?? BLOOM_THRESH);
  }

  // A/B switches used to attribute "softness" to a specific stage. Applied
  // after the trim so a probe measures exactly the value it asks for. Never
  // set during a real render.
  if (DBG.bloomOff) st.fx.bloom = 0;
  if (DBG.bloomSet !== undefined) st.fx.bloom = DBG.bloomSet;
  if (DBG.thresh !== undefined) st.fx.bloomThresh = DBG.thresh;
  if (DBG.chromaOff) { st.fx.chroma = 0; st.fx.chromaR = 0; }
  if (DBG.grainOff) st.fx.grain = 0;
  if (DBG.scanOff) st.fx.scan = 0;
  if (DBG.vigOff) st.fx.vignette = 0;
  if (DBG.noText) st.textActive = false;
  if (DBG.noNeb) { st.bg.nebMix = 1; st.bg.haze = 0; }

  LAST = st;
  E.render(st, frame);
  return st;
}

const DBG = {
  noTrim: false, bloomOff: false, chromaOff: false, grainOff: false,
  scanOff: false, vigOff: false, noText: false, noNeb: false,
};
window.__dbg = DBG;

// ------------------------------------------------------------------ boot ----

async function boot() {
  const res = await fetch("../analysis/out/analysis.json");
  if (!res.ok) throw new Error("analysis.json missing: " + res.status);
  const json = await res.json();
  AUDIO = new Audio(json);
  ready = true;
  window.__ready = true;
  window.__info = { fps: FPS, frames: FRAMES, w: W, h: H, bpm: json.bpm, scenes: TIMELINE.length };
  return window.__info;
}

window.__meta = { fps: FPS, frames: FRAMES, total: TOTAL, W, H };
window.renderFrame = (i) => { renderFrame(i | 0); return true; };
window.renderAt = (t) => { renderFrame(Math.round(t * FPS)); return true; };
window.renderToDataURL = (i, kind = "image/png") => { renderFrame(i | 0); return canvas.toDataURL(kind); };

// ------------------------------------------------------------- capture ------
// The frame never crosses CDP as a string (a 4.6 MB dataURL wedges
// Runtime.evaluate). Instead the page renders, dumps the framebuffer with
// readPixels and POSTs the raw RGBA straight to the local server, which pipes
// it into ffmpeg. ~8.3 MB per frame over loopback, no encode step at all.

let CAPTURE = false;
window.__capStart = () => {
  CAPTURE = true;
  E.captureFlip = true;
  // During capture every pixel lands in rtOut, so the default framebuffer is
  // dead weight. Chrome still composites it every frame (preserveDrawingBuffer
  // forces the copy), and with several workers sharing one integrated GPU that
  // compositing is what starves the renderers. Shrink it to nothing.
  canvas.width = 16;
  canvas.height = 16;
  canvas.style.width = "16px";
  canvas.style.height = "16px";
  return true;
};
window.__capStop = () => {
  CAPTURE = false;
  E.captureFlip = false;
  canvas.width = W;
  canvas.height = H;
  canvas.style.width = "960px";
  canvas.style.height = "540px";
  return true;
};

window.__captureRaw = async (frame, url) => {
  renderFrame(frame | 0);
  const px = E.readback();
  try {
    const r = await fetch(url, {
      method: "POST",
      headers: { "content-type": "application/octet-stream" },
      body: px,
    });
    if (!r.ok) return "HTTP " + r.status;
    const j = await r.json();
    return j.written === px.length ? true : "short " + j.written;
  } catch (e) {
    return "ERR " + e.message;
  }
};

// Timed, no-network variant used to measure readback cost on its own.
window.__timeReadback = (frame, n = 20) => {
  const t0 = performance.now();
  for (let k = 0; k < n; k++) renderFrame((frame + k) | 0);
  const t1 = performance.now();
  for (let k = 0; k < n; k++) { renderFrame((frame + k) | 0); E.readback(); }
  const t2 = performance.now();
  return { render: +((t1 - t0) / n).toFixed(2), renderPlusReadback: +((t2 - t1) / n).toFixed(2) };
};

window.__tyClear = () => { ty.clear(); return true; };

// splits a frame into JS submission, GPU drain and the readback copy so we can
// tell which side of the pipeline a slow timepoint is actually paying for.
window.__bench = (frame, n = 10) => {
  const gl = E.gl;
  renderFrame(frame);                       // warm
  const t0 = performance.now();
  for (let k = 0; k < n; k++) renderFrame(frame + k);
  const t1 = performance.now();
  gl.finish();
  const t2 = performance.now();
  E.readback();
  const t3 = performance.now();
  E.readback();
  const t4 = performance.now();
  window.renderFrame(frame + 1);
  gl.finish();
  const t5 = performance.now();
  return {
    jsSubmit: +((t1 - t0) / n).toFixed(2),   // per frame, CPU only
    gpuDrain: +((t2 - t1) / n).toFixed(2),   // per frame, real GPU time
    readFirst: +(t3 - t2).toFixed(2),
    readAgain: +(t4 - t3).toFixed(2),
    oneFrameGpu: +(t5 - t4).toFixed(2),
    add: E.addCount, solid: E.solidCount, texts: LAST ? LAST.texts.length : -1,
  };
};

window.__glInfo = () => {
  const gl = E.gl;
  const dbg = gl.getExtension("WEBGL_debug_renderer_info");
  return {
    version: gl.getParameter(gl.VERSION),
    vendor: dbg ? gl.getParameter(dbg.UNMASKED_VENDOR_WEBGL) : gl.getParameter(gl.VENDOR),
    renderer: dbg ? gl.getParameter(dbg.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER),
    maxTex: gl.getParameter(gl.MAX_TEXTURE_SIZE),
    floatOK: E.floatOK,
  };
};

window.__setTextMode = (m) => { E.textUploadMode = m; return E.textUploadMode; };
window.__typeFlags = TYPE_FLAGS;
window.__ty = ty;

// per-frame GPU time, measured by forcing a sync after EVERY frame with a 1x1
// readPixels (gl.finish() is a no-op on this ANGLE/D3D11 backend)
window.__gpu = (frame, n = 6) => {
  const gl = E.gl;
  const px = new Uint8Array(4);
  const sync = () => {
    gl.bindFramebuffer(gl.FRAMEBUFFER, E.rtOut.fb);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, px);
  };
  renderFrame(frame | 0); sync();
  const t0 = performance.now();
  for (let k = 0; k < n; k++) { renderFrame((frame | 0) + k); sync(); }
  const t1 = performance.now();
  return { perFrame: +((t1 - t0) / n).toFixed(2), readback: (() => {
    const a = performance.now(); E.readback(); return +(performance.now() - a).toFixed(2);
  })() };
};

// ---------------------------------------------------------------- debug ----

function readStats(target) {
  const gl = E.gl;
  const w = target ? target.w : E.W;
  const h = target ? target.h : E.H;
  if (target) target.bind(); else { gl.bindFramebuffer(gl.FRAMEBUFFER, null); }
  const buf = new Uint8Array(w * h * 4);
  gl.readPixels(0, 0, w, h, gl.RGBA, gl.UNSIGNED_BYTE, buf);
  let max = 0, sum = 0, nz = 0;
  for (let i = 0; i < buf.length; i += 4) {
    const v = Math.max(buf[i], buf[i + 1], buf[i + 2]);
    if (v > max) max = v;
    sum += v;
    if (v > 4) nz++;
  }
  return { w, h, max, mean: +(sum / (w * h)).toFixed(3), litFrac: +(nz / (w * h)).toFixed(4) };
}

window.__E = E;
window.__stats = () => readStats(null);
window.__statsScene = () => readStats(E.rtScene);
window.__statsNeb = () => readStats(E.rtNeb);
window.__statsText = () => {
  const c = E.textCanvas;
  const d = E.ctx.getImageData(0, 0, c.width, c.height).data;
  let max = 0, nz = 0;
  for (let i = 0; i < d.length; i += 4) {
    const v = Math.max(d[i], d[i + 1], d[i + 2]);
    if (v > max) max = v;
    if (v > 4) nz++;
  }
  return { max, litFrac: +(nz / (c.width * c.height)).toFixed(4) };
};
window.__state = () => {
  const st = LAST;
  return st ? {
    t: +st.t.toFixed(2), add: E.addCount, solid: E.solidCount, texts: st.texts.length,
    jobs: st.jobs.length, textActive: st.textActive,
    fade: st.fx.fade, exposure: st.fx.exposure, bloom: st.fx.bloom, vignette: st.fx.vignette,
    sat: st.fx.sat, grain: st.fx.grain, bloomThresh: st.fx.bloomThresh,
    bgGain: st.bg.gain, bgGlowAmt: st.bg.glowAmt, bgGridAmt: st.bg.gridAmt,
    c0: st.bg.c0.map((v) => +v.toFixed(4)), c2: st.bg.c2.map((v) => +v.toFixed(4)),
    zoom: +st.cam.zoom.toFixed(4), pal: st._pal && st._pal.name,
    floatOK: E.floatOK,
  } : null;
};

window.__ready = false;
window.__boot = boot;
window.__hasAudio = () => ready;
