// ---------------------------------------------------------------------------
// capture.mjs - render a frame range of the film straight into a video segment
//
//   node capture.mjs --start 0 --end 1830 --out seg_00.mkv [--fps 60] [--crf 16]
//
// The page never sends pixels through CDP (a 4.6 MB dataURL wedges
// Runtime.evaluate). Instead the page renders, dumps the rtOut framebuffer with
// gl.readPixels and POSTs the raw RGBA to a private loopback server, which
// pipes it straight into ffmpeg's stdin. No intermediate files, no image
// encoder in the hot loop.
// ---------------------------------------------------------------------------
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import path from "node:path";
import { once } from "node:events";
import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";
import { findFFmpeg } from "./ffmpeg.mjs";

const W = 1920, H = 1080;
export const FRAME_BYTES = W * H * 4;

// Everything a segment owns, so a stalled run can be torn down from outside
// instead of leaving browsers and encoders behind.
const ACTIVE = new Set();

export function killAllActive() {
  for (const r of ACTIVE) {
    try { r.enc?.stdin?.destroy(); } catch {}
    try { r.enc?.kill(); } catch {}
    try { r.page?.close(); } catch {}
    try { r.srv?.close(); } catch {}
  }
  ACTIVE.clear();
}

function parseArgs(argv) {
  const a = { start: 0, end: 0, out: null, fps: 60, crf: 16, preset: "veryfast",
              workers: 0, headless: true, verify: true };
  for (let i = 0; i < argv.length; i++) {
    const k = argv[i];
    if (k === "--start") a.start = Number(argv[++i]);
    else if (k === "--end") a.end = Number(argv[++i]);
    else if (k === "--out") a.out = argv[++i];
    else if (k === "--fps") a.fps = Number(argv[++i]);
    else if (k === "--crf") a.crf = Number(argv[++i]);
    else if (k === "--preset") a.preset = argv[++i];
    else if (k === "--headful") a.headless = false;
    else throw new Error("unknown arg " + k);
  }
  if (!a.out) throw new Error("--out is required");
  return a;
}

/**
 * Render frames [start, end) into args.out (a video file).
 * Returns { frames, bytes, seconds, ms }.
 */
export async function captureSegment({ start, end, out, fps = 60, crf = 16,
                                       preset = "veryfast", headless = true,
                                       root, quiet = false, onProgress }) {
  const nFrames = end - start;
  if (nFrames <= 0) throw new Error("empty range");

  mkdirSync(path.dirname(path.resolve(out)), { recursive: true });
  const ff = findFFmpeg();

  // --- ffmpeg: raw RGBA on stdin -> H.264 in a matroska container -----------
  const ffArgs = [
    "-hide_banner", "-loglevel", "error", "-y",
    "-f", "rawvideo", "-pix_fmt", "rgba", "-s", `${W}x${H}`, "-r", String(fps),
    "-i", "pipe:0",
    "-an",
    "-c:v", "libx264", "-preset", preset, "-crf", String(crf),
    "-pix_fmt", "yuv420p",
    "-g", String(fps), "-keyint_min", String(fps), "-sc_threshold", "0",
    "-f", "matroska",
    out,
  ];
  const res = { enc: null, page: null, srv: null };
  ACTIVE.add(res);

  const enc = spawn(ff, ffArgs, { stdio: ["pipe", "ignore", "pipe"] });
  res.enc = enc;
  let encErr = "";
  enc.stderr.on("data", (d) => { encErr += d.toString(); });
  const encDone = once(enc, "close");

  let written = 0;
  let bytes = 0;
  let broken = null;
  enc.stdin.on("error", (e) => { broken = e; });

  const srv = await startServer(root, {
    onFrame: async (buf) => {
      if (broken) throw broken;
      if (buf.length !== FRAME_BYTES) {
        throw new Error(`bad frame size ${buf.length} != ${FRAME_BYTES}`);
      }
      written++;
      bytes += buf.length;
      if (!enc.stdin.write(buf)) await once(enc.stdin, "drain");
    },
  });

  res.srv = srv;

  const page = await launch({
    url: srv.url("/renderer/index.html"),
    headless,
    width: W,
    height: H,
  });
  res.page = page;

  const t0 = Date.now();
  try {
    // wait for boot
    for (let i = 0;; i++) {
      const st = await page.eval(
        "({ready: window.__ready===true, e: window.__bootError||null, errs:(window.__errors||[]).slice(0,2)})");
      if (st.e) throw new Error("boot failed: " + st.e);
      if (st.errs.length) throw new Error("page errors: " + st.errs.join(" | "));
      if (st.ready) break;
      if (i > 200) throw new Error("renderer never became ready");
      await new Promise((r) => setTimeout(r, 100));
    }

    await page.eval("window.__capStart()");
    const url = JSON.stringify(srv.url("/_frame"));

    for (let f = start; f < end; f++) {
      const t1 = Date.now();
      const r = await page.eval(`window.__captureRaw(${f}, ${url})`);
      if (r !== true) throw new Error(`frame ${f}: ${r}`);

      if (onProgress) onProgress(f - start + 1, nFrames);
      else if (!quiet && (f - start + 1) % 300 === 0) {
        const done = f - start + 1;
        const el = (Date.now() - t0) / 1000;
        const rate = done / el;
        process.stdout.write(
          `    ${done}/${nFrames}  ${rate.toFixed(1)} fps  eta ${(((nFrames - done) / rate) / 60).toFixed(1)} min\n`);
      }
      // a frame that takes minutes means the page wedged; fail loudly
      if (Date.now() - t1 > 180000) throw new Error(`frame ${f} stalled`);
      if (page.errored) throw new Error("page error: " + page.errored);
    }

    enc.stdin.end();
    const [code] = await encDone;
    if (code !== 0) throw new Error(`ffmpeg exited ${code}: ${encErr.slice(0, 400)}`);
    if (written !== nFrames) throw new Error(`wrote ${written} frames, expected ${nFrames}`);
  } finally {
    ACTIVE.delete(res);
    try { enc.stdin.destroy(); } catch {}
    try { enc.kill(); } catch {}
    await page.close().catch(() => {});
    await srv.close().catch(() => {});
  }

  return { frames: written, bytes, ms: Date.now() - t0, out };
}

// ------------------------------------------------------------------ cli -----

if (import.meta.url === `file://${process.argv[1].replace(/\\/g, "/")}` ||
    process.argv[1]?.endsWith("capture.mjs")) {
  const a = parseArgs(process.argv.slice(2));
  const root = path.resolve(path.dirname(new URL(import.meta.url).pathname.replace(/^\//, "")), "..");
  const r = await captureSegment({ ...a, root });
  console.log(`\n  ${r.frames} frames -> ${a.out}`);
  console.log(`  ${(r.bytes / 1e9).toFixed(2)} GB pushed, ${(r.ms / 1000 / 60).toFixed(2)} min, ` +
              `${(r.frames / (r.ms / 1000)).toFixed(1)} fps`);
}
