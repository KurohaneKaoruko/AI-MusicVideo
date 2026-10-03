//! Scene-transition and post effects: wipes, glitch slices, CRT collapse, noise.

use crate::gfx::{Color, Grid, Rng};

/// Column wipe: covers `frac` (0..1) of the grid from the left with `bg`,
/// with a soft leading edge of accent blocks.
pub fn wipe(g: &mut Grid, frac: f64, bg: Color, edge: Color) {
    let p = frac.clamp(0.0, 1.0);
    if p <= 0.0 {
        return;
    }
    let w = g.w as i64;
    let cols = ((w as f64) * p) as i64;
    for x in 0..cols.min(w) {
        for y in 0..g.h as i64 {
            g.put(x, y, ' ', bg);
            let i = y as usize * g.w + x as usize;
            g.cells[i].bg = bg;
        }
    }
    // leading edge
    let ex = cols;
    if ex < w {
        for y in 0..g.h as i64 {
            let ch = if y % 2 == 0 { '▓' } else { '▒' };
            g.put(ex, y, ch, edge);
        }
    }
}

/// Vertical curtain (rows).
pub fn curtain(g: &mut Grid, prog: f64, bg: Color) {
    let rows = (g.h as f64 * prog.clamp(0.0, 1.0)) as i64;
    for y in 0..rows.min(g.h as i64) {
        for x in 0..g.w as i64 {
            g.put(x, y, ' ', bg);
            let i = y as usize * g.w + x as usize;
            g.cells[i].bg = bg;
        }
    }
}

/// Horizontal slice glitch: offsets random bands, adds color fringing + noise.
pub fn glitch(g: &mut Grid, intensity: f64, seed: u64) {
    let w = g.w;
    let h = g.h;
    let mut rng = Rng::new(seed);
    let bands = (8.0 + intensity * 14.0) as usize;
    for _ in 0..bands {
        let y = rng.i64(0, h as i64 - 1) as usize;
        let bh = rng.i64(1, 3) as usize;
        let dx = rng.i64(-6, 6) * (1 + intensity as i64);
        if dx == 0 {
            continue;
        }
        for yy in y..(y + bh).min(h) {
            let row: Vec<_> = (0..w)
                .map(|x| {
                    let sx = x as i64 - dx;
                    if sx >= 0 && (sx as usize) < w {
                        g.cells[yy * w + sx as usize]
                    } else {
                        g.cells[yy * w + x]
                    }
                })
                .collect();
            for (x, c) in row.into_iter().enumerate() {
                g.cells[yy * w + x] = c;
            }
        }
    }
    // stray noise
    let dots = (intensity * 300.0) as usize;
    for _ in 0..dots {
        let x = rng.i64(0, w as i64 - 1);
        let y = rng.i64(0, h as i64 - 1);
        let c = [Color::hex(0x8a93a6), Color::hex(0xff6d8a), Color::hex(0x6f87b8)][rng.i64(0, 2) as usize];
        g.put_alpha(x, y, ['░', '▒', '·'][rng.i64(0, 2) as usize], c, 0.35 * intensity);
    }
}

/// CRT power-off: rows collapse toward the middle line, then a bright flash.
pub fn crt_collapse(g: &mut Grid, prog: f64, bg: Color) {
    let p = prog.clamp(0.0, 1.0);
    let h = g.h as i64;
    let cy = h / 2;
    // copy grid
    let snap: Vec<_> = g.cells.clone();
    let scale = 1.0 - p * 0.94;
    for y in 0..h {
        let src = cy + ((y - cy) as f64 / scale.max(0.02)).round() as i64;
        for x in 0..g.w as i64 {
            let i = y as usize * g.w + x as usize;
            if src >= 0 && src < h {
                let mut c = snap[src as usize * g.w + x as usize];
                if p > 0.7 {
                    // brighten toward white line
                    let f = 1.0 + (p - 0.7) * 2.5;
                    c.fg = c.fg.scale(f);
                    c.bg = c.bg.scale(f);
                    c.bold = true;
                }
                g.cells[i] = c;
            } else {
                g.cells[i].ch = ' ';
                g.cells[i].fg = bg;
                g.cells[i].bg = bg;
                g.cells[i].bold = false;
            }
        }
    }
}

/// Fade whole grid toward bg by `amount` 0..1.
pub fn fade(g: &mut Grid, amount: f64) {
    let a = amount.clamp(0.0, 1.0);
    if a <= 0.0 {
        return;
    }
    for c in g.cells.iter_mut() {
        c.fg = c.fg.over(c.bg, 1.0 - a);
    }
}

/// Desaturate everything toward gray by `amount`.
pub fn desaturate(g: &mut Grid, amount: f64) {
    let a = amount.clamp(0.0, 1.0);
    if a <= 0.0 {
        return;
    }
    for c in g.cells.iter_mut() {
        let l = c.fg.lum() * 255.0;
        let gray = Color::rgb(l as u8, l as u8, l as u8);
        c.fg = Color::lerp(c.fg, gray, a);
    }
}

/// Fade toward a specific color (e.g. sepia for memories).
pub fn tint_all(g: &mut Grid, color: Color, amount: f64) {
    let a = amount.clamp(0.0, 1.0);
    if a <= 0.0 {
        return;
    }
    for c in g.cells.iter_mut() {
        c.fg = Color::lerp(c.fg, color, a);
    }
}

/// Soft film-grain noise overlay. Only lands on empty cells so text stays readable.
pub fn grain(g: &mut Grid, t: f64, alpha: f64) {
    let mut rng = Rng::new((t * 60.0) as u64 ^ 0x9e37);
    let n = (g.w * g.h) as f64 * alpha * 0.05;
    for _ in 0..n as usize {
        let x = rng.i64(0, g.w as i64 - 1);
        let y = rng.i64(0, g.h as i64 - 1);
        if g.cells[y as usize * g.w + x as usize].ch != ' ' {
            continue;
        }
        let a = alpha * (0.05 + rng.f64() * 0.12);
        g.put_alpha(x, y, '·', Color::hex(0xd9c3a5), a);
    }
}

/// Typewriter helper: how many chars of `s` are visible at time `elapsed` (speed chars/s).
pub fn typed(s: &str, elapsed: f64, speed: f64) -> usize {
    let n = (elapsed.max(0.0) * speed) as usize;
    n.min(s.chars().count())
}
