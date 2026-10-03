import React, { useLayoutEffect, useRef } from "react";
import { continueRender, delayRender, useCurrentFrame } from "remotion";
import { Engine } from "./engine";
import type { GlState } from "../lib/timeline";

/**
 * The GL stage. Draws exactly one frame synchronously inside a layout effect
 * (before paint) and gates Remotion's screenshot with delayRender/continueRender
 * so every captured frame includes the finished WebGL draw.
 */
export const Stage: React.FC<{ state: GlState }> = ({ state }) => {
  const ref = useRef<HTMLCanvasElement | null>(null);
  const engine = useRef<Engine | null>(null);
  const frame = useCurrentFrame();

  useLayoutEffect(() => {
    const handle = delayRender("gl-stage");
    let ok = false;
    try {
      if (!engine.current && ref.current) {
        engine.current = new Engine(ref.current);
      }
      engine.current?.draw(state, frame);
      ok = true;
    } finally {
      continueRender(handle);
    }
    // surface GL failures loudly: a failed draw must never render as black
    if (!ok) throw new Error("GL stage draw failed");
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [frame]);

  return (
    <canvas
      ref={ref}
      width={1920}
      height={1080}
      style={{
        position: "absolute",
        inset: 0,
        width: "100%",
        height: "100%",
        display: "block",
      }}
    />
  );
};
