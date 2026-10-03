//! Scene: outro (4:35–4:50) — puppy, then a tomato full of juice.

use super::Ctx;
use crate::art;
use crate::gfx::{clamp01, Color, Grid, Rng};
use crate::scenes::ui;

const CREAM: Color = Color::hex(0xe8c39e);
const DIM: Color = Color::hex(0x55607a);
const ROSE: Color = Color::hex(0xff6d8a);
const TOM_RED: Color = Color::hex(0xc94436);
const JUICE: Color = Color::hex(0xe86a5e);

pub fn render(g: &mut Grid, tl: f64, tg: f64, ctx: &Ctx) {
    let w = g.w as i64;
    let h = g.h as i64;

    ui::headline(g, 4, "world/ · continuing…", CREAM, DIM);

    // dog phase: tl < 7
    if tl < 7.0 {
        let (dw, dh) = art::sprite_size(&art::DOG_A, 2);
        let cx = w / 2;
        let cy = h / 2 - 3;
        let ox = cx - dw / 2;
        let oy = cy - dh / 2;
        let wag = if (tg * 3.0).sin() > 0.0 { &art::DOG_B } else { &art::DOG_A };
        let bob = ((tg * 2.0).sin() * 0.5).round() as i64;
        art::draw(g, wag, ox, oy + bob, 2, true, 1.0, None);
        // "ruff!" on the ruff line
        if tl > 4.6 {
            let k = ctx.beats.beat_index(tg);
            let bt = ctx.beats.beat_time(k);
            let age = tg - bt;
            if age < 0.9 {
                let x = ox + dw - 8 + (age * 8.0) as i64;
                let y = oy - 2 - (age * 3.0) as i64;
                g.text(x, y, "ruff!", CREAM.glow(0.35 * (1.0 - age)));
            }
        }
        g.text(ox, oy + dh + 2, "a little puppy", DIM);
    } else {
        // tomato phase
        let (tw, th) = art::sprite_size(&art::TOMATO, 2);
        let cx = w / 2;
        let cy = h / 2 - 1;
        let ox = cx - tw / 2;
        let oy = cy - th / 2;
        art::draw(g, &art::TOMATO, ox, oy, 2, true, 1.0, None);

        // "full of juice": interior fills up
        let fill_t = clamp01((tl - 11.4) / 2.4);
        let (ix, iy, iw, ih) = (ox + 4, oy + 6, tw - 9, th - 7);
        for row in 0..ih {
            for col in 0..iw {
                let cell = row as f64 / ih as f64;
                if cell < fill_t {
                    g.put(ix + col, iy + row, if (col + row) % 3 == 0 { '≈' } else { '~' }, JUICE);
                }
            }
        }
        // juice level label + droplets
        g.text(ox, oy + th + 1, &format!("juice: {}%", (fill_t * 100.0) as i64), JUICE);
        let mut rng = Rng::new((tg * 20.0) as u64);
        for _ in 0..(3.0 * fill_t) as usize {
            let life = (tg * 1.5 + rng.f64() * 2.0) % 2.0;
            let dx = ox + tw - 3 + rng.i64(-2, 2);
            let dy = oy + th - 1 + (life * 6.0) as i64;
            g.put_alpha(dx, dy, '·', JUICE, 0.7 * (1.0 - life / 2.0));
        }
        // heart
        g.put(ox + tw + 2, oy + 2, '♥', ROSE);
    }
    let _ = TOM_RED;
}
