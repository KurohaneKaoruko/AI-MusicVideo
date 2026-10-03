// Render still frames to .preview/*.png so the film can be inspected visually.
//   node preview.mjs 5 12 30 61 105 176
//   node preview.mjs --at 0:183:12      (12 evenly spaced)
import { launch } from "./cdp.mjs";
import { startServer } from "./server.mjs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { writeFileSync, mkdirSync } from "node:fs";

const DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(DIR, "..");
const OUTDIR = path.join(ROOT, ".preview");
mkdirSync(OUTDIR, { recursive: true });

const argv = process.argv.slice(2);
let times = [];
let headless = true;
const rest = [];
for (let i = 0; i < argv.length; i++) {
  if (argv[i] === "--headful") headless = false;
  else if (argv[i] === "--at") {
    const [a, b, n] = argv[++i].split(":").map(Number);
    for (let k = 0; k < n; k++) times.push(a + (b - a) * (k / Math.max(n - 1, 1)));
  } else rest.push(Number(argv[i]));
}
if (rest.length) times = rest;
if (!times.length) times = [5, 12, 30, 61, 105, 176];
times = times.filter((x) => Number.isFinite(x)).sort((a, b) => a - b);

const srv = await startServer(ROOT);
const page = await launch({
  url: srv.url("/renderer/index.html"),
  headless,
  width: 1920,
  height: 1080,
});
try {
  // wait for the analysis JSON + boot
  let ok = false;
  for (let i = 0; i < 100; i++) {
    const st = await page.eval("({ready: window.__ready===true, bootErr: window.__bootError||null, errs: (window.__errors||[]).slice(0,3)})");
    if (st.bootErr) throw new Error("boot failed: " + st.bootErr);
    if (st.errs && st.errs.length) throw new Error("page errors:\n" + st.errs.join("\n"));
    if (st.ready) { ok = true; break; }
    await new Promise((r) => setTimeout(r, 120));
  }
  if (!ok) throw new Error("renderer never became ready");
  console.log("info:", JSON.stringify(await page.eval("window.__info")));

  for (const t of times) {
    const r = await page.eval(`(()=>{
      try { window.renderAt(${t}); } catch(e){ return {err: String(e && e.stack || e)}; }
      return {err: null};
    })()`);
    if (r.err) throw new Error(`renderAt(${t}) threw:\n${r.err}`);
    const errs = await page.eval("(window.__errors||[]).slice(0,2)");
    if (errs.length) throw new Error("page errors:\n" + errs.join("\n"));
    const data = await page.eval(`document.getElementById("c").toDataURL("image/png")`);
    if (!data || data.length < 2000) throw new Error(`t=${t}: empty capture (${data ? data.length : 0} chars)`);
    const buf = Buffer.from(data.split(",")[1], "base64");
    const tag = t.toFixed(2).replace(".", "_");
    const f = path.join(OUTDIR, `v2_t${tag}.png`);
    writeFileSync(f, buf);
    console.log(`  t=${t.toFixed(2)}s -> ${path.basename(f)}  (${(buf.length / 1024).toFixed(0)} KB)`);
  }
} finally {
  await page.close();
  await srv.close();
}
