// Corrected upload probe: the canvas is REPAINTED every iteration, which is
// what happens in the film. (The previous probe uploaded a static canvas and
// Chrome elided the transfer, making every path look free.)
import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\//, "");
const srv = await startServer(ROOT);
const page = await launch({ url: srv.url("/renderer/index.html"), headless: true, width: 1920, height: 1080 });
await page.eval(`new Promise(r => { const k = () => (window.__ready ? r(1) : setTimeout(k, 60)); k(); })`);

const probe = `(() => {
  const E = window.__E, gl = E.gl, W = E.W, H = E.H;
  const out = {};
  const N = 6;
  const read = () => { const a = performance.now(); E.readback(); return +(performance.now() - a).toFixed(2); };

  const paint = (ctx, seed) => {
    ctx.globalCompositeOperation = "source-over";
    ctx.clearRect(0, 0, W, H);
    ctx.globalCompositeOperation = "lighter";
    ctx.font = "700 96px Consolas, monospace";
    ctx.fillStyle = "#9df";
    for (let i = 0; i < 30; i++) {
      ctx.fillText("NotToNotice(); // A" + seed + " " + i, 40, 60 + i * 32);
      ctx.fillText("人間は、はなまる ★ " + i, 900, 80 + i * 32);
    }
    ctx.globalCompositeOperation = "source-over";
  };

  const mk = (attr) => {
    const c = document.createElement("canvas"); c.width = W; c.height = H;
    return { c, ctx: c.getContext("2d", attr) };
  };
  const HW = mk({ alpha: true });
  const SW = mk({ alpha: true, willReadFrequently: true });

  const tex = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

  const run = (label, v, upload) => {
    for (let i = 0; i < 2; i++) { paint(v.ctx, i); upload(v); read(); }
    let paintMs = 0, upMs = 0;
    const a = performance.now();
    for (let i = 0; i < N; i++) {
      const p0 = performance.now(); paint(v.ctx, i + 10); const p1 = performance.now();
      upload(v); const p2 = performance.now();
      paintMs += p1 - p0; upMs += p2 - p1;
    }
    const b = performance.now();
    out[label + "_paint"] = +(paintMs / N).toFixed(2);
    out[label + "_upload"] = +(upMs / N).toFixed(2);
    out[label + "_wall"] = +((b - a) / N).toFixed(2);
    out[label + "_read"] = read();
  };

  run("S0_texImage_hw", HW, (v) => {
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, W, H, 0, gl.RGBA, gl.UNSIGNED_BYTE, v.c);
  });

  run("S1_getData_hw", HW, (v) => {
    const d = v.ctx.getImageData(0, 0, W, H);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, W, H, gl.RGBA, gl.UNSIGNED_BYTE, d.data);
  });

  run("S2_getData_sw", SW, (v) => {
    const d = v.ctx.getImageData(0, 0, W, H);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, W, H, gl.RGBA, gl.UNSIGNED_BYTE, d.data);
  });

  run("S3_texImage_sw", SW, (v) => {
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, W, H, 0, gl.RGBA, gl.UNSIGNED_BYTE, v.c);
  });

  // what does the text texture actually COST once a draw consumes it?
  const drawIt = () => {
    E.rtScene.bind();
    gl.enable(gl.BLEND); gl.blendFunc(gl.ONE, gl.ONE);
    gl.useProgram(E.pTex.prog);
    E.bindTex(E.pTex.u.uTex, 0, tex);
    gl.uniform1f(E.pTex.u.uGain, 1.0);
    gl.uniform1f(E.pTex.u.uAlpha, 1.0);
    gl.uniform3f(E.pTex.u.uMul, 1, 1, 1);
    gl.bindVertexArray(E.quad);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    gl.disable(gl.BLEND);
  };
  const a2 = performance.now();
  for (let i = 0; i < N; i++) { paint(HW.ctx, i + 50); gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA8,W,H,0,gl.RGBA,gl.UNSIGNED_BYTE,HW.c); drawIt(); }
  const b2 = performance.now();
  out.S4_withDraw_wall = +((b2 - a2) / N).toFixed(2);
  out.S4_read = read();
  return out;
})()`;

const r = await page.eval(probe);
for (const [k, v] of Object.entries(r)) console.log(String(k).padEnd(24), v);
await page.close();
await srv.close();
