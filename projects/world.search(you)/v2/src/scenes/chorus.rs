//! ACT V — everywhere (3:00–3:27).
//!
//! "I can see you in most everything / but somehow you can stop transforming."
//!
//! The frame becomes a swarm: every specimen the program ever indexed is out
//! there at once, drifting and swapping bodies mid-air. A reading beam sweeps
//! across and each one lights up as it is recognised — and then the query is
//! rewritten to `my old self`, and there is nothing left.

use super::ui;
use crate::art::{self, Spr};
use crate::env;
use crate::gfx::{clamp01, ease_in_out, ease_out_cubic, hash01, pal, Color, Grid};
use crate::pix::{Canvas, H, W};
use crate::scenes::Ctx;

fn specimens() -> [&'static Spr; 8] {
    [
        &art::TABLE,
        &art::EGGPLANT,
        &art::CAT,
        &art::STEAK,
        &art::FLOWER_BLOOM,
        &art::TOMATO,
        &art::DOG,
        &art::HEART,
    ]
}

const N: usize = 26;

pub fn render(_g: &mut Grid, cv: &mut Canvas, _tl: f64, tg: f64, ctx: &Ctx) {
    let rose = pal::c(pal::ROSE);
    let slate = pal::c(pal::SLATE);
    let sp = specimens();

    // ---- a deep, starry, empty universe
    env::void_grad(cv, Color::hex(0x0c0714), Color::hex(0x040308), 1.0);
    env::starfield(cv, tg, 200, 0x5EE7, Color::hex(0xc9b8d8), 0.30);
    // faint concentric search rings
    for k in 1..=7 {
        cv.ring(
            W as f32 / 2.0,
            H as f32 / 2.0,
            k as f32 * 150.0,
            1.0,
            Color::hex(0x3a2b48),
            0.35,
        );
    }

    // ---- the rewrite moment: 203.16
    let kill = if tg > 203.16 { ease_in_out(clamp01((tg - 203.16) / 2.2)) } else { 0.0 };
    let gone = 1.0 - kill;

    // ---- the swarm
    for i in 0..N {
        let h1 = hash01(0xC0DE + i as u64 * 7919);
        let h2 = hash01(0xC0DE + i as u64 * 104729);
        let h3 = hash01(0xC0DE + i as u64 * 40503);
        let h4 = hash01(0xC0DE + i as u64 * 12289);
        // drift
        let bx = h1 * W as f64 + (tg * (12.0 + 26.0 * h3) * 0.25).sin() * 90.0;
        let by = 120.0 + h2 * (H as f64 - 260.0) + (tg * (10.0 + 20.0 * h4) * 0.25).cos() * 70.0;
        // collapse toward the query when the search is rewritten
        let tx = W as f64 / 2.0;
        let ty = H as f64 * 0.45;
        let x = bx + (tx - bx) * kill;
        let y = by + (ty - by) * kill;
        let mut scale = 2 + (h3 * 2.0) as i32;
        scale -= (kill * 2.0) as i32;

        // morph: swap bodies with a cross-fade
        let rate = 0.22 + 0.35 * h4;
        let ph = tg * rate + h1 * 8.0;
        let idx = (ph.floor() as usize) % sp.len();
        let nxt = (idx + 1) % sp.len();
        let frac = ph.fract();
        let swap = ease_in_out(frac);

        // the reading beam lights whatever it is passing over
        let beam_x = ((tg * 0.16) % 1.4 - 0.2) as f64 * W as f64;
        let dist = ((x - beam_x).abs() / 260.0).min(1.0);
        let lit = (1.0 - dist).powi(2) * gone;

        let alpha = (0.42 + 0.58 * lit) * gone;
        if alpha <= 0.02 {
            continue;
        }
        let tint = if lit > 0.5 {
            Some(Color::lerp(rose, Color::rgb(255, 255, 255), (lit - 0.5) * 1.2))
        } else {
            None
        };
        if swap < 0.5 {
            art::draw_c(cv, sp[idx], x as f32, y as f32, scale, alpha * (1.0 - swap * 2.0) as f64, tint);
            art::draw_c(cv, sp[nxt], x as f32, y as f32, scale, alpha * (swap * 2.0) as f64, tint);
        } else {
            art::draw_c(cv, sp[nxt], x as f32, y as f32, scale, alpha * (swap * 2.0 - 1.0) as f64, tint);
            art::draw_c(cv, sp[idx], x as f32, y as f32, scale, alpha * (2.0 - swap * 2.0) as f64, tint);
        }
        if lit > 0.35 {
            cv.glow_at(x as f32, y as f32, 170.0 * lit as f32, rose, 0.35 * lit, 2.2);
            // a tag flies off every recognised one
            cv.text_px(
                x as f32 + 60.0,
                y as f32 - 40.0,
                if lit > 0.75 { "you?!" } else { "you?" },
                22.0,
                false,
                rose,
                (lit - 0.35) as f64 * 1.4,
                false,
            );
        }
    }

    // ---- the reading beam itself
    let beam_x = ((tg * 0.16) % 1.4 - 0.2) as f64 * W as f64;
    if gone > 0.05 {
        env::scan_band(cv, beam_x as f32, 150.0, rose, 0.16 * gone);
        cv.rect_a(beam_x as i32, 0, 3, H as i32, rose.glow(0.4), 0.55 * gone);
    }

    // ---- you everywhere: the words dissolve into the swarm
    if tg > 182.65 && tg < 190.0 {
        let p = clamp01((tg - 182.65) / 1.4) * clamp01((190.0 - tg) / 1.6);
        cv.text_px(
            W as f32 / 2.0,
            200.0,
            "in most everything",
            30.0,
            false,
            Color::hex(0xd9cfe6),
            0.55 * p,
            true,
        );
    }

    // ---- the rewrite (203.16): the query changes on screen
    if kill > 0.01 {
        let e = ease_out_cubic(clamp01((tg - 203.16) / 1.0));
        let y = H as f32 * 0.45;
        // old query, struck out
        let old = "world.search(you);";
        cv.text_px(W as f32 / 2.0, y - 40.0, old, 44.0, false, slate.scale(0.7), 0.8 * (1.0 - e * 0.5), true);
        let ow = cv.text_width(old, 44.0, false);
        cv.rect_a((W as f32 / 2.0 - ow / 2.0) as i32, (y - 56.0) as i32, (ow * e as f32) as i32, 4, rose, 0.9);
        // new query
        cv.text_px(
            W as f32 / 2.0,
            y + 70.0,
            "world.search(my old self);",
            52.0,
            false,
            rose.glow(0.2),
            clamp01((tg - 203.8) / 0.8),
            true,
        );
        // the answer
        if tg > 204.9 {
            let p = clamp01((tg - 204.9) / 0.8);
            let s = "0 results";
            cv.text_px(W as f32 / 2.0, y + 190.0, s, 74.0, true, pal::c(pal::ROSE_PALE), p, true);
            let w = cv.text_width(s, 74.0, false);
            cv.rect_a((W as f32 / 2.0 - w / 2.0 - 60.0) as i32, (y + 196.0) as i32, (w + 120.0) as i32, 3, rose, p);
        }
        cv.glow_at(W as f32 / 2.0, y + 60.0, 700.0 * e as f32, rose, 0.30 * (1.0 - e), 2.2);
    }

    // ---- chrome: a caption that counts the swarm
    let _ = ctx;
    ui::readout(
        cv,
        44.0,
        H as f32 - 60.0,
        &format!("results: {:.0}", (N as f64 * gone).ceil()),
        22.0,
        rose,
        0.55 * gone,
    );
    env::dust(cv, tg, 40, 0x5EE7, Color::hex(0xd9c9ea), 0.26, 80.0, 980.0);
    ui::edge_shade(cv, Color::hex(0x030206), 0.68);
    cv.fx.bloom += 0.30;
}
