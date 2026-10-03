// ---------------------------------------------------------------------------
// structure.js - the machine vocabulary.
//
// Everything here draws into the additive 2D text layer as vector art.  Since
// the GL bloom pass already runs on the whole scene, strokes come out glowing
// for free, and (after shadowBlur was removed) vector drawing is cheap enough
// to spend a few hundred paths per frame.
//
// The intent: CRYMACHINA's Eden is a dyson-scale engineering object and ENOA is
// a machine that cradles human souls.  So the recurring shapes are rings,
// reticles, circuit traces, cradles and swarms - not a literal cartoon figure.
// ---------------------------------------------------------------------------

import { TAU, clamp, lerp, smoothstep, ease, hash2, hash3, noise1, gauss } from "./core.js";
import { ALLGLYPH, KATA } from "./type.js";

export function hexA(c, a) {
  return `rgba(${Math.round(clamp(c[0]) * 255)},${Math.round(clamp(c[1]) * 255)},${Math.round(clamp(c[2]) * 255)},${(+a).toFixed(3)})`;
}

// --------------------------------------------------------------- the rig ----

/**
 * Eden's ring rig: concentric tilted rings with tick marks, running nodes and
 * a hub.  The backbone of most scenes.
 *   o = { rings, r, squash, ang, rot, tick, node, alpha, lw, hub, colFlip }
 */
export function ringRig(c, x, y, t, col, col2, o = {}) {
  const rings = o.rings ?? 5;
  const r0 = o.r0 ?? 150;
  const r1 = o.r1 ?? 620;
  const alpha = o.alpha ?? 0.5;
  const rot = (o.rot ?? 0) + t * (o.spin ?? 0.05);
  const squash = o.squash ?? 0.34;
  const lw = o.lw ?? 1.6;

  c.save();
  c.translate(x, y);
  c.lineCap = "round";

  for (let i = 0; i < rings; i++) {
    const u = rings === 1 ? 0 : i / (rings - 1);
    const r = lerp(r0, r1, u);
    const tilt = squash + 0.30 * Math.sin(i * 1.7 + t * 0.11 + (o.phase ?? 0));
    const a = alpha * (0.35 + 0.65 * (1 - Math.abs(u - 0.45) * 1.5)) * (o.gain ?? 1);
    if (a <= 0.004) continue;

    c.strokeStyle = hexA(i % 2 ? col2 : col, a);
    c.lineWidth = lw * (1 - u * 0.45);
    c.beginPath();
    c.ellipse(0, 0, r, r * tilt, rot * (i % 2 ? 1 : -0.6) + i * 0.5, 0, TAU);
    c.stroke();

    // tick marks: the ring reads as a graduated instrument
    if (o.tick !== 0) {
      const nt = o.tick ?? Math.max(10, Math.round(r / 26));
      const tickLen = (o.tickLen ?? 9) * (1 + 0.5 * u);
      c.strokeStyle = hexA(col, a * 0.75);
      c.lineWidth = lw * 0.8;
      c.beginPath();
      for (let k = 0; k < nt; k++) {
        const th = (k / nt) * TAU + rot * (i % 2 ? 1 : -0.6);
        const ct = Math.cos(th), stt = Math.sin(th) * tilt;
        const px = ct * r, py = stt * r;
        const nx = ct * (r + tickLen), ny = stt * (r + tickLen);
        c.moveTo(px, py); c.lineTo(nx, ny);
      }
      c.stroke();
    }

    // travelling node: a dot that runs the ring, brightest on accent hits
    if (o.node !== 0 && r > r0) {
      const speed = 0.55 + hash2(i, 91) * 0.9;
      const th = (t * speed + hash2(i, 17) * TAU) * (i % 2 ? -1 : 1);
      const ct = Math.cos(th), stt = Math.sin(th) * tilt;
      const px = ct * r, py = stt * r;
      const g = c.createRadialGradient(px, py, 0, px, py, 26);
      g.addColorStop(0, hexA(col2, a * 1.6));
      g.addColorStop(0.35, hexA(col, a * 0.7));
      g.addColorStop(1, hexA(col, 0));
      c.fillStyle = g;
      c.beginPath(); c.arc(px, py, 26, 0, TAU); c.fill();
      c.fillStyle = hexA([1, 1, 1], a * 1.3);
      c.beginPath(); c.arc(px, py, 2.6, 0, TAU); c.fill();
    }
  }

  // hub: a bright core with reticle arms. Kept well under 1.0 because the bloom
  // pyramid amplifies it and a saturated disc swallows everything near it.
  if (o.hub !== 0) {
    const hubR = o.hubR ?? 42;
    const pulse = o.hubPulse ?? 0;
    const g = c.createRadialGradient(0, 0, 0, 0, 0, hubR * (1 + pulse * 0.5));
    g.addColorStop(0, hexA(col2, alpha * 0.80));
    g.addColorStop(0.22, hexA(col, alpha * 0.70));
    g.addColorStop(1, hexA(col, 0));
    c.fillStyle = g;
    c.beginPath(); c.arc(0, 0, hubR * (1 + pulse * 0.5), 0, TAU); c.fill();

    c.strokeStyle = hexA(col2, alpha * 0.9);
    c.lineWidth = lw;
    c.beginPath();
    for (let k = 0; k < 4; k++) {
      const th = (k / 4) * TAU + rot;
      c.moveTo(Math.cos(th) * hubR * 0.28, Math.sin(th) * hubR * 0.28);
      c.lineTo(Math.cos(th) * hubR * 1.25, Math.sin(th) * hubR * 1.25);
    }
    c.stroke();
  }
  c.restore();
}

/** a thin cross-hair reticle - the machine's "eye" on the subject */
export function reticle(c, x, y, r, t, col, o = {}) {
  const a = o.alpha ?? 0.5;
  const spin = (o.spin ?? 0.12) * t;
  const gap = o.gap ?? 0.22;
  c.save();
  c.translate(x, y);
  c.rotate(spin);
  c.strokeStyle = hexA(col, a);
  c.lineWidth = o.lw ?? 1.4;
  c.beginPath();
  for (let k = 0; k < 4; k++) {
    c.save(); c.rotate((k / 4) * TAU);
    c.moveTo(0, -r * (1 - gap)); c.lineTo(0, -r);
    c.moveTo(0, -r); c.lineTo(r * 0.10, -r * 0.90);
    c.restore();
  }
  c.stroke();
  if (o.inner) {
    c.strokeStyle = hexA(col, a * 0.6);
    c.beginPath(); c.arc(0, 0, r * 0.52, 0, TAU); c.stroke();
  }
  c.restore();
}

/** orthogonal circuit traces that radiate from a point and light up in waves */
export function circuitTraces(c, ox, oy, t, col, o = {}) {
  const n = o.n ?? 14;
  const len = o.len ?? 380;
  const a0 = o.alpha ?? 0.38;
  const seed = o.seed ?? 0;
  const seg = o.seg ?? 3;
  c.save();
  c.translate(ox, oy);
  c.lineCap = "square";
  for (let i = 0; i < n; i++) {
    const th = (i / n) * TAU + (hash2(seed + i, 5) - 0.5) * 0.34;
    const dirx = Math.cos(th), diry = Math.sin(th);
    // orthogonal manhattan path: alternate axis-aligned runs
    let px = 0, py = 0;
    const pts = [[0, 0]];
    let travelled = 0;
    const total = len * (0.45 + 0.85 * hash2(seed + i, 13));
    for (let s = 0; s < seg; s++) {
      const step = total / seg * (0.5 + hash2(seed + i * 7 + s, 23));
      travelled += step;
      if (s % 2 === 0) px += dirx * step; else py += diry * step;
      pts.push([px, py]);
    }
    const wave = (t * 0.75 + hash2(seed + i, 31)) % 1.6;
    const lit = clamp((wave - 0.0) / 0.5) * (1 - clamp((wave - 0.9) / 0.7));
    const a = a0 * (0.30 + 0.70 * lit);
    if (a < 0.01) continue;
    c.strokeStyle = hexA(col, a);
    c.lineWidth = o.lw ?? 1.8;
    c.beginPath();
    c.moveTo(pts[0][0], pts[0][1]);
    for (let k = 1; k < pts.length; k++) c.lineTo(pts[k][0], pts[k][1]);
    c.stroke();
    // terminal pad
    const [ex, ey] = pts[pts.length - 1];
    c.fillStyle = hexA(col, a * 1.4);
    c.beginPath(); c.arc(ex, ey, 3 + 3 * lit, 0, TAU); c.fill();
  }
  c.restore();
}

/** many small bodies on elliptical orbits, with motion trails */
export function orbitSwarm(c, x, y, t, col, o = {}) {
  const n = o.n ?? 90;
  const r0 = o.r0 ?? 120, r1 = o.r1 ?? 560;
  const a0 = o.alpha ?? 0.55;
  const squash = o.squash ?? 0.42;
  const seed = o.seed ?? 0;
  c.save();
  c.translate(x, y);
  for (let i = 0; i < n; i++) {
    const rr = lerp(r0, r1, Math.pow(hash2(seed + i, 3), 0.7));
    const tilt = squash * (0.6 + 0.7 * hash2(seed + i, 11));
    const sp = (0.14 + 0.5 * hash2(seed + i, 19)) * (hash2(seed + i, 23) > 0.5 ? 1 : -1);
    const th = t * sp + hash2(seed + i, 29) * TAU;
    const px = Math.cos(th) * rr, py = Math.sin(th) * rr * tilt;
    const size = 1.2 + 2.2 * hash2(seed + i, 37);
    const a = a0 * (0.35 + 0.65 * hash2(seed + i, 41));
    c.fillStyle = hexA(col, a * 0.5);
    c.beginPath(); c.arc(px, py, size * 2.6, 0, TAU); c.fill();
    c.fillStyle = hexA(col, a);
    c.beginPath(); c.arc(px, py, size, 0, TAU); c.fill();
  }
  c.restore();
}

/** a vertical data spine: stacked glyph bars that stream toward the hub */
export function dataSpine(c, x, y0, y1, t, col, o = {}) {
  const n = o.n ?? 26;
  const w = o.w ?? 42;
  const a0 = o.alpha ?? 0.4;
  const speed = o.speed ?? 0.25;
  c.save();
  c.lineCap = "butt";
  for (let i = 0; i < n; i++) {
    const u = (i / n + t * speed) % 1;
    const y = lerp(y1, y0, u);
    const h = (y1 - y0) / n * (0.3 + 0.6 * hash2(i, 7));
    const a = a0 * (0.2 + 0.8 * hash2(i, 13)) * smoothstep(0, 0.14, u) * (1 - smoothstep(0.86, 1, u));
    if (a < 0.01) continue;
    const ww = w * (0.35 + 0.65 * hash2(i, 19));
    c.fillStyle = hexA(col, a);
    c.fillRect(x - ww / 2, y, ww, h);
  }
  c.restore();
}

// ------------------------------------------------------------- the cradle ---
/**
 * ENOA's cradle: two arcs holding a glowing core, with data pouring in.
 * This is the film's stand-in for "a machine that keeps a human soul alive".
 */
export function corePod(c, x, y, r, t, col, col2, o = {}) {
  const a0 = o.alpha ?? 0.8;
  const pulse = o.pulse ?? 0;
  const open = o.open ?? 0.55;
  c.save();
  c.translate(x, y);

  // outer shell arcs (an open cocoon)
  c.lineCap = "round";
  for (const dir of [-1, 1]) {
    c.strokeStyle = hexA(col, a0 * 0.75);
    c.lineWidth = o.lw ?? 3.2;
    c.beginPath();
    c.ellipse(0, 0, r * 1.02, r * 1.18, 0,
              dir < 0 ? Math.PI * 0.62 : Math.PI * 0.62 + Math.PI * open,
              dir < 0 ? Math.PI * 0.62 + Math.PI * (1 - open) : Math.PI * 2.62 - Math.PI * (1 - open));
    c.stroke();
  }
  // ribs
  c.strokeStyle = hexA(col, a0 * 0.45);
  c.lineWidth = (o.lw ?? 3.2) * 0.5;
  c.beginPath();
  for (let i = 0; i <= 6; i++) {
    const u = i / 6;
    const th = Math.PI * (0.62 + u * (1 - open)) * 1.0;
    for (const s of [-1, 1]) {
      c.moveTo(s * Math.cos(th) * r * 0.80, Math.sin(th) * r * 0.94);
      c.lineTo(s * Math.cos(th) * r * 1.04, Math.sin(th) * r * 1.20);
    }
  }
  c.stroke();

  // the core - kept tight, because with additive blending an oversized white
  // blob simply erases whatever is drawn over it
  const cr = r * (0.30 + 0.06 * pulse);
  const g = c.createRadialGradient(0, 0, 0, 0, 0, cr * 2.3);
  g.addColorStop(0, hexA([1, 1, 1], a0 * 0.60));
  g.addColorStop(0.14, hexA(col2, a0 * 0.55));
  g.addColorStop(0.42, hexA(col, a0 * 0.28));
  g.addColorStop(1, hexA(col, 0));
  c.fillStyle = g;
  c.beginPath(); c.arc(0, 0, cr * 2.3, 0, TAU); c.fill();

  // containment rings
  c.strokeStyle = hexA(col2, a0 * 0.8);
  c.lineWidth = (o.lw ?? 3.2) * 0.7;
  for (let i = 0; i < 3; i++) {
    const rr = cr * (1.5 + i * 0.5);
    const tilt = 0.24 + 0.34 * i;
    const rot = t * (0.5 + i * 0.35) * (i % 2 ? -1 : 1);
    c.beginPath(); c.ellipse(0, 0, rr, rr * tilt, rot, 0, TAU); c.stroke();
  }
  c.restore();
}

// -------------------------------------------------------------- the figure --
/**
 * A stylised mechanical figure drawn as a wireframe.  Deliberately abstract and
 * faint: at this scale a bold outline reads as a cartoon, a thin one with
 * visible joint articulation reads as engineering.
 */
export function mechFigure(c, x, y, h, t, col, o = {}) {
  const s = h / 400;
  const a = o.alpha ?? 0.8;
  const lw = (o.lw ?? 2.0) / s;
  const stroke = typeof col === "string" ? col : hexA(col, 1);
  c.save();
  c.translate(x, y);
  c.scale(s, s);
  if (o.rot) c.rotate(o.rot);
  c.globalAlpha = a;
  c.strokeStyle = stroke;
  c.lineWidth = lw;
  c.lineJoin = "round";
  c.lineCap = "round";

  const breathe = o.breathe ? 1 + 0.012 * Math.sin(t * 1.6) : 1;
  c.scale(breathe, breathe);
  const sway = Math.sin(t * 0.9) * 0.03;
  c.rotate(sway);

  const P = (px, py) => [px, py];

  // --- head: a narrow visor helm --------------------------------------------
  c.beginPath();
  c.moveTo(-19, -316);
  c.quadraticCurveTo(-23, -372, 0, -378);
  c.quadraticCurveTo(23, -372, 19, -316);
  c.quadraticCurveTo(10, -300, 0, -300);
  c.quadraticCurveTo(-10, -300, -19, -316);
  c.closePath();
  c.stroke();
  // visor slit
  c.beginPath(); c.moveTo(-13, -334); c.lineTo(13, -334); c.stroke();
  // halo: a floating arc with a gap, not a bar welded to the crown
  c.beginPath();
  c.ellipse(0, -394, 31, 10, 0, Math.PI * 1.06, Math.PI * 1.94);
  c.stroke();

  // --- hair: two long locks with a hard tip ---------------------------------
  c.beginPath();
  c.moveTo(-21, -312); c.quadraticCurveTo(-50, -232, -38, -146);
  c.moveTo(21, -312); c.quadraticCurveTo(50, -232, 38, -146);
  c.stroke();

  // --- neck + clavicle -------------------------------------------------------
  c.beginPath();
  c.moveTo(-9, -300); c.lineTo(-9, -276);
  c.moveTo(9, -300); c.lineTo(9, -276);
  c.moveTo(-40, -268); c.quadraticCurveTo(0, -284, 40, -268);
  c.stroke();

  // --- torso: an hourglass, not a barrel ------------------------------------
  c.beginPath();
  c.moveTo(-40, -268);
  c.quadraticCurveTo(-58, -210, -46, -150);   // ribcage in
  c.quadraticCurveTo(-52, -110, -60, -60);    // waist out to hips
  c.lineTo(-56, 30);
  c.quadraticCurveTo(0, 52, 56, 30);
  c.lineTo(60, -60);
  c.quadraticCurveTo(52, -110, 46, -150);
  c.quadraticCurveTo(58, -210, 40, -268);
  c.closePath();
  c.stroke();
  // sternum + core
  c.beginPath(); c.moveTo(0, -256); c.lineTo(0, -60); c.stroke();
  c.beginPath(); c.arc(0, -178, 13, 0, TAU); c.stroke();
  c.beginPath(); c.arc(0, -178, 4.5, 0, TAU); c.stroke();

  // --- shoulders: pauldrons --------------------------------------------------
  for (const sgn of [-1, 1]) {
    c.beginPath();
    c.ellipse(sgn * 52, -256, 26, 15, sgn * 0.3, 0, TAU);
    c.stroke();
  }

  // --- arms with elbow joints ------------------------------------------------
  for (const sgn of [-1, 1]) {
    const ex = sgn * 74, ey = -160, wx = sgn * 66, wy = -34;
    c.beginPath();
    c.moveTo(sgn * 52, -250); c.lineTo(ex, ey);
    c.moveTo(ex, ey);
    if (o.reach) c.quadraticCurveTo(sgn * 96, ey + 60, sgn * 40, wy - 30);
    else c.lineTo(wx, wy);
    c.stroke();
    c.beginPath(); c.arc(ex, ey, 5.5, 0, TAU); c.stroke();
    c.beginPath(); c.arc(wx, wy, 4.5, 0, TAU); c.stroke();
  }

  // --- hips + legs -----------------------------------------------------------
  c.beginPath(); c.moveTo(-56, 30); c.quadraticCurveTo(0, 44, 56, 30); c.stroke();
  for (const sgn of [-1, 1]) {
    const kx = sgn * 26, ky = 120;
    c.beginPath();
    c.moveTo(sgn * 26, 34); c.lineTo(kx, ky); c.lineTo(sgn * 20, 250);
    c.stroke();
    c.beginPath(); c.arc(kx, ky, 5, 0, TAU); c.stroke();
  }

  // --- skirt / thruster flare (optional) ------------------------------------
  if (o.flare) {
    c.beginPath();
    c.moveTo(-56, 20);
    c.quadraticCurveTo(-110, 90, -78, 168);
    c.moveTo(56, 20);
    c.quadraticCurveTo(110, 90, 78, 168);
    c.stroke();
  }
  c.restore();
}

// ---------------------------------------------------------------- ambience --

/** a horizon of light columns, like a server cathedral */
export function columnHall(c, W, H, horizonY, t, col, o = {}) {
  const n = o.n ?? 15;
  const a0 = o.alpha ?? 0.28;
  c.save();
  for (let i = 0; i < n; i++) {
    const u = (i + 0.5) / n;
    const depth = 0.35 + 0.65 * hash2(i + (o.seed ?? 0), 3);
    // skip some bays so the hall is not a picket fence
    if (hash2(i + (o.seed ?? 0), 41) < 0.18) continue;
    const x = W * (0.05 + u * 0.90) + (hash2(i, 11) - 0.5) * 34;
    // a slow travelling swells the hall so a 24-second act is never a still frame
    const swell = 0.55 + 0.45 * Math.sin(u * 5.2 - t * 0.42 + (o.seed ?? 0));
    const hgt = (H * 0.78) * depth * (0.45 + 0.75 * swell);
    const wd = 3 + 15 * depth;
    const y = horizonY - hgt;
    const flick = 0.55 + 0.45 * noise1(t * 0.9 + i * 3.1, 5);
    const g = c.createLinearGradient(0, y, 0, horizonY);
    // fade at the top so the columns dissolve into the dark instead of ending
    g.addColorStop(0, hexA(col, 0));
    g.addColorStop(0.18, hexA(col, a0 * 0.55 * flick));
    g.addColorStop(0.72, hexA(col, a0 * 0.26 * flick));
    g.addColorStop(1, hexA(col, 0));
    c.fillStyle = g;
    c.fillRect(x - wd / 2, y, wd, hgt);
    c.fillStyle = hexA(col, a0 * 0.40 * flick);
    c.fillRect(x - wd / 2, y, wd, 2.5);
  }
  c.restore();
}

/** a receding perspective floor of glyph-free light tiles */
export function lightFloor(c, W, H, horizonY, t, col, o = {}) {
  const rows = o.rows ?? 26;
  const a0 = o.alpha ?? 0.18;
  c.save();
  c.strokeStyle = hexA(col, a0);
  c.lineWidth = 1.2;
  for (let r = 1; r <= rows; r++) {
    const u = r / rows;
    const y = horizonY + (H - horizonY) * Math.pow(u, 2.1) * 1.25;
    if (y > H + 4) break;
    const a = a0 * (0.25 + 0.75 * u);
    c.strokeStyle = hexA(col, a * (0.7 + 0.3 * noise1(t * 1.2 + r * 0.6, 9)));
    c.beginPath(); c.moveTo(-40, y); c.lineTo(W + 40, y); c.stroke();
  }
  // radial lane lines toward the vanishing point
  const vpx = W / 2, vpy = horizonY;
  for (let i = -9; i <= 9; i++) {
    const u = i / 9;
    const a = a0 * (1 - Math.abs(u) * 0.55);
    c.strokeStyle = hexA(col, a);
    c.beginPath();
    c.moveTo(vpx + u * 160, vpy);
    c.lineTo(vpx + u * (W * 1.5), H + 40);
    c.stroke();
  }
  c.restore();
}

/** a dense field of code glyphs as a background texture (cheap, very on-theme) */
export function glyphMatrix(c, W, H, t, col, o = {}) {
  const chars = o.chars || ALLGLYPH;
  const cols = o.cols ?? 48;
  const rows = o.rows ?? 27;
  const size = o.size ?? 16;
  const a0 = o.alpha ?? 0.10;
  const dx = W / cols, dy = H / rows;
  const cx = o.cx ?? W / 2, cy = o.cy ?? H / 2;
  const maxR = Math.hypot(W / 2, H / 2);
  const seed = o.seed ?? 0;
  c.save();
  c.font = `${o.weight ?? 400} ${size}px "Cascadia Mono","Consolas",monospace`;
  c.textAlign = "center";
  c.textBaseline = "middle";
  // a reveal band sweeps through so the wall is never uniformly lit
  const band = (t * 0.085) % 1.55;
  for (let r = 0; r < rows; r++) {
    for (let k = 0; k < cols; k++) {
      const x = k * dx + dx / 2, y = r * dy + dy / 2;
      // radial falloff: the field should read as atmosphere, not wallpaper
      const rad = Math.hypot(x - cx, y - cy) / maxR;
      const fall = clamp(1 - rad * rad * 0.92);
      // whole columns drop out for a while, which breaks up the grid
      const colGate = hash2(k * 13 + seed, 71) > 0.24 ? 1 : 0.12;
      const w = ((k / cols) * 0.72 + band) % 1.5;
      const lit = clamp((w - 0.34) / 0.24) * (1 - clamp((w - 0.74) / 0.30));
      const base = 0.10 + 0.24 * hash2(k * 31 + r * 7 + seed, 11);
      const a = a0 * fall * colGate * (base + 1.15 * lit);
      if (a < 0.012) continue;
      const idx = Math.floor(hash2(k * 131 + r * 17 + Math.floor(t * 6) + seed, 23) * chars.length);
      c.fillStyle = hexA(col, a);
      c.fillText(chars[idx % chars.length], x, y);
    }
  }
  c.restore();
}

// ------------------------------------------------------------- dispatcher ---

/**
 * Draw a scene's structural backdrop.  `rig` is one descriptor or an array of
 * them, e.g.
 *   rig: [ {kind:"matrix", alpha:0.13}, {kind:"rings", r0:200, r1:760} ]
 * Scenes declare this in the TIMELINE so every act gets its own scaffolding
 * without repeating boilerplate in 19 scene functions.
 */
export function drawStructure(c, W, H, t, pal, rig, cx, cy) {
  const list = Array.isArray(rig) ? rig : [rig];
  for (const r of list) {
    if (!r || r.kind === "none") continue;
    const col = r.col === "alt" ? pal.alt : r.col === "hot" ? pal.hot : pal.key;
    const col2 = r.col2 === "alt" ? pal.alt : col;
    const X = r.x ?? cx, Y = r.y ?? cy;
    switch (r.kind) {
      case "rings":  ringRig(c, X, Y, t, col, col2, r); break;
      case "reticle": reticle(c, X, Y, r.r ?? 220, t, col, r); break;
      case "traces": circuitTraces(c, X, Y, t, col, r); break;
      case "swarm":  orbitSwarm(c, X, Y, t, col, r); break;
      case "pod":    corePod(c, X, Y, r.r ?? 190, t, col, col2, r); break;
      case "spine":  dataSpine(c, X, Y, r.y1 ?? Y + 420, t, col, r); break;
      case "figure": mechFigure(c, X, Y, r.h ?? 320, t, col, r); break;
      case "floor":  lightFloor(c, W, H, r.horizon ?? H * 0.66, t, col, r); break;
      case "hall":   columnHall(c, W, H, r.horizon ?? H * 0.78, t, col, r); break;
      case "glyphs": glyphMatrix(c, W, H, t, col, r); break;
      default: break;
    }
  }
}
