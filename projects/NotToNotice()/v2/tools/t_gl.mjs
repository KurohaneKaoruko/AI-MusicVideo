// What is the GL backend, and where does a frame's wall time actually go?
import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\//, "");
const srv = await startServer(ROOT);
const page = await launch({ url: srv.url("/renderer/index.html"), headless: true, width: 1920, height: 1080 });
await page.eval(`new Promise(r => { const k = () => (window.__ready ? r(1) : setTimeout(k, 60)); k(); })`);

console.log("GL:", JSON.stringify(await page.eval(`window.__glInfo()`), null, 1));
console.log("\n  t    jsSubmit  gpuDrain  read1   read2   oneFrame   instances");
for (const s of [0, 5, 20, 30, 55, 75, 95, 120, 150, 178]) {
  const r = await page.eval(`window.__bench(${s} * 60, 10)`);
  console.log(`  ${String(s).padStart(3)}s ${String(r.jsSubmit).padStart(8)} ${String(r.gpuDrain).padStart(9)} ${String(r.readFirst).padStart(7)} ${String(r.readAgain).padStart(7)} ${String(r.oneFrameGpu).padStart(9)}   add=${r.add} solid=${r.solid} txt=${r.texts}`);
}
await page.close();
await srv.close();
