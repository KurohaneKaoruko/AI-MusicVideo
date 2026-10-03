//! Persistent chrome: title bar, beat dots, section progress, restoration meter.

use crate::beats::BeatGrid;
use crate::gfx::{pal, Color, Grid};
use crate::scenes::section_at;

pub fn render(g: &mut Grid, t: f64, beats: &BeatGrid, dur: f64) {
    let sec = section_at(t);
    let accent = sec.color();

    top_bar(g, t, beats, dur, sec.label, accent);
    beat_dots(g, t, beats, accent);
    section_bar(g, t, dur);
    restoration_meter(g, t, dur);
    bottom_line(g, t);
}

fn top_bar(g: &mut Grid, t: f64, beats: &BeatGrid, _dur: f64, label: &str, accent: Color) {
    let w = g.w as i64;
    // thin rule under the bar zone
    for x in 0..w {
        g.put(x, 1, '─', pal::c(pal::DIM).scale(0.8));
    }

    // left: session id
    let mut x = g.text(2, 0, "EDEN://", pal::c(pal::MID));
    x = g.text(x, 0, "NotToNotice", pal::c(pal::BRIGHT));
    g.text(x, 0, "();", pal::c(pal::ICE));

    // center: scene label with play marker
    let tag = format!("» {}", label);
    let tx = (w - Grid::measure(&tag)) / 2;
    g.text(tx, 0, &tag, accent.glow(0.2 + 0.25 * beats.pulse(t)));

    // right: clock
    let mm = (t / 60.0) as i64;
    let ss = t % 60.0;
    let clock = format!("T+{:02}:{:05.2}", mm, ss);
    g.text_right(w - 2, 0, &clock, pal::c(pal::MID));
}

fn beat_dots(g: &mut Grid, t: f64, beats: &BeatGrid, accent: Color) {
    let y = 2;
    let idx = beats.beat_index(t);
    let phase = beats.beat_phase(t);
    let in_bar = idx.rem_euclid(4);
    let bar = idx.div_euclid(4);
    // four dots, current one glowing
    for k in 0..4 {
        let x = 3 + k * 2;
        let col = if k as i64 == in_bar {
            accent.glow(0.45 * (1.0 - phase))
        } else {
            pal::c(pal::DIM)
        };
        let ch = if k as i64 == in_bar { '◆' } else { '·' };
        g.put(x, y, ch, col);
    }
    let label = format!("BAR {:04}", bar.max(0));
    g.text(12, y, &label, pal::c(pal::DIM));
}

fn section_bar(g: &mut Grid, t: f64, dur: f64) {
    let secs = crate::scenes::SECTIONS;
    let w = g.w as i64;
    let x0 = 2i64;
    let x1 = w - 3;
    let bw = x1 - x0;
    let y = 40;

    // total time -> x position
    let to_x = |tt: f64| x0 + ((tt / dur.max(1.0)) * bw as f64).round() as i64;

    // baseline
    for x in x0..=x1 {
        g.put(x, y, '─', pal::c(pal::DIM).scale(0.75));
    }

    // per-section tinted ranges (thin, on the same row)
    for (i, s) in secs.iter().enumerate() {
        let a = to_x(s.t);
        let b = to_x(secs.get(i + 1).map(|s| s.t).unwrap_or(dur)) - 1;
        let b = b.min(x1);
        if b < a {
            continue;
        }
        let passed = t >= s.t;
        let col = s.color().scale(if passed { 0.95 } else { 0.42 });
        for x in a..=b {
            g.put(x, y, '━', col);
        }
        // separator tick + label on first sections
        g.put(a, y - 1, '┊', pal::c(pal::DIM));
        if (b - a) > 8 && i % 2 == 0 {
            let mm = (s.t / 60.0) as i64;
            let ss = (s.t % 60.0) as i64;
            let lab = format!("{}:{:02}", mm, ss);
            g.text(a.min(x1 - 4), y + 1, &lab, pal::c(pal::DIM));
        }
    }

    // playhead
    let px = to_x(t).clamp(x0, x1);
    g.put(px, y, '█', Color::rgb(244, 248, 255));
    g.put(px, y - 1, '▼', pal::c(pal::WHITE).glow(0.3));
}

/// MIND RESTORATION meter — fills across the whole song; the "E×P" of the piece.
fn restoration_meter(g: &mut Grid, t: f64, dur: f64) {
    let p = (t / dur.max(1.0)).clamp(0.0, 1.0);
    let y = 42;
    let w = g.w as i64;
    let label = "MIND RESTORATION · 精神再生";
    let lw = Grid::measure(label);
    let bx = (w - (lw + 2 + 66 + 8)) / 2 + lw + 2;

    g.text(2 + ((w - 4 - lw - 74) / 2).max(2), y, label, pal::c(pal::DIM));

    let meter_w = 60i64;
    let fill = ((meter_w as f64) * p).round() as i64;
    let gold = pal::c(pal::GOLD);
    for k in 0..meter_w {
        let x = bx + k;
        let col = if k < fill {
            if k > meter_w as i64 - 4 {
                gold.glow(0.5)
            } else {
                gold.scale(0.9)
            }
        } else {
            pal::c(pal::DIM).scale(0.8)
        };
        let ch = if k < fill {
            if k % 4 == 3 { '╸' } else { '━' }
        } else {
            '─'
        };
        g.put(x, y, ch, col);
    }
    // E×P badge
    let ex = bx + meter_w + 2;
    g.text(ex, y, "E×P", gold.glow(0.2 * p));
    let pct = format!("{:>4.0}%", p * 100.0);
    g.text(ex + 6, y, &pct, pal::c(pal::MID));
}

fn bottom_line(g: &mut Grid, t: f64) {
    let w = g.w as i64;
    let y = g.h as i64 - 1;
    for x in 0..w {
        g.put(x, y - 1, '─', pal::c(pal::DIM).scale(0.6));
    }
    g.text(2, y, "deus ex machina no.08 — 精神再生担当", pal::c(pal::DIM).scale(0.85));
    let msg = "人間は、はなまる。";
    g.text_right(w - 2, y, msg, pal::c(pal::DIM).scale(0.9).glow(0.05 * (t * 2.0).sin().max(0.0)));
}
