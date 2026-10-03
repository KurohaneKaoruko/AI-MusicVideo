//! Scene: buckets (3:27–3:53) — each of us carries a bucket of love; we get cut into
//! cells, then merged back — evolved.

use super::Ctx;
use crate::art;
use crate::fx;
use crate::gfx::{clamp01, ease_out_cubic, Color, Grid, Rng};

const LIQ_A: Color = Color::hex(0xff6d8a);
const LIQ_B: Color = Color::hex(0xffa07f);
const TEAL: Color = Color::hex(0x6fc7b8);
const VIOLET: Color = Color::hex(0x9a8dd7);
const DIM: Color = Color::hex(0x55607a);
const ROSE: Color = Color::hex(0xff6d8a);

pub fn render(g: &mut Grid, tl: f64, tg: f64, ctx: &Ctx) {
    let w = g.w as i64;
    let h = g.h as i64;

    // section-local phases: 0–7.2 buckets · 14.0 cut into cells · 21.0 merge/evolve
    let cut = clamp01((tl - 14.0) / 2.2);
    let merge = clamp01((tl - 21.0) / 2.6);
    let evolve = clamp01((tl - 23.6) / 2.2);

    let n = 4;
    let total_w = n * 14;
    let x0 = (w - total_w) / 2;
    let base_y = h / 2 - 2;

    // ---- cell inset (top-right) during the cut
    if cut > 0.0 && merge < 0.9 {
        let px = w - 24;
        let py = 5;
        g.box_rounded(px, py, 20, 8, DIM.over(g.row_bg(py as usize), 0.8 * (1.0 - merge)), None, Some(("zoom: cell", DIM)));
        // a cell dividing
        let p = (cut * 2.0) % 1.0;
        let cxm = px + 10;
        let cym = py + 4;
        g.put(cxm - 2, cym, '(', TEAL);
        g.put(cxm + 1, cym, ')', TEAL);
        g.put(cxm - 2, cym - 1, '.', TEAL);
        g.put(cxm - 2, cym + 1, '`', TEAL);
        // two daughter cells drifting apart
        let split = clamp01(p * 1.4);
        let off = (split * 5.0) as i64;
        for s in [-1i64, 1] {
            let x = cxm + s * off;
            let ch = ['o', 'o'][0];
            g.put(x, cym, ch, VIOLET);
        }
    }

    if merge < 0.98 {
        // ---- buckets
        for i in 0..n {
            let bx = x0 + i * 14;
            // liquid height varies per bucket (some full, some empty)
            let level: f64 = [0.9, 0.25, 0.6, 0.1][i as usize];
            let spr = &art::BUCKET;
            let appear = clamp01((tl - i as f64 * 0.25) / 0.5);
            // dissolve into cells as cut progresses
            let a = appear * (1.0 - cut * 0.7);
            if a < 0.02 { continue }

            art::draw(g, spr, bx, base_y, 2, true, a, None);

            // love-o-meter label
            let pct = (level * 100.0) as i64;
            let lcol = if level > 0.7 { LIQ_A } else if level > 0.35 { LIQ_B } else { DIM };
            g.text_alpha(bx, base_y - 2, &format!("love {}%", pct), lcol, a);

            // liquid in bucket: `~` layers
            let lx = bx + 3;
            let ly = base_y + 1;
            let layers = (level * 2.0).round() as i64;
            for l in 0..layers {
                let col = if i % 2 == 0 { LIQ_A } else { LIQ_B };
                let wave = ((tg * 2.0 + l as f64) * 0.8).sin() * 0.5;
                for k in 0..4 {
                    let ch = if level > 0.7 { '♥' } else if (k as f64 + wave) as i64 % 2 == 0 { '~' } else { '-' };
                    g.put_alpha(lx + k, ly + l, ch, col, a);
                }
            }
            // overflow hearts for the fullest
            if level > 0.8 {
                let hp = (tg % 2.0) / 2.0;
                g.put_alpha(lx + 1, ly - 1 - (hp * 2.0) as i64, '♥', ROSE, a * (1.0 - hp));
            }
            // a name tag
            g.text_alpha(bx, base_y + 19, &format!("{}", i + 1), DIM, a * 0.7);
        }

        // cutting: shatter the buckets into cell particles
        if cut > 0.0 {
            let mut rng = Rng::new(0xC711);
            for _ in 0..(90.0 * cut) as usize {
                let sx = x0 + rng.i64(0, total_w);
                let sy = base_y + rng.i64(0, 10);
                let vx = rng.range(-1.5, 1.5);
                let vy = rng.range(0.5, 3.0);
                let px = sx + (tg * vx * 4.0) as i64;
                let py = sy + (tg * vy * 4.0) as i64;
                if px < 0 || px >= w || py >= h - 10 { continue }
                let ch = ['o', '°', '.', '·'][rng.i64(0, 3) as usize];
                let col = if rng.f64() < 0.5 { TEAL } else { VIOLET };
                g.put_alpha(px, py, ch, col, (1.0 - merge) * 0.95);
            }
        }
    } else {
        // ---- merged creature (evolved)
        let (sw, sh) = art::sprite_size(&art::CHIMERA, 2);
        let cx = w / 2 - sw / 2;
        let cy = h / 2 - sh / 2 - 1;
        let a = ease_out_cubic(evolve);
        // assembling: draw sprite with dithered reveal by column groups
        for (ry, row) in art::CHIMERA.rows.iter().enumerate() {
            for (rx, ch) in row.chars().enumerate() {
                if ch == ' ' { continue }
                let col = art::CHIMERA.pal.iter().find(|(k, _)| *k == ch).map(|(_, hx)| Color::hex(*hx)).unwrap_or(ROSE);
                let thresh = (rx as f64 / 12.0) * 0.5 + (ry as f64 / 12.0) * 0.5;
                if a > thresh {
                    g.put_alpha(cx + rx as i64 * 2, cy + ry as i64, ch, col, clamp01((a - thresh) * 4.0));
                    g.put_alpha(cx + rx as i64 * 2 + 1, cy + ry as i64, ch, col, clamp01((a - thresh) * 4.0));
                }
            }
        }
        // evolved stamp
        if evolve > 0.7 {
            let lbl = "EVOLVED [ok]";
            let a2 = clamp01((evolve - 0.7) * 3.0);
            g.text_center(cy + sh + 2, lbl, TEAL.glow(0.3).over(g.row_bg((cy + sh + 2) as usize), a2));
        }
    }

    // merge energy: particles converge into the center
    if merge > 0.0 && merge < 1.0 {
        let mut rng = Rng::new(0xE0E);
        for _ in 0..40 {
            let a0 = rng.range(0.0, std::f64::consts::TAU);
            let r0 = rng.range(8.0, 30.0);
            let sx = w / 2 + (a0.cos() * r0) as i64;
            let sy = h / 2 + (a0.sin() * r0 * 0.5) as i64;
            let pull = ease_out_cubic(merge);
            let px = sx + ((w / 2 - sx) as f64 * pull) as i64;
            let py = sy + ((h / 2 - sy) as f64 * pull) as i64;
            let ch = ['o', '·', '°'][rng.i64(0, 2) as usize];
            g.put_alpha(px, py, ch, TEAL, (1.0 - merge) * 0.9);
        }
    }

    let _ = (ctx, fx::typed("", 0.0, 1.0));
}
