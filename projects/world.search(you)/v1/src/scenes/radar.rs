//! Scene: radar search interlude (0:68, 2:17, 4:21) — "I'm searching".
//! A slow radar arm sweeping a grid, blips that light up as it passes, and one
//! blip that keeps coming back as a little heart.

use super::Ctx;
use crate::gfx::{clamp01, Color, Grid, Rng};

const ROSE: Color = Color::hex(0xff6d8a);
const SLATE: Color = Color::hex(0x8a93a6);
const DIM: Color = Color::hex(0x55607a);
const BLIP: Color = Color::hex(0x7ee0a3);

#[derive(Clone, Copy)]
pub struct RadarCfg {
    pub sweep_speed: f64, // rad/s
    pub blips: usize,
    pub dim: f64,
    pub glitch: bool,
    pub heart: bool,
}

/// blip layout (angle, radius fraction) — stable across the whole song
fn blip_at(k: usize) -> (f64, f64) {
    const A: [f64; 10] = [0.42, 1.31, 2.11, 2.74, 3.55, 4.31, 5.02, 5.61, 0.94, 2.41];
    const R: [f64; 10] = [0.55, 0.83, 0.34, 0.68, 0.91, 0.47, 0.74, 0.29, 0.62, 0.88];
    (A[k % 10], R[k % 10])
}

pub fn render(g: &mut Grid, tl: f64, tg: f64, _ctx: &Ctx, cfg: RadarCfg) {
    let w = g.w as f64;
    let h = g.h as f64;
    let cx = w / 2.0;
    let cy = h * 0.45;
    let r = (w * 0.42).min(h * 0.62);

    let sweep = tg * cfg.sweep_speed;
    let pulse = (tg * 2.0).sin() * 0.5 + 0.5;

    // ---- static grid rings + spokes
    for ri in 1..=3 {
        let rr = r * ri as f64 / 3.0;
        let steps = ((rr * 9.0) as i64).max(12);
        for s in 0..steps {
            let a = s as f64 * std::f64::consts::TAU / steps as f64;
            let x = cx + a.cos() * rr;
            let y = cy + a.sin() * rr * 0.52;
            g.put_alpha(x as i64, y as i64, '·', SLATE, cfg.dim * (0.16 + 0.10 * pulse));
        }
    }
    for k in 0..4 {
        let a = k as f64 * std::f64::consts::PI / 4.0;
        let steps = (r * 4.0) as i64;
        for s in 0..steps {
            let f = s as f64 / steps as f64;
            let x = cx + a.cos() * r * f;
            let y = cy + a.sin() * r * 0.52 * f;
            g.put_alpha(x as i64, y as i64, '·', SLATE, cfg.dim * 0.12);
        }
    }
    g.put(cx as i64, cy as i64, '+', SLATE.scale(cfg.dim * 0.5));

    // ---- the sweep arm with a short fading tail
    let tail = 14;
    for k in 0..tail {
        let a = sweep - k as f64 * 0.035;
        let fade = 1.0 - k as f64 / tail as f64;
        let ch = if k < 2 { '▓' } else if k < 5 { '▒' } else { '·' };
        for ri in 1..=9 {
            let rad = r * ri as f64 / 9.0;
            let x = cx + a.cos() * rad;
            let y = cy + a.sin() * rad * 0.52;
            g.put_alpha(x as i64, y as i64, ch, ROSE, cfg.dim * fade * fade * 0.75);
        }
    }
    // arm head
    {
        let x = cx + sweep.cos() * r;
        let y = cy + sweep.sin() * r * 0.52;
        g.put_alpha(x as i64, y as i64, '█', ROSE.glow(0.5), cfg.dim);
        g.put_alpha(x as i64, y as i64 - 1, '·', ROSE.glow(0.4), cfg.dim * 0.5);
    }

    // ---- blips (flash as the arm passes, then settle into a slow blink)
    for k in 0..cfg.blips {
        let (a, rf) = blip_at(k);
        let x = cx + a.cos() * r * rf;
        let y = cy + a.sin() * r * 0.52 * rf;
        let da = (sweep - a).rem_euclid(std::f64::consts::TAU);
        let fresh = clamp01(1.0 - da / std::f64::consts::TAU);
        let blink = ((tg * 2.2 + k as f64 * 0.35) % 2.0) as f64;
        let alpha = cfg.dim * (0.22 + 0.55 * fresh) * (0.45 + 0.55 * blink);
        let ch = if fresh > 0.75 { '█' } else { '•' };
        g.put_alpha(x as i64, y as i64, ch, BLIP, alpha);
    }

    // the special one — a heart, found again and again
    if cfg.heart {
        let (ha, hf) = (2.2f64, 0.62f64);
        let x = cx + ha.cos() * r * hf;
        let y = cy + ha.sin() * r * 0.52 * hf;
        let da = (sweep - ha).rem_euclid(std::f64::consts::TAU);
        let fresh = clamp01(1.0 - da / std::f64::consts::TAU);
        let beat = (tg * 3.0).sin() * 0.5 + 0.5;
        g.put_alpha(
            x as i64,
            y as i64,
            '♥',
            ROSE.glow(0.25 + 0.25 * fresh),
            cfg.dim * (0.45 + 0.35 * fresh) * (0.7 + 0.3 * beat),
        );
    }

    // ---- captions
    g.text(3, 3, "scanning world", ROSE.scale(cfg.dim * 0.85));
    let scans = (tg * 12.0) as u64;
    g.text(
        3,
        4,
        &format!("query 0x{:04x} · arm {:.1} rad/s", scans % 0x10000, cfg.sweep_speed),
        DIM.scale(cfg.dim + 0.4),
    );
    g.text_right(w as i64 - 3, 3, "world.search", DIM.scale(cfg.dim + 0.3));
    if ((tg * 2.0) as i64) % 2 == 0 {
        g.put(w as i64 - 3, 4, '●', ROSE.scale(cfg.dim));
        g.text_right(w as i64 - 5, 4, "rec", DIM.scale(cfg.dim + 0.4));
    }
    g.text_center(cy as i64 + (r * 0.52) as i64 + 3, "target: you", SLATE.scale(cfg.dim));

    // ---- query log at the bottom left
    let queries = [
        "world.search(you) → 0 results… retry",
        "world.search(you) → 0 results… retry…",
        "world.search(you) → 0 results… are you hiding?",
        "world.search(you) → scanning sector world/",
    ];
    let qi = ((tg * 0.6) as usize) % queries.len();
    g.text(3, g.h as i64 - 13, queries[qi], DIM.scale(cfg.dim + 0.3));

    if cfg.glitch {
        let period = 2.0;
        if (tg % period) < 0.08 {
            crate::fx::glitch(g, 0.3, (tg * 100.0) as u64);
        }
    }
    let _ = tl;
    let _ = Rng::new(1);
}
