//! 界面外壳：v2 把整台"终端"伪装成一个代码编辑器。
//!
//! ┌ 标签页栏 ─ me.rs ● │ world.rs │ love.rs        main*   ● user@localhost ┐
//! │  编辑区（场景 = 编码现场）                                                │
//! ├ 歌词条 EN：// Switch on the power line [PROTECTION] …（关键词仍上色块）    │
//! ├ 歌词条 ZH                                                                 │
//! ├ 进度条（带场景刻度）                                                      │
//! └ 状态栏：⎇ main x3 ⚠7 │ Ln 42, Col 17 │ Rust │ rust-analyzer ✓ │ ▶ 01:23  │
//!
//! 隐藏叙事沿用 v1：唱到"你走了"之后，右下角的 user@localhost 逐步掉线。

use crate::buf::{Canvas, Rect, Rgb, cw, sw};
use crate::fx::{self, RAIN};
use crate::lyrics::{Line, Lyrics};
use crate::theme::{self, Kw};
use crate::view::{Link, View};

pub struct Layout {
    pub stage: Rect,
    pub y_en: i32,
    pub y_zh: i32,
    pub y_progress: i32,
}

impl Layout {
    pub fn compute(w: i32, h: i32) -> Self {
        Layout {
            stage: Rect::new(0, 1, w, (h - 5).max(3)),
            y_en: h - 4,
            y_zh: h - 3,
            y_progress: h - 2,
        }
    }
    pub fn min_size_ok(w: i32, h: i32) -> bool {
        w >= 72 && h >= 22
    }
}

pub fn time_str(t: f32) -> String {
    let t = t.max(0.0);
    let m = (t / 60.0) as i32;
    let s = t - m as f32 * 60.0;
    format!("{m:02}:{s:04.1}")
}

/// 按列宽揭示文本：已揭示的正常显示，未揭示的留空，交界处放乱码。
pub fn reveal_str(text: &str, revealed: i32, rng: &mut fx::Rng, wide_step: bool) -> String {
    let mut o = String::with_capacity(text.len());
    let mut col = 0i32;
    for ch in text.chars() {
        let w = cw(ch);
        if w == 0 {
            continue;
        }
        let step = if wide_step { w } else { w };
        if col + step <= revealed {
            o.push(ch);
        } else if col >= revealed {
            o.push(' ');
        } else if ch == ' ' {
            o.push(' ');
        } else {
            o.push(*rng.pick(RAIN));
        }
        col += step;
    }
    o
}

// ── IDE 状态（随剧情演化的"假状态栏"）──────────────────────
struct IdeState {
    errors: i32,
    warnings: i32,
    /// 状态消息 + 是否带转圈
    status: &'static str,
    busy: bool,
    /// me.rs 标签页是否未保存
    dirty: bool,
    /// 文件是否已被关闭（终局）
    closed: bool,
}

fn ide_state(v: &View) -> IdeState {
    let t = v.t;
    let (errors, warnings) = if t < 64.0 {
        (0, 0)
    } else if t < 74.0 {
        (1, 0)
    } else if t < 89.2 {
        (1, 3)
    } else if t < 110.9 {
        (1, 1)
    } else if t < 118.3 {
        (1, 2)
    } else if t < 125.7 {
        (7, 2)
    } else if t < 133.3 {
        (9, 3)
    } else if t < 162.6 {
        (12, 3)
    } else if t < 191.4 {
        (13, 3)
    } else if t < 205.8 {
        (14, 3)
    } else {
        (0, 0)
    };
    let (status, busy) = if t < 3.0 {
        ("rust-analyzer: indexing crate graph…", true)
    } else if t < 16.0 {
        ("building me…", true)
    } else if t < 29.7 {
        ("running `me`", false)
    } else if (133.3..147.7).contains(&t) {
        ("cargo check --future-incompat…", true)
    } else if (162.6..177.2).contains(&t) {
        ("cargo build --executions 12", true)
    } else if (89.2..101.5).contains(&t) {
        ("refactoring gender…", true)
    } else {
        ("rust-analyzer ✓", false)
    };
    IdeState {
        errors,
        warnings,
        status,
        busy,
        dirty: t >= 110.9,
        closed: v.finished || t >= 209.4,
    }
}

// ── 标签页栏（第 0 行）──────────────────────────────────────
pub fn draw_header(cv: &mut Canvas, v: &View, ly: &Lyrics, dur: f32) {
    let w = cv.w;
    let bg = theme::PANEL;
    for x in 0..w {
        cv.put(x, 0, ' ', theme::TEXT_DIM, bg);
    }
    let st = ide_state(v);

    // ── 标签页 ──
    let mut x = 1;
    if st.closed {
        let t1 = " (no file open) ";
        x = cv.text(x, 0, t1, theme::TEXT_FAINT, bg) + 1;
    } else {
        // 活动标签：me.rs（带未保存圆点 / 终局前的 ×）
        let name = if st.dirty { "me.rs ●" } else { "me.rs ×" };
        let tw = sw(name) + 2;
        let tab_bg = theme::BG;
        let nc = if st.dirty { theme::AMBER } else { theme::CYAN };
        cv.fill(Rect::new(x, 0, tw, 1), ' ', nc, tab_bg);
        cv.text(x + 1, 0, name, nc, tab_bg);
        cv.put(x, 0, '▐', nc.mul(0.9), nc.mul(0.4));
        cv.put(x + tw - 1, 0, '▌', nc.mul(0.9), nc.mul(0.4));
        x += tw;
        // 非活动标签
        for other in ["world.rs ", "love.rs "] {
            let s = format!("{other}");
            cv.text(x + 1, 0, &s, theme::TEXT_FAINT, bg);
            x += sw(&s) + 2;
        }
        let plus = "＋";
        cv.text(x + 1, 0, plus, theme::TEXT_FAINT, bg);
        x += sw(plus) + 2;
    }

    // 幕号 + 幕名（MV 观感，放在标签区右侧）
    let scene = format!(
        "── {:02}/{:02} {}",
        v.scene_idx + 1,
        v.scene_count,
        v.scene_name
    );
    if x + sw(&scene) + 2 < w / 2 {
        cv.text(x + 2, 0, &scene, theme::CYAN.mul(0.55), bg);
    }

    // ── 右侧：分支 / 语言 / 链路（隐藏叙事）──
    let mut right: Vec<(String, Rgb)> = Vec::new();
    let (link_s, link_c) = match v.link {
        Link::Linked => ("● user@localhost".to_string(), theme::GREEN),
        Link::Unstable => ("◐ link unstable".to_string(), theme::AMBER),
        Link::Lost => ("○ NO CARRIER".to_string(), theme::RED.mul(0.9)),
    };
    right.push((link_s, link_c));
    right.push(("main*".to_string(), theme::AMBER.mul(0.85)));
    right.push(("Rust".to_string(), theme::TEXT_DIM));

    let mut rx = w - 2;
    for (s, c) in right {
        let wdt = sw(&s);
        rx -= wdt;
        if rx < x + 2 {
            break;
        }
        cv.text(rx, 0, &s, c, bg);
        rx -= 2;
    }
    let _ = ly;
    let _ = dur;
}

// ── 歌词条（代码注释形态）──────────────────────────────────
struct Piece {
    x: i32,
    s: String,
    kw: Option<Kw>,
    w: i32,
}

pub fn draw_lyrics(cv: &mut Canvas, v: &View, ly: &Lyrics, ye: i32, yz: i32) {
    let w = cv.w;
    for y in [ye, yz] {
        for x in 0..w {
            let d = ((x - w / 2) as f32 / (w as f32 / 2.0)).abs();
            let c = theme::PANEL.mix(theme::BG, 0.5 + 0.35 * d);
            cv.put(x, y, ' ', theme::TEXT_DIM, c);
        }
    }

    let idx = match v.lyric_idx {
        Some(i) => i,
        None => {
            let dots = (v.t * 2.0) as i32 % 4;
            let s = format!("// {}▮", "· ".repeat(dots as usize));
            cv.text_center(w / 2, ye, &s, theme::TEXT_FAINT, theme::PANEL);
            cv.text_center(w / 2, yz, "//! 等待信号…", theme::TEXT_FAINT, theme::PANEL);
            return;
        }
    };
    let l: &Line = &ly.lines[idx];
    let el = v.elapsed_in_lyric();
    let flash = fx::pulse(el, 0.02, 0.34);

    // 换行瞬间的横向亮带
    if flash > 0.01 {
        let rad = (flash * w as f32 * 0.62) as i32;
        let c = w / 2;
        for x in (c - rad)..(c + rad) {
            if x < 0 || x >= w {
                continue;
            }
            let cell = *cv.get(x, ye).unwrap_or(&Default::default());
            let nb = cell.bg.mix(theme::CYAN.mul(0.35), flash * 0.45);
            cv.put(x, ye, ' ', cell.fg, nb);
        }
    }

    // 注释前缀占 3 列
    let prefix_w = 3;
    let mut pieces: Vec<Piece> = Vec::new();
    let mut x = prefix_w;
    for (i, tok) in l.tokens.iter().enumerate() {
        match tok.kw {
            Some(_) => {
                if i > 0 {
                    x += 1;
                }
                let word = tok.s.trim().to_string();
                let wd = sw(&word) + 2;
                pieces.push(Piece {
                    x,
                    s: word,
                    kw: tok.kw,
                    w: wd,
                });
                x += wd;
            }
            None => {
                let wd = sw(&tok.s);
                pieces.push(Piece {
                    x,
                    s: tok.s.clone(),
                    kw: None,
                    w: wd,
                });
                x += wd;
            }
        }
    }
    let total = x;
    let sx = ((w - total) / 2).max(1);

    let rev = fx::reveal_count(el, 54.0, 1_000);
    let mut rng = fx::Rng::new(v.frame * 7919 + 13);
    let mut rng2 = fx::Rng::new(v.frame * 104_729 + 7);

    // `//` 前缀
    let pf_col = theme::SYN_CMT.mul(0.9 + 0.5 * v.bass);
    cv.text(sx - prefix_w, ye, "// ", pf_col, theme::PANEL);

    for p in &pieces {
        let px = sx + p.x;
        match p.kw {
            Some(k) => {
                let (fg, cbg) = k.colors();
                let chip_rev = (rev - p.x).clamp(0, p.w);
                let decoding = chip_rev >= p.w && el < 0.18;
                let bgc = if chip_rev > 0 {
                    cbg.mul(0.85 + 0.4 * v.hit)
                } else {
                    theme::PANEL
                };
                for i in 0..p.w {
                    let cx = px + i;
                    if cx < 0 || cx >= w {
                        continue;
                    }
                    cv.put(cx, ye, ' ', fg, bgc);
                }
                if chip_rev > 0 {
                    let shown = if decoding {
                        fx::scramble(&p.s, 1, &mut rng, false)
                    } else {
                        p.s.clone()
                    };
                    let s = reveal_str(&shown, chip_rev - 1, &mut rng2, false);
                    cv.text(px + 1, ye, &s, fg, bgc);
                    let g = k.accent();
                    for i in 0..(p.w - 1).max(0) {
                        cv.glow(px + 1 + i, ye, g, 0.2);
                    }
                }
            }
            None => {
                let s = reveal_str(&p.s, rev - p.x, &mut rng2, false);
                let col = theme::TEXT.mix(theme::WHITE, 0.30 * flash);
                cv.text(px, ye, &s, col, theme::PANEL);
            }
        }
    }

    let accent = pieces
        .iter()
        .find_map(|p| p.kw)
        .map(|k| k.accent())
        .unwrap_or(theme::CYAN);
    let a = 0.35 + 0.65 * v.bass;
    cv.put(0, ye, '▌', accent.mul(a), theme::PANEL);
    cv.put(w - 1, ye, '▐', accent.mul(a), theme::PANEL);

    // 中文行（文档注释形态）
    let zrev = fx::reveal_count(el - 0.2, 46.0, 1_000);
    if let Some(zh) = &l.zh {
        let zfull = format!("//! {zh}");
        let zw = sw(&zfull);
        let zx = ((w - zw) / 2).max(2);
        let shown = reveal_str(&zfull, zrev, &mut rng, false);
        let fade = ((el - 0.2) / 0.5).clamp(0.0, 1.0);
        let col = theme::SYN_DOC.mix(theme::TEXT.mix(theme::CYAN, 0.35), fade);
        cv.text(zx, yz, &shown, col, theme::PANEL);
    } else if l.en.contains("execute") {
        cv.text_center(
            w / 2,
            yz,
            "/// world.execute(me); — 参数是「我」",
            theme::TEXT_FAINT,
            theme::PANEL,
        );
    }
}

// ── 进度条 ──────────────────────────────────────────────────
pub fn draw_progress(cv: &mut Canvas, v: &View, y: i32, dur: f32, marks: &[(f32, &'static str)]) {
    let w = cv.w;
    for x in 0..w {
        cv.put(x, y, ' ', theme::TEXT_DIM, theme::BG);
    }
    let lchip = format!(" {} ", time_str(v.t));
    let rchip = format!(" -{} ", time_str((dur - v.t).max(0.0)));
    let lw = sw(&lchip);
    let rw = sw(&rchip);
    cv.text(1, y, &lchip, theme::VOID, theme::PANEL_HI.mul(0.75));
    cv.text(
        w - 1 - rw,
        y,
        &rchip,
        theme::TEXT_DIM,
        theme::PANEL_HI.mul(0.5),
    );

    let x0 = 1 + lw + 1;
    let x1 = w - 2 - rw;
    let bw = (x1 - x0).max(4);
    let ratio = (v.t / dur.max(0.001)).clamp(0.0, 1.0);
    let pos = x0 + (ratio * (bw - 1) as f32) as i32;

    for i in 0..bw {
        let x = x0 + i;
        if x <= pos {
            let t = (i as f32 / bw as f32 * 0.75 + v.bass * 0.3).min(1.0);
            cv.put(x, y, '━', theme::heat(t).mul(0.95), theme::BG);
        } else {
            cv.put(x, y, '╌', theme::BLUE_DIM.mul(0.8), theme::BG);
        }
    }
    for (mt, _) in marks {
        let x = x0 + ((mt / dur.max(0.001)).clamp(0.0, 1.0) * (bw - 1) as f32) as i32;
        if x > pos + 1 && x < x1 {
            cv.put(x, y, '┊', theme::BLUE.mul(0.9), theme::BG);
        }
    }
    let ch = if v.paused { '▮' } else { '◆' };
    let pulse_col = theme::WHITE.mix(theme::CYAN, 0.3 + 0.6 * v.hit);
    if pos >= x0 && pos <= x1 {
        cv.put(pos, y, ch, pulse_col, theme::BG);
    }
}

// ── 状态栏（最后一行）──────────────────────────────────────
pub fn draw_status(cv: &mut Canvas, v: &View, ly: &Lyrics, dur: f32) {
    let w = cv.w;
    let st = ide_state(v);
    let y = cv.h - 1;
    for x in 0..w {
        cv.put(x, y, ' ', theme::TEXT_DIM, theme::PANEL);
    }
    let mut x = 0;

    let seg = |cv: &mut Canvas, x: &mut i32, s: &str, fg: Rgb, bg: Rgb, pad: i32| {
        let wd = sw(s) + pad * 2;
        if *x + wd > w {
            return;
        }
        cv.fill(Rect::new(*x, y, wd, 1), ' ', fg, bg);
        cv.text(*x + pad, y, s, fg, bg);
        *x += wd;
    };

    seg(cv, &mut x, " main ", theme::VOID, theme::BLUE.mul(0.75), 0);
    // 诊断计数
    let err_bg = if st.errors > 0 {
        theme::RED.mul(0.85)
    } else {
        theme::PANEL_HI.mul(0.6)
    };
    seg(
        cv,
        &mut x,
        &format!(" E:{} W:{} ", st.errors, st.warnings),
        if st.errors > 0 {
            theme::WHITE
        } else {
            theme::TEXT_DIM
        },
        err_bg,
        0,
    );
    // 行:列（跟着打字节奏跳动）
    let ln = 1 + ((v.t * 3.1) as i32 % 97);
    let col = 1 + (((v.t * 60.0) as i32 * 7 + 13) % 58);
    seg(
        cv,
        &mut x,
        &format!(" Ln {ln}, Col {col} "),
        theme::TEXT_DIM,
        theme::PANEL,
        0,
    );
    seg(
        cv,
        &mut x,
        " Spaces: 4  UTF-8  LF  Rust ",
        theme::TEXT_FAINT,
        theme::PANEL,
        0,
    );

    // rust-analyzer 状态（带转圈）
    let (msg, mc) = if v.finished {
        (" rust-analyzer: done. ".to_string(), theme::GREEN.mul(0.9))
    } else {
        let sp = if st.busy {
            format!("{} ", crate::fx2::spinner(v.frame))
        } else {
            String::new()
        };
        (
            format!(" {sp}{} ", st.status),
            if st.busy {
                theme::AMBER
            } else {
                theme::GREEN.mul(0.9)
            },
        )
    };
    seg(cv, &mut x, &msg, mc, theme::PANEL, 0);

    // 右侧：播放状态 + 时间
    let state = if v.finished {
        ("■ EOF", theme::TEXT_FAINT)
    } else if v.paused {
        ("|| PAUSED", theme::AMBER)
    } else {
        ("▶ PLAY", theme::GREEN)
    };
    let rt = format!(" {} {}/{} ", state.0, time_str(v.t), time_str(dur));
    let rx = w - sw(&rt) - 1;
    cv.text(rx.max(x + 1), y, &rt, state.1, theme::PANEL_HI.mul(0.7));
    let _ = ly;
}

// ── 帮助浮层 ────────────────────────────────────────────────
pub fn draw_help(cv: &mut Canvas) {
    let lines = [
        ("SPACE", "暂停 / 继续"),
        ("←  →", "后退 / 前进 5 秒"),
        ("R", "从头重播"),
        ("H", "开关本帮助"),
        ("Q / ESC", "退出"),
        ("", ""),
        ("", "v2「SOURCE」：整部 MV 是一个 .rs 文件。"),
        ("", "歌词是注释，剧情是编译，死亡是 drop。"),
    ];
    let w = 62;
    let h = lines.len() as i32 + 4;
    let r = Rect::new((cv.w - w) / 2, (cv.h - h) / 2, w, h);
    cv.fill(r, ' ', theme::TEXT, theme::VOID);
    cv.frame(
        r,
        crate::buf::Frame::Rounded,
        theme::CYAN.mul(0.8),
        theme::VOID,
    );
    cv.text(
        r.x + 3,
        r.y,
        " KEYBOARD SHORTCUTS ",
        theme::CYAN,
        theme::VOID,
    );
    for (i, (k, d)) in lines.iter().enumerate() {
        let y = r.y + 2 + i as i32;
        if !k.is_empty() {
            cv.text(r.x + 5, y, k, theme::AMBER, theme::VOID);
        }
        cv.text(r.x + 17, y, d, theme::TEXT_DIM, theme::VOID);
    }
}

/// 启动前的标题卡：一次 `cargo run` 开机 + 大字标题
pub fn draw_title_card(cv: &mut Canvas, hint: &str, t: f32) {
    let cx = cv.w / 2;
    let cy = cv.h / 2;
    // 伪 cargo 启动日志
    let boot = [
        "$ cargo run --release",
        "   Compiling world v0.4.9",
        "    Building `me` (src/me.rs)",
        "    Finished `release` in 3.42s",
        "     Running `me`…",
    ];
    let total_cps = 60.0;
    let mut acc = 0.0;
    for (i, l) in boot.iter().enumerate() {
        let shown = fx::reveal_count(t - i as f32 * 0.28, total_cps, sw(l) as i32);
        acc += sw(l) as f32;
        let col = match i {
            0 => theme::TEXT,
            4 => theme::GREEN,
            _ => theme::TEXT_DIM,
        };
        cv.text(
            4,
            cy - 9 + i as i32,
            &reveal_str(l, shown, &mut fx::Rng::new(i as u64 + 7), false),
            col,
            theme::BG,
        );
    }
    let big_y = cy - 1;
    let title = "world.execute(me);";
    let rev = fx::reveal_count(t - 1.6, 9.0, sw(title) as i32);
    crate::bigfont::draw_reveal(
        cv,
        cx - crate::bigfont::width(title, 1) / 2,
        big_y,
        title,
        rev as f32,
        theme::CYAN,
        theme::PURPLE_DIM,
        theme::BG,
        1,
    );
    cv.text_center(
        cx,
        big_y + 6,
        "// v2 「SOURCE」— 全程由 Rust 代码演绎",
        theme::SYN_DOC,
        theme::BG,
    );
    let blink = ((t * 1.6) as i32) % 2 == 0;
    if blink && t > 2.6 {
        cv.text_center(cx, big_y + 8, hint, theme::AMBER, theme::BG);
    }
    cv.text_center(
        cx,
        big_y + 10,
        "h 查看操作 · 基于 Mili《world.execute(me);》",
        theme::TEXT_FAINT,
        theme::BG,
    );
    let _ = acc;
}

/// 终端太小
pub fn draw_too_small(cv: &mut Canvas) {
    cv.clear();
    let msg = "// 终端窗口过小，请放大到至少 72 × 22";
    cv.text_center(cv.w / 2, cv.h / 2, msg, theme::AMBER, theme::BG);
    cv.text_center(
        cv.w / 2,
        cv.h / 2 + 2,
        "terminal too small — resize to at least 72x22",
        theme::TEXT_DIM,
        theme::BG,
    );
}
