//! ACT I — the open (0:00–0:14). A cold machine wakes in the dark, is handed a
//! query, and opens its acquisition reticle for the first time.
//!
//! Composition: nothing but a cursor, then a line of type, then the ring.
//! No panels, no chrome, no lyrics — the film earns its furniture later.

use super::ui;
use crate::env;
use crate::gfx::{clamp01, ease_in_out, ease_out_cubic, pal, Color, Grid};
use crate::pix::{Canvas, H, W};
use crate::scenes::Ctx;

pub fn render(_g: &mut Grid, cv: &mut Canvas, tl: f64, tg: f64, ctx: &Ctx) {
    let rose = pal::c(pal::ROSE);
    let slate = pal::c(pal::SLATE);
    let cx = W as f32 / 2.0;
    let cy = H as f32 / 2.0;
    let beat = ctx.beats.pulse(tg);

    // ---- the void: cold, almost empty, a few distant points
    env::void_grad(cv, Color::hex(0x070a12), Color::hex(0x03040a), 1.0);
    env::starfield(cv, tg, 90, 0x5EA1, Color::hex(0x9fb2cf), 0.28);

    // ---- the machine's own pulse: a slow breathing ring far behind
    let hr = 300.0 + 14.0 * (tg * 0.6).sin() as f32;
    cv.ring(cx, cy, hr, 1.2, Color::hex(0x1b2433), 0.55);

    let q = "world.search(you);";

    if tl < 4.6 {
        // ---- phase 1: a cursor, alone
        let on = ((tg * 1.6) as i64) % 2 == 0;
        if on {
            cv.rect_a((cx - 7.0) as i32, (cy - 20.0) as i32, 14, 38, slate.glow(0.15), 0.85);
        }
        let a = clamp01((tl - 0.9) / 1.4) * clamp01((4.6 - tl) / 0.9);
        if a > 0.02 {
            let cz = pal::c(pal::BRIGHT).scale(0.72);
            cv.text_px(cx, cy + 120.0, crate::lyrics::CREDITS[0].0, 25.0, false, cz, a * 0.85, true);
            cv.text_px(cx, cy + 152.0, crate::lyrics::CREDITS[1].0, 25.0, false, cz, a * 0.85, true);
            cv.rect_a((cx - 70.0) as i32, (cy + 100.0) as i32, 140, 1, slate, a * 0.4);
        }
    } else {
        // ---- phase 2: the query is typed into the void
        let typed = ((tl - 4.6) * 11.0).clamp(0.0, q.chars().count() as f64) as usize;
        let shown: String = q.chars().take(typed).collect();
        let full_w = cv.text_width(q, 62.0, false);
        let x0 = cx - full_w / 2.0;
        let base = cy + 8.0;
        // the world brightens a little as the query lands
        let lit = clamp01((tl - 4.6) / 3.0);
        cv.glow_at(cx, base - 20.0, 620.0, rose, 0.05 + 0.05 * lit, 2.6);

        cv.text_px(x0, base, &shown, 62.0, false, Color::hex(0xdfe7f4), 0.94, false);
        // every landed glyph ticks
        if typed > 0 && ((tl - 4.6) * 11.0).fract() < 0.35 {
            let w = cv.text_width(&shown, 62.0, false);
            cv.rect_a((x0 + w + 4.0) as i32, (base - 46.0) as i32, 3, 52, rose, 0.9);
        }
        // the caret
        if (((tl - 4.6) * 3.0) as i64) % 2 == 0 || typed < q.chars().count() {
            let w = cv.text_width(&shown, 62.0, false);
            cv.rect_a((x0 + w + 10.0) as i32, (base - 46.0) as i32, 12, 52, rose, 0.85);
        }

        // ---- phase 3: the reticle blossoms out of the word "you"
        if tl > 8.2 {
            let p = clamp01((tl - 8.2) / 2.6);
            let e = ease_in_out(p);
            let r = 40.0 + 470.0 * e as f32;
            let open = ease_out_cubic(clamp01((tl - 8.2) / 1.4));
            // highlight the target word
            let you_x = x0 + cv.text_width("world.search(", 62.0, false);
            let you_w = cv.text_width("you", 62.0, false);
            cv.rect_a(
                (you_x - 8.0) as i32,
                (base - 58.0) as i32,
                (you_w + 16.0) as i32,
                68,
                rose,
                0.10 + 0.10 * beat,
            );
            cv.text_px(you_x, base, "you", 62.0, false, rose.glow(0.35), 1.0, false);
            ui::reticle(cv, cx, base - 20.0, r, open, tg * 0.5, rose, 0.75 * (1.0 - 0.6 * e), 0.0);
        }

        // ---- phase 4: the boot log, then the ring snaps shut
        if tl > 11.0 {
            let lines: [(&str, f64); 4] = [
                ("world .............. mounted", 11.0),
                ("memories ........... indexed", 11.9),
                ("love_sensor ........ calibrating", 12.7),
                ("candidates: 8,141,596,233", 13.4),
            ];
            let lx = x0 - 30.0;
            let mut ly = base + 110.0;
            for (s, at) in lines {
                let e = tl - at;
                if e > 0.0 {
                    let n = ((e * 46.0) as usize).min(s.chars().count());
                    let t: String = s.chars().take(n).collect();
                    let col = if s.starts_with("candidates") { rose } else { pal::c(pal::GREEN_OK).scale(0.85) };
                    cv.text_px(lx, ly, &t, 27.0, false, col, clamp01((e * 2.0) as f64) * 0.9, false);
                }
                ly += 40.0;
            }
        }
        // the final quarter-second: the ring collapses to a point (act break)
        if tl > 13.9 {
            let p = clamp01((tl - 13.9) / 0.46);
            let f = (1.0 - p) as f32;
            cv.ring(cx, cy, 470.0 * f + 2.0, 3.0, rose, 0.9 * (1.0 - p as f32) as f64);
        }
    }

    // a single scan line passing over everything, twice per bar
    let sweep = ((tg * 0.28) % 1.0) as f32 * H as f32;
    env::haze(cv, sweep, 26.0, Color::hex(0x2b3547), 0.10);

    // dust so the dark never reads as dead
    env::dust(cv, tg, 22, 0x11A3, slate, 0.16, 60.0, H as f32 - 60.0);
}
