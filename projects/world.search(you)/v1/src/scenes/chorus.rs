//! Scene: chorus (2:59–3:27) — "I can see you in most everything".
//! A world crowded with every form of you, a search beam that lights them up one by
//! one, and then the terrible turn: the query is rewritten to "my old self".

use super::Ctx;
use crate::art;
use crate::fx;
use crate::gfx::{clamp01, ease_out_cubic, Color, Grid, Rng};

const ROSE: Color = Color::hex(0xff6d8a);
const DIM: Color = Color::hex(0x55607a);
const GOLD: Color = Color::hex(0xffd27f);

/// the forms of "you" — they keep transforming, they never settle
fn forms() -> [&'static art::Sprite; 8] {
    [
        &art::EGGPLANT,
        &art::CAT_A,
        &art::STEAK,
        &art::FLOWER_2,
        &art::HUMAN,
        &art::TABLE_TOP,
        &art::DOG_A,
        &art::TOMATO,
    ]
}

fn form_colors() -> [Color; 8] {
    [
        Color::hex(0x9a6dd7),
        Color::hex(0x9fb2cf),
        Color::hex(0xc9a86a),
        Color::hex(0xf2a7c3),
        Color::hex(0xe8c39e),
        Color::hex(0xd9a066),
        Color::hex(0xd9b98a),
        Color::hex(0xc94436),
    ]
}

pub fn render(g: &mut Grid, tl: f64, tg: f64, ctx: &Ctx) {
    let w = g.w as i64;
    let hi = g.h as i64;
    let sprites = forms();
    let cols = form_colors();

    // the turn: from ~20s in ("A-Ah") the world empties out
    let turn = clamp01((tl - 19.4) / 7.0);

    // ---- drifting forms (they morph every 4 beats)
    let beat4 = (ctx.beats.beat_index(tg) / 4).max(0);
    let n_forms = 8;
    let spots: [(i64, i64); 8] = [
        (4, 8), (30, 7), (58, 9), (84, 8),
        (10, 20), (38, 21), (66, 20), (90, 21),
    ];
    for k in 0..n_forms {
        let idx = ((k as i64 + beat4).rem_euclid(8)) as usize;
        let spr = &sprites[idx];
        let col = cols[idx];
        let (bx0, by0) = spots[k as usize];
        let sp = 0.25 + (k % 3) as f64 * 0.12;
        let ph = k as f64 * 1.7;
        let px = bx0 as f64 + (tg * sp + ph).sin() * 2.5;
        let py = by0 as f64 + (tg * sp * 0.6 + ph).cos() * 1.2;

        // the search beam lights what it touches
        let beam = beam_pos(tg, w);
        let lit = ((px as i64) - beam).abs() < 4;
        let a = (1.0 - turn) * if lit { 1.0 } else { 0.5 };

        let (sw, sh) = art::sprite_size(spr, 1);
        let x = px as i64 - sw / 2;
        let y = py as i64 - sh / 2;
        art::draw(g, spr, x, y, 1, true, a, Some(if lit { col.glow(0.25) } else { col.scale(0.7) }));
        if lit {
            let hp = (tg * 2.0 + k as f64 * 0.7) % 2.0;
            if hp < 0.4 {
                g.put_alpha(x + sw / 2, y - 2 - (hp * 6.0) as i64, '♥', ROSE, (0.4 - hp) * 2.0);
            }
            g.put_alpha(x + sw / 2, y - 1, '·', GOLD, 0.3);
        }
    }

    // ---- the search beam
    if turn < 0.7 {
        let bx = beam_pos(tg, w);
        for y in 2..hi - 6 {
            let dx = ((bx as f64) - (w as f64 / 2.0)) / (w as f64 / 2.0);
            let inten = (1.0 - dx.abs()).max(0.0);
            if inten > 0.04 {
                let ch = if (y as usize + (tg * 14.0) as usize) % 3 == 0 { '┃' } else { '│' };
                g.put_alpha(bx, y, ch, GOLD, 0.10 * inten);
                g.put_alpha(bx - 1, y, '·', GOLD, 0.05 * inten);
                g.put_alpha(bx + 1, y, '·', GOLD, 0.05 * inten);
            }
        }
        g.put(bx, 1, '▼', GOLD.scale(0.8));
    }

    // ---- floating "searching for…" fragments
    if turn < 0.4 {
        let mut r2 = Rng::new((tg * 3.0) as u64);
        for _ in 0..3 {
            let y = 4 + r2.i64(0, (hi as f64 * 0.3) as i64);
            let x = ((tg * (7.0 + r2.range(0.0, 9.0)) + r2.range(0.0, 100.0)) % (w as f64 + 34.0)) as i64 - 17;
            g.text_alpha(x, y, "searching for", ROSE, 0.12);
        }
    }

    // ---- the turn: the forms dissolve into falling dust
    if tl > 19.4 {
        let a = clamp01((tl - 19.4) / 2.5) * (1.0 - ease_out_cubic(turn));
        let mut r2 = Rng::new(0xD057);
        for _ in 0..90 {
            let x = r2.i64(0, w);
            let base = r2.i64(4, (hi as f64 * 0.55) as i64);
            let fall = ((tg * 1.6 + r2.range(0.0, 6.0)) % (hi as f64)) as i64;
            g.put_alpha(x, base + fall / 2, ['.', '·', ','][r2.i64(0, 2) as usize], DIM, a * 0.45);
        }
        fx::desaturate(g, turn * 0.55);
        fx::fade(g, turn * 0.25);
    }

    // ---- the query rewrites itself
    if tl > 19.0 {
        let p = clamp01((tl - 19.0) / 2.6);
        let y = 4;
        let prompt = "> world.search(";
        g.text(4, y, prompt, DIM.scale(0.9));
        let qx = 4 + Grid::measure(prompt);
        if p < 0.35 {
            // hold on "you", then glitch-scramble it away
            let hold = ease_out_cubic(p / 0.35);
            g.text_alpha(qx, y, "you", ROSE, 0.4 + 0.6 * hold);
            if ((tg * 12.0) as i64) % 7 < 2 && p > 0.15 {
                g.put_alpha(qx + 1, y, ['/', '\\', '#', '?'][((tg * 30.0) as usize) % 4], ROSE, 0.8);
            }
            if ((tg * 2.2) as i64) % 2 == 0 {
                g.put(qx + 3, y, '▌', ROSE);
            }
        } else {
            let target = "my old self";
            let sp = clamp01((p - 0.35) / 0.45);
            let shown: String = target
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if sp * target.len() as f64 > i as f64 {
                        c
                    } else {
                        ['·', '/', '#', '?', ' '][((tg * 40.0) as usize + i) % 5]
                    }
                })
                .collect();
            g.text(qx, y, &shown, GOLD.scale(0.95));
            if ((tg * 2.2) as i64) % 2 == 0 && sp < 1.0 {
                g.put(qx + Grid::measure(&shown) - 1, y, '▌', GOLD);
            }
        }
        g.text(qx + 13, y, ")", DIM.scale(0.9));
        if p >= 0.95 {
            g.text_alpha(4, y + 1, "  … 0 results. it left a long time ago.", DIM, 0.9 * (p - 0.95) / 0.05);
        }
    }

    // ---- A-Ah: a held breath of rose dust
    if tl > 18.9 && tl < 20.2 {
        let a = clamp01(1.0 - (tl - 18.9) / 1.3) * 0.5;
        let mut r2 = Rng::new(0xA4);
        for _ in 0..40 {
            let x = r2.i64(0, w);
            let y = r2.i64(2, hi - 6);
            g.put_alpha(x, y, '·', ROSE, a);
        }
    }

    // ---- the last breath: one dim heart left in the dust
    if tl > 22.0 {
        let a = clamp01((tl - 22.0) / 1.5) * (1.0 - clamp01((tl - 25.0) / 2.0));
        let y = hi / 2 - 2;
        g.put(w / 2, y, '♥', ROSE.scale(0.7 + 0.3 * a).over(g.row_bg(y as usize), a));
        if a > 0.2 {
            g.text_alpha(
                w / 2 - Grid::measure("0 found") / 2,
                y + 3,
                "0 found — 1 remembered",
                ROSE.scale(0.8),
                a * 0.8,
            );
        }
    }
}

fn beam_pos(tg: f64, w: i64) -> i64 {
    (w as f64 * (0.5 + 0.42 * (tg * 0.42).sin())) as i64
}
