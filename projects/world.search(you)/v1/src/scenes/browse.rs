//! Scene: the world.search file-manager frame (table / eggplant / cat / steak).
//! Every object is a "candidate file" opened from world/.

use super::Ctx;
use crate::art;
use crate::gfx::{clamp01, Color, Grid, Rng};
use crate::scenes::ui;

#[derive(Clone, Copy, PartialEq)]
pub enum Obj {
    Table,
    Eggplant,
    Cat,
    Steak,
}

pub const ENTRIES: [(&str, &str); 8] = [
    ("you.table", "4 legs, one wobbly"),
    ("you.eggplant", "purple, scratched"),
    ("you.cat", "bluepoint, no purr"),
    ("you.steak", "medium rare"),
    ("you.flower", "rare bloom"),
    ("you.human", "warm hands"),
    ("you.dog", "ruff ruff ruff"),
    ("you.tomato", "full of juice"),
];

const WOOD: Color = Color::hex(0xd9a066);
const PURPLE: Color = Color::hex(0x9a6dd7);
const BLUEPT: Color = Color::hex(0x9fb2cf);
const SEAR: Color = Color::hex(0xc9a86a);
const DIM: Color = Color::hex(0x55607a);
const ROSE: Color = Color::hex(0xff6d8a);

fn obj_meta(obj: Obj) -> (usize, &'static str, Color, f64) {
    // (entry index, file name, accent, section-local stamp time)
    match obj {
        Obj::Table => (0, "you.table", WOOD, 10.25),
        Obj::Eggplant => (1, "you.eggplant", PURPLE, 10.35),
        Obj::Cat => (2, "you.cat", BLUEPT, -1.0),
        Obj::Steak => (3, "you.steak", SEAR, -1.0),
    }
}

pub fn render(g: &mut Grid, tl: f64, tg: f64, ctx: &Ctx, obj: Obj) {
    let (idx, name, accent, stamp_t) = obj_meta(obj);
    let w = g.w as i64;
    let h = g.h as i64;

    // ---- outer content frame (leaves room for the lyric zone below)
    let box_h = h - 16;
    let frame = DIM.over(g.row_bg(3), 0.85);
    g.box_rounded(1, 3, w - 2, box_h, frame, None, Some(("world.search — results", DIM)));

    // ---- column divider between the file tree and the stage
    let div_x = 26;
    for y in 4..(3 + box_h) {
        g.put_alpha(div_x, y, '│', DIM, 0.45);
    }
    g.put(div_x, 3, '┬', DIM.scale(0.9));
    g.put(div_x, 2 + box_h, '┴', DIM.scale(0.9));

    // ---- command echo (inside the stage pane)
    ui::command(g, div_x + 2, 4, &format!("world.search(you) => match: {name}"), tl, accent);

    // ---- file tree (progressive discovery)
    ui::file_tree(g, 3, 6, &ENTRIES, idx + 1, Some(idx), accent, tg);

    // ---- properties panel (right)
    let px = w - 28;
    let py = 7;
    let entered = tl - 0.6;
    match obj {
        Obj::Table => ui::props(
            g, px, py,
            &[("height", "0.72m (elbow)", true), ("legs", "4 (1 stubby)", true), ("wobble", "cozy", true), ("love", "♥♥♥♥♥", false)],
            entered, accent,
        ),
        Obj::Eggplant => ui::props(
            g, px, py,
            &[("color", "purple", true), ("role", "friend", true), ("marks", "3 scratchy", true), ("love", "♥♥♥♥♥", false)],
            entered, accent,
        ),
        Obj::Cat => ui::props(
            g, px, py,
            &[("coat", "bluepoint", true), ("meow", "a lot", true), ("hiss", "a lot", true), ("purr", "NOT FOUND", false), ("love", "♥♥♥♥♥", false)],
            entered, accent,
        ),
        Obj::Steak => ui::props(
            g, px, py,
            &[("cut", "fillet mignon", true), ("sauce", "mushroom", true), ("sear", "golden", true), ("doneness", "medium rare", true), ("love", "♥♥♥♥♥", false)],
            entered, accent,
        ),
    }

    // ---- the object itself, centered between tree and properties
    let cx = (div_x + px + 4) / 2;
    let cy = 6 + (box_h - 4) / 2;
    match obj {
        Obj::Table => table(g, tl, tg, ctx, cx, cy, accent),
        Obj::Eggplant => eggplant(g, tl, tg, ctx, cx, cy, accent),
        Obj::Cat => cat(g, tl, tg, ctx, cx, cy, accent),
        Obj::Steak => steak(g, tl, tg, ctx, cx, cy, accent),
    }

    // ---- PERFECT stamp (section-local time)
    if stamp_t > 0.0 && tl >= stamp_t {
        ui::stamp_perfect(g, tl - stamp_t, cx, cy, accent);
    }
}

// ------------------------------------------------------------------ table ---

fn table(g: &mut Grid, tl: f64, tg: f64, _ctx: &Ctx, cx: i64, cy: i64, accent: Color) {
    // assemble during the first line (section-local 0.05 → 1.65)
    let born = clamp01((tl - 0.05) / 1.6);
    let wob_line = tl > 6.81 && tl < 10.1;
    let ruler_line = tl > 3.24 && tl < 6.81;
    let wob = if wob_line { (tg * 5.2).sin() } else { (tg * 1.1).sin() * 0.35 };

    let (bw, bh) = art::sprite_size(&art::TABLE_TOP, 2);
    let ox = cx - bw / 2;
    let oy = cy - bh - 1;
    let body_dy = wob.round() as i64;

    // scattered assemble: columns drift in from noise
    let mut rng = Rng::new(0x7AB1E);
    let scatter = (1.0 - born) * 9.0;
    let a = (born * 1.6).clamp(0.0, 1.0);

    for (ry, row) in art::TABLE_TOP.rows.iter().enumerate() {
        let y = oy + ry as i64 + body_dy;
        for (rx, ch) in row.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let col = art::TABLE_TOP
                .pal
                .iter()
                .find(|(k, _)| *k == ch)
                .map(|(_, hx)| Color::hex(*hx))
                .unwrap_or(accent);
            let jx = (rng.f64() * scatter * 2.0 - scatter) as i64;
            let x = ox + rx as i64 * 2 + jx;
            if a < 1.0 {
                g.put_alpha(x, y, ch, col, a);
            } else {
                g.put(x, y, ch, col);
                g.put(x + 1, y, ch, col);
            }
        }
    }

    // legs: 3 long + 1 stubby (the wobbly one)
    let leg_xs = [ox + 6, ox + 15, ox + 24, ox + 33];
    let leg_len: [i64; 4] = [3, 3, 3, 1];
    let leg_start: [i64; 4] = [0, 0, 0, 2]; // stubby starts lower
    let leg_col = Color::hex(0x8a6242);
    for (i, lx) in leg_xs.iter().enumerate() {
        let phase: f64 = [0.0, 0.9, 1.7, 2.6][i];
        let extra = if i == 3 { wob * 2.2 } else { wob * 0.6 + phase.sin() * 0.4 };
        for k in 0..leg_len[i] {
            let y = oy + bh + leg_start[i] + k + body_dy + (extra * k as f64 * 0.3).round() as i64;
            if i == 3 {
                // the odd stubby leg: thick, short
                g.put(*lx, y, '█', leg_col.glow(0.15));
                g.put(*lx + 1, y, '█', leg_col.glow(0.15));
            } else {
                g.put(*lx, y, '│', leg_col);
                g.put(*lx + 1, y, '│', leg_col);
            }
        }
        // floor line
        let fy = oy + bh + leg_start[i] + leg_len[i] + body_dy;
        g.put_alpha(*lx - 1, fy, '·', leg_col.scale(0.7), 0.6);
        g.put_alpha(*lx + 2, fy, '·', leg_col.scale(0.7), 0.6);
    }

    // elbow-height ruler during "you must be at the height of my elbows"
    if ruler_line {
        let rx = ox + bw + 2;
        let top = oy - 1;
        let bot = oy + bh + 3;
        g.vline(rx, top, bot, '│', DIM.scale(0.9));
        for y in top..=bot {
            if (y - top) % 2 == 0 {
                g.put(rx + 1, y, '·', DIM);
            }
        }
        // elbow marker at tabletop height
        g.put(rx + 1, oy + body_dy, '<', accent.glow(0.3));
        g.text(rx + 3, top, "elbow", accent);
        g.text(rx + 3, top + 1, "0.72m", accent.scale(0.85));
    }

    // creak while wobbling
    if wob_line && (tg * 5.2).sin() > 0.86 {
        g.text(ox + bw / 2 - 1, oy + bh + 4, "⁑", accent);
    }
}

// --------------------------------------------------------------- eggplant ---

fn eggplant(g: &mut Grid, tl: f64, tg: f64, ctx: &Ctx, cx: i64, cy: i64, _accent: Color) {
    let sway = (tg * 1.4).sin() * 2.0;
    let ox = cx - 12 + sway.round() as i64;
    let oy = cy - 5;
    let bounce = ((tg * 1.4).sin() * 0.5 + 0.5) * -1.0;
    art::draw(g, &art::EGGPLANT, ox, oy, 2, true, 1.0, None);

    // purple friend face (line 2: local 3.37 → 6.76)
    if tl > 3.37 && tl < 6.76 {
        let fx = ox + 3 * 2;
        g.put(fx, oy + 6, '^', Color::hex(0x2b1b45));
        g.put(fx + 6, oy + 6, '^', Color::hex(0x2b1b45));
        g.put(fx + 3, oy + 8, 'u', Color::hex(0x2b1b45));
    }

    // scratchy nail marks (line 3, local ≥ 6.76): one per beat + spark
    if tl > 6.76 {
        let marks: [(i64, i64, char); 4] = [(4, 5, '/'), (13, 6, '\\'), (6, 7, '/'), (12, 5, '\\')];
        let n_marks = marks.len() as i64;
        let local = tl - 6.76;
        let shown = ((local / 0.85) as i64 + 1).min(n_marks);
        for (i, (mx, my, ch)) in marks.iter().enumerate() {
            if (i as i64) < shown {
                let x = ox + mx;
                let y = oy + my;
                g.put(x, y, *ch, Color::hex(0xe8a7c3));
                // sparkle on a fresh mark
                let age = local - i as f64 * 0.85;
                if age < 0.5 {
                    g.put(x + 1, y - 1, '*', Color::hex(0xe8a7c3).glow(0.5));
                }
            }
        }
    }

    // gentle drop shadow
    let sh_y = oy + 10 + bounce as i64;
    for dx in -4..5 {
        g.put_alpha(ox + 10 + dx, sh_y, if dx % 2 == 0 { '·' } else { ' ' }, PURPLE.scale(0.5), 0.25);
    }
    let _ = ctx;
}

// --------------------------------------------------------------------- cat ---

fn cat(g: &mut Grid, tl: f64, tg: f64, ctx: &Ctx, cx: i64, cy: i64, accent: Color) {
    let beat = ctx.beats.pulse(tg);
    let ox = cx - 12;
    let oy = cy - 5;

    let meow_line = tl > 6.87 && tl < 9.69;
    let hiss_line = tl > 9.69 && tl < 11.5;
    let nopurr_line = tl > 11.5;

    let spr = if meow_line {
        &art::CAT_MEOW
    } else if hiss_line {
        &art::CAT_HISS
    } else if nopurr_line {
        &art::CAT_SLEEP
    } else if ((tg * 1.6) as i64) % 2 == 0 {
        &art::CAT_A
    } else {
        &art::CAT_B
    };
    let jx = if hiss_line { ((tg * 9.0) as i64) % 3 - 1 } else { 0 };
    art::draw(g, spr, ox + jx, oy, 2, true, 1.0, None);

    // meow bubbles on beats
    if meow_line {
        let k = ctx.beats.beat_index(tg);
        let bt = ctx.beats.beat_time(k);
        let age = tg - bt;
        let bx = ox + 24;
        let by = oy + 1 - (age * 3.0) as i64;
        if age < 1.2 {
            g.text(bx, by, "meow~", accent.glow(0.2 * (1.0 - age)));
        }
    }
    // hiss sparks
    if hiss_line {
        let mut rng = Rng::new((tg * 30.0) as u64);
        for _ in 0..4 {
            let hx = ox + 20 + rng.i64(0, 8);
            let hy = oy + 1 + rng.i64(0, 3);
            g.put_alpha(hx, hy, ['s', '!', '·'][rng.i64(0, 2) as usize], accent, 0.7);
        }
        g.text(ox + 26, oy - 1, "sss!!", accent);
    }
    // never purrs: z z + red NOT FOUND note
    if nopurr_line {
        let zz = ((tg * 1.5) % 3.0) as i64;
        g.put_alpha(ox + 26, oy - 1 - zz, 'z', accent, 0.6);
        g.put_alpha(ox + 28, oy - 3 - zz, 'z', accent, 0.35);
        let note = "purr: NOT FOUND";
        let a = clamp01((tl - 11.71) * 2.0);
        g.text_alpha(ox + 2, oy + 11, note, Color::hex(0xc9526a), a * (0.7 + 0.3 * beat));
    }

    // idle heart
    let hp = (tg % 3.5) / 3.5;
    g.put_alpha(ox - 3, oy + 6 - (hp * 4.0) as i64, '♥', ROSE, 0.4 * (1.0 - hp));
}

// ------------------------------------------------------------------- steak ---

fn steak(g: &mut Grid, tl: f64, tg: f64, _ctx: &Ctx, cx: i64, cy: i64, accent: Color) {
    let (w, hgt) = art::sprite_size(&art::STEAK, 2);
    let ox = cx - w / 2 - 4;
    let oy = cy - hgt / 2 - 1;
    let sizzle = 0.8 + 0.2 * (tg * 7.0).sin();
    art::draw(g, &art::STEAK, ox, oy, 2, true, sizzle, None);

    // sizzle sparks above
    let mut rng = Rng::new((tg * 24.0) as u64 ^ 0x57EA1);
    for _ in 0..6 {
        let life = rng.f64();
        let sx = ox + rng.i64(4, w - 6);
        let sy = oy - (life * 4.0) as i64;
        let ch = ['·', '\'', '°', '*'][rng.i64(0, 3) as usize];
        g.put_alpha(sx, sy, ch, SEAR.glow(0.3), (1.0 - life) * 0.7);
    }
    // steam
    for c in 0..3 {
        let base_x = ox + 8 + c * (w / 4);
        for k in 0..5 {
            let yy = oy - 1 - k - ((tg * 4.0 + c as f64) % 2.0) as i64;
            let xx = base_x + ((tg * 2.0 + k as f64 * 0.8).sin() * 2.0).round() as i64;
            g.put_alpha(xx, yy, '~', Color::hex(0x8a93a6), 0.28 - k as f64 * 0.04);
        }
    }

    // sear emphasis during "browned on the outside" (local 7.11 → 10.02)
    if tl > 7.11 && tl < 10.02 {
        let pulse = (tg * 4.0).sin() * 0.5 + 0.5;
        for (ry, row) in art::STEAK.rows.iter().enumerate() {
            for (rx, ch) in row.chars().enumerate() {
                if ch == '≡' {
                    let a = 0.5 + 0.5 * pulse;
                    g.put_alpha(ox + rx as i64 * 2, oy + ry as i64, ch, Color::hex(0x6b3b28).glow(0.3), a);
                    g.put_alpha(ox + rx as i64 * 2 + 1, oy + ry as i64, ch, Color::hex(0x6b3b28).glow(0.3), a);
                }
            }
        }
    }

    // cross-section inspection panel (replaces the props while it shows)
    if tl > 9.56 {
        let a = clamp01((tl - 9.56) * 2.0);
        let pxw = 26;
        let px = g.w as i64 - 28;
        let py = 6;
        let bg = g.row_bg(py as usize);
        let border = SEAR.over(bg, a);
        g.box_rounded(px, py, pxw, 9, border, Some(bg), Some(("cross-section", DIM.scale(0.9))));
        // outer brown ring, inner pink
        for yy in 2..6 {
            for xx in 1..pxw - 1 {
                let edge = yy == 2 || yy == 5 || xx == 1 || xx == pxw - 2;
                let ch = if edge { '▓' } else { '▒' };
                let col = if edge { Color::hex(0x8a5a3a) } else { Color::hex(0xe8a3a3) };
                g.put_alpha(px + xx, py + yy, ch, col, a * 0.8);
            }
        }
        g.text_alpha(px + 2, py + 6, "55C medium rare", accent, a);
        // doneness gauge
        let gy = py + 7;
        for k in 0..14 {
            let col = if k < 5 {
                Color::hex(0xe86a5e)
            } else if k < 9 {
                Color::hex(0xe8a3a3)
            } else {
                Color::hex(0x8a5a3a)
            };
            g.put_alpha(px + 2 + k, gy, '▓', col, a * 0.8);
        }
        g.put_alpha(px + 6, gy, '▲', Color::rgb(255, 255, 255), a);
    }
}
