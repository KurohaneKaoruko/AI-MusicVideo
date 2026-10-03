import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { writeFileSync } from "node:fs";

const DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(DIR, "..");
const T = Number(process.argv[2] || 5);

const srv = await startServer(ROOT);
const page = await launch({ url: srv.url("/renderer/index.html"), headless: true, width: 1920, height: 1080 });
const step = async (label, fn) => {
  const t0 = Date.now();
  try {
    const r = await fn();
    console.log(`  ${label.padEnd(28)} ${String(Date.now() - t0).padStart(7)} ms   ${typeof r === "string" ? r.slice(0, 90) : JSON.stringify(r)}`);
    return r;
  } catch (e) {
    console.log(`  ${label.padEnd(28)} ${String(Date.now() - t0).padStart(7)} ms   FAILED: ${e.message.slice(0, 200)}`);
    throw e;
  }
};

try {
  await step("wait ready", async () => {
    for (let i = 0; i < 200; i++) {
      if (await page.eval("window.__ready===true")) return "ready";
      await new Promise((r) => setTimeout(r, 100));
    }
    throw new Error("never ready");
  });
  await step("__info", () => page.eval("JSON.stringify(window.__info)"));
  await step(`renderAt(${T})`, () => page.eval(`(()=>{const s=performance.now();window.renderAt(${T});return Math.round(performance.now()-s);})()`));
  await step("in-page toDataURL timing", () => page.eval(`(()=>{const s=performance.now();const d=document.getElementById("c").toDataURL("image/png");return "ms="+Math.round(performance.now()-s)+" chars="+d.length;})()`));
  const url = await step("fetch dataURL to node", () => page.eval(`document.getElementById("c").toDataURL("image/png")`));
  const buf = Buffer.from(String(url).split(",")[1], "base64");
  writeFileSync(path.join(ROOT, ".preview", `probe_t${T}.png`), buf);
  console.log(`  wrote probe_t${T}.png (${(buf.length / 1024).toFixed(0)} KB)`);
} finally {
  await page.close();
  await srv.close();
}
