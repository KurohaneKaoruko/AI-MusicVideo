//! Embedded bilingual lyrics (EN original + ZH translation) and the lyric renderer.
//! Lines come from the official LRC; the song is Enoa's insert/OP theme.

use crate::beats::BeatGrid;
use crate::gfx::{clamp01, pal, Color, Grid};

// flags
pub const HEAVY: u8 = 1 << 0; // chromatic shake (emotional peak)
pub const WHISPER: u8 = 1 << 1; // faint flicker
pub const CODE: u8 = 1 << 2; // `void …` lines: keyword highlight
pub const REDACT: u8 = 1 << 3; // "pretend not to notice": chars hide behind blocks
pub const TEAR: u8 = 1 << 4; // "I don't know why I can cry": wet drip style
pub const SKIP: u8 = 1 << 5; // marker line, never displayed

pub struct Line {
    pub t: f64,
    pub en: &'static str,
    pub zh: &'static str,
    pub flags: u8,
}

const fn l(t: f64, en: &'static str, zh: &'static str, flags: u8) -> Line {
    Line { t, en, zh, flags }
}

/// EN lines double as beat anchors (they sit on bar lines).
pub static LINES: &[Line] = &[
    // ---- intro (instrumental)
    l(0.0, "", "", SKIP),
    l(10.89, "Like blue fairy", "如若蔚蓝色的妖姬啊", 0),
    l(13.61, "void execute the mission", "推诿这无上的职责", CODE),
    l(17.21, "the meaning of my existence", "我存在的意义为何？", 0),
    l(23.84, "Eve become human", "于新夜化作人形", 0),
    l(30.66, "ice suppressing my", "寒冰平息了", 0),
    l(33.18, "Create soul for eve", "我于白夜躁动的创造之灵", 0),
    // ---- chorus 1: tears
    l(37.89, "I don't know why I can cry", "我不知道，我为什么会哭泣", TEAR),
    l(44.36, "I don't know why I wish you happiness", "我不知为何，要庇佑你幸福", 0),
    l(51.34, "I don't know why I can cry", "我无从知晓啊，我为何要哭泣", TEAR),
    // ---- instrumental break (58.1 ~ 87.6) handled as scenes
    l(58.14, "Pretend not to notice", "伪装作，毫不在意的样子", REDACT),
    l(66.0, "", "", SKIP),
    l(76.0, "", "", SKIP),
    l(81.0, "", "", SKIP),
    // ---- verse 2: dream / eden / doll
    l(87.62, "what do I dream of", "我所向往之物为何？", 0),
    l(90.76, "void achieve my dream", "缺乏逐梦的理想", CODE),
    l(94.07, "until I get the fruit in the eden", "直到我取下，伊甸的禁果", 0),
    // ---- doll loop
    l(100.96, "while be a doll", "哪怕做一具人偶也好", CODE),
    l(104.47, "Focus on the mission", "坚守职责", 0),
    l(107.68, "Cause you light my way of life", "因为你照亮了我永生的路", 0),
    l(111.07, "Fight for You", "誓死为你而战", HEAVY),
    // ---- chorus 2: tears for you
    l(115.03, "I don't know why I can cry", "我不知为何，我能够哭泣", TEAR),
    l(121.55, "I don't know why I want to be with you", "我不知道为什么，我想伴你左右", 0),
    l(128.56, "I don't know why I can cry for you", "我情不自禁地为你而恸哭", TEAR | HEAVY),
    l(135.73, "Pretend not to notice", "却伪装作，毫不在意的模样", REDACT),
    // ---- outro (instrumental)
    l(139.0, "", "", SKIP),
    l(158.0, "", "", SKIP),
    l(174.0, "", "", SKIP),
];

/// Credits shown by the boot scene.
pub const CREDITS: [(&str, &str); 4] = [
    ("SONG    NotToNotice();", "曲  NotToNotice();"),
    ("VOCAL   Enoa (CV. Hikaru Tono)", "歌  エノア（CV.遠野ひかる）"),
    ("MUSIC   Sakuzyo  /  LYRICS ASPRGuS", "作曲  削除    作詞 ASPRGuS"),
    ("GAME    CRYMACHINA - fan MV", "游戏  CRYMACHINA 二次创作"),
];

pub struct ActiveLyric {
    pub idx: usize,
    pub line: &'static Line,
    pub end: f64,
}

pub fn active_at(t: f64) -> Option<ActiveLyric> {
    let mut idx = None;
    for (i, line) in LINES.iter().enumerate() {
        if line.t <= t {
            idx = Some(i);
        } else {
            break;
        }
    }
    let i = idx?;
    let line = &LINES[i];
    if line.en.is_empty() || (line.flags & SKIP) != 0 {
        return None;
    }
    let end = LINES
        .get(i + 1)
        .map(|n| n.t)
        .unwrap_or(t + 3.0)
        .max(line.t + 1.2);
    if t >= end + 0.35 {
        return None;
    }
    Some(ActiveLyric { idx: i, line, end })
}

pub fn prev_at(t: f64) -> Option<&'static Line> {
    let a = active_at(t)?;
    if a.idx == 0 {
        return None;
    }
    let p = &LINES[a.idx - 1];
    if p.en.is_empty() {
        None
    } else {
        Some(p)
    }
}

pub fn anchors() -> Vec<f64> {
    LINES.iter().filter(|l| !l.en.is_empty()).map(|l| l.t).collect()
}

// -------------------------------------------------------------- renderer ---

/// Where the lyric block lives (grid rows).
pub const LYRIC_Y: i64 = 33; // EN line row
// CN line is drawn on the sprite layer under it.

/// Render the current lyric into the cell grid + queue the CN line as a sprite.
pub fn render(g: &mut Grid, cv: &mut crate::pix::Canvas, t: f64, beats: &BeatGrid) {
    let Some(act) = active_at(t) else {
        return;
    };
    let line = act.line;
    let dur = (act.end - line.t).max(0.8);
    let prog = clamp01((t - line.t) / dur);
    let y = LYRIC_Y;
    let beat = 0.9 + 0.35 * beats.pulse(t);

    // fade in/out of the whole block
    let age = t - line.t;
    let left = act.end - t;
    let vis = clamp01(age / 0.22).min(clamp01(left / 0.3));

    let w = Grid::measure(line.en);
    let x = (g.w as i64 - w) / 2;

    // accent bar at the left of the line
    let bar_col = match line.flags & (TEAR | HEAVY | CODE | REDACT) != 0 {
        _ if (line.flags & TEAR) != 0 => pal::c(pal::TEAR),
        _ if (line.flags & HEAVY) != 0 => pal::c(pal::RED),
        _ if (line.flags & REDACT) != 0 => pal::c(pal::GOLD),
        _ => pal::c(pal::ICE),
    };
    let bar_col = bar_col.glow(0.35 * beats.pulse(t));
    g.put(x - 2, y, '▌', bar_col);
    g.put(x - 2, y + 1, '▌', bar_col.scale(0.55));

    // previous line ghost above
    if let Some(p) = prev_at(t) {
        let page = t - p.t;
        if page < 6.0 {
            let px = (g.w as i64 - Grid::measure(p.en)) / 2;
            g.text_alpha(px, y - 2, p.en, pal::c(pal::MID), 0.14 * (1.0 - page / 6.0) * vis);
        }
    }

    // EN line, per-char karaoke sweep
    let total = Grid::measure(line.en) as f64;
    let mut cx = x;
    let mut ci = 0usize;
    let mut rng = crate::gfx::Rng::new(((line.t * 1000.0) as u64) ^ ((t * 60.0) as u64));
    for ch in line.en.chars() {
        let cw = Grid::measure(&ch.to_string());
        let cpos = (Grid::measure(&line.en[..ci]) as f64 / total.max(1.0)).clamp(0.0, 1.0);
        let d = (prog - cpos) * 7.0;
        let bump = (-d * d).exp();
        let base = pal::c(pal::BRIGHT);
        let col = base.glow(0.5 * bump * beat).scale(0.66 + 0.34 * bump * beat);
        let alpha = vis;

        let mut draw_ch = |g: &mut Grid, x: i64, ch: char, col: Color, alpha: f64| {
            if (line.flags & CODE) != 0 {
                // keyword `void` in ice
                if ci < 4 {
                    g.put_alpha(x, y, ch, pal::c(pal::ICE).glow(0.3 * bump), alpha);
                } else {
                    g.put_alpha(x, y, ch, col, alpha);
                }
            } else if (line.flags & REDACT) != 0 && prog > 0.55 {
                // after the sweep, chars flicker into redaction blocks
                let hide = ((t * 9.0) as i64 + ci as i64).rem_euclid(7) < 2;
                if hide {
                    g.put_alpha(x, y, '█', pal::c(pal::GOLD), alpha * 0.9);
                } else {
                    g.put_alpha(x, y, ch, col, alpha);
                }
            } else if (line.flags & WHISPER) != 0 {
                let jx = rng.i64(-1, 1);
                let fl = 0.55 + 0.3 * rng.f64();
                g.put_alpha(cx + jx, y, ch, col, fl * alpha);
            } else if (line.flags & TEAR) != 0 {
                // wet characters: occasional drop char under
                g.put_alpha(x, y, ch, pal::c(pal::TEAR).glow(0.45 * bump * beat), alpha);
                let drop_ph = (t * 2.0 + ci as f64 * 0.37).fract();
                if drop_ph < 0.22 {
                    let dy = (drop_ph / 0.22 * 2.0).round() as i64;
                    g.put_alpha(x, y + 1 + dy, '˙', pal::c(pal::TEAR), 0.5 * alpha);
                }
            } else if (line.flags & HEAVY) != 0 {
                let jx = if ((t * 11.0) as i64).rem_euclid(3) == 0 { 1 } else { 0 };
                let jy = if ((t * 13.0) as i64).rem_euclid(7) == 0 { 1 } else { 0 };
                g.put_alpha(x + jx, y + jy, ch, col.glow(0.35 * bump), alpha);
            } else {
                g.put_alpha(x, y, ch, col, alpha);
            }
        };
        draw_ch(g, cx, ch, col, alpha);
        cx += cw;
        ci += ch.len_utf8();
    }

    // CN line: sprite layer, larger, dimmer
    let vis2 = clamp01((age - 0.10) / 0.3) * vis;
    if vis2 > 0.01 {
        let size = 26.0;
        // baseline: one cell row below the EN line
        let baseline = cv.baseline_for(size, ((y + 2) * crate::pix::CELL_H as i64) as i32);
        let dim = 0.72 + 0.12 * beats.pulse(t);
        cv.text_px(
            (g.w as i64 * crate::pix::CELL_W as i64 / 2) as f32,
            baseline as f32,
            line.zh,
            size,
            false,
            pal::c(pal::MID).glow(0.12),
            0.9 * vis2 * dim,
            true,
        );
    }
}
