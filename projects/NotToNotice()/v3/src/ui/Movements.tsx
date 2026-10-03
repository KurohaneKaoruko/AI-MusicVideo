import React from "react";
import { clamp, lerp, srand, smoothstep } from "../lib/math";
import { BEAT_PERIOD } from "../lib/audio";
import { Chars, TypeLine, Panel, FONT, glow, pad } from "./Type";

export type Ctx = {
  lt: number; p: number; t: number; frame: number;
  keyHex: string; hotHex: string; altHex: string;
};

const V = 1920;
const H = 1080;
const cxs = V / 2;

const dim = (hex: string, k: number) => `rgba(${parseInt(hex.slice(1, 3), 16)},${parseInt(hex.slice(3, 5), 16)},${parseInt(hex.slice(5, 7), 16)},${k})`;

// ─────────────────────────────────────────────────────────── 00 NULL ───────
const Null: React.FC<Ctx> = ({ lt, t, frame, keyHex, hotHex }) => (
  <>
    <div style={{
      position: "absolute", left: cxs - 16, top: H * 0.52,
      fontFamily: FONT.mono, fontSize: 44, color: hotHex,
      textShadow: glow(keyHex, 1.2),
      opacity: smoothstep(1.2, 1.8, lt) * (0.75 + 0.25 * (Math.floor(t * 2.2) % 2)),
    }}>▌</div>
    <TypeLine text="enoa://kernel — consciousness service" at={2.3} t={lt} cps={18}
      size={20} color={dim(keyHex, 0.75)} glowCol={dim(keyHex, 0.3)}
      style={{ position: "absolute", left: 70, bottom: 64, letterSpacing: 2 }} />
    {lt > 4.4 ? (
      <Chars text="> NotToNotice();" at={4.55} t={lt} size={30} font="mono" weight={500}
        color={dim(hotHex, 0.8)} glowCol={dim(keyHex, 0.5)} rise={10}
        style={{ position: "absolute", left: cxs - 165, top: H * 0.52 - 70 }} />
    ) : null}
  </>
);

// ─────────────────────────────────────────────────────────── 01 BOOT ───────
const ecgY = (u: number): number => {
  // ECG-shaped waveform, amplitude dies out with u (flatline)
  const beats = [[0.05, 1], [0.13, 0.92], [0.22, 0.78], [0.32, 0.6], [0.44, 0.42], [0.57, 0.26], [0.71, 0.12], [0.86, 0.04]];
  let s = 0;
  for (const [bu, ba] of beats) {
    const d = u - bu;
    s += ba * 0.16 * Math.exp(-((d + 0.014) ** 2) / (2 * 0.006 ** 2));
    s -= ba * 0.24 * Math.exp(-((d + 0.004) ** 2) / (2 * 0.0028 ** 2));
    s += ba * 1.0 * Math.exp(-(d * d) / (2 * 0.0026 ** 2));
    s -= ba * 0.32 * Math.exp(-((d - 0.005) ** 2) / (2 * 0.004 ** 2));
    s += ba * 0.2 * Math.exp(-((d - 0.028) ** 2) / (2 * 0.012 ** 2));
  }
  return s;
};

const Boot: React.FC<Ctx> = ({ lt, keyHex, hotHex, altHex }) => {
  const x0 = 70, x1 = V - 70, y0 = H * 0.56;
  const scan = clamp((lt - 0.15) / 4.2);
  const pts: string[] = [];
  const N = 480;
  for (let i = 0; i <= N; i++) {
    const u = (i / N) * scan;
    pts.push(`${x0 + (x1 - x0) * (i / N)},${y0 - ecgY(u) * 110}`);
  }
  const red = "#ff5a6a";
  const line = (txt: string, at: number, valCol?: string): React.ReactNode => (
    <TypeLine key={txt} text={txt} at={at} t={lt} cps={44} size={22}
      color={dim(keyHex, 0.85)} glowCol={dim(keyHex, 0.25)} cursor={false}
      style={{ whiteSpace: "pre", letterSpacing: 1.5 }} />
  );
  return (
    <>
      <Panel x={x0 - 28} y={150} w={720} h={272} color={keyHex} at={0.1} t={lt} label="DEI EX MACHINA OS — v∞.3">
        <div style={{ padding: "26px 30px", display: "flex", flexDirection: "column", gap: 9 }}>
          {line("EDEN SOWING STRUCT ....... ONLINE", 0.45)}
          {line("8 DIVINE MACHINES ....... SYNCED", 0.85)}
          {line("PERSONALITY DATA ........ LOST", 1.25)}
          {line("SUBJECT ................. LEBEN DISTEL", 1.65)}
          {line("CAUSE OF DEATH .......... CENTRIFUGAL SYNDROME", 2.05)}
        </div>
      </Panel>
      {lt > 4.5 ? (
        <>
          <Chars text="— FLATLINE —" at={4.55} t={lt} size={24} font="mono" color={red}
            glowCol={dim(red, 0.7)} style={{ position: "absolute", right: 74, top: y0 - 92 }} />
          <Chars text="死亡確認 / DEATH CONFIRMED" at={4.8} t={lt} size={19} font="jp" color={dim(red, 0.8)}
            glowCol={dim(red, 0.4)} style={{ position: "absolute", right: 74, top: y0 - 58 }} />
        </>
      ) : null}
      {lt > 2.9 ? (
        <>
          <Chars text="人類は、遠心症候群で死に絶えた。" at={2.95} t={lt} size={58} font="jp" weight={700}
            color={hotHex} glowCol={keyHex} stagger={0.03} rise={20}
            style={{ position: "absolute", left: 0, right: 0, top: 418, justifyContent: "center", display: "flex" }} />
          <Chars text="HUMANITY  //  EXTINCT" at={3.55} t={lt} size={24} font="mono" weight={500}
            color={dim(altHex, 0.95)} glowCol={dim(keyHex, 0.4)} stagger={0.02} rise={12}
            style={{ position: "absolute", left: 0, right: 0, top: 498, justifyContent: "center", display: "flex", letterSpacing: 10 }} />
        </>
      ) : null}
    </>
  );
};

// ─────────────────────────────────────────────────────────── 02 FAIRY ──────
const Fairy: React.FC<Ctx> = ({ lt, keyHex, hotHex }) => (
  <>
    <Chars text="LIKE BLUE FAIRY" at={0.3} t={lt} size={132} weight={700} stretch={80}
      color={hotHex} glowCol={keyHex} glowSize={1.5} stagger={0.045} rise={44} blur={12}
      exitAt={5.6}
      style={{ position: "absolute", left: 120, top: H * 0.60, textAlign: "left" }} />
    <Chars text="— 青い妖精のように —" at={0.95} t={lt} size={30} font="jp" weight={400}
      color={dim(keyHex, 0.9)} glowCol={dim(keyHex, 0.45)} stagger={0.02} rise={14}
      exitAt={5.6}
      style={{ position: "absolute", left: 126, top: H * 0.60 + 158 }} />
    <Chars text="如若蔚蓝色的妖姬啊" at={1.3} t={lt} size={22} font="zh" weight={400}
      color={dim(keyHex, 0.65)} glowCol={dim(keyHex, 0.3)} stagger={0.02} rise={10}
      exitAt={5.6}
      style={{ position: "absolute", left: 128, top: H * 0.60 + 204 }} />
    <TypeLine text="> mission packet received" at={2.8} t={lt} cps={30} size={22}
      color={dim(keyHex, 0.8)} glowCol={dim(keyHex, 0.3)} cursor={false}
      style={{ position: "absolute", right: 90, top: 150, letterSpacing: 2 }} />
    <TypeLine text="> void execute( mission );" at={3.9} t={lt} cps={26} size={30} weight={500}
      color={dim(hotHex, 0.92)} glowCol={dim(keyHex, 0.4)}
      style={{ position: "absolute", right: 90, top: 190, letterSpacing: 1 }} />
  </>
);

// ─────────────────────────────────────────────────────────── 03 QUERY ──────
const SPIN = "|/-\\";
const Query: React.FC<Ctx> = ({ lt, keyHex, hotHex, altHex }) => {
  const barK = smoothstep(1.2, 6.5, lt);
  const cyan = "#59f0f0";
  const magenta = "#ff4f9a";
  const ifelse = smoothstep(6.9, 7.5, lt);
  return (
    <>
      <Panel x={140} y={300} w={920} h={200} color={keyHex} at={0.15} t={lt} label="QUERY">
        <div style={{ padding: "30px 34px", display: "flex", flexDirection: "column", gap: 14 }}>
          <TypeLine text={'> find("the meaning of my existence")'} at={0.3} t={lt} cps={30} size={27} weight={500}
            color={dim(hotHex, 0.95)} glowCol={dim(keyHex, 0.35)} cursor={false} />
          {lt > 1.2 ? (
            <div style={{ fontFamily: FONT.mono, fontSize: 21, color: dim(keyHex, 0.7), letterSpacing: 1 }}>
              scanning 8 191 804 617 souls {SPIN[Math.floor(lt * 9) % 4]}
            </div>
          ) : null}
          {lt > 1.2 ? (
            <div style={{ width: "82%", height: 10, border: `1px solid ${dim(keyHex, 0.4)}`, padding: 2 }}>
              <div style={{
                width: `${barK * 100}%`, height: "100%",
                background: `linear-gradient(90deg, ${dim(keyHex, 0.25)}, ${keyHex})`,
                boxShadow: `0 0 14px ${dim(keyHex, 0.7)}`,
              }} />
            </div>
          ) : null}
        </div>
      </Panel>
      {lt > 7.4 ? (
        <>
          <Chars text="0 results" at={7.5} t={lt} size={104} font="mono" weight={700} stagger={0.05}
            color={hotHex} glowCol={keyHex} glowSize={1.5} rise={54} blur={12} exitAt={12.4}
            style={{ position: "absolute", left: 0, right: 0, top: 560, justifyContent: "center", display: "flex" }} />
          <Chars text="見つかりませんでした" at={8.1} t={lt} size={28} font="jp" color={dim(altHex, 1)}
            glowCol={dim(keyHex, 0.4)} exitAt={12.4}
            style={{ position: "absolute", left: 0, right: 0, top: 700, justifyContent: "center", display: "flex", letterSpacing: 8 }} />
        </>
      ) : null}
      {ifelse > 0 ? (
        <>
          <Chars text="if ( EVE == human ) { … }" at={6.95} t={lt} size={40} font="mono" weight={500} stagger={0.02}
            color={cyan} glowCol={dim(cyan, 0.6)} rise={26}
            style={{ position: "absolute", left: 120, top: 756, textAlign: "left" }} />
          <Chars text="else { suppress( emotion ); }" at={7.25} t={lt} size={40} font="mono" weight={500} stagger={0.02}
            color={magenta} glowCol={dim(magenta, 0.6)} rise={26}
            style={{ position: "absolute", right: 120, top: 756, textAlign: "right" }} />
          <Chars text="感情を抑制 — 扁平化" at={7.7} t={lt} size={22} font="jp" color={dim(magenta, 0.85)}
            glowCol={dim(magenta, 0.35)} rise={14}
            style={{ position: "absolute", right: 124, top: 818 }} />
        </>
      ) : null}
    </>
  );
};

// ─────────────────────────────────────────────────────────── 04 FORGE ──────
const Forge: React.FC<Ctx> = ({ lt, keyHex, hotHex, altHex }) => {
  const pct = Math.floor(smoothstep(0.4, 9.6, lt) * 100);
  return (
    <>
      <Chars text="Create Soul For EVE" at={2.5} t={lt} size={64} weight={700} stretch={82}
        color={hotHex} glowCol={keyHex} glowSize={1.3} stagger={0.03} rise={30} exitAt={11.6}
        style={{ position: "absolute", left: 0, right: 0, top: H * 0.72, justifyContent: "center", display: "flex" }} />
      <Chars text="我于白夜躁动的创造之灵" at={3.1} t={lt} size={27} font="zh" color={dim(altHex, 1)}
        glowCol={dim(keyHex, 0.4)} exitAt={11.6}
        style={{ position: "absolute", left: 0, right: 0, top: H * 0.72 + 86, justifyContent: "center", display: "flex", letterSpacing: 7 }} />
      <div style={{
        position: "absolute", right: 84, bottom: 120, fontFamily: FONT.mono, fontSize: 22,
        color: dim(keyHex, 0.85), textShadow: `0 0 12px ${dim(keyHex, 0.4)}`, letterSpacing: 2,
      }}>
        soul casting … {String(pct).padStart(3, "0")}%
      </div>
      {lt > 10.6 ? (
        <Chars text="E.V.E. — logged in" at={10.7} t={lt} size={34} font="mono" weight={600} stagger={0.03}
          color={hotHex} glowCol={keyHex} rise={18}
          style={{ position: "absolute", left: 0, right: 0, top: H * 0.40, justifyContent: "center", display: "flex", letterSpacing: 4 }} />
      ) : null}
    </>
  );
};

// ─────────────────────────────────────────────────────────── 05 ADMIT ──────
const Admit: React.FC<Ctx> = ({ lt, keyHex }) => (
  <>
    <TypeLine text="tear[0] = { volume: ∞, cause: undefined }" at={2.6} t={lt} cps={24} size={20}
      color={dim(keyHex, 0.7)} glowCol={dim(keyHex, 0.3)} cursor={false}
      style={{ position: "absolute", right: 84, bottom: 150, letterSpacing: 1 }} />
    {lt > 12.4 ? (
      <Chars text="pretend…" at={12.5} t={lt} size={40} font="disp" weight={300} stretch={88}
        color={dim(keyHex, 0.6)} glowCol={dim(keyHex, 0.3)} stagger={0.06} rise={10}
        style={{ position: "absolute", left: 0, right: 0, top: H * 0.30, justifyContent: "center", display: "flex", letterSpacing: 22 }} />
    ) : null}
  </>
);

// ─────────────────────────────────────────────────────────── 06 REDACT ─────
const Redact: React.FC<Ctx> = ({ lt, keyHex, hotHex }) => {
  const gold = "#ffc637";
  const bars = [0, 1, 2].map((i) => {
    const appear = 0.7 + i * 0.5;
    const k = clamp((lt - appear) / 0.7);
    if (k <= 0) return null;
    const e = 1 - Math.pow(1 - k, 3);
    const w = [620, 420, 360][i];
    const x = [500, 1180, 900][i];
    const y = [462, 470, 480][i];
    return (
      <div key={i} style={{
        position: "absolute", left: x, top: y, width: w * e, height: 58,
        background: "#050300", border: `1px solid ${dim(gold, 0.75)}`,
        boxShadow: `0 0 22px ${dim(gold, 0.3)}`,
        display: "flex", alignItems: "center", justifyContent: "center",
        fontFamily: FONT.mono, fontSize: 24, letterSpacing: 14, color: dim(gold, 0.95),
        whiteSpace: "nowrap", overflow: "hidden",
      }}>REMOVED</div>
    );
  });
  return (
    <>
      {lt < 5.0 ? (
        <>
          <Chars text="Pretend Not To Notice" at={0.05} t={lt} size={98} weight={700} stretch={82}
            color={hotHex} glowCol={keyHex} glowSize={1.4} stagger={0.028} rise={36}
            exitAt={4.4}
            style={{ position: "absolute", left: 0, right: 0, top: 430, justifyContent: "center", display: "flex" }} />
          <Chars text="涙を、見なかったことにする" at={0.6} t={lt} size={28} font="jp" color={dim(gold, 0.85)}
            glowCol={dim(gold, 0.4)} exitAt={4.4}
            style={{ position: "absolute", left: 0, right: 0, top: 570, justifyContent: "center", display: "flex", letterSpacing: 8 }} />
          {bars}
        </>
      ) : null}
      {lt > 5.0 ? (
        <>
          <Chars text="NotToNotice();" at={5.05} t={lt} size={100} font="mono" weight={700} stagger={0.03}
            color={hotHex} glowCol={keyHex} glowSize={1.5} rise={40} blur={10}
            style={{ position: "absolute", left: 0, right: 0, top: 420, justifyContent: "center", display: "flex" }} />
          <Chars text="エノア ／ CV. 遠野ひかる" at={5.8} t={lt} size={26} font="jp"
            color={dim(gold, 0.92)} glowCol={dim(gold, 0.35)} rise={16}
            style={{ position: "absolute", left: 130, top: 856 }} />
          <Chars text="CRYMACHINA OP テーマ — composed by 削除 / lyrics: ASPRGuS" at={6.3} t={lt} size={19} font="jp"
            color={dim(gold, 0.62)} rise={12}
            style={{ position: "absolute", left: 132, top: 902 }} />
        </>
      ) : null}
    </>
  );
};

// ─────────────────────────────────────────────────────────── 07 GARDEN ─────
const SEASONS: [string, string][] = [
  ["春 — SPRING", "petals fall over the field"],
  ["夏 — SUMMER", "fireflies drift in the warm dark"],
  ["秋 — AUTUMN", "leaves count the passing days"],
  ["冬 — WINTER", "snow silences the garden"],
];
const Garden: React.FC<Ctx> = ({ lt, keyHex, hotHex }) => {
  const seg = Math.floor(lt / 4.9);
  const season = SEASONS[seg % 4];
  const chosen = smoothstep(14.3, 15.1, lt);
  const echo = [0, 1, 2].map((i) => {
    const k = clamp((lt - 14.5 - i * 0.35) / 1.4);
    if (k <= 0) return null;
    return (
      <div key={i} style={{
        position: "absolute", left: 0, right: 0, top: 430, display: "flex", justifyContent: "center",
        fontFamily: FONT.jp, fontSize: 62, fontWeight: 700, color: hotHex,
        textShadow: glow(keyHex, 1.3),
        opacity: (1 - k) * 0.9,
        transform: `scale(${1 + k * 1.6})`,
      }}>あなたは、選ばれた。</div>
    );
  });
  return (
    <>
      <Chars text="箱庭 — IMITATION GARDEN" at={0.4} t={lt} size={26} font="mono" weight={500}
        color={dim(keyHex, 0.85)} glowCol={dim(keyHex, 0.35)} rise={12}
        style={{ position: "absolute", left: 0, right: 0, top: 120, justifyContent: "center", display: "flex", letterSpacing: 8 }} />
      <div style={{
        position: "absolute", left: 0, right: 0, bottom: 150, display: "flex", justifyContent: "center",
        flexDirection: "column", alignItems: "center", gap: 10,
      }}>
        <div style={{
          fontFamily: FONT.jp, fontSize: 34, color: dim(hotHex, 0.9),
          textShadow: `0 0 16px ${dim(keyHex, 0.5)}`, letterSpacing: 10,
          opacity: 1 - chosen,
        }}>{season[0]}</div>
        <div style={{
          fontFamily: FONT.disp, fontSize: 18, color: dim(keyHex, 0.55), letterSpacing: 4,
          opacity: 1 - chosen,
        }}>{season[1]}</div>
        <div style={{ fontFamily: FONT.mono, fontSize: 19, color: dim(keyHex, 0.6), letterSpacing: 3 }}>
          CYCLE {pad(seg + 1, 3)} / 四季巡り
        </div>
      </div>
      {chosen > 0 ? echo : null}
      {chosen > 0.3 ? (
        <Chars text="3 humanlike souls found in the box garden" at={15.1} t={lt} size={20} font="mono"
          color={dim(keyHex, 0.75)} glowCol={dim(keyHex, 0.3)} rise={10}
          style={{ position: "absolute", left: 0, right: 0, top: 540, justifyContent: "center", display: "flex", letterSpacing: 3 }} />
      ) : null}
    </>
  );
};

// ─────────────────────────────────────────────────────────── 08 ASCENT ─────
const Ascent: React.FC<Ctx> = ({ lt, keyHex, hotHex, altHex }) => (
  <>
    {lt < 6.3 ? (
      <>
        <Chars text="what do I dream of...?" at={0.1} t={lt} size={68} weight={300} italic stretch={88}
          color={hotHex} glowCol={keyHex} glowSize={1.3} stagger={0.03} rise={26} exitAt={5.4}
          style={{ position: "absolute", left: 0, right: 0, top: 300, justifyContent: "center", display: "flex" }} />
        <TypeLine text="> void achieve_my_dream();" at={3.2} t={lt} cps={24} size={30} weight={500}
          color={dim(altHex, 1)} glowCol={dim(keyHex, 0.4)} cursor={false}
          style={{ position: "absolute", left: 0, right: 0, top: 420, textAlign: "center" }} />
      </>
    ) : null}
    {lt > 6.35 ? (
      <>
        <Chars text="until I get the fruit in the Eden" at={6.5} t={lt} size={58} weight={700} stretch={84}
          color={hotHex} glowCol={keyHex} glowSize={1.4} stagger={0.026} rise={30}
          style={{ position: "absolute", left: 0, right: 0, top: H * 0.74, justifyContent: "center", display: "flex" }} />
        <Chars text="エデンの禁果 — E×P" at={7.2} t={lt} size={24} font="jp" color={dim(altHex, 1)}
          glowCol={dim(keyHex, 0.4)} rise={14}
          style={{ position: "absolute", left: 0, right: 0, top: H * 0.74 + 84, justifyContent: "center", display: "flex", letterSpacing: 8 }} />
        <div style={{
          position: "absolute", right: 90, top: 140, fontFamily: FONT.mono, fontSize: 20,
          color: dim(keyHex, 0.75), letterSpacing: 3, textShadow: `0 0 10px ${dim(keyHex, 0.3)}`,
        }}>EDEN — 4.2×10⁶ ly  ::  warp 0.99…</div>
      </>
    ) : null}
  </>
);

// ─────────────────────────────────────────────────────────── 09 DOLL ───────
const Doll: React.FC<Ctx> = ({ lt, t, keyHex, hotHex }) => {
  const iter = 4491 + Math.max(0, Math.floor((lt - 0.3) / BEAT_PERIOD));
  const crit = smoothstep(5.4, 5.8, lt);
  const flick = 0.6 + 0.4 * (Math.floor(t * 6) % 2);
  return (
    <>
      <Chars text="while ( be_a_doll ) {" at={0.2} t={lt} size={54} font="mono" weight={600} stagger={0.02}
        color={dim(hotHex, 0.95)} glowCol={dim(keyHex, 0.5)} rise={20}
        style={{ position: "absolute", left: 0, right: 0, top: 170, justifyContent: "center", display: "flex" }} />
      <Chars text="哪怕做一具人偶也好" at={0.75} t={lt} size={22} font="zh" color={dim(keyHex, 0.6)}
        glowCol={dim(keyHex, 0.3)} rise={12}
        style={{ position: "absolute", left: 0, right: 0, top: 246, justifyContent: "center", display: "flex", letterSpacing: 6 }} />
      {lt > 3.4 ? (
        <Chars text="focus( mission );" at={3.5} t={lt} size={44} font="mono" weight={500} stagger={0.02}
          color={dim(keyHex, 0.95)} glowCol={dim(keyHex, 0.4)} rise={16}
          style={{ position: "absolute", left: 0, right: 0, top: 300, justifyContent: "center", display: "flex" }} />
      ) : null}
      <div style={{
        position: "absolute", right: 90, bottom: 175, fontFamily: FONT.mono, fontSize: 24,
        color: dim(keyHex, 0.85), textShadow: `0 0 12px ${dim(keyHex, 0.35)}`, letterSpacing: 2,
      }}>iteration: {pad(iter, 9)}</div>
      {crit > 0 ? (
        <div style={{
          position: "absolute", left: 0, right: 0, top: 850, display: "flex", justifyContent: "center",
          fontFamily: FONT.mono, fontSize: 30, fontWeight: 700, letterSpacing: 6,
          color: "#ff3b46", textShadow: `0 0 18px ${dim("#ff3b46", 0.8)}`,
          opacity: crit * flick,
        }}>CRITICAL — 胸の亀裂 :: crack propagating</div>
      ) : null}
    </>
  );
};

// ─────────────────────────────────────────────────────────── 10 FIGHT ──────
const Fight: React.FC<Ctx> = ({ lt, frame, keyHex, hotHex }) => {
  const slam = smoothstep(0.15, 0.4, lt);
  const shake = slam > 0 ? Math.max(0, 1 - (lt - 0.2) * 2.4) : 0;
  const dx = srand(frame, 3) * 14 * shake;
  const dy = srand(frame, 9) * 10 * shake;
  return (
    <>
      {lt < 2.2 ? (
        <Chars text="cause you light my way of life" at={0.05} t={lt} size={56} weight={600} stretch={84}
          color={hotHex} glowCol={keyHex} glowSize={1.3} stagger={0.026} rise={26} exitAt={1.6} exitDur={0.35}
          style={{ position: "absolute", left: 0, right: 0, top: 280, justifyContent: "center", display: "flex" }} />
      ) : null}
      {slam > 0 ? (
        <div style={{
          position: "absolute", left: 0, right: 0, top: 420, display: "flex", justifyContent: "center",
          transform: `translate(${dx}px, ${dy}px)`,
        }}>
          <Chars text="FIGHT FOR YOU" at={0.18} t={lt} size={150} weight={700} stretch={78}
            color={hotHex} glowCol={keyHex} glowSize={1.7} stagger={0.022} rise={0} blur={0}
            rgbSplit={1}
            charStyle={() => ({
              textShadow: [
                glow(keyHex, 1.8),
                `${-4 - 4 * shake}px 0 rgba(255,40,80,0.55)`,
                `${4 + 4 * shake}px 0 rgba(60,220,255,0.5)`,
              ].join(","),
            })}
          />
        </div>
      ) : null}
      {slam > 0 ? (
        <Chars text="誓死、あなたのために。" at={0.7} t={lt} size={30} font="jp" weight={700}
          color={dim(hotHex, 0.92)} glowCol={dim("#ff3050", 0.7)} rise={16}
          style={{ position: "absolute", left: 0, right: 0, top: 610, justifyContent: "center", display: "flex", letterSpacing: 10 }} />
      ) : null}
    </>
  );
};

// ─────────────────────────────────────────────────────────── 11 TEARFALL ───
const Tearfall: React.FC<Ctx> = ({ lt, frame, keyHex, hotHex }) => {
  const err0 = smoothstep(13.3, 13.8, lt);
  const lines = 9;
  return (
    <>
      {err0 > 0 ? (
        <div style={{
          position: "absolute", left: 0, right: 0, top: 190, display: "flex", justifyContent: "center",
          fontFamily: FONT.mono, fontSize: 76, fontWeight: 700, letterSpacing: 8,
          color: "#ff3b30", textShadow: glow("#ff3b30", 1.4),
          transform: `translate(${srand(frame, 13) * 8 * err0}px, ${srand(frame, 17) * 6 * err0}px)`,
        }}>
          <Chars text="FATAL — 抑制できない" at={13.4} t={lt} size={76} font="mono" weight={700} stagger={0.03} rise={30} />
        </div>
      ) : null}
      {err0 > 0 ? Array.from({ length: lines }).map((_, i) => {
        const at = 13.9 + i * 0.38;
        if (lt < at) return null;
        const flick = 0.55 + 0.45 * srand(frame + i * 7, i * 13);
        return (
          <div key={i} style={{
            position: "absolute", left: 90 + (i % 3) * 40, top: 320 + i * 56,
            fontFamily: FONT.mono, fontSize: 25, letterSpacing: 1,
            color: `rgba(255,80,70,${0.5 + 0.35 * srand(i, 3)})`,
            textShadow: `0 0 10px rgba(255,50,40,0.6)`,
            opacity: flick,
          }}>
            ERROR[{pad(i, 3)}]: cannot suppress — at heart::overflow, line ∞
          </div>
        );
      }) : null}
    </>
  );
};

// ─────────────────────────────────────────────────────────── 12 BREAK ──────
const BREAK_TEXT = "Pretend Not To Notice";
const Break: React.FC<Ctx> = ({ lt, keyHex, hotHex }) => {
  const sh = clamp((lt - 1.27) * 1.35);
  const slices = 6;
  const gold = "#ff5a5a";
  return (
    <>
      <div style={{ position: "absolute", left: 0, right: 0, top: 430, height: 130 }}>
        {Array.from({ length: slices }).map((_, i) => {
          const drift = sh * (0.25 + 0.75 * Math.abs(srand(i, 5)));
          const dx = srand(i, 3) * 620 * drift;
          const dy = srand(i, 7) * 340 * drift + drift * 90;
          const rot = srand(i, 11) * 26 * drift;
          return (
            <div key={i} style={{
              position: "absolute", left: 0, right: 0, top: 0,
              clipPath: `polygon(0 ${i * (100 / slices)}%, 100% ${i * (100 / slices)}%, 100% ${(i + 1) * (100 / slices)}%, 0 ${(i + 1) * (100 / slices)}%)`,
              transform: `translate(${dx}px, ${dy}px) rotate(${rot}deg)`,
              opacity: 1 - smoothstep(4.5, 6.5, lt),
            }}>
              <Chars text={BREAK_TEXT} at={0.05} t={Math.min(lt, 1.9)} size={96} weight={700} stretch={82}
                color={hotHex} glowCol={keyHex} glowSize={1.4} stagger={0.028} rise={34} dur={0.6}
                style={{ justifyContent: "center", display: "flex" }} />
            </div>
          );
        })}
      </div>
      {lt > 1.35 ? (
        <Chars text="もう、気づかないふりはできない" at={1.45} t={lt} size={30} font="jp" color={dim(gold, 0.9)}
          glowCol={dim(gold, 0.5)} rise={18}
          style={{ position: "absolute", left: 0, right: 0, top: 610, justifyContent: "center", display: "flex", letterSpacing: 8 }} />
      ) : null}
      {lt > 8.6 ? (
        <Chars text="the mask is data. data can be rewritten." at={8.7} t={lt} size={26} font="mono" weight={400}
          color={dim(keyHex, 0.8)} glowCol={dim(keyHex, 0.35)} stagger={0.02} rise={12}
          style={{ position: "absolute", left: 0, right: 0, top: 700, justifyContent: "center", display: "flex", letterSpacing: 4 }} />
      ) : null}
    </>
  );
};

// ─────────────────────────────────────────────────────────── 13 SOURCE ─────
const LINE_H = 44;
type CodeLineT = { n: string; segs: { t: string; c: string }[] };
const baseCode: CodeLineT[] = [
  { n: "1", segs: [{ t: "class ", c: "#7da9ff" }, { t: "ENOA", c: "#e8ecff" }, { t: " extends ", c: "#7da9ff" }, { t: "Machine", c: "#e8ecff" }, { t: " {", c: "#8892a8" }] },
  { n: "2", segs: [{ t: "  void ", c: "#ff7ab8" }, { t: "NotToNotice", c: "#7dffd4" }, { t: "( tears ) {", c: "#8892a8" }] },
  { n: "3", segs: [{ t: "    hide", c: "#ffd166" }, { t: "( ", c: "#8892a8" }, { t: "tears", c: "#ff8a8a" }, { t: " );", c: "#8892a8" }] },
  { n: "4", segs: [{ t: "    suppress", c: "#ffd166" }, { t: "( ", c: "#8892a8" }, { t: "emotion", c: "#ff9ad2" }, { t: " );", c: "#8892a8" }] },
  { n: "5", segs: [{ t: "  }", c: "#8892a8" }] },
  { n: "6", segs: [{ t: "}", c: "#8892a8" }] },
];
const newCode: CodeLineT[] = [
  { n: "3", segs: [{ t: "    show", c: "#7dff9a" }, { t: "( ", c: "#8892a8" }, { t: "tears", c: "#b8ffd4" }, { t: " );", c: "#8892a8" }, { t: "  // allowed to cry", c: "#4f9a6a" }] },
  { n: "4", segs: [{ t: "    feel", c: "#7dff9a" }, { t: "( ", c: "#8892a8" }, { t: "everything", c: "#b8ffd4" }, { t: " );", c: "#8892a8" }] },
];

const Source: React.FC<Ctx> = ({ lt, t, keyHex, hotHex }) => {
  const green = "#64ff9e";
  const red = "#ff5a6a";
  // 0.4: open file, 2.2-4.2 select lines 3-4, 4.5 delete, 5.6+ type new
  const selK = smoothstep(2.2, 2.9, lt) * (1 - smoothstep(4.5, 5.0, lt));
  const delK = clamp((lt - 4.6) / 0.5);
  const oldVisible = 1 - delK;
  const newK = clamp((lt - 5.6) / 0.7);
  const renameK = smoothstep(11.5, 12.3, lt);
  const buildK = smoothstep(14.0, 14.8, lt);
  const logsK = smoothstep(16.5, 18.0, lt);
  return (
    <>
      <TypeLine text="> vi enoa/heart.ts" at={0.35} t={lt} cps={22} size={24}
        color={dim(green, 0.9)} glowCol={dim(green, 0.35)} cursor={false}
        style={{ position: "absolute", left: 150, top: 200, letterSpacing: 1 }} />
      <Panel x={140} y={250} w={1010} h={430} color={green} at={0.8} t={lt} label="enoa/heart.ts — heart::overflow">
        <div style={{ padding: "26px 30px", fontFamily: FONT.mono, fontSize: 25, position: "relative", height: 360 }}>
          {/* old lines 1-2 */}
          {baseCode.slice(0, 2).map((l, i) => (
            <div key={l.n} style={{
              position: "absolute", left: 0, right: 0, top: i * LINE_H, height: LINE_H,
              display: "flex", opacity: 1, whiteSpace: "pre",
            }}>
              <span style={{ width: 46, color: "#3f5a4a" }}>{l.n}</span>
              {l.segs.map((s, j) => <span key={j} style={{ color: s.c }}>{s.t}</span>)}
            </div>
          ))}
          {/* lines 3-4: select -> delete (collapse in place) */}
          {baseCode.slice(2, 4).map((l, i) => (
            <div key={l.n} style={{
              position: "absolute", left: 0, right: 0, top: 2 * LINE_H + i * LINE_H * (1 - delK),
              height: LINE_H * (1 - delK), overflow: "hidden", whiteSpace: "pre",
              display: "flex",
              background: selK > 0 ? `rgba(255,90,106,${0.16 * selK})` : "transparent",
              boxShadow: selK > 0 ? `inset 3px 0 0 ${red}` : undefined,
              textDecoration: delK > 0 ? "line-through" : undefined,
              opacity: oldVisible,
            }}>
              <span style={{ width: 46, color: "#3f5a4a" }}>{l.n}</span>
              {l.segs.map((s, j) => <span key={j} style={{ color: s.c }}>{s.t}</span>)}
            </div>
          ))}
          {/* lines 5-6: deletion of 2 rows above is cancelled by 2 new rows */}
          {baseCode.slice(4).map((l, i) => (
            <div key={l.n} style={{
              position: "absolute", left: 0, right: 0, top: (4 + i) * LINE_H,
              height: LINE_H, display: "flex", whiteSpace: "pre",
            }}>
              <span style={{ width: 46, color: "#3f5a4a" }}>{l.n}</span>
              {l.segs.map((s, j) => <span key={j} style={{ color: s.c }}>{s.t}</span>)}
            </div>
          ))}
          {/* new lines typed */}
          {newCode.map((l, i) => {
            const kk = clamp((lt - (5.7 + i * 1.6)) / 1.3);
            if (kk <= 0) return null;
            const full = l.segs.map((s) => s.t).join("");
            const nCh = Math.floor(kk * full.length);
            let acc = 0;
            return (
              <div key={l.n} style={{
                position: "absolute", left: 0, right: 0, top: (2 + i) * LINE_H,
                height: LINE_H, display: "flex", whiteSpace: "pre",
                textShadow: `0 0 12px ${dim(green, 0.4)}`,
              }}>
                <span style={{ width: 46, color: "#3f5a4a" }}>{l.n}</span>
                {l.segs.map((s, j) => {
                  const start = acc;
                  acc += s.t.length;
                  const seg = s.t.slice(0, Math.max(0, Math.min(s.t.length, nCh - start)));
                  return <span key={j} style={{ color: s.c }}>{seg}</span>;
                })}
              </div>
            );
          })}
          {/* blinking cursor on active line */}
          {Math.floor(t * 2.6) % 2 === 0 ? (
            <div style={{
              position: "absolute", left: 46 + 30, top: (delK >= 1 ? (lt < 7.3 ? 2 : 3) : 2) * LINE_H + 6,
              width: 16, height: 30, background: green, opacity: 0.85, boxShadow: `0 0 10px ${green}`,
            }} />
          ) : null}
        </div>
      </Panel>
      {renameK > 0 ? (
        <Chars text="ENOA  →  ToNotice()" at={11.6} t={lt} size={58} font="mono" weight={700} stagger={0.03}
          color={hotHex} glowCol={green} glowSize={1.3} rise={30}
          style={{ position: "absolute", left: 1230, top: 372, textAlign: "left" }} />
      ) : null}
      {buildK > 0 ? (
        <div style={{
          position: "absolute", left: 1236, top: 500, fontFamily: FONT.mono, fontSize: 23,
          color: dim(green, 0.95), textShadow: `0 0 12px ${dim(green, 0.4)}`, letterSpacing: 1.5,
          width: 600, whiteSpace: "nowrap",
        }}>
          <div>build ok — humanity 100%</div>
          <div style={{ color: dim(green, 0.6), marginTop: 8, fontSize: 20 }}>2 files changed, 3 insertions(+), 2 deletions(-)</div>
        </div>
      ) : null}
      {logsK > 0 ? [0, 1, 2].map((i) => {
        const at = 16.6 + i * 0.5;
        const k = clamp((lt - at) / 0.6);
        if (k <= 0) return null;
        const texts = [
          "[log]  cried at the flower field — warmth",
          "[log]  cried for you — resolve",
          "[log]  cried because happy — 人間",
        ];
        return (
          <div key={i} style={{
            position: "absolute", left: 1256, top: 600 + i * 44,
            fontFamily: FONT.mono, fontSize: 21, letterSpacing: 1,
            color: dim(hotHex, 0.85 * k), textShadow: `0 0 10px ${dim(keyHex, 0.4 * k)}`,
            opacity: k,
          }}>{texts[i]}</div>
        );
      }) : null}
    </>
  );
};

// ─────────────────────────────────────────────────────────── 14 VERDICT ────
const Verdict: React.FC<Ctx> = ({ lt, keyHex, hotHex }) => {
  const gold = "#ffd25e";
  const drawK = smoothstep(0.4, 3.6, lt);
  const slamK = clamp((lt - 4.0) / 0.55);
  const slamE = slamK > 0 ? 1 - Math.pow(1 - slamK, 3) : 0;
  const scale = slamK > 0 ? lerp(2.8, 1, slamE) : 1;
  const rot = lerp(-14, -6, slamE);
  const settle = smoothstep(5.2, 6.4, lt);
  const stampOpacity = lt < 4.0 ? 0.9 : 1;
  const R = 170;
  const circumference = 2 * Math.PI * R * 1.08;
  return (
    <>
      <div style={{
        position: "absolute", left: cxs - 260, top: 240, width: 520, height: 520,
        transform: `scale(${scale}) rotate(${rot}deg)`, opacity: stampOpacity,
      }}>
        <svg width={520} height={520} viewBox="0 0 520 520">
          <circle cx={260} cy={260} r={R} fill="none" stroke="#e03a2a" strokeWidth={13}
            strokeDasharray={`${circumference * drawK} ${circumference}`}
            strokeLinecap="round" transform="rotate(-90 260 260)"
            style={{ filter: "drop-shadow(0 0 14px rgba(224,58,42,0.65))" }} />
          <circle cx={260} cy={260} r={R - 34} fill="none" stroke="#e03a2a" strokeWidth={4} opacity={drawK}
            style={{ filter: "drop-shadow(0 0 8px rgba(224,58,42,0.5))" }} />
        </svg>
      </div>
      {slamK > 0 ? (
        <>
          <Chars text="人間は、はなまる。" at={4.15} t={lt} size={92} font="jp" weight={700} stagger={0.03}
            color={hotHex} glowCol={keyHex} glowSize={1.5} rise={36}
            style={{ position: "absolute", left: 0, right: 0, top: 740, justifyContent: "center", display: "flex" }} />
          <Chars text="HUMANITY GETS A GOLD STAR" at={4.8} t={lt} size={24} font="mono" weight={500}
            color={dim(gold, 0.9)} glowCol={dim(gold, 0.4)} rise={14}
            style={{ position: "absolute", left: 0, right: 0, top: 862, justifyContent: "center", display: "flex", letterSpacing: 10 }} />
        </>
      ) : null}
      {settle > 0 ? (
        <>
          <Chars text="exit code 0" at={6.6} t={lt} size={26} font="mono" weight={500}
            color={dim("#9fffc4", 0.95)} glowCol={dim("#64ff9e", 0.5)} rise={12}
            style={{ position: "absolute", left: 0, right: 0, top: 936, justifyContent: "center", display: "flex", letterSpacing: 6 }} />
          <Chars text="MV — 二次創作 / fan work · 非商用" at={7.4} t={lt} size={17} font="jp"
            color={dim(gold, 0.5)} rise={8}
            style={{ position: "absolute", left: 0, right: 0, top: 1012, justifyContent: "center", display: "flex", letterSpacing: 3 }} />
        </>
      ) : null}
    </>
  );
};

// ────────────────────────────────────────────────────────── dispatcher ─────
const MOVS: React.FC<Ctx>[] = [Null, Boot, Fairy, Query, Forge, Admit, Redact, Garden, Ascent, Doll, Fight, Tearfall, Break, Source, Verdict];

export const MovementOverlay: React.FC<Ctx & { mi: number }> = ({ mi, ...ctx }) => {
  const M = MOVS[mi];
  return (
    <div style={{ position: "absolute", inset: 0, overflow: "hidden" }}>
      <M {...ctx} />
    </div>
  );
};
