// Measure the raw-RGBA capture path end to end, split into its three legs:
//   render   - GL command submission (JS side only)
//   readback - gl.readPixels out of the rtOut FBO
//   post     - loopback POST of 8.29 MB to the local server
import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\//, "");
const SAMPLES = [5, 30, 55, 75, 95, 120, 150, 178];

const t = { bytes: 0, posts: 0 };
const srv = await startServer(ROOT, {
  onFrame: (buf) => { t.bytes += buf.length; t.posts++; },
});

const page = await launch({
  url: `${srv.url("/renderer/index.html")}`,
  headless: true,
  width: 1920,
  height: 1080,
});

await page.eval(`new Promise(r => { const k = () => (window.__ready ? r(1) : setTimeout(k, 60)); k(); })`);
console.log("info:", JSON.stringify(await page.eval(`window.__info`)));

const FRAME_BYTES = 1920 * 1080 * 4;
const U = JSON.stringify(srv.url("/_frame"));

await page.eval(`window.__capStart()`);

// warm every scene once so no shader compile or texture alloc lands mid-measure
await page.eval(`(async () => { for (const s of [5,30,55,75,95,120,150,178]) await window.__captureRaw(s*60, ${U}); return true; })()`);
t.bytes = 0; t.posts = 0;

const N = 8;
console.log("\n-- per-leg timings (ms/frame) ---------------------------------");
console.log("  t     render   readback   post    total");
const rows = [];
for (const s of SAMPLES) {
  const r = await page.eval(`(async () => {
     const f = ${s} * 60, N = ${N}, u = ${U};
     let t0 = performance.now();
     for (let k = 0; k < N; k++) window.renderFrame(f + k);
     let t1 = performance.now();
     for (let k = 0; k < N; k++) { window.renderFrame(f + k); window.__E.readback(); }
     let t2 = performance.now();
     for (let k = 0; k < N; k++) await window.__captureRaw(f + k, u);
     let t3 = performance.now();
     return { render: (t1-t0)/N, read: (t2-t1)/N, post: (t3-t2)/N - (t2-t1)/N };
  })()`);
  rows.push({ s, ...r });
  console.log(`  ${String(s).padStart(3)}s  ${r.render.toFixed(1).padStart(6)}  ${r.read.toFixed(1).padStart(8)}  ${r.post.toFixed(1).padStart(6)}  ${(r.render+r.read+r.post).toFixed(1).padStart(7)}`);
}

const best = rows.reduce((a, b) => (a.render + a.read + a.post < b.render + b.read + b.post ? a : b));
console.log(`\nbest t=${best.s}s  => ${(best.render + best.read + best.post).toFixed(1)} ms/frame`);

const W = 60;
const t0 = Date.now();
await page.eval(`(async () => { const u = ${U};
  for (let k = 0; k < ${W}; k++) await window.__captureRaw(60*115 + k, u); return true; })()`);
const dt = Date.now() - t0;
console.log(`throughput over 1s of film (t=115..116): ${(dt / W).toFixed(1)} ms/frame, ${(1000 / (dt / W)).toFixed(1)} fps`);
console.log(`bytes ${(t.bytes / 1e6).toFixed(0)} MB / ${t.posts} posts (expect ${(W * FRAME_BYTES / 1e6).toFixed(0)} MB)`);
console.log(`whole film single-process estimate: ${((dt / W) * 10980 / 60000).toFixed(1)} min`);
console.log(`        8-way split estimate    : ${((dt / W) * 10980 / 60000 / 8).toFixed(1)} min`);

await page.close();
await srv.close();
