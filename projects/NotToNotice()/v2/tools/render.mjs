// ---------------------------------------------------------------------------
// render.mjs - the whole film.
//
//   node render.mjs                          -> ../dist/NotToNotice v2 [1080p60].mp4
//   node render.mjs --workers 6 --crf 16
//   node render.mjs --frames 0:600 --out x.mkv   (a slice, no audio mux)
//
// Splits [0, FRAMES) into N contiguous ranges, renders them in parallel (each
// worker owns a Chrome, a loopback server and an ffmpeg), concatenates the
// segments losslessly and muxes the song.
// ---------------------------------------------------------------------------
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, unlinkSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { captureSegment, killAllActive } from "./capture.mjs";
import { findFFmpeg } from "./ffmpeg.mjs";

const DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(DIR, "..");
const PROJ = path.resolve(ROOT, "..");

const FPS = 60;
const TOTAL = 183.0;
const FRAMES = Math.round(TOTAL * FPS);      // 10980

function parse(argv) {
  const a = { workers: 6, crf: 16, preset: "veryfast", out: null, frames: null, tmp: null };
  for (let i = 0; i < argv.length; i++) {
    const k = argv[i];
    if (k === "--workers") a.workers = Math.max(1, Number(argv[++i]));
    else if (k === "--crf") a.crf = Number(argv[++i]);
    else if (k === "--preset") a.preset = argv[++i];
    else if (k === "--out") a.out = argv[++i];
    else if (k === "--tmp") a.tmp = argv[++i];
    else if (k === "--frames") {
      const [x, y] = argv[++i].split(":").map(Number);
      a.frames = [x, y];
    } else throw new Error("unknown arg " + k);
  }
  return a;
}

const a = parse(process.argv.slice(2));
const [F0, F1] = a.frames || [0, FRAMES];
// a bare slice (no --out) just leaves the segments on disk for inspection
const sliceOnly = !!a.frames && !a.out;

const tmp = a.tmp || path.join(ROOT, ".tmp", "seg" + Date.now());
mkdirSync(tmp, { recursive: true });

const nSeg = Math.min(a.workers, Math.max(1, F1 - F0));
// round segment boundaries to whole frames, distribute the remainder
const bounds = [];
for (let i = 0; i <= nSeg; i++) {
  bounds.push(Math.round(F0 + ((F1 - F0) * i) / nSeg));
}
const segs = [];
for (let i = 0; i < nSeg; i++) {
  segs.push({ i, start: bounds[i], end: bounds[i + 1], out: path.join(tmp, `seg_${String(i).padStart(2, "0")}.mkv`) });
}

console.log(`NotToNotice(); v2  —  ${F1 - F0} frames @ ${FPS}fps  (${((F1 - F0) / FPS).toFixed(1)}s)`);
console.log(`${nSeg} workers, crf ${a.crf}, preset ${a.preset}`);
console.log(`segments: ${segs.map((s) => `${s.start}-${s.end}`).join("  ")}\n`);

const t0 = Date.now();
const prog = segs.map(() => 0);
const total = F1 - F0;
let lastPrint = 0;

let lastPct = -1;
// If a worker wedges (browser dies, page stops answering) the whole run would
// otherwise hang forever, because the other workers keep node alive and nobody
// is watching. Bail out loudly after 5 minutes of zero frames anywhere.
const STALL_MS = 5 * 60 * 1000;
let lastDone = -1, lastMoveAt = Date.now(), stalled = false;

const tick = setInterval(() => {
  const done = prog.reduce((x, y) => x + y, 0);
  if (done !== lastDone) { lastDone = done; lastMoveAt = Date.now(); }
  else if (!stalled && Date.now() - lastMoveAt > STALL_MS) {
    stalled = true;
    clearInterval(tick);
    process.stderr.write(
      `\n  !! no frames for ${(STALL_MS / 60000).toFixed(0)} min at ${done}/${total} — aborting\n`);
    killAllActive();
    setTimeout(() => process.exit(2), 1500);
    return;
  }
  const pct = Math.floor((done / total) * 200);
  if (pct === lastPct) return;
  lastPct = pct;
  const el = (Date.now() - t0) / 1000;
  const rate = done / Math.max(el, 1e-3);
  const eta = rate > 0 ? (total - done) / rate : 0;
  const bar = "#".repeat(Math.round((done / total) * 30)).padEnd(30, ".");
  process.stdout.write(
    `\r  [${bar}] ${done}/${total}  ${rate.toFixed(1)} fps  ${(el / 60).toFixed(1)}m  eta ${(eta / 60).toFixed(1)}m   `);
}, 1000);

let failed = null;
try {
  await Promise.all(segs.map((s) =>
    captureSegment({
      start: s.start, end: s.end, out: s.out,
      fps: FPS, crf: a.crf, preset: a.preset,
      root: ROOT, quiet: true,
      onProgress: (n) => { prog[s.i] = n; },
    }).catch((e) => { failed = failed || e; throw e; })
  ));
} finally {
  clearInterval(tick);
  process.stdout.write("\n");
}
if (failed) throw failed;

const wall = (Date.now() - t0) / 1000;
console.log(`\n  rendered in ${(wall / 60).toFixed(2)} min`);

// ---------------------------------------------------------------- assemble --
const ff = findFFmpeg();
const run = (args, label) => new Promise((res, rej) => {
  const p = spawn(ff, args, { stdio: ["ignore", "ignore", "pipe"] });
  let err = "";
  p.stderr.on("data", (d) => { err += d; });
  p.on("close", (c) => (c === 0 ? res() : rej(new Error(`${label} failed: ${err.slice(-500)}`))));
});

if (sliceOnly) {
  console.log(`  slice written: ${segs[0].out}`);
} else {
  const outPath = a.out || path.join(PROJ, "dist", "NotToNotice - Enoa [v2 1080p60].mp4");
  mkdirSync(path.dirname(outPath), { recursive: true });

  const listFile = path.join(tmp, "list.txt");
  writeFileSync(listFile, segs.map((s) => `file '${s.out.replace(/\\/g, "/").replace(/'/g, "'\\''")}'`).join("\n") + "\n");

  const audio = path.join(PROJ, "NotToNotice.mp3");
  const hasAudio = existsSync(audio);

  // 1. lossless concat of the segments
  const joined = path.join(tmp, "joined.mkv");
  await run(["-hide_banner", "-loglevel", "error", "-y",
    "-f", "concat", "-safe", "0", "-i", listFile,
    "-c", "copy", joined], "concat");

  // 2. mux the song, pad the tail, faststart for streaming
  const muxArgs = ["-hide_banner", "-loglevel", "error", "-y", "-i", joined];
  if (hasAudio) muxArgs.push("-i", audio);
  muxArgs.push("-map", "0:v:0");
  if (hasAudio) {
    muxArgs.push("-map", "1:a:0", "-c:a", "aac", "-b:a", "256k", "-af", "apad");
  }
  muxArgs.push("-c:v", "copy", "-t", TOTAL.toFixed(3), "-movflags", "+faststart", outPath);
  await run(muxArgs, "mux");

  const mb = statSync(outPath).size / 1e6;
  console.log(`  -> ${outPath}  (${mb.toFixed(1)} MB)`);

  for (const s of segs) { try { unlinkSync(s.out); } catch {} }
  try { unlinkSync(joined); } catch {}
  try { unlinkSync(listFile); } catch {}
}
