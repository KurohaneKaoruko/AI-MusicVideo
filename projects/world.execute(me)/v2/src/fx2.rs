//! v2 代码特效：token 雨 / 行涟漪 / 转圈指示器 / 文本仪表。
//!
//! 舞台上永远只有"代码与编辑器部件"，特效也全部由代码字符构成：
//! 雨是 falling tokens，涟漪是行位移，仪表是注释里的进度块。

// 工具箱按幕取用
#![allow(dead_code)]

use crate::buf::{Canvas, Rect, Rgb, SKIP};
use crate::fx::Rng;
use crate::theme;

/// 代码专用字符集（雨 / 噪声用，比 v1 的 RAIN 更"源码"）
pub const CODE: &[char] = &[
    'w', 'o', 'r', 'l', 'd', 'e', 'x', 'c', 'u', 't', 'm', '(', ')', '{', '}', '[', ']', ';', ':',
    ':', '-', '>', '=', '<', '+', '*', '/', '!', '&', '|', '#', '_', '0', '1', '2', '3', '…', '·',
];

/// braille 转圈（rust-analyzer 正在思考…）
pub const SPIN: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

#[inline]
pub fn spinner(frame: u64) -> char {
    SPIN[(frame / 6 % 10) as usize]
}

/// 已打出的字符数（列数）
#[inline]
pub fn typed_char(lt: f32, t0: f32, cps: f32) -> i32 {
    crate::code::typed(lt, t0, cps)
}

/// 文本进度仪表（注释里那种）：`▓▓▓▓░░░░ 62%`
pub fn gauge(ratio: f32, w: i32) -> (String, Rgb) {
    let r = ratio.clamp(0.0, 1.0);
    let full = (r * w as f32).round() as i32;
    let mut s = String::new();
    for i in 0..w {
        s.push(if i < full { '▓' } else { '░' });
    }
    let col = if r >= 1.0 {
        theme::GREEN
    } else if r > 0.8 {
        theme::AMBER
    } else {
        theme::CYAN
    };
    (s, col)
}

// ── token 雨 ────────────────────────────────────────────────
struct Col {
    y: f32,
    speed: f32,
    len: i32,
    ch: Vec<char>,
}

/// 下落的代码 token 列。稀疏、暗淡，只在标题 / 处刑等段落出现。
pub struct TokenRain {
    cols: Vec<Col>,
    w: i32,
    h: i32,
}

impl TokenRain {
    pub fn new(w: i32, h: i32, rng: &mut Rng) -> Self {
        let mut cols = Vec::new();
        for _ in 0..w {
            cols.push(Col {
                y: rng.range(-60.0, h as f32),
                speed: rng.range(3.0, 14.0),
                len: rng.irange(4, 18),
                ch: (0..48)
                    .map(|_| CODE[(rng.next_u64() % CODE.len() as u64) as usize])
                    .collect(),
            });
        }
        TokenRain { cols, w, h }
    }

    pub fn draw(&mut self, cv: &mut Canvas, rect: Rect, dt: f32, head: Rgb, alpha: f32) {
        if rect.w != self.w || rect.h != self.h {
            return;
        }
        let tail = head.mul(0.16);
        for (i, c) in self.cols.iter_mut().enumerate() {
            c.y += c.speed * dt;
            if c.y - c.len as f32 > self.h as f32 {
                c.y = -c.len as f32;
                c.len = 4 + (i as i32 * 5 % 14);
            }
            let head_y = c.y as i32;
            for k in 0..c.len {
                let y = head_y - k;
                if y < rect.y || y > rect.bottom() {
                    continue;
                }
                let f = if k == 0 {
                    1.0
                } else {
                    (1.0 - k as f32 / c.len as f32).powf(1.5)
                };
                let col = head.mix(tail, 1.0 - f);
                let x = rect.x + i as i32;
                let e = cv.get(x, y).copied().unwrap_or_default();
                if e.ch != ' ' && e.ch != SKIP && f < 0.9 {
                    continue; // 不压代码
                }
                let ch = c.ch[((y.max(0) + k) as usize) % c.ch.len()];
                let a = (f * alpha).clamp(0.0, 1.0);
                let fg = e.fg.mix(col, a);
                cv.put(x, y, ch, fg, e.bg);
            }
        }
    }
}

// ── 行涟漪 ──────────────────────────────────────────────────
/// 以 cy 为中心的水平冲击波：逐行正弦位移 + 亮化。
/// 纯代码界面的"落章冲击波"。
pub fn ripple(cv: &mut Canvas, cy: i32, radius: f32, band: f32, amt: f32) {
    for y in 0..cv.h {
        let d = (y - cy) as f32;
        let k = 1.0 - (d / band).abs();
        if k <= 0.0 {
            continue;
        }
        let shift = (k * amt).round() as i32 * if d < 0.0 { 1 } else { -1 };
        if shift != 0 {
            cv.row_shift(y, shift);
        }
        if d.abs() < radius {
            let row = y * cv.w;
            for x in 0..cv.w {
                let c = &mut cv.cells[(row + x) as usize];
                c.fg = c.fg.mix(theme::WHITE, 0.25 * k);
            }
        }
    }
}

// ── 打字辅助 ────────────────────────────────────────────────
/// 一行文本在 t0 之后按 cps 打字时，光标前的可见文本
pub fn typed_str(s: &str, lt: f32, t0: f32, cps: f32) -> String {
    let n = crate::code::typed(lt, t0, cps);
    let mut o = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let c = crate::buf::cw(ch);
        if w + c > n {
            break;
        }
        o.push(ch);
        w += c;
    }
    o
}

/// 字符串的显示宽度（缓冲用）
pub fn wid(s: &str) -> i32 {
    crate::buf::sw(s)
}
