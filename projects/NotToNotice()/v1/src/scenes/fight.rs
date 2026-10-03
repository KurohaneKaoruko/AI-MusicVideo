//! S11 FIGHT — "Fight for You": a four-second battle stinger.
//! Three comets (the E.V.E) slash across the screen and converge.

use crate::art;
use crate::gfx::{pal, Grid, Rng};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, sd: f64, ctx: &Ctx) {
    let _ = sd;
    let w = g.w as i64;
    let beat = ctx.beats;
    let energy = ctx.energy;
    let mut rng = Rng::new(((t * 60.0) as u64) ^ 0xF17);

    // ---- camera shake
    let mag = 0.5 + 1.2 * energy.bass_at(t) as f64 + 0.5 * beat.pulse(t);
    let (sx, sy) = crate::fx::shake(t, mag, 4);

    // ---- slash arcs at three beats
    let slashes = [(0.25, pal::ICE), (1.35, pal::RED), (2.45, pal::GOLD)];
    for (si, (t0, col)) in slashes.iter().enumerate() {
        let hit = hit_envelope(lt, *t0, 0.06, 0.5);
        if hit > 0.01 {
            // a diagonal sweep of blocks from one corner
            let dir = if si % 2 == 0 { 1.0 } else { -1.0 };
            let prog = (hit * 1.6).min(1.0);
            for k in 0..30 {
                let u = k as f64 / 30.0;
                if u > prog {
                    break;
                }
                let x = (if dir > 0.0 { u } else { 1.0 - u } * (w as f64 - 6.0) + 3.0) as i64;
                let y = (6.0 + u * 22.0) as i64;
                let ch = if u > prog - 0.08 { '▓' } else { '▒' };
                let a = if u > prog - 0.08 { 0.9 } else { 0.4 * (1.0 - hit) };
                g.put_alpha(x, y, ch, pal::c(*col), a);
                // afterimage sparks
                if rng.f64() < 0.2 {
                    g.put_alpha(x + rng.i64(-2, 2), y + rng.i64(-1, 1), '·', pal::c(*col), 0.5);
                }
            }
            cv.fx.flash = cv.fx.flash.max(hit * 0.25);
            cv.fx.glitch = cv.fx.glitch.max(hit * 0.45);
        }
    }

    // ---- three comets converge to center
    let cols = [pal::RED, pal::AMBER, pal::GREEN];
    for (i, col) in cols.iter().enumerate() {
        let p = crate::gfx::clamp01((lt - 0.4) / 2.9);
        let e = crate::gfx::ease_in_cubic(p);
        let from = [(0.0, 0.15), (1.0, 0.2), (0.5, 0.0)][i];
        let to = (0.5, 0.55);
        let nx = from.0 + (to.0 - from.0) * e;
        let ny = from.1 + (to.1 - from.1) * e;
        let px = nx * W_PX as f64;
        let py = (STAGE_Y0 as f64 + ny * STAGE_H as f64 + 1.0) * crate::pix::CELL_H as f64;
        // trail
        for k in 1..12 {
            let te = (e - k as f64 * 0.012).max(0.0);
            let tx = (from.0 + (to.0 - from.0) * te) * W_PX as f64;
            let ty = (STAGE_Y0 as f64 + (from.1 + (to.1 - from.1) * te) * STAGE_H as f64 + 1.0)
                * crate::pix::CELL_H as f64;
            cv.sprite('·', tx as f32, ty as f32, 13.0 - k as f32, pal::c(*col), 0.5 * (1.0 - k as f64 / 12.0));
        }
        if p < 1.0 {
            cv.sprite('●', px as f32, py as f32, 16.0, pal::c(*col), 1.0);
        }
    }

    // ---- impact (3.3): everything converges in a burst
    let impact = hit_envelope(lt, 3.3, 0.05, 0.8);
    if impact > 0.01 {
        let cx = w / 2;
        let cy = 18;
        art::burst(g, cx, cy, 1.0 - impact, pal::c(pal::WHITE));
        for r in 1..4 {
            let rr = ((1.0 - impact) * 9.0 * r as f64) as i64;
            for k in 0..12 {
                let a = k as f64 / 12.0 * std::f64::consts::PI * 2.0;
                g.put_alpha(cx + (a.cos() * rr as f64 * 2.0) as i64, cy + (a.sin() * rr as f64) as i64, '·', pal::c(pal::STAR), impact * 0.8);
            }
        }
        cv.fx.flash = cv.fx.flash.max(impact * 0.85);
        cv.fx.glitch = cv.fx.glitch.max(impact * 0.8);
        cv.fx.aberration = cv.fx.aberration.max(impact);
    }

    // ---- the big chroma text
    {
        let p = crate::gfx::clamp01(lt / 0.3);
        let shake_txt = ((t * 15.0).sin() * (1.2 + 2.0 * beat.pulse(t))) as i64;
        let baseline = mid_baseline(cv, 46.0, 16) + sy as f32 * 6.0;
        cv.text_px(
            CX_PX + shake_txt as f32 + sx as f32 * 8.0,
            baseline,
            "FIGHT FOR YOU",
            46.0,
            true,
            pal::c(pal::WHITE),
            p,
            true,
        );
        // red echo
        cv.text_px(
            CX_PX + shake_txt as f32 + 4.0,
            baseline + 2.0,
            "FIGHT FOR YOU",
            46.0,
            true,
            pal::c(pal::RED),
            p * 0.4,
            true,
        );
    }

    // ---- fx
    cv.fx.bloom = 0.5 + 0.3 * energy.rms_at(t) as f64;
    cv.fx.grain = 0.08;
    cv.fx.scanline = 0.3;
    let _ = rng;
}
