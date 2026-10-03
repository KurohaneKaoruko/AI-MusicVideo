import React from "react";
import { clamp, smoothstep } from "../lib/math";
import { BPM } from "../lib/audio";
import { FONT, pad } from "./Type";

const MACHINES = ["PROPATOR", "ECCLESIA", "NOEIN", "ANTHROPOS", "LETHEIA", "LOGOS", "ZOE", "ENOA"];

const hexA = (hex: string, a: number): string => {
  const h = hex.replace("#", "");
  const n = parseInt(h, 16);
  return `rgba(${(n >> 16) & 255},${(n >> 8) & 255},${n & 255},${a})`;
};

/** standing telemetry: machine strand + E×P counter + timecode */
export const Hud: React.FC<{
  t: number;
  keyHex: string;
  hotHex: string;
  mvIndex: number;
  mvName: string;
  mvTitle: string;
}> = ({ t, keyHex, hotHex, mvIndex, mvName, mvTitle }) => {
  const gate = smoothstep(10.6, 12.4, t) * (1 - smoothstep(169.5, 173.0, t));
  if (gate <= 0.01) return null;
  const x0 = 96;
  const x1 = 1920 - 96;
  const y = 1014;
  const prog = clamp(t / 182.0);
  const px = x0 + (x1 - x0) * prog;
  const expPct = (prog * prog * 100).toFixed(1);
  const tc = `${pad(t / 60, 2)}:${pad(t % 60, 2)}:${pad((t * 60) % 60, 2)}`;
  const near = (i: number): number => Math.exp(-Math.pow((prog - i / 7) * 20, 2));

  return (
    <div style={{ position: "absolute", inset: 0, opacity: gate, pointerEvents: "none" }}>
      {/* strand */}
      <div style={{
        position: "absolute", left: x0, top: y, width: x1 - x0, height: 2,
        background: hexA(keyHex, 0.18),
      }} />
      <div style={{
        position: "absolute", left: x0, top: y, width: px - x0, height: 2,
        background: `linear-gradient(90deg, ${hexA(keyHex, 0.25)}, ${hotHex})`,
        boxShadow: `0 0 10px ${hexA(keyHex, 0.8)}, 0 0 22px ${hexA(keyHex, 0.4)}`,
      }} />
      {MACHINES.map((nm, i) => {
        const u = i / 7;
        const x = x0 + (x1 - x0) * u;
        const reached = prog >= u;
        const nr = near(i);
        const size = 5 + 6 * nr;
        return (
          <React.Fragment key={nm}>
            <div style={{
              position: "absolute", left: x - size / 2, top: y - size / 2 + 1, width: size, height: size,
              borderRadius: "50%", transform: "rotate(45deg)",
              background: reached ? hotHex : hexA(keyHex, 0.5),
              boxShadow: reached ? `0 0 10px ${hotHex}` : undefined,
              opacity: 0.5 + 0.5 * nr,
            }} />
            <div style={{
              position: "absolute", left: x, top: y - 34, transform: "translateX(-50%)",
              fontFamily: FONT.mono, fontSize: 13, letterSpacing: 2.5,
              color: hotHex, opacity: nr * 0.95,
              textShadow: `0 0 8px ${hexA(keyHex, 0.7)}`,
            }}>{nm}</div>
          </React.Fragment>
        );
      })}
      <div style={{
        position: "absolute", right: 96, top: y - 44, fontFamily: FONT.mono, fontSize: 17,
        letterSpacing: 2.5, color: hotHex, textShadow: `0 0 10px ${hexA(keyHex, 0.7)}`,
        whiteSpace: "nowrap",
      }}>E×P {expPct.padStart(5)}%</div>

      {/* top-left session tag */}
      <div style={{
        position: "absolute", left: 70, top: 54, fontFamily: FONT.mono, fontSize: 16,
        letterSpacing: 3.5, color: hexA(keyHex, 0.8), textShadow: `0 0 8px ${hexA(keyHex, 0.4)}`,
      }}>
        DEI EX MACHINA // SESSION 0xENOA
      </div>
      <div style={{
        position: "absolute", left: 70, top: 82, fontFamily: FONT.mono, fontSize: 14,
        letterSpacing: 3, color: hexA(keyHex, 0.5),
      }}>
        {String(mvIndex + 1).padStart(2, "0")} / 15 — {mvName} — {mvTitle}
      </div>

      {/* top-right timecode */}
      <div style={{
        position: "absolute", right: 70, top: 54, fontFamily: FONT.mono, fontSize: 16,
        letterSpacing: 3, color: hexA(keyHex, 0.8), textShadow: `0 0 8px ${hexA(keyHex, 0.4)}`,
      }}>
        {tc} · {BPM.toFixed(1)} BPM
      </div>
    </div>
  );
};
