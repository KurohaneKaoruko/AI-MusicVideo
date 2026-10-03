//! Bilingual lyrics: the LRC-derived timeline (verbatim — it is calibrated to
//! the song) plus a renderer that typesets each line for the scene it sits in,
//! on the pixel layer when the film is being rendered and on the cell grid when
//! it is played live in a terminal.

use crate::beats::BeatGrid;
use crate::gfx::{clamp01, pal, Color, Grid};
use crate::pix::Canvas;

pub const ECHO: u8 = 1 << 0;
pub const HEAVY: u8 = 1 << 1;
pub const WHISPER: u8 = 1 << 2;
pub const CENSOR: u8 = 1 << 3;
#[allow(dead_code)]
pub const STRETCH: u8 = 1 << 4;

pub struct Line {
    pub t: f64,
    pub en: &'static str,
    pub zh: &'static str,
    pub flags: u8,
    /// (en_head, en_fill, en_tail) for stretched lines
    pub stretch: Option<(&'static str, char, &'static str)>,
}

const fn l(t: f64, en: &'static str, zh: &'static str, flags: u8) -> Line {
    Line { t, en, zh, flags, stretch: None }
}

pub const CREDITS: [(&str, &str); 2] = [
    ("作词 : momocashew", "作词 : momocashew"),
    ("作曲 : momocashew / Yamato Kasai", "作曲 : momocashew / Yamato Kasai"),
];

pub static LINES: &[Line] = &[
    // ---- instrumental
    l(0.0, "", "", 0),
    l(6.98, "", "", 0),
    // ---- verse: memory sort
    l(14.36, "Side by side, the lovelier goes up", "肩并着肩，更可爱的那个越升越高", 0),
    l(21.18, "Reach for the top, till the bubble pops", "向着顶端攀去，直到泡沫啪地破掉", 0),
    l(28.05, "Take a memory, insert to the past", "取出一段记忆，插入到往昔之中", 0),
    l(35.05, "Which one is the sweetest? Save it before the last", "哪一段最甜？趁还来得及，把它存档", 0),
    l(39.86, "", "", 0),
    // ---- table
    l(40.90, "If you turn into a table", "如果你变成一张桌子", 0),
    l(44.14, "You must be at the height of my elbows", "一定是正好到我手肘的高度", 0),
    l(47.71, "Odd stubby leg makes you wobbly", "一条歪歪的短腿，让你摇摇晃晃", 0),
    l(51.00, "You're the only perfect table for me", "你是我唯一完美的桌子", 0),
    l(54.46, "", "", 0),
    // ---- eggplant
    l(54.46, "If you turn into an eggplant", "如果你变成一根茄子", 0),
    l(57.83, "You'll be my purple friend", "你会是我紫色的朋友", 0),
    l(61.22, "Scratchy nail marks on your body", "你的身上留着我抓出的指甲痕", 0),
    l(64.66, "You're the only perfect eggplant for me", "你是我唯一完美的茄子", 0),
    l(68.17, "I'm searching", "我在寻找", ECHO),
    l(70.91, "", "", 0),
    // ---- cat
    l(82.89, "If you're a cat", "如果你是一只猫", 0),
    l(85.49, "A little kitten", "一只小猫咪", 0),
    l(87.46, "You must be bluepoint", "那一定是蓝色重点色", 0),
    l(89.76, "Meowing a lot", "喵喵叫个不停", 0),
    l(92.58, "Hissing a lot", "又哈气连连", 0),
    l(94.39, "But you never purr", "可你从来不肯打呼噜", 0),
    // ---- steak
    l(96.64, "If you're a steak", "如果你是一块牛排", 0),
    l(99.50, "A fillet mignon covered in mushroom sauce", "那就是淋满蘑菇酱的菲力牛排", 0),
    l(103.75, "Browned on the outside", "外表煎得焦香", 0),
    l(106.66, "Medium rare inside", "内里三分熟", 0),
    // ---- flower
    l(109.72, "If you turn into a flower", "如果你变成一朵花", 0),
    l(113.15, "You must be the most delicate ever", "那你一定是世间最娇嫩的一朵", 0),
    l(116.60, "Even if you bloom just occasionally", "哪怕你只是偶尔才肯绽放", 0),
    l(119.96, "You're the only perfect flower for me", "你是我唯一完美的花", 0),
    // ---- human
    l(123.44, "If you turn into a human", "如果你变成一个人", 0),
    l(126.82, "I'd like all the warmth coming from your hands", "我喜欢你双手传来的每一分温度", 0),
    l(130.24, "Bit of stinky breath, with hair that's unruly", "呼吸有点臭，头发乱糟糟", 0),
    l(133.65, "You're the only perfect human for me", "你是我唯一完美的人", 0),
    l(137.09, "I'm searching", "我在寻找", ECHO),
    l(139.7, "", "", 0),
    // ---- bridge: the forgotten words
    l(151.84, "If you're a piece", "如果你是一块", 0),
    l(154.79, "A piece of ****", "一块××××", CENSOR),
    l(156.42, "You must be… U-Um?", "你一定是……那、那个词？", WHISPER),
    l(158.85, "What? What's it called?", "什么？叫什么来着？", WHISPER),
    l(161.64, "What's that word again", "那个词叫什么来着", WHISPER),
    Line {
        t: 163.66,
        en: "\"Do-o-o-o-ope\"?",
        zh: "「超————棒？」",
        flags: 0,
        stretch: Some(("\"D", 'o', "pe\"?")),
    },
    l(165.73, "If you could stay", "如果你能留下来", 0),
    l(168.60, "Stay there where you are", "就留在原地，别走", 0),
    l(170.43, "So again… U-Um?", "那再来一次……那、那个？", WHISPER),
    l(172.28, "Oh, what? What's it called?", "哎，什么？叫什么来着？", WHISPER),
    l(175.43, "What's that word again?", "那个词到底叫什么来着？", WHISPER),
    Line {
        t: 177.35,
        en: "\"Lo-o-o-o-ove\"?",
        zh: "「爱————？」",
        flags: 0,
        stretch: Some(("\"L", 'o', "ve\"?")),
    },
    l(179.41, "", "", 0),
    // ---- chorus
    l(179.47, "I'm searching for", "我在寻找", ECHO),
    l(180.97, "Searching for you everywhere", "在世界的每个角落寻找你", 0),
    l(182.65, "I can see you in most everything", "仿佛万物之中，都有你的身影", 0),
    l(186.43, "I'm searching for", "我在寻找", ECHO),
    l(187.76, "Searching for you everywhere", "在世界的每个角落寻找你", 0),
    l(189.37, "But somehow you can stop transforming", "可你却不知为何，始终变个不停", 0),
    l(193.28, "I'm searching for, searching for, searching for", "我在寻找，寻找，寻找", ECHO),
    l(196.32, "Searching for, searching for, searching for", "寻找着，寻找着，寻找着", ECHO),
    l(198.93, "A-Ah", "啊——啊", 0),
    l(200.25, "I'm searching for, searching for, searching for", "我在寻找，寻找，寻找", ECHO),
    l(203.16, "My old self that's gone since you have left", "那个自你离开后就不在了的、从前的我", HEAVY),
    // ---- buckets
    l(207.14, "Each one of us has a bucket of love", "我们每个人，都有一桶爱", 0),
    l(214.35, "Some of us empty, some of us filled up", "有人空空如也，有人满得快溢出来", 0),
    l(221.18, "Cut us into pieces, break us down to the cell", "把我们切成碎片，分解到细胞", 0),
    l(228.16, "Merge us back up, we have evolved now", "再重新拼合起来——我们进化了", 0),
    l(233.60, "", "", 0),
    // ---- flower (reprise)
    l(233.77, "If you turn into a flower", "如果你变成一朵花", 0),
    l(237.09, "You must be the most delicate ever", "那你一定是世间最娇嫩的一朵", 0),
    l(240.48, "Even if you bloom just occasionally", "哪怕你只是偶尔才肯绽放", 0),
    l(243.94, "You're the only perfect flower for me", "你是我唯一完美的花", 0),
    // ---- human (reprise)
    l(247.37, "If you turn into a human", "如果你变成一个人", 0),
    l(250.78, "I like all the warmth coming from your hands", "我喜欢你双手传来的每一分温度", 0),
    l(254.22, "Bitter stinky breath with hair that is unruly", "口气苦涩发臭，头发乱糟糟", 0),
    l(257.56, "You're the only perfect human for me", "你是我唯一完美的人", 0),
    l(261.06, "I'm searching", "我在寻找", ECHO),
    l(263.6, "", "", 0),
    // ---- outro
    l(275.75, "If you're a dog", "如果你是一条狗", 0),
    l(278.68, "A little puppy", "一只小奶狗", 0),
    l(280.55, "Going ruff ruff ruff", "汪！汪！汪！", 0),
    l(282.73, "If you're a fruit", "如果你是一种水果", 0),
    l(285.58, "A tomato", "那是一颗番茄", 0),
    l(287.35, "Full of juice", "装满了满满的果汁", 0),
    l(290.37, "", "", 0),
];

/// Beat-grid calibration anchors: every sung line sits on a musical phrase.
pub fn anchors() -> Vec<f64> {
    LINES.iter().filter(|l| !l.en.is_empty()).map(|l| l.t).collect()
}

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
    if line.en.is_empty() {
        return None;
    }
    let end = LINES.get(i + 1).map(|n| n.t).unwrap_or(t + 3.0).max(line.t + 1.0);
    if t >= end + 0.45 {
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

// ------------------------------------------------------------- typesetting ---

#[derive(Clone, Copy, PartialEq)]
pub enum Anchor {
    Bottom,
    Center,
    Upper,
}

/// Where the current section wants its words.
pub fn anchor_for(t: f64) -> Anchor {
    use crate::scenes::Sc;
    match crate::scenes::section_at(t).sc {
        Sc::Radar | Sc::Bridge | Sc::Chorus | Sc::Query => Anchor::Center,
        Sc::Outro | Sc::Fin => Anchor::Upper,
        _ => Anchor::Bottom,
    }
}

fn stretch_n(prog: f64) -> usize {
    let grow = crate::gfx::ease_out_cubic(prog * 2.2);
    let wob = (prog * 9.0).sin() * 0.8 * (1.0 - prog);
    (7.0 * grow + wob).round().clamp(0.0, 11.0) as usize
}

/// Draw one line of text glyph by glyph, choosing a colour per glyph.
#[allow(clippy::too_many_arguments)]
fn typeset(
    g: &mut Grid,
    cv: &mut Canvas,
    x: f32,
    baseline: f32,
    row: i64,
    s: &str,
    size: f32,
    color_of: &dyn Fn(usize, char) -> (Color, f64),
    alpha: f64,
    pixel: bool,
) -> f32 {
    let mut pen = x;
    for (i, ch) in s.chars().enumerate() {
        let (col, a) = color_of(i, ch);
        if pixel {
            cv.sprite(ch, pen, baseline, size, col, a * alpha);
        } else if a * alpha > 0.06 && row >= 0 && row < g.h as i64 {
            let cx = (pen / crate::pix::CELL_W as f32).round() as i64;
            g.put_alpha(cx, row, ch, col, a * alpha);
        }
        pen += cv.advance(ch, size, false);
    }
    pen - x
}

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, beats: &BeatGrid, pixel: bool) {
    let Some(act) = active_at(t) else {
        return;
    };
    let line = act.line;
    let dur = (act.end - line.t).max(0.5);
    let prog = clamp01((t - line.t) / dur);
    let beat = 0.85 + 0.3 * beats.pulse(t);

    let anchor = anchor_for(t);
    let (size_en, size_zh, base_y, row) = match anchor {
        Anchor::Bottom => (37.0f32, 27.0f32, crate::pix::H as f32 - 152.0, g.h as i64 - 5),
        Anchor::Center => (43.0f32, 31.0f32, crate::pix::H as f32 * 0.60, g.h as i64 / 2 + 7),
        Anchor::Upper => (39.0f32, 28.0f32, crate::pix::H as f32 * 0.24, g.h as i64 / 4),
    };
    let row_zh = row + 3;

    let tint = pal::c(pal::ROSE);
    let cx = crate::pix::W as f32 / 2.0;

    // ---- the previous line, fading away upward
    if let Some(p) = prev_at(t) {
        let age = t - p.t;
        if age < 6.5 {
            let a = 0.20 * (1.0 - age / 6.5);
            let w = cv.text_width(p.en, size_en * 0.82, false);
            let f = |_: usize, _: char| (tint, 1.0);
            typeset(g, cv, cx - w / 2.0, base_y - 62.0, row - 3, p.en, size_en * 0.82, &f, a, pixel);
        }
    }

    // ---- the EN line
    let text: String = match line.stretch {
        Some((h, f, tl)) => format!("{}{}{}", h, f.to_string().repeat(stretch_n(prog)), tl),
        None => line.en.to_string(),
    };
    let total_w = cv.text_width(&text, size_en, false);
    let x0 = cx - total_w / 2.0;
    let censor = (line.flags & CENSOR) != 0;
    let whisper = (line.flags & WHISPER) != 0;
    let heavy = (line.flags & HEAVY) != 0;

    let n_ch = text.chars().count().max(1);
    let sweep = prog * (n_ch as f64 + 4.0) - 2.0;
    let mut rng = crate::gfx::Rng::new(((line.t * 1000.0) as u64) ^ ((t * 60.0) as u64));

    // HEAVY: chromatic ghosts under the main pass
    if heavy {
        for (off, col) in [(-3.0f32, Color::rgb(255, 64, 110)), (3.0, Color::rgb(64, 170, 255))] {
            let f = |_: usize, _: char| (col, 0.55);
            typeset(g, cv, x0 + off, base_y, row, &text, size_en, &f, 0.9 * beat, pixel);
        }
    }

    let color_of = |i: usize, ch: char| -> (Color, f64) {
        if censor && ch == '*' {
            return (Color::hex(0xc9526a), 1.0);
        }
        let d = i as f64 - sweep;
        let bump = (-d * d * 0.42).exp();
        let col = tint.glow(0.55 * bump).scale(0.66 + 0.44 * bump * beat);
        let a = if whisper { 0.45 + 0.4 * bump } else { 1.0 };
        (col, a)
    };
    let shake = if heavy && ((t * 11.0) as i64) % 3 == 0 {
        (((t * 97.0) as i64) % 3 - 1) as f32
    } else {
        0.0
    };
    typeset(g, cv, x0 + shake, base_y, row, &text, size_en, &color_of, 1.0, pixel);

    // redaction blocks for censored characters
    if censor {
        let mut pen = x0;
        for ch in text.chars() {
            let adv = cv.advance(ch, size_en, false);
            if ch == '*' && pixel {
                let pick = [0x2b3547u32, 0x3a4658, 0xc9526a][rng.i64(0, 2) as usize];
                let a = 0.55 + 0.45 * rng.f64();
                cv.rect_a(
                    pen as i32 - 1,
                    (base_y - size_en * 0.74) as i32,
                    adv as i32 + 2,
                    (size_en * 0.86) as i32,
                    Color::hex(pick),
                    a,
                );
                if rng.f64() > 0.6 {
                    cv.rect_a(pen as i32, (base_y - size_en * 0.30) as i32, adv as i32, 2, tint, 0.6);
                }
            }
            pen += adv;
        }
    }

    if whisper && pixel {
        for _ in 0..6 {
            let hx = x0 + (rng.f64() * total_w as f64) as f32;
            let hy = base_y - 46.0 + (rng.f64() * 70.0) as f32;
            cv.sprite('·', hx, hy, 16.0, tint, 0.35 * rng.f64());
        }
    }

    // ---- echo ghosts (chorus)
    if (line.flags & ECHO) != 0 {
        let ph = beats.beat_phase(t);
        for k in 1..=2 {
            let a = 0.20 / k as f64 * (0.4 + 0.6 * (1.0 - ph));
            let dx = k as f32 * (if beats.beat_index(t) % 2 == 0 { 5.0 } else { -5.0 });
            let f = |_: usize, _: char| (tint, 1.0);
            typeset(
                g,
                cv,
                x0 + dx,
                base_y - k as f32 * 36.0,
                row - 2 * k as i64,
                &text,
                size_en * (1.0 - 0.1 * k as f32),
                &f,
                a,
                pixel,
            );
        }
    }

    // ---- ZH line, revealed left to right
    let zh_col = pal::c(pal::BRIGHT).scale(0.78);
    let zch: Vec<char> = line.zh.chars().collect();
    let zw = cv.text_width(line.zh, size_zh, false);
    let zx0 = cx - zw / 2.0;
    let shown = (zch.len() as f64 * crate::gfx::ease_out_cubic(prog * 1.7)).ceil() as usize;
    let mut pen = zx0;
    for (i, ch) in zch.iter().enumerate() {
        let adv = cv.advance(*ch, size_zh, false);
        if i < shown {
            let a = 0.68 + 0.22 * beats.pulse(t);
            if pixel {
                cv.sprite(*ch, pen, base_y + 56.0, size_zh, zh_col.scale(a + 0.30), 0.88);
            } else if row_zh < g.h as i64 {
                g.put((pen / crate::pix::CELL_W as f32) as i64, row_zh, *ch, zh_col);
            }
        }
        pen += adv;
    }

    // ---- a thin rule under the words when they hang in the middle of the frame
    if pixel && anchor != Anchor::Bottom {
        let a = 0.10 + 0.12 * beats.pulse(t);
        cv.rect_a((x0 - 24.0) as i32, (base_y + 88.0) as i32, (total_w + 48.0) as i32, 1, tint, a);
    }
}
