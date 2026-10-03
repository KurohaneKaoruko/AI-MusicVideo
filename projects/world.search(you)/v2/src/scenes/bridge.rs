//! ACT IV — the lost word (2:17–3:00).
//!
//! The indexing has failed and the film falls out of the machine into a place:
//! a corridor with somebody standing at the far end of it. Questions fall like
//! rain, the word is redacted, tried, stretched — and finally lands.
//!
//! Composition: one-point perspective. Everything converges on a figure that
//! never gets any closer.

use super::ui;
use crate::art;
use crate::env;
use crate::gfx::{clamp01, ease_in_out, ease_out_cubic, hash01, pal, Color, Grid, Rng};
use crate::pix::{Canvas, H, W};
use crate::scenes::Ctx;

const VX: f32 = W as f32 / 2.0;
const VY: f32 = 470.0;

pub fn render(g: &mut Grid, cv: &mut Canvas, _tl: f64, tg: f64, ctx: &Ctx) {
    let slate = pal::c(pal::SLATE);
    let rose = pal::c(pal::ROSE);

    // ---- the corridor
    env::void_grad(cv, Color::hex(0x0a0b12), Color::hex(0x04050a), 1.0);
    // walls receding to the vanishing point
    for k in 0..26 {
        let u = k as f32 / 26.0;
        let spread = (1.0 - u).powf(1.8);
        let x_l = VX - spread * (W as f32 * 0.92);
        let x_r = VX + spread * (W as f32 * 0.92);
        cv.rect_a(x_l as i32, 0, 2, H as i32, Color::hex(0x1b2433), (0.30 * (0.3 + u)) as f64);
        cv.rect_a(x_r as i32, 0, 2, H as i32, Color::hex(0x1b2433), (0.30 * (0.3 + u)) as f64);
        if k % 3 == 0 {
            // ceiling / floor ribs
            cv.line(x_l, 0.0, VX, VY - 40.0, Color::hex(0x161e2b), 0.28, 1.4);
            cv.line(x_r, 0.0, VX, VY - 40.0, Color::hex(0x161e2b), 0.28, 1.4);
            cv.line(x_l, H as f32, VX, VY + 40.0, Color::hex(0x161e2b), 0.28, 1.4);
            cv.line(x_r, H as f32, VX, VY + 40.0, Color::hex(0x161e2b), 0.28, 1.4);
        }
        // a dim window slit on each wall
        if k % 5 == 2 {
            cv.rect_a((x_l + 6.0) as i32, (VY - 90.0 * (1.0 - u) - 40.0) as i32, 10, (60.0 * (1.0 - u) + 8.0) as i32, Color::hex(0x6f87b8), 0.25);
            cv.rect_a((x_r - 16.0) as i32, (VY - 90.0 * (1.0 - u) - 40.0) as i32, 10, (60.0 * (1.0 - u) + 8.0) as i32, Color::hex(0x6f87b8), 0.25);
        }
    }
    // floor
    env::floor_grid(cv, VY as i32 + 40, tg, Color::hex(0x2b3547), 0.22);

    // ---- the figure at the end: never closer
    let flick = if ((tg * 6.0) as i64) % 11 == 0 { 0.35 } else { 1.0 };
    let fw = art::size(&art::FIGURE, 5).0 as f32;
    cv.glow_at(VX, VY + 30.0, 130.0, slate, 0.30 * flick, 2.4);
    art::draw_at(cv, &art::FIGURE, (VX - fw / 2.0) as i32, (VY + 30.0 - 60.0) as i32, 5, flick, None);
    // the rose dot where a heart would be
    cv.disc(VX, VY + 10.0, 3.0, rose, 0.8 * flick);

    // ---- question rain (156.4 → 175.4)
    if tg > 156.0 && tg < 176.0 {
        let intensity = clamp01((tg - 156.0) / 3.0) * clamp01((176.0 - tg) / 3.0);
        for i in 0..70 {
            let a = hash01(0x0D1A + i as u64 * 7919);
            let b = hash01(0x0D1A + i as u64 * 104729);
            let speed = 220.0 + 520.0 * b;
            let y = ((tg * speed + b * 1400.0) % (H as f64 + 200.0)) - 100.0;
            let x = a * W as f64;
            let sz = 20.0 + 34.0 * hash01(0x0D1A + i as u64 * 40503);
            cv.text_px(x as f32, y as f32, "?", sz as f32, false, slate, 0.30 * intensity, false);
        }
    }

    // ---- the resolution panel: a word list that keeps failing
    // (a small dictionary the program keeps guessing from, pinned to the wall)
    let trie: [(&str, f64); 6] = [
        ("piece", 151.84),
        ("dope", 156.42),
        ("door", 158.85),
        ("hope", 161.64),
        ("dope*", 163.66),
        ("love", 177.35),
    ];
    ui::panel(g, 3, 3, 30, 12, "guess", slate, Some(pal::c(pal::BG0).scale(1.1)));
    let mut row = 0i64;
    for (w, at) in trie {
        if tg < at {
            continue;
        }
        let hit = w.starts_with("love");
        let col = if hit { rose } else { pal::c(pal::MID).scale(0.9) };
        g.text(5, 6 + row, w, col);
        if hit {
            g.text(5 + Grid::measure(w) + 1, 6 + row, "√", pal::c(pal::GREEN_OK));
            g.text(5 + Grid::measure(w) + 3, 6 + row, "1 match", pal::c(pal::BRIGHT).scale(0.8));
        } else {
            g.text(5 + Grid::measure(w) + 1, 6 + row, "×", pal::c(pal::ROSE).scale(0.7));
        }
        row += 1;
        if row > 4 {
            break;
        }
    }

    // ---- "A piece of ****" (154.79): a redaction bar tears across the frame
    if tg > 154.79 && tg < 157.4 {
        let p = clamp01((tg - 154.79) / 0.4) * clamp01((157.4 - tg) / 1.2);
        let w = W as f32 * p as f32;
        cv.rect_a((VX - w / 2.0) as i32, (VY - 44.0) as i32, w as i32, 88, Color::hex(0x0b0d12), 0.92 * p);
        // the block itself
        let bw = 620.0 * p as f32;
        cv.rect_a((VX - bw / 2.0) as i32, (VY - 34.0) as i32, bw as i32, 68, Color::hex(0xd7e0ee).scale(0.10), p);
        let mut rng = Rng::new((tg * 60.0) as u64);
        for k in 0..10 {
            let bx = VX - bw / 2.0 + (k as f32 + 0.5) * bw / 10.0;
            cv.rect_a((bx - 26.0) as i32, (VY - 30.0) as i32, 52, 60, Color::hex(0xc9526a), (0.55 + 0.45 * rng.f64()) as f64 * p);
        }
        cv.text_px(VX, VY + 100.0, "REDACTED BY THE SEARCH PROGRAM", 24.0, true, slate, p * 0.8, true);
    }

    // ---- the stretch words: they physically pull apart
    for (at, head, fill, tail) in [
        (163.66f64, "\"D", 'o', "pe\"?"),
        (177.35, "\"L", 'o', "ve\"?"),
    ] {
        if tg < at {
            continue;
        }
        let p = clamp01((tg - at) / 3.0);
        let n = (12.0 * ease_out_cubic(p * 1.6)) as usize;
        let s = format!("{}{}{}", head, fill.to_string().repeat(n), tail);
        let a = (1.0 - clamp01((p - 0.82) / 0.18)) * 0.95;
        let size = 66.0 + 26.0 * (p as f32).sin();
        cv.text_px(VX, VY + 30.0, &s, size, false, rose.glow(0.35), a, true);
        // the "word" bar under it stretches with it
        let w = cv.text_width(&s, size, false);
        cv.rect_a((VX - w / 2.0) as i32, (VY + 52.0) as i32, w as i32, 2, rose, a * 0.6);
    }

    // ---- "If you could stay / stay there where you are" (165.7 / 168.6)
    if tg > 165.73 && tg < 170.4 {
        let p = clamp01((tg - 165.73) / 1.0);
        let fade = clamp01((170.4 - tg) / 1.2);
        // the floor lights up and the figure gains a little ground, then loses it
        let approach = ease_in_out(clamp01((tg - 165.73) / 3.0));
        cv.glow_at(VX, VY + 40.0, 200.0 + 120.0 * approach as f32, rose, 0.22 * p * fade, 2.4);
        let scale = 5 + (approach * 2.0) as i32;
        let w2 = art::size(&art::FIGURE, scale).0 as f32;
        art::draw_at(cv, &art::FIGURE, (VX - w2 / 2.0) as i32, VY as i32, scale, p * fade, None);
    }

    // ---- the answer: LOVE lands and floods the corridor (177.35)
    if tg > 177.35 {
        let p = clamp01((tg - 177.35) / 2.4);
        let e = ease_out_cubic(p);
        cv.glow_at(VX, VY + 40.0, 1400.0 * e as f32, rose, 0.55 * e, 1.9);
        cv.rect_a(0, 0, W as i32, H as i32, rose, 0.10 * e);
        // light races down the corridor walls toward the figure
        for k in 0..22 {
            let u = k as f32 / 22.0;
            let spread = (1.0 - u).powf(1.6);
            cv.rect_a((VX - spread * W as f32 * 0.9) as i32, 0, 2, H as i32, rose, 0.30 * e * (0.2 + u) as f64);
            cv.rect_a((VX + spread * W as f32 * 0.9) as i32, 0, 2, H as i32, rose, 0.30 * e * (0.2 + u) as f64);
        }
        // the figure finally resolves
        art::draw_at(cv, &art::FIGURE, (VX - 20.0) as i32, VY as i32, 7, e * (0.55 + 0.45 * p), None);
    }

    // ---- the program's own voice, coming apart
    if tg > 175.4 {
        let p = clamp01((tg - 175.4) / 0.8);
        ui::panel(g, 3, 3, 34, 6, "guess", slate.scale(0.9), Some(pal::c(pal::BG0).scale(1.1)));
        g.text(5, 6, "love", rose);
        g.text(11, 6, "√", pal::c(pal::GREEN_OK));
        g.text(15, 6, "··· 1 match", pal::c(pal::BRIGHT).scale(0.85));
        let _ = p;
    }

    ui::reticle(cv, VX, VY + 20.0, 340.0, 0.4, tg * 0.2, rose, 0.10, 0.2);
    env::dust(cv, tg, 30, 0xB2D6, slate, 0.20, 100.0, 900.0);
    ui::edge_shade(cv, Color::hex(0x020306), 0.72);
    let _ = ctx;
}
