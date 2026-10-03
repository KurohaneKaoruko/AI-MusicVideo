//! Character-grid graphics: colors, cells, drawing primitives and ANSI serialization.

use std::fmt::Write as _;

// ---------------------------------------------------------------- colors ---

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
    pub const fn hex(h: u32) -> Color {
        Color {
            r: (h >> 16) as u8,
            g: (h >> 8) as u8,
            b: h as u8,
        }
    }
    pub fn lerp(a: Color, b: Color, t: f64) -> Color {
        let t = clamp01(t);
        Color {
            r: (a.r as f64 + (b.r as f64 - a.r as f64) * t) as u8,
            g: (a.g as f64 + (b.g as f64 - a.g as f64) * t) as u8,
            b: (a.b as f64 + (b.b as f64 - a.b as f64) * t) as u8,
        }
    }
    /// Multiply brightness.
    pub fn scale(self, f: f64) -> Color {
        Color {
            r: (self.r as f64 * f).clamp(0.0, 255.0) as u8,
            g: (self.g as f64 * f).clamp(0.0, 255.0) as u8,
            b: (self.b as f64 * f).clamp(0.0, 255.0) as u8,
        }
    }
    /// Blend `self` over `bg` with alpha.
    pub fn over(self, bg: Color, alpha: f64) -> Color {
        Color::lerp(bg, self, alpha)
    }
    /// Mix toward white.
    pub fn glow(self, amount: f64) -> Color {
        Color::lerp(self, Color::rgb(255, 250, 244), amount)
    }
    /// Perceived luminance 0..1.
    pub fn lum(self) -> f64 {
        (0.299 * self.r as f64 + 0.587 * self.g as f64 + 0.114 * self.b as f64) / 255.0
    }
}

pub fn clamp01(t: f64) -> f64 {
    t.clamp(0.0, 1.0)
}

// ---------------------------------------------------------------- easing ---

pub fn ease_out_cubic(t: f64) -> f64 {
    let t = clamp01(t);
    1.0 - (1.0 - t).powi(3)
}
pub fn ease_in_out(t: f64) -> f64 {
    let t = clamp01(t);
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}
pub fn ease_out_back(t: f64) -> f64 {
    let t = clamp01(t);
    let c = 1.70158;
    1.0 + (c + 1.0) * (t - 1.0).powi(3) + c * (t - 1.0).powi(2)
}
#[allow(dead_code)]
pub fn bounce(t: f64) -> f64 {
    let t = clamp01(t);
    (1.0 - t) * (t * 6.0 * std::f64::consts::PI).sin().abs()
}

// ------------------------------------------------------------------- rng ---

/// Tiny deterministic xorshift — same seed, same picture (preview captures stay stable).
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.max(1))
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.f64()
    }
    pub fn i64(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u64() % ((hi - lo + 1) as u64)) as i64
    }
}

// ---------------------------------------------------------------- cells ---

#[derive(Clone, Copy)]
pub struct Cell {
    /// `\0` = continuation of a wide char (skip emission).
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
}

pub struct Grid {
    pub w: usize,
    pub h: usize,
    pub cells: Vec<Cell>,
    pub bg: Color,
    bg_dark: Color,
}

impl Grid {
    pub fn new(w: usize, h: usize, bg: Color) -> Grid {
        let mut g = Grid {
            w,
            h,
            cells: Vec::new(),
            bg,
            bg_dark: bg.scale(0.72),
        };
        g.cells.resize(w * h, Cell { ch: ' ', fg: Color::rgb(0, 0, 0), bg, bold: false });
        g
    }

    #[allow(dead_code)]
    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        self.cells.clear();
        self.cells.resize(w * h, Cell { ch: ' ', fg: Color::rgb(0, 0, 0), bg: self.bg, bold: false });
    }

    pub fn clear(&mut self) {
        // deep background with faint CRT scanlines
        for y in 0..self.h {
            let row_bg = if y % 3 == 1 { self.bg_dark } else { self.bg };
            for x in 0..self.w {
                let c = &mut self.cells[y * self.w + x];
                c.ch = ' ';
                c.fg = row_bg;
                c.bg = row_bg;
                c.bold = false;
            }
        }
    }

    #[inline]
    pub fn in_bounds(&self, x: i64, y: i64) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h
    }

    #[inline]
    pub fn row_bg(&self, y: usize) -> Color {
        if y % 3 == 1 {
            self.bg_dark
        } else {
            self.bg
        }
    }

    #[inline]
    pub fn put(&mut self, x: i64, y: i64, ch: char, fg: Color) {
        if !self.in_bounds(x, y) || ch == '\0' {
            return;
        }
        let i = y as usize * self.w + x as usize;
        let c = &mut self.cells[i];
        c.ch = ch;
        c.fg = fg;
        c.bold = false;
    }

    pub fn put_bold(&mut self, x: i64, y: i64, ch: char, fg: Color) {
        if !self.in_bounds(x, y) || ch == '\0' {
            return;
        }
        let i = y as usize * self.w + x as usize;
        let c = &mut self.cells[i];
        c.ch = ch;
        c.fg = fg;
        c.bold = true;
    }

    /// Paint with alpha over current background (for particles / ghosts).
    pub fn put_alpha(&mut self, x: i64, y: i64, ch: char, fg: Color, alpha: f64) {
        if alpha <= 0.03 {
            return;
        }
        if !self.in_bounds(x, y) {
            return;
        }
        let i = y as usize * self.w + x as usize;
        let bg = self.cells[i].bg;
        let f = fg.over(bg, clamp01(alpha));
        self.put(x, y, ch, f);
    }

    pub fn fill(&mut self, x0: i64, y0: i64, w: i64, h: i64, ch: char, fg: Color, bg: Option<Color>) {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                if !self.in_bounds(x, y) {
                    continue;
                }
                let i = y as usize * self.w + x as usize;
                let c = &mut self.cells[i];
                c.ch = ch;
                c.fg = fg;
                if let Some(b) = bg {
                    c.bg = b;
                }
                c.bold = false;
            }
        }
    }

    pub fn fill_bg(&mut self, x0: i64, y0: i64, w: i64, h: i64, bg: Color) {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                if !self.in_bounds(x, y) {
                    continue;
                }
                let i = y as usize * self.w + x as usize;
                let c = &mut self.cells[i];
                c.bg = bg;
            }
        }
    }

    pub fn hline(&mut self, x0: i64, x1: i64, y: i64, ch: char, fg: Color) {
        for x in x0.min(x1)..=x1.max(x0) {
            self.put(x, y, ch, fg);
        }
    }
    pub fn vline(&mut self, x: i64, y0: i64, y1: i64, ch: char, fg: Color) {
        for y in y0.min(y1)..=y1.max(y0) {
            self.put(x, y, ch, fg);
        }
    }

    /// Width of a string in terminal cells (CJK = 2).
    pub fn measure(s: &str) -> i64 {
        s.chars().map(|c| char_w(c) as i64).sum()
    }

    /// Draw text, returns x after the last cell. Assumes monospace + our width fn.
    pub fn text(&mut self, x0: i64, y: i64, s: &str, fg: Color) -> i64 {
        let mut x = x0;
        for ch in s.chars() {
            let w = char_w(ch) as i64;
            if self.in_bounds(x, y) {
                let i = y as usize * self.w + x as usize;
                let c = &mut self.cells[i];
                c.ch = ch;
                c.fg = fg;
                c.bold = false;
                for k in 1..w {
                    let kx = x + k;
                    if kx >= 0 && (kx as usize) < self.w {
                        let i2 = y as usize * self.w + kx as usize;
                        self.cells[i2].ch = '\0';
                        self.cells[i2].fg = fg;
                    }
                }
            }
            x += w;
        }
        x
    }

    pub fn text_alpha(&mut self, x0: i64, y: i64, s: &str, fg: Color, alpha: f64) -> i64 {
        let mut x = x0;
        for ch in s.chars() {
            let w = char_w(ch) as i64;
            self.put_alpha(x, y, ch, fg, alpha);
            x += w;
        }
        x
    }

    #[allow(dead_code)]
    pub fn text_bold(&mut self, x0: i64, y: i64, s: &str, fg: Color) -> i64 {
        let mut x = x0;
        for ch in s.chars() {
            let w = char_w(ch) as i64;
            if self.in_bounds(x, y) {
                self.put_bold(x, y, ch, fg);
                for k in 1..w {
                    let kx = x + k;
                    if kx >= 0 && (kx as usize) < self.w {
                        let i2 = y as usize * self.w + kx as usize;
                        self.cells[i2].ch = '\0';
                        self.cells[i2].fg = fg;
                    }
                }
            }
            x += w;
        }
        x
    }

    pub fn text_center(&mut self, y: i64, s: &str, fg: Color) -> i64 {
        let x = (self.w as i64 - Grid::measure(s)) / 2;
        self.text(x, y, s, fg)
    }

    #[allow(dead_code)]
    pub fn text_center_bold(&mut self, y: i64, s: &str, fg: Color) -> i64 {
        let x = (self.w as i64 - Grid::measure(s)) / 2;
        self.text_bold(x, y, s, fg)
    }

    pub fn text_right(&mut self, x1: i64, y: i64, s: &str, fg: Color) -> i64 {
        let x = x1 - Grid::measure(s);
        self.text(x, y, s, fg)
    }

    /// Rounded box with optional title. Returns inner region (x,y,w,h).
    pub fn box_rounded(
        &mut self,
        x0: i64,
        y0: i64,
        w: i64,
        h: i64,
        border: Color,
        bg: Option<Color>,
        title: Option<(&str, Color)>,
    ) -> (i64, i64, i64, i64) {
        if w < 2 || h < 2 {
            return (x0, y0, w, h);
        }
        let x1 = x0 + w - 1;
        let y1 = y0 + h - 1;
        if let Some(b) = bg {
            self.fill_bg(x0, y0, w, h, b);
        }
        self.put(x0, y0, '╭', border);
        self.put(x1, y0, '╮', border);
        self.put(x0, y1, '╰', border);
        self.put(x1, y1, '╯', border);
        self.hline(x0 + 1, x1 - 1, y0, '─', border);
        self.hline(x0 + 1, x1 - 1, y1, '─', border);
        self.vline(x0, y0 + 1, y1 - 1, '│', border);
        self.vline(x1, y0 + 1, y1 - 1, '│', border);
        if let Some((t, tc)) = title {
            let tw = Grid::measure(t);
            if tw + 2 < w - 2 {
                let tx = x0 + 2;
                self.put(tx - 1, y0, ' ', border);
                self.text(tx, y0, t, tc);
                self.put(tx + tw, y0, ' ', border);
            }
        }
        (x0 + 1, y0 + 1, w - 2, h - 2)
    }

    /// Chromatic-aberration text (red / blue fringes + white core) — for heavy moments.
    pub fn chroma_text(&mut self, _x0: i64, y: i64, s: &str, core: Color, amt: i64) {
        let w = Grid::measure(s);
        let x = (self.w as i64 - w) / 2;
        // fringes
        for (off, col) in [(-amt, Color::rgb(255, 64, 110)), (amt, Color::rgb(64, 170, 255))] {
            self.text_alpha(x + off, y, s, col, 0.55);
        }
        self.text(x, y, s, core);
    }

    /// Animated stretch-text: `head` + `fill`×n + `tail`, fill chars fade toward the tip.
    pub fn stretch_text(&mut self, y: i64, head: &str, fill: char, tail: &str, n: usize, color: Color) {
        let fill_w = char_w(fill) as i64;
        let total = Grid::measure(head) + fill_w * n as i64 + Grid::measure(tail);
        let mut x = (self.w as i64 - total) / 2;
        x = self.text(x, y, head, color);
        let bg = self.row_bg(y as usize);
        for k in 0..n {
            let fade = (1.0 - (k as f64 + 1.0) / (n.max(2) as f64) * 0.8).max(0.12);
            self.text(x, y, &fill.to_string(), color.over(bg, fade));
            x += fill_w;
        }
        self.text(x, y, tail, color);
    }

    // ------------------------------------------------------- serialization ---

    /// Run-length encoded ANSI frame. Cursor is homed per row; synchronized-update
    /// markers reduce tearing on modern terminals.
    pub fn serialize(&self, out: &mut String) {
        out.clear();
        out.push_str("\u{1b}[?2026h");
        let mut cur_fg = Color { r: !0, g: !0, b: !0 };
        let mut cur_bg = cur_fg;
        let mut cur_bold = false;
        let mut first = true;
        for y in 0..self.h {
            let _ = write!(out, "\u{1b}[{};1H", y + 1);
            for x in 0..self.w {
                let c = &self.cells[y * self.w + x];
                if c.ch == '\0' {
                    continue;
                }
                if first || c.fg != cur_fg || c.bg != cur_bg || c.bold != cur_bold {
                    cur_fg = c.fg;
                    cur_bg = c.bg;
                    cur_bold = c.bold;
                    first = false;
                    let _ = write!(
                        out,
                        "\u{1b}[{};38;2;{};{};{};48;2;{};{};{}m",
                        if c.bold { 1 } else { 22 },
                        c.fg.r,
                        c.fg.g,
                        c.fg.b,
                        c.bg.r,
                        c.bg.g,
                        c.bg.b
                    );
                }
                out.push(c.ch);
            }
        }
        out.push_str("\u{1b}[0m\u{1b}[?2026l");
    }
}

/// Terminal cell width of a char (covers CJK + fullwidth + Hangul).
pub fn char_w(c: char) -> usize {    let cp = c as u32;
    if cp == 0 {
        return 0;
    }
    if (0x1100..=0x115F).contains(&cp)
        || (0x2E80..=0x303E).contains(&cp)
        || (0x3041..=0x33FF).contains(&cp)
        || (0x3400..=0x4DBF).contains(&cp)
        || (0x4E00..=0x9FFF).contains(&cp)
        || (0xA000..=0xA4CF).contains(&cp)
        || (0xAC00..=0xD7A3).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0xFE10..=0xFE19).contains(&cp)
        || (0xFE30..=0xFE6F).contains(&cp)
        || (0xFF00..=0xFF60).contains(&cp)
        || (0xFFE0..=0xFFE6).contains(&cp)
        || (0x1F300..=0x1F64F).contains(&cp)
        || (0x1F900..=0x1F9FF).contains(&cp)
    {
        2
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sgr_sequence_is_well_formed() {
        let mut g = Grid::new(4, 2, Color::hex(0x0a0d13));
        g.put(0, 0, 'X', Color::rgb(255, 109, 138));
        let mut s = String::new();
        g.serialize(&mut s);
        // must contain the full fg+bg SGR, never a bare `;2;r;g;b` run
        let seq = s.split('\u{1b}').find(|p| p.contains('X')).unwrap();
        assert!(
            seq.starts_with("[22;38;2;255;109;138;48;2;") || seq.starts_with("[1;38;2;255;109;138;48;2;"),
            "bad SGR: {seq:?}"
        );
    }
}