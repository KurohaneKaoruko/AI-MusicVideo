// ---------------------------------------------------------------------------
// math.ts - deterministic maths. every "random" value is a pure function of
// its inputs so any frame renders bit-identical on any machine.
// ---------------------------------------------------------------------------

export const TAU = Math.PI * 2;

export const clamp = (v: number, a = 0, b = 1): number =>
  v < a ? a : v > b ? b : v;

export const lerp = (a: number, b: number, t: number): number => a + (b - a) * t;

export const smoothstep = (a: number, b: number, x: number): number => {
  const t = clamp((x - a) / (b - a || 1e-9));
  return t * t * (3 - 2 * t);
};

export const gauss = (x: number, s: number): number => {
  const u = x / (s || 1e-9);
  return Math.exp(-u * u);
};

export function hash1(n: number): number {
  let x = Math.imul(n ^ 0x9e3779b9, 0x85ebca6b) >>> 0;
  x ^= x >>> 13;
  x = Math.imul(x, 0xc2b2ae35) >>> 0;
  x ^= x >>> 16;
  return (x >>> 0) / 4294967296;
}

export function hash2(a: number, b: number): number {
  return hash1((Math.imul(a | 0, 73856093) ^ Math.imul(b | 0, 19349663)) >>> 0);
}

/** deterministic value in [-1,1] */
export const srand = (a: number, b: number): number => hash2(a, b) * 2 - 1;

/** smooth 1D value noise */
export function noise1(x: number, salt = 0): number {
  const i = Math.floor(x);
  const f = x - i;
  const u = f * f * (3 - 2 * f);
  return lerp(srand(i, salt), srand(i + 1, salt), u);
}

export function fbm1(x: number, salt = 0, oct = 3): number {
  let s = 0;
  let a = 0.5;
  let f = 1;
  let n = 0;
  for (let i = 0; i < oct; i++) {
    s += a * noise1(x * f, salt + i * 977);
    n += a;
    a *= 0.5;
    f *= 2.03;
  }
  return s / n;
}

export type EaseKind =
  | "linear" | "inQuad" | "outQuad" | "inOutQuad" | "inCubic" | "outCubic"
  | "inOutCubic" | "outQuart" | "outQuint" | "inOutQuint" | "outExpo"
  | "inExpo" | "outBack" | "outElastic";

export function ease(t: number, kind: EaseKind = "outCubic"): number {
  t = clamp(t);
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
    case "outBack": {
      const c = 1.70158;
      const c3 = c + 1;
      return 1 + c3 * Math.pow(t - 1, 3) + c * Math.pow(t - 1, 2);
    }
    case "outElastic": {
      const c4 = TAU / 3;
      return t <= 0 ? 0 : t >= 1 ? 1 : Math.pow(2, -10 * t) * Math.sin((t * 10 - 0.75) * c4) + 1;
    }
    default: return t;
  }
}

// ------------------------------------------------------------------ colour ---

export type RGB = [number, number, number];

export const rgb = (hex: string): RGB => {
  const n = parseInt(hex.replace("#", ""), 16);
  return [((n >> 16) & 255) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
};

export const mixc = (a: RGB, b: RGB, t: number): RGB => [
  lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t),
];

export const rgba = (c: RGB, alpha = 1): string =>
  `rgba(${Math.round(clamp(c[0]) * 255)},${Math.round(clamp(c[1]) * 255)},${Math.round(clamp(c[2]) * 255)},${alpha.toFixed(3)})`;

export const hexc = (c: RGB): string => {
  const h = (v: number) => Math.round(clamp(v) * 255).toString(16).padStart(2, "0");
  return `#${h(c[0])}${h(c[1])}${h(c[2])}`;
};

export const scaleC = (c: RGB, k: number): RGB => [c[0] * k, c[1] * k, c[2] * k];
