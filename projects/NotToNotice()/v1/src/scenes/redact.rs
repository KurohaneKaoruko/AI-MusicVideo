//! S05 REDACT — "Pretend not to notice": the title function hides the tears.

use crate::gfx::{pal, Grid, Rng};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, _sd: f64, ctx: &Ctx) {
    let _ = ctx;
    let w = g.w as i64;

    // ---- frozen aftermath: the closed eye, dimmed
    let eye_cx = w / 2;
    let op = (1.0 - crate::gfx::clamp01(lt / 0.5)).max(0.0);
    draw_machine_eye(g, cv, eye_cx, 9, op * 0.9, t, 0.25, 0.0);

    // ---- ghost tears stopped mid-air, being erased
    if lt < 3.4 {
        for k in 0..2 {
            let x = eye_cx + if k == 0 { 16 } else { -16 };
            let y = 14 + k * 3;
            let alpha = (1.0 - lt / 3.4).max(0.0);
            g.put_alpha(x, y, '●', pal::c(pal::TEAR), alpha * 0.5);
        }
    }

    // ---- the call stamps in (0.55 ~ 1.5)
    if lt > 0.55 {
        let p = crate::gfx::clamp01((lt - 0.55) / 0.4);
        let e = crate::gfx::ease_out_back(p);
        let title = "NotToNotice( tears );";
        let tw = crate::art::big_text_width(title, 2);
        let x = (w - tw) / 2;
        let y0 = 12 + ((1.0 - e) * -5.0) as i64;
        crate::art::big_text(g, x + 1, y0 + 1, title, pal::c(pal::GOLD_DEEP).scale(0.55), 2, 0.55 * p);
        crate::art::big_text(g, x, y0, title, pal::c(pal::GOLD), 2, p);

        let hit = hit_envelope(lt, 0.95, 0.04, 0.55);
        cv.fx.flash = hit * 0.4;
        cv.fx.aberration = hit * 0.9;
        cv.fx.glitch = hit * 0.5;
    }

    // ---- redaction sweep (1.6 ~ 3.4): gold blocks eat the screen
    let sw0 = 1.6;
    let sw1 = 3.4;
    if lt > sw0 && lt < sw1 {
        let p = crate::gfx::ease_in_out((lt - sw0) / (sw1 - sw0));
        let cols = ((w as f64) * p) as i64;
        let mut rng = Rng::new(((t * 20.0) as u64) ^ 0xD1);
        for x in 0..cols.min(w) {
            for y in 4..32 {
                if rng.f64() < 0.22 {
                    let ch = ['█', '▓', '▒'][rng.i64(0, 2) as usize];
                    let a = 0.35 + 0.5 * rng.f64();
                    g.put_alpha(x, y, ch, pal::c(pal::GOLD_DEEP), a);
                }
            }
        }
        // leading edge
        let ex = cols;
        if ex < w {
            for y in 4..32 {
                g.put(ex, y, if y % 2 == 0 { '▌' } else { '░' }, pal::c(pal::GOLD));
            }
        }
        cv.fx.glitch = 0.3;
    }

    // ---- the sweep resolves into a clean "ALL NOMINAL" console (3.4+)
    if lt > sw1 {
        let p = crate::gfx::clamp01((lt - sw1) / 0.5);
        let line1 = "tears: ██████████ → [suppressed]";
        let line2 = "status: ALL SYSTEMS NOMINAL";
        let y = 17;
        g.text_alpha((w - Grid::measure(line1)) / 2, y, line1, pal::c(pal::DIM), p);
        let led = if ((t * 1.4) as i64) % 2 == 0 { '●' } else { '●' };
        let lx = (w - Grid::measure(line2)) / 2 - 2;
        g.put_alpha(lx, y + 2, led, pal::c(pal::GREEN), p * (0.6 + 0.4 * (t * 2.2).sin()));
        g.text_alpha((w - Grid::measure(line2)) / 2, y + 2, line2, pal::c(pal::GREEN).scale(0.9), p);

        // one last tear, hidden in the corner, still glinting
        if lt > 4.2 {
            let glint = 0.4 + 0.35 * (t * 3.1).sin().max(0.0);
            g.put_alpha(5, 30, '·', pal::c(pal::TEAR), glint);
            cv.sprite('●', 5.0 * 10.0 + 5.0, 30.0 * 24.0 + 10.0, 6.0, pal::c(pal::TEAR), glint * 0.5);
        }
    }

    // caption
    if lt > 4.6 {
        caption(g, 27, "— she pretends not to notice —", pal::c(pal::DIM), crate::gfx::clamp01((lt - 4.6) / 0.5));
    }

    cv.fx.bloom = 0.3;
    cv.fx.grain = 0.07;
    cv.fx.scanline = 0.5;
    // fade out toward hakoniwa
    if lt > 6.9 {
        cv.fx.fade = ((lt - 6.9) / 0.9).min(1.0);
    }
}

