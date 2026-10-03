// A/B the text-layer upload strategy inside the real renderer.
import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\//, "");
const srv = await startServer(ROOT);
const page = await launch({ url: srv.url("/renderer/index.html"), headless: true, width: 1920, height: 1080 });
await page.eval(`new Promise(r => { const k = () => (window.__ready ? r(1) : setTimeout(k, 60)); k(); })`);

const TS = [5, 30, 55, 75, 95, 120, 150, 178];
const MODES = ["off", "sub", "tex", "getdata"];

const head = "  t   " + MODES.map((m) => m.padStart(9)).join("");
console.log("per-frame GPU ms (render + forced sync)\n" + head);
const sums = {};
for (const m of MODES) sums[m] = 0;
for (const s of TS) {
  const cells = [];
  for (const m of MODES) {
    await page.eval(`window.__setTextMode(${JSON.stringify(m)})`);
    const r = await page.eval(`window.__gpu(${s} * 60, 3)`);
    cells.push(r.perFrame);
    sums[m] += r.perFrame;
  }
  console.log(`  ${String(s).padStart(3)}s ` + cells.map((c) => String(c).padStart(9)).join(""));
}
console.log("\nsum:" + MODES.map((m) => (m + "=" + sums[m].toFixed(0) + "ms").padStart(16)).join(""));

await page.close();
await srv.close();
