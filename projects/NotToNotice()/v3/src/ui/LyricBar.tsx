import React from "react";
import { clamp, smoothstep } from "../lib/math";
import { lyricAt, type Lyric } from "../lib/lyrics";
import { FONT, glow } from "./Type";

const hexA = (hex: string, a: number): string => {
  const h = hex.replace("#", "");
  const n = parseInt(h, 16);
  return `rgba(${(n >> 16) & 255},${(n >> 8) & 255},${n & 255},${a})`;
};

const perChar = (s: string, p: number): number[] => {
  const n = s.length;
  return Array.from({ length: n }, (_, i) => clamp(p * n * 1.06 - i, 0, 1));
};

/** the standing karaoke lyric bar: big EN lead + zh/jp follow */
export const LyricBar: React.FC<{
  t: number;
  keyHex: string;
  hotHex: string;
  suppressed?: boolean;
  y?: number;
  scale?: number;
}> = ({ t, keyHex, hotHex, suppressed = false, y = 866, scale = 1 }) => {
  if (suppressed) return null;
  const L: { line: Lyric; age: number; hold: number; p: number } | null = (() => {
    const s = lyricAt(t);
    return s.line ? { line: s.line, age: s.age, hold: s.hold, p: s.p } : null;
  })();
  if (!L) return null;
  const left = L.hold - L.age;
  const out = L.hold > 2.4 ? smoothstep(0, 0.8, left) : 1;
  const inA = smoothstep(0, 0.35, L.age);
  const alpha = out * inA;
  if (alpha <= 0.01) return null;
  const ens = perChar(L.line.en, L.p);
  const zhs = perChar(L.line.zh, Math.max(0, (L.p - 0.12) / 0.88));
  return (
    <div style={{
      position: "absolute", left: 0, right: 0, top: y, display: "flex",
      flexDirection: "column", alignItems: "center", gap: 14 * scale,
      opacity: alpha, transform: `translateY(${(1 - inA) * 26}px)`,
    }}>
      <div style={{ display: "flex", transform: `scale(${scale})` }}>
        {Array.from(L.line.en).map((ch, i) => (
          <span key={i} style={{
            fontFamily: FONT.disp, fontSize: 54, fontWeight: 700, fontStretch: "84%",
            whiteSpace: "pre",
            color: ch === " " ? "transparent" : `rgb(255,255,255)`,
            opacity: ch === " " ? 0.3 : 0.28 + 0.72 * ens[i],
            textShadow: glow(hotHex, 0.6 + 0.9 * ens[i]),
            transform: `translateY(${(1 - ens[i]) * 6}px)`,
          }}>{ch}</span>
        ))}
      </div>
      <div style={{ display: "flex", transform: `scale(${scale})` }}>
        {Array.from(L.line.zh).map((ch, i) => (
          <span key={i} style={{
            fontFamily: FONT.zh, fontSize: 30, fontWeight: 400, whiteSpace: "pre",
            color: hexA(hotHex, 0.95),
            opacity: 0.16 + 0.78 * zhs[i],
            textShadow: `0 0 ${8 + 14 * zhs[i]}px ${hexA(keyHex, 0.5 + 0.4 * zhs[i])}`,
          }}>{ch}</span>
        ))}
      </div>
    </div>
  );
};
