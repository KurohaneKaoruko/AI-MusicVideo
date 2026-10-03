//! Character-grid layer: colors, cells, drawing primitives.
//! The grid is the "terminal" — 192x45 cells of 10x24 px each = 1920x1080.
//! Pixel-space rendering (glyphs, sprites, post FX) lives in `pix.rs`.

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
        Color::lerp(self, Color::rgb(240, 250, 255), amount)
    }
}

// ------------------------------------------------------------- CRYMACHINA palette ---

pub mod pal {
    pub use super::Color;
    pub const BG0: u32 = 0x04060d; // deep space navy (base background)
    pub const BG1: u32 = 0x0a1020; // panel
    pub const DIM: u32 = 0x2e3a56; // faint text
    pub const MID: u32 = 0x6b7a9e; // secondary text
    pub const BRIGHT: u32 = 0xdce6f8; // primary text
    pub const WHITE: u32 = 0xf4f8ff;
    pub const ICE: u32 = 0x66d4ff; // Enoa / blue fairy
    pub const ICE_DEEP: u32 = 0x2b8fc4;
    pub const ICE_PALE: u32 = 0xcfeeff;
    pub const GOLD: u32 = 0xffd27a; // souls / E.V.E / E x P
    pub const GOLD_DEEP: u32 = 0xb8863b;
    pub const RED: u32 = 0xff5468; // alerts / Leben
    pub const AMBER: u32 = 0xffb454; // Mikoto
    pub const GREEN: u32 = 0x7ee2a8; // Ami
    pub const TEAR: u32 = 0xbfe9ff; // tears
    pub const STAR: u32 = 0xfff3d6; // the star inside Eden

    pub fn c(h: u32) -> Color {
        Color::hex(h)
    }
}

pub fn clamp01(t: f64) -> f64 {
    t.clamp(0.0, 1.0)
}

// ---------------------------------------------------------------- easing ---

pub fn ease_in_cubic(t: f64) -> f64 {
    let t = clamp01(t);
    t * t * t
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

// ------------------------------------------------------------------- rng ---

/// Tiny deterministic xorshift — same seed, same picture (renders stay stable).
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

/// Deterministic hash noise 0..1 from integer coordinates.
pub fn hash01(x: u64) -> f64 {
    let mut z = x.wrapping_mul(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    (z ^ (z >> 31)) as f64 / u64::MAX as f64
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
}

impl Grid {
    pub fn new(w: usize, h: usize, bg: Color) -> Grid {
        let mut g = Grid {
            w,
            h,
            cells: Vec::new(),
            bg,
        };
        g.cells
            .resize(w * h, Cell { ch: ' ', fg: bg, bg, bold: false });
        g
    }

    pub fn clear(&mut self, bg: Color) {
        self.bg = bg;
        for c in self.cells.iter_mut() {
            c.ch = ' ';
            c.fg = bg;
            c.bg = bg;
            c.bold = false;
        }
    }

    #[inline]
    pub fn in_bounds(&self, x: i64, y: i64) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h
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
        if alpha <= 0.03 || ch == '\0' {
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

    pub fn fill_bg(&mut self, x0: i64, y0: i64, w: i64, h: i64, bg: Color) {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                if !self.in_bounds(x, y) {
                    continue;
                }
                self.cells[y as usize * self.w + x as usize].bg = bg;
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

    /// Draw text; chars wider than 1 cell emit '\0' continuations. Returns x after.
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
            for k in 1..w {
                if self.in_bounds(x + k, y) && alpha > 0.03 {
                    // keep wide-char continuation blank but colored
                    let i2 = y as usize * self.w + (x + k) as usize;
                    self.cells[i2].ch = ' ';
                }
            }
            x += w;
        }
        x
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

    /// ANSI serialization for the interactive live player.
    pub fn serialize(&self, out: &mut String) {
        use std::fmt::Write as _;
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

/// Terminal cell width of a char: 2 for East-Asian Wide/Fullwidth, else 1.
///
/// Only Unicode's `W`/`F` classes count. The `A` (Ambiguous) class — arrows,
/// math operators, geometric shapes, dingbats — is deliberately treated as 1:
/// those are single-cell glyphs in Consolas, the face the film is set in. Counting
/// them as wide, as this used to, gave `√ ▼ ◆ ● ○ ★ ♪ ♥` a two-cell slot *and*
/// switched them to the CJK face, so a marker dropped into a Latin line rendered
/// from a different typeface at double width. That stray offset is what reads as
/// a skewed character grid.
pub fn char_w(c: char) -> usize {
    let cp = c as u32;
    if cp == 0 {
        return 0;
    }
    let wide = matches!(cp,
        // Hangul Jamo
        0x1100..=0x115F
        // CJK radicals · Kangxi · ideographic description · CJK punctuation
        | 0x2E80..=0x303E
        // kana · bopomofo · hangul compat jamo · kanbun · enclosed CJK · CJK compat
        | 0x3041..=0x33FF
        // CJK unified ideographs (+ ext A, compat)
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xA000..=0xA4CF
        // Hangul syllables
        | 0xAC00..=0xD7A3
        // CJK compatibility ideographs
        | 0xF900..=0xFAFF
        // vertical forms · CJK compatibility forms
        | 0xFE10..=0xFE19
        | 0xFE30..=0xFE6F
        // fullwidth forms
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        // enclosed alphanumerics (①..⑳ are Wide)
        | 0x2460..=0x24FF
        // emoji
        | 0x1F300..=0x1F64F
        | 0x1F900..=0x1F9FF
    );
    if wide {
        2
    } else {
        1
    }
}
