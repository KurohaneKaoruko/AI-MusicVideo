//! ACT II-C / III-C / VI-C — the radar (1:08, 2:17, 4:21).
//!
//! Same instrument, three states: calm, tearing itself apart, and dim. It is
//! the only scene that is pure instrument — no room, no specimen — and the
//! whole frame is the scan. A heart keeps surfacing on the scope, and the
//! program keeps failing to hold it.

use super::ui;
use crate::art;
use crate::env;
use crate::gfx::{clamp01, hash01, pal, Color, Grid, Rng};
use crate::pix::{Canvas, H, W};
use crate::scenes::Ctx;

#[derive(Clone, Copy)]
pub struct RadarCfg {
    pub sweep: f64,
    pub blips: usize,
    pub dim: f64,
    pub glitch: bool,
    pub heart: bool,
    pub accent: u32,
}

impl RadarCfg {
    pub fn calm() -> RadarCfg {
        RadarCfg { sweep: 1.05, blips: 15, dim: 1.0, glitch: false, heart: true, accent: 0x6fc7b8 }
    }
    pub fn glitch() -> RadarCfg {
        RadarCfg { sweep: 2.1, blips: 22, dim: 1.15, glitch: true, heart: true, accent: 0xff6d8a }
    }
    pub fn dim() -> RadarCfg {
        RadarCfg { sweep: 0.42, blips: 6, dim: 0.42, glitch: false, heart: false, accent: 0x55607a }
    }
}

pub fn render(g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx, cfg: RadarCfg) {
    let accent = pal::c(cfg.accent);
    let cxp = W as f32 / 2.0;
    let cyp = H as f32 * 0.54;
    let r = 430.0f32;
    let beat = ctx.beats.pulse(tg);

    // ---- the scope: a black bowl with a faint phosphor wash
    env::void_grad(cv, Color::hex(0x06100e), Color::hex(0x030607), 1.0);
    cv.disc(cxp, cyp, r * 1.06, accent.scale(0.10), 0.85 * cfg.dim);
    cv.ring(cxp, cyp, r, 3.0, accent, 0.55 * cfg.dim);
    cv.ring(cxp, cyp, r * 0.995, 12.0, accent.scale(0.35), 0.25 * cfg.dim);
    for k in 1..=4 {
        cv.ring(cxp, cyp, r * k as f32 / 5.0, 1.4, accent.scale(0.8), 0.30 * cfg.dim);
    }
    // graticule
    for k in 0..36 {
        let th = k as f64 / 36.0 * std::f64::consts::TAU;
        let long = k % 3 == 0;
        let (s, c) = th.sin_cos();
        cv.line(
            cxp + (c as f32) * r,
            cyp + (s as f32) * r,
            cxp + (c as f32) * (r - if long { 26.0 } else { 12.0 }),
            cyp + (s as f32) * (r - if long { 26.0 } else { 12.0 }),
            accent,
            0.32 * cfg.dim,
            1.2,
        );
    }

    // ---- the sweep, with a decaying trail
    let ang = tl * cfg.sweep * std::f64::consts::TAU / 4.0;
    for k in 0..56 {
        let back = k as f64 * 0.035;
        let a = ang - back;
        let fade = (1.0 - k as f64 / 56.0).powi(2) * 0.22 * cfg.dim;
        let (s, c) = a.sin_cos();
        cv.line(cxp, cyp, cxp + (c as f32) * r, cyp + (s as f32) * r, accent, fade, 1.6);
    }
    // the leading edge
    {
        let (s, c) = ang.sin_cos();
        cv.line(cxp, cyp, cxp + (c as f32) * r, cyp + (s as f32) * r, accent.glow(0.5), 0.95 * cfg.dim, 3.0);
        cv.glow_at(cxp + (c as f32) * r, cyp + (s as f32) * r, 90.0, accent, 0.4 * cfg.dim, 2.0);
    }
    cv.disc(cxp, cyp, 7.0, accent.glow(0.6), 0.9 * cfg.dim);

    // ---- returns: blips lit by the passing arm
    for i in 0..cfg.blips {
        let a = hash01(0xB117 + i as u64 * 7919);
        let b = hash01(0xB117 + i as u64 * 104729);
        let rad = (0.18 + 0.78 * b) * r as f64;
        let th = a * std::f64::consts::TAU;
        let (s, c) = th.sin_cos();
        let x = cxp + (c as f32) * rad as f32;
        let y = cyp + (s as f32) * rad as f32;
        // how recently the arm passed this bearing
        let mut d = (ang - th).rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU;
        if d > 0.5 {
            d = 1.0 - d;
        }
        let lit = (1.0 - d * 3.2).max(0.0);
        if lit > 0.02 {
            let fresh = b > 0.82;
            let col = if fresh { pal::c(pal::ROSE) } else { accent };
            cv.disc(x, y, (3.0 + 5.0 * lit as f32) * if fresh { 1.5 } else { 1.0 }, col, lit * 0.95 * cfg.dim);
            cv.glow_at(x, y, 46.0 * lit as f32 + 8.0, col, 0.5 * lit * cfg.dim, 2.2);
            if fresh {
                // candidates carry a tag
                cv.text_px(x + 16.0, y, "you?", 20.0, false, col, lit * 0.75, false);
            }
        }
    }

    // ---- the heart that keeps surfacing and sinking
    if cfg.heart {
        let period = 6.5;
        let ph = (tg % period) / period;
        let appear = clamp01((ph - 0.28) / 0.16) * clamp01((0.88 - ph) / 0.18);
        if appear > 0.02 {
            let hx = cxp + 190.0 + (tg * 0.5).sin() as f32 * 60.0;
            let hy = cyp - 120.0 + (tg * 0.7).cos() as f32 * 50.0;
            cv.glow_at(hx, hy, 150.0 * appear as f32, pal::c(pal::ROSE), 0.6 * appear, 2.2);
            crate::art::draw_c(cv, &art::HEART, hx, hy, 4, appear * cfg.dim, None);
            // the lock attempt: a shrinking reticle that never quite closes
            let lockp = clamp01((ph - 0.34) / 0.30);
            ui::reticle(cv, hx, hy, 300.0 - 170.0 * lockp as f32, lockp, tg * 0.8, pal::c(pal::ROSE), appear * 0.65 * cfg.dim, lockp);
            if ph > 0.68 {
                let lost = clamp01((ph - 0.68) / 0.18);
                cv.text_px(hx + 40.0, hy - 20.0, "0 results", 26.0, true, pal::c(pal::ROSE), lost * appear * 0.9, false);
                cv.rect_a((hx + 36.0) as i32, (hy - 8.0) as i32, 150, 2, pal::c(pal::ROSE), lost * appear);
            }
        }
    }

    // ---- glitch variant: the instrument tears itself apart
    if cfg.glitch {
        let mut rng = Rng::new((tg * 60.0) as u64 | 1);
        for _ in 0..6 {
            let y = rng.i64(0, H as i64 - 1) as f32;
            let h = 6.0 + rng.f64() as f32 * 26.0;
            let col = if rng.f64() > 0.5 { pal::c(pal::ROSE) } else { accent };
            cv.rect_a(0, y as i32, W as i32, h as i32, col, 0.10 + 0.14 * rng.f64());
        }
        for _ in 0..40 {
            let x = rng.i64(0, W as i64 - 1) as i32;
            let y = rng.i64(0, H as i64 - 1) as i32;
            cv.rect_a(x, y, 4 + (rng.f64() * 40.0) as i32, 2, accent.glow(0.4), 0.35);
        }
        for k in 0..3 {
            let yy = H as f32 * (0.25 + 0.3 * k as f32);
            cv.text_px(
                60.0,
                yy,
                ["SIGNAL LOST", "RE-INDEXING…", "0 results for \"you\""][k],
                28.0,
                true,
                pal::c(pal::ROSE).scale(0.85),
                0.35 + 0.35 * (tg * 2.0 + k as f64).sin().abs(),
                false,
            );
        }
    }

    // ---- the program's panel
    let found = (tl * 1.4) as u64;
    ui::panel(g, 3, 3, 38, 9, "world.search — scan", accent, Some(pal::c(pal::BG0).scale(1.1)));
    let rows: [(&str, String); 4] = [
        ("sweep", format!("{:.2} Hz", cfg.sweep)),
        ("returns", format!("{}", cfg.blips)),
        ("signature", "you?".to_string()),
        ("lock", if cfg.glitch { "FAILED".into() } else { "…".to_string() }),
    ];
    for (i, (k, v)) in rows.iter().enumerate() {
        ui::prop(g, 5, 6 + i as i64, 34, k, v, 1.0, accent);
    }
    ui::readout(cv, 40.0, H as f32 - 60.0, &format!("pass {found:05}"), 22.0, accent, 0.5 * cfg.dim);

    // beat tick around the bezel
    if ctx.beats.downbeat(tg) {
        cv.ring(cxp, cyp, r + 16.0, 3.0, accent.glow(0.4), 0.5 * cfg.dim * (0.5 + 0.5 * beat));
    }
    env::dust(cv, tg, 24, 0x9AD4, accent, 0.16 * cfg.dim, 100.0, 980.0);
    ui::edge_shade(cv, Color::hex(0x020405), 0.7);
    cv.fx.bloom += 0.25;
    let _ = cfg.dim;
}
