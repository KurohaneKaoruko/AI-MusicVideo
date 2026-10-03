// ---------------------------------------------------------------------------
// timeline.ts - the film as 15 movements. one continuous space, reconfigured.
// each movement declares palette, sky/geometry/particle state and fx trim.
// ---------------------------------------------------------------------------

import { clamp, smoothstep, mixc } from "./math";
import { mixPal, pal, type PalLive } from "./palette";
import type { AudioFrame } from "./audio";

export type Movement = {
  id: string;
  name: string;
  title: string;      // kanji/latin tag for HUD
  t0: number;
  t1: number;
};

export const MOVEMENTS: Movement[] = [
  { id: "null",     name: "NULL",            title: "無",     t0: 0.0,    t1: 6.2 },
  { id: "boot",     name: "COLD BOOT",       title: "起動",   t0: 6.2,    t1: 10.89 },
  { id: "fairy",    name: "LIKE BLUE FAIRY", title: "青妖",   t0: 10.89,  t1: 17.21 },
  { id: "query",    name: "THE QUERY",       title: "問",     t0: 17.21,  t1: 30.66 },
  { id: "forge",    name: "SOUL FORGE",      title: "鋳",     t0: 30.66,  t1: 43.5 },
  { id: "admit",    name: "FIRST TEAR",      title: "涙",     t0: 43.5,   t1: 58.14 },
  { id: "redact",   name: "REDACT",          title: "隠",     t0: 58.14,  t1: 68.0 },
  { id: "garden",   name: "IMITATION GARDEN",title: "箱庭",   t0: 68.0,   t1: 87.62 },
  { id: "ascent",   name: "THE ASCENT",      title: "昇",     t0: 87.62,  t1: 100.96 },
  { id: "doll",     name: "BE A DOLL",       title: "傀儡",   t0: 100.96, t1: 107.68 },
  { id: "fight",    name: "FIGHT FOR YOU",   title: "戦",     t0: 107.68, t1: 115.03 },
  { id: "tearfall", name: "TEARFALL",        title: "泣",     t0: 115.03, t1: 135.73 },
  { id: "break",    name: "THE MASK BREAKS", title: "壊",     t0: 135.73, t1: 148.0 },
  { id: "source",   name: "SOURCE EDIT",     title: "改",     t0: 148.0,  t1: 172.0 },
  { id: "verdict",  name: "HANAMARU",        title: "花丸",   t0: 172.0,  t1: 183.0 },
];

export const CUTS = MOVEMENTS.slice(1).map((m) => m.t0);

export const movementAt = (t: number): { m: Movement; mi: number; lt: number; p: number } => {
  let mi = 0;
  for (let i = 0; i < MOVEMENTS.length; i++) {
    if (t >= MOVEMENTS[i].t0) mi = i;
  }
  const m = MOVEMENTS[mi];
  const lt = t - m.t0;
  const p = clamp(lt / (m.t1 - m.t0));
  return { m, mi, lt, p };
};

// --------------------------------------------------------------- gl state ---

export type GlState = {
  pal: PalLive;
  // sky
  neb: number;
  nebScale: number;
  flow: number;
  stars: number;
  rays: number;
  focal: [number, number];
  focalR: number;
  grid: number;
  gridSpeed: number;
  // geometry centrepiece
  // 0 none 1 vital 2 disc 3 mandala 4 strings 5 cracks 6 orb 7 slashes 8 strike 9 doll
  geo: number;
  geoA: [number, number, number, number];
  geoB: [number, number, number, number];
  pulseVal: number;
  // particles
  partMode: number;
  partAmt: number;
  partCol: [number, number, number];
  partCol2: [number, number, number];
  // post
  bloom: number;
  streak: number;
  chroma: number;
  grain: number;
  vig: number;
  scan: number;
  sharp: number;
  exposure: number;
  sat: number;
  glitch: number;
  flash: number;
  flashCol: [number, number, number];
  fade: number;
  camX: number;
  camY: number;
  camZoom: number;
  rings: number[];
  comet: number;
  cometT0: number;
  ringGain: number;
  seed: number;
};

const S = (o: Partial<GlState>): GlState => ({
  pal: pal("null"),
  neb: 0.5, nebScale: 2.2, flow: 1, stars: 0, rays: 0,
  focal: [0, 0.05], focalR: 0.45,
  grid: 0, gridSpeed: 0.4,
  geo: 0, geoA: [0, 0, 0, 0], geoB: [0, 0, 0, 0], pulseVal: 0,
  partMode: 0, partAmt: 0, partCol: [0.5, 0.8, 1], partCol2: [1, 1, 1],
  bloom: 0.55, streak: 0.2, chroma: 0.3, grain: 0.032, vig: 0.55, scan: 0.05, sharp: 0.32,
  exposure: 1, sat: 1.12, glitch: 0, flash: 0, flashCol: [1, 1, 1], fade: 1,
  camX: 0, camY: 0, camZoom: 1.06,
  rings: [], comet: 0, cometT0: 0, ringGain: 0.45, seed: 0,
});

/** geometric motif per movement: the abstract "figure" of each act */
const geoFor = (mi: number, lt: number, p: number): { mode: number; a: [number, number, number, number]; b: [number, number, number, number] } => {
  switch (mi) {
    case 0: return { mode: 6, a: [0, 0.05, 0.045, 0], b: [0, 0, 0, 0.35] };                                   // a distant seed of light
    case 1: return { mode: 1, a: [0, 0.44, 0, 0], b: [0, 0, 1 - smoothstep(0.25, 0.9, p), 1] };               // vital line -> flatline
    case 2: return { mode: 0, a: [0, 0, 0, 0], b: [0, 0, 0, 0] };                                             // comet carries it
    case 3: return { mode: 2, a: [0.3, 0.06, 0.42, 0], b: [0, 0, 0, 0.4] };                                   // the "0" halo
    case 4: return { mode: 6, a: [0, -0.03, 0.11 + 0.1 * smoothstep(0, 0.8, p), 0], b: [0, 0, 0, 0.5 + 0.5 * smoothstep(0, 0.6, p)] }; // forging core
    case 5: return { mode: 6, a: [0.3, 0.24, 0.12, 0], b: [0, 0, 0, 0.55] };                                  // moon
    case 6: return { mode: 2, a: [0, 0.14, 0.37, 0.55], b: [0, 0, 0, 1] };                                    // gold sun disc
    case 7: return { mode: 3, a: [0, -0.03, 0.30, 0], b: [0.16, 0, 0, 0.55] };                                // petal mandala
    case 8: return { mode: 6, a: [0, 0.04, 0.14 + 0.16 * smoothstep(0.55, 0.95, p), 0], b: [0, 0, 0, 0.25 + 0.75 * smoothstep(0.5, 0.95, p)] }; // the fruit
    case 9: return { mode: 9, a: [0, -0.02, 0.15, 0], b: [smoothstep(0.3, 1, p), 0, 0, 1] };                  // strings + cracks
    case 10: return { mode: 7, a: [0, 0, 0, 107.7], b: [2.3, 1, 0, 1] };                                      // three slashes converge
    case 11: return { mode: 0, a: [0, 0, 0, 0], b: [0, 0, 0, 0] };                                            // rain carries it
    case 12: return { mode: 5, a: [0, 0.02, 0.42, 0], b: [0.35 + 0.65 * smoothstep(0.08, 0.35, lt), 0, 0, 1] }; // breaking cracks
    case 13: return { mode: 8, a: [-0.2, 0.06, 0.2, 0], b: [smoothstep(11.5, 12.3, lt), 0, 0, 0.9] };         // the strike
    case 14: return { mode: 2, a: [0, 0.1, 0.44, 0.12], b: [0, 0, 0, 0.9] };                                  // hanamaru halo
    default: return { mode: 0, a: [0, 0, 0, 0], b: [0, 0, 0, 0] };
  }
};

const seasonPal = (lt: number): PalLive => {
  // four seasons across ~19.6s: 4.9s each, 0.7s cross-fades
  const order = ["spring", "summer", "autumn", "winter"];
  const seg = 4.9;
  const u = lt / seg;
  const i = Math.floor(u) % 4;
  const f = u % 1;
  const t = smoothstep(0.82, 1, f);
  return mixPal(order[i], order[(i + 1) % 4], t);
};

export const glStateAt = (t: number, frame: number, a: AudioFrame, rings: number[], mi: number, lt: number, p: number): GlState => {
  const pulse = a.pulseS;
  const s = S({});
  s.rings = rings;
  s.seed = mi * 17.31;
  const geo = geoFor(mi, lt, p);
  s.geo = geo.mode;
  s.geoA = geo.a;
  s.geoB = geo.b;
  s.pulseVal = Math.max(rings.length ? rings[1] : 0, a.beatPulse * 0.55);
  s.flashCol = [1, 1, 1];

  // base camera life (shared, deterministic)
  s.camX = 0;
  s.camY = 0;
  s.camZoom = 1.06 + 0.014 * pulse;

  switch (mi) {
    case 0: { // NULL — void
      s.pal = pal("null");
      s.neb = 0.18; s.nebScale = 1.6; s.flow = 0.35; s.stars = 0.15;
      s.focal = [0, 0.06]; s.focalR = 0.3; s.rays = 0.12;
      s.partMode = 7; s.partAmt = 0.05;
      s.partCol = s.pal.key;
      s.bloom = 0.45; s.chroma = 0.16; s.grain = 0.022; s.vig = 0.72; s.scan = 0.10; s.sharp = 0.22;
      s.ringGain = 0.9;
      s.camZoom = 1.16 - smoothstep(0, 6.2, lt) * 0.1;
      break;
    }
    case 1: { // BOOT
      s.pal = pal("boot");
      s.neb = 0.4; s.stars = 0.3; s.rays = 0.2;
      s.focal = [-0.35, 0.18]; s.focalR = 0.5;
      s.partMode = 7; s.partAmt = 0.07; s.partCol = s.pal.key;
      s.bloom = 0.5; s.streak = 0.22; s.chroma = 0.24; s.scan = 0.12; s.vig = 0.62; s.sharp = 0.3;
      s.flash = smoothstep(10.62, 10.88, t) * 0.65;
      break;
    }
    case 2: { // FAIRY
      s.pal = pal("fairy");
      s.neb = 0.55; s.nebScale = 2.6; s.flow = 1.4; s.stars = 1.0; s.rays = 0.15;
      s.focal = [0.25 * Math.cos(lt * 0.7), 0.1 + 0.06 * Math.sin(lt * 0.9)];
      s.focalR = 0.4;
      s.partMode = 7; s.partAmt = 0.14; s.partCol = s.pal.key;
      s.bloom = 0.68; s.streak = 0.28; s.chroma = 0.32; s.grain = 0.027; s.vig = 0.5; s.scan = 0.04;
      s.comet = 1; s.cometT0 = 10.95; s.ringGain = 0.12;
      break;
    }
    case 3: { // QUERY
      s.pal = pal("query");
      s.neb = 0.3; s.nebScale = 1.8; s.flow = 0.5; s.stars = 0.35; s.rays = 0.1;
      s.focal = [0, 0.02]; s.focalR = 0.36;
      s.partMode = 7; s.partAmt = 0.08; s.partCol = s.pal.key;
      s.bloom = 0.5; s.chroma = 0.2; s.grain = 0.033; s.vig = 0.6; s.scan = 0.08; s.sharp = 0.34;
      // if/else: colour seam late in the movement
      const k = smoothstep(0.62, 0.72, p) * (1 - smoothstep(0.96, 1, p));
      s.pal = k > 0 ? {
        ...s.pal,
        c1: mixc(s.pal.c1, [0.09, 0.05, 0.22], k), c2: mixc(s.pal.c2, [0.42, 0.1, 0.5], k),
        key: mixc(s.pal.key, [0.35, 0.95, 0.95], k * 0.7), alt: mixc(s.pal.alt, [1.0, 0.31, 0.6], k * 0.8),
        keyHex: "#59f0f0", altHex: "#ff4f9a",
      } : s.pal;
      s.rays = 0.1 + 0.25 * k;
      break;
    }
    case 4: { // FORGE
      s.pal = pal("forge");
      s.neb = 0.65; s.nebScale = 2.4; s.flow = 1.8; s.stars = 0.4; s.rays = 0.3;
      s.focal = [0, 0.12]; s.focalR = 0.34;
      s.partMode = 3; s.partAmt = 0.35 + 0.15 * pulse; s.partCol = s.pal.key; s.partCol2 = s.pal.alt;
      s.bloom = 0.72; s.streak = 0.28; s.chroma = 0.36; s.grain = 0.03; s.vig = 0.5; s.scan = 0.04; s.sharp = 0.34;
      s.camZoom = 1.06 + 0.02 * pulse;
      break;
    }
    case 5: { // ADMIT — first tear
      s.pal = pal("admit");
      s.neb = 0.5; s.nebScale = 2.8; s.flow = 0.8; s.stars = 0.6; s.rays = 0.22;
      s.focal = [0.22, 0.16]; s.focalR = 0.5;
      s.partMode = 5; s.partAmt = 0.05 + smoothstep(0.4, 0.9, p) * 0.12; s.partCol = s.pal.key;
      s.bloom = 0.58; s.streak = 0.22; s.chroma = 0.24; s.grain = 0.033; s.vig = 0.58; s.scan = 0.04;
      break;
    }
    case 6: { // REDACT
      s.pal = pal("redact");
      s.neb = 0.5; s.nebScale = 2.0; s.flow = 0.9; s.stars = 0.2; s.rays = 0.5;
      s.focal = [0, 0.08]; s.focalR = 0.55;
      s.partMode = 7; s.partAmt = 0.1; s.partCol = s.pal.key; s.partCol2 = [1, 0.4, 0.4];
      s.bloom = 0.68; s.streak = 0.36; s.chroma = 0.4; s.grain = 0.036; s.vig = 0.6; s.scan = 0.06;
      s.glitch = smoothstep(0.05, 0.3, lt) * (1 - smoothstep(0.7, 0.95, p)) * (0.35 + 0.4 * pulse);
      s.flash = a.hit * 0.12;
      s.flashCol = [1, 0.85, 0.5];
      break;
    }
    case 7: { // GARDEN
      s.pal = seasonPal(lt);
      s.neb = 0.55; s.nebScale = 2.6; s.flow = 1.0; s.stars = 0.3; s.rays = 0.25;
      s.focal = [0, 0.1]; s.focalR = 0.4;
      const sp = smoothstep(14.2, 15.0, lt);
      const seg = Math.floor(lt / 4.9) % 4;
      s.partMode = sp > 0 ? 8 : seg === 1 ? 8 : seg === 3 ? 4 : 2; // summer fireflies, winter snow, spring/autumn petals-leaves
      s.partAmt = 0.22 + 0.25 * sp; s.partCol = s.pal.key; s.partCol2 = s.pal.alt;
      s.bloom = 0.62; s.streak = 0.2; s.chroma = 0.24; s.grain = 0.03; s.vig = 0.52; s.scan = 0.03; s.sharp = 0.3;
      break;
    }
    case 8: { // ASCENT
      s.pal = mixPal("ascent", "eden", smoothstep(0.55, 0.85, p));
      s.neb = 0.6; s.nebScale = 3.0; s.flow = 2.2; s.stars = 0.9; s.rays = 0.3 + 0.3 * smoothstep(0.5, 0.9, p);
      s.focal = [0, 0.05]; s.focalR = 0.3 + 0.2 * smoothstep(0.55, 0.95, p);
      s.camZoom = 1.06 + smoothstep(0.0, 0.55, p) * 0.10 - smoothstep(0.6, 1, p) * 0.06;
      s.partMode = smoothstep(0.2, 0.5, p) > 0.5 ? 6 : 7;
      s.partAmt = 0.2 + 0.2 * smoothstep(0.2, 0.6, p); s.partCol = s.pal.key; s.partCol2 = s.pal.alt;
      s.bloom = 0.7; s.streak = 0.3; s.chroma = 0.32; s.grain = 0.03; s.vig = 0.5; s.scan = 0.04; s.sharp = 0.34;
      break;
    }
    case 9: { // DOLL
      const crack = smoothstep(0.3, 1, p);
      s.pal = pal("doll");
      s.neb = 0.28; s.nebScale = 1.8; s.flow = 0.4; s.stars = 0.12; s.rays = 0.08;
      s.focal = [0, 0.15]; s.focalR = 0.3;
      s.partMode = 7; s.partAmt = 0.06; s.partCol = s.pal.key;
      s.bloom = 0.52; s.streak = 0.18; s.chroma = 0.24 + 0.2 * crack; s.grain = 0.04; s.vig = 0.66; s.scan = 0.06; s.sharp = 0.34;
      s.glitch = crack * 0.25 * (0.5 + 0.5 * pulse);
      s.flash = a.hit * 0.1 * crack;
      s.flashCol = [1, 0.3, 0.35];
      break;
    }
    case 10: { // FIGHT
      s.pal = pal("fight");
      s.neb = 0.5; s.nebScale = 2.2; s.flow = 2.5; s.stars = 0.3; s.rays = 0.35;
      s.focal = [0, 0]; s.focalR = 0.3;
      s.partMode = 6; s.partAmt = 0.32 + 0.15 * pulse; s.partCol = s.pal.key; s.partCol2 = [1, 0.85, 0.7];
      s.bloom = 0.75; s.streak = 0.38; s.chroma = 0.36; s.grain = 0.033; s.vig = 0.55; s.scan = 0.04; s.sharp = 0.36;
      s.camZoom = 1.06 + 0.05 * smoothstep(0.55, 0.9, p);
      const imp = smoothstep(2.5, 2.56, lt) * (1 - smoothstep(2.56, 3.05, lt));
      s.flash = imp * 0.5 + a.hit * 0.08;
      s.glitch = imp * 0.35;
      break;
    }
    case 11: { // TEARFALL
      const err = smoothstep(13.5, 15.5, lt);
      s.pal = err > 0 ? mixPal("tearfall", "overflow", err * 0.8) : pal("tearfall");
      s.neb = 0.5; s.nebScale = 2.6; s.flow = 1.2; s.stars = 0.25; s.rays = 0.2;
      s.focal = [0, 0.2]; s.focalR = 0.42;
      s.partMode = 5; s.partAmt = 0.5 + 0.25 * pulse; s.partCol = s.pal.key; s.partCol2 = [1, 1, 1];
      s.bloom = 0.68; s.streak = 0.25; s.chroma = 0.3 + 0.22 * err; s.grain = 0.033; s.vig = 0.55; s.scan = 0.05; s.sharp = 0.28;
      s.glitch = err * (0.4 + 0.35 * pulse);
      s.camZoom = 1.06 + 0.018 * pulse;
      break;
    }
    case 12: { // BREAK
      s.pal = pal("break");
      s.neb = 0.45; s.nebScale = 2.0; s.flow = 0.8; s.stars = 0.2; s.rays = 0.3;
      s.focal = [0, 0.05]; s.focalR = 0.4;
      s.partMode = 6; s.partAmt = 0.28 * smoothstep(0.08, 0.2, lt) * (1 - smoothstep(0.8, 1, p));
      s.partCol = [1, 0.9, 0.85]; s.partCol2 = s.pal.key;
      s.bloom = 0.78; s.streak = 0.33; s.chroma = 0.4 * smoothstep(0.08, 0.25, lt); s.grain = 0.04; s.vig = 0.5; s.scan = 0.04; s.sharp = 0.34;
      s.glitch = smoothstep(0.06, 0.2, lt) * (1 - smoothstep(0.35, 0.6, lt)) * 0.7;
      s.flash = smoothstep(0.06, 0.14, lt) * (1 - smoothstep(0.14, 0.6, lt)) * 0.5;
      s.flashCol = [1, 0.95, 0.9];
      break;
    }
    case 13: { // SOURCE
      s.pal = pal("source");
      s.neb = 0.3; s.nebScale = 1.9; s.flow = 0.6; s.stars = 0.15; s.rays = 0.1;
      s.focal = [0, 0.1]; s.focalR = 0.32;
      s.partMode = 7; s.partAmt = 0.08; s.partCol = s.pal.key;
      s.bloom = 0.55; s.streak = 0.2; s.chroma = 0.22; s.grain = 0.033; s.vig = 0.6; s.scan = 0.07;
      break;
    }
    case 14: { // VERDICT
      s.pal = pal("verdict");
      s.neb = 0.6; s.nebScale = 2.2; s.flow = 0.8; s.stars = 0.5; s.rays = 0.45;
      s.focal = [0, 0.12]; s.focalR = 0.5;
      s.partMode = 2; s.partAmt = 0.24 + 0.12 * smoothstep(3.5, 5.5, lt); s.partCol = s.pal.key; s.partCol2 = s.pal.alt;
      s.bloom = 0.72; s.streak = 0.28; s.chroma = 0.28; s.grain = 0.033; s.vig = 0.5; s.scan = 0.03; s.sharp = 0.3;
      const stamp = smoothstep(4.0, 4.12, lt) * (1 - smoothstep(4.12, 4.6, lt));
      s.flash = stamp * 0.55;
      s.flashCol = [1, 0.9, 0.7];
      s.camZoom = 1.06 + stamp * 0.05;
      break;
    }
  }

  // global fade in/out + end fade
  s.fade = smoothstep(0.4, 2.4, t) * (1 - smoothstep(181.2, 182.8, t));
  return s;
};
