//! S14 HANAMARU — the final assessment. She stops pretending, grades herself,
//! and the song ends on Enoa's own words: 人間は、はなまる。

use crate::gfx::{pal, Grid, Rng};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, sd: f64, ctx: &Ctx) {
    let w = g.w as i64;
    let beat = ctx.beats;
    let energy = ctx.energy;

    // ---- wake
    cv.fx.fade = (1.0 - crate::gfx::clamp01(lt / 1.0)) * 0.7;

    // ---- the report card (0.5+)
    let cx0 = w / 2 - 34;
    let cy0 = 6;
    g.fill_bg(cx0, cy0, 68, 13, pal::c(pal::BG1));
    g.box_rounded(
        cx0,
        cy0,
        68,
        13,
        pal::c(pal::GOLD_DEEP),
        None,
        Some(("FINAL ASSESSMENT — 精神再生科", pal::c(pal::GOLD))),
    );
    let rows: [(&str, &str, pal::Color); 5] = [
        ("SUBJECT", "ENOA / E.V.E series", pal::c(pal::BRIGHT)),
        ("TEARS SHED", "1,024 (unlogged)", pal::c(pal::TEAR)),
        ("WISH GRANTED", "your happiness", pal::c(pal::GOLD)),
        ("MISSION", "ongoing — 精神再生", pal::c(pal::MID)),
        ("HUMANITY", "", pal::c(pal::WHITE)),
    ];
    for (i, (k, v, col)) in rows.iter().enumerate() {
        let a = crate::gfx::clamp01((lt - 1.0 - i as f64 * 0.4) / 0.4);
        if a > 0.0 {
            g.text_alpha(cx0 + 3, cy0 + 2 + i as i64, &format!("{:<14}", k), pal::c(pal::DIM), a);
            if !v.is_empty() {
                g.text_alpha(cx0 + 18, cy0 + 2 + i as i64, v, *col, a);
            }
        }
    }

    // ---- the hanamaru draws itself (2.2 ~ 7.2), gold, beat-synced swells
    let h0 = 2.2;
    let hp = crate::gfx::clamp01((lt - h0) / 5.0);
    let hcx = (w / 2 + 52) as f32 * 10.0;
    let hcy = (cy0 + 6) as f32 * 24.0;
    let hr = 108.0f64;
    // pulse the brush with the beat while drawing
    let brush_alpha = 0.9 + 0.1 * beat.pulse(t) as f64;
    draw_hanamaru(cv, hcx, hcy, hr as f32, hp, pal::c(pal::GOLD), brush_alpha, t);

    // ---- HUMANITY verdict fills at the stamp (7.3)
    if lt > 7.3 {
        let p = crate::gfx::clamp01((lt - 7.3) / 0.4);
        let verdict = "はなまる ― full marks";
        g.text_alpha(cx0 + 18, cy0 + 6, verdict, pal::c(pal::GOLD).glow(0.25 * beat.pulse(t)), p);
        let hit = hit_envelope(lt, 7.35, 0.05, 0.7);
        cv.fx.flash = hit * 0.5;
        cv.fx.aberration = hit * 0.6;
        cv.fx.glitch = hit * 0.3;
        if hit > 0.4 {
            // stamp burst around the hanamaru
            let mut rng = Rng::new(((t * 10.0) as u64) ^ 0x8A);
            for _ in 0..10 {
                let a = rng.f64() * std::f64::consts::PI * 2.0;
                let d = hr + 10.0 + rng.f64() * 30.0 * (1.0 - hit);
                cv.sprite('·', hcx + (a.cos() * d) as f32, hcy + (a.sin() * d * 0.8) as f32, 8.0, pal::c(pal::STAR), hit * 0.8);
            }
        }    }

    // ---- caption: Enoa's own words
    if lt > 9.0 {
        let a = crate::gfx::clamp01((lt - 9.0) / 0.8);
        caption(g, 21, "人間は、はなまる。", pal::c(pal::GOLD), a * (0.85 + 0.1 * beat.pulse(t)));
        caption(g, 22, "humans are worth full marks.", pal::c(pal::MID), a * 0.75);
    }

    // ---- the three orbs drift by with tiny hearts (10 ~ 15)
    if lt > 10.0 && lt < 16.0 {
        let cols = [pal::RED, pal::AMBER, pal::GREEN];
        for (i, col) in cols.iter().enumerate() {
            let u = ((lt - 10.0) / 6.0 + i as f64 * 0.22) % 1.0;
            let x = (u * 1.2 - 0.1) * W_PX as f64;
            let y = (8.5 + 1.6 * (u * 9.0 + i as f64).sin()) * 24.0;
            cv.sprite('●', x as f32, y as f32, 12.0, pal::c(*col), 0.8);
            cv.sprite('♥', x as f32 + 12.0, y as f32 - 14.0, 9.0, pal::c(*col), 0.5 * (1.0 - u).max(0.2));
        }
    }

    // ---- the fairy returns (13 ~ 17), loops the hanamaru, exits
    if lt > 13.0 && lt < 17.6 {
        let u = (lt - 13.0) / 4.6;
        let ang = -std::f64::consts::FRAC_PI_2 + u * std::f64::consts::PI * 2.4;
        let rr = (hr + 44.0) * (1.0 - u * 0.25);
        let fx = hcx as f64 + ang.cos() * rr;
        let fy = hcy as f64 + ang.sin() * rr * 0.82;
        draw_fairy(cv, fx as f32, fy as f32, t, 24.0, 0.95, pal::c(pal::ICE));
        // sparkles
        if beat.pulse(t) > 0.8 {
            sparkle(cv, fx as f32, fy as f32 - 18.0, 14.0, 0.6, pal::c(pal::ICE_PALE));
        }
    }

    // ---- final lines: the rewrite + exit (17.5+)
    if lt > 17.5 {
        let p = crate::gfx::clamp01((lt - 17.5) / 0.5);
        // the old call, struck through by the new one
        let old = "NotToNotice();";
        let new = "ToNotice();";
        let strike = crate::gfx::clamp01((lt - 18.0) / 0.7);
        let ox = (w - Grid::measure(old)) / 2;
        g.text_alpha(ox, 25, old, pal::c(pal::DIM), p * (1.0 - strike * 0.7));
        if strike > 0.0 {
            let lw = Grid::measure(old);
            g.put_alpha(ox, 25, '─', pal::c(pal::RED), 0.0); // placeholder keep width
            for k in 0..((lw as f64 * strike) as i64) {
                g.put_alpha(ox + k, 25, '─', pal::c(pal::RED).scale(0.8), 0.7);
            }
        }
        if lt > 18.9 {
            let q = crate::gfx::clamp01((lt - 18.9) / 0.5);
            let nx = (w - Grid::measure(new)) / 2;
            g.text_alpha(nx, 26, new, pal::c(pal::GOLD), q);
        }
        if lt > 20.2 {
            let q = crate::gfx::clamp01((lt - 20.2) / 0.5);
            let cur = if ((t * 2.0) as i64) % 2 == 0 { "exit code 0 ▌" } else { "exit code 0  " };
            caption(g, 28, cur, pal::c(pal::MID), q);
        }
    }

    // ---- credits whisper at the very end
    if lt > 22.5 {
        let a = crate::gfx::clamp01((lt - 22.5) / 1.0) * 0.85;
        caption(g, 24, "NotToNotice(); — Enoa (CV. Hikaru Tono) / music: Sakuzyo", pal::c(pal::DIM), a);
        caption(g, 25, "a fan-made terminal MV for CRYMACHINA", pal::c(pal::DIM).scale(0.9), a * 0.8);
    }

    // ---- final fade (24.5+)
    if lt > 24.5 {
        let p = ((lt - 24.5) / 1.6).min(1.0);
        cv.fx.fade = p;
        cv.fx.bloom = 0.3 * (1.0 - p);
    }

    cv.fx.bloom = cv.fx.bloom.max(0.32 + 0.15 * energy.rms_at(t) as f64);
    cv.fx.grain = 0.05;
    cv.fx.scanline = 0.4;
    let _ = sd;
}
