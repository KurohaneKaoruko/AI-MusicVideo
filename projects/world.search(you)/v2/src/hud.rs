//! The only persistent chrome left: a diegetic "search program" readout.
//!
//! There is no player UI in the film. What stays on screen is part of the
//! fiction — the query being run, the index it is grinding through, and a
//! hairline of progress along the bottom. Scenes may switch it off entirely
//! (see `scenes::chrome_at`), and emotional scenes do.

use crate::beats::BeatGrid;
use crate::gfx::{clamp01, pal, Color, Grid};
use crate::scenes::section_at;

const HAIR: Color = Color::hex(0x1a2230);

pub fn render(g: &mut Grid, t: f64, beats: &BeatGrid, dur: f64) {
    let w = g.w as i64;
    let h = g.h as i64;

    // ---- bottom hairline: the read head of the whole search
    let prog = clamp01(t / dur.max(1.0));
    let bx = 3i64;
    let bw = w - 6;
    g.hline(bx, bx + bw - 1, h - 1, '─', HAIR);
    let lit = (bw as f64 * prog) as i64;
    for k in 0..lit {
        let x = bx + k;
        // section colour leaks into the read head
        let sec = section_at(t);
        g.put(x, h - 1, '─', sec.accent.scale(0.42));
    }
    g.put(bx + lit, h - 1, '▌', pal::c(pal::ROSE).glow(0.25));
    // one pixel-column of light rising off the read head
    for k in 1..3 {
        g.put_alpha(bx + lit, h - 1 - k, '│', pal::c(pal::ROSE), 0.30 / k as f64);
    }

    if !crate::scenes::chrome_at(t) {
        return;
    }

    // ---- top-left: the query the program is running
    let q = "world.search(you);";
    g.text(3, 0, q, pal::c(pal::ROSE).scale(0.88));
    let qw = Grid::measure(q);
    if (t * 2.0) as i64 % 2 == 0 {
        g.put(3 + qw, 0, '▌', pal::c(pal::ROSE_PALE));
    }

    // ---- top-right: wall clock of the run
    let time_s = fmt_time(t);
    let dur_s = fmt_time(dur);
    g.text_right(w - 3, 0, &format!("{time_s} / {dur_s}"), pal::c(pal::SLATE).scale(0.72));

    // ---- the index counter, grinding (deterministic, tied to the beat grid)
    let total: u64 = 8_141_596_233;
    let frac = clamp01(t / dur.max(1.0));
    let val = total - ((total as f64 * frac.powf(0.72)) as u64);
    let bi = beats.beat_index(t);
    let spin = ['|', '/', '─', '\\'][bi.rem_euclid(4) as usize];
    g.text(3, 1, &format!("{spin} indexing {val:>13}"), pal::c(pal::DIM).glow(0.05) .scale(0.95));
    // a faint beat tick on the hairline
    if beats.downbeat(t) {
        g.put(bx + lit, h - 1, '┃', pal::c(pal::ROSE_PALE));
    }
}

pub fn fmt_time(t: f64) -> String {
    let t = t.max(0.0);
    let m = (t / 60.0).floor() as i64;
    let s = (t - m as f64 * 60.0).floor() as i64;
    format!("{m}:{s:02}")
}

/// Interactive-only: the frame-cap badge, shown only when a cap is set.
pub fn fps_badge(g: &mut Grid, cap: f64) {
    if cap <= 0.0 {
        return;
    }
    let w = g.w as i64;
    g.text_right(w - 3, 1, &format!("cap {cap:.0}fps"), pal::c(pal::DIM).scale(0.85));
}

/// Interactive-only: the terminal is too small to hold the film.
///
/// The grid is a fixed 192x45 and is written with absolute cursor moves, so a
/// smaller window folds every row into the one below it. Drawing nothing and
/// saying why beats drawing garbage. The card degrades in three tiers so it
/// stays legible even in a very narrow window.
pub fn too_small(g: &mut Grid, cols: usize, rows: usize) {
    // Only the region the window actually shows may be written to.
    let vw = (cols as i64).min(g.w as i64);
    let vh = (rows as i64).min(g.h as i64);
    for c in g.cells.iter_mut() {
        c.ch = ' ';
        c.fg = pal::c(pal::BG0);
        c.bg = pal::c(pal::BG0);
    }

    // (tier, lines) — the first tier whose widest line and line count fit wins.
    let full = vec![
        ("terminal too small".to_string(), pal::c(pal::ROSE)),
        (
            format!("this film is a fixed {} x {} character grid", crate::pix::COLS, crate::pix::ROWS),
            pal::c(pal::SLATE),
        ),
        (format!("window now: {cols} x {rows}"), pal::c(pal::MID)),
        ("maximise the window, or shrink the font".to_string(), pal::c(pal::BRIGHT)),
        ("need at least 192 columns by 45 rows".to_string(), pal::c(pal::MID)),
    ];
    let mid = vec![
        ("terminal too small".to_string(), pal::c(pal::ROSE)),
        ("need 192x45".to_string(), pal::c(pal::BRIGHT)),
        (format!("now {cols}x{rows}"), pal::c(pal::DIM)),
    ];
    let tiny = vec![("too small".to_string(), pal::c(pal::ROSE))];

    let screen = [full, mid, tiny]
        .into_iter()
        .find(|lines| {
            let widest = lines.iter().map(|(s, _)| Grid::measure(s)).max().unwrap_or(0);
            widest <= vw - 2 && lines.len() as i64 <= vh - 1
        })
        .unwrap_or_else(|| vec![("!".to_string(), pal::c(pal::ROSE))]);

    let n = screen.len();
    let y0 = ((vh - n as i64 - 1) / 2).max(0);
    for (i, (s, c)) in screen.iter().enumerate() {
        if i as i64 >= vh {
            break;
        }
        let x = ((vw - Grid::measure(s)) / 2).max(0);
        g.text(x, y0 + i as i64, s, *c);
    }
    let hint = "[q] quit";
    if y0 + n as i64 + 1 < vh {
        let x = ((vw - Grid::measure(hint)) / 2).max(0);
        g.text(x, y0 + n as i64 + 1, hint, pal::c(pal::ROSE_PALE));
    }
}

/// Interactive-only: the help card.
pub fn help(g: &mut Grid) {
    let w = g.w as i64;
    let h = g.h as i64;
    let lines: [(&str, Color); 9] = [
        ("world.search (you) ;", pal::c(pal::ROSE)),
        ("a terminal MV for Mili — momocashew / Yamato Kasai", pal::c(pal::MID)),
        ("", pal::c(pal::MID)),
        ("[space]  pause / resume", pal::c(pal::BRIGHT)),
        ("[← →]    seek 5 seconds", pal::c(pal::BRIGHT)),
        ("[+ -]    volume", pal::c(pal::BRIGHT)),
        ("[f]      frame cap: off / 60 / 30", pal::c(pal::BRIGHT)),
        ("[h]      close this help", pal::c(pal::BRIGHT)),
        ("[q]      quit", pal::c(pal::BRIGHT)),
    ];
    let bw = 54;
    let bh = lines.len() as i64 + 3;
    let (ix, iy, _, _) = g.box_rounded(
        (w - bw) / 2,
        (h - bh) / 2,
        bw,
        bh,
        pal::c(pal::ROSE).scale(0.8),
        Some(pal::c(pal::BG1)),
        None,
    );
    for (i, (s, c)) in lines.iter().enumerate() {
        if s.is_empty() {
            continue;
        }
        g.text(ix + 3, iy + 1 + i as i64, s, *c);
    }
}

pub fn paused_overlay(g: &mut Grid) {
    let w = g.w as i64;
    let h = g.h as i64;
    for c in g.cells.iter_mut() {
        c.fg = c.fg.over(c.bg, 0.30);
    }
    let s = "▌▌  paused — [space] to resume";
    let bw = Grid::measure(s) + 6;
    g.box_rounded((w - bw) / 2, h / 2 - 1, bw, 3, pal::c(pal::MID), Some(pal::c(pal::BG1)), None);
    g.text_center(h / 2, s, pal::c(pal::BRIGHT));
}

pub fn loading(g: &mut Grid, p: f64) {
    let w = g.w as i64;
    let h = g.h as i64;
    g.text_center(h / 2 - 2, "world.search (you) ;", pal::c(pal::ROSE));
    g.text_center(h / 2, &format!("indexing the world… {:>3.0}%", p * 100.0), pal::c(pal::SLATE));
    let bw = 46.min(w - 12);
    let bx = (w - bw) / 2;
    g.hline(bx, bx + bw - 1, h / 2 + 2, '░', pal::c(pal::DIM));
    let filled = (bw as f64 * p) as i64;
    for k in 0..filled {
        g.put(bx + k, h / 2 + 2, '▓', pal::c(pal::ROSE));
    }
}
