//! Embedded bilingual lyrics (EN original + ZH translation) and the lyric renderer.

use crate::beats::BeatGrid;
use crate::gfx::{clamp01, Color, Grid};

#[allow(dead_code)]
pub const STRETCH: u8 = 1 << 0;
pub const ECHO: u8 = 1 << 1;
pub const HEAVY: u8 = 1 << 2;
pub const WHISPER: u8 = 1 << 3;
pub const CENSOR: u8 = 1 << 4;
pub const SKIP: u8 = 1 << 5; // marker line, never displayed

pub struct Line {
    pub t: f64,
    pub en: &'static str,
    pub zh: &'static str,
    pub flags: u8,
    /// (en_head, en_fill, en_tail, zh_head, zh_fill, zh_tail) for STRETCH lines
    pub stretch: Option<(&'static str, char, &'static str, &'static str, char, &'static str)>,
}

const fn l(t: f64, en: &'static str, zh: &'static str, flags: u8) -> Line {
    Line { t, en, zh, flags, stretch: None }
}
const fn lf(t: f64, en: &'static str, zh: &'static str, flags: u8) -> Line {
    Line { t, en, zh, flags, stretch: None }
}

/// zh translations of the credit lines (shown by the boot scene).
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
    lf(68.17, "I'm searching", "我在寻找", ECHO),
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
    lf(137.09, "I'm searching", "我在寻找", ECHO),
    l(139.7, "", "", 0),
    // ---- bridge: the forgotten words
    l(151.84, "If you're a piece", "如果你是一块", 0),
    lf(154.79, "A piece of ****", "一块××××", CENSOR),
    l(156.42, "You must be… U-Um?", "你一定是……那、那个词？", WHISPER),
    l(158.85, "What? What's it called?", "什么？叫什么来着？", WHISPER),
    l(161.64, "What's that word again", "那个词叫什么来着", WHISPER),
    Line {
        t: 163.66,
        en: "\"Do-o-o-o-ope\"?",
        zh: "「超————棒？」",
        flags: 0,
        stretch: Some(("\"D", 'o', "pe\"?", "「超", '—', "棒？」")),
    },
    l(165.73, "If you could stay", "如果你能留下来", 0),
    l(168.60, "Stay there where you are", "就留在原地，别走", 0),
    l(170.43, "So again… U-Um?", "那再来一次……那、那个？", WHISPER),
    l(172.28, "Oh, what? What's it called?", "哎，什么？叫什么来着？", WHISPER),
    l(175.43, "What's that word again?", "那个词到底叫什么来着？", WHISPER),
    Line {
        t: 177.35,
        en: "\"Lo-o-o-ove\"?",
        zh: "「爱————？」",
        flags: 0,
        stretch: Some(("\"L", 'o', "ve\"?", "「爱", '—', "？」")),
    },
    l(179.41, "", "", 0),
    // ---- chorus
    lf(179.47, "I'm searching for", "我在寻找", ECHO),
    l(180.97, "Searching for you everywhere", "在世界的每个角落寻找你", 0),
    l(182.65, "I can see you in most everything", "仿佛万物之中，都有你的身影", 0),
    lf(186.43, "I'm searching for", "我在寻找", ECHO),
    l(187.76, "Searching for you everywhere", "在世界的每个角落寻找你", 0),
    l(189.37, "But somehow you can stop transforming", "可你却不知为何，始终变个不停", 0),
    l(193.28, "I'm searching for, searching for, searching for", "我在寻找，寻找，寻找", ECHO),
    l(196.32, "Searching for, searching for, searching for", "寻找着，寻找着，寻找着", ECHO),
    l(198.93, "A-Ah", "啊——啊", 0),
    lf(200.25, "I'm searching for, searching for, searching for", "我在寻找，寻找，寻找", ECHO),
    lf(203.16, "My old self that's gone since you have left", "那个自你离开后就不在了的、从前的我", HEAVY),
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
    lf(261.06, "I'm searching", "我在寻找", ECHO),
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

// -------------------------------------------------------------- renderer ---

#[derive(Clone, Copy, PartialEq)]
pub enum Anchor {
    Bottom,
    Center,
    Upper,
}

pub struct LyricStyle {
    pub tint: Color,
    pub anchor: Anchor,
}

/// Stretch-length animation for `Do-o-o-o-ope` style lines.
fn stretch_count(prog: f64) -> usize {
    let grow = crate::gfx::ease_out_cubic(prog * 2.2);
    let wob = (prog * 9.0).sin() * 0.8 * (1.0 - prog);
    ((0.0 + 6.0 * grow + wob).round() as i64).clamp(0, 9) as usize
}

/// Draw one line's EN text with all styles. Returns the x where it started.
#[allow(clippy::too_many_arguments)]
fn draw_en(
    g: &mut Grid,
    y: i64,
    x: i64,
    line: &'static Line,
    prog: f64,
    reveal_t: f64,
    t: f64,
    base: Color,
    beat: f64,
    rng_seed: u64,
) {
    let _ = reveal_t;
    // per-line styles
    if (line.flags & HEAVY) != 0 {
        let shake = if ((t * 11.0) as i64) % 3 == 0 { ((t * 97.0) as i64) % 3 - 1 } else { 0 };
        g.chroma_text(x + shake, y, line.en, base.glow(0.15 * beat), 1);
        return;
    }

    let bg = g.row_bg(y as usize);
    let mut rng = crate::gfx::Rng::new(rng_seed ^ ((t * 60.0) as u64));

    // stretched lines
    if let Some((h, f, tl, _, _, _)) = line.stretch {
        let n = stretch_count(prog);
        g.stretch_text(y, h, f, tl, n, base);
        return;
    }

    // char-by-char with karaoke sweep + censor blocks
    let total = Grid::measure(line.en) as f64;
    let mut cx = x;
    let mut ci = 0usize;
    for ch in line.en.chars() {
        let cw = Grid::measure(&ch.to_string());
        let cpos = (Grid::measure(&line.en[..ci]) as f64 / total.max(1.0)).clamp(0.0, 1.0);
        // karaoke bump
        let d = (prog - cpos) * 6.5;
        let bump = (-d * d).exp();
        let col = base
            .glow(0.55 * bump * beat)
            .scale(0.62 + 0.38 * bump * beat)
            .over(bg, 1.0);
        let alpha = 1.0f64;

        if ch == '*' && (line.flags & CENSOR) != 0 {
            // flickering redaction block
            let pick = ["▓", "▒", "█", "?", "▓", "▚", "▓"][rng.i64(0, 6) as usize];
            let flick = 0.75 + 0.25 * rng.f64();
            let c = Color::hex(0xc9526a).glow(0.3 * flick);
            g.put_alpha(cx, y, pick.chars().next().unwrap(), c, flick);
        } else if (line.flags & WHISPER) != 0 {
            let jx = rng.i64(-1, 1);
            let fl = 0.55 + 0.3 * rng.f64();
            g.put_alpha(cx + jx, y, ch, col, fl);
        } else {
            let _ = alpha;
            g.put(cx, y, ch, col);
        }
        cx += cw;
        ci += ch.len_utf8();
    }
}

pub fn render(g: &mut Grid, t: f64, beats: &BeatGrid, style: &LyricStyle) {
    let Some(act) = active_at(t) else {
        return;
    };
    let line = act.line;
    let dur = (act.end - line.t).max(0.5);
    let prog = clamp01((t - line.t) / dur);

    // anchor position
    let base = match style.anchor {
        Anchor::Bottom => g.h as i64 - 9,
        Anchor::Center => g.h as i64 / 2 + 2,
        Anchor::Upper => (g.h as i64 / 5).max(3),
    };

    let beat = 0.9 + 0.25 * beats.pulse(t);

    // previous line ghost
    if let Some(p) = prev_at(t) {
        let age = t - p.t;
        if age < 7.0 {
            let py = base - 3;
            let px = (g.w as i64 - Grid::measure(p.en)) / 2;
            let a = 0.18 * (1.0 - age / 7.0);
            g.text_alpha(px, py, p.en, style.tint, a);
        }
    }

    // accent bar
    let bar_col = style.tint.glow(0.3 * beats.pulse(t));
    let w = Grid::measure(line.en);
    let x = (g.w as i64 - w) / 2;
    let bar_ch = if beats.downbeat(t) { '▌' } else { '▌' };
    let _ = bar_ch;
    g.put(x - 2, base, '▌', bar_col);
    g.put(x - 2, base + 1, '▌', bar_col.scale(0.6));

    // EN line reveal
    draw_en(g, base, x, line, prog, t - line.t, t, style.tint, beat, (line.t * 1000.0) as u64);

    // echo ghosts (chorus)
    if (line.flags & ECHO) != 0 {
        let ph = beats.beat_phase(t);
        for k in 1..=2 {
            let a = 0.22 / k as f64 * (0.5 + 0.5 * (1.0 - ph));
            let dx = (k as i64) * (if beats.beat_index(t) % 2 == 0 { 1 } else { -1 });
            g.text_alpha(x + dx, base - k * 2, line.en, style.tint, a);
        }
    }

    // ZH line
    let zh_col = Color::hex(0xb9c2d0);
    let zh_x = (g.w as i64 - Grid::measure(line.zh)) / 2;
    let rev = clamp01((t - line.t - 0.18) / 0.4);
    let zh_prog = prog;
    let zglyphs: Vec<char> = line.zh.chars().collect();
    let n_show = (zglyphs.len() as f64 * crate::gfx::ease_out_cubic(zh_prog * 1.6 + rev * 0.4))
        .ceil()
        .clamp(0.0, zglyphs.len() as f64) as usize;
    let mut zx = zh_x;
    for (i, ch) in zglyphs.iter().enumerate() {
        let cw = crate::gfx::char_w(*ch) as i64;
        if i < n_show {
            let dim = 0.62 + 0.2 * beats.pulse(t);
            g.put(zx, base + 2, *ch, zh_col.scale(dim));
        }
        zx += cw;
    }

    // dotted rule under the lyric zone
    let rule_y = base + 4;
    if rule_y < g.h as i64 - 6 {
        let cols = g.w as i64;
        for rx in (0..cols).step_by(2) {
            let a = 0.10 + 0.10 * ((rx as f64 / 7.0 + t * 2.0).sin() * 0.5 + 0.5);
            g.put_alpha(rx, rule_y, '·', style.tint, a);
        }
    }
}
