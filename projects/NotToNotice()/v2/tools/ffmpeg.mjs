// Locate the ffmpeg that ships inside the Python imageio-ffmpeg wheel; the
// machine has no ffmpeg on PATH.
import { existsSync, readdirSync } from "node:fs";
import path from "node:path";

const CANDIDATES = [
  "C:/Users/Kurohanekaoruko/AppData/Roaming/Python/Python39/site-packages/imageio_ffmpeg/binaries",
  "C:/Users/Kurohanekaoruko/AppData/Local/Programs/Python/Python39/Lib/site-packages/imageio_ffmpeg/binaries",
];

let cached = null;

export function findFFmpeg() {
  if (cached) return cached;
  for (const dir of CANDIDATES) {
    if (!existsSync(dir)) continue;
    for (const f of readdirSync(dir)) {
      if (f.startsWith("ffmpeg") && f.endsWith(".exe")) {
        cached = path.join(dir, f);
        return cached;
      }
    }
  }
  throw new Error("ffmpeg not found (looked in imageio_ffmpeg/binaries)");
}

export const FFPROBE = () => findFFmpeg(); // ffmpeg can probe too
