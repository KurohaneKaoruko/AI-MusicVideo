//! Environment fields — the full-frame backdrops every scene lives inside.
//! All of them are deterministic and draw straight into the pixel buffer.

use crate::gfx::{hash01, Color};
use crate::pix::{Canvas, H, W};

/// Full-frame vertical gradient (the base atmosphere of a scene).
pub fn void_grad(cv: &mut Canvas, top: Color, bottom: Color, alpha: f64) {
    cv.vgrad(0, 0, W as i32, H as i32, top, bottom, alpha);
}

/// Rect gradient passthrough (same palette call sites read better this way).
#[allow(clippy::too_many_arguments)]
pub fn vgrad(cv: &mut Canvas, x: i32, y: i32, w: i32, h: i32, top: Color, bottom: Color, alpha: f64) {
    cv.vgrad(x, y, w, h, top, bottom, alpha);
}

/// Twinkling star field. `n` stars, drifting very slowly.
pub fn starfield(cv: &mut Canvas, t: f64, n: usize, seed: u64, color: Color, alpha: f64) {
    for i in 0..n {
        let a = hash01(seed + i as u64 * 7919);
        let b = hash01(seed + i as u64 * 104729);
        let c = hash01(seed + i as u64 * 1299709);
        let x = (a * W as f64) as i32;
        let y = (b * H as f64) as i32;
        let tw = 0.45 + 0.55 * ((t * (0.5 + c * 1.6) + c * 9.0).sin() * 0.5 + 0.5);
        let r = 0.7 + c * 1.4;
        cv.disc(x as f32, y as f32, r as f32, color, alpha * tw);
    }
}

/// Thin streaks of light rain with a slant.
pub fn rain(cv: &mut Canvas, t: f64, n: usize, seed: u64, color: Color, slant: f32, alpha: f64) {
    for i in 0..n {
        let a = hash01(seed + i as u64 * 2654435761);
        let b = hash01(seed + i as u64 * 40503);
        let c = hash01(seed + i as u64 * 12289);
        let speed = 900.0 + 1500.0 * c;
        let len = 22.0 + 60.0 * b;
        let y = ((t * speed + b * (H as f64) * 2.0) % (H as f64 + len * 3.0)) - len * 2.0;
        let x = a * (W as f64 + 400.0) + (y * slant as f64);
        let x = x.rem_euclid(W as f64 + 400.0) - 200.0;
        let yf = y as f32;
        let xf = x as f32;
        let lenf = len as f32;
        cv.line(xf, yf, xf - slant * lenf, yf + lenf, color, alpha * (0.35 + 0.5 * c), 1.0);
    }
}

/// Slow floating dust motes, warm, used in interior scenes.
pub fn dust(cv: &mut Canvas, t: f64, n: usize, seed: u64, color: Color, alpha: f64, y0: f32, y1: f32) {
    for i in 0..n {
        let a = hash01(seed + i as u64 * 7919);
        let b = hash01(seed + i as u64 * 104729);
        let c = hash01(seed + i as u64 * 1299709);
        let sp = 8.0 + 26.0 * c;
        let x = (a * W as f64 + (t * sp * 0.4).sin() * 26.0).rem_euclid(W as f64);
        let y = (b * (y1 - y0) as f64 + y0 as f64 - t * sp) as f64;
        let yy = (y - (y0 as f64)).rem_euclid((y1 - y0).max(1.0) as f64) + y0 as f64;
        let tw = 0.4 + 0.6 * ((t * (0.7 + c) + a * 7.0).sin() * 0.5 + 0.5);
        cv.disc(x as f32, yy as f32, (0.7 + c * 1.3) as f32, color, alpha * tw);
    }
}

/// Perspective floor grid receding to `horizon` (a corridor / a data plane).
pub fn floor_grid(cv: &mut Canvas, horizon: i32, t: f64, color: Color, alpha: f64) {
    let cx = W as f32 / 2.0;
    // vanishing lines
    for k in -26..=26 {
        let spread = k as f32 * 120.0;
        cv.line(cx + spread * 0.06, horizon as f32, cx + spread * 3.2, H as f32, color, alpha * 0.5, 1.0);
    }
    // depth rows (scrolling toward the viewer)
    let mut yy = horizon as f32;
    let mut step = 2.0f32;
    let mut k = 0;
    while yy < H as f32 {
        let f = ((yy - horizon as f32) / (H as f32 - horizon as f32)).powf(1.7) as f64;
        let a = alpha * (0.15 + 0.85 * f);
        cv.line(0.0, yy, W as f32, yy, color, a, 1.0);
        let scroll = ((t * 0.6) % 1.0) as f32;
        let _ = scroll;
        step *= 1.22 + 0.02 * ((k as f64 * 0.7).sin() as f32);
        yy += step.max(1.5);
        k += 1;
    }
}

/// Ellipse of light on the floor / a vignette pool.
pub fn pool(cv: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, color: Color, alpha: f64) {
    cv.ellipse(cx, cy, rx, ry, color, alpha);
}

/// Diagonal light shafts (window light), sheared rectangles.
pub fn shafts(cv: &mut Canvas, t: f64, seed: u64, color: Color, alpha: f64) {
    for i in 0..5 {
        let a = hash01(seed + i * 2654435761);
        let b = hash01(seed + i * 40503);
        let x0 = a * (W as f64) * 1.2 - 200.0;
        let wid = 60.0 + 190.0 * b;
        let slope = (0.55 + 0.25 * a) as f32;
        let breathe = 0.75 + 0.25 * (t * 0.6 + a * 6.0).sin();
        let a_eff = alpha * breathe;
        let mut y = 0i32;
        while y < H as i32 {
            let xx = x0 as f32 + (y as f32) * slope;
            let run = (wid * (0.55 + 0.45 * (y as f64 / H as f64))) as i32;
            cv.rect_a(xx as i32, y, run.max(1), 1, color, a_eff);
            y += 1;
        }
    }
}

/// Soft horizontal haze band.
pub fn haze(cv: &mut Canvas, cy: f32, h: f32, color: Color, alpha: f64) {
    let y0 = (cy - h) as i32;
    let y1 = (cy + h) as i32;
    for y in y0..=y1 {
        if y < 0 || y >= H as i32 {
            continue;
        }
        let d = ((y as f32 - cy) / h).abs().min(1.0) as f64;
        let a = alpha * (1.0 - d).powi(2);
        cv.rect_a(0, y, W as i32, 1, color, a);
    }
}

/// A vertical scan band sweeping across the frame (the search reading the world).
pub fn scan_band(cv: &mut Canvas, x: f32, w: f32, color: Color, alpha: f64) {
    let x0 = (x - w) as i32;
    let x1 = (x + w) as i32;
    let n = (x1 - x0).max(1);
    for xx in x0..=x1 {
        if xx < 0 || xx >= W as i32 {
            continue;
        }
        let d = ((xx as f32 - x) / w).abs().min(1.0) as f64;
        let a = alpha * (1.0 - d).powi(3);
        cv.rect_a(xx, 0, 1, H as i32, color, a);
    }
    let _ = n;
}

/// Voronoi / cellular field — the "cut us into pieces" biology.
pub fn cellular(cv: &mut Canvas, t: f64, n: usize, seed: u64, edge_color: Color, fill: Color, alpha: f64) {
    let step = 6usize;
    let sw = W / step;
    let sh = H / step;
    let mut pts: Vec<(f32, f32, f32)> = Vec::with_capacity(n);
    for i in 0..n {
        let a = hash01(seed + i as u64 * 7919);
        let b = hash01(seed + i as u64 * 104729);
        let c = hash01(seed + i as u64 * 1299709);
        let x = (a * sw as f64 + (t * 0.25 + c * 6.0).sin() * 2.0) as f32;
        let y = (b * sh as f64 + (t * 0.21 + a * 6.0).cos() * 2.0) as f32;
        pts.push((x, y, c as f32));
    }
    for gy in 0..sh {
        for gx in 0..sw {
            let fx = gx as f32;
            let fy = gy as f32;
            let mut best = f32::MAX;
            let mut second = f32::MAX;
            let mut bi = 0usize;
            for (i, (px, py, _)) in pts.iter().enumerate() {
                let d = (px - fx) * (px - fx) + (py - fy) * (py - fy);
                if d < best {
                    second = best;
                    best = d;
                    bi = i;
                } else if d < second {
                    second = d;
                }
            }
            let edge = (second.sqrt() - best.sqrt()) < 1.0;
            let col = if edge {
                edge_color
            } else {
                Color::lerp(fill, fill.glow(0.25), pts[bi].2 as f64)
            };
            let a = if edge { alpha } else { alpha * 0.45 };
            cv.rect_a((gx * step) as i32, (gy * step) as i32, step as i32, step as i32, col, a);
        }
    }
}

/// Distant city / structure silhouette along a horizon.
pub fn skyline(cv: &mut Canvas, horizon: i32, seed: u64, color: Color, alpha: f64) {
    let mut x = -40i32;
    let mut i = 0u64;
    while x < W as i32 + 40 {
        let a = hash01(seed + i * 2654435761);
        let b = hash01(seed + i * 40503);
        let w = (26.0 + 90.0 * a) as i32;
        let h = (24.0 + 190.0 * b * b) as i32;
        cv.rect_a(x, horizon - h, w, h, color, alpha);
        // a few lit windows
        let lit = (b * 40.0) as usize % 12;
        for k in 0..lit {
            let wx = x + 4 + ((k * 13 + i as usize * 7) % (w.max(8)) as usize) as i32;
            let wy = horizon - h + 6 + ((k * 29) % (h.max(12)) as usize) as i32;
            cv.rect_a(wx, wy, 2, 3, Color::rgb(255, 214, 170), alpha * 0.55);
        }
        x += w + (4.0 + 22.0 * b) as i32;
        i += 1;
    }
}

/// Droplets clinging to glass (the memory scenes look through a wet window).
pub fn rain_on_glass(cv: &mut Canvas, t: f64, n: usize, seed: u64, color: Color, alpha: f64) {
    for i in 0..n {
        let a = hash01(seed + i as u64 * 7919);
        let b = hash01(seed + i as u64 * 104729);
        let c = hash01(seed + i as u64 * 1299709);
        let speed = 6.0 + 40.0 * c;
        let cycle = 6.0 + 12.0 * a;
        let ph = ((t / cycle + b) % 1.0) as f32;
        let x = (a * W as f64) as f32;
        let y = ph * H as f32;
        let r = (2.0 + 5.0 * c) as f32;
        cv.disc(x, y, r, color, alpha * 0.5);
        cv.disc(x - r * 0.3, y - r * 0.3, r * 0.4, Color::rgb(240, 248, 255), alpha * 0.5);
        let _ = speed;
    }
}

/// Old-film scratches + dust for the memory reel.
pub fn film(cv: &mut Canvas, t: f64, seed: u64, alpha: f64) {
    let mut rng = crate::gfx::Rng::new(((t * 24.0) as u64) ^ seed);
    // vertical scratches
    let n = 2 + (rng.f64() * 3.0) as usize;
    for _ in 0..n {
        let x = rng.i64(0, W as i64 - 1) as i32;
        let a = alpha * (0.25 + 0.4 * rng.f64());
        cv.rect_a(x, 0, 1, H as i32, Color::rgb(230, 226, 214), a * 0.5);
    }
    // dust specks
    for _ in 0..70 {
        let x = rng.i64(0, W as i64 - 1) as i32;
        let y = rng.i64(0, H as i64 - 1) as i32;
        cv.rect_a(x, y, 2, 2, Color::rgb(232, 226, 210), alpha * (0.2 + 0.5 * rng.f64()));
    }
}

/// Warm fog / bloom haze rising from the bottom of the frame.
pub fn ground_fog(cv: &mut Canvas, t: f64, color: Color, alpha: f64, height: f32) {
    let y0 = (H as f32 - height) as i32;
    for y in y0..H as i32 {
        if y < 0 {
            continue;
        }
        let u = (y - y0) as f32 / height.max(1.0);
        let a = alpha * u.powf(1.6) as f64;
        let wob = ((t * 0.7 + y as f64 * 0.05).sin() * 0.5 + 0.5) as f64;
        cv.rect_a(0, y, W as i32, 1, color, a * (0.75 + 0.25 * wob));
    }
}

/// A simple horizon band with a soft edge — the ground plane.
pub fn ground(cv: &mut Canvas, horizon: i32, color: Color, alpha: f64) {
    cv.rect_a(0, horizon, W as i32, H as i32 - horizon, color, alpha);
}
