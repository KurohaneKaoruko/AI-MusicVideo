//! Scene: boot sequence (0:00–0:14) — cursor, credits, logo materializing, boot log.

use super::Ctx;
use crate::art;
use crate::gfx::{Color, Grid, Rng};
use crate::lyrics::CREDITS;

const SLATE: Color = Color::hex(0x8a93a6);
const ROSE: Color = Color::hex(0xff6d8a);

pub fn render(g: &mut Grid, tl: f64, tg: f64, ctx: &Ctx) {
    // ---- phase 1: lone blinking cursor in the dark
    if tl < 6.0 {
        let cx = g.w as i64 / 2 - 28;
        let cy = g.h as i64 / 2;
        let on = (tg * 2.2) as i64 % 2 == 0;
        if on {
            g.put_bold(cx, cy, '█', SLATE);
        }
        // hint of a drive light
        if (tg * 0.7) as i64 % 3 == 0 {
            g.put_alpha(g.w as i64 - 3, 2, '·', ROSE, 0.25);
        }
        return;
    }

    // ---- logo materializes column by column
    let title = "world.search(you);";
    let scale = if g.w >= 118 { 2 } else { 1 };
    let lw = art::big_text_width(title, scale);
    let lx = (g.w as i64 - lw) / 2;
    let ly = (g.h as i64 / 2) - 7;
    let appear = crate::gfx::clamp01((tl - 6.0) / 3.2);
    let reveal_x = lx + (lw as f64 * crate::gfx::ease_in_out(appear) * 1.08) as i64;
    let beat = ctx.beats.pulse(tg);

    // chromatic ghost while materializing
    if appear < 1.0 {
        art::big_text(g, lx + 1, ly, title, Color::rgb(255, 80, 120), scale, 0.25 * (1.0 - appear));
        art::big_text(g, lx - 1, ly, title, Color::rgb(90, 160, 255), scale, 0.25 * (1.0 - appear));
    }
    // main body with per-column reveal
    draw_big_clipped(g, lx, ly, title, scale, reveal_x, ROSE.glow(0.18 * beat));

    // leading edge glow + sparks
    if appear < 1.0 {
        for dy in 0..5 * scale {
            let yy = ly + dy;
            let flick = Rng::new((tg * 60.0) as u64 ^ (yy as u64)).f64();
            g.put_alpha(reveal_x, yy, '▓', ROSE.glow(0.5), 0.35 + 0.4 * flick);
        }
        let mut rng = Rng::new((tg * 47.0) as u64);
        for _ in 0..4 {
            let sx = reveal_x + rng.i64(-4, 4);
            let sy = ly + rng.i64(0, 5 * scale - 1);
            g.put_alpha(sx, sy, '*', ROSE, 0.5 * (1.0 - appear));
        }
    }

    // ---- boot log
    let lines: [(&str, f64, Color); 6] = [
        ("$ world.search --target \"you\"", 8.9, SLATE),
        ("loading world ................. OK", 9.9, Color::hex(0x6fc7a0)),
        ("indexing memories ............. OK", 10.7, Color::hex(0x6fc7a0)),
        ("calibrating love_sensor ....... OK", 11.5, Color::hex(0x6fc7a0)),
        ("candidates: 8,141,596,233", 12.3, SLATE),
        ("> run", 13.3, ROSE.glow(0.2)),
    ];
    let logx = lx.max(4);
    let mut logy = ly + 5 * scale + 2;
    for (s, at, col) in lines {
        let e = tl - at;
        if e > 0.0 {
            let n = crate::fx::typed(s, e, 34.0);
            let shown: String = s.chars().take(n).collect();
            g.text(logx, logy, &shown, col);
            if n < s.chars().count() && ((e * 6.0) as i64) % 2 == 0 {
                g.put(logx + Grid::measure(&shown), logy, '▌', col);
            }
        }
        logy += 1;
    }
    // countdown on the candidates line (search narrowing…)
    if tl > 12.3 {
        let base: u64 = 8_141_596_233;
        let frac = crate::gfx::clamp01((tl - 12.3) / 2.0);
        let val = base - ((base as f64 * frac * frac) as u64);
        let s = format!("candidates: {val}");
        let w = Grid::measure(&s);
        g.fill(logx, logy - 1, w as i64 + 2, 1, ' ', SLATE, Some(g.row_bg((logy - 1) as usize)));
        g.text(logx, logy - 1, &s, SLATE.over(g.row_bg((logy - 1) as usize), 0.9));
    }

    // ---- credits (from the LRC header)
    if tl > 1.0 && tl < 6.0 {
        let a = crate::gfx::clamp01((tl - 1.0) / 1.2) * crate::gfx::clamp01((6.0 - tl) / 1.0);
        let cy = g.h as i64 / 2 + 3;
        for (i, (en, _zh)) in CREDITS.iter().enumerate() {
            g.text_alpha((g.w as i64 - Grid::measure(en)) / 2, cy + i as i64, en, SLATE, a * 0.75);
        }
    }
}

/// pixel font with a hard x clip for the materialize effect
fn draw_big_clipped(g: &mut Grid, x0: i64, y0: i64, s: &str, scale: i64, clip_x: i64, fg: Color) {
    let mut x = x0;
    for ch in s.chars() {
        let Some(rows) = art::glyph(ch) else { continue };
        let gw = rows[0].chars().count() as i64;
        let gw_px = gw * scale;
        if x >= clip_x {
            break;
        }
        let vis = (clip_x - x).min(gw_px); // visible columns of this glyph
        let full = vis >= gw_px;
        for (ry, row) in rows.iter().enumerate() {
            for (rx, pc) in row.chars().enumerate() {
                if pc != '#' {
                    continue;
                }
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = x + rx as i64 * scale + sx;
                        let py = y0 + ry as i64 * scale + sy;
                        if full || px < x + vis - 1 {
                            g.put_bold(px, py, '█', fg);
                        } else if px < x + vis {
                            g.put_alpha(px, py, '▒', fg.glow(0.4), 0.8);
                        }
                    }
                }
            }
        }
        x += gw_px;
    }
}
