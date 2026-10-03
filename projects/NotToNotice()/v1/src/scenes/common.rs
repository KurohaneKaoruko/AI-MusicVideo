//! Shared scene components: creatures, windows, captions, procedural art.

use crate::gfx::{hash01, pal, Color, Grid, Rng};
use crate::pix::Canvas;

/// Stage area in cells (between the beat dots and the lyric zone).
pub const STAGE_Y0: i64 = 4;pub const STAGE_Y1: i64 = 32; // exclusive
pub const STAGE_H: i64 = STAGE_Y1 - STAGE_Y0; // height in cells
pub const STAGE_H_PX: i64 = (STAGE_Y1 - STAGE_Y0) * crate::pix::CELL_H as i64;
pub const W_PX: i64 = crate::pix::W as i64;
pub const CX_PX: f32 = (crate::pix::W / 2) as f32;
/// Baseline y for pixel text centred on cell `row`.
///
/// Delegates to the canvas so sprite text and cell text share one baseline model.
/// A local `size * 0.72` descent approximation used to sit ~0.5px off the cell
/// layer's own baseline, which is enough to see when the two are on adjacent rows.
pub fn mid_baseline(cv: &Canvas, size: f32, row: i64) -> f32 {
    cv.baseline_for(size, (row * crate::pix::CELL_H as i64) as i32)
}

// --------------------------------------------------------------- creatures ---

/// The blue fairy: a glowing wisp with flapping wings (sprite layer, sub-cell smooth).
pub fn draw_fairy(cv: &mut Canvas, x: f32, y: f32, phase: f64, size: f32, alpha: f64, color: Color) {
    let flap = (phase * 7.0).sin();
    let ice = color;
    let pale = Color::rgb(234, 247, 255);
    // wings
    let wing_out = 0.55 + 0.25 * flap;
    let wx = size * wing_out as f32;
    let wy = -size * (0.10 + 0.16 * flap) as f32;
    cv.sprite('(', x - wx, y + wy * 0.4, size * 0.62, ice, alpha * 0.75);
    cv.sprite(')', x + wx, y + wy * 0.4, size * 0.62, ice, alpha * 0.75);
    // halo
    cv.sprite_c('○', x, y - size * 0.04, size * 1.25, ice, alpha * 0.35);
    cv.sprite_c('○', x, y - size * 0.04, size * 0.8, pale, alpha * 0.3);
    // core
    cv.sprite_c('●', x, y, size * 0.46, pale, alpha);
    cv.sprite('·', x + size * 0.06, y - size * 0.12, size * 0.3, pale, alpha * 0.9);
}

/// A falling tear: head dot with a stretched tail.
pub fn draw_tear(cv: &mut Canvas, x: f32, y: f32, size: f32, alpha: f64, stretch: f32) {
    let col = pal::c(pal::TEAR);
    let pale = Color::rgb(234, 247, 255);
    if stretch > 1.2 {
        cv.sprite('|', x, y - size * 0.55 * stretch, size * 0.4, col, alpha * 0.5);
    }
    cv.sprite_c('●', x, y, size, col, alpha);
    cv.sprite('˙', x - size * 0.12, y + size * 0.2, size * 0.34, pale, alpha * 0.8);
}

/// Enoa's hanamaru: a brush-drawn circle with an inner curl. `p` 0..1.
pub fn draw_hanamaru(cv: &mut Canvas, cx: f32, cy: f32, r: f32, p: f64, color: Color, alpha: f64, t: f64) {
    if p <= 0.0 {
        return;
    }
    let sweep = std::f64::consts::PI * 2.0 * 1.22 * p.min(1.0); // a bit over full circle
    let start = -std::f64::consts::FRAC_PI_2;
    let n = 130usize;
    let mut rng = Rng::new(0xBEEF);
    let rf = r as f64;
    for i in 0..n {
        let u = i as f64 / n as f64;
        if u > p.min(1.0) {
            break;
        }
        let th = start + sweep * u;
        // brush pressure: slight radius wobble, thinner tail
        let wob = 1.0 + 0.035 * (u * 17.0).sin();
        let tail: f64 = if u > 0.94 { 1.0 - (u - 0.94) / 0.06 * 0.45 } else { 1.0 };
        // after a full circle, curl inward
        let curl = if th > start + std::f64::consts::PI * 2.0 {
            let over = (th - (start + std::f64::consts::PI * 2.0)) / (std::f64::consts::PI * 0.22);
            1.0 - 0.28 * over.clamp(0.0, 1.0)
        } else {
            1.0
        };
        let rr = rf * wob * tail * curl;
        let x = cx + (th.cos() * rr) as f32;
        let y = cy + (th.sin() * rr * 0.96) as f32;
        let jitter = 1.5;
        let dx = (rng.f64() - 0.5) * jitter;
        let dy = (rng.f64() - 0.5) * jitter;
        let bright = color.glow(0.35 * (0.5 + 0.5 * (t * 3.0 + u * 9.0).sin()));
        cv.sprite('●', x + dx as f32, y + dy as f32, r * 0.135, bright, alpha * (0.55 + 0.45 * u.max(0.3)));
    }
}

/// Small star/sparkle at pixel pos.
pub fn sparkle(cv: &mut Canvas, x: f32, y: f32, size: f32, alpha: f64, color: Color) {
    cv.sprite('＋', x, y, size * 0.9, color, alpha * 0.8);
    cv.sprite('·', x, y, size * 0.5, Color::rgb(244, 248, 255), alpha);
}

/// The machine eye: procedural optics — concentric rings, iris, scanline.
/// `openness` 0..1 (eyelid), `tear_glint` adds a wet highlight.
pub fn draw_machine_eye(
    g: &mut Grid,
    cv: &mut Canvas,
    cx: i64,
    cy: i64,
    openness: f64,
    t: f64,
    intensity: f64,
    tear_glint: f64,
) {
    let cpx = cx as f32 * crate::pix::CELL_W as f32 + 5.0;
    let cyp = (cy as f64 + 0.5) * crate::pix::CELL_H as f64;
    let op = openness.clamp(0.0, 1.0);
    if op <= 0.02 {
        // closed: a calm seam
        for k in -8..=8 {
            g.put_alpha(cx + k, cy, if k.rem_euclid(3) == 0 { '─' } else { '⌐' }, pal::c(pal::MID), 0.5);
        }
        return;
    }
    let intensity = intensity as f64;
    let cyp = cyp as f64;
    let cyp = cyp as f32;
    // outer housing ring
    cv.sprite_c('○', cpx, cyp, 200.0, pal::c(pal::ICE_DEEP), 0.30 + 0.12 * intensity as f64);
    cv.sprite_c('○', cpx, cyp, 168.0, pal::c(pal::ICE), 0.5);
    // iris (drops a little when the lid lowers)
    let iris_c = cyp + ((1.0 - op) * 26.0) as f32;
    cv.sprite_c('●', cpx, iris_c, 66.0 + 8.0 * intensity as f32, pal::c(pal::ICE), 0.9 * op);
    cv.sprite_c('●', cpx, iris_c, 34.0, pal::c(pal::TEAR), op);
    // highlight
    cv.sprite_c('˙', cpx - 7.0, iris_c - 9.0, 26.0, Color::rgb(240, 250, 255), op);
    if tear_glint > 0.02 {
        cv.sprite_c('˙', cpx + 26.0, iris_c + 30.0, 20.0, pal::c(pal::TEAR), tear_glint);
    }
    // scanning line across the iris
    let scan_row = cy;
    let phase = ((t * 1.8) % 1.0 * 18.0) as i64 - 9;
    for k in -9..=9 {
        let a = 0.22 + (-(k - phase).abs() as f64 * 0.14).exp() * 0.55;
        g.put_alpha(cx + k, scan_row, '─', pal::c(pal::ICE).glow(0.4), a * op);
    }
    // eyelids: cover lines from top/bottom by openness
    let lid = ((1.0 - op) * 3.0).round() as i64;
    for k in 1..=lid {
        let yy = cy - k * 2;
        for x in cx - 9..=cx + 9 {
            g.put_alpha(x, yy, '▔', pal::c(pal::BG1), 0.9);
        }
        let yy2 = cy + k * 2;
        for x in cx - 9..=cx + 9 {
            g.put_alpha(x, yy2, '▁', pal::c(pal::BG1), 0.9);
        }
    }
}

// ----------------------------------------------------------------- panels ---

/// Scrolling log window. `lines` = newest last. Colors by tag.
pub fn log_window(
    g: &mut Grid,
    x: i64,
    y: i64,
    w: i64,
    h: i64,
    title: &str,
    lines: &[(&str, Color)],
    show_n: usize,
) {
    let (ix, iy, iw, ih) = g.box_rounded(x, y, w, h, pal::c(pal::DIM), Some(pal::c(pal::BG1)), Some((title, pal::c(pal::MID))));
    let n = show_n.min(ih as usize).min(lines.len());
    let start = lines.len() - n;
    for (k, (txt, col)) in lines[start..].iter().enumerate() {
        let mut xx = ix;
        let yy = iy + k as i64;
        // clip to inner width
        let mut w_left = iw;
        for ch in txt.chars() {
            let cw = crate::gfx::char_w(ch) as i64;
            if cw > w_left {
                break;
            }
            g.put(xx, yy, ch, *col);
            for j in 1..cw {
                g.put(xx + j, yy, ' ', *col);
            }
            xx += cw;
            w_left -= cw;
        }
    }
}

/// Caption: centered dim label (grid cells).
pub fn caption(g: &mut Grid, row: i64, s: &str, col: Color, alpha: f64) {
    let x = (g.w as i64 - Grid::measure(s)) / 2;
    g.text_alpha(x, row, s, col, alpha);
}

// ------------------------------------------------------------------ fields ---

/// Gentle dust motes over the stage (deterministic).
pub fn motes(g: &mut Grid, t: f64, n: usize, color: Color, seed: u64, alpha: f64) {
    for i in 0..n {
        let s0 = hash01(seed + (i as u64) * 7919);
        let s1 = hash01(seed + (i as u64) * 104729);
        let s2 = hash01(seed + (i as u64) * 1299709);
        let sp = 0.4 + 1.1 * s2;
        let x = ((s0 * g.w as f64 + t * sp) as i64).rem_euclid(g.w as i64);
        let y = STAGE_Y0 + (((s1 * STAGE_H as f64 - t * sp * 1.7) as i64).rem_euclid(STAGE_H));
        let tw = 0.5 + 0.5 * (t * (1.0 + 2.0 * s2) + s0 * 6.0).sin();
        g.put_alpha(x, y, if s2 > 0.7 { '·' } else { '.' }, color, alpha * tw);
    }
}

/// Rising soul-lights (small warm dots drifting upward).
pub fn rising_lights(g: &mut Grid, cv: &mut Canvas, t: f64, n: usize, seed: u64, col: Color, alpha: f64) {
    for i in 0..n {
        let s0 = hash01(seed + (i as u64) * 65537);
        let s1 = hash01(seed + (i as u64) * 2654435761);
        let cycle = 6.0 + 7.0 * s0;
        let ph = (t / cycle + s1) % 1.0;
        let x = (s0 * (g.w as f64 - 4.0) + 2.0 + (t * 0.7 + s1 * 9.0).sin() * 1.6) as i64;
        let yf = STAGE_Y1 as f64 - ph * STAGE_H as f64;
        let y = yf as i64;
        let a = alpha * (ph.min(0.15) / 0.15).min((1.0 - ph).max(0.0) / 0.2).min(1.0);
        if a <= 0.02 || y < STAGE_Y0 {
            continue;
        }
        if s2_hash(i as u64, seed) > 0.75 {
            let px = (x * crate::pix::CELL_W as i64 + 5) as f32;
            let py = ((y + 1) * crate::pix::CELL_H as i64) as f32;
            cv.sprite('♥', px, py, 11.0 + 4.0 * s0 as f32, col, a * 0.8);
        } else {
            g.put_alpha(x, y, '·', col, a);
        }
    }
}

fn s2_hash(i: u64, seed: u64) -> f64 {
    hash01(seed ^ i.wrapping_mul(0x9E3779B97F4A7C15).rotate_left(17))
}

/// Tiny flower growing from a cell: stem + petals by `p` 0..1.
pub fn sprout(g: &mut Grid, x: i64, y: i64, p: f64, col_stem: Color, col_fl: Color) {
    let stem_h = (1.0 + 2.0 * p).round() as i64;
    for k in 0..stem_h {
        g.put(x, y - k, if k % 2 == 0 { '|' } else { '¦' }, col_stem);
    }
    if p > 0.55 {
        let top = y - stem_h;
        g.put(x, top, '*', col_fl);
        if p > 0.8 {
            g.put_alpha(x - 1, top, '*', col_fl, 0.7);
            g.put_alpha(x + 1, top, '*', col_fl, 0.7);
            g.put_alpha(x, top - 1, '*', col_fl, 0.8);
        }
    }
}

/// The Enoa signature stamp used across scenes (cell art).
pub fn enoa_sigil(g: &mut Grid, x: i64, y: i64, alpha: f64) {
    crate::art::draw(g, &crate::art::SIGIL, x, y, alpha, None);
}

/// Utility: pulse envelope right after a moment (attack-decay).
pub fn hit_envelope(t: f64, moment: f64, attack: f64, decay: f64) -> f64 {
    if t < moment {
        return 0.0;
    }
    let dt = t - moment;
    if dt < attack {
        dt / attack
    } else {
        (-(dt - attack) / decay).exp()
    }
}
