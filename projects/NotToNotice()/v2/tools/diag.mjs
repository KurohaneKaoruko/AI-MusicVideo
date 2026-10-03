// one-off diagnostic: report what each stage of the pipeline actually produced
import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(DIR, "..");

const times = process.argv.slice(2).map(Number).filter(Number.isFinite);
const probe = times.length ? times : [5, 12, 30, 61, 105, 176];

const srv = await startServer(ROOT);
const page = await launch({ url: srv.url("/renderer/index.html"), headless: true, width: 1920, height: 1080 });
try {
  for (let i = 0; i < 100; i++) {
    if (await page.eval("window.__ready===true")) break;
    await new Promise((r) => setTimeout(r, 120));
  }
  console.log("info:", JSON.stringify(await page.eval("window.__info")));
  for (const t of probe) {
    const r = await page.eval(`(()=>{
      try { window.renderAt(${t}); } catch(e){ return {thrown:String(e && e.stack || e)}; }
      return {
        state: window.__state(),
        neb: window.__statsNeb(),
        scene: window.__statsScene(),
        out: window.__stats(),
        text: window.__statsText(),
      };
    })()`);
    console.log(`\n--- t=${t}s ---`);
    if (r.thrown) { console.log("  THREW:", r.thrown.split("\n").slice(0, 4).join("\n  ")); continue; }
    console.log("  state:", JSON.stringify(r.state));
    console.log("  neb  :", JSON.stringify(r.neb));
    console.log("  scene:", JSON.stringify(r.scene));
    console.log("  out  :", JSON.stringify(r.out));
    console.log("  text :", JSON.stringify(r.text));
  }
} finally {
  await page.close();
  await srv.close();
}
