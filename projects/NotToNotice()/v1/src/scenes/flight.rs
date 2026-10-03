//! S07 EDEN FLIGHT — warp through the starfield into the dyson ring of Eden.

use crate::gfx::{hash01, pal, Grid};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, sd: f64, ctx: &Ctx) {
    let _ = sd;
    let w = g.w as i64;
    let energy = ctx.energy;

    let cxp = w as f64 / 2.0;
    let cyp = (STAGE_Y0 + STAGE_H / 2) as f64;
    // warp speed ramps up
    let speed = 4.0 + (lt / sd).powf(1.4) * 95.0;

    // ---- star streaks radiating from center
    let n = 150;
    for i in 0..n {
        let s0 = hash01(i as u64 * 7349);
        let s1 = hash01(i as u64 * 9241);
        let ang = s0 * std::f64::consts::PI * 2.0;
        let phase = s1;
        let dist = (phase + t * speed / 40.0) % 1.2;
        let d = dist.powf(1.6) * 90.0;
        let x = cxp + ang.cos() * d * 2.0;
        let y = cyp + ang.sin() * d;
        if d < 2.0 {
            continue;
        }
        let bright = (dist * 1.4).clamp(0.0, 1.0);
        // streak: stretch toward center by speed
        let stretch = (speed / 18.0).clamp(0.6, 3.2);
        let ch = if stretch > 2.2 {
            '|'
        } else if stretch > 1.4 {
            ':'
        } else if bright > 0.7 {
            '·'
        } else {
            '.'
        };
        let col = if s1 > 0.8 {
            pal::c(pal::ICE_PALE)
        } else {
            pal::c(pal::BRIGHT).scale(0.6 + 0.4 * bright)
        };
        g.put_alpha(x as i64, y as i64, ch, col, 0.25 + 0.65 * bright);
    }

    // ---- the dyson ring ahead (appears at lt>2.5), grows
    if lt > 2.5 {
        let rp = crate::gfx::clamp01((lt - 2.5) / 9.0);
        let scale = 0.14 + rp * rp * 1.2;
        let rx = 46.0 * scale;
        let ry = 13.0 * scale;
        let segs = 56;
        for k in 0..segs {
            let a = k as f64 / segs as f64 * std::f64::consts::PI * 2.0 + lt * 0.14;
            let x = cxp + a.cos() * rx * 2.0;
            let y = cyp + a.sin() * ry;
            if x < 0.0 || x >= w as f64 || y < STAGE_Y0 as f64 || y > STAGE_Y1 as f64 {
                continue;
            }
            // near/far shading: far side dimmer
            let near = (a.sin() + 1.0) / 2.0;
            let col = pal::c(pal::GOLD).scale(0.5 + 0.5 * near).glow(0.2 * near);
            let ch = if near > 0.75 { '◆' } else if near > 0.4 { '◆' } else { '◇' };
            g.put(x as i64, y as i64, ch, col);
            // connector spokes
            if k % 4 == 0 && near > 0.5 {
                let mx = cxp + a.cos() * rx * 1.4 * 2.0;
                let my = cyp + a.sin() * ry * 1.4;
                if mx >= 0.0 && mx < w as f64 {
                    g.put_alpha(mx as i64, my as i64, '·', pal::c(pal::GOLD_DEEP), 0.4);
                }
            }
        }
    }

    // ---- the central star, swelling
    let star_b = crate::gfx::clamp01((lt - 1.8) / 2.6);
    if star_b > 0.0 {
        let sx = cxp as i64;
        let sy = cyp as i64;
        let pulse = 0.8 + 0.25 * (t * 2.2).sin() + 0.5 * energy.bass_at(t) as f64;
        let sxp = sx;
        let syp = sy;
        for (rr, ch, a) in [
            (1i64, '◆', star_b),
            (2, '◆', star_b * 0.8),
            (3, '◆', star_b * 0.55),
            (5, '◇', star_b * 0.3),
            (7, '·', star_b * 0.2),
        ] {
            for k in -rr..=rr {
                for j in -rr..=rr {
                    if k * k * 4 + j * j <= rr * rr + 1 && (k.abs() + j.abs()) % 2 == (rr % 2) as i64 {
                        g.put_alpha(sxp + k, syp + j, ch, pal::c(pal::STAR), (a * pulse).min(1.0));
                    }
                }
            }
        }
        cv.fx.bloom = 0.4 + star_b * 0.5 + 0.15 * energy.rms_at(t) as f64;
    }

    // ---- captions
    if lt > 1.0 && lt < 5.0 {
        caption(g, 28, "vector locked — 播種構造体「エデン」", pal::c(pal::MID), 0.7);
    }
    if lt > 6.5 && lt < 9.2 {
        caption(g, 28, "EDEN — where souls are farmed from dreams", pal::c(pal::GOLD_DEEP), 0.7);
    }

    // ---- final dive: whiteout
    let dive = crate::gfx::clamp01((lt - sd - 1.2) / 1.0);
    let _ = dive; // computed below with section end
    if lt > 10.4 {
        let p = ((lt - 10.4) / 1.2).min(1.0);
        cv.fx.flash = p * 0.95;
        cv.fx.bloom += p * 0.4;
    }
    // camera shake near the end
    if lt > 9.0 {
        cv.fx.glitch = 0.12 * ((lt - 9.0) / 2.2).min(1.0);
    }
    cv.fx.grain = 0.06;
    cv.fx.scanline = 0.35;
}
