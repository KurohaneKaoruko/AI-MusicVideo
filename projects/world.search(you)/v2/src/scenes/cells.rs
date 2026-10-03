//! ACT VI-A — merge / evolve (3:27–3:54).
//!
//! "Each one of us has a bucket of love … cut us into pieces, break us down to
//! the cell … merge us back up, we have evolved now."
//!
//! Composition: a lab bench, then a microscope. The four buckets are measured,
//! everything is cut apart into a living cell field, and something new is
//! grown out of it in the middle of the frame.

use super::ui;
use crate::art;
use crate::env;
use crate::gfx::{clamp01, ease_in_out, ease_out_cubic, hash01, pal, Color, Grid, Rng};
use crate::pix::{Canvas, H, W};
use crate::scenes::Ctx;

const BUCKETS: [(f64, &str); 4] = [(0.90, "us"), (0.25, "them"), (0.60, "you"), (0.10, "me")];

pub fn render(g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx) {
    let green = pal::c(pal::GREEN_OK);
    let rose = pal::c(pal::ROSE);

    // ---- the bench
    env::void_grad(cv, Color::hex(0x061210), Color::hex(0x030807), 1.0);
    cv.rect_a(0, 820, W as i32, 260, Color::hex(0x0b1a18), 1.0);
    cv.rect_a(0, 816, W as i32, 4, Color::hex(0x1d3a35), 0.9);
    for k in 0..24 {
        cv.rect_a((k * 84) as i32, 820, 2, 260, Color::hex(0x122421), 0.6);
    }

    // ---- phase 1: four buckets, measured
    // shatter (221.18) breaks them apart; merge (228.16) grows the chimera
    let cut = if tg > 221.18 { clamp01((tg - 221.18) / 1.6) } else { 0.0 };
    let merge = if tg > 228.16 { ease_out_cubic(clamp01((tg - 228.16) / 3.4)) } else { 0.0 };

    if merge < 0.98 {
        for (i, (fill, name)) in BUCKETS.iter().enumerate() {
            let bx = 260.0 + i as f32 * 470.0;
            let (bw, bh) = art::size(&art::BUCKET, 5);
            let by = 816.0 - bh as f32;
            // lift and shatter
            let lift = (cut * 90.0) as f32;
            let shard = cut * (1.0 - merge);
            art::draw_at(
                cv,
                &art::BUCKET,
                bx as i32,
                (by - lift) as i32,
                5,
                (1.0 - merge) * (1.0 - cut * 0.55),
                None,
            );
            // the love inside, with a wobbling surface
            let inner_w = bw as f32 - 40.0;
            let level = (fill * 0.78 * (1.0 - cut * 0.4)) as f32;
            let surf = ((tg * 2.2 + i as f64).sin() * 3.0) as f32;
            let top = by - lift + (bh as f32 * (1.0 - level)) + surf;
            cv.rect_a(
                (bx + 20.0) as i32,
                top as i32,
                inner_w as i32,
                ((bh as f32 - 24.0) * level) as i32,
                rose,
                0.72 * (1.0 - merge) as f64,
            );
            cv.rect_a((bx + 20.0) as i32, top as i32, inner_w as i32, 3, rose.glow(0.45), 0.85 * (1.0 - merge) as f64);
            // a floating heart surface
            if *fill > 0.4 {
                crate::art::draw_c(
                    cv,
                    &art::HEART,
                    bx + bw as f32 / 2.0,
                    top + 30.0,
                    3,
                    0.55 * (1.0 - merge) as f64,
                    None,
                );
            }
            cv.text_px(bx + bw as f32 / 2.0, by + 60.0, name, 30.0, false, green.scale(0.85), (1.0 - merge) as f64 * 0.8, true);
            cv.text_px(
                bx + bw as f32 / 2.0,
                by + 100.0,
                &format!("{:.0}%", fill * 100.0),
                26.0,
                true,
                rose,
                (1.0 - merge) as f64 * 0.9,
                true,
            );
            // shards flying off
            if cut > 0.02 && shard > 0.02 {
                let mut rng = Rng::new(0x5A17 ^ (i as u64 * 7919));
                for _ in 0..7 {
                    let sx = bx + rng.f64() as f32 * bw as f32;
                    let sy = by - lift + rng.f64() as f32 * bh as f32 * 0.7;
                    let vx = (rng.f64() as f32 - 0.5) * 420.0 * cut as f32;
                    let vy = -(60.0 + rng.f64() as f32 * 240.0) * cut as f32;
                    cv.rect_a(
                        (sx + vx) as i32,
                        (sy + vy) as i32,
                        (14.0 - 8.0 * cut as f32).max(3.0) as i32,
                        (10.0 - 6.0 * cut as f32).max(2.0) as i32,
                        Color::hex(0x9aa4b4).glow(0.2),
                        0.7 * shard,
                    );
                }
            }
        }
    }

    // ---- phase 2: the cell field (221.18 → )
    if cut > 0.02 {
        let a = cut * (1.0 - merge * 0.85);
        env::cellular(cv, tg, 44, 0xCE11, green, Color::hex(0x0d2a26), 0.55 * a);
        // the cut: sweeping blades
        if cut < 1.0 {
            for k in 0..5 {
                let u = (cut * 1.4 - k as f64 * 0.16).clamp(0.0, 1.0);
                let y = u * H as f64;
                cv.rect_a(0, y as i32, W as i32, 3, rose, 0.6 * (1.0 - u));
            }
            for k in 0..5 {
                let u = (cut * 1.4 - k as f64 * 0.16).clamp(0.0, 1.0);
                let x = u * W as f64;
                cv.rect_a(x as i32, 0, 3, H as i32, rose, 0.5 * (1.0 - u));
            }
        }
    }

    // ---- phase 3: merge (228.16) — cells converge and grow a chimera
    if merge > 0.01 {
        // cells streaming to the centre
        for i in 0..90 {
            let a = hash01(0x0E12 + i as u64 * 7919);
            let b = hash01(0x0E12 + i as u64 * 104729);
            let ang = a * std::f64::consts::TAU;
            let r0 = 900.0 * (1.0 - merge * 0.75);
            let x = W as f64 / 2.0 + ang.cos() * r0 + (tg * 0.4 + b * 6.0).sin() * 30.0;
            let y = H as f64 / 2.0 + ang.sin() * r0 * 0.6 + (tg * 0.5 + a * 6.0).cos() * 30.0;
            cv.disc(x as f32, y as f32, (7.0 - 5.0 * merge as f32).max(1.5), green, 0.7 * (1.0 - merge * 0.4));
            cv.rect_a((x - 2.0) as i32, (y - 2.0) as i32, 4, 4, Color::hex(0x9fe8c4), 0.5);
        }
        cv.glow_at(W as f32 / 2.0, H as f32 / 2.0, 780.0 * merge as f32, green, 0.45 * merge, 2.0);
        // the chimera resolves
        let (cw, chh) = art::size(&art::CHIMERA, 11);
        let a = clamp01((merge - 0.35) / 0.5);
        if a > 0.01 {
            cv.glow_at(W as f32 / 2.0, H as f32 / 2.0, 420.0, rose, 0.30 * a, 2.2);
            art::draw_at(
                cv,
                &art::CHIMERA,
                (W as f32 / 2.0 - cw as f32 / 2.0) as i32,
                (H as f32 / 2.0 - chh as f32 / 2.0) as i32,
                11,
                a,
                None,
            );
            cv.text_px(W as f32 / 2.0, H as f32 / 2.0 + 300.0, "evolved", 40.0, true, green.glow(0.3), a, true);
        }
    }

    // ---- the program's measurements
    ui::panel(g, 3, 3, 40, 11, "love.measure", green, Some(pal::c(pal::BG0).scale(1.1)));
    for (i, (fill, name)) in BUCKETS.iter().enumerate() {
        let st = clamp01((tl - 0.3 - i as f64 * 0.5) * 1.5);
        if st <= 0.02 {
            continue;
        }
        let y = 6 + i as i64 * 2;
        g.text_alpha(5, y, name, pal::c(pal::MID), st);
        let barw = 16i64;
        for k in 0..barw {
            let on = (k as f64 / barw as f64) < *fill;
            let col = if on { rose } else { pal::c(pal::DIM) };
            g.put_alpha(13 + k, y, if on { '█' } else { '·' }, col, st);
        }
        g.text_alpha(31, y, &format!("{:.0}%", fill * 100.0), green.scale(0.9), st);
    }
    if cut > 0.3 {
        g.text(5, 12, "→ cut into 44 cells", pal::c(pal::ROSE).scale(0.9));
    }

    // the reticle watches the merge
    ui::reticle(cv, W as f32 / 2.0, H as f32 / 2.0, 380.0, 0.3 + 0.7 * merge, tg * 0.3, green, 0.16 + 0.2 * merge, merge);
    env::dust(cv, tg, 30, 0x0E12, green, 0.22, 120.0, 900.0);
    ui::edge_shade(cv, Color::hex(0x020605), 0.66);
    cv.fx.bloom += 0.25;
    let _ = (ease_in_out, ctx);
}
