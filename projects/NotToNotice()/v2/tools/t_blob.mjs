import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(DIR, "..");
const T = Number(process.argv[2] || 30);

const srv = await startServer(ROOT);
const page = await launch({ url: srv.url("/renderer/index.html"), headless: true, width: 1920, height: 1080 });
try {
  for (let i = 0; i < 200; i++) {
    if (await page.eval("window.__ready===true")) break;
    await new Promise((r) => setTimeout(r, 100));
  }
  // expose a blob-measuring helper
  await page.eval(`window.__blobTest = async (t, kind, q) => {
    const runs = [];
    let last = 0;
    for (let i = 0; i < 4; i++) {
      window.renderAt(t);
      const s = performance.now();
      const blob = await new Promise((res) => document.getElementById("c").toBlob(res, kind, q));
      runs.push(performance.now() - s);
      last = blob.size;
    }
    runs.shift();
    return { ms: runs.reduce((a, b) => a + b, 0) / runs.length, bytes: last };
  }`);
  for (const [kind, q] of [["image/png", undefined], ["image/jpeg", 0.98], ["image/jpeg", 0.92], ["image/webp", 0.95]]) {
    const r = await page.eval(`window.__blobTest(${T}, ${JSON.stringify(kind)}, ${q === undefined ? "undefined" : q})`);
    console.log(`  ${(kind + (q ? " q" + q : "")).padEnd(20)} ${r.ms.toFixed(1).padStart(7)} ms   ${(r.bytes / 1024).toFixed(0).padStart(5)} KB`);
  }
  // pure render cost
  const rr = await page.eval(`(()=>{window.renderAt(${T});const s=performance.now();
    for(let i=0;i<8;i++) window.renderAt(${T}+i/60); return (performance.now()-s)/8;})()`);
  console.log(`  render only          ${rr.toFixed(1).padStart(7)} ms`);
} finally {
  await page.close();
  await srv.close();
}
