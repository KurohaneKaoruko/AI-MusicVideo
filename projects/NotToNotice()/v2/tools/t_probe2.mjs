// Bisect the readPixels stall: is it the copy, the sync, or the canvas upload?
import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\//, "");
const srv = await startServer(ROOT);
const page = await launch({ url: srv.url("/renderer/index.html"), headless: true, width: 1920, height: 1080 });
await page.eval(`new Promise(r => { const k = () => (window.__ready ? r(1) : setTimeout(k, 60)); k(); })`);

const probe = `(() => {
  const E = window.__E, gl = E.gl, W = E.W, H = E.H;
  const R = () => { const a = performance.now(); E.readback(); return +(performance.now() - a).toFixed(2); };
  const out = {};
  const ms = (f) => { const a = performance.now(); f(); return +(performance.now() - a).toFixed(2); };

  // ---- A. nothing but a clear -------------------------------------------
  E.rtOut.bind(); gl.clearColor(0.1,0.2,0.3,1); gl.clear(gl.COLOR_BUFFER_BIT);
  out.A_clear_read1 = R();
  out.A_clear_read2 = R();

  // ---- B. clear + gl.finish + read --------------------------------------
  gl.clear(gl.COLOR_BUFFER_BIT); gl.finish();
  out.B_finish_ms = ms(() => {});
  out.B_clear_read = R();

  // ---- C. a full render, then a 1x1 sync-read, then the big read ---------
  window.renderFrame(75 * 60);
  out.C_pixel_read = ms(() => { const p = new Uint8Array(4); gl.readPixels(0,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,p); });
  out.C_full_read_after = R();

  // ---- D. full render then immediate full read (the known-bad case) ------
  window.renderFrame(75 * 60);
  out.D_full_read = R();
  out.D_full_read2 = R();

  // ---- E. is it the text canvas upload? ---------------------------------
  const t0 = performance.now();
  for (let k = 0; k < 5; k++) window.renderFrame(75 * 60 + k);
  out.E_five_js = +(performance.now() - t0).toFixed(2);
  out.E_after_five_read = R();

  // ---- F. five renders WITHOUT the text upload --------------------------
  const realUpload = E.uploadText.bind(E);
  E.uploadText = () => {};
  const t1 = performance.now();
  for (let k = 0; k < 5; k++) window.renderFrame(75 * 60 + k);
  out.F_five_js_notext = +(performance.now() - t1).toFixed(2);
  out.F_after_five_read = R();
  E.uploadText = realUpload;

  // ---- G. five renders then read, with text restored --------------------
  for (let k = 0; k < 5; k++) window.renderFrame(75 * 60 + k);
  out.G_read = R();
  out.G_read2 = R();
  out.G_read3 = R();
  return out;
})()`;

const r = await page.eval(probe);
for (const [k, v] of Object.entries(r)) console.log(String(k).padEnd(22), v);
await page.close();
await srv.close();
