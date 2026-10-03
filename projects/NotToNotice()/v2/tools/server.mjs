// tiny static file server for the renderer (no dependencies)
import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import path from "node:path";

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".svg": "image/svg+xml",
  ".wasm": "application/wasm",
};

/**
 * @param root  directory to serve
 * @param opts  { onFrame?: (buf: Buffer, req) => (void|Promise<void>) }
 *              onFrame receives the body of every POST /_frame. Returning
 *              normally (or resolving) answers the page with {ok,written}.
 */
export function startServer(root, opts = {}) {
  const abs = path.resolve(root);
  const server = createServer(async (req, res) => {
    try {
      const url = new URL(req.url, "http://x");

      if (req.method === "POST" && url.pathname === "/_frame") {
        const chunks = [];
        let n = 0;
        for await (const ch of req) {
          chunks.push(ch);
          n += ch.length;
        }
        const body = Buffer.concat(chunks, n);
        if (opts.onFrame) await opts.onFrame(body, req);
        res.writeHead(200, { "content-type": "application/json", "cache-control": "no-store" });
        res.end(JSON.stringify({ ok: true, written: body.length }));
        return;
      }

      if (req.method !== "GET" && req.method !== "HEAD") {
        res.writeHead(405).end("method not allowed");
        return;
      }

      let p = decodeURIComponent(url.pathname);
      if (p === "/") p = "/renderer/index.html";
      const file = path.join(abs, p);
      if (!file.startsWith(abs)) { res.writeHead(403).end("forbidden"); return; }
      const st = await stat(file);
      if (st.isDirectory()) { res.writeHead(404).end("dir"); return; }
      const body = await readFile(file);
      res.writeHead(200, {
        "content-type": MIME[path.extname(file).toLowerCase()] || "application/octet-stream",
        "content-length": body.length,
        "cache-control": "no-store",
      });
      res.end(body);
    } catch (e) {
      if (!res.headersSent) res.writeHead(404, { "content-type": "text/plain" });
      res.end("error: " + e.message);
    }
  });
  return new Promise((resolve) => {
    server.listen(0, "127.0.0.1", () => resolve({
      port: server.address().port,
      url: (p) => `http://127.0.0.1:${server.address().port}${p}`,
      close: () => new Promise((r) => server.close(r)),
    }));
  });
}
