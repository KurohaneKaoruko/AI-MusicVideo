//! Shared scene furniture: the search program's own widgets, plus the one
//! motif that recurs in every act — the acquisition reticle.

use crate::gfx::{clamp01, pal, Color, Grid, Rng};
use crate::pix::{Canvas, CELL_H, CELL_W, H, W};

pub const CELLW: f32 = CELL_W as f32;
pub const CELLH: f32 = CELL_H as f32;

#[inline]
pub fn cx_px(col: f32) -> f32 {
    col * CELLW
}
#[inline]
pub fn cy_px(row: f32) -> f32 {
    row * CELLH
}
/// Baseline for text vertically centred in cell row `row`.
pub fn baseline(size: f32, row: f32) -> f32 {
    row * CELLH + (CELLH + size * 0.72) / 2.0
}

// ---------------------------------------------------------------- panels ---

/// A search-OS panel: rounded box with a title, drawn on the cell layer.
pub fn panel(g: &mut Grid, x: i64, y: i64, w: i64, h: i64, title: &str, accent: Color, bg: Option<Color>) {
    g.box_rounded(
        x,
        y,
        w,
        h,
        accent.scale(0.55),
        bg.or(Some(pal::c(pal::BG1))),
        if title.is_empty() { None } else { Some((title, accent.scale(0.95))) },
    );
}

/// A property row: `key ............ value   √`
pub fn prop(g: &mut Grid, x: i64, y: i64, w: i64, key: &str, val: &str, state: f64, accent: Color) {
    let on = clamp01(state);
    if on <= 0.02 {
        return;
    }
    let dim = accent.over(pal::c(pal::BG1), on);
    g.text_alpha(x, y, key, pal::c(pal::MID).scale(0.95), on);
    let kw = Grid::measure(key);
    let dots = (w - kw - Grid::measure(val) - 4).max(1);
    for k in 0..dots {
        if k % 2 == 0 {
            g.put_alpha(x + kw + k, y, '·', pal::c(pal::DIM), on * 0.8);
        }
    }
    let vx = x + w - Grid::measure(val) - 2;
    let vcol = if val.contains("NOT FOUND") { pal::c(pal::ROSE) } else { dim };
    g.text_alpha(vx, y, val, vcol, on);
    let ok = !val.contains("NOT FOUND");
    g.put_alpha(x + w, y, if ok { '√' } else { '×' }, if ok { pal::c(pal::GREEN_OK) } else { pal::c(pal::ROSE) }, on);
}

/// Typewriter command echo.
pub fn cmd(g: &mut Grid, x: i64, y: i64, s: &str, elapsed: f64, accent: Color) {
    let n = ((elapsed.max(0.0) * 42.0) as usize).min(s.chars().count());
    let shown: String = s.chars().take(n).collect();
    g.text(x, y, "$ ", accent.scale(0.8));
    g.text(x + 2, y, &shown, pal::c(pal::BRIGHT).scale(0.92));
    if n < s.chars().count() && ((elapsed * 6.0) as i64) % 2 == 0 {
        g.put(x + 2 + Grid::measure(&shown), y, '▌', accent);
    }
}

// ------------------------------------------------------------- reticle ---

/// The acquisition reticle — the film's recurring motif. `open` 0..1 opens the
/// four corner brackets; `spin` rotates the tick ring.
#[allow(clippy::too_many_arguments)]
pub fn reticle(cv: &mut Canvas, cx: f32, cy: f32, r: f32, open: f64, spin: f64, color: Color, alpha: f64, locked: f64) {
    let o = clamp01(open) as f32;
    let a = alpha;
    if a <= 0.004 {
        return;
    }
    // tick ring
    let ticks = 48;
    for i in 0..ticks {
        let th = i as f64 / ticks as f64 * std::f64::consts::TAU + spin;
        let long = i % 4 == 0;
        let r0 = r * if long { 1.0 } else { 1.06 };
        let r1 = r * if long { 0.90 } else { 1.0 };
        let (s, c) = th.sin_cos();
        cv.line(
            cx + (c as f32) * r0,
            cy + (s as f32) * r0,
            cx + (c as f32) * r1,
            cy + (s as f32) * r1,
            color,
            a * if long { 0.85 } else { 0.45 },
            1.4,
        );
    }
    // inner ring, thickening when locked on
    cv.ring(cx, cy, r * 0.78, 1.6 + 2.2 * locked as f32, color, a * (0.35 + 0.55 * locked));
    if locked > 0.02 {
        cv.ring(cx, cy, r * 0.78 * (1.0 + 0.06 * (1.0 - locked) as f32), 2.0, pal::c(pal::ROSE_PALE), a * locked);
    }
    // four corner brackets opening outward
    let br = r * (0.42 + 0.42 * o);
    let len = r * 0.34;
    for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let x = cx + sx * br;
        let y = cy + sy * br;
        cv.rect_a(x as i32 - (len as i32) / 2, y as i32 - 1, len as i32, 2, color, a * 0.9);
        cv.rect_a(x as i32 - 1, y as i32 - (len as i32) / 2, 2, len as i32, color, a * 0.9);
    }
    // crosshair
    cv.rect_a(cx as i32 - (r * 0.16) as i32, cy as i32, (r * 0.32) as i32, 1, color, a * 0.7);
    cv.rect_a(cx as i32, cy as i32 - (r * 0.16) as i32, 1, (r * 0.32) as i32, color, a * 0.7);
}

/// A thin acquisition bracket around a specimen (like a scanner frame).
pub fn bracket(cv: &mut Canvas, x: f32, y: f32, w: f32, h: f32, color: Color, alpha: f64, corner: f32) {
    let c = corner;
    let a = alpha;
    let mut p = |x0: f32, y0: f32, x1: f32, y1: f32| {
        cv.rect_a(x0 as i32, y0 as i32, (x1 - x0).max(1.0) as i32, (y1 - y0).max(1.0) as i32, color, a);
    };
    // corners
    p(x, y, x + c, y + 2.0);
    p(x, y, x + 2.0, y + c);
    p(x + w - c, y, x + w, y + 2.0);
    p(x + w - 2.0, y, x + w, y + c);
    p(x, y + h - 2.0, x + c, y + h);
    p(x, y + h - c, x + 2.0, y + h);
    p(x + w - c, y + h - 2.0, x + w, y + h);
    p(x + w - 2.0, y + h - c, x + w, y + h);
}

/// Dashed rectangle (a target zone / a redaction frame).
pub fn dashed_rect(cv: &mut Canvas, x: f32, y: f32, w: f32, h: f32, dash: f32, color: Color, alpha: f64) {
    let mut t = 0.0f32;
    while t < w {
        let seg = dash.min(w - t);
        cv.rect_a((x + t) as i32, y as i32, seg as i32, 1, color, alpha);
        cv.rect_a((x + t) as i32, (y + h) as i32, seg as i32, 1, color, alpha);
        t += dash * 2.0;
    }
    let mut t = 0.0f32;
    while t < h {
        let seg = dash.min(h - t);
        cv.rect_a(x as i32, (y + t) as i32, 1, seg as i32, color, alpha);
        cv.rect_a((x + w) as i32, (y + t) as i32, 1, seg as i32, color, alpha);
        t += dash * 2.0;
    }
}

/// A scanner pass-line sweeping over a rectangle.
pub fn scanline(cv: &mut Canvas, x: f32, y: f32, w: f32, h: f32, phase: f64, color: Color, alpha: f64) {
    let yy = y + (h as f64 * (phase % 1.0)) as f32;
    cv.rect_a(x as i32, yy as i32, w as i32, 2, color, alpha);
    cv.rect_a(x as i32, (yy - 3.0) as i32, w as i32, 1, color, alpha * 0.4);
    for k in 0..10 {
        let u = k as f32 / 10.0;
        cv.rect_a(
            (x + w * u) as i32,
            (yy + 4.0 + k as f32 * 0.6) as i32,
            8,
            1,
            color,
            alpha * 0.25 * (1.0 - u) as f64,
        );
    }
}

/// The PERFECT stamp — a rubber stamp landing on the specimen.
pub fn stamp(cv: &mut Canvas, g: &mut Grid, cx: f32, cy: f32, p: f64, accent: Color) {
    let p = clamp01(p);
    if p <= 0.0 {
        return;
    }
    // impact scale: overshoot then settle
    let s = if p < 0.25 {
        2.4 - 1.4 * (p / 0.25)
    } else {
        1.0 + 0.12 * (-((p - 0.25) * 7.0)).exp() * ((p - 0.25) * 40.0).sin()
    };
    let a = clamp01(p * 3.0) * (1.0 - clamp01((p - 0.80) / 0.20));
    let w = 320.0 * s as f32;
    let h = 92.0 * s as f32;
    let x = cx - w / 2.0;
    let y = cy - h / 2.0;
    let col = accent;
    // frame
    cv.rect_a(x as i32, y as i32, w as i32, 3, col, a * 0.95);
    cv.rect_a(x as i32, (y + h - 3.0) as i32, w as i32, 3, col, a * 0.95);
    cv.rect_a(x as i32, y as i32, 3, h as i32, col, a * 0.95);
    cv.rect_a((x + w - 3.0) as i32, y as i32, 3, h as i32, col, a * 0.95);
    dashed_rect(cv, x + 8.0, y + 8.0, w - 16.0, h - 16.0, 7.0, col, a * 0.5);
    cv.text_px(cx, cy + 26.0 * s as f32, "PERFECT", (64.0 * s) as f32, true, col, a, true);
    // a small caption under the stamp
    let _ = g;
    let mut rng = Rng::new(0x5EED);
    for _ in 0..8 {
        let ix = cx + (rng.f64() as f32 - 0.5) * w;
        let iy = cy + (rng.f64() as f32 - 0.5) * h;
        cv.sprite('·', ix, iy, 14.0, col, a * 0.5);
    }
}

// ---------------------------------------------------------------- misc ---

/// A readout strip along the bottom of a panel.
pub fn readout(cv: &mut Canvas, x: f32, y: f32, s: &str, size: f32, color: Color, alpha: f64) {
    cv.text_px(x, y, s, size, false, color, alpha, false);
}

/// Soft floor shadow under a subject.
pub fn shadow(cv: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, dark: Color, alpha: f64) {
    cv.ellipse(cx, cy, rx, ry, dark, alpha);
}

/// Warm light spilling from a point.
pub fn lamp(cv: &mut Canvas, x: f32, y: f32, r: f32, color: Color, amount: f64) {
    cv.glow_at(x, y, r, color, amount, 2.2);
    cv.disc(x, y, r * 0.05, Color::rgb(255, 250, 240), 0.9);
}

/// Deterministic value noise in 0..1 for uneven organic motion.
pub fn wob(t: f64, seed: u64, speed: f64) -> f64 {
    let s = (t * speed) as f64 + (seed % 977) as f64;
    (s.sin() * 0.5 + (s * 1.7).sin() * 0.3 + (s * 0.37).sin() * 0.2) * 0.5 + 0.5
}

/// Screen-space vignette rectangle in pixels (the frame edge of the world).
pub fn edge_shade(cv: &mut Canvas, color: Color, alpha: f64) {
    let _ = (W, H);
    let band = 90;
    for k in 0..band {
        let a = alpha * (1.0 - k as f64 / band as f64).powi(2);
        cv.rect_a(0, k, W as i32, 1, color, a);
        cv.rect_a(0, H as i32 - 1 - k, W as i32, 1, color, a);
        cv.rect_a(k, 0, 1, H as i32, color, a);
        cv.rect_a(W as i32 - 1 - k, 0, 1, H as i32, color, a);
    }
}
