//! Scene: ending (4:50 →) — the search terminates. 0 results, 1 warmth.

use super::Ctx;
use crate::art;
use crate::fx;
use crate::gfx::{clamp01, Color, Grid};

const ROSE: Color = Color::hex(0xff6d8a);
const GRAY: Color = Color::hex(0x8a93a6);
const DIM: Color = Color::hex(0x55607a);
const GOLD: Color = Color::hex(0xffd27f);

pub fn render(g: &mut Grid, tl: f64, tg: f64, _ctx: &Ctx) {
    let w = g.w as i64;
    let h = g.h as i64;
    let cx = w / 2;

    // slow fade-in from black
    let a = clamp01(tl / 1.2);
    if a < 1.0 {
        fx::fade(g, 1.0 - a);
    }

    let y0 = h / 2 - 3;

    // the command line, typed one last time
    let line1 = "world.search(you);";
    let n1 = fx::typed(line1, tl, 11.0);
    let s1: String = line1.chars().take(n1).collect();
    g.text_center(y0, &s1, ROSE.glow(0.15));
    if n1 < line1.chars().count() && (tg % 0.5) < 0.3 {
        let wm = Grid::measure(&s1);
        g.put(cx + wm / 2, y0, '█', ROSE);
    }

    // results
    if tl > 3.0 {
        let a2 = clamp01((tl - 3.0) / 0.6);
        g.text_alpha(cx - 20, y0 + 2, "0 results for \"you\"", GRAY, a2 * 0.8);
    }
    if tl > 4.2 {
        let a3 = clamp01((tl - 4.2) / 0.6);
        g.text_alpha(cx - 20, y0 + 3, "∞ results for \"warmth\"", GOLD, a3);
    }

    // a single heart, beating slowly
    if tl > 5.0 {
        let beat = (tg * 0.8).sin();
        let size = 1 + (beat > 0.0) as i64;
        let hy = y0 + 6;
        let ch = if size > 1 { '♥' } else { '♡' };
        g.put(cx, hy, ch, ROSE.glow(0.3 * (beat * 0.5 + 0.5)));
        if size > 1 {
            g.put(cx - 1, hy, '♥', ROSE.scale(0.7));
            g.put(cx + 1, hy, '♥', ROSE.scale(0.7));
        }
    }

    // thank-you / replay hint
    if tl > 7.0 {
        let a4 = clamp01((tl - 7.0) / 1.0);
        g.text_center(y0 + 9, "— Mili · momocashew —", DIM.over(g.row_bg((y0 + 9) as usize), a4));
    }
    if tl > 8.0 {
        g.text_center(y0 + 11, "[ q ] quit   [ ←→ ] replay a moment", DIM.scale(0.8));
    }

    // sparse drifting dust
    let mut rng = crate::gfx::Rng::new(0xD057);
    for _ in 0..12 {
        let life = (tg * 0.4 + rng.f64() * 6.0) % 6.0;
        let x = rng.i64(0, w);
        let y = (h as f64 - life * 8.0) as i64;
        g.put_alpha(x, y, '·', GRAY, 0.25 * (1.0 - life / 6.0));
    }
    let _ = art::glyph('q');
}
