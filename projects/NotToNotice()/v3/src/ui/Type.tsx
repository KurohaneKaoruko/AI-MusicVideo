import React from "react";
import { spring, useVideoConfig } from "remotion";
import { clamp, lerp } from "../lib/math";

export type FontKind = "disp" | "mono" | "jp" | "zh";

export const FONT: Record<FontKind, string> = {
  disp: 'Bahnschrift, "Segoe UI", sans-serif',
  mono: '"Cascadia Code", Consolas, monospace',
  jp: '"Yu Gothic", "Meiryo", sans-serif',
  zh: '"Microsoft YaHei", "Yu Gothic", sans-serif',
};

/** layered neon glow for dark bg */
export const glow = (color: string, size = 1): string =>
  [
    `0 0 ${10 * size}px ${color}`,
    `0 0 ${28 * size}px ${color}88`,
    `0 0 ${68 * size}px ${color}44`,
  ].join(", ");

const eased = (k: number): number => 1 - Math.pow(1 - clamp(k), 3);

export type CharsProps = {
  text: string;
  /** absolute seconds when the entrance begins */
  at: number;
  t: number;
  stagger?: number;
  dur?: number;
  rise?: number;
  blur?: number;
  size: number;
  weight?: number | string;
  font?: FontKind;
  color?: string;
  glowCol?: string;
  glowSize?: number;
  stretch?: number;
  italic?: boolean;
  /** per-char karaoke: value 0..1 per char (1 = lit) */
  lit?: (i: number, n: number) => number;
  litDim?: number;
  exitAt?: number;
  exitDur?: number;
  exitDy?: number;
  rgbSplit?: number;
  opacity?: number;
  style?: React.CSSProperties;
  charStyle?: (i: number, n: number, on: number) => React.CSSProperties;
};

/** per-character choreographed text. every value is a pure function of t. */
export const Chars: React.FC<CharsProps> = ({
  text, at, t,
  stagger = 0.035, dur = 0.5,
  rise = 26, blur = 8,
  size, weight = 700,
  font = "disp",
  color = "#fff",
  glowCol, glowSize = 1,
  stretch, italic,
  lit, litDim = 0.38,
  exitAt, exitDur = 0.7, exitDy = -18,
  rgbSplit = 0,
  opacity = 1,
  style,
  charStyle,
}) => {
  const { fps } = useVideoConfig();
  const chars = Array.from(text);
  const exitK = exitAt !== undefined ? clamp((t - exitAt) / exitDur) : 0;
  const globalAlpha = opacity * (1 - eased(exitK));
  if (globalAlpha <= 0.004) return null;
  const dy = exitDy * eased(exitK);
  const outBlur = exitAt !== undefined ? eased(exitK) * 6 : 0;

  return (
    <div
      style={{
        fontFamily: FONT[font],
        fontSize: size,
        fontWeight: weight,
        fontStyle: italic ? "italic" : undefined,
        fontStretch: stretch !== undefined ? `${stretch}%` : undefined,
        display: "flex",
        flexWrap: "wrap",
        justifyContent: style?.textAlign === "left" ? "flex-start" : style?.textAlign === "right" ? "flex-end" : "center",
        opacity: globalAlpha,
        transform: `translateY(${dy}px)`,
        filter: outBlur > 0.3 ? `blur(${outBlur}px)` : undefined,
        lineHeight: 1.16,
        ...style,
      }}
    >
      {chars.map((ch, i) => {
        const sf = (t - at - i * stagger) * fps;
        const k = sf <= 0 ? 0 : spring({ frame: sf, fps, config: { damping: 22, stiffness: 170, mass: 0.9 }, durationInFrames: Math.max(1, Math.round(dur * fps)) });
        const e = eased(k);
        const litK = lit ? lerp(litDim, 1, clamp(lit(i, chars.length))) : 1;
        const col = lit ? mixColor(color, litDim, lit(i, chars.length)) : color;
        return (
          <span
            key={i}
            style={{
              display: "inline-block",
              opacity: e * litK,
              transform: `translateY(${(1 - e) * rise}px)`,
              filter: (1 - e) * blur > 0.4 ? `blur(${(1 - e) * blur}px)` : undefined,
              textShadow: glowCol ? glow(glowCol, glowSize) : undefined,
              color: col,
              whiteSpace: "pre",
              ...(charStyle ? charStyle(i, chars.length, k) : {}),
            }}
          >
            {ch}
          </span>
        );
      })}
    </div>
  );
};

const hex2rgb = (hex: string): [number, number, number] => {
  const h = hex.replace("#", "");
  const n = parseInt(h.length === 3 ? h.split("").map((c) => c + c).join("") : h, 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
};

/** dim a hex colour toward black by factor (0=black-ish, 1=full) */
export const mixColor = (hex: string, dim: number, k: number): string => {
  const [r, g, b] = hex2rgb(hex);
  const f = lerp(dim, 1, clamp(k));
  return `rgb(${Math.round(r * f)},${Math.round(g * f)},${Math.round(b * f)})`;
};

// ------------------------------------------------------------- typewriter ---

export const TypeLine: React.FC<{
  text: string;
  at: number;
  t: number;
  cps?: number;             // chars per second
  size?: number;
  font?: FontKind;
  weight?: number | string;
  color?: string;
  glowCol?: string;
  cursor?: boolean;
  style?: React.CSSProperties;
}> = ({ text, at, t, cps = 26, size = 26, font = "mono", weight = 400, color = "#fff", glowCol, cursor = true, style }) => {
  const n = Math.max(0, Math.floor((t - at) * cps));
  const shown = text.slice(0, n);
  const typing = n < text.length;
  const blink = Math.floor(t * 2.6) % 2 === 0;
  return (
    <div style={{
      fontFamily: FONT[font], fontSize: size, fontWeight: weight, color,
      whiteSpace: "pre", textShadow: glowCol ? `0 0 12px ${glowCol}` : undefined, ...style,
    }}>
      {shown}
      {cursor && (typing || blink) ? (
        <span style={{ opacity: typing ? 1 : 0.85 }}>▌</span>
      ) : null}
    </div>
  );
};

// ------------------------------------------------------------------ panel ---

export const Panel: React.FC<{
  x: number; y: number; w: number; h: number;
  color: string; alpha?: number; t: number; at: number;
  children?: React.ReactNode; label?: string;
}> = ({ x, y, w, h, color, alpha = 1, t, at, children, label }) => {
  const k = clamp((t - at) / 0.45);
  if (k <= 0) return null;
  const e = 1 - Math.pow(1 - k, 3);
  const corner = 22;
  const cStyle = (sx: number, sy: number): React.CSSProperties => ({
    position: "absolute", width: corner, height: corner,
    [sx < 0 ? "left" : "right"]: Math.abs(sx),
    [sy < 0 ? "top" : "bottom"]: Math.abs(sy),
    [sx < 0 ? "borderLeft" : "borderRight"]: `2px solid ${color}`,
    [sy < 0 ? "borderTop" : "borderBottom"]: `2px solid ${color}`,
    opacity: e,
  } as React.CSSProperties);
  return (
    <div style={{
      position: "absolute", left: x, top: y, width: w, height: h,
      opacity: alpha * e,
      background: `linear-gradient(180deg, ${color}0d, transparent 65%)`,
      boxShadow: `inset 0 0 0 1px ${color}26, 0 0 ${34}px ${color}14`,
      transform: `translateY(${(1 - e) * 14}px)`,
    }}>
      {children}
      <div style={cStyle(-8, -8)} />
      <div style={cStyle(8, -8)} />
      <div style={cStyle(8, 8)} />
      <div style={cStyle(-8, 8)} />
      {label ? (
        <div style={{
          position: "absolute", top: -11, left: 18, padding: "0 8px",
          fontFamily: FONT.mono, fontSize: 15, letterSpacing: 3, color,
          background: "#02040a", textShadow: `0 0 10px ${color}`,
        }}>{label}</div>
      ) : null}
    </div>
  );
};

/** fraction formatting helper */
export const pad = (v: number, n: number): string => String(Math.floor(v)).padStart(n, "0");
