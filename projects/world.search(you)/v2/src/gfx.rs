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
    /// Perceived luminance 0..1.
    pub fn lum(self) -> f64 {
        (0.299 * self.r as f64 + 0.587 * self.g as f64 + 0.114 * self.b as f64) / 255.0
    }
}

// ------------------------------------------------- world.search (you) palette ---

pub mod pal {
    pub use super::Color;
    // ---- the machine: cold slate the search program speaks in
    pub const BG0: u32 = 0x05070c; // deep night (base background)
    pub const BG1: u32 = 0x0a0f18; // panel
    pub const BG2: u32 = 0x121a26; // brighter panel / highlight fill
    pub const BG3: u32 = 0x1b2433; // raised card
    pub const DIM: u32 = 0x2b3547; // faint text
    pub const MID: u32 = 0x66748c; // secondary text
    pub const SLATE: u32 = 0x8a93a6; // the program's own voice
    pub const BRIGHT: u32 = 0xd7e0ee; // primary text
    pub const WHITE: u32 = 0xf2f6fc;

    // ---- the heart: the signature of "you"
    pub const ROSE: u32 = 0xff6d8a; // the rose dot / the heart
    pub const ROSE_DEEP: u32 = 0xb23a55;
    pub const ROSE_PALE: u32 = 0xffc2d0;

    // ---- the world: every candidate carries its own colour
    pub const WOOD: u32 = 0xd9a066; // table
    pub const PURPLE: u32 = 0x9a6dd7; // eggplant
    pub const BLUEPT: u32 = 0x9fb2cf; // bluepoint cat
    pub const SEAR: u32 = 0xc9a86a; // steak
    pub const PINK: u32 = 0xf2a7c3; // flower
    pub const SKIN: u32 = 0xe8c39e; // human hands
    pub const TOMATO: u32 = 0xc94436; // tomato
    pub const DOG: u32 = 0xd9b98a; // puppy
    pub const GOLD: u32 = 0xf2c14e; // the perfect stamp
    pub const AMBER: u32 = 0xe8c39e; // memory / sepia
    pub const GREEN_OK: u32 = 0x6fc7a0; // log OK
    pub const GREEN_LEAF: u32 = 0x5f9e57;

    pub fn c(h: u32) -> Color {
        Color::hex(h)
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
pub fn bounce(t: f64) -> f64 {
    let t = clamp01(t);
    (1.0 - t) * (t * 6.0 * std::f64::consts::PI).sin().abs()
}
/// ping-pong 0..1..0
pub fn pingpong(t: f64) -> f64 {
    let t = t.rem_euclid(2.0);
    if t < 1.0 {
        t
    } else {
        2.0 - t
    }
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
    pub fn pick<'a, T>(&mut self, arr: &'a [T]) -> &'a T {
        &arr[self.i64(0, arr.len() as i64 - 1) as usize]
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

    pub fn put_cell(&mut self, x: i64, y: i64, ch: char, fg: Color, bg: Color, bold: bool) {
        if !self.in_bounds(x, y) || ch == '\0' {
            return;
        }
        let i = y as usize * self.w + x as usize;
        let c = &mut self.cells[i];
        c.ch = ch;
        c.fg = fg;
        c.bg = bg;
        c.bold = bold;
    }

    pub fn fill(&mut self, x0: i64, y0: i64, w: i64, h: i64, ch: char, fg: Color) {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                self.put(x, y, ch, fg);
            }
        }
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

    pub fn shade_bg(&mut self, x0: i64, y0: i64, w: i64, h: i64, f: f64) {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                if !self.in_bounds(x, y) {
                    continue;
                }
                let i = y as usize * self.w + x as usize;
                let c = &mut self.cells[i];
                c.bg = c.bg.scale(f);
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

    /// ANSI serialization for the interactive live player.
    pub fn serialize(&self, out: &mut String) {
        self.serialize_view(self.w, self.h, out);
    }

    /// Serialize only the top-left `cols x rows` region of the grid.
    ///
    /// Every row is written with absolute cursor moves, so emitting more columns
    /// than the window has makes the terminal wrap each row into the one below,
    /// and emitting more rows than fit makes it scroll. In the alternate screen
    /// that reads as a picture that jumps and flickers every repaint. Clamping
    /// to the window keeps the write inside the screen it is aimed at.
    pub fn serialize_view(&self, cols: usize, rows: usize, out: &mut String) {
        use std::fmt::Write as _;
        let vw = cols.min(self.w);
        let vh = rows.min(self.h);
        out.clear();
        out.push_str("\u{1b}[?2026h");
        let mut cur_fg = Color { r: !0, g: !0, b: !0 };
        let mut cur_bg = cur_fg;
        let mut cur_bold = false;
        let mut first = true;
        for y in 0..vh {
            let _ = write!(out, "\u{1b}[{};1H", y + 1);
            for x in 0..vw {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Terminal columns a serialized chunk actually paints, SGR skipped.
    fn visible_len(s: &str) -> usize {
        let mut n = 0;
        let mut it = s.chars().peekable();
        while let Some(c) = it.next() {
            if c == '\u{1b}' {
                if it.peek() == Some(&'[') {
                    it.next();
                    while let Some(&f) = it.peek() {
                        it.next();
                        if ('@'..='~').contains(&f) {
                            break;
                        }
                    }
                }
                continue;
            }
            n += 1;
        }
        n
    }

    /// Rows are written with absolute cursor moves, so a frame wider or taller
    /// than the window makes the terminal wrap and scroll it — the flickering
    /// "terminal too small" card. The card must stay inside the window it was
    /// drawn for, at any size a user can drag the window to.
    #[test]
    fn too_small_card_stays_inside_the_window() {
        let sizes = [
            (120usize, 30usize),
            (192, 44),
            (100, 30),
            (80, 24),
            (60, 20),
            (40, 12),
            (24, 6),
            (12, 3),
            (4, 2),
            (1, 1),
        ];
        for (cols, rows) in sizes {
            let mut g = Grid::new(crate::pix::COLS, crate::pix::ROWS, crate::pal::c(crate::pal::BG0));
            crate::hud::too_small(&mut g, cols, rows);
            let mut out = String::new();
            g.serialize_view(cols, rows, &mut out);

            let mut lowest = 0usize;
            let mut longest = 0usize;
            for chunk in out.split('\u{1b}') {
                let rest = match chunk.strip_prefix('[') {
                    Some(r) => r,
                    None => continue,
                };
                // Row markers are "[<n>;1H"; colour and sync sequences are not.
                let (num, tail) = match rest.split_once(";1H") {
                    Some(p) => p,
                    None => continue,
                };
                if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit()) {
                    continue;
                }
                lowest = lowest.max(num.parse::<usize>().unwrap());
                let text = tail.split('\u{1b}').next().unwrap_or("");
                longest = longest.max(visible_len(text));
            }
            assert!(lowest <= rows, "{cols}x{rows}: wrote as far down as row {lowest}");
            assert!(longest <= cols, "{cols}x{rows}: wrote {longest} columns wide");
        }
    }

    /// The full-grid frame the player sends while the window fits is unchanged.
    #[test]
    fn full_frame_matches_the_window_sized_path() {
        let mut g = Grid::new(8, 3, crate::pal::c(crate::pal::BG0));
        g.text(0, 0, "hello", crate::pal::c(crate::pal::BRIGHT));
        let (mut a, mut b) = (String::new(), String::new());
        g.serialize(&mut a);
        g.serialize_view(8, 3, &mut b);
        assert_eq!(a, b);
    }
}

/// Terminal cell width of a char (CJK + fullwidth + Hangul + emoji-ish).
pub fn char_w(c: char) -> usize {
    let cp = c as u32;
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
        || (0x2460..=0x24FF).contains(&cp)
        || (0x25A0..=0x25FF).contains(&cp)
        || (0x2600..=0x26FF).contains(&cp)
        || (0x2700..=0x27BF).contains(&cp)
        || (0x2190..=0x22FF).contains(&cp)
        || (0x1F300..=0x1F64F).contains(&cp)
        || (0x1F900..=0x1F9FF).contains(&cp)
    {
        2
    } else {
        1
    }
}
