// Minimal Chrome DevTools Protocol driver - no npm dependencies.
// Node 18+ (native fetch + WebSocket).
import { spawn } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { existsSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

const CHROME_CANDIDATES = [
  "C:/Program Files/Google/Chrome/Application/chrome.exe",
  "C:/Program Files (x86)/Google/Chrome/Application/chrome.exe",
  "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
];

export function findChrome() {
  for (const c of CHROME_CANDIDATES) if (existsSync(c)) return c;
  throw new Error("no chromium browser found");
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** true if something is already serving DevTools on this port */
async function isListening(port) {
  try {
    await fetch(`http://127.0.0.1:${port}/json/version`, { signal: AbortSignal.timeout(400) });
    return true;
  } catch {
    return false;
  }
}

export class Page {
  constructor(ws, sessionId, proc, profile) {
    this.ws = ws;
    this.sessionId = sessionId;
    this.proc = proc;
    this.profile = profile;
    this.id = 0;
    this.pending = new Map();
    this.events = new Map();
    ws.addEventListener("message", (ev) => {
      let msg;
      try {
        msg = JSON.parse(ev.data);
      } catch {
        return;
      }
      if (msg.id !== undefined && this.pending.has(msg.id)) {
        const { resolve, reject } = this.pending.get(msg.id);
        this.pending.delete(msg.id);
        msg.error ? reject(new Error(JSON.stringify(msg.error))) : resolve(msg.result);
      } else if (msg.method) {
        const key = msg.method + (msg.sessionId ? "@" + msg.sessionId : "");
        const list = this.events.get(key);
        if (list) for (const fn of list.splice(0)) fn(msg.params);
        const any = this.events.get(msg.method);
        if (any) for (const fn of any.splice(0)) fn(msg.params);
      }
    });
  }

  send(method, params = {}, sessionId = this.sessionId) {
    const id = ++this.id;
    const payload = { id, method, params };
    if (sessionId) payload.sessionId = sessionId;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.ws.send(JSON.stringify(payload));
      setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id);
          reject(new Error(`CDP timeout: ${method}`));
        }
      }, 120000);
    });
  }

  once(method, timeoutMs = 30000) {
    return new Promise((resolve, reject) => {
      const list = this.events.get(method) || [];
      const t = setTimeout(() => reject(new Error(`event timeout ${method}`)), timeoutMs);
      list.push((p) => {
        clearTimeout(t);
        resolve(p);
      });
      this.events.set(method, list);
    });
  }

  on(method, fn) {
    const list = this.events.get(method) || [];
    list.push(fn);
    this.events.set(method, list);
  }

  async goto(url) {
    const loaded = this.once("Page.loadEventFired", 60000);
    await this.send("Page.navigate", { url });
    await loaded.catch(() => {});
    return this;
  }

  /** evaluate a JS expression in the page, awaiting promises, returning JSON */
  async eval(expression) {
    const r = await this.send("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
      allowUnsafeEvalBlockedByCSP: true,
    });
    if (r.exceptionDetails) {
      throw new Error(
        "page exception: " +
          (r.exceptionDetails.exception?.description || r.exceptionDetails.text)
      );
    }
    return r.result.value;
  }

  async screenshot({ format = "png", quality } = {}) {
    const r = await this.send("Page.captureScreenshot", {
      format,
      quality,
      fromSurface: true,
      captureBeyondViewport: false,
      optimizeForSpeed: format === "png",
    });
    return Buffer.from(r.data, "base64");
  }

  async setViewport(width, height) {
    await this.send("Emulation.setDeviceMetricsOverride", {
      width,
      height,
      deviceScaleFactor: 1,
      mobile: false,
    });
  }

  async close() {
    try {
      this.ws.close();
    } catch {}
    try {
      this.proc?.kill();
    } catch {}
    try {
      await rm(this.profile, { recursive: true, force: true });
    } catch {}
  }
}

export async function launch({
  url = "about:blank",
  headless = true,
  width = 1920,
  height = 1080,
  port = 0,
  chrome = null,
  extraArgs = [],
  timeoutMs = 30000,
} = {}) {
  const exe = chrome || findChrome();
  const profile = await mkdtemp(path.join(tmpdir(), "cdpprof-"));

  // Port allocation is the one thing here that must not be guessed. Picking
  // `9200 + random(800)` and then polling /json/version until it answers means
  // that when two workers collide on the same number, the second one happily
  // attaches to the *first one's* already-running Chrome: two renderer pages in
  // one browser, one shared GPU, and whichever worker finishes first calls
  // page.close() and kills the browser out from under the other. That is a
  // silent 15-minute stall with zero frames delivered (measured, not theorised:
  // port 9247 held pages for loopback servers 49640 and 49641 at once).
  //
  // So: let Chrome pick, then read the port it chose out of DevToolsActivePort,
  // and verify via the browser websocket path that we attached to the process
  // we just spawned rather than to somebody else's.
  if (port && (await isListening(port))) {
    throw new Error(`debug port ${port} is already in use by another browser`);
  }
  const p = port || 0;
  const args = [
    `--remote-debugging-port=${p}`,
    `--user-data-dir=${profile}`,
    `--window-size=${width},${height}`,
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-extensions",
    "--disable-background-networking",
    "--disable-sync",
    "--hide-scrollbars",
    "--force-device-scale-factor=1",
    "--autoplay-policy=no-user-gesture-required",
    "--disable-features=CalculateNativeWinOcclusion",
    ...extraArgs,
  ];
  if (headless) args.push("--headless=new");
  const proc = spawn(exe, args, { stdio: "ignore", windowsHide: true });

  // wait for the debugging endpoint: Chrome writes "port\n/devtools/browser/<id>"
  // into DevToolsActivePort once it is listening.
  const portFile = path.join(profile, "DevToolsActivePort");
  let actual = p;
  let browserPath = null;
  let version = null;
  const t0 = Date.now();
  while (Date.now() - t0 < timeoutMs * 2) {
    if (!browserPath) {
      try {
        const lines = readFileSync(portFile, "utf8").split("\n");
        const num = Number((lines[0] || "").trim());
        if (num > 0) { actual = num; browserPath = (lines[1] || "").trim() || null; }
      } catch { /* not written yet */ }
    }
    if (actual) {
      try {
        const r = await fetch(`http://127.0.0.1:${actual}/json/version`);
        const v = await r.json();
        // identity check: this endpoint must be the browser we just started
        if (browserPath && v.webSocketDebuggerUrl && !v.webSocketDebuggerUrl.endsWith(browserPath)) {
          proc.kill();
          throw new Error(
            `attached to a foreign browser on port ${actual} (expected ${browserPath})`);
        }
        version = v;
        break;
      } catch (e) {
        if (String(e.message).includes("foreign browser")) throw e;
      }
    }
    await sleep(120);
  }
  const p2 = actual;
  if (!version) {
    try { proc.kill(); } catch {}
    throw new Error("chrome debugging port never came up");
  }

  // create the page target
  let target = null;
  const t1 = Date.now();
  while (Date.now() - t1 < timeoutMs) {
    try {
      const r = await fetch(`http://127.0.0.1:${p2}/json/new?${encodeURIComponent(url)}`, {
        method: "PUT",
      });
      target = await r.json();
      if (target?.webSocketDebuggerUrl) break;
    } catch {}
    await sleep(150);
  }
  if (!target?.webSocketDebuggerUrl) {
    try { proc.kill(); } catch {}
    throw new Error("could not create a page target");
  }

  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((res, rej) => {
    ws.addEventListener("open", res, { once: true });
    ws.addEventListener("error", rej, { once: true });
  });

  const page = new Page(ws, null, proc, profile);
  page.version = version;
  page.errored = null;
  await page.send("Page.enable", {}, null);
  await page.send("Runtime.enable", {}, null);
  // surface uncaught in-page exceptions on the driver object so a long render
  // can fail fast instead of silently producing black frames
  page.on("Runtime.exceptionThrown", (p) => {
    const d = p?.exceptionDetails;
    page.errored = d?.exception?.description || d?.text || "unknown page exception";
  });
  await page.setViewport(width, height);
  if (url && url !== "about:blank") await page.goto(url);
  return page;
}
