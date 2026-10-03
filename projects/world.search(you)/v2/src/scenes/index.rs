//! ACT II-A — `memory.sort` (0:14–0:41).
//!
//! Composition: the frame IS the data. Forty-four vertical memory bars span the
//! full width and the program bubble-sorts them in front of you, one pass per
//! bar of music, until "the bubble pops". A memory is then plucked out and
//! filed into `past/`.
//!
//! No panel, no lyric box — the words hang in the empty upper third.

use super::ui;
use crate::env;
use crate::gfx::{clamp01, ease_in_out, ease_out_cubic, hash01, pal, Color, Grid};
use crate::pix::{Canvas, W};
use crate::scenes::Ctx;

const N: usize = 44;

fn height_of(i: usize) -> f64 {
    // a fixed, deliberately unsorted memory set
    let a = hash01(0x4D454D ^ (i as u64) * 2654435761);
    let b = hash01(0x4D454D ^ (i as u64) * 40503);
    (0.22 + 0.78 * (a * 0.7 + b * 0.3)).powf(1.15)
}

/// Run `steps` bubble-sort comparisons; returns the array and the pair in play.
fn snapshot(steps: usize) -> (Vec<f64>, Option<(usize, usize)>) {
    let mut a: Vec<f64> = (0..N).map(height_of).collect();
    let mut last = None;
    let mut done = 0usize;
    'outer: for pass in 0..N {
        let mut swapped = false;
        for i in 0..N - 1 - pass {
            if done >= steps {
                break 'outer;
            }
            if a[i] > a[i + 1] {
                a.swap(i, i + 1);
                swapped = true;
                last = Some((i, i + 1));
            }
            done += 1;
        }
        if !swapped {
            break;
        }
    }
    (a, last)
}

const TOP: f32 = 300.0; // top of the tallest bar, in px
const BASE: f32 = 940.0; // the memory floor

pub fn render(_g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx) {
    let span = 26.54f64;
    let prog = clamp01(tl / span);

    // ---- the room: a warm dark archive
    env::void_grad(cv, Color::hex(0x0c0a10), Color::hex(0x050408), 1.0);
    env::floor_grid(cv, 940, tg, Color::hex(0x241d24), 0.16);
    // shelving unit behind everything
    for k in 0..15 {
        let y = TOP - 120.0 + k as f32 * 26.0;
        cv.rect_a(0, y as i32, W as i32, 1, Color::hex(0x1a151d), 0.5);
    }

    // ---- the sorting pass: 1 comparison per 0.30s of music
    let steps = (tl / 0.30) as usize;
    let (arr, pair) = snapshot(steps);
    let bw = W as f32 / N as f32;
    let slot = |i: usize| i as f32 * bw + bw * 0.5;

    // the pair currently being compared gets an arc lift
    let phase = (tl / 0.30).fract();
    let arc = (phase * std::f64::consts::PI).sin();

    for i in 0..N {
        let h = (BASE - TOP) * arr[i] as f32;
        let x = slot(i);
        let mut y0 = BASE - h;
        let mut lift = 0.0f32;
        if let Some((a, b)) = pair {
            if i == a || i == b {
                lift = (arc * 26.0) as f32;
                y0 -= lift;
            }
        }
        // warmth rises with the value — the sweetest memories float up
        let u = arr[i];
        let col = if u < 0.5 {
            Color::lerp(Color::hex(0x2f3a4f), Color::hex(0xe8c39e), u / 0.5)
        } else {
            Color::lerp(Color::hex(0xe8c39e), pal::c(pal::ROSE), (u - 0.5) / 0.5)
        };
        let w = (bw - 3.0).max(2.0) as i32;
        cv.vgrad(x as i32 - w / 2, y0 as i32, w, (BASE - y0) as i32, col.glow(0.18), col.scale(0.42), 0.92);
        // a bright cap
        cv.rect_a(x as i32 - w / 2, y0 as i32, w, 3, col.glow(0.5), 0.95);
        // index id under the floor line
        if i % 4 == 0 {
            cv.text_px(x, BASE + 26.0, &format!("m{i:02}"), 17.0, false, pal::c(pal::DIM), 0.8, true);
        }
        // the lifting pair gets a guide arc
        if lift > 1.0 {
            cv.rect_a(x as i32 - w / 2, (y0 + h) as i32, w, lift as i32, col.scale(0.5), 0.20);
        }
    }
    // baseline
    cv.rect_a(0, BASE as i32, W as i32, 2, Color::hex(0x2b3547), 0.85);
    cv.rect_a(0, BASE as i32, (W as f32 * prog as f32) as i32, 2, pal::c(pal::AMBER).scale(0.8), 0.9);

    // ---- the bubble: rises on "till the bubble pops" (21.18 → 24.2)
    if tg > 21.18 && tg < 24.6 {
        let p = clamp01((tg - 21.18) / 3.0);
        let bx = W as f32 * 0.66;
        let by = BASE - 120.0 - 300.0 * ease_out_cubic(p) as f32;
        let r = 14.0 + 34.0 * p as f32;
        cv.ring(bx, by, r, 2.4, pal::c(pal::ROSE_PALE), 0.7 * (1.0 - p * 0.6) as f64);
        cv.glow_at(bx, by, r * 1.5, pal::c(pal::ROSE), 0.42 * (1.0 - p) as f64, 2.4);
        cv.disc(bx - r * 0.3, by - r * 0.3, r * 0.22, Color::rgb(255, 255, 255), 0.75 * (1.0 - p) as f64);
        // the moment it pops
        if tg > 23.9 {
            let q = clamp01((tg - 23.9) / 0.6);
            cv.ring(bx, by, r + 220.0 * q as f32, 3.0 * (1.0 - q) as f32 + 0.5, pal::c(pal::ROSE), 0.7 * (1.0 - q));
            for k in 0..14 {
                let th = k as f64 / 14.0 * std::f64::consts::TAU;
                let d = 40.0 + 190.0 * q as f32;
                cv.disc(
                    bx + (th.cos() as f32) * d,
                    by + (th.sin() as f32) * d * 0.8,
                    4.0 * (1.0 - q as f32) + 0.8,
                    pal::c(pal::ROSE_PALE),
                    0.9 * (1.0 - q),
                );
            }
            // and the memory that popped falls away from its bar
            cv.rect_a((bx - 26.0) as i32, by as i32, 52, 2, pal::c(pal::ROSE), 0.5 * (1.0 - q));
        }
    }

    // ---- "Take a memory, insert to the past" (28.05): pluck + file it away
    if tg > 28.05 && tg < 31.4 {
        let p = clamp01((tg - 28.05) / 2.6);
        let e = ease_in_out(p);
        // the sweetest bar (last in the array) is lifted
        let (arr2, _) = snapshot((28.05 / 0.30) as usize);
        let best = arr2
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(N - 1);
        let sx = slot(best);
        let sy = BASE - (BASE - TOP) * arr2[best] as f32;
        let hx = W as f32 * 0.10;
        let hy = 300.0;
        let x = sx + (hx - sx) * e as f32;
        let y = sy + (hy - sy) * e as f32 - (e * std::f64::consts::PI).sin() as f32 * 90.0;
        // the flying card
        cv.rect_a((x - 16.0) as i32, (y - 22.0) as i32, 32, 44, pal::c(pal::AMBER), 0.85 * (1.0 - clamp01((p - 0.86) / 0.14)));
        cv.rect_a(x as i32 - 16, y as i32 - 22, 32, 3, pal::c(pal::ROSE), 0.8);
        cv.text_px(x, y + 8.0, "m", 18.0, true, Color::hex(0x241d24), 0.9, true);
        // the destination drawer
        let a = clamp01((p - 0.55) / 0.45);
        ui::dashed_rect(cv, hx - 46.0, hy - 40.0, 92.0, 80.0, 8.0, pal::c(pal::AMBER), 0.35 + 0.4 * a);
        cv.text_px(hx, hy + 78.0, "past/", 24.0, false, pal::c(pal::AMBER).scale(0.9), a, true);
    }

    // ---- "Which one is the sweetest? Save it before the last" (35.05)
    if tg > 35.05 && tg < 39.4 {
        let p = clamp01((tg - 35.05) / 3.4);
        let (arr2, _) = snapshot((35.05 / 0.30) as usize);
        let best = arr2
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(N - 1);
        let sx = slot(best);
        let sy = BASE - (BASE - TOP) * arr2[best] as f32;
        let pulse = 0.5 + 0.5 * (tg * 5.0).sin();
        cv.glow_at(sx, sy, 90.0 + 30.0 * pulse as f32, pal::c(pal::ROSE), 0.35 * (1.0 - p) as f64, 2.2);
        ui::reticle(cv, sx, sy - 40.0, 46.0, (p * 3.0).min(1.0), tg, pal::c(pal::ROSE), 0.6, clamp01(p * 2.0 - 0.4));
        if p > 0.55 {
            let a = clamp01((p - 0.55) / 0.3);
            cv.text_px(sx, sy - 96.0, "SWEETEST", 26.0, true, pal::c(pal::ROSE), a, true);
        }
    }

    // ---- atmosphere: motes rising out of the archive
    env::dust(cv, tg, 40, 0x3A1D, pal::c(pal::AMBER), 0.22, 200.0, 900.0);
    env::haze(cv, 250.0, 200.0, Color::hex(0x2a1f28), 0.16);
    ui::edge_shade(cv, Color::hex(0x050408), 0.55);

    // ---- the program's readout, written on the data itself
    let _ = ctx;
}
