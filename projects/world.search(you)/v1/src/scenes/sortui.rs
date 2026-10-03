//! Scene: memory.sort (0:14–0:41) — "side by side, the lovelier goes up".
//! Memory cards bubble-sorted by sweetness; a bubble rises and pops;
//! the sweetest card is saved into past/.

use super::Ctx;
use crate::art;
use crate::gfx::{clamp01, ease_in_out, ease_out_cubic, Color, Grid, Rng};
use crate::scenes::ui;

const AMBER: Color = Color::hex(0xe8c39e);
const ROSE: Color = Color::hex(0xff6d8a);
const CARD_BORDER: Color = Color::hex(0x55607a);
const DIM: Color = Color::hex(0x55607a);

const RATINGS: [f64; 4] = [3.5, 4.8, 2.1, 4.2];
/// (section-local time, slot_a, slot_b) swaps of the bubble sort (sweetest first)
const SWAPS: [(f64, usize, usize); 3] = [(2.0, 0, 1), (3.7, 2, 3), (5.4, 1, 2)];
/// local time when the big bubble pops
const POP_T: f64 = 13.19;
/// local time when the sweetest card starts flying into past/
const FLY_T: f64 = 14.54;

pub fn render(g: &mut Grid, tl: f64, _tg: f64, _ctx: &Ctx) {
    let w = g.w as i64;
    let h = g.h as i64;
    let t = tl; // section-local

    ui::headline(g, 4, "memory.sort — sweetest first", AMBER, DIM);

    // ---------------- ambient bubbles rising from the bottom
    let mut rng = Rng::new(0xB0BB1E);
    for k in 0..12 {
        let bx = 3 + rng.i64(2, (w - 6).max(4));
        let speed = rng.range(1.4, 2.6);
        let phase = rng.range(0.0, 40.0);
        let yy = (h - 3) as f64 - ((t * speed + phase) % (h as f64 - 6.0));
        let r = if k % 3 == 0 { 2 } else { 1 };
        art::bubble(g, bx + ((t * 2.0 + k as f64).sin() * 1.5).round() as i64, yy as i64, r, AMBER, 0.14);
    }

    // ---------------- the big bubble: rise … and pop
    let pop_t = POP_T;
    if t > 7.3 && t < pop_t {
        let p = clamp01((t - 7.3) / (pop_t - 7.3));
        let yy = (h - 5) as f64 - p * (h - 12) as f64;
        let xx = w / 2 + ((t * 2.6).sin() * 2.0).round() as i64;
        let r = 2 + (p * 1.6) as i64;
        let a = 0.55 + 0.25 * (t * 6.0).sin();
        draw_bubble_big(g, xx, yy as i64, r.min(3), a);
        // rising sparkle trail
        let mut rng = Rng::new((t * 10.0) as u64);
        for _ in 0..3 {
            let sx = xx + rng.i64(-4, 4);
            let sy = yy as i64 + rng.i64(2, 5);
            g.put_alpha(sx, sy, '·', AMBER, 0.3 * (1.0 - p * 0.4));
        }
    }
    if t >= pop_t && t < pop_t + 0.9 {
        let pt = t - pop_t;
        art::pop_burst(g, w / 2, (h - 16) as i64, pt * 1.2, AMBER.glow(0.4));
        if pt < 0.5 {
            g.text(w / 2 + 4, (h - 17) as i64, "POP!", AMBER.glow(0.5));
        }
    }

    // ---------------- the cards
    let cw = 15i64;
    let gap = 2i64;
    let total_w = 4 * cw + 3 * gap;
    let x0 = (w - total_w) / 2;
    let cy = h / 2 + 1;

    // slot of each card id at time t (apply swaps)
    let mut order = [0usize, 1, 2, 3];
    let mut last_swap_t = 0f64;
    let mut last_swap = None;
    for (st, a, b) in SWAPS {
        if t >= st {
            order.swap(a, b);
            last_swap_t = st;
            last_swap = Some((a, b));
        }
    }
    let mut slot_of = [0usize; 4];
    for (slot, &id) in order.iter().enumerate() {
        slot_of[id] = slot;
    }

    // highlight pair before a swap
    let mut hl_pair: Option<(usize, usize)> = None;
    for (st, a, b) in SWAPS {
        if t > st - 1.2 && t <= st {
            hl_pair = Some((a.min(b), a.max(b)));
        }
    }
    // winner glow in the last phase
    let winner_phase = t > FLY_T;
    let winner_id = 1usize;

    for id in 0..4 {
        // current animated position: if this card just swapped, lerp from old slot
        let slot = slot_of[id];
        let mut disp = slot as f64;
        let mut dy = 0i64;
        if let Some((a, b)) = last_swap {
            if t - last_swap_t < 0.5 && (slot == a || slot == b) {
                let from = if slot == a { b } else { a };
                let p = ease_in_out((t - last_swap_t) / 0.5);
                disp = from as f64 + (slot as f64 - from as f64) * p;
                // the lovelier one goes UP — arc
                dy = -((std::f64::consts::PI * p).sin() * 2.5).round() as i64;
            }
        }
        let cx = x0 + (disp.round() as i64) * (cw + gap);
        let lift = if winner_phase && id == winner_id { -2 } else { 0 };

        // appearance stagger
        let born = 0.24 + id as f64 * 0.35;
        let alpha = clamp01((t - born) / 0.6);
        if alpha <= 0.0 {
            continue;
        }

        let highlighted = hl_pair.map(|(a, b)| slot == a || slot == b).unwrap_or(false);
        let col = if id == winner_id && winner_phase {
            AMBER.glow(0.15)
        } else if highlighted {
            AMBER
        } else {
            CARD_BORDER
        };

        let cy_card = cy + dy + lift;
        card(
            g,
            cx,
            cy_card,
            cw,
            7,
            id,
            RATINGS[id],
            col,
            alpha,
            t,
            highlighted || (winner_phase && id == winner_id),
        );

        // comparison meters in the final phase
        if t > 20.6 && !winner_phase_inserted(t) {
            let pct = (RATINGS[id] / 5.0 * 100.0) as i64;
            let label = format!("{pct}%");
            let lx = cx + cw / 2 - 2;
            g.text(lx, cy_card - 2, &label, if id == winner_id { ROSE } else { DIM });
        }
    }

    // ---------------- insert the sweetest into past/
    if winner_phase_inserted(t) {
        let ins_x = w - 22;
        let ins_y = 8;
        // flight path of winner card
        let fly_t = FLY_T;
        if t < fly_t + 1.1 {
            let p = ease_out_cubic(clamp01((t - fly_t) / 1.1));
            let fx_ = (x0 as f64) + ((ins_x - x0) as f64 * p);
            let fy = (cy as f64) - (std::f64::consts::PI * p).sin() * 8.0;
            card(
                g,
                fx_.round() as i64,
                fy.round() as i64,
                cw,
                7,
                winner_id,
                RATINGS[winner_id],
                AMBER.glow(0.2),
                1.0,
                t,
                true,
            );
        }
        // slot
        let slotw = 17;
        g.box_rounded(ins_x - 1, ins_y - 1, slotw + 2, 8, DIM, None, Some(("INSERT", DIM)));
        g.text(ins_x, ins_y, "to: past/", DIM);
        let prog = clamp01((t - fly_t - 0.8) / 2.6);
        let filled = (prog * 12.0) as i64;
        let mut bar = String::new();
        for k in 0..12 {
            bar.push(if k < filled { '▓' } else { '░' });
        }
        g.text(ins_x, ins_y + 2, &bar, AMBER);
        g.text(ins_x, ins_y + 3, "mem_0417.sav", Color::hex(0x8a93a6));
        if prog >= 1.0 {
            g.text(ins_x, ins_y + 4, "SAVED ✓", Color::hex(0x8fe0a8));
        }
        // remaining cards dim out
        if t > fly_t + 1.1 {
            for id in 0..4 {
                if id == winner_id {
                    continue;
                }
                let slot = slot_of[id];
                let cx = x0 + (slot as i64) * (cw + gap);
                card(g, cx, cy, cw, 7, id, RATINGS[id], DIM, 0.45, t, false);
            }
        }
        // save stamp
        if t > 21.3 {
            let s = "★ SAVED";
            g.text(ins_x + 2, ins_y + 5, s, ROSE.glow(0.2));
        }
    }
}

fn winner_phase_inserted(t: f64) -> bool {
    t > FLY_T
}

#[allow(clippy::too_many_arguments)]
fn card(
    g: &mut Grid,
    x: i64,
    y: i64,
    w: i64,
    h: i64,
    id: usize,
    rating: f64,
    border: Color,
    alpha: f64,
    t: f64,
    hl: bool,
) {
    if alpha <= 0.02 {
        return;
    }
    let dim = Color::hex(0x8a93a6);
    // box
    let (ix, iy, iw, ih) = g.box_rounded(x, y, w, h, border.over(g.row_bg(y as usize), alpha), None, None);
    let _ = (ix, iy, iw, ih);
    if alpha < 1.0 {
        // dim whole card text
        let title = format!("MEM #00{}", 41 + id);
        g.text_alpha(x + 3, y + 1, &title, dim, alpha * 0.8);
        let hearts: String = std::iter::repeat('♥').take(rating.round() as usize).collect();
        g.text_alpha(x + 3, y + 3, &hearts, ROSE, alpha);
        let m = meter(rating);
        g.text_alpha(x + 3, y + 4, &m, AMBER, alpha * 0.9);
        return;
    }
    let title = format!("MEM #00{}", 41 + id);
    g.text(x + 3, y + 1, &title, dim);
    let hearts: String = std::iter::repeat('♥').take(rating.round() as usize).collect();
    g.text(x + 3, y + 3, &hearts, ROSE);
    g.text(x + 3, y + 4, &meter(rating), AMBER);
    if hl {
        // shimmer along the top edge
        let ph = ((t * 8.0) % (w as f64)) as i64;
        g.put(x + 1 + ph.clamp(0, w - 2), y, '·', border.glow(0.5));
    }
}

fn meter(r: f64) -> String {
    let filled = (r / 5.0 * 8.0).round() as usize;
    let mut s = String::new();
    for k in 0..8 {
        s.push(if k < filled { '▓' } else { '░' });
    }
    s
}

/// bigger fancier bubble (3-radius ring)
fn draw_bubble_big(g: &mut Grid, cx: i64, cy: i64, r: i64, alpha: f64) {
    if r <= 2 {
        art::bubble(g, cx, cy, 2, AMBER, alpha);
        return;
    }
    let c = AMBER;
    let pts: [(i64, i64, char); 15] = [
        (-3, -2, ','), (-2, -2, '-'), (-1, -2, '.'), (1, -2, '.'), (2, -2, '.'), (3, -2, ','),
        (-4, -1, '/'), (3, -1, '\\'),
        (-4, 0, '('), (3, 0, ')'),
        (-4, 1, '('), (3, 1, ')'),
        (-3, 2, '`'), (0, 2, '-'), (3, 2, '´'),
    ];
    for (dx, dy, ch) in pts {
        g.put_alpha(cx + dx, cy + dy, ch, c, alpha);
    }
    // shine
    g.put_alpha(cx - 2, cy - 1, '\'', c, alpha * 0.9);
    g.put_alpha(cx - 1, cy - 1, '·', c, alpha * 0.6);
}
