// ---------------------------------------------------------------------------
// glyphs.ts - builds the katakana glyph atlas used by the data-rain shader.
// 16x16 cells, 64px each, drawn once at engine init.
// ---------------------------------------------------------------------------

const CHARS =
  "ｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ" +
  "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワン" +
  "0123456789ABCDEFXYZ" +
  "<>{}[]();=+*#/\\|-_.$%&?!";

export const ATLAS_CELLS = 16;
export const ATLAS_SIZE = 1024;

export function buildGlyphAtlas(): HTMLCanvasElement {
  const cv = document.createElement("canvas");
  cv.width = ATLAS_SIZE;
  cv.height = ATLAS_SIZE;
  const c = cv.getContext("2d")!;
  c.clearRect(0, 0, ATLAS_SIZE, ATLAS_SIZE);
  c.fillStyle = "#fff";
  c.textAlign = "center";
  c.textBaseline = "middle";
  const cell = ATLAS_SIZE / ATLAS_CELLS;
  let seed = 1337;
  const rnd = () => {
    seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
    return seed / 4294967296;
  };
  for (let i = 0; i < ATLAS_CELLS * ATLAS_CELLS; i++) {
    const ch = CHARS[Math.floor(rnd() * CHARS.length)];
    const x = (i % ATLAS_CELLS) * cell;
    const y = Math.floor(i / ATLAS_CELLS) * cell;
    const big = rnd() > 0.75;
    c.font = `${big ? 52 : 44}px "Yu Gothic", "Meiryo", "Consolas", monospace`;
    c.save();
    c.translate(x + cell / 2, y + cell / 2 + 2);
    c.fillText(ch, 0, 0);
    c.restore();
  }
  return cv;
}
