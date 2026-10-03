//! S10 LIGHT — three E.V.E orbs rise and become her path of light.

use crate::gfx::{pal, Grid};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, sd: f64, ctx: &Ctx) {
    let _ = sd;
    let w = g.w as i64;
    let beat = ctx.beats;
    let energy = ctx.energy;

    let cols = [pal::RED, pal::AMBER, pal::GREEN];

    // ---- darkness with faint dust
    motes(g, t, 16, pal::c(pal::DIM), 66, 0.15);

    // ---- three orbs rise from the floor (0 ~ 2.0)
    let rise = crate::gfx::ease_in_out(crate::gfx::clamp01(lt / 2.0));
    let align = crate::gfx::clamp01((lt - 2.0) / 1.0);
    for (i, col) in cols.iter().enumerate() {
        // start: spread along the bottom; end: on a diagonal path
        let sx_n = 0.22 + i as f64 * 0.28;
        let sy_n = 0.95;
        let ex_n = 0.24 + i as f64 * 0.20;
        let ey_n = 0.72 - i as f64 * 0.22;
        let nx = sx_n * (1.0 - align) + ex_n * align;
        let ny = sy_n * (1.0 - rise * (1.0 - align).min(1.0)) * (1.0 - align) + ey_n * align;
        let px = nx * W_PX as f64;
        let py = (STAGE_Y0 as f64 + ny * STAGE_H as f64 + 1.0) * crate::pix::CELL_H as f64;
        let pulse = 0.75 + 0.35 * beat.pulse(t) as f64;
        // trail
        for k in 1..7 {
            let ty = py - k as f64 * 14.0 * (1.0 - align);
            cv.sprite('·', px as f32 + 4.0, ty as f32, 12.0 - k as f32 * 1.2, pal::c(*col), 0.35 * (1.0 - k as f64 / 7.0));
        }
        cv.sprite_c('●', px as f32, py as f32, 30.0 * pulse as f32, pal::c(*col), 1.0);
        cv.sprite_c('○', px as f32, py as f32, 46.0 * pulse as f32, pal::c(*col), 0.42);
        cv.sprite_c('○', px as f32, py as f32, 66.0 * pulse as f32, pal::c(*col), 0.2);
    }

    // ---- the path: when aligned, cells light up diagonally
    if align > 0.2 {
        let pl = (align - 0.2) / 0.8;
        let steps = 22;
        let lit = (pl * steps as f64).ceil() as i64;
        for k in 0..=lit.min(steps) {
            let u = k as f64 / steps as f64;
            let x = (0.20 + u * 0.58) * w as f64;
            let y = STAGE_Y1 as f64 - u * (STAGE_H as f64 - 5.0) - 1.0;
            let ci = (u * 3.0) as usize % 3;
            let col = pal::c(cols[ci]);
            let a = 0.5 + 0.4 * (1.0 - (lit - k) as f64 / 4.0).max(0.0);
            g.put_alpha(x as i64, y as i64, '▓', col, a.min(1.0) * align);
        }
    }

    // ---- her little light walking the path upward (2.4+)
    if lt > 2.4 {
        let wp = crate::gfx::clamp01((lt - 2.4) / (sd - 2.4));
        let x = (0.20 + wp * 0.58) * W_PX as f64;
        let y = ((STAGE_Y1 as f64 - wp * (STAGE_H as f64 - 5.0)) + 1.0) * crate::pix::CELL_H as f64;
        cv.sprite('◆', x as f32, y as f32, 20.0 + 5.0 * beat.pulse(t) as f32, pal::c(pal::WHITE), 0.95);
        cv.sprite('○', x as f32, y as f32, 34.0, pal::c(pal::ICE), 0.4);
    }

    // ---- merge flash at the end
    if lt > sd - 0.7 {
        let p = ((lt - (sd - 0.7)) / 0.7).min(1.0);
        cv.fx.flash = p * p;
        cv.fx.bloom = 0.5 + p * 0.6;
    }

    // ---- caption
    if lt > 0.5 && lt < 2.4 {
        caption(g, 27, "E.V.E × 3 — detected: warmth", pal::c(pal::AMBER), 0.8);
    }

    cv.fx.grain = 0.05;
    cv.fx.scanline = 0.4;
    cv.fx.bloom = cv.fx.bloom.max(0.35 + 0.25 * energy.rms_at(t) as f64);
}
