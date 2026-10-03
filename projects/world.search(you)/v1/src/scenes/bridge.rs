//! Scene: bridge (2:31–2:59) — the words she can't remember. Dim, cold, uncertain.
//! Falling '?' rain, a typing terminal that keeps redacting, a "love" that almost
//! resolves — then the chorus bursts in.

use super::Ctx;
use crate::art;
use crate::fx;
use crate::gfx::{clamp01, Color, Grid, Rng};
use crate::scenes::ui;
const GRAY: Color = Color::hex(0x8a93a6);
const DIMGRAY: Color = Color::hex(0x55607a);
const COLD: Color = Color::hex(0x5b6b85);
const LOVE: Color = Color::hex(0xff8aa8);
const ROSE: Color = Color::hex(0xff6d8a);

pub fn render(g: &mut Grid, tl: f64, tg: f64, ctx: &Ctx) {
    let w = g.w as i64;
    let h = g.h as i64;

    // ---- dim, cold base
    fx::desaturate(g, 0.4);

    // ---- '?' rain, faster then slower (the storm of confusion calms down)
    let rain_speed = if tl > 15.0 { 3.0 } else { 9.0 };
    let mut rng = Rng::new(0x2E0D);
    for _ in 0..40 {
        let x = rng.i64(0, w);
        let speed = rng.range(6.0, 16.0) * (rain_speed / 9.0);
        let y = ((tg * speed + rng.range(0.0, 100.0)) % (h as f64 + 10.0)) as i64 - 5;
        g.put_alpha(x, y, '?', COLD, rng.range(0.15, 0.5));
    }

    // ---- typing terminal (upper area)
    let ty = 6;
    ui::command(g, 4, ty, "let me say it…", tl - 0.2, GRAY);

    // "you are a piece of " struggles into existence (local 0.4 → 3.0)
    if tl > 0.4 && tl < 3.0 {
        let n = fx::typed("you are a piece of ", tl - 0.4, 14.0);
        let s: String = "you are a piece of ".chars().take(n).collect();
        g.text(6, ty + 1, &s, GRAY.scale(0.85));
    }

    // U-Um stutter (local 4.6 → 7.0)
    if tl > 4.6 && tl < 7.4 {
        let a = clamp01((tl - 4.6) / 0.5) * clamp01((7.4 - tl) / 0.7);
        g.text_alpha(6, ty + 2, "u- um… what was it?", GRAY, a);
    }
    // and again (local 7.0 → 9.8)
    if tl > 7.0 && tl < 10.2 {
        let a = clamp01((tl - 7.0) / 0.5) * clamp01((10.2 - tl) / 0.7);
        g.text_alpha(6, ty + 3, "what was it…? what was it?", DIMGRAY, a);
    }

    // "Do-o-o-o-ope" tries to resolve: a word overtyping itself (local 12.1 → 13.9)
    if tl > 12.1 && tl < 14.0 {
        let p = clamp01((tl - 12.1) / 1.9);
        let y = ty + 5;
        let shown = if p < 0.45 { "do-ope" } else if p < 0.8 { "do-o-ope" } else { "do-…-ope" };
        let flick = if ((tg * 9.0) as i64) % 2 == 0 { 0.75 } else { 0.35 };
        g.text_alpha(6, y, shown, GRAY, flick);
        if p > 0.8 {
            g.text_alpha(6 + Grid::measure(shown) + 2, y, "✕", Color::hex(0xc9526a), (p - 0.8) * 5.0);
        }
    }

    // ---- the object is dissolving into pieces (local 2.0 → 7.0)
    if tl > 1.2 && tl < 7.0 {
        let p = clamp01((tl - 1.2) / 5.0);
        // it starts as a whole shape, then crumbles
        let alpha = (1.0 - p).powf(0.8) * 0.9;
        if p < 0.7 {
            art::draw(g, &art::TABLE_TOP, w / 2 - 17, h / 2 - 4, 2, true, alpha, Some(COLD));
        }
        let mut r2 = Rng::new(0xBEEF);
        for _ in 0..(p * 46.0) as usize {
            let a0 = r2.range(0.0, std::f64::consts::TAU);
            let sp = r2.range(0.6, 2.4);
            let x = w as f64 / 2.0 + a0.cos() * sp * (2.0 + p * 14.0);
            let y = h as f64 / 2.0 - 4.0 + a0.sin() * sp * (1.0 + p * 6.0) + p * p * 12.0;
            let ch = ['.', ',', '·', '`'][r2.i64(0, 3) as usize];
            g.put_alpha(x as i64, y as i64, ch, COLD, 0.5 * (1.0 - p * 0.6));
        }
    }

    // ---- wrong words, tried and crossed out (local 2.9 → 12.1)
    {
        let cands = ["dung", "crud", "junk", "shit", "crap"];
        let mut y = ty + 4;
        for (i, c) in cands.iter().enumerate() {
            let at = 2.9 + i as f64 * 1.8;
            if tl < at || tl > at + 2.2 {
                y += 1;
                continue;
            }
            let k = (tl - at) / 2.2;
            let a = clamp01(k * 4.0) * clamp01((1.0 - k) * 5.0);
            g.text_alpha(6, y, c, GRAY.scale(0.9), a * 0.9);
            if k > 0.55 {
                g.text_alpha(6, y, "----", Color::hex(0xc9526a), (k - 0.55) * 2.0);
            }
            if k > 0.8 {
                g.put_alpha(11, y, 'x', Color::hex(0xc9526a), (k - 0.8) * 4.0);
            }
            y += 1;
        }
    }

    // ---- a lonely figure far away ("if you could stay", local ≥ 13.9)
    if tl > 13.89 {
        let a = clamp01((tl - 13.89) / 2.0);
        let fx_ = w - 24;
        let fy = h / 2 - 4;
        let fig = [" ,---, ", "( .-. )", " `---' ", "  |||  ", "  |||  "];
        for (ry, row) in fig.iter().enumerate() {
            for (rx, ch) in row.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                g.put_alpha(fx_ + rx as i64, fy + ry as i64, ch, COLD, a * 0.9);
            }
        }
        // dotted line reaching toward it
        if tl > 15.0 {
            let a2 = clamp01((tl - 15.0) / 1.5);
            for x in (w / 2)..(fx_ - 4) {
                if (x - w / 2) % 4 == 0 {
                    g.put_alpha(x, fy + 2, '·', GRAY, a2 * 0.5);
                }
            }
        }
    }

    // ---- "love" almost lands (local 25.5 →)
    if tl > 25.5 {
        let p = clamp01((tl - 25.5) / 0.9);
        let glow = clamp01((tl - 26.4) / 1.0);
        // letters arrive out of the rain, then glow warm
        let y = h / 2 - 3;
        let word: Vec<(char, f64, f64)> = vec![('L', 0.0, 0.0), ('O', 0.35, -1.0), ('V', 0.7, 1.0), ('E', 1.0, 0.0)];
        let total_w = 4 * 7 * 2;
        let mut x0 = (w - total_w) / 2;
        for (ch, delay, dy) in word {
            let settle = clamp01((p - delay * 0.6) / 0.45);
            let ox = ((1.0 - settle) * 10.0 * (1.0 + delay)).round() as i64;
            let oy = ((1.0 - settle) * 5.0 * (dy + 1.0)).round() as i64;
            let a = (0.25 + 0.75 * glow) * settle;
            art::big_text(g, x0 + ox, y + oy, &ch.to_string(), ROSE.glow(0.3 * glow), 2, a);
            x0 += 7 * 2;
        }
        if glow > 0.35 {
            fx::tint_all(g, LOVE, 0.05 * (glow - 0.35));
            let mut r2 = Rng::new((tg * 8.0) as u64);
            for _ in 0..7 {
                let x = w / 2 + r2.i64(-14, 14);
                let y2 = h / 2 + r2.i64(-4, 6);
                g.put_alpha(x, y2, '♥', LOVE, 0.5 * (glow - 0.35));
            }
        }
    }

    // occasional cursor blink at a lonely prompt
    if ((tg * 1.5) as i64) % 2 == 0 {
        g.put(4, ty + 7, '▌', DIMGRAY);
    }
    let _ = (ctx.beats.pulse(tg), art::glyph('l'));
}
