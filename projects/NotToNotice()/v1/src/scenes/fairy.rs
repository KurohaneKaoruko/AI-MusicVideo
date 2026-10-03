//! S01 BLUE FAIRY — the fairy crosses the void and becomes the mission.

use crate::beats::BeatGrid;
use crate::gfx::{pal, Grid};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, sd: f64, ctx: &Ctx) {
    let _ = sd;
    let w = g.w as i64;
    let beat: &BeatGrid = ctx.beats;

    // ---- starfield
    crate::fx::starfield(g, t, 0, STAGE_Y0, w, STAGE_H, 130, 3.2, pal::c(pal::BRIGHT).scale(0.75), 11);

    // ---- fairy path (normalized coords -> pixels)
    let (nx, ny) = {
        if lt < 2.2 {
            let p = crate::gfx::ease_in_out(lt / 2.2);
            (-0.08 + p * 0.45, 0.78 - p * 0.36)
        } else if lt < 5.9 {
            let u = lt - 2.2;
            let cx = 0.40 + 0.03 * u;
            (
                cx + 0.13 * (u * 1.5).sin(),
                0.40 + 0.10 * (u * 2.3).sin() - 0.02 * u,
            )
        } else {
            let p = crate::gfx::clamp01((lt - 5.9) / 0.45);
            let e = crate::gfx::ease_in_out(p);
            (0.60 + e * 0.06, 0.30 + e * 0.10)
        }
    };
    let gone = lt >= 6.32;
    let fx_px = nx * W_PX as f64;
    let fy_px = (STAGE_Y0 as f64 + ny * STAGE_H as f64 + 0.9) * crate::pix::CELL_H as f64;
    let size = 40.0 + 6.0 * beat.pulse(t) as f32;

    if !gone {
        // trail: sample past positions
        for k in 1..9 {
            let tp = lt - k as f64 * 0.055;
            if tp < 0.0 {
                break;
            }
            let (pnx, pny) = fairy_pos(tp);
            let px = (pnx * W_PX as f64) as f32;
            let py = ((STAGE_Y0 as f64 + pny * STAGE_H as f64 + 0.9) * crate::pix::CELL_H as f64) as f32;
            let a = 0.30 * (1.0 - k as f64 / 9.0);
            cv.sprite('·', px + 3.0, py, 10.0 - k as f32, pal::c(pal::ICE), a);
        }
        draw_fairy(cv, fx_px as f32, fy_px as f32, t, size, 1.0, pal::c(pal::ICE));
        // sparkles shed by the wings
        if beat.pulse(t) > 0.75 {
            let mut rng = crate::gfx::Rng::new(((t * 7.0) as u64) ^ 0xF117);
            let ox = (rng.f64() - 0.5) * 30.0;
            let oy = (rng.f64() - 0.5) * 22.0;
            sparkle(cv, (fx_px + ox) as f32, (fy_px + oy) as f32, 16.0, 0.5, pal::c(pal::ICE_PALE));
        }
    } else {
        // dissolve burst into code glyphs
        let p = crate::gfx::clamp01((lt - 6.32) / 0.6);
        let mut rng = crate::gfx::Rng::new(0xAC3);
        for i in 0..26 {
            let a = i as f64 / 26.0 * std::f64::consts::PI * 2.0;
            let d = p * (30.0 + 70.0 * rng.f64());
            let x = fx_px + a.cos() * d;
            let y = fy_px + a.sin() * d * 0.7 + p * p * 40.0;
            let ch = ['0', '1', 'i', 'd', ';', '(', ')', '{', '}'][i % 9];
            cv.sprite(ch, x as f32, y as f32, 17.0, pal::c(pal::ICE), (1.0 - p) * 0.9);
        }
    }

    // ---- mission code materializes where the fairy lands
    if lt > 6.1 {
        let p = crate::gfx::clamp01((lt - 6.1) / 0.5);
        let rows = [
            "void execute(mission) {",
            "  target   = 本物の人間;",
            "  souls    = forge(E.V.E);",
            "  feelings = null;   // not needed",
            "}",
        ];
        let bx = w / 2 - 18;
        let by = 12;
        g.fill_bg(bx - 3, by - 1, 44, rows.len() as i64 + 2, pal::c(pal::BG1));
        g.box_rounded(bx - 3, by - 1, 44, rows.len() as i64 + 2, pal::c(pal::ICE_DEEP), None, Some(("mission.rs", pal::c(pal::MID))));
        let total: f64 = 0.9;
        let mut acc = 0.0;
        for (i, line) in rows.iter().enumerate() {
            let lp = crate::gfx::clamp01((p * total - acc) / (line.len() as f64 / 100.0).max(0.12));
            acc += (line.len() as f64 / 100.0).max(0.12);
            let n = (lp * line.chars().count() as f64).round() as usize;
            let s: String = line.chars().take(n).collect();
            let col = if line.contains("null") { pal::c(pal::DIM) } else { pal::c(pal::BRIGHT) };
            g.text_alpha(bx, by + i as i64, &s, col, p);
        }
    }

    // ---- caption
    if lt > 0.4 && lt < 6.1 {
        caption(g, STAGE_Y1 - 1, "— a blue fairy in the sea of stars —", pal::c(pal::ICE_DEEP), 0.55);
    }

    cv.fx.bloom = 0.38 + 0.15 * ctx.energy.high_at(t) as f64;
    cv.fx.scanline = 0.45;
    cv.fx.grain = 0.05;
}

fn fairy_pos(lt: f64) -> (f64, f64) {
    if lt < 2.2 {
        let p = crate::gfx::ease_in_out(lt / 2.2);
        (-0.08 + p * 0.45, 0.78 - p * 0.36)
    } else if lt < 5.9 {
        let u = lt - 2.2;
        let cx = 0.40 + 0.03 * u;
        (cx + 0.13 * (u * 1.5).sin(), 0.40 + 0.10 * (u * 2.3).sin() - 0.02 * u)
    } else {
        let p = crate::gfx::clamp01((lt - 5.9) / 0.45);
        let e = crate::gfx::ease_in_out(p);
        (0.60 + e * 0.06, 0.30 + e * 0.10)
    }
}
