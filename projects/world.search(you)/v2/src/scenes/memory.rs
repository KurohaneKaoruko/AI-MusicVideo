//! ACT VI-B — memory (3:54–4:21).
//!
//! The same two closes as act III, but replayed. Compositionally this is the
//! one act that is *framed*: a strip of film hangs in the middle of the dark
//! and the present-tense world is pushed outside it. Sepia, dust, scratches,
//! and — for the hands — rain on the glass between you and the memory.

use super::ui;
use crate::art;
use crate::env;
use crate::gfx::{clamp01, hash01, pal, Color, Grid};
use crate::pix::{Canvas, H, W};
use crate::scenes::Ctx;

const WIN_X: f32 = 250.0;
const WIN_Y: f32 = 190.0;
const WIN_W: f32 = 1420.0;
const WIN_H: f32 = 700.0;

fn frame(cv: &mut Canvas, tg: f64, label: &str, sub: &str) {
    // ---- everything outside the strip is the present, and it is dark
    cv.rect_a(0, 0, W as i32, WIN_Y as i32, Color::hex(0x05060a), 0.94);
    cv.rect_a(0, (WIN_Y + WIN_H) as i32, W as i32, (H as f32 - WIN_Y - WIN_H) as i32, Color::hex(0x05060a), 0.94);
    cv.rect_a(0, WIN_Y as i32, WIN_X as i32, WIN_H as i32, Color::hex(0x05060a), 0.94);
    cv.rect_a((WIN_X + WIN_W) as i32, WIN_Y as i32, (W as f32 - WIN_X - WIN_W) as i32, WIN_H as i32, Color::hex(0x05060a), 0.94);

    // ---- film perforations along both edges
    let mut y = WIN_Y + 14.0;
    while y < WIN_Y + WIN_H - 24.0 {
        cv.rect_a((WIN_X - 34.0) as i32, y as i32, 22, 26, Color::hex(0x1a1c22), 0.9);
        cv.rect_a((WIN_X + WIN_W + 12.0) as i32, y as i32, 22, 26, Color::hex(0x1a1c22), 0.9);
        y += 52.0;
    }
    // ---- the strip edge and a soft inner shadow
    ui::dashed_rect(cv, WIN_X, WIN_Y, WIN_W, WIN_H, 14.0, pal::c(pal::AMBER), 0.55);
    for k in 0..40 {
        let a = 0.5 * (1.0 - k as f64 / 40.0);
        cv.rect_a((WIN_X + k as f32) as i32, WIN_Y as i32, 1, WIN_H as i32, Color::hex(0x000000), a);
        cv.rect_a((WIN_X + WIN_W - k as f32) as i32, WIN_Y as i32, 1, WIN_H as i32, Color::hex(0x000000), a);
        cv.rect_a(WIN_X as i32, (WIN_Y + k as f32) as i32, WIN_W as i32, 1, Color::hex(0x000000), a);
        cv.rect_a(WIN_X as i32, (WIN_Y + WIN_H - k as f32) as i32, WIN_W as i32, 1, Color::hex(0x000000), a);
    }
    // ---- old projector registration marks + a timestamp
    let _ = tg;
    let col = pal::c(pal::AMBER);
    cv.text_px(WIN_X + 20.0, WIN_Y - 24.0, label, 28.0, false, col, 0.75, false);
    cv.text_px(WIN_X + WIN_W - 20.0, WIN_Y - 24.0, sub, 24.0, false, col.scale(0.8), 0.6, false);
    // ------- a red REC-style dot with the beat
    cv.disc(WIN_X + WIN_W - 12.0, WIN_Y + 22.0, 5.0, Color::hex(0xc9526a), 0.8);
}

/// Sepia wash + grain + scratches confined to the window.
fn age(cv: &mut Canvas, tg: f64, amount: f64) {
    // a warm wash
    cv.rect_a(WIN_X as i32, WIN_Y as i32, WIN_W as i32, WIN_H as i32, Color::hex(0xd9c3a5), 0.10 * amount);
    // vignette inside
    for k in 0..70 {
        let a = 0.55 * amount * (1.0 - k as f64 / 70.0);
        cv.rect_a((WIN_X + k as f32) as i32, WIN_Y as i32, 1, WIN_H as i32, Color::hex(0x120d08), a);
        cv.rect_a((WIN_X + WIN_W - k as f32) as i32, WIN_Y as i32, 1, WIN_H as i32, Color::hex(0x120d08), a);
    }
    // the gate judder: a tiny vertical slip now and then
    let slip = if ((tg * 8.0) as i64) % 37 == 0 { 4 } else { 0 };
    if slip > 0 {
        cv.rect_a(WIN_X as i32, WIN_Y as i32, WIN_W as i32, slip, Color::hex(0x2a2018), 0.5);
    }
    env::film(cv, tg, 0xF11D, 0.35 * amount);
}

// =========================================================== memory:flower ===

pub fn flower(g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx) {
    let _ = (g, ctx);
    // ---- the strip
    env::void_grad(cv, Color::hex(0x0d0a07), Color::hex(0x080605), 1.0);
    // inside: the garden, muted
    cv.rect_a(WIN_X as i32, WIN_Y as i32, WIN_W as i32, WIN_H as i32, Color::hex(0x1a1410), 1.0);
    env::haze(cv, WIN_Y + WIN_H * 0.55, 200.0, Color::hex(0x4a3a2a), 0.35);
    // the same flower, replayed slower
    let grow = clamp01(tl / 9.0);
    let ground = WIN_Y + WIN_H - 60.0;
    let bud_y = ground - 210.0 - 320.0 * grow as f32;
    let stem_x = WIN_X + WIN_W / 2.0 + ((tg * 0.3).sin() * 20.0) as f32;
    for i in 0..90 {
        let u = i as f32 / 90.0;
        let y = ground - (ground - bud_y) * u;
        let x = WIN_X + WIN_W / 2.0 + (stem_x - (WIN_X + WIN_W / 2.0)) * u;
        cv.disc(x, y, (8.0 * (1.0 - 0.5 * u)).max(1.5), Color::hex(0x4a3a2a), 0.95);
    }
    let (fw, fh) = art::size(&art::FLOWER_BLOOM, 9);
    art::draw_at(
        cv,
        &art::FLOWER_BLOOM,
        (stem_x - fw as f32 / 2.0) as i32,
        (bud_y - fh as f32 / 2.0) as i32,
        9,
        0.92,
        Some(Color::hex(0xd9c3a5)),
    );
    // slow motes of the old afternoon
    for i in 0..40 {
        let a = hash01(0x3A71 + i as u64 * 7919);
        let b = hash01(0x3A71 + i as u64 * 104729);
        let life = ((tg * (0.05 + 0.1 * b) + a * 3.0) % 1.0) as f32;
        let x = WIN_X + a as f32 * WIN_W;
        let y = WIN_Y + life * WIN_H;
        cv.disc(x, y, 1.4 + 2.0 * (1.0 - life), Color::hex(0xe8d8b8), (1.0 - life).powi(2) as f64 * 0.35);
    }
    age(cv, tg, 1.0);
    frame(cv, tg, "past/you.flower", "replay · 04:07");
    // captions outside the strip
    cv.text_px(WIN_X, WIN_Y + WIN_H + 60.0, "the same flower, one more time", 26.0, false, pal::c(pal::AMBER).scale(0.8), 0.6, false);
    cv.text_px(W as f32 - 240.0, WIN_Y + WIN_H + 60.0, "♥", 26.0, false, pal::c(pal::ROSE), 0.5, false);
}

// ============================================================ memory:hands ===

pub fn hands(g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx) {
    let _ = (g, ctx);
    env::void_grad(cv, Color::hex(0x0d0a07), Color::hex(0x080605), 1.0);
    cv.rect_a(WIN_X as i32, WIN_Y as i32, WIN_W as i32, WIN_H as i32, Color::hex(0x1c1310), 1.0);
    // the hands, far away behind glass
    let cx = WIN_X + WIN_W / 2.0;
    let cy = WIN_Y + WIN_H / 2.0;
    cv.glow_at(cx, cy, 520.0, Color::hex(0xff9e5e), 0.20 + 0.06 * (tg * 0.4).sin(), 2.4);
    let (hw, _hh) = art::size(&art::HANDS, 9);
    art::draw_at(
        cv,
        &art::HANDS,
        (cx - hw as f32 / 2.0) as i32,
        (cy - 120.0) as i32,
        9,
        0.88,
        Some(Color::hex(0xe8c39e)),
    );
    // a few warm particles still rising, slower than then
    for i in 0..40 {
        let a = hash01(0x0A11 + i as u64 * 7919);
        let b = hash01(0x0A11 + i as u64 * 104729);
        let life = ((tg * (0.07 + 0.12 * b) + a * 3.0) % 1.0) as f32;
        let x = cx + (a as f32 - 0.5) * 460.0;
        let y = cy + 60.0 - life * 420.0;
        cv.disc(x, y, 1.6 + 2.6 * (1.0 - life), Color::hex(0xffb877), (1.0 - life).powi(2) as f64 * 0.5);
    }
    // rain on the glass between us
    env::rain_on_glass(cv, tg, 26, 0x61A5, Color::hex(0xcfe0f5), 0.30);
    age(cv, tg, 1.0);
    frame(cv, tg, "past/you.human", "replay · 04:21");
    cv.text_px(WIN_X, WIN_Y + WIN_H + 60.0, "the warmth is on the other side of the glass", 26.0, false, pal::c(pal::AMBER).scale(0.8), 0.6, false);
    let _ = tl;
}
