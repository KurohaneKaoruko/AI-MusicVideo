//! Cell-level effects: typewriter, starfields, glyph rain, camera shake.
//!
//! The heavier cell distortions (slice glitch, wipe, full-grid fade) live in the
//! pixel layer instead — `pix::post` applies them after rasterization, so they
//! can also touch sprites and are not limited to cell granularity.

use crate::gfx::{hash01, Color, Grid, Rng};

/// How many chars of `s` are visible after `elapsed` at `speed` chars/s.
pub fn typed(s: &str, elapsed: f64, speed: f64) -> usize {
    let n = (elapsed.max(0.0) * speed) as usize;
    n.min(s.chars().count())
}

/// Starfield layer: deterministic stars drifting (parallax by depth).
pub fn starfield(
    g: &mut Grid,
    t: f64,
    x0: i64,
    y0: i64,
    w: i64,
    h: i64,
    n: u32,
    speed: f64,
    color: Color,
    seed: u64,
) {
    for i in 0..n {
        let s0 = hash01(seed.wrapping_add((i as u64) << 8));
        let s1 = hash01(seed.wrapping_add((i as u64) << 8).wrapping_add(7));
        let s2 = hash01(seed.wrapping_add((i as u64) << 8).wrapping_add(13));
        let depth = 0.3 + 0.7 * s2;
        let x = x0 + ((s0 * w as f64 + t * speed * depth) as i64).rem_euclid(w);
        let y = y0 + (s1 * h as f64) as i64;
        let tw = 0.5 + 0.5 * (t * (1.0 + 3.0 * s2) + s0 * 9.0).sin();
        let a = (0.25 + 0.55 * depth) * tw;
        let ch = if s2 > 0.8 { '+' } else if s2 > 0.5 { '·' } else { '.' };
        g.put_alpha(x, y, ch, color, a);
    }
}

/// Matrix-style falling glyph rain within a window (for data streams).
pub fn glyph_rain(
    g: &mut Grid,
    t: f64,
    x0: i64,
    y0: i64,
    w: i64,
    h: i64,
    density: f64,
    color: Color,
    seed: u64,
) {
    const GLYPHS: &[char] = &[
        '0', '1', 'ά', 'λ', 'ψ', 'σ', 'η', '·', ':', 'x', '+', 'e', 'i', '0', '1', '1', '0',
    ];
    let cols = w;
    for c in 0..cols {
        let cs = hash01(seed.wrapping_add((c as u64) << 12));
        if cs > density {
            continue;
        }
        let speed = 4.0 + 9.0 * hash01(seed ^ ((c as u64) << 3));
        let phase = hash01(seed ^ ((c as u64) << 20)) * (h + 14) as f64;
        let head_y = (phase + t * speed) % (h as f64 + 14.0) - 7.0;
        let len = 4.0 + 9.0 * hash01(seed ^ ((c as u64) << 26));
        for k in 0..len as i64 {
            let y = head_y as i64 - k;
            if y < 0 || y >= h {
                continue;
            }
            let a = (1.0 - k as f64 / len) * 0.8;
            let ch = GLYPHS[(
                hash01(seed ^ ((c as u64) << 6) ^ ((y as u64) << 9) ^ ((t * 3.0) as u64)) * 18.0
            ) as usize % GLYPHS.len()];
            let col = if k == 0 { color.glow(0.5) } else { color };
            g.put_alpha(x0 + c, y0 + y, ch, col, a);
        }
    }
}

/// Shake offset helper: returns (dx,dy) for camera shake with `mag` (cells).
pub fn shake(t: f64, mag: f64, seed: u64) -> (i64, i64) {
    if mag <= 0.01 {
        return (0, 0);
    }
    let mut rng = Rng::new(seed ^ ((t * 30.0) as u64));
    (
        rng.range(-mag, mag).round() as i64,
        (rng.range(-mag, mag).round() as f64 * 0.55) as i64,
    )
}
