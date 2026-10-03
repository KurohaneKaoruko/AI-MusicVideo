//! Shared scene UI helpers: file tree, command echo, stamps, property panels.

use crate::art;
use crate::fx;
use crate::gfx::{Color, Grid};

pub const UI_DIM: u32 = 0x55607a;
pub const UI_FG: u32 = 0xaab4c8;
pub const ROSE: u32 = 0xff6d8a;

/// A `>` command line typed at `elapsed` chars/s.
pub fn command(g: &mut Grid, x: i64, y: i64, cmd: &str, elapsed: f64, color: Color) -> i64 {
    g.put(x, y, '>', color.glow(0.2));
    let n = fx::typed(cmd, elapsed, 22.0);
    let shown: String = cmd.chars().take(n).collect();
    let end = g.text(x + 2, y, &shown, color);
    if n < cmd.chars().count() {
        if ((elapsed * 6.0) as i64) % 2 == 0 {
            let w = Grid::measure(&shown);
            g.put(x + 2 + w, y, '█', color);
        }
    }
    end
}

/// Static log line (already "printed").
#[allow(dead_code)]
pub fn log(g: &mut Grid, x: i64, y: i64, s: &str, color: Color, alpha: f64) {
    g.text_alpha(x, y, s, color, alpha);
}

/// file-tree entries with progressive reveal; `found` = how many are discovered.
pub fn file_tree(
    g: &mut Grid,
    x: i64,
    y0: i64,
    entries: &[(&'static str, &'static str)],
    found: usize,
    selected: Option<usize>,
    accent: Color,
    t: f64,
) {
    g.text(x, y0, "world/", Color::hex(UI_FG));
    for (i, (name, _desc)) in entries.iter().enumerate() {
        if i >= found {
            break;
        }
        let y = y0 + 1 + i as i64;
        let is_sel = selected == Some(i);
        let color = if is_sel {
            accent.glow(0.15 * ((t * 3.0).sin() * 0.5 + 0.5))
        } else {
            Color::hex(UI_DIM)
        };
        let branch = if i == entries.len() - 1 { "└─" } else { "├─" };
        if is_sel {
            g.put(x, y, '>', accent.glow(0.2));
        } else {
            g.put(x, y, ' ', color);
        }
        g.text(x + 2, y, branch, Color::hex(UI_DIM));
        g.text(x + 5, y, name, color);
    }
}

/// BIG "PERFECT" stamp with pop-in scale + shake.
pub fn stamp_perfect(g: &mut Grid, entered: f64, cx: i64, cy: i64, color: Color) {
    if entered < 0.0 {
        return;
    }
    let grow = crate::gfx::ease_out_back((entered / 0.35).clamp(0.0, 1.0));
    let settle = ((entered / 0.35).clamp(0.0, 1.0) - 1.0) * 2.0;
    let jx = (settle * 2.0).round() as i64;
    let label = "PERFECT";
    let w = art::big_text_width(label, 1) + 7;
    let h = 8;
    let x = cx - w / 2 + jx;
    let y = cy - h / 2;
    let border = color.glow(0.25 * grow);
    if grow < 1.0 {
        // slamming in: draw at partial width
        let vis = (w as f64 * grow) as i64;
        g.box_rounded(x, y, vis.max(2), h, border, None, None);
        return;
    }
    // let the art read through the stamp
    let a = (0.9 - (entered - 0.5) * 0.35).clamp(0.55, 0.9);
    let faint = Color::hex(0x3a4356);
    g.box_rounded(x, y, w, h, Color::lerp(faint, border, a), None, None);
    art::big_text(g, x + 4, y + 2, label, Color::lerp(faint, color, a), 1, a);
    g.put(x + w - 4, y + 2, '♥', Color::lerp(faint, Color::hex(ROSE), a * 0.9));
    // sparkles
    let mut rng = crate::gfx::Rng::new(0xC0FFEE);
    for _ in 0..4 {
        let sx = x + rng.i64(0, w - 1);
        let sy = y + rng.i64(0, h - 1);
        g.put_alpha(sx, sy, '*', color.glow(0.5), 0.35 * a);
    }
}

/// Key–value property panel lines (typed in progressively).
pub fn props(g: &mut Grid, x: i64, y0: i64, rows: &[(&str, &str, bool)], entered: f64, accent: Color) {
    for (i, (k, v, ok)) in rows.iter().enumerate() {
        let delay = i as f64 * 0.35;
        let e = entered - delay;
        if e < 0.0 {
            continue;
        }
        let y = y0 + i as i64;
        g.text(x, y, k, Color::hex(UI_DIM));
        g.text(x + 11, y, ":", Color::hex(UI_DIM));
        let vc = if *ok {
            Color::hex(0x8fe0a8)
        } else if v.starts_with("NOT") {
            Color::hex(0xc9526a)
        } else {
            Color::hex(UI_FG)
        };
        let n = fx::typed(v, e, 24.0);
        let shown: String = v.chars().take(n).collect();
        let after = g.text(x + 13, y, &shown, vc);
        if n >= v.chars().count() {
            if *ok {
                g.text(after + 1, y, "+", Color::hex(0x8fe0a8));
            } else if v.starts_with("NOT") {
                g.text(after + 1, y, "x", Color::hex(0xc9526a));
            }
        } else if ((e * 6.0) as i64) % 2 == 0 {
            g.put(after, y, '▎', accent);
        }
    }
}

/// Section headline inside content, e.g. `── you.table ──`.
pub fn headline(g: &mut Grid, y: i64, label: &str, accent: Color, dim: Color) {
    let w = Grid::measure(label);
    let x = (g.w as i64 - w) / 2;
    for rx in 0..x - 2 {
        g.put_alpha(rx, y, '─', dim, 0.35);
    }
    for rx in x + w + 2..g.w as i64 {
        g.put_alpha(rx, y, '─', dim, 0.35);
    }
    g.text_center(y, label, accent);
}
