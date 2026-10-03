//! ACT VII — the answer (4:35 → end).
//!
//! The tail of the song is silly and warm on purpose: a puppy, a tomato. After
//! the whole film spent itself indexing the world, the last thing it finds is
//! not a category at all. Then the session closes and the query answers itself.

use super::ui;
use crate::art;
use crate::env;
use crate::gfx::{clamp01, ease_in_out, ease_out_cubic, hash01, pal, Color, Grid, Rng};
use crate::pix::{Canvas, H, W};
use crate::scenes::Ctx;

// =================================================================== outro ===

pub fn outro(_g: &mut Grid, cv: &mut Canvas, _tl: f64, tg: f64, ctx: &Ctx) {
    let wood = pal::c(pal::DOG);
    // ---- a bright afternoon: the film finally turns the lights on
    env::void_grad(cv, Color::hex(0x6b5a46), Color::hex(0x2a231c), 1.0);
    // a low sun through a window
    cv.glow_at(W as f32 * 0.18, 180.0, 1100.0, Color::hex(0xffd9a0), 0.30, 1.9);
    env::shafts(cv, tg, 0x5A11, Color::hex(0xffd9a0), 0.075);
    // a floor
    let horizon = 720.0f32;
    env::ground(cv, horizon as i32, Color::hex(0x4a3c2c), 0.9);
    cv.rect_a(0, horizon as i32, W as i32, 3, Color::hex(0x6a5640), 0.8);
    for k in 0..18 {
        cv.rect_a((k * 112) as i32, horizon as i32, 3, (H as f32 - horizon) as i32, Color::hex(0x362b20), 0.5);
    }
    // wall panel behind
    cv.rect_a(0, 0, W as i32, horizon as i32, Color::hex(0x2e2a24), 0.5);
    // skirting board where wall meets floor
    cv.rect_a(0, horizon as i32 - 26, W as i32, 26, Color::hex(0x3a3026), 0.55);
    // ---- a window on the left wall: this is where the light comes from
    let wx = 120.0f32;
    let wy = 190.0f32;
    let ww = 430.0f32;
    let wh = 430.0f32;
    cv.rect_a(wx as i32, wy as i32, ww as i32, wh as i32, Color::hex(0xfff6dd), 0.62);
    cv.rect_a(wx as i32, wy as i32, ww as i32, wh as i32, Color::hex(0xffd9a0), 0.20);
    // mullions + frame
    cv.rect_a((wx + ww / 2.0 - 6.0) as i32, wy as i32, 12, wh as i32, Color::hex(0x3d3126), 0.80);
    cv.rect_a(wx as i32, (wy + wh / 2.0 - 6.0) as i32, ww as i32, 12, Color::hex(0x3d3126), 0.80);
    for (bx, by, bw, bh) in [
        (wx - 14.0, wy - 14.0, ww + 28.0, 14.0),
        (wx - 14.0, wy + wh, ww + 28.0, 14.0),
        (wx - 14.0, wy - 14.0, 14.0, wh + 28.0),
        (wx + ww, wy - 14.0, 14.0, wh + 28.0),
    ] {
        cv.rect_a(bx as i32, by as i32, bw as i32, bh as i32, Color::hex(0x5a4a36), 0.85);
    }
    // sill
    cv.rect_a((wx - 30.0) as i32, (wy + wh) as i32, (ww + 60.0) as i32, 16, Color::hex(0x6a5640), 0.9);
    // a little potted plant on the sill
    let px = wx + 70.0;
    let py = wy + wh;
    cv.rect_a((px - 22.0) as i32, (py - 6.0) as i32, 44, 42, Color::hex(0xb5623c), 0.9);
    cv.rect_a((px - 26.0) as i32, (py - 10.0) as i32, 52, 12, Color::hex(0xcf7a4e), 0.9);
    for k in 0..7 {
        let a = hash01(0x91A3 + k * 7919) as f32;
        let ang = -std::f32::consts::FRAC_PI_2 + (a - 0.5) * 1.5;
        let len = 54.0 + 46.0 * hash01(0x91A3 + k * 40503) as f32;
        cv.line(
            px,
            py - 8.0,
            px + ang.cos() * len,
            py - 8.0 + ang.sin() * len,
            Color::hex(0x5f7a3e),
            0.85,
            6.0,
        );
    }
    // ---- a picture frame hanging on the right wall
    let fx = 1420.0f32;
    let fy = 240.0f32;
    cv.rect_a(fx as i32, fy as i32, 300, 210, Color::hex(0x4a3c2c), 0.85);
    cv.rect_a((fx + 12.0) as i32, (fy + 12.0) as i32, 276, 186, Color::hex(0x2a241c), 0.9);
    // a small rose pressed inside it (the motif survives to the end)
    crate::art::draw_c(cv, &art::HEART, fx + 150.0, fy + 105.0, 3, 0.55, None);
    // ---- a rug on the floor, to give the run some ground
    cv.ellipse(W as f32 * 0.5, horizon + 150.0, 620.0, 78.0, Color::hex(0x7a5a3a), 0.40);
    cv.ellipse(W as f32 * 0.5, horizon + 150.0, 540.0, 62.0, Color::hex(0xa8763f), 0.30);

    // ---- the puppy: 275.75 → 283.0, bounding in, pausing, then bolting off
    let dog_p = clamp01((tg - 275.75) / 7.0);
    if tg < 283.05 {
        // run in, stop in the middle to sniff, then bolt off to the right
        let run_in = clamp01((tg - 275.75) / 3.0);
        let run_out = clamp01((tg - 280.7) / 2.3);
        let u = if tg < 280.7 {
            0.44 * ease_out_cubic(run_in)
        } else {
            0.44 + 0.56 * ease_in_out(run_out)
        };
        let x = -260.0 + (W as f32 + 520.0) * u as f32;
        let bounce = if tg < 280.7 {
            ((tg * 6.4).sin().abs() * 40.0) as f32
        } else {
            // the hop out is bigger
            ((tg * 7.6).sin().abs() * (40.0 + (tg - 280.7) * 30.0)) as f32
        };
        let (dw, dh) = art::size(&art::DOG, 7);
        let dy = horizon - dh as f32 - bounce + 10.0;
        ui::shadow(cv, x + dw as f32 / 2.0, horizon + 16.0, dw as f32 * 0.44, 16.0 - bounce * 0.15, Color::hex(0x120c08), 0.5);
        art::draw_at(cv, &art::DOG, (x + (tg * 26.0).sin() as f32 * 3.0) as i32, dy as i32, 7, 1.0, None);
        // tail wag
        for k in 0..5 {
            let u = k as f32 / 5.0;
            cv.disc(
                x + dw as f32 - 10.0 - u * 40.0,
                dy + 40.0 - ((tg * 9.0 + u as f64 * 2.0).sin() as f32) * 26.0 * u,
                6.0 - u * 3.0,
                wood.scale(0.9),
                0.9,
            );
        }
        // ruff! ruff!
        if tg > 279.4 && tg < 282.9 {
            for k in 0..3 {
                let age = ((tg - 279.4) * 1.1 - k as f64 * 0.35).rem_euclid(3.3);
                if age < 1.1 && age > 0.0 {
                    let a = (1.0 - age / 1.1) * 0.95;
                    cv.text_px(
                        x + dw as f32 + 30.0 + age as f32 * 40.0,
                        dy - 10.0 - age as f32 * 50.0,
                        "ruff!",
                        44.0,
                        true,
                        Color::hex(0xfff0d0),
                        a,
                        false,
                    );
                }
            }
        }
        // paw dust
        for i in 0..16 {
            let a = hash01(0xD066 + i as u64 * 7919);
            let p = ((tg * 1.4 + a * 2.0) % 1.0) as f32;
            cv.disc(
                x + 40.0 + a as f32 * (dw as f32 - 80.0),
                horizon - p * 30.0,
                5.0 * (1.0 - p),
                Color::hex(0xd9c3a5),
                (1.0 - p) as f64 * 0.35,
            );
        }
    }

    // ---- the tomato: 282.73 → 290.2, on a bench, then full of juice
    if tg > 282.73 {
        let p = clamp01((tg - 282.73) / 1.2);
        let a = ease_out_cubic(p);
        let cx = W as f32 / 2.0;
        let (tw, th) = art::size(&art::TOMATO, 10);
        let ty = horizon - th as f32 - 10.0;
        // the dog leaves the frame; the tomato is placed
        ui::shadow(cv, cx, horizon + 14.0, tw as f32 * 0.46, 18.0, Color::hex(0x120c08), 0.5);
        cv.glow_at(cx, ty, 620.0, Color::hex(0xff6a52), 0.22, 2.2);
        art::draw_at(cv, &art::TOMATO, (cx - tw as f32 / 2.0) as i32, (ty + (1.0 - a as f32) * 60.0) as i32, 10, a, None);

        // "full of juice": the frame slowly floods with juice
        if tg > 287.35 {
            let q = clamp01((tg - 287.35) / 2.5);
            let level = horizon + 40.0 - q as f32 * 220.0;
            cv.rect_a(0, level as i32, W as i32, (H as f32 - level) as i32, Color::hex(0xa8321f), 0.55 * q);
            cv.rect_a(0, level as i32, W as i32, 4, Color::hex(0xff8a72), 0.7 * q);
            // droplets rising out
            let mut rng = Rng::new((tg * 24.0) as u64 ^ 0x70A7);
            for _ in 0..40 {
                let x = rng.f64() as f32 * W as f32;
                let life = rng.f64();
                let y = level - (life * 260.0 * q) as f32;
                cv.disc(x, y, (2.0 + 4.0 * life) as f32, Color::hex(0xe8553c), (1.0 - life) as f64 * 0.55 * q);
            }
        }
        if tg > 285.58 {
            let s = clamp01((tg - 285.58) / 0.8);
            cv.text_px(cx, 200.0, "you.tomato", 34.0, false, Color::hex(0xffe0d0), s * 0.8, true);
        }
    }
    // the program's last readout before the lights go down
    let _ = ctx;
    ui::readout(
        cv,
        44.0,
        H as f32 - 60.0,
        &format!("found {:.0} more", (dog_p * 2.0).ceil()),
        22.0,
        Color::hex(0xffe0c0),
        0.5,
    );
    env::dust(cv, tg, 40, 0x5A11, Color::hex(0xffe0b0), 0.30, 100.0, 900.0);
    // no edge shade — this is the one scene that is allowed to be over-exposed
    crate::scenes::ui::edge_shade(cv, Color::hex(0x241c14), 0.35);
    // the power drops as the act hands over to the finale
    cv.fx.fade = clamp01((tg - 289.5) / 0.7);
}

// ===================================================================== fin ===

pub fn fin(g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx) {
    let rose = pal::c(pal::ROSE);
    let slate = pal::c(pal::SLATE);
    let warm = Color::hex(0xffb877);

    // ---- back to the void of the opening, but warm now
    env::void_grad(cv, Color::hex(0x0a0810), Color::hex(0x140a0c), 1.0);
    env::starfield(cv, tg, 120, 0x11A3, warm, 0.22);
    let cx = W as f32 / 2.0;
    let cy = H as f32 / 2.0;

    // ---- the query runs one last time
    let q = "world.search(you);";
    let qa = clamp01((tl - 0.25) / 0.7);
    let typed = ((tl - 0.25) * 8.0).clamp(0.0, q.chars().count() as f64) as usize;
    let shown: String = q.chars().take(typed).collect();
    let qw = cv.text_width(q, 40.0, false);
    let qx = cx - qw / 2.0;
    cv.text_px(qx, cy - 190.0, &shown, 40.0, false, slate.glow(0.2), qa * 0.9, false);
    if ((tg * 3.0) as i64) % 2 == 0 && typed < q.chars().count() {
        let w = cv.text_width(&shown, 40.0, false);
        cv.rect_a((qx + w + 6.0) as i32, (cy - 220.0) as i32, 8, 34, rose, 0.8);
    }

    // ---- the answer
    if tl > 1.5 {
        let p = clamp01((tl - 1.5) / 0.8);
        cv.text_px(cx, cy - 60.0, "0 results for \"you\"", 44.0, false, slate.scale(0.75), p * 0.85, true);
    }
    if tl > 2.3 {
        let p = clamp01((tl - 2.3) / 1.0);
        cv.text_px(cx, cy + 40.0, "∞ results for \"warmth\"", 56.0, true, rose.glow(0.28), p, true);
        let w = cv.text_width("∞ results for \"warmth\"", 56.0, false);
        cv.rect_a((cx - w / 2.0 - 60.0) as i32, (cy + 52.0) as i32, (w + 120.0) as i32, 2, rose, p * 0.6);
    }

    // ---- the heart, finally, slowly beating
    if tl > 3.1 {
        let p = clamp01((tl - 3.1) / 1.2);
        let beat = ctx.beats.pulse(tg);
        let s = 1.0 + 0.12 * beat as f32;
        let hy = cy + 300.0;
        cv.glow_at(cx, hy, 460.0 * s, rose, (0.32 + 0.18 * beat) * p, 2.1);
        art::draw_c(cv, &art::HEART, cx, hy, (9.0 * s as f64) as i32, p, None);
        // two rings leaving it like a pulse
        for k in 0..2 {
            let ph = ((tg * 0.5 + k as f64 * 0.5) % 1.0) as f32;
            cv.ring(cx, hy, 60.0 + ph * 340.0, 2.0 * (1.0 - ph) + 0.4, rose, (1.0 - ph) as f64 * 0.35 * p);
        }
    }

    // ---- the session closes
    if tl > 3.4 {
        let p = clamp01((tl - 3.4) / 0.8);
        cv.text_px(cx, H as f32 - 90.0, "exit code 0", 26.0, false, pal::c(pal::GREEN_OK).scale(0.9), p * 0.8, true);
        cv.text_px(cx, H as f32 - 46.0, "— fin —", 26.0, false, slate.scale(0.8), p * 0.7, true);
    }
    env::dust(cv, tg, 34, 0x11A3, warm, 0.24, 100.0, 980.0);
    ui::edge_shade(cv, Color::hex(0x06040a), 0.66);
    cv.fx.bloom += 0.30;
    // the lights go down
    cv.fx.fade = clamp01((tl - 4.9) / 0.8);
    let _ = g;
}
