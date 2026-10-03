//! ACT II-B — the four specimens (0:41–1:50). Each one is opened like a file
//! and shown in its own world, with its own composition:
//!
//!   table    a room, seen from the side; the camera dollies past it
//!   eggplant a chopping board along the bottom; metadata reads as a top strip
//!   cat      a night window; the subject is small and the room is huge
//!   steak    top-down in a skillet; the "doneness" arrives as an inset scan
//!
//! All four share exactly one thing: the reticle that locks onto them.

use super::ui;
use crate::art;
use crate::env;
use crate::gfx::{clamp01, ease_in_out, ease_out_cubic, pal, Color, Grid, Rng};
use crate::pix::{Canvas, H, W};
use crate::scenes::Ctx;

const CX: f32 = W as f32 / 2.0;

// ================================================================== table ===

pub fn table(g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx) {
    let accent = pal::c(pal::WOOD);
    // ---- the room: a warm interior with one window
    env::void_grad(cv, Color::hex(0x120f14), Color::hex(0x070610), 1.0);
    let horizon = 700.0f32;
    env::ground(cv, horizon as i32, Color::hex(0x1a1420), 0.85);
    cv.rect_a(0, horizon as i32, W as i32, 3, Color::hex(0x2a2030), 0.9);
    // floorboards
    for k in 0..22 {
        let y = horizon + 8.0 + k as f32 * 24.0;
        cv.rect_a(0, y as i32, W as i32, 1, Color::hex(0x120e16), 1.0 - k as f64 / 26.0);
    }
    // the window, and the light it throws
    ui::dashed_rect(cv, 120.0, 120.0, 300.0, 340.0, 10.0, Color::hex(0x3a4056), 0.5);
    cv.rect_a(120, 120, 300, 340, Color::hex(0x6f87b8), 0.10);
    cv.rect_a(266, 120, 8, 340, Color::hex(0x3a4056), 0.6);
    cv.rect_a(120, 284, 300, 8, Color::hex(0x3a4056), 0.6);
    env::shafts(cv, tg, 0x7AB1E, Color::hex(0x6f87b8), 0.055);

    // ---- the table: assembled over the first line, then alive
    let born = clamp01((tl - 0.15) / 1.5);
    let wobble_line = tl > 6.81 && tl < 10.1;
    let ruler_line = tl > 3.24 && tl < 6.81;
    let shake = if wobble_line { ((tg * 7.4).sin() * 5.0) as f32 } else { ((tg * 1.1).sin() * 1.2) as f32 };
    let (tw, th) = art::size(&art::TABLE, 7);
    let tx = 760.0 + shake;
    let ty = horizon - th as f32 + 6.0;
    // floor shadow
    ui::shadow(cv, tx + tw as f32 / 2.0 + 20.0, horizon + 10.0, tw as f32 * 0.62, 22.0, Color::hex(0x000000), 0.55);

    let a = ease_out_cubic(born);
    if a < 1.0 {
        let sc = 6;
        art::draw_at(cv, &art::TABLE, tx as i32, (ty + (1.0 - a as f32) * 60.0) as i32, sc, a, None);
    } else {
        // draw the table row by row so the stubby leg can wobble on its own
        let rows = art::TABLE.rows;
        for (ry, row) in rows.iter().enumerate() {
            for (rx, ch) in row.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let col = art::TABLE
                    .pal
                    .iter()
                    .find(|(k, _)| *k == ch)
                    .map(|(_, hx)| Color::hex(*hx))
                    .unwrap_or(accent);
                // the last leg (columns 31..37 on the lower rows) is the stubby one
                let stubby = rx >= 31 && rx <= 37 && ry >= 11;
                let dx = if stubby {
                    (tg * 9.0).sin() * 5.0
                } else {
                    ((tg * 1.4).sin() * 1.2) + (ry as f64 * 0.02).sin()
                };
                cv.rect_a(
                    tx as i32 + rx as i32 * 7 + dx as i32,
                    ty as i32 + ry as i32 * 7,
                    7,
                    7,
                    col,
                    1.0,
                );
            }
        }
        // one grain highlight sweeping along the table top
        let sweep = 300.0 + (tg * 40.0).rem_euclid(tw as f64 + 400.0) as f32;
        cv.rect_a(tx as i32 + sweep as i32, ty as i32, 90, 12, Color::hex(0xf0c690), 0.18);
    }

    // ---- the elbow ruler
    if ruler_line {
        let p = clamp01((tl - 3.24) / 1.2);
        let rx = tx + tw as f32 + 90.0;
        let top = ty - 40.0;
        cv.rect_a(rx as i32, top as i32, 2, (horizon - top) as i32, accent, 0.7 * p);
        for k in 0..14 {
            let y = top + k as f32 * (horizon - top) / 14.0;
            let long = k % 2 == 0;
            cv.rect_a(rx as i32, y as i32, if long { 26 } else { 14 }, 2, accent, 0.6 * p);
        }
        cv.text_px(rx + 40.0, ty + 6.0, "elbow 0.72m", 26.0, false, accent.glow(0.3), p, false);
        cv.rect_a((rx - 20.0) as i32, (ty + 4.0) as i32, 40, 3, pal::c(pal::ROSE), 0.9 * p);
    }

    // ---- creaks while it wobbles
    if wobble_line && (tg * 7.4).sin() > 0.8 {
        for k in 0..5 {
            let th = (tg * 20.0 + k as f64 * 1.3) % std::f64::consts::TAU;
            cv.disc(
                tx + tw as f32 * 0.72 + (th.cos() as f32) * 26.0,
                horizon - 40.0 + (th.sin() as f32) * 14.0,
                3.0,
                accent.glow(0.4),
                0.7,
            );
        }
    }

    // ---- metadata: a column of facts pinned to the left wall
    let entered = tl - 0.9;
    ui::panel(g, 3, 3, 34, 14, "you.table", accent, Some(pal::c(pal::BG0).scale(1.15)));
    let props: [(&str, &str); 5] = [
        ("type", "furniture", ),
        ("height", "0.72m"),
        ("legs", "4 (1 stubby)"),
        ("wobble", "cozy"),
        ("love", "♥♥♥♥♥"),
    ];
    for (i, (k, v)) in props.iter().enumerate() {
        let st = clamp01((entered - i as f64 * 0.22) * 1.6);
        ui::prop(g, 5, 6 + i as i64, 30, k, v, st, accent);
    }
    ui::cmd(g, 5, 13, "world.search(you) => you.table", tl - 0.2, accent);

    // ---- the reticle locks on
    let lock = clamp01((tl - 1.2) / 1.4);
    ui::bracket(
        cv,
        tx - 40.0,
        ty - 46.0,
        tw as f32 + 80.0,
        horizon - ty + 70.0,
        accent,
        0.55,
        54.0,
    );
    ui::reticle(
        cv,
        tx + tw as f32 / 2.0,
        horizon - 150.0,
        210.0,
        0.25,
        tg,
        accent,
        0.20 + 0.12 * ctx.beats.pulse(tg),
        lock,
    );

    // ---- PERFECT (51.00)
    if tg > 51.00 {
        ui::stamp(cv, g, tx + tw as f32 / 2.0, horizon - 170.0, (tg - 51.00) / 1.1, pal::c(pal::GOLD));
    }
    env::dust(cv, tg, 30, 0xB0A7, accent, 0.20, 120.0, 900.0);
    ui::edge_shade(cv, Color::hex(0x08060c), 0.62);
}

// =============================================================== eggplant ===

pub fn eggplant(g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx) {
    let accent = pal::c(pal::PURPLE);
    // ---- a kitchen: cool tiled wall over a board along the bottom
    env::vgrad(cv, 0, 0, W as i32, 720, Color::hex(0x141a24), Color::hex(0x0b0f18), 1.0);
    for gy in 0..10 {
        for gx in 0..26 {
            let x = gx as f32 * (W as f32 / 26.0);
            let y = gy as f32 * 72.0;
            cv.rect_a(x as i32, y as i32, 1, 72, Color::hex(0x1e2634), 0.7);
            cv.rect_a(x as i32, y as i32, (W as f32 / 26.0) as i32, 1, Color::hex(0x1e2634), 0.7);
        }
    }
    // the board
    let board_y = 720.0;
    cv.rect_a(0, board_y as i32, W as i32, 8, Color::hex(0xb5834e), 0.95);
    cv.vgrad(0, (board_y + 8.0) as i32, W as i32, H as i32 - board_y as i32 - 8, Color::hex(0x5e4029), Color::hex(0x2e1f16), 1.0);
    for k in 0..40 {
        let y = board_y + 20.0 + k as f32 * 12.0;
        cv.rect_a(0, y as i32, W as i32, 1, Color::hex(0x3a281b), 0.45);
    }

    // ---- the eggplant: big, centre, gently swaying
    let sway = ((tg * 1.3).sin() * 12.0) as f32;
    let (ew, eh) = art::size(&art::EGGPLANT, 11);
    let ex = CX - ew as f32 / 2.0 + sway;
    let ey = board_y - eh as f32 + 16.0;
    ui::shadow(cv, ex + ew as f32 / 2.0, board_y + 12.0, ew as f32 * 0.58, 26.0, Color::hex(0x000000), 0.5);
    art::draw_at(cv, &art::EGGPLANT, ex as i32, ey as i32, 11, 1.0, None);
    // rim light from the left
    art::draw_at(cv, &art::EGGPLANT, (ex - 7.0) as i32, ey as i32, 11, 0.30, Some(Color::hex(0xc7a3ee)));

    // the purple friend: a face appears
    if tl > 3.3 && tl < 6.8 {
        let a = clamp01((tl - 3.3) / 0.6) * clamp01((6.8 - tl) / 0.5);
        let fxx = ex + ew as f32 * 0.36;
        let fyy = ey + eh as f32 * 0.40;
        cv.disc(fxx, fyy, 11.0, Color::hex(0x2b1b45), a);
        cv.disc(fxx + 66.0, fyy, 11.0, Color::hex(0x2b1b45), a);
        cv.disc(fxx + 33.0, fyy + 66.0, 16.0, Color::hex(0x2b1b45), a);
        cv.disc(fxx - 3.0, fyy - 3.0, 3.4, Color::rgb(255, 255, 255), a * 0.9);
        cv.disc(fxx + 63.0, fyy - 3.0, 3.4, Color::rgb(255, 255, 255), a * 0.9);
    }

    // ---- nail marks, one per beat, after 61.22
    if tg > 61.22 {
        let marks: [(f32, f32, f32); 6] = [
            (0.30, 0.30, 0.0),
            (0.62, 0.42, 0.6),
            (0.42, 0.58, 1.2),
            (0.70, 0.66, 1.8),
            (0.52, 0.76, 2.4),
            (0.36, 0.52, 3.0),
        ];
        for (mx, my, at) in marks {
            let age = tg - 61.22 - at as f64;
            if age > 0.0 {
                let a = clamp01(age * 2.5) * (1.0 - clamp01((age - 2.4) / 2.4) * 0.35);
                let x = ex + ew as f32 * mx;
                let y = ey + eh as f32 * my;
                for k in 0..3 {
                    cv.rect_a(
                        (x + k as f32 * 3.0) as i32,
                        (y + k as f32 * 5.0) as i32,
                        3,
                        12,
                        Color::hex(0xe8a7c3),
                        a * 0.85,
                    );
                }
                cv.rect_a(x as i32 - 2, y as i32 - 4, 12, 3, Color::hex(0x2b1b45), a);
                if age < 0.5 {
                    cv.glow_at(x, y, 60.0, Color::hex(0xe8a7c3), 0.5 * (1.0 - age * 2.0), 2.5);
                }
            }
        }
    }

    // ---- metadata: a horizontal strip along the top
    let entered = tl - 0.7;
    let props: [(&str, &str); 5] = [
        ("colour", "purple"),
        ("role", "friend"),
        ("scar", "purple friend"),
        ("marks", "3 scratchy"),
        ("love", "♥♥♥♥♥"),
    ];
    let sw = 30i64;
    for (i, (k, v)) in props.iter().enumerate() {
        let x = 4 + i as i64 * (sw + 2);
        let st = clamp01((entered - i as f64 * 0.2) * 1.6);
        ui::panel(g, x, 3, sw, 5, "", accent, Some(pal::c(pal::BG0).scale(1.1)));
        ui::prop(g, x + 2, 5, sw - 3, k, v, st, accent);
    }

    let lock = clamp01((tl - 1.0) / 1.4);
    ui::reticle(cv, CX, ey + eh as f32 * 0.5, 330.0, 0.2, tg * 0.4, accent, 0.18, lock);
    if tg > 64.66 {
        ui::stamp(cv, g, CX, board_y - 240.0, (tg - 64.66) / 1.1, pal::c(pal::GOLD));
    }
    env::dust(cv, tg, 26, 0xE661, accent, 0.18, 120.0, 700.0);
    ui::edge_shade(cv, Color::hex(0x070a12), 0.58);
    let _ = ctx;
}

// ==================================================================== cat ===

pub fn cat(g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx) {
    let accent = pal::c(pal::BLUEPT);
    // ---- a big, cold, empty room at night
    env::vgrad(cv, 0, 0, W as i32, H as i32, Color::hex(0x0d1220), Color::hex(0x05070e), 1.0);
    // window: a tall bright rectangle, moonlight
    let wx = 1180.0f32;
    cv.rect_a(wx as i32, 90, 420, 620, Color::hex(0x22314c), 0.85);
    cv.rect_a(wx as i32, 90, 420, 620, Color::hex(0x9fb2cf), 0.10);
    cv.rect_a((wx + 204.0) as i32, 90, 10, 620, Color::hex(0x44557a), 0.8);
    env::shafts(cv, tg, 0xCA7, Color::hex(0x9fb2cf), 0.05);
    // the floor, and the pool of moonlight on it
    let horizon = 760.0f32;
    env::ground(cv, horizon as i32, Color::hex(0x0a0e18), 0.95);
    cv.ellipse(wx + 210.0, horizon + 40.0, 460.0, 90.0, Color::hex(0x9fb2cf), 0.10);

    // ---- the kitten: small, on the sill, silhouetted
    let sit = ((tg * 0.7).sin() * 4.0) as f32;
    let (cw, chh) = art::size(&art::CAT, 9);
    let cxx = wx + 210.0 - cw as f32 / 2.0;
    let cyy = 700.0 - chh as f32 + sit;
    ui::shadow(cv, cxx + cw as f32 / 2.0, 706.0, cw as f32 * 0.5, 16.0, Color::hex(0x000000), 0.6);
    art::draw_at(cv, &art::CAT, cxx as i32, cyy as i32, 9, 1.0, None);
    // rim of moonlight on the left edge
    art::draw_at(cv, &art::CAT, (cxx - 6.0) as i32, cyy as i32, 9, 0.22, Some(Color::hex(0xd8e6ff)));

    // the eyes catch the light and blink
    let blink = if ((tg * 0.55) % 1.0) < 0.06 { 0.0 } else { 1.0 };
    cv.glow_at(cxx + cw as f32 * 0.36, cyy + chh as f32 * 0.30, 60.0, pal::c(pal::BLUEPT), 0.5 * blink, 2.6);
    cv.glow_at(cxx + cw as f32 * 0.64, cyy + chh as f32 * 0.30, 60.0, pal::c(pal::BLUEPT), 0.5 * blink, 2.6);

    // ---- meow (89.76–92.5): pixel speech, sound-rings
    if tg > 89.76 && tg < 92.58 {
        let p = clamp01((tg - 89.76) / 2.8);
        let k = ctx.beats.beat_index(tg);
        let age = tg - ctx.beats.beat_time(k);
        let a = clamp01(1.0 - age / 1.4);
        cv.text_px(
            cxx + cw as f32 + 40.0,
            cyy - 60.0 - (age * 30.0) as f32,
            "meow~",
            40.0,
            false,
            accent.glow(0.35),
            a,
            false,
        );
        cv.ring(cxx + cw as f32 / 2.0, cyy + 40.0, 90.0 + (age * 200.0) as f32, 2.0, accent, a * 0.5);
        let _ = p;
    }
    // ---- hiss (92.58–94.39)
    if tg > 92.58 && tg < 94.39 {
        let a = clamp01((tg - 92.58) / 0.3) * clamp01((94.39 - tg) / 0.4);
        let mut rng = Rng::new((tg * 44.0) as u64);
        for _ in 0..26 {
            let hx = cxx + cw as f32 * 0.6 + (rng.f64() * 260.0) as f32;
            let hy = cyy + 20.0 + (rng.f64() * 160.0) as f32 - 80.0;
            cv.rect_a(hx as i32, hy as i32, (6.0 + 10.0 * rng.f64()) as i32, 3, accent, a * (0.4 + 0.6 * rng.f64()));
        }
        cv.text_px(cxx + cw as f32 + 60.0, cyy + 6.0, "sss!!", 38.0, true, accent, a, false);
    }
    // ---- purr: NEVER. the readout breaks (94.39)
    if tg > 94.39 {
        let p = clamp01((tg - 94.39) / 1.2);
        let note = "purr .... NOT FOUND";
        let nw = cv.text_width(note, 34.0, true);
        cv.text_px(cxx + cw as f32 + 60.0, cyy + 20.0, note, 34.0, true, pal::c(pal::ROSE), p, false);
        cv.rect_a((cxx + cw as f32 + 56.0) as i32, (cyy + 30.0) as i32, nw as i32 + 8, 2, pal::c(pal::ROSE), p * 0.9);
        // a z z z that never becomes a purr
        for k in 0..3 {
            let zz = ((tg * 1.4 + k as f64 * 0.35) % 1.0) as f32;
            cv.text_px(
                cxx - 40.0 - k as f32 * 26.0,
                cyy - 10.0 - zz * 60.0,
                "z",
                22.0 + k as f32 * 6.0,
                true,
                accent,
                (1.0 - zz) as f64 * 0.6 * p,
                false,
            );
        }
    }

    // ---- metadata: a single slim card, bottom-left, out of the way
    let entered = tl - 1.0;
    ui::panel(g, 3, 28, 40, 8, "you.cat", accent, Some(pal::c(pal::BG0).scale(1.15)));
    let props: [(&str, &str); 4] = [
        ("coat", "bluepoint"),
        ("meow", "a lot"),
        ("hiss", "a lot"),
        ("purr", "NOT FOUND"),
    ];
    for (i, (k, v)) in props.iter().enumerate() {
        let st = clamp01((entered - i as f64 * 0.35) * 1.5);
        ui::prop(g, 5, 30 + i as i64, 36, k, v, st, accent);
    }

    ui::reticle(cv, cxx + cw as f32 / 2.0, cyy + chh as f32 * 0.4, 190.0, 0.3, tg * 0.3, accent, 0.16, 0.4);
    let hx = (cxx - 60.0) as i32;
    let hy = (cyy + 40.0 - (((tg * 1.2) % 3.0) as f32) * 40.0) as i32;
    cv.sprite('♥', hx as f32, hy as f32, 26.0, pal::c(pal::ROSE), 0.35);
    ui::edge_shade(cv, Color::hex(0x04060c), 0.66);
}

// ================================================================== steak ===

pub fn steak(g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx) {
    let accent = pal::c(pal::SEAR);
    // ---- top-down on a hot pan: an iron disc filling the frame
    env::void_grad(cv, Color::hex(0x100b08), Color::hex(0x050302), 1.0);
    let pcx = CX;
    let pcy = H as f32 * 0.52;
    cv.disc(pcx, pcy, 720.0, Color::hex(0x1c1712), 1.0);
    cv.ring(pcx, pcy, 700.0, 26.0, Color::hex(0x2a241d), 1.0);
    cv.ring(pcx, pcy, 660.0, 4.0, Color::hex(0x3a3129), 0.8);
    for k in 0..60 {
        let th = k as f64 * 0.7;
        let rr = 300.0 + ((k * 97) % 340) as f32;
        cv.disc(
            pcx + (th.cos() as f32) * rr,
            pcy + (th.sin() as f32) * rr,
            1.6,
            Color::hex(0x2e2a24),
            0.5,
        );
    }
    // heat haze rising
    env::ground_fog(cv, tg, Color::hex(0x3a2418), 0.16, 260.0);

    // ---- the steak
    let sizzle = 0.9 + 0.1 * (tg * 8.0).sin();
    let (sw, sh) = art::size(&art::STEAK, 13);
    let sx = CX - sw as f32 / 2.0;
    let sy = pcy - sh as f32 / 2.0 + 20.0;
    ui::shadow(cv, CX + 14.0, pcy + sh as f32 * 0.42, sw as f32 * 0.5, 40.0, Color::hex(0x000000), 0.55);
    art::draw_at(cv, &art::STEAK, sx as i32, sy as i32, 13, sizzle, None);
    // the sear darkens on "browned on the outside" (103.75)
    if tg > 103.75 {
        let a = clamp01((tg - 103.75) / 1.6) * 0.35;
        art::draw_at(cv, &art::STEAK, sx as i32, sy as i32, 13, a, Some(Color::hex(0x5a2a16)));
    }
    // sizzle sparks
    let mut rng = Rng::new((tg * 30.0) as u64 ^ 0x57EA1);
    for _ in 0..26 {
        let life = rng.f64();
        let ang = rng.f64() * std::f64::consts::TAU;
        let rr = 200.0 + rng.f64() * 460.0;
        let px = pcx + (ang.cos() as f32) * rr as f32;
        let py = pcy + (ang.sin() as f32) * rr as f32 * 0.6 - (life * 90.0) as f32;
        cv.disc(px, py, (1.6 + 2.4 * life) as f32, accent.glow(0.5), (1.0 - life) * 0.85);
    }
    // steam columns
    for c in 0..5 {
        let bx = pcx - 300.0 + c as f32 * 150.0;
        for k in 0..16 {
            let u = k as f32 / 16.0;
            let yy = pcy - 200.0 - u * 420.0;
            let xx = bx + ((tg * 1.1 + k as f64 * 0.5).sin() as f32) * 40.0 * u;
            cv.disc(xx, yy, (14.0 - u * 8.0).max(2.0), Color::hex(0x8a93a6), (0.055 * (1.0 - u)) as f64);
        }
    }

    // ---- cross-section scan (106.66 "medium rare inside") slides in
    if tg > 106.66 {
        let p = clamp01((tg - 106.66) / 0.9);
        let e = ease_out_cubic(p) as f32;
        let pw = 470.0;
        let ph = 300.0;
        let x = W as f32 - 90.0 - pw * e;
        let y = 130.0;
        cv.rect_a(0, 0, W as i32, H as i32, Color::hex(0x000000), 0.35 * p);
        cv.rect_a(x as i32, y as i32, pw as i32, ph as i32, Color::hex(0x0b0d12), 0.92 * p);
        ui::dashed_rect(cv, x, y, pw, ph, 10.0, accent, 0.8 * p);
        // the cut: outer ring brown, inner pink
        cv.ellipse(x + pw / 2.0, y + ph / 2.0, 170.0, 100.0, Color::hex(0x8a4f30), p);
        cv.ellipse(x + pw / 2.0, y + ph / 2.0, 138.0, 76.0, Color::hex(0xd98a8a), p);
        cv.ellipse(x + pw / 2.0 - 22.0, y + ph / 2.0 - 10.0, 70.0, 40.0, Color::hex(0xf0a8a8), p * 0.8);
        cv.text_px(x + 20.0, y + 34.0, "CROSS-SECTION", 24.0, true, accent, p, false);
        cv.text_px(x + 20.0, y + ph - 20.0, "core 55°C · medium rare", 26.0, false, pal::c(pal::BRIGHT), p, false);
        // doneness gauge
        for k in 0..28 {
            let u = k as f32 / 27.0;
            let col = if u < 0.28 {
                Color::hex(0xe86a5e)
            } else if u < 0.55 {
                Color::hex(0xe8a3a3)
            } else {
                Color::hex(0x8a4f30)
            };
            cv.rect_a((x + 20.0 + u * (pw - 40.0)) as i32, (y + ph - 60.0) as i32, 12, 14, col, p * 0.9);
        }
        cv.rect_a((x + 20.0 + 0.36 * (pw - 40.0)) as i32, (y + ph - 66.0) as i32, 4, 26, Color::rgb(255, 255, 255), p);
    }

    // ---- metadata strip, vertical on the left edge
    let entered = tl - 0.8;
    let props: [(&str, &str); 5] = [
        ("cut", "fillet mignon"),
        ("sauce", "mushroom"),
        ("sear", "golden"),
        ("inside", "medium rare"),
        ("love", "♥♥♥♥♥"),
    ];
    for (i, (k, v)) in props.iter().enumerate() {
        let st = clamp01((entered - i as f64 * 0.18) * 1.6);
        ui::prop(g, 3, 4 + i as i64 * 2, 44, k, v, st, accent);
    }
    ui::reticle(cv, CX, pcy, 300.0, 0.22, -tg * 0.35, accent, 0.14, 0.5);
    env::dust(cv, tg, 30, 0x5723, accent, 0.24, 200.0, 900.0);
    ui::edge_shade(cv, Color::hex(0x050302), 0.72);
    let _ = (ease_in_out, ctx);
}
