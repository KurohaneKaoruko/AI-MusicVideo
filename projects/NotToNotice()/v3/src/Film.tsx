import React, { useMemo } from "react";
import { AbsoluteFill, Audio, staticFile, useCurrentFrame, useVideoConfig } from "remotion";
import { FPS } from "./lib/consts";
import { audioAt, recentRings } from "./lib/audio";
import { movementAt, CUTS, glStateAt } from "./lib/timeline";
import { noise1, srand, clamp, smoothstep } from "./lib/math";
import { Stage } from "./gl/Stage";
import { MovementOverlay } from "./ui/Movements";
import { LyricBar } from "./ui/LyricBar";
import { Hud } from "./ui/Hud";

/** movements whose overlay already presents the current lyric */
const LYRIC_SUPPRESS = new Set([2, 4, 6, 8, 9, 10, 12]);

export const Film: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  void fps;
  const t = frame / FPS;

  const a = useMemo(() => audioAt(frame), [frame]);
  const { m, mi, lt, p } = useMemo(() => movementAt(t), [t]);
  const rings = useMemo(() => recentRings(frame), [frame]);
  const gl = useMemo(() => glStateAt(t, frame, a, rings, mi, lt, p), [frame]); // eslint-disable-line react-hooks/exhaustive-deps

  // ---- camera life -------------------------------------------------------
  // DOM layers move via CSS (vector text stays crisp); the GL world receives
  // the same camera through uniforms (procedural zoom = zero resampling).
  let camX = noise1(t * 0.067, 21) * 15 + noise1(t * 0.231, 31) * 5;
  let camY = noise1(t * 0.058, 41) * 10 + noise1(t * 0.192, 51) * 4;
  let camRot = noise1(t * 0.047, 71) * 0.3;
  let cssZoom = 1.035 + 0.011 * noise1(t * 0.131, 61) + 0.015 * a.pulseS;
  for (const c of CUTS) {
    const d = t - c;
    if (d > 0 && d < 0.22) {
      const k = 1 - d / 0.22;
      cssZoom += 0.038 * k * k;
      camX += srand(frame, 7) * 13 * k;
      camY += srand(frame, 9) * 9 * k;
      camRot += srand(frame, 11) * 0.24 * k;
    }
  }
  // accent shakes in the violent movements
  if (mi === 10 || mi === 11 || mi === 12) {
    const sh = Math.max(0, a.onsetR - 0.45) * 12;
    camX += srand(frame, 13) * sh;
    camY += srand(frame, 17) * sh;
  }
  if (mi === 14) {
    const stampK = smoothstep(176.0, 176.12, t) * (1 - smoothstep(176.12, 176.9, t));
    camX += srand(frame, 23) * 16 * stampK;
    camY += srand(frame, 29) * 12 * stampK;
    cssZoom += 0.04 * stampK;
  }
  // doll scene: heartbeat tightening as cracks spread
  if (mi === 9) {
    cssZoom += 0.05 * smoothstep(0.3, 1, p) * (0.5 + 0.5 * a.pulseS);
  }
  const worldZoom = gl.camZoom * cssZoom;

  // ---- cut flash ---------------------------------------------------------
  let cutFlash = 0;
  for (const c of CUTS) {
    const d = t - c;
    if (d >= 0 && d < 0.32) cutFlash = Math.max(cutFlash, Math.pow(1 - d / 0.32, 2) * 0.34);
  }

  const keyHex = gl.pal.keyHex;
  const hotHex = gl.pal.hotHex;
  const altHex = gl.pal.altHex;

  return (
    <AbsoluteFill style={{ background: "#000", overflow: "hidden" }}>
      <style>{`
        ::selection { background: transparent; }
        html, body { overflow: hidden !important; background: #000; }
        ::-webkit-scrollbar { display: none; width: 0; height: 0; }
        * { scrollbar-width: none; }
      `}</style>

      {/* GL stage: pixel-perfect 1:1, camera applied inside the shaders */}
      <Stage state={{ ...gl, camZoom: worldZoom, camX, camY }} />

      {/* DOM typography rides the CSS camera (vector-crisp at any zoom) */}
      <AbsoluteFill style={{
        transform: `translate(${camX.toFixed(2)}px, ${camY.toFixed(2)}px) scale(${cssZoom.toFixed(4)}) rotate(${camRot.toFixed(3)}deg)`,
        transformOrigin: "center center",
        willChange: "transform",
      }}>
        <MovementOverlay
          mi={mi}
          lt={lt}
          p={p}
          t={t}
          frame={frame}
          keyHex={keyHex}
          hotHex={hotHex}
          altHex={altHex}
        />
      </AbsoluteFill>

      {/* stable UI above the camera */}
      <LyricBar t={t} keyHex={keyHex} hotHex={hotHex} suppressed={LYRIC_SUPPRESS.has(mi)} />
      <Hud t={t} keyHex={keyHex} hotHex={hotHex} mvIndex={mi} mvName={m.name} mvTitle={m.title} />

      {/* white flash on movement cuts */}
      {cutFlash > 0.004 ? (
        <AbsoluteFill style={{ background: "#fff", opacity: clamp(cutFlash), mixBlendMode: "screen" }} />
      ) : null}

      <Audio src={staticFile("audio.mp3")} />
    </AbsoluteFill>
  );
};
