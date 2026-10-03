//! Scene: flower (1:49, 3:53 reprise) — bud → bloom → "just occasionally" → full bloom.
//! In the reprise the same flower plays back inside a sepia memory window.

use super::Ctx;
use crate::art;
use crate::fx;
use crate::gfx::{clamp01, ease_out_cubic, Color, Grid, Rng};
use crate::scenes::ui;

const PETAL: Color = Color::hex(0xf2a7c3);
const GOLD: Color = Color::hex(0xf2c14e);
const STEM: Color = Color::hex(0x5a9e6e);
const ROSE: Color = Color::hex(0xff6d8a);

/// local start times of the four lines (first verse; the reprise is close enough)
const SEGS: [f64; 4] = [0.0, 3.43, 6.88, 10.24];

pub fn render(g: &mut Grid, tl: f64, tg: f64, ctx: &Ctx, memory: bool) {
    let w = g.w as i64;
    let h = g.h as i64;
    let cx = w / 2;
    let cy = h / 2;

    if memory {
        let bw = (w - 12).min(80);
        let bh = h - 14;
        g.box_rounded(
            cx - bw / 2,
            cy - bh / 2,
            bw,
            bh,
            Color::hex(0x7a6a52).over(g.row_bg(cy as usize), 0.7),
            None,
            Some(("MEM//flower.log", Color::hex(0xa8956f))),
        );
    }

    // which lyric line are we on
    let seg = if tl >= SEGS[3] {
        3
    } else if tl >= SEGS[2] {
        2
    } else if tl >= SEGS[1] {
        1
    } else {
        0
    };
    let seg_t = tl - SEGS[seg];
    let beat = ctx.beats.pulse(tg);

    // ---- stage: light shaft + ground so the flower has a place to live
    let top = 4;
    let ground = h - 13;
    for y in top..ground {
        let f = 1.0 - (y - top) as f64 / (ground - top) as f64;
        if (y % 3) == 0 {
            g.put_alpha(cx, y, '│', PETAL, 0.05 + 0.05 * f);
        }
    }
    for x in (cx - 16)..(cx + 16) {
        let ch = if (x - cx).abs() % 6 == 0 { '_' } else { '.' };
        g.put_alpha(x, ground + 1, ch, STEM.scale(0.7), 0.5);
    }

    // ---- stem + leaves (procedural, sways)
    let sway = (tg * 1.1).sin() * 2.0;
    let head_y = cy - 2;
    for y in head_y + 6..ground {
        let dy = y as f64;
        let x = cx + (sway * (dy - head_y as f64) / 10.0).round() as i64;
        g.put(x, y, '│', STEM);
        g.put(x + 1, y, '│', STEM.scale(0.7));
    }
    // small grass tufts at the far sides (clear of the lyric zone)
    for side in [-1i64, 1] {
        let gx = cx + side * 22;
        for k in 0..3 {
            let hh = 2 + (k as i64);
            g.put_alpha(gx + k, ground + 1 - hh, '/', STEM.scale(0.8), 0.5 - k as f64 * 0.1);
        }
    }

    // ---- the flower head
    let (fw, fh) = art::sprite_size(&art::FLOWER_0, 2);
    let ox = cx - fw / 2 + sway.round() as i64;
    let oy = head_y - fh + 4;
    match seg {
        0 => {
            let o = ease_out_cubic(clamp01(seg_t / 2.2));
            art::draw(g, &art::FLOWER_0, ox, oy + ((1.0 - o) * 2.0) as i64, 2, true, 0.65 + 0.35 * o, None);
            if beat > 0.7 {
                g.put_alpha(ox + 5, oy - 1, '*', PETAL, 0.6);
                g.put_alpha(ox + 20, oy - 2, '*', PETAL, 0.4);
            }
        }
        1 => {
            art::draw(g, &art::FLOWER_1, ox, oy, 2, true, 1.0, None);
            sparkle_ring(g, cx, oy + 4, seg_t, GOLD, 0.45);
        }
        2 => {
            // "just occasionally": opens and closes
            let ph = (tg * 0.45) % 2.0;
            let openness = if ph < 1.0 { ease_out_cubic(ph) } else { 1.0 - ease_out_cubic(ph - 1.0) };
            let c1 = clamp01((1.0 - openness) * 2.0);
            let c2 = clamp01(openness * 2.0 - 1.0);
            if c1 > 0.0 {
                art::draw(g, &art::FLOWER_1, ox, oy, 2, true, c1, None);
            }
            if c2 > 0.0 {
                art::draw(g, &art::FLOWER_2, ox, oy, 2, true, c2, None);
            }
        }
        _ => {
            let a = clamp01(seg_t / 0.4);
            art::draw(g, &art::FLOWER_2, ox, oy, 2, true, a, None);
            // burst of light
            if seg_t < 1.4 {
                let p = seg_t / 1.4;
                for k in 0..12 {
                    let ang = k as f64 * std::f64::consts::TAU / 12.0;
                    let d = p * 12.0;
                    let x = cx as f64 + ang.cos() * d;
                    let y = (oy + 4) as f64 + ang.sin() * d * 0.55;
                    g.put_alpha(x as i64, y as i64, '*', GOLD, (1.0 - p) * 0.8);
                }
            }
            // petals drifting down
            let mut rng = Rng::new((tg * 6.0) as u64);
            for _ in 0..7 {
                let life = (tg * 0.45 + rng.f64() * 3.0) % 3.0;
                let x = cx + rng.i64(-16, 16);
                let y = oy + 5 + life as i64 * 2;
                g.put_alpha(
                    x,
                    y,
                    ['*', '.', ','][rng.i64(0, 2) as usize],
                    PETAL,
                    0.45 * (1.0 - life / 3.0),
                );
            }
            if !memory && seg_t > 0.6 {
                ui::stamp_perfect(g, seg_t - 0.6, cx, cy - 9, PETAL);
            }
        }
    }

    // ---- ambient sparkle field
    let mut rng = Rng::new((tg * 2.0) as u64);
    for _ in 0..10 {
        let x = rng.i64(2, w - 3);
        let y = rng.i64(3, ground);
        let a = 0.06 + 0.10 * rng.f64();
        g.put_alpha(x, y, '*', ROSE, a);
    }

    if memory {
        fx::tint_all(g, Color::hex(0xd9c3a5), 0.5);
        fx::grain(g, tg, 0.7);
    }
}

fn sparkle_ring(g: &mut Grid, cx: i64, cy: i64, t: f64, color: Color, a: f64) {
    let p = (t % 1.5) / 1.5;
    for k in 0..6 {
        let ang = k as f64 * std::f64::consts::TAU / 6.0;
        let d = p * 8.0;
        g.put_alpha(
            cx + (ang.cos() * d) as i64,
            cy + (ang.sin() * d * 0.6) as i64,
            '*',
            color,
            (1.0 - p) * a,
        );
    }
}
