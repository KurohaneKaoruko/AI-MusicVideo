// ---------------------------------------------------------------------------
// core.js - deterministic maths, audio features, WebGL plumbing
// ---------------------------------------------------------------------------

export const TAU = Math.PI * 2;

// ---------------------------------------------------------------- hashing ---
// All "randomness" is a pure function of (index, salt) so that any frame can be
// re-rendered on any machine and come out bit-identical.

export function hash1(n) {
  let x = Math.imul(n ^ 0x9e3779b9, 0x85ebca6b) >>> 0;
  x ^= x >>> 13;
  x = Math.imul(x, 0xc2b2ae35) >>> 0;
  x ^= x >>> 16;
  return (x >>> 0) / 4294967296;
}

export function hash2(a, b) {
  return hash1((Math.imul(a | 0, 73856093) ^ Math.imul(b | 0, 19349663)) >>> 0);
}

export function hash3(a, b, c) {
  return hash1((Math.imul(a | 0, 73856093) ^ Math.imul(b | 0, 19349663) ^ Math.imul(c | 0, 83492791)) >>> 0);
}

/** deterministic value in [-1,1] */
export function srand(a, b) {
  return hash2(a, b) * 2 - 1;
}

export function clamp(v, a = 0, b = 1) {
  return v < a ? a : v > b ? b : v;
}

export function lerp(a, b, t) {
  return a + (b - a) * t;
}

export function smoothstep(a, b, x) {
  const t = clamp((x - a) / (b - a || 1e-9));
  return t * t * (3 - 2 * t);
}

export function smootherstep(a, b, x) {
  const t = clamp((x - a) / (b - a || 1e-9));
  return t * t * t * (t * (t * 6 - 15) + 10);
}

/** gaussian falloff - avoids the `-x ** 2` precedence trap */
export function gauss(x, s) {
  const u = x / (s || 1e-9);
  return Math.exp(-u * u);
}

export function ease(t, kind = "outCubic") {  t = clamp(t);
  switch (kind) {
    case "inQuad": return t * t;
    case "outQuad": return 1 - (1 - t) * (1 - t);
    case "inOutQuad": return t < 0.5 ? 2 * t * t : 1 - 2 * (1 - t) * (1 - t);
    case "inCubic": return t * t * t;
    case "outCubic": return 1 - Math.pow(1 - t, 3);
    case "inOutCubic": return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
    case "outQuart": return 1 - Math.pow(1 - t, 4);
    case "outQuint": return 1 - Math.pow(1 - t, 5);
    case "inOutQuint": return t < 0.5 ? 16 * t ** 5 : 1 - Math.pow(-2 * t + 2, 5) / 2;
    case "outExpo": return t >= 1 ? 1 : 1 - Math.pow(2, -10 * t);
    case "inExpo": return t <= 0 ? 0 : Math.pow(2, 10 * t - 10);
    case "outBack": { const c = 1.70158, c3 = c + 1; return 1 + c3 * Math.pow(t - 1, 3) + c * Math.pow(t - 1, 2); }
    case "outElastic": {
      const c4 = TAU / 3;
      return t <= 0 ? 0 : t >= 1 ? 1 : Math.pow(2, -10 * t) * Math.sin((t * 10 - 0.75) * c4) + 1;
    }
    case "outBounce": {
      const n1 = 7.5625, d1 = 2.75;
      if (t < 1 / d1) return n1 * t * t;
      if (t < 2 / d1) return n1 * (t -= 1.5 / d1) * t + 0.75;
      if (t < 2.5 / d1) return n1 * (t -= 2.25 / d1) * t + 0.9375;
      return n1 * (t -= 2.625 / d1) * t + 0.984375;
    }
    default: return t;
  }
}

/** value noise in 1D - smooth deterministic motion */
export function noise1(x, salt = 0) {
  const i = Math.floor(x), f = x - i;
  const u = f * f * (3 - 2 * f);
  return lerp(srand(i, salt), srand(i + 1, salt), u);
}

export function fbm1(x, salt = 0, oct = 3) {
  let s = 0, a = 0.5, f = 1, n = 0;
  for (let i = 0; i < oct; i++) { s += a * noise1(x * f, salt + i * 977); n += a; a *= 0.5; f *= 2.03; }
  return s / n;
}

// ------------------------------------------------------------------ colour ---

export function rgb(hex) {
  const n = parseInt(hex.replace("#", ""), 16);
  return [((n >> 16) & 255) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
}

export function mixc(a, b, t) {
  return [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)];
}

export function scaleC(c, k) {
  return [c[0] * k, c[1] * k, c[2] * k];
}

export function css(c, a = 1) {
  const r = Math.round(clamp(c[0]) * 255), g = Math.round(clamp(c[1]) * 255), b = Math.round(clamp(c[2]) * 255);
  return a >= 1 ? `rgb(${r},${g},${b})` : `rgba(${r},${g},${b},${a.toFixed(3)})`;
}

// --------------------------------------------------------- audio features ---

const SMOOTH_FRAMES = 5;

export class Audio {
  constructor(json) {
    this.data = json;
    this.fps = json.fps;
    this.frames = json.frames;
    this.hits = json.hits;
    this.beats = json.beats;
    this.anchors = json.anchors;
    this.bpm = json.bpm;

    // lightly smoothed copies for visuals (one-pole IIR, causal => deterministic)
    this.s = {};
    for (const k of ["rms", "low", "mid", "high", "centroid", "onset", "kick", "hat", "pulse"]) {
      const src = json[k];
      const out = new Float32Array(src.length);
      let acc = 0;
      const a = 0.55;
      for (let i = 0; i < src.length; i++) { acc = acc * a + src[i] * (1 - a); out[i] = acc; }
      this.s[k + "S"] = out;
      this.s[k + "R"] = this.rise(src);
    }
    this.hitsByFrame = new Map();
    for (const [t, s] of json.hits) {
      const f = Math.round(t * this.fps);
      this.hitsByFrame.set(f, Math.max(this.hitsByFrame.get(f) || 0, s));
    }
  }

  /** fast-attack / slow-release follower */
  rise(src) {
    const out = new Float32Array(src.length);
    let acc = 0;
    const up = 0.9, dn = 0.12;
    for (let i = 0; i < src.length; i++) {
      acc = src[i] > acc ? acc + (src[i] - acc) * up : acc + (src[i] - acc) * dn;
      out[i] = acc;
    }
    return out;
  }

  at(frame) {
    const i = Math.max(0, Math.min(this.frames - 1, frame | 0));
    const d = this.data;
    const s = this.s;
    const f = frame - i;
    const ip = (arr) => {
      const a = arr[i], b = arr[Math.min(arr.length - 1, i + 1)];
      return a + (b - a) * f;
    };
    return {
      rms: ip(d.rms), low: ip(d.low), mid: ip(d.mid), high: ip(d.high),
      centroid: ip(d.centroid), onset: ip(d.onset), kick: ip(d.kick), hat: ip(d.hat),
      pulse: ip(d.pulse),
      rmsS: ip(s.rmsS), lowS: ip(s.lowS), midS: ip(s.midS), highS: ip(s.highS),
      onsetS: ip(s.onsetS), kickS: ip(s.kickS), hatS: ip(s.hatS), pulseS: ip(s.pulseS),
      rmsR: ip(s.rmsR), onsetR: ip(s.onsetR), kickR: ip(s.kickR),
      hit: this.hitAt(frame),
      // a second-order "energy of the last ~1.2 s" for section-level intensity
      loud: this.loudAt(frame),
      // the raw accent list travels with the frame so scene helpers such as
      // recentHits()/accentRings() can take the same object as everything else
      hits: this.hits,
      beats: this.beats,
      bpm: this.bpm,
    };
  }

  /** strongest onset hit at (or within 1 frame of) this frame */
  hitAt(frame) {
    let best = 0;
    for (let d = -1; d <= 1; d++) {
      const v = this.hitsByFrame.get(frame + d);
      if (v !== undefined) best = Math.max(best, v);
    }
    // gated: only count as an accent if it stands out
    return best > 0.35 ? best : 0;
  }

  loudAt(frame) {
    const i = Math.max(0, Math.min(this.frames - 1, frame | 0));
    const r = this.data.rms;
    let acc = 0, n = 0;
    for (let k = i - 36; k <= i + 36; k += 3) {
      if (k < 0 || k >= r.length) continue;
      acc += r[k]; n++;
    }
    return n ? acc / n : 0;
  }
}

// -------------------------------------------------------------------- gl -----

export function createGL(canvas, opts = {}) {
  const gl = canvas.getContext("webgl2", {
    antialias: false,
    alpha: false,
    depth: false,
    stencil: false,
    premultipliedAlpha: false,
    preserveDrawingBuffer: true, // required so the capture readback sees the frame
    powerPreference: "high-performance",
    ...opts,
  });
  if (!gl) throw new Error("WebGL2 unavailable");
  gl.disable(gl.DEPTH_TEST);
  gl.disable(gl.CULL_FACE);
  return gl;
}

export function compile(gl, type, src, name = "shader") {
  const s = gl.createShader(type);
  gl.shaderSource(s, src);
  gl.compileShader(s);
  if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
    const log = gl.getShaderInfoLog(s);
    throw new Error(`${name} compile failed:\n${log}\n` +
      src.split("\n").map((l, i) => `${String(i + 1).padStart(3)}| ${l}`).join("\n"));
  }
  return s;
}

export function program(gl, vs, fs, name = "prog") {
  const p = gl.createProgram();
  gl.attachShader(p, compile(gl, gl.VERTEX_SHADER, vs, name + ".vs"));
  gl.attachShader(p, compile(gl, gl.FRAGMENT_SHADER, fs, name + ".fs"));
  gl.linkProgram(p);
  if (!gl.getProgramParameter(p, gl.LINK_STATUS)) {
    throw new Error(`${name} link failed: ${gl.getProgramInfoLog(p)}`);
  }
  // cache uniform locations
  const u = {};
  const n = gl.getProgramParameter(p, gl.ACTIVE_UNIFORMS);
  for (let i = 0; i < n; i++) {
    const info = gl.getActiveUniform(p, i);
    const nm = info.name.replace(/\[0\]$/, "");
    u[nm] = gl.getUniformLocation(p, nm);
  }
  return { prog: p, u, name };
}

export function makeQuad(gl) {
  const vao = gl.createVertexArray();
  gl.bindVertexArray(vao);
  const buf = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buf);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
  gl.enableVertexAttribArray(0);
  gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
  gl.bindVertexArray(null);
  return vao;
}

export function makeRT(gl, w, h, { float = false, linear = true } = {}) {
  const tex = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, tex);
  const internal = float ? gl.RGBA16F : gl.RGBA8;
  const type = float ? gl.HALF_FLOAT : gl.UNSIGNED_BYTE;
  gl.texImage2D(gl.TEXTURE_2D, 0, internal, w, h, 0, gl.RGBA, type, null);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, linear ? gl.LINEAR : gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, linear ? gl.LINEAR : gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  const fb = gl.createFramebuffer();
  gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
  gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
  if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) !== gl.FRAMEBUFFER_COMPLETE) {
    throw new Error("incomplete framebuffer");
  }
  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  return { tex, fb, w, h, bind() { gl.bindFramebuffer(gl.FRAMEBUFFER, fb); gl.viewport(0, 0, w, h); } };
}
