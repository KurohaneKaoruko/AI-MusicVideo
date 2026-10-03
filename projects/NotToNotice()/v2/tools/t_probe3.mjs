// Find a cheap way to get the 2D text layer into a GL texture.
import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\//, "");
const srv = await startServer(ROOT);
const page = await launch({ url: srv.url("/renderer/index.html"), headless: true, width: 1920, height: 1080 });
await page.eval(`new Promise(r => { const k = () => (window.__ready ? r(1) : setTimeout(k, 60)); k(); })`);

const probe = `(() => {
  const E = window.__E, gl = E.gl, W = E.W, H = E.H;
  const N = 6;
  const out = {};
  const read = () => { const a = performance.now(); E.readback(); return +(performance.now() - a).toFixed(2); };

  const paint = (c, seed) => {
    c.clearRect(0, 0, W, H);
    c.globalCompositeOperation = "lighter";
    c.font = "700 96px Consolas, monospace";
    c.fillStyle = "#9df";
    for (let i = 0; i < 40; i++) {
      c.fillText("NotToNotice(); // " + seed + " " + i, 40, 60 + i * 26);
      c.fillText("人間は、はなまる ★ " + i, 900, 80 + i * 26);
    }
  };

  const mkVariant = (attr) => {
    const c = document.createElement("canvas");
    c.width = W; c.height = H;
    const ctx = c.getContext("2d", attr);
    paint(ctx, attr && attr.willReadFrequently ? "sw" : "hw");
    return { c, ctx };
  };

  const variants = {
    hw: mkVariant({ alpha: true }),
    sw: mkVariant({ alpha: true, willReadFrequently: true }),
  };

  const tex = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

  const bench = (label, upload) => {
    // warm
    for (let i = 0; i < 2; i++) { upload(); read(); }
    const a = performance.now();
    for (let i = 0; i < N; i++) upload();
    const b = performance.now();
    const r = read();
    out[label + "_upload"] = +((b - a) / N).toFixed(2);
    out[label + "_after_read"] = r;
  };

  // 1. texImage2D straight from an accelerated canvas (the current path)
  bench("V1_texImage_canvas", () => {
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, W, H, 0, gl.RGBA, gl.UNSIGNED_BYTE, variants.hw.c);
  });

  // 2. getImageData from the accelerated canvas + texSubImage2D
  bench("V2_getImageData_hw", () => {
    const d = variants.hw.ctx.getImageData(0, 0, W, H);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, W, H, gl.RGBA, gl.UNSIGNED_BYTE, d.data);
  });

  // 3. getImageData from a willReadFrequently (CPU-backed) canvas + texSubImage2D
  bench("V3_getImageData_sw", () => {
    const d = variants.sw.ctx.getImageData(0, 0, W, H);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, W, H, gl.RGBA, gl.UNSIGNED_BYTE, d.data);
  });

  // 4. createImageBitmap + texImage2D
  out.V4_note = "async, measured separately below";

  // 5. what does the paint itself cost on each canvas kind?
  const tp = (ctx, tag) => { const a = performance.now(); paint(ctx, tag); return +(performance.now() - a).toFixed(2); };
  out.paint_hw = tp(variants.hw.ctx, "hw");
  out.paint_sw = tp(variants.sw.ctx, "sw");

  // 6. raw getImageData cost alone
  const g = (ctx) => { const a = performance.now(); ctx.getImageData(0, 0, W, H); return +(performance.now() - a).toFixed(2); };
  out.getImageData_hw = g(variants.hw.ctx);
  out.getImageData_sw = g(variants.sw.ctx);

  return out;
})()`;

const r = await page.eval(probe);
for (const [k, v] of Object.entries(r)) console.log(String(k).padEnd(24), v);

// createImageBitmap path, measured with await
const bmp = await page.eval(`(async () => {
  const E = window.__E, gl = E.gl, W = E.W, H = E.H;
  const c = document.createElement("canvas"); c.width = W; c.height = H;
  const ctx = c.getContext("2d", { alpha: true });
  ctx.font = "700 96px Consolas, monospace"; ctx.fillStyle = "#9df";
  for (let i = 0; i < 40; i++) ctx.fillText("NotToNotice(); " + i, 40, 60 + i * 26);
  const tex = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  const up = async () => {
    const b = await createImageBitmap(c);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, gl.RGBA, gl.UNSIGNED_BYTE, b);
    b.close();
  };
  for (let i = 0; i < 2; i++) { await up(); E.readback(); }
  const a = performance.now();
  const N = 6;
  for (let i = 0; i < N; i++) await up();
  const bb = performance.now();
  const t = performance.now(); E.readback();
  return { upload: +((bb-a)/N).toFixed(2), read: +(performance.now()-t).toFixed(2) };
})()`);
console.log("V4_createImageBitmap   ", bmp.upload, " read:", bmp.read);

await page.close();
await srv.close();
