// Render a set of stills through the *real* capture path (so what I look at is
// exactly what the film will be), then build a contact sheet with ffmpeg.
//
//   node survey.mjs                      -> every 6 s, one sheet
//   node survey.mjs --every 3
//   node survey.mjs --t 5,30,75,120
//   node survey.mjs --tile 6 --cell 320
import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";
import { findFFmpeg } from "./ffmpeg.mjs";
import { spawn } from "node:child_process";
import { mkdirSync, writeFileSync, existsSync, createWriteStream, unlinkSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const W = 1920, H = 1080, FRAME_BYTES = W * H * 4;
const DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(DIR, "..");

const argv = process.argv.slice(2);
let every = 6, tileCols = 6, cell = 320, times = null;
for (let i = 0; i < argv.length; i++) {
  if (argv[i] === "--every") every = Number(argv[++i]);
  else if (argv[i] === "--tile") tileCols = Number(argv[++i]);
  else if (argv[i] === "--cell") cell = Number(argv[++i]);
  else if (argv[i] === "--t") times = argv[++i].split(",").map(Number);
}
if (!times) {
  times = [];
  for (let t = 1; t < 182.5; t += every) times.push(+t.toFixed(2));
}
times = times.filter(Number.isFinite).sort((a, b) => a - b);

// Each run gets its own directory: rmSync is intercepted by this machine's
// safe-delete guard, so the tool never deletes anything.
const BASE = path.join(ROOT, ".survey");
mkdirSync(BASE, { recursive: true });
let runIdx = 1;
while (existsSync(path.join(BASE, "r" + runIdx))) runIdx++;
const OUT = path.join(BASE, "r" + runIdx);
mkdirSync(OUT, { recursive: true });

// --- collect sampled frames as one contiguous raw stream --------------------
const rawPath = path.join(OUT, "sample.rgba");
const raw = createWriteStream(rawPath);
let pending = Promise.resolve();
let got = 0;
const srv = await startServer(ROOT, {
  onFrame: (buf) => {
    if (buf.length !== FRAME_BYTES) throw new Error("bad frame " + buf.length);
    got++;
    pending = pending.then(() => new Promise((res) => raw.write(buf, res)));
    return pending;
  },
});

const page = await launch({ url: srv.url("/renderer/index.html"), headless: true, width: W, height: H });
await page.eval("new Promise(r => { const k = () => (window.__ready ? r(window.__info) : setTimeout(k, 60)); k(); })");
await page.eval("window.__capStart()");
const U = JSON.stringify(srv.url("/_frame"));

console.log(`sampling ${times.length} frames every ${every}s ...`);
const t0 = Date.now();
for (const t of times) {
  const f = Math.round(t * 60);
  const r = await page.eval(`window.__captureRaw(${f}, ${U})`);
  if (r !== true) throw new Error(`t=${t}: ${r}`);
  process.stdout.write(`\r  ${got}/${times.length}  ${(got / ((Date.now() - t0) / 1000)).toFixed(1)} fps   `);
}
console.log();
await new Promise((res) => raw.end(res));
await page.close();
await srv.close();

// --- ffmpeg: raw stream -> numbered PNGs + a tiled contact sheet ------------
const ff = findFFmpeg();
const pngPat = path.join(OUT, "s_%03d.png");
const run = (args) => new Promise((res, rej) => {
  const p = spawn(ff, args, { stdio: ["ignore", "ignore", "inherit"] });
  p.on("close", (c) => (c === 0 ? res() : rej(new Error("ffmpeg " + c))));
});

await run(["-hide_banner", "-loglevel", "error", "-y",
  "-f", "rawvideo", "-pix_fmt", "rgba", "-s", `${W}x${H}`, "-framerate", "1",
  "-i", rawPath, pngPat]);

const rows = Math.ceil(times.length / tileCols);
await run(["-hide_banner", "-loglevel", "error", "-y", "-framerate", "1", "-i", pngPat,
  "-vf", `scale=${cell}:-2,tile=${tileCols}x${rows}:padding=2:color=0x141414`,
  "-frames:v", "1", path.join(OUT, "sheet.png")]);

try { unlinkSync(rawPath); } catch { /* the safe-delete shim may block this */ }
const sheet = path.join(OUT, "sheet.png");
console.log(`\n  ${times.length} stills -> ${OUT}`);
console.log(`  contact sheet: ${sheet}  (${tileCols}x${rows} @ ${cell}px)`);
console.log(`  individual:    ${pngPat}`);
console.log(`  times: ${times.join(", ")}`);
