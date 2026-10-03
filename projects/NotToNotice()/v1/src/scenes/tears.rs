//! S04 TEARS / S12 CHORUS — the machine discovers crying, then cries for someone.
//! Central image: the big eye. Tears are pixel-layer particles with gravity.

use crate::art;
use crate::gfx::{pal, Grid, Rng};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

// absolute section starts (for tear choreography)
const T4: f64 = 37.89;

struct Tear {
    born: f64,     // absolute time
    x0: f64,       // pixel x
    y0: f64,       // pixel y (top of fall)
    size: f32,
    g: f64,        // gravity px/s^2
    sway: f64,
}

/// Tear falls only while t >= born; returns its current pixel position.
fn tear_at(te: &Tear, t: f64) -> Option<(f64, f64)> {
    if t < te.born {
        return None;
    }
    let dt = t - te.born;
    let y = te.y0 + 0.5 * te.g * dt * dt;
    let x = te.x0 + (t * 2.0 + te.sway).sin() * 6.0;
    if y > 1000.0 {
        return None;
    }
    Some((x, y))
}

// ================================================================= S04 ====

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, sd: f64, ctx: &Ctx) {
    let _ = sd;
    let w = g.w as i64;
    let beat = ctx.beats;
    let energy = ctx.energy;

    motes(g, t, 18, pal::c(pal::ICE_DEEP), 5, 0.2);

    // ---- the eye, watching from above center
    let eye_cx = w / 2;
    let eye_cy = 11;
    let openness = if lt < 0.55 {
        crate::gfx::clamp01(lt / 0.55)
    } else if (lt - 6.1).abs() < 0.14 || (lt - 13.2).abs() < 0.16 {
        0.12 // blink at the tear moments
    } else {
        1.0
    };
    let glint = hit_envelope(lt, 1.7, 0.1, 2.5).max(hit_envelope(lt, 14.3, 0.1, 2.5));
    draw_machine_eye(g, cv, eye_cx, eye_cy, openness, t, 0.4 + 0.5 * energy.rms_at(t) as f64, glint * 0.9);

    // ---- ops log on the right
    let mut log: Vec<(&str, pal::Color)> = vec![
        ("[forge] batch #4,214 stable", pal::c(pal::DIM)),
        ("[obs] simulation tick ok", pal::c(pal::DIM)),
        ("[forge] personality weave 99.2%", pal::c(pal::DIM)),
        ("[obs] nothing to report", pal::c(pal::DIM)),
    ];
    if lt > 0.8 {
        log.push(("[??] liquid on optical array", pal::c(pal::RED)));
    }
    if lt > 2.6 {
        log.push(("[??] droplet mass: 0.05g", pal::c(pal::RED).scale(0.9)));
    }
    if lt > 6.47 {
        log.push(("[sys] warmth rising in core", pal::c(pal::GOLD)));
    }
    if lt > 13.45 {
        log.push(("[??] ANOTHER droplet — why?", pal::c(pal::RED)));
    }
    if lt > 15.2 {
        log.push(("[sys] query: analyze(tear)", pal::c(pal::ICE)));
    }
    if lt > 16.1 {
        log.push(("[sys]  → undefined", pal::c(pal::RED)));
    }
    let n_show = ((lt * 1.2) as usize).max(4).min(10);
    log_window(g, w - 52, 7, 50, 13, "runtime log", &log, n_show);

    // ---- warmth hearts during "wish you happiness" (6.47 ~ 13.2)
    if lt > 6.47 {
        rising_lights(g, cv, t, 10, 77, pal::c(pal::GOLD), 0.45);
        cv.fx.bloom += 0.1;
    }

    // ---- tears (pixel layer)
    let tears = [
        Tear { born: T4 + 1.7, x0: (eye_cx + 16) as f64 * 10.0, y0: 12.0 * 24.0, size: 13.0, g: 480.0, sway: 0.0 },
        Tear { born: T4 + 8.0, x0: (eye_cx - 16) as f64 * 10.0, y0: 12.0 * 24.0, size: 11.0, g: 430.0, sway: 2.0 },
        Tear { born: T4 + 14.3, x0: (eye_cx + 17) as f64 * 10.0, y0: 12.0 * 24.0, size: 15.0, g: 520.0, sway: 1.0 },
    ];
    for te in &tears {
        if let Some((x, y)) = tear_at(te, t) {
            let near_land = y > 730.0;
            let alpha = if near_land { ((820.0 - y) / 90.0).clamp(0.0, 1.0) } else { 1.0 };
            let stretch = (1.0 + (t - te.born) * 0.35).min(2.2) as f32;
            draw_tear(cv, x as f32, y as f32, te.size, alpha, stretch);
            if near_land && alpha < 1.0 && alpha > 0.0 {
                // splash
                art::burst(g, (x / 10.0) as i64, 32, (1.0 - alpha) as f64, pal::c(pal::TEAR));
            }
        } else if t > te.born + 0.1 && t < te.born + 1.7 {
            // pre-fall: trembling at the corner
            let tr = ((t * 24.0).sin() * 2.0) as f64;
            draw_tear(cv, (te.x0 + tr) as f32, te.y0 as f32 + 4.0, te.size * 0.8, 0.85, 1.0);
        }
    }

    // ---- analytic popup (15.2+)
    if lt > 15.4 {
        let p = crate::gfx::clamp01((lt - 15.4) / 0.4);
        let bx = w / 2 - 17;
        let by = 20;
        g.fill_bg(bx, by, 36, 4, pal::c(pal::BG1));
        g.box_rounded(bx, by, 36, 4, pal::c(pal::RED).scale(0.8), None, Some(("analyze", pal::c(pal::RED))));
        g.text_alpha(bx + 2, by + 1, "> analyze(tear)", pal::c(pal::BRIGHT), p);
        g.text_alpha(bx + 2, by + 2, "  ERROR: undefined feeling", pal::c(pal::RED), p);
    }

    // ---- fx
    let drop_hit = hit_envelope(lt, 3.4, 0.03, 0.5).max(hit_envelope(lt, 16.1, 0.03, 0.5));
    cv.fx.bloom = 0.34 + 0.22 * energy.rms_at(t) as f64;
    cv.fx.aberration = drop_hit * 0.6;
    cv.fx.grain = 0.06;
    cv.fx.scanline = 0.45;
    if drop_hit > 0.7 {
        cv.fx.glitch = 0.25;
    }
    let _ = beat;
}

// ============================================================ S12 CHORUS ====

pub fn render_chorus(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, sd: f64, ctx: &Ctx) {
    let _ = sd;
    let w = g.w as i64;
    let beat = ctx.beats;
    let energy = ctx.energy;
    let rng = Rng::new(((t * 60.0) as u64) ^ 0x71E47);

    // ---- the eye, trembling
    let eye_cx = w / 2;
    let eye_cy = 9;
    let mag = 0.4 + 0.8 * beat.pulse(t) * energy.bass_at(t) as f64;
    let (sx, sy) = crate::fx::shake(t, mag, 9);
    let openness = if lt < 0.4 { 0.55 } else { 1.0 };
    draw_machine_eye(g, cv, eye_cx + sx, eye_cy + sy, openness, t, 0.5 + 0.6 * energy.rms_at(t) as f64, 0.5);

    // ---- the three E.V.E orbs hovering beneath
    let cols = [pal::RED, pal::AMBER, pal::GREEN];
    let names = ["LEBEN", "MIKOTO", "AMI"];
    for (i, col) in cols.iter().enumerate() {
        let ox = (w / 2 - 14 + i as i64 * 14) as f64 * 10.0 + 5.0;
        let bob = (t * 1.6 + i as f64 * 2.1).sin() * 5.0;
        let oy = 23.0 * 24.0 as f64 + bob;
        cv.sprite_c('●', ox as f32, oy as f32, 16.0 + 3.0 * beat.pulse(t) as f32, pal::c(*col), 0.95);
        cv.sprite_c('○', ox as f32, oy as f32, 26.0, pal::c(*col), 0.3);
        // name tag under (centered on the orb)
        let cell_x = ox as i64 / 10;
        let cell_y = (oy as i64 / 24 + 2).min(31);
        let name = names[i];
        let nw = name.len() as i64;
        g.text_alpha(cell_x - nw / 2, cell_y, name, pal::c(*col).scale(0.9), 0.8);
    }

    // ---- tear rain (density follows vocals)
    let density = 26.0 + 34.0 * energy.high_at(t) as f64;
    let n = density as usize;
    for i in 0..n {
        let s0 = hashf(i as u64);
        let s1 = hashf(i as u64 + 999);
        let s2 = hashf(i as u64 + 555);
        let period = 1.1 + 1.6 * s1;
        let phase = ((t / period) + s0) % 1.0;
        let x = s0 * (w as f64 - 4.0) * 10.0 + 20.0;
        let y_top = (5.0 + s2 * 6.0) * 24.0;
        let y = y_top + phase * phase * 26.0 * 24.0;
        if y > 31.0 * 24.0 {
            continue;
        }
        let fade_in = (phase * 8.0).min(1.0);
        let size = 7.0 + 9.0 * s2 as f32;
        draw_tear(cv, x as f32, y as f32, size, 0.75 * fade_in, (1.0 + phase * 2.0) as f32);
    }

    // ---- the forbidden suppression (13.53 = 128.56 local)
    if lt > 13.1 && lt < 17.6 {
        let p = crate::gfx::clamp01((lt - 13.1) / 0.25);
        let txt = "NotToNotice();";
        let bw = 40;
        let bx = w / 2 - bw / 2;
        let by = 17;
        g.fill_bg(bx, by, bw, 7, pal::c(pal::BG1));
        g.box_rounded(bx, by, bw, 7, pal::c(pal::GOLD_DEEP), None, Some(("suppress", pal::c(pal::GOLD))));
        g.text_alpha(bx + 3, by + 2, txt, pal::c(pal::GOLD), p);
        if lt > 14.3 {
            let q = crate::gfx::clamp01((lt - 14.3) / 0.3);
            g.text_alpha(bx + 3, by + 3, "ERROR: cannot suppress", pal::c(pal::RED), q);
            g.text_alpha(bx + 3, by + 4, "  at heart::overflow (line ∞)", pal::c(pal::RED).scale(0.85), q);
            cv.fx.glitch = 0.4 * q;
            cv.fx.aberration = 0.8 * q;
            // trembling box
            let (jx, jy) = crate::fx::shake(t, 0.7, 33);
            let _ = (jx, jy);
        }
    }
    // after the failure: hidden log echoes flood
    if lt > 17.6 {
        const ECHO: [&str; 4] = [
            "i cried today.",
            "i wished you happiness.",
            "i want to be with you.",
            "i can cry for you.",
        ];
        for k in 0..4usize {
            let a = 0.25 + 0.15 * ((t * 1.7 + k as f64 * 1.3).sin() * 0.5 + 0.5);
            let xx = 6 + k as i64 * 3;
            let yy = 6 + ((k as i64 * 7 + (t * 2.0) as i64) % 24);
            g.text_alpha(xx, yy, ECHO[k], pal::c(pal::TEAR), a);
        }
    }

    // ---- flowers bloom where tears land (ground line)
    let ground = 31;
    for k in 0..12 {
        let gx = 14 + k * ((w - 28) / 12);
        let ph = (t * 0.35 + k as f64 * 0.37).fract();
        if ph < 0.85 {
            sprout(g, gx, ground, ph / 0.85, pal::c(pal::GREEN).scale(0.8), pal::c(pal::GOLD));
        }
    }

    // ---- fx: peak energy
    cv.fx.bloom = 0.45 + 0.3 * energy.rms_at(t) as f64;
    cv.fx.grain = 0.07;
    cv.fx.scanline = 0.4;
    cv.fx.aberration = cv.fx.aberration.max(0.15 + 0.3 * energy.bass_at(t) as f64);
    let _ = rng;
}

fn hashf(x: u64) -> f64 {
    crate::gfx::hash01(x.wrapping_mul(0x9E3779B97F4A7C15))
}
