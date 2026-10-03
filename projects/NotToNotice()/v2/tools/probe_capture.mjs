// Realistic capture-latency probe: render load ~N=4 (a real per-frame budget)
// then measure the cost of getting that frame out of the browser.
import { launch } from "./cdp.mjs";
import { pathToFileURL } from "node:url";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { writeFileSync } from "node:fs";

const DIR = path.dirname(fileURLToPath(import.meta.url));
const URL = pathToFileURL(path.join(DIR, "probe", "budget.html")).href;
const LOAD = 4;

for (const headless of [true]) {
  console.log(`=== ${headless ? "headless" : "headful"} ===  (render load N=${LOAD})`);
  const page = await launch({ url: URL, headless, width: 1920, height: 1080 });
  try {
    console.log("  render only      :", (await page.eval(`window.measure(${LOAD}, 20)`)).toFixed(2), "ms");

    // screenshot, warming up first
    for (const fmt of ["png", "jpeg"]) {
      const N = 12;
      let bytes = 0;
      await page.eval(`window.measure(${LOAD}, 2)`);
      await page.screenshot(fmt === "png" ? { format: "png" } : { format: "jpeg", quality: 95 });
      const t0 = Date.now();
      for (let i = 0; i < N; i++) {
        await page.eval(`window.measure(${LOAD}, 1)`);
        const b = await page.screenshot(
          fmt === "png" ? { format: "png" } : { format: "jpeg", quality: 95 });
        bytes += b.length;
        if (i === 0) writeFileSync(path.join(DIR, "probe", `cap_${fmt}.${fmt === "png" ? "png" : "jpg"}`), b);
      }
      const ms = (Date.now() - t0) / N;
      console.log(`  + screenshot ${fmt.padEnd(4)}: ${ms.toFixed(1)} ms/frame  (${(bytes / N / 1024).toFixed(0)} KB)`);
    }

    // in-page toDataURL
    for (const spec of ["image/png"]) {
      const r = await page.eval(`(()=>{const c=document.getElementById('c');
        window.measure(${LOAD},2);
        const t=performance.now(); let n=0;
        for(let i=0;i<12;i++){ window.measure(${LOAD},1); n+=c.toDataURL(${JSON.stringify(spec)}).length; }
        return {ms:(performance.now()-t)/12, kb:Math.round(n/12/1024)};})()`);
      console.log(`  + toDataURL png  : ${r.ms.toFixed(1)} ms/frame  (${r.kb} KB)`);
    }
  } catch (e) {
    console.log("  FAILED:", e.message);
  } finally {
    await page.close();
  }
}
