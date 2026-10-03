import { launch } from "./cdp.mjs";
import { pathToFileURL } from "node:url";
import path from "node:path";
import { fileURLToPath } from "node:url";

const DIR = path.dirname(fileURLToPath(import.meta.url));
const URL = pathToFileURL(path.join(DIR, "probe", "budget.html")).href;

for (const headless of [true, false]) {
  console.log(`\n=== ${headless ? "headless" : "headful"} ===`);
  const page = await launch({
    url: URL, headless, width: 1920, height: 1080,
    extraArgs: headless ? [] : ["--window-position=40,40"],
  });
  try {
    console.log("  ", JSON.stringify(await page.eval("window.gpuInfo()")));
    console.log("  load (noise iterations/pixel) -> ms/frame @1920x1080");
    for (const n of [0, 1, 2, 4, 8, 16, 32]) {
      const r = await page.eval(`window.measure(${n}, ${n === 0 ? 30 : n <= 4 ? 20 : 6})`);
      const budget = (1000 / 60 / r).toFixed(1);
      console.log(`    N=${String(n).padStart(3)}  ${r.toFixed(2).padStart(8)} ms   (${budget}x a 16.7ms frame)`);
    }
    console.log("  resolution scaling at N=8:");
    for (const [w, h] of [[1920, 1080], [1280, 720], [960, 540], [640, 360]]) {
      const r = await page.eval(`window.measureAt(${w}, ${h}, 8, 20)`);
      console.log(`    ${w}x${h}  ${r.toFixed(2)} ms`);
    }
  } catch (e) {
    console.log("  FAILED:", e.message);
  } finally {
    await page.close();
  }
}
