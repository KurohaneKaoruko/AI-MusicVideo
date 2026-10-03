// Attribute image softness to a specific stage of the pipeline.
//
//   node tools/sharp.mjs --t 129 --crop 1400:300:260:400
//
// Renders the SAME frame several ways and crops the region 1:1 (no rescaling,
// which would hide exactly the thing we are measuring):
//
//   0 default                  - what the film ships
//   1 bloom = 0                - is the halation veil the culprit?
//   2 chroma = 0               - is the RGB fringing the culprit?
//   3 grain/scan/vignette = 0  - is the screen-space texture the culprit?
//   4 no text layer            - how much of the "sharp" content is text at all
//   5 the shipped mp4 frame    - how much is the H.264 4:2:0 encode
//
// Alongside the picture it prints a gradient-energy number per variant: mean
// |dL/dx| over the crop. Sharper edges => higher number. That turns "looks
// blurry" into something falsifiable.
import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";
import { findFFmpeg } from "./ffmpeg.mjs";
import { spawn, spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync, existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const W = 1920, H = 1080, FRAME_BYTES = W * H * 4;
const DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(DIR, "..");
const FPS = 60;

const argv = process.argv.slice(2);
let tAt = 129, crop = [1400, 300, 260, 400], mp4 = null;
for (let i = 0; i < argv.length; i++) {
  if (argv[i] === "--t") tAt = Number(argv[++i]);
  else if (argv[i] === "--crop") crop = argv[++i].split(":").map(Number);
  else if (argv[i] === "--mp4") mp4 = argv[++i];
}
if (!mp4) {
  const cand = [
    path.join(ROOT, "..", "dist", "NotToNotice - Enoa [v2 1080p60].mp4"),
    path.join(ROOT, "dist", "NotToNotice - Enoa [v2 1080p60].mp4"),
  ];
  mp4 = cand.find(existsSync) || null;
}

const VARIANTS = process.argv.includes("--sweep") ? [
  { name: "0 old bloom (noTrim)", dbg: { noTrim: true } },
  { name: "1 x0.50 max0.70 th0.55", dbg: { trimScale: 0.50, trimMax: 0.70, trimThresh: 0.55 } },
  { name: "2 x0.35 max0.55 th0.65", dbg: {} },                       // shipped
  { name: "3 x0.30 max0.45 th0.72", dbg: { trimScale: 0.30, trimMax: 0.45, trimThresh: 0.72 } },
  { name: "4 x0.22 max0.35 th0.78", dbg: { trimScale: 0.22, trimMax: 0.35, trimThresh: 0.78 } },
  { name: "5 bloom = 0 (lower bound)", dbg: { bloomOff: true } },
] : [
  { name: "0 shipped", dbg: {} },
  { name: "1 old bloom (noTrim)", dbg: { noTrim: true } },
  { name: "2 bloom = 0", dbg: { bloomOff: true } },
  { name: "3 chroma = 0", dbg: { chromaOff: true } },
  { name: "4 no text layer", dbg: { noText: true } },
];

const BASE = path.join(ROOT, ".sharp");
mkdirSync(BASE, { recursive: true });
let runIdx = 1;
while (existsSync(path.join(BASE, "r" + runIdx))) runIdx++;
const OUT = path.join(BASE, "r" + runIdx);
mkdirSync(OUT, { recursive: true });

// --- capture ---------------------------------------------------------------
const frames = [];
const srv = await startServer(ROOT, {
  onFrame: (buf) => {
    if (buf.length !== FRAME_BYTES) throw new Error("bad frame " + buf.length);
    frames.push(Buffer.from(buf));
    return true;
  },
});
const page = await launch({ url: srv.url("/renderer/index.html"), headless: true, width: W, height: H });
await page.eval("new Promise(r => { const k = () => (window.__ready ? r(1) : setTimeout(k, 60)); k(); })");
await page.eval("window.__capStart()");
const U = JSON.stringify(srv.url("/_frame"));
const F = Math.round(tAt * FPS);

for (const v of VARIANTS) {
  await page.eval(`(()=>{ const d=window.__dbg;
    for (const k of Object.keys(d)) delete d[k];
    Object.assign(d, ${JSON.stringify(v.dbg)});
    return true; })()`);
  const r = await page.eval(`window.__captureRaw(${F}, ${U})`);
  if (r !== true) throw new Error(`${v.name}: ${r}`);
}
await page.close();
await srv.close();

// --- objective sharpness per variant --------------------------------------
// mean |dL/dx| inside the crop, ignoring near-black background so that empty
// scenes do not flatter themselves.
const [cw, ch, cx, cy] = crop;
function gradEnergy(buf) {
  let sum = 0, n = 0;
  for (let y = cy; y < cy + ch; y++) {
    let prev = null;
    for (let x = cx; x < cx + cw; x++) {
      const o = (y * W + x) * 4;
      const l = 0.2126 * buf[o] + 0.7152 * buf[o + 1] + 0.0722 * buf[o + 2];
      if (prev !== null && l > 12 && prev > 12) { sum += Math.abs(l - prev); n++; }
      prev = l;
    }
  }
  return n ? sum / n : 0;
}

const ff = findFFmpeg();
const run = (args) => new Promise((res, rej) => {
  const p = spawn(ff, args, { stdio: ["ignore", "ignore", "inherit"] });
  p.on("close", (c) => (c === 0 ? res() : rej(new Error("ffmpeg " + c))));
});

// write the GL variants out as PNGs
const pngs = [];
for (let i = 0; i < frames.length; i++) {
  const raw = path.join(OUT, `v${i}.rgba`);
  writeFileSync(raw, frames[i]);
  const png = path.join(OUT, `v${i}.png`);
  await run(["-hide_banner", "-loglevel", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgba",
    "-s", `${W}x${H}`, "-i", raw, png]);
  pngs.push(png);
}

console.log(`\nt = ${tAt}s (frame ${F})   crop ${cw}x${ch}+${cx}+${cy}\n`);
for (let i = 0; i < VARIANTS.length; i++) {
  console.log(`  ${VARIANTS[i].name.padEnd(28)} grad = ${gradEnergy(frames[i]).toFixed(3)}`);
}

// --- the shipped mp4, same frame ------------------------------------------
const inputs = [...pngs];
if (mp4) {
  const mp4png = path.join(OUT, "mp4.png");
  await run(["-hide_banner", "-loglevel", "error", "-y", "-i", mp4,
    "-vf", `select=eq(n\\,${F})`, "-frames:v", "1", mp4png]);
  const buf = pngToRgba(mp4png);
  inputs.push(mp4png);
  if (buf) console.log(`  ${"5 shipped mp4 (H.264)".padEnd(28)} grad = ${gradEnergy(buf).toFixed(3)}`);
}

// --- stack the crops 1:1 ---------------------------------------------------
const parts = inputs.map((_, i) => `[${i}:v]crop=${cw}:${ch}:${cx}:${cy}[r${i}]`).join(";");
const stack = inputs.map((_, i) => `[r${i}]`).join("") + `vstack=inputs=${inputs.length}[v]`;
const out = path.join(OUT, "compare.png");
await run(["-hide_banner", "-loglevel", "error", "-y",
  ...inputs.flatMap((p) => ["-i", p]),
  "-filter_complex", parts + ";" + stack,
  "-map", "[v]", out]);

console.log(`\n  rows top->bottom: ${VARIANTS.map((v) => v.name).join(" | ")}${mp4 ? " | 5 shipped mp4" : ""}`);
console.log(`  ${out}\n`);

// decode a PNG back to raw RGBA so the mp4 row is measured the same way
function pngToRgba(p) {
  const r = spawnSync(ff, ["-hide_banner", "-loglevel", "error", "-i", p,
    "-f", "rawvideo", "-pix_fmt", "rgba", "-"], { maxBuffer: W * H * 4 + 1e6 });
  return r.status === 0 ? r.stdout : null;
}
