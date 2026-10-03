//! Scene: human (2:03, 4:07 reprise) — warmth from the hands, stinky breath, unruly hair.

use super::Ctx;
use crate::art;
use crate::fx;
use crate::gfx::{clamp01, Color, Grid, Rng};
use crate::scenes::ui;

const SKIN: Color = Color::hex(0xe8c39e);
const WARM: Color = Color::hex(0xff9e5e);
const GOLD: Color = Color::hex(0xffd27f);
const STINK: Color = Color::hex(0x8fbf6a);
const DIM: Color = Color::hex(0x55607a);

const SEGS: [f64; 4] = [0.0, 3.38, 6.80, 10.21];

pub fn render(g: &mut Grid, tl: f64, tg: f64, ctx: &Ctx, memory: bool) {
    let w = g.w as i64;
    let h = g.h as i64;
    let cx = w / 2;
    let cy = h / 2;

    if memory {
        let bw = (w - 12).min(80);
        let bh = h - 14;
        g.box_rounded(
            cx - bw / 2,
            cy - bh / 2,
            bw,
            bh,
            Color::hex(0x7a6a52).over(g.row_bg(cy as usize), 0.7),
            None,
            Some(("MEM//human.log", Color::hex(0xa8956f))),
        );
    }

    let seg = if tl >= SEGS[3] {
        3
    } else if tl >= SEGS[2] {
        2
    } else if tl >= SEGS[1] {
        1
    } else {
        0
    };
    let seg_t = tl - SEGS[seg];
    let beat = ctx.beats.pulse(tg);
    let sway = (tg * 0.8).sin();

    // ---- stage: a warm room, floor line, quiet dust
    let ground = h - 13;
    for x in 0..w {
        if (x - cx) % 7 == 0 {
            g.put_alpha(x, ground, '·', WARM.scale(0.5), 0.25);
        }
    }
    let mut rng = Rng::new((tg * 2.0) as u64);
    for _ in 0..12 {
        let x = rng.i64(2, w - 3);
        let y = rng.i64(3, ground);
        g.put_alpha(x, y, '·', GOLD, 0.05 + 0.07 * rng.f64());
    }

    // ---- the figure
    let (bw_, bh_) = (15i64, 12i64);
    let mut by = cy - bh_ - 1;
    let mut bx = cx - bw_ + sway.round() as i64;
    if memory {
        // keep the figure inside the memory window
        by = cy - bh_ / 2;
        bx = cx - bw_;
    }
    art::draw(g, &art::HUMAN, bx, by, 2, true, 1.0, None);

    // unruly hair flicks on the beat
    if beat > 0.6 {
        g.put(bx + 5, by, '^', SKIN.glow(0.25));
        g.put(bx + 22, by, '^', SKIN.glow(0.25));
    }

    // hand positions in the sprite
    let hand_l = (bx + 1 * 2, by + 5);
    let hand_r = (bx + 13 * 2, by + 5);

    // ---- warmth from the hands
    let warm_on = seg == 1 || seg == 3 || (memory && seg == 1);
    if warm_on || seg == 0 {
        let strength = if seg == 1 { 1.0 } else { 0.55 };
        let mut rng = Rng::new((tg * 30.0) as u64);
        for k in 0..16 {
            let life = (tg * 1.1 + rng.f64() * 3.0) % 3.0;
            let hx = if k % 2 == 0 { hand_l.0 as f64 } else { hand_r.0 as f64 };
            let spread = rng.range(-16.0, 16.0);
            let x = hx + spread * (life / 3.0);
            let y = hand_l.1 as f64 - 7.0 - life as f64 * 2.2 + (spread * 0.2).sin();
            let ch = ['*', '.', '\'', '°'][rng.i64(0, 3) as usize];
            g.put_alpha(
                x as i64,
                y as i64,
                ch,
                if k % 3 == 0 { GOLD } else { WARM },
                0.75 * (1.0 - life / 3.0) * strength,
            );
        }
        // expanding warmth rings on the beat
        if beat > 0.5 {
            for hx in [hand_l.0, hand_r.0] {
                let d = 3.0 + (1.0 - beat) * 9.0;
                for k in 0..8 {
                    let ang = k as f64 * std::f64::consts::TAU / 8.0;
                    g.put_alpha(
                        hx + (ang.cos() * d) as i64,
                        hand_l.1 - 3 - (ang.sin() * d * 0.5) as i64,
                        '~',
                        WARM,
                        beat * 0.35 * strength,
                    );
                }
            }
        }
    }

    // ---- stinky breath
    if seg == 2 {
        let a = clamp01(seg_t / 0.8) * clamp01((3.0 - seg_t) / 0.8 + 0.25);
        for k in 0..4 {
            let y = by + 2 - k;
            let x = bx + 30 + k as i64 * 2;
            let wig = ((tg * 3.0 + k as f64) * 2.0).sin().round() as i64;
            g.put_alpha(x + wig, y, '~', STINK, 0.5 * a);
        }
        let fa = tg * 4.0;
        let fx_ = bx + 28 + (fa.cos() * 5.0) as i64;
        let fy = by + 1 + (fa.sin() * 2.0) as i64;
        g.put_alpha(fx_, fy, '.', STINK, 0.8);
        g.text_alpha(bx + 34, by + 3, "(stinky...)", STINK, 0.55 * a);
    }

    // ---- the only perfect human
    if seg == 3 {
        if !memory && seg_t > 0.5 {
            ui::stamp_perfect(g, seg_t - 0.5, cx, cy - 8, WARM);
        }
        let p = clamp01(1.0 - seg_t / 2.2);
        for (rx, row) in art::HUMAN.rows.iter().enumerate() {
            for (cc, ch) in row.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let col = if cc % 5 == 0 { GOLD } else { WARM };
                g.put_alpha(bx + cc as i64 * 2, by + rx as i64, ch, col, 0.28 * p * beat);
            }
        }
    }

    if memory {
        fx::tint_all(g, Color::hex(0xd9c3a5), 0.5);
        fx::grain(g, tg, 0.7);
    }
    let _ = DIM;
}
