//! ACT III — bloom (1:50–2:17). Two closes: a flower that grows the whole
//! height of the frame, and two hands held far too close to the lens.
//!
//! Both are deliberately warm, centred and quiet — the film stops indexing for
//! a moment and just looks at something.

use super::ui;
use crate::art;
use crate::env;
use crate::gfx::{clamp01, ease_in_out, ease_out_cubic, hash01, pal, Color, Grid, Rng};
use crate::pix::{Canvas, H, W};
use crate::scenes::Ctx;

const CX: f32 = W as f32 / 2.0;

/// A quadratic curve through three points, drawn as a tapering stem.
fn stem(cv: &mut Canvas, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, color: Color, alpha: f64, w0: f32) {
    let n = 90;
    for i in 0..=n {
        let u = i as f32 / n as f32;
        let iu = 1.0 - u;
        let x = iu * iu * x0 + 2.0 * iu * u * x1 + u * u * x2;
        let y = iu * iu * y0 + 2.0 * iu * u * y1 + u * u * y2;
        let w = w0 * (1.0 - 0.55 * u);
        cv.disc(x, y, w.max(1.0), color, alpha);
    }
}

// ================================================================= flower ===

pub fn flower(g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx) {
    let accent = pal::c(pal::PINK);
    // ---- a garden at dusk: graded sky, grass along the very bottom
    env::void_grad(cv, Color::hex(0x1a1526), Color::hex(0x2a1a22), 1.0);
    env::haze(cv, 900.0, 260.0, Color::hex(0x3a2434), 0.35);
    // bokeh
    for i in 0..26 {
        let a = hash01(0xF10A + i as u64 * 7919);
        let b = hash01(0xF10A + i as u64 * 104729);
        let x = a * W as f64;
        let y = b * H as f64 * 0.8;
        let r = 10.0 + 46.0 * hash01(0xF10A + i as u64 * 40503);
        let tw = 0.5 + 0.5 * (tg * 0.5 + a * 8.0).sin();
        cv.disc(x as f32, y as f32, r as f32, Color::hex(0xffd8e6), 0.035 * tw);
    }
    // grass
    for i in 0..90 {
        let a = hash01(0x69A5 + i as u64 * 2654435761);
        let x = a * W as f64;
        let h = 60.0 + 190.0 * hash01(0x69A5 + i as u64 * 40503);
        let sway = ((tg * 0.8 + a * 5.0).sin() * 26.0) as f32;
        cv.line(
            x as f32,
            H as f32,
            x as f32 + sway,
            (H as f64 - h) as f32,
            Color::hex(0x2c3f2c),
            0.55,
            3.0,
        );
    }

    // ---- growth: 0 → 1 across the section, stalling at "occasionally"
    let grow = ease_out_cubic(clamp01(tl / 9.0));
    // "even if you bloom just occasionally" (116.60): open/close pulse
    let occasionally = if tg > 116.60 && tg < 119.9 {
        let p = clamp01((tg - 116.60) / 0.9);
        let osc = (0.5 + 0.5 * ((tg - 116.60) * 1.9).sin()) as f64;
        1.0 - p * 0.75 + p * 0.75 * osc
    } else {
        1.0
    };
    let open = (grow * occasionally).clamp(0.0, 1.0);

    let ground = H as f32 - 60.0;
    let bud_y = ground - 210.0 - 430.0 * grow as f32;
    let stem_x = CX + ((tg * 0.55).sin() * 26.0) as f32;

    // stem + leaves
    stem(
        cv,
        CX,
        ground,
        CX + 40.0 * (1.0 - grow as f32),
        ground - 240.0,
        stem_x,
        bud_y + 40.0,
        Color::hex(0x4a7c48),
        0.95,
        11.0,
    );
    let leaf = |cv: &mut Canvas, at: f32, dir: f32| {
        let y = ground - at * (ground - bud_y);
        let x = CX + (stem_x - CX) * (1.0 - at);
        let len = 150.0 * (1.0 - at * 0.6);
        for k in 0..40 {
            let u = k as f32 / 40.0;
            let lx = x + dir * u * len;
            let ly = y - (u * u * 60.0) + u * 30.0;
            cv.disc(lx, ly, (16.0 * (1.0 - u * 0.7)).max(2.0), Color::hex(0x4f8a4c), 0.85);
        }
    };
    leaf(cv, 0.30, -1.0);
    leaf(cv, 0.52, 1.0);

    // the bloom itself
    let spr = if open < 0.5 { &art::FLOWER_BUD } else { &art::FLOWER_BLOOM };
    let (fw, fh) = art::size(spr, 13);
    let scale = if open < 0.5 { 13 } else { 13 + (open * 4.0) as i32 };
    let (fw2, fh2) = ((fw as f32 * scale as f32 / 13.0) as i32, (fh as f32 * scale as f32 / 13.0) as i32);
    cv.glow_at(stem_x, bud_y, 300.0, accent, 0.30 * open, 2.3);
    art::draw_at(cv, spr, (stem_x - fw2 as f32 / 2.0) as i32, (bud_y - fh2 as f32 / 2.0) as i32, scale, 1.0, None);

    // ---- petals falling once it is open
    if open > 0.8 && tg > 118.0 {
        for i in 0..28 {
            let a = hash01(0x9E41 + i as u64 * 7919);
            let b = hash01(0x9E41 + i as u64 * 104729);
            let life = ((tg * (0.25 + 0.3 * b) + a * 4.0) % 1.0) as f32;
            let y = 200.0 + life * (H as f32 - 200.0);
            let x = (a * W as f64) as f32 + ((life * 30.0).sin() * 90.0);
            let rot = life * 6.0;
            let pw = 16.0 + 10.0 * b as f32;
            // a petal as a small rotated lozenge
            for k in 0..10 {
                let u = k as f32 / 10.0 - 0.5;
                cv.disc(
                    x + (rot + u * 0.6).sin() * pw * 0.5,
                    y + u * pw * 1.6,
                    pw * 0.32,
                    accent,
                    (1.0 - life) as f64 * 0.75,
                );
            }
        }
    }

    // ---- occasionally: closed petals drifting away as the frame dims
    if occasionally < 0.98 {
        let a = (1.0 - occasionally) as f64;
        cv.rect_a(0, 0, W as i32, H as i32, Color::hex(0x0a0610), 0.35 * a);
        cv.text_px(CX, 300.0, "(sometimes)", 27.0, false, pal::c(pal::PINK).scale(0.7), a * 0.8, true);
    }

    // ---- the reticle treats it as a specimen anyway
    ui::reticle(cv, stem_x, bud_y, 300.0, 0.35, tg * 0.25, accent, 0.14, 0.35);
    if tg > 119.96 {
        ui::stamp(cv, g, CX, 330.0, (tg - 119.96) / 1.1, pal::c(pal::GOLD));
    }
    // ---- a slim side caption instead of a panel
    ui::readout(cv, 46.0, H as f32 - 120.0, "you.flower", 30.0, accent, 0.65);
    ui::readout(
        cv,
        46.0,
        H as f32 - 80.0,
        &format!("bloom {:>3.0}%", open * 100.0),
        22.0,
        accent.scale(0.8),
        0.5,
    );
    env::dust(cv, tg, 30, 0xF10A, Color::hex(0xffd8e6), 0.30, 120.0, H as f32 - 80.0);
    ui::edge_shade(cv, Color::hex(0x120a14), 0.62);
    let _ = ctx;
}

// ================================================================== hands ===

pub fn hands(g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx) {
    let accent = pal::c(pal::SKIN);
    // ---- a warm, almost-black room; the only light is the hands
    env::void_grad(cv, Color::hex(0x1a1108), Color::hex(0x080503), 1.0);
    let cx = CX;
    let cy = H as f32 * 0.54;

    // breathing glow behind them
    let breath = 0.5 + 0.5 * (tg * 0.35).sin();
    cv.glow_at(cx, cy, 620.0, Color::hex(0xff9e5e), 0.22 + 0.10 * breath, 2.4);
    cv.glow_at(cx, cy + 60.0, 380.0, Color::hex(0xffd0a0), 0.18, 2.6);

    // ---- the hands
    let (hw, hh) = art::size(&art::HANDS, 13);
    let breathe = ((tg * 0.35).sin() * 8.0) as f32;
    let hx = cx - hw as f32 / 2.0;
    let hy = cy - hh as f32 / 2.0 + breathe;
    art::draw_at(cv, &art::HANDS, hx as i32, hy as i32, 13, 1.0, None);
    // a second, brighter pass on the upper surfaces reads as the key light
    art::draw_at(cv, &art::HANDS, (hx - 8.0) as i32, (hy - 8.0) as i32, 13, 0.22, Some(Color::hex(0xffe0c0)));

    // ---- warmth particles streaming off the palms
    let heat = if tg > 126.82 { 1.0 } else { 0.45 };
    for i in 0..90 {
        let a = hash01(0x0A11 + i as u64 * 7919);
        let b = hash01(0x0A11 + i as u64 * 104729);
        let life = ((tg * (0.12 + 0.22 * b) + a * 3.0) % 1.0) as f32;
        let x = cx + (a as f32 - 0.5) * 620.0 + ((life * 5.0).sin() * 40.0);
        let y = cy + 40.0 - life * 460.0;
        let al = (1.0 - life).powi(2) as f64 * 0.8 * heat;
        cv.disc(x, y, 1.6 + 3.4 * (1.0 - life), Color::hex(0xffb877), al);
        if i % 9 == 0 {
            cv.glow_at(x, y, 40.0, Color::hex(0xff9e5e), al * 0.5, 2.2);
        }
    }

    // ---- "hair that's unruly" (130.24): strands start misbehaving
    if tg > 130.24 {
        let unruly = clamp01((tg - 130.24) / 1.5);
        let mut rng = Rng::new(0x11A1);
        for _ in 0..26 {
            let x0 = hx + 60.0 + rng.f64() as f32 * (hw as f32 - 120.0);
            let y0 = hy - 6.0;
            let lean = (rng.f64() as f32 - 0.5) * 120.0;
            let wig = ((tg * (2.0 + rng.f64())) as f32).sin() * 40.0 * unruly as f32;
            let col = Color::hex(0x3a2a20).glow(0.25);
            cv.line(x0, y0, x0 + lean * 0.5, y0 - 60.0, col, 0.55 * unruly, 5.0);
            cv.line(x0 + lean * 0.5, y0 - 60.0, x0 + lean + wig, y0 - 110.0, col, 0.5 * unruly, 4.0);
        }
        // and a puff of very ordinary breath drifting up-right
        let puff = clamp01((tg - 131.6) / 0.6) * clamp01((tg < 133.4) as u8 as f64 + clamp01((133.9 - tg) / 0.5));
        if puff > 0.02 {
            for i in 0..34 {
                let a = hash01(0x8B12 + i as u64 * 2654435761);
                let b = hash01(0x8B12 + i as u64 * 40503);
                let life = ((tg * 0.35 + a) % 1.0) as f32;
                let x = hx + hw as f32 * 0.62 + life * 420.0 + (a as f32 - 0.5) * 120.0;
                let y = hy + 20.0 - life * 260.0 + (b as f32 - 0.5) * 80.0;
                cv.disc(x, y, 16.0 + 30.0 * life, Color::hex(0xc9d2c4), (1.0 - life) as f64 * 0.16 * puff);
            }
            cv.text_px(hx + hw as f32 * 0.8, hy - 40.0, "…", 30.0, false, pal::c(pal::MID), puff * 0.7, false);
        }
    }

    // ---- PERFECT (133.65)
    if tg > 133.65 {
        ui::stamp(cv, g, cx, cy - 40.0, (tg - 133.65) / 1.1, pal::c(pal::GOLD));
    }

    // ---- the program speaks once, quietly, bottom-left
    if tl < 9.0 {
        ui::readout(cv, 46.0, H as f32 - 120.0, "you.human", 30.0, accent, 0.6);
        ui::readout(cv, 46.0, H as f32 - 80.0, "hands · 36.4°C", 22.0, accent.scale(0.8), 0.5);
    }
    // a heart drifting off the palms
    let hp = (tg % 4.0) / 4.0;
    crate::art::draw_c(
        cv,
        &art::HEART,
        cx + 240.0 + (hp as f32 * 20.0),
        cy - 120.0 - hp as f32 * 220.0,
        3,
        (1.0 - hp) as f64 * 0.5,
        None,
    );
    env::dust(cv, tg, 24, 0x0A11, Color::hex(0xffd0a0), 0.22, 120.0, 900.0);
    ui::edge_shade(cv, Color::hex(0x080503), 0.7);
    cv.fx.bloom += 0.30;
    let _ = (ease_in_out, ctx);
}
