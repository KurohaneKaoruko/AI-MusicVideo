//! Heads-up display: top status bar, bottom progress + keys, overlays.

use crate::gfx::{clamp01, Color, Grid};
use crate::scenes::{section_at, Ctx, Ui, SECTIONS};

const ROSE: Color = Color::hex(0xff6d8a);
const DIM: Color = Color::hex(0x55607a);
const FG: Color = Color::hex(0xaab4c8);
const TRACK: Color = Color::hex(0x232c3c);
const FUTURE: Color = Color::hex(0x1b2331);

pub fn render(g: &mut Grid, t: f64, ui: &Ui, ctx: &Ctx) {
    let w = g.w as i64;
    let h = g.h as i64;
    if h < 14 || w < 56 {
        g.text_center(h / 2, "terminal too small (need 56x14)", DIM);
        return;
    }

    // ---------------- top bar
    g.text(2, 0, "world.search (you) ;", ROSE.glow(0.1));
    let lw = Grid::measure("world.search (you) ;");
    g.text(2 + lw + 1, 0, "Mili", DIM);

    // right: time
    let time_s = fmt_time(t);
    let dur_s = fmt_time(ctx.dur);
    g.text_right(w - 2, 0, &format!("{time_s} / {dur_s}"), FG);
    let tw = Grid::measure(&format!("{time_s} / {dur_s}"));
    // beat dots
    let bi = ctx.beats.beat_index(t);
    for k in 0..4 {
        let on = bi.rem_euclid(4) == k;
        let ch = if on { '●' } else { '○' };
        let col = if on { section_at(t).accent.glow(0.4) } else { DIM };
        g.put(w - 2 - tw - 3 - k as i64, 0, ch, col);
    }

    // separator
    for x in 0..w {
        g.put_alpha(x, 1, '─', DIM, 0.55);
    }
    // section label sits on the separator
    let sec = section_at(t);
    let lw2 = Grid::measure(sec.label);
    g.text((w - lw2) / 2, 1, sec.label, sec.accent.scale(0.95));

    // ---------------- bottom: progress bar with section markers
    let py = h - 2;
    let bx = 2;
    let bw = w - 4;
    let prog = clamp01(t / ctx.dur.max(1.0));
    g.hline(bx, bx + bw - 1, py, '─', TRACK);
    for s in SECTIONS {
        let x0 = bx + (bw as f64 * (s.t0 / ctx.dur.max(1.0))) as i64;
        let x1 = bx + (bw as f64 * ((s.t1.min(ctx.dur)) / ctx.dur.max(1.0))) as i64;
        let active = t >= s.t0 && t < s.t1;
        let col = if active {
            s.accent.glow(0.2)
        } else if t >= s.t1 {
            s.accent.scale(0.5)
        } else {
            FUTURE
        };
        for x in x0.max(bx)..x1.min(bx + bw) {
            g.put(x, py, '─', col);
        }
        if s.t0 > 0.0 {
            g.put_alpha(x0, py, '┼', if active { s.accent } else { DIM }, 0.9);
        }
    }
    // playhead
    let ph = bx + (bw as f64 * prog) as i64;
    g.put(ph, py, '▌', ROSE.glow(0.4));
    g.put_alpha(ph, py - 1, '▼', ROSE, 0.75);

    // ---------------- keys + volume
    let keys = "[space] pause  [←→] seek  [+/-] vol  [f] fps  [h] help  [q] quit";
    let kw = Grid::measure(keys);
    let vx = w - 7; // volume block at the far right
    let kx = ((w - kw) / 2).min(vx - kw / 2 - 2).max(bx);
    g.text(kx, h - 1, keys, DIM.scale(0.9));
    g.text(vx - 4, h - 1, "vol", DIM);
    for k in 0..5 {
        let on = ui.volume > k as f32 / 5.0;
        g.put(vx + k, h - 1, if on { '█' } else { '·' }, if on { FG } else { DIM });
    }

    // ---------------- fps
    if ui.show_fps {
        g.text_right(w - 2, 2, &format!("{:.0} fps", ui.fps), DIM);
    }

    // ---------------- overlays
    if ui.paused {
        for c in g.cells.iter_mut() {
            c.fg = c.fg.over(c.bg, 0.35);
        }
        let pw = 22;
        g.box_rounded((w - pw) / 2, h / 2 - 2, pw, 4, GRAY2, Some(g.bg.scale(1.4)), None);
        g.text_center(h / 2 - 1, "▌▌  PAUSED", FG.glow(0.1));
        g.text_center(h / 2 + 1, "[space] resume", DIM);
    }
    if ui.finished {
        let a = 0.5 + 0.5 * ctx.beats.pulse(t);
        g.text_center(h - 4, "♪  playback ended — the search continues in your heart", ROSE.scale(a));
    }
}

const GRAY2: Color = Color::hex(0x8a93a6);

pub fn fmt_time(t: f64) -> String {
    let t = t.max(0.0);
    let m = (t / 60.0).floor() as i64;
    let s = (t - m as f64 * 60.0).floor() as i64;
    format!("{m}:{s:02}")
}

pub fn help(g: &mut Grid) {
    let w = g.w as i64;
    let h = g.h as i64;
    let lines: [(&str, Color); 13] = [
        ("world.search (you) ;", ROSE),
        ("a terminal MV for Mili — momocashew", GRAY2),
        ("", FG),
        ("[space]  pause / resume", FG),
        ("[← →]   seek 5 seconds", FG),
        ("[+ -]   volume", FG),
        ("[f]     toggle fps", FG),
        ("[h]     close this help", FG),
        ("[q]     quit", FG),
        ("", FG),
        ("every form of \"you\" is a file in world/", GRAY2),
        ("and you are still looking for the old self", GRAY2),
        ("", FG),
    ];
    let bw = 46;
    let bh = lines.len() as i64 + 2;
    let (ix, iy, _, _) =
        g.box_rounded((w - bw) / 2, (h - bh) / 2, bw, bh, GRAY2.scale(1.2), Some(g.bg.scale(1.7)), None);
    for (i, (s, c)) in lines.iter().enumerate() {
        if s.is_empty() {
            continue;
        }
        g.text(ix + 3, iy + i as i64, s, *c);
    }
}
