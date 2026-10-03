//! Pixel-space rendering: 1920x1080 RGB canvas driven by the character grid.
//!
//! Pipeline per frame:
//!   1. `begin_frame`  — fill with base background, reset sprite list + fx params
//!   2. scenes draw into the cell `Grid` (and may queue sprites)
//!   3. `paint_cells`  — cell backgrounds as solid rects, glyphs alpha-blended on top
//!   4. `flush_sprites`— free-floating glyphs at pixel positions (tears, petals…)
//!   5. `post`         — bloom -> glitch -> chromatic aberration -> scanlines ->
//!                       vignette -> grain -> fade/flash
//!   6. write raw RGB24 (video) or PNG (preview captures)
//!
//! Everything is deterministic: no wall-clock, no thread-local RNG.

use crate::gfx::{Color, Grid};
use std::collections::HashMap;

pub const CELL_W: usize = 10;
pub const CELL_H: usize = 24;
pub const COLS: usize = 192;
pub const ROWS: usize = 45;
pub const W: usize = COLS * CELL_W; // 1920
pub const H: usize = ROWS * CELL_H; // 1080
pub const BASE_PX: f32 = 18.0; // latin glyph size inside a cell

// ---------------------------------------------------------------- fonts ---

pub struct Fonts {
    pub latin: fontdue::Font,
    pub latin_b: fontdue::Font,
    pub cjk: Option<fontdue::Font>,
}

fn win_font(name: &str) -> Option<Vec<u8>> {
    let p = std::path::Path::new(r"C:\Windows\Fonts").join(name);
    std::fs::read(p).ok()
}

pub fn load_fonts() -> Fonts {
    let mk = |data: Vec<u8>, coll: u32| {
        fontdue::Font::from_bytes(
            data,
            fontdue::FontSettings {
                scale: BASE_PX,
                collection_index: coll,
                load_substitutions: true,
            },
        )
        .expect("font parse failed")
    };
    let latin = mk(
        win_font("consola.ttf").expect("consola.ttf not found (needed for latin glyphs)"),
        0,
    );
    let latin_b = match win_font("consolab.ttf") {
        Some(d) => mk(d, 0),
        None => mk(win_font("consola.ttf").unwrap(), 0),
    };
    let cjk = win_font("msyh.ttc")
        .map(|d| mk(d, 0))
        .or_else(|| win_font("SIMHEI.TTF").map(|d| mk(d, 0)));
    Fonts { latin, latin_b, cjk }
}

impl Fonts {
    #[inline]
    pub fn pick(&self, ch: char, bold: bool) -> &fontdue::Font {
        if crate::gfx::char_w(ch) >= 2 {
            if let Some(f) = &self.cjk {
                return f;
            }
        }
        if bold {
            &self.latin_b
        } else {
            &self.latin
        }
    }
}

// ---------------------------------------------------------------- raster cache ---

pub struct Raster {
    pub w: usize,
    pub h: usize,
    /// coverage 0..255, row-major
    pub data: Vec<u8>,
    /// x offset of bitmap left edge from the pen origin
    pub xmin: i32,
    /// y offset: bitmap top edge relative to baseline (px, positive up)
    pub ymin: i32,
    /// horizontal advance in px
    pub advance: f32,
}

fn key(cp: u32, size: u16, bold: bool, cjk: bool) -> u64 {
    (cp as u64) | ((size as u64) << 24) | ((bold as u64) << 40) | ((cjk as u64) << 41)
}

/// Get (or rasterize) a glyph. Free fn so callers can borrow cache & buf separately.
fn raster_cached<'a>(
    cache: &'a mut HashMap<u64, Raster>,
    fonts: &Fonts,
    ch: char,
    size: u16,
    bold: bool,
) -> &'a Raster {
    let cjk = crate::gfx::char_w(ch) >= 2;
    let k = key(ch as u32, size, bold, cjk);
    if !cache.contains_key(&k) {
        if cache.len() > 40000 {
            cache.clear();
        }
        let font = fonts.pick(ch, bold);
        let (m, data) = font.rasterize(ch, size as f32);
        let r = Raster {
            w: m.width,
            h: m.height,
            data,
            xmin: m.xmin,
            ymin: m.ymin,
            advance: m.advance_width,
        };
        cache.insert(k, r);
    }
    cache.get(&k).unwrap()
}

/// Blit one glyph coverage bitmap into an RGB buffer.
fn blit_raster(buf: &mut [u8], r: &Raster, px: i32, py: i32, color: Color, alpha: f64) {
    let a0 = alpha.clamp(0.0, 1.0);
    if a0 <= 0.004 || r.w == 0 || r.h == 0 {
        return;
    }
    for ry in 0..r.h {
        let y = py + ry as i32;
        if y < 0 || y >= H as i32 {
            continue;
        }
        let row = &r.data[ry * r.w..(ry + 1) * r.w];
        let dy = (y as usize * W) * 3;
        for rx in 0..r.w {
            let x = px + rx as i32;
            if x < 0 || x >= W as i32 {
                continue;
            }
            let cov = row[rx] as f64 / 255.0 * a0;
            if cov <= 0.004 {
                continue;
            }
            let i = dy + x as usize * 3;
            let inv = 1.0 - cov;
            buf[i] = (color.r as f64 * cov + buf[i] as f64 * inv) as u8;
            buf[i + 1] = (color.g as f64 * cov + buf[i + 1] as f64 * inv) as u8;
            buf[i + 2] = (color.b as f64 * cov + buf[i + 2] as f64 * inv) as u8;
        }
    }
}

/// Baseline y for a glyph vertically centered in a cell row.
pub fn baseline_of(fonts: &Fonts, size: f32, cell_top: i32) -> f32 {
    let (asc, desc) = match fonts.latin.horizontal_line_metrics(size) {
        Some(m) => (m.ascent, -m.descent),
        None => (size * 0.8, size * 0.2),
    };
    cell_top as f32 + (CELL_H as f32 + asc - desc) / 2.0
}

// ---------------------------------------------------------------- fx params ---

#[derive(Clone, Copy)]
pub struct Fx {
    /// additive bloom amount 0..1
    pub bloom: f64,
    /// horizontal slice glitch 0..1
    pub glitch: f64,
    /// chromatic aberration 0..1 (up to ~3px split)
    pub aberration: f64,
    /// film grain 0..1
    pub grain: f64,
    /// fade to black 0..1
    pub fade: f64,
    /// white flash 0..1
    pub flash: f64,
    /// scanline strength 0..1
    pub scanline: f64,
    /// rng seed for the frame's noise (deterministic per frame index)
    pub seed: u64,
}

impl Default for Fx {
    fn default() -> Self {
        Fx {
            bloom: 0.35,
            glitch: 0.0,
            aberration: 0.0,
            grain: 0.06,
            fade: 0.0,
            flash: 0.0,
            scanline: 0.5,
            seed: 1,
        }
    }
}

struct Blit {
    cp: u32,
    x: f32,
    y: f32, // baseline position
    size: u16,
    bold: bool,
    color: Color,
    alpha: f64,
}

// ---------------------------------------------------------------- canvas ---

pub struct Canvas {
    pub buf: Vec<u8>, // RGB, W*H*3
    pub fonts: Fonts,
    cache: HashMap<u64, Raster>,
    sprites: Vec<Blit>,
    pub fx: Fx,
    vign: Vec<u8>,    // per-pixel multiplier (255 = untouched)
    noise: Vec<i8>,   // precomputed grain noise table
    small_a: Vec<u8>, // bloom scratch (W/4 x H/4)
    small_b: Vec<u8>,
    /// Pixel layer switched off. Every painting call returns immediately;
    /// font metrics stay live so layout math is unchanged. See `terminal_sink`.
    pub dim: bool,
}

impl Canvas {
    pub fn new() -> Canvas {
        // radial vignette table
        let mut vign = vec![255u8; W * H];
        let cx = W as f64 / 2.0;
        let cy = H as f64 / 2.0;
        let maxd = (cx * cx + cy * cy).sqrt();
        for y in 0..H {
            for x in 0..W {
                let dx = (x as f64 - cx) / cx;
                let dy = (y as f64 - cy) / cy;
                let d = (dx * dx + dy * dy).sqrt() / maxd;
                let f = 1.0 - 0.34 * d.powf(1.9);
                vign[y * W + x] = (f.clamp(0.58, 1.0) * 255.0) as u8;
            }
        }
        Canvas {
            buf: vec![0u8; W * H * 3],
            fonts: load_fonts(),
            cache: HashMap::new(),
            sprites: Vec::with_capacity(512),
            fx: Fx::default(),
            vign,
            noise: {
                let mut rng = crate::gfx::Rng::new(0x9E3779B9);
                (0..1 << 14).map(|_| rng.range(-128.0, 127.0) as i8).collect()
            },
            small_a: vec![0u8; (W / 4) * (H / 4) * 3],
            small_b: vec![0u8; (W / 4) * (H / 4) * 3],
            dim: false,
        }
    }

    /// A canvas for the terminal player: the pixel layer is switched off.
    ///
    /// The player only ever paints the character grid — a terminal cannot show
    /// a 1920x1080 buffer, and the environments, sprites and bloom that make
    /// the video are never sampled there. Running the whole software renderer
    /// each frame and discarding it costs most of the frame budget and grows
    /// the sprite queue without bound. This builds a canvas with no backing
    /// store and no scratch buffers; painting calls become no-ops.
    pub fn terminal_sink() -> Canvas {
        Canvas {
            buf: Vec::new(),
            fonts: load_fonts(),
            cache: HashMap::new(),
            sprites: Vec::new(),
            fx: Fx::default(),
            vign: Vec::new(),
            noise: Vec::new(),
            small_a: Vec::new(),
            small_b: Vec::new(),
            dim: true,
        }
    }

    // ------------------------------------------------------------- frame ---

    #[inline]
    pub fn begin_frame(&mut self, bg: Color) {
        let px = [bg.r, bg.g, bg.b];
        for p in self.buf.chunks_exact_mut(3) {
            p[0] = px[0];
            p[1] = px[1];
            p[2] = px[2];
        }
        self.sprites.clear();
        self.fx = Fx::default();
    }

    pub fn baseline_for(&self, size: f32, cell_top: i32) -> f32 {
        baseline_of(&self.fonts, size, cell_top)
    }

    // -------------------------------------------------------------- cells ---

    /// Paint the whole cell grid: backgrounds as rects, then glyphs.
    pub fn paint_cells(&mut self, g: &Grid) {
        if self.dim {
            return;
        }
        let base = g.bg;
        // pass 1: backgrounds that differ from base (run-length compressed)
        for cy in 0..g.h {
            let y0 = (cy * CELL_H) as i32;
            let mut cx = 0;
            while cx < g.w {
                let bg = g.cells[cy * g.w + cx].bg;
                if bg == base {
                    cx += 1;
                    continue;
                }
                let mut run = 1;
                while cx + run < g.w && g.cells[cy * g.w + cx + run].bg == bg {
                    run += 1;
                }
                self.fill_rect(cx as i32 * CELL_W as i32, y0, run * CELL_W, CELL_H, bg);
                cx += run;
            }
        }
        // pass 2: glyphs
        for cy in 0..g.h {
            let top = (cy * CELL_H) as i32;
            let baseline = baseline_of(&self.fonts, BASE_PX, top) as i32;
            for cx in 0..g.w {
                let c = g.cells[cy * g.w + cx];
                if c.ch == ' ' || c.ch == '\0' {
                    continue;
                }
                let cw = crate::gfx::char_w(c.ch);
                let r = raster_cached(&mut self.cache, &self.fonts, c.ch, BASE_PX as u16, c.bold);
                let span = (cw.max(1) * CELL_W) as i32;
                let px = cx as i32 * CELL_W as i32 + (span - r.w as i32) / 2;
                let py = baseline - (r.ymin + r.h as i32);
                blit_raster(&mut self.buf, r, px, py, c.fg, if c.bold { 1.0 } else { 0.92 });
            }
        }
    }

    pub fn fill_rect(&mut self, x0: i32, y0: i32, w: usize, h: usize, color: Color) {
        if self.dim {
            return;
        }
        let px = [color.r, color.g, color.b];
        if y0 + h as i32 <= 0 || y0 >= H as i32 {
            return;
        }
        for y in (y0.max(0) as usize)..((y0 as usize + h).min(H)) {
            let xlo = x0.max(0) as usize;
            let xhi = ((x0 as usize) + w).min(W);
            if xlo >= xhi {
                continue;
            }
            let row = y * W * 3;
            for x in xlo..xhi {
                let i = row + x * 3;
                self.buf[i] = px[0];
                self.buf[i + 1] = px[1];
                self.buf[i + 2] = px[2];
            }
        }
    }

    // ------------------------------------------------------------ sprites ---

    /// Queue a glyph centered on (x, y): compensates the baseline offset so
    /// that circles of different sizes share one visual center.
    pub fn sprite_c(&mut self, ch: char, x: f32, y: f32, size: f32, color: Color, alpha: f64) {
        if self.dim {
            return;
        }
        self.sprite(ch, x, y + size * 0.36, size, color, alpha);
    }

    /// Queue a glyph at a pixel position (baseline anchored). Drawn over cells.
    pub fn sprite(&mut self, ch: char, x: f32, y: f32, size: f32, color: Color, alpha: f64) {
        if self.dim {
            return;
        }
        if alpha <= 0.01 {
            return;
        }
        self.sprites.push(Blit {
            cp: ch as u32,
            x,
            y,
            size: size.round().clamp(4.0, 220.0) as u16,
            bold: false,
            color,
            alpha,
        });
    }

    /// Advance width of one glyph at a size, in pixels.
    pub fn advance(&mut self, ch: char, size: f32, bold: bool) -> f32 {
        let s = size.round().clamp(4.0, 220.0) as u16;
        let r = raster_cached(&mut self.cache, &self.fonts, ch, s, bold);
        r.advance
    }

    /// Total width of a string at a size, in pixels.
    pub fn text_width(&mut self, s: &str, size: f32, bold: bool) -> f32 {
        let sz = size.round().clamp(4.0, 220.0) as u16;
        let mut w = 0.0f32;
        for ch in s.chars() {
            let r = raster_cached(&mut self.cache, &self.fonts, ch, sz, bold);
            w += r.advance;
        }
        w
    }

    /// Queue a whole string; `center=true` centers horizontally on x. Returns end x.
    #[allow(clippy::too_many_arguments)]
    pub fn text_px(
        &mut self,
        x: f32,
        y: f32, // baseline
        s: &str,
        size: f32,
        bold: bool,
        color: Color,
        alpha: f64,
        center: bool,
    ) -> f32 {
        if self.dim {
            return 0.0;
        }
        let size_r = size.round().clamp(4.0, 220.0) as u16;
        if alpha > 0.01 {
            let mut w = 0.0f32;
            for ch in s.chars() {
                let r = raster_cached(&mut self.cache, &self.fonts, ch, size_r, bold);
                w += r.advance;
            }
            let mut pen = if center { x - w / 2.0 } else { x };
            for ch in s.chars() {
                self.sprites.push(Blit {
                    cp: ch as u32,
                    x: pen,
                    y,
                    size: size_r,
                    bold,
                    color,
                    alpha,
                });
                let r = raster_cached(&mut self.cache, &self.fonts, ch, size_r, bold);
                pen += r.advance;
            }
        }
        0.0
    }

    pub fn flush_sprites(&mut self) {
        if self.dim {
            return;
        }
        let sprites = std::mem::take(&mut self.sprites);
        for b in sprites {
            let ch = char::from_u32(b.cp).unwrap_or(' ');
            let r = raster_cached(&mut self.cache, &self.fonts, ch, b.size, b.bold);
            let px = b.x.round() as i32 + r.xmin;
            let py = b.y.round() as i32 - (r.ymin + r.h as i32);
            blit_raster(&mut self.buf, r, px, py, b.color, b.alpha);
        }
    }

    // ------------------------------------------------------------ pixel paint ---

    /// Single pixel, alpha-blended.
    #[inline]
    pub fn px(&mut self, x: i32, y: i32, color: Color, alpha: f64) {
        if self.dim {
            return;
        }
        let a = alpha.clamp(0.0, 1.0);
        if a <= 0.004 || x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        let i = (y as usize * W + x as usize) * 3;
        let inv = 1.0 - a;
        self.buf[i] = (color.r as f64 * a + self.buf[i] as f64 * inv) as u8;
        self.buf[i + 1] = (color.g as f64 * a + self.buf[i + 1] as f64 * inv) as u8;
        self.buf[i + 2] = (color.b as f64 * a + self.buf[i + 2] as f64 * inv) as u8;
    }

    /// Additive pixel (for glows, stars, embers).
    #[inline]
    pub fn px_add(&mut self, x: i32, y: i32, color: Color, amount: f64) {
        if self.dim {
            return;
        }
        if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        let k = amount.clamp(0.0, 4.0);
        let i = (y as usize * W + x as usize) * 3;
        self.buf[i] = (self.buf[i] as f64 + color.r as f64 * k).min(255.0) as u8;
        self.buf[i + 1] = (self.buf[i + 1] as f64 + color.g as f64 * k).min(255.0) as u8;
        self.buf[i + 2] = (self.buf[i + 2] as f64 + color.b as f64 * k).min(255.0) as u8;
    }

    /// Alpha-blended rectangle.
    pub fn rect_a(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color, alpha: f64) {
        if self.dim {
            return;
        }
        let a = alpha.clamp(0.0, 1.0);
        if a <= 0.004 {
            return;
        }
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(W as i32);
        let y1 = (y + h).min(H as i32);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let inv = 1.0 - a;
        for yy in y0..y1 {
            let row = yy as usize * W * 3;
            for xx in x0..x1 {
                let i = row + xx as usize * 3;
                self.buf[i] = (color.r as f64 * a + self.buf[i] as f64 * inv) as u8;
                self.buf[i + 1] = (color.g as f64 * a + self.buf[i + 1] as f64 * inv) as u8;
                self.buf[i + 2] = (color.b as f64 * a + self.buf[i + 2] as f64 * inv) as u8;
            }
        }
    }

    /// Vertical gradient rectangle (c0 at the top, c1 at the bottom).
    pub fn vgrad(&mut self, x: i32, y: i32, w: i32, h: i32, c0: Color, c1: Color, alpha: f64) {
        if self.dim {
            return;
        }
        if h <= 0 || w <= 0 {
            return;
        }
        for k in 0..h {
            let u = k as f64 / (h - 1).max(1) as f64;
            let c = Color::lerp(c0, c1, u);
            self.rect_a(x, y + k, w, 1, c, alpha);
        }
    }

    /// Horizontal gradient rectangle.
    pub fn hgrad(&mut self, x: i32, y: i32, w: i32, h: i32, c0: Color, c1: Color, alpha: f64) {
        if self.dim {
            return;
        }
        if h <= 0 || w <= 0 {
            return;
        }
        for k in 0..w {
            let u = k as f64 / (w - 1).max(1) as f64;
            let c = Color::lerp(c0, c1, u);
            self.rect_a(x + k, y, 1, h, c, alpha);
        }
    }

    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Color, alpha: f64, width: f32) {
        if self.dim {
            return;
        }
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();
        let n = (len.ceil() as i32).max(1);
        let r = (width * 0.5).max(0.5);
        for i in 0..=n {
            let u = i as f32 / n as f32;
            let x = x0 + dx * u;
            let y = y0 + dy * u;
            if r <= 0.75 {
                self.px(x.round() as i32, y.round() as i32, color, alpha);
            } else {
                self.disc(x, y, r, color, alpha);
            }
        }
    }

    /// Filled disc, hard edge with 1px antialias.
    pub fn disc(&mut self, cx: f32, cy: f32, r: f32, color: Color, alpha: f64) {
        if self.dim {
            return;
        }
        if r <= 0.0 {
            return;
        }
        let x0 = (cx - r - 1.0).floor().max(0.0) as i32;
        let x1 = (cx + r + 1.0).ceil().min(W as f32 - 1.0) as i32;
        let y0 = (cy - r - 1.0).floor().max(0.0) as i32;
        let y1 = (cy + r + 1.0).ceil().min(H as f32 - 1.0) as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let d = (dx * dx + dy * dy).sqrt();
                let cov = (r + 0.5 - d).clamp(0.0, 1.0);
                if cov > 0.004 {
                    self.px(x, y, color, alpha * cov as f64);
                }
            }
        }
    }

    /// Ellipse (rx, ry), for pools of light and floor shadows.
    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, color: Color, alpha: f64) {
        if self.dim {
            return;
        }
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }
        let x0 = (cx - rx - 1.0).floor().max(0.0) as i32;
        let x1 = (cx + rx + 1.0).ceil().min(W as f32 - 1.0) as i32;
        let y0 = (cy - ry - 1.0).floor().max(0.0) as i32;
        let y1 = (cy + ry + 1.0).ceil().min(H as f32 - 1.0) as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = (x as f32 + 0.5 - cx) / rx;
                let dy = (y as f32 + 0.5 - cy) / ry;
                let d = (dx * dx + dy * dy).sqrt();
                let cov = ((1.0 - d) * rx.min(ry)).clamp(0.0, 1.0);
                if cov > 0.004 {
                    self.px(x, y, color, alpha * cov as f64);
                }
            }
        }
    }

    /// Ring / annulus of the given thickness.
    pub fn ring(&mut self, cx: f32, cy: f32, r: f32, thickness: f32, color: Color, alpha: f64) {
        if self.dim {
            return;
        }
        let rin = (r - thickness * 0.5).max(0.0);
        let rout = r + thickness * 0.5;
        let x0 = (cx - rout - 1.0).floor().max(0.0) as i32;
        let x1 = (cx + rout + 1.0).ceil().min(W as f32 - 1.0) as i32;
        let y0 = (cy - rout - 1.0).floor().max(0.0) as i32;
        let y1 = (cy + rout + 1.0).ceil().min(H as f32 - 1.0) as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let d = (dx * dx + dy * dy).sqrt();
                let cov = (d - rin + 0.5).clamp(0.0, 1.0) * (rout + 0.5 - d).clamp(0.0, 1.0);
                if cov > 0.004 {
                    self.px(x, y, color, alpha * cov as f64);
                }
            }
        }
    }

    /// Soft radial glow (additive) — the workhorse for light sources.
    pub fn glow_at(&mut self, cx: f32, cy: f32, r: f32, color: Color, amount: f64, falloff: f64) {
        if self.dim {
            return;
        }
        if r <= 0.0 || amount <= 0.002 {
            return;
        }
        let x0 = (cx - r).floor().max(0.0) as i32;
        let x1 = (cx + r).ceil().min(W as f32 - 1.0) as i32;
        let y0 = (cy - r).floor().max(0.0) as i32;
        let y1 = (cy + r).ceil().min(H as f32 - 1.0) as i32;
        let fo = falloff.max(0.2);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let d = (dx * dx + dy * dy).sqrt() / r;
                if d >= 1.0 {
                    continue;
                }
                let f = (1.0 - d).powf(fo as f32);
                self.px_add(x, y, color, amount * f as f64);
            }
        }
    }

    /// Draw a pixel-art sprite: `rows` is a char map, `pal` maps chars to colours.
    /// `scale` = pixels per authored pixel. Anchored top-left at (x, y).
    pub fn bitmap(
        &mut self,
        x: i32,
        y: i32,
        rows: &[&str],
        pal: &[(char, u32)],
        scale: i32,
        alpha: f64,
        tint: Option<Color>,
    ) {
        if self.dim {
            return;
        }
        let s = scale.max(1);
        for (ry, row) in rows.iter().enumerate() {
            for (rx, ch) in row.chars().enumerate() {
                if ch == ' ' || ch == '.' {
                    continue;
                }
                let color = match tint {
                    Some(c) => c,
                    None => match pal.iter().find(|(k, _)| *k == ch) {
                        Some((_, hx)) => Color::hex(*hx),
                        None => continue,
                    },
                };
                self.rect_a(x + rx as i32 * s, y + ry as i32 * s, s, s, color, alpha);
            }
        }
    }

    /// Sprite size in pixels for a given scale.
    pub fn bitmap_size(rows: &[&str], scale: i32) -> (i32, i32) {
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i32;
        (w * scale.max(1), rows.len() as i32 * scale.max(1))
    }

    /// Deterministic film grain / starfield noise rectangle (bright specks).
    pub fn speckle(&mut self, x: i32, y: i32, w: i32, h: i32, n: usize, seed: u64, color: Color, alpha: f64) {
        if self.dim {
            return;
        }
        let mut rng = crate::gfx::Rng::new(seed | 1);
        for _ in 0..n {
            let px = x + rng.i64(0, (w - 1).max(0) as i64) as i32;
            let py = y + rng.i64(0, (h - 1).max(0) as i64) as i32;
            let a = alpha * (0.35 + 0.65 * rng.f64());
            self.px(px, py, color, a);
        }
    }

    // --------------------------------------------------------------- post ---

    pub fn post(&mut self, fx: &Fx) {
        if self.dim {
            return;
        }
        if fx.bloom > 0.01 {
            self.bloom(fx.bloom);
        }
        if fx.glitch > 0.01 {
            self.glitch(fx.glitch, fx.seed);
        }
        if fx.aberration > 0.01 {
            self.aberrate(fx.aberration);
        }
        self.scanlines(fx.scanline);
        self.final_pass(fx);
    }

    fn bloom(&mut self, amount: f64) {
        let sw = W / 4;
        let sh = H / 4;
        // 1) explicit 4x4 box downsample with threshold
        for y in 0..sh {
            for x in 0..sw {
                let (mut r, mut g, mut b) = (0u32, 0u32, 0u32);
                for dy in 0..4u32 {
                    let row = (y * 4 + dy as usize) * W * 3;
                    for dx in 0..4u32 {
                        let i = row + (x * 4 + dx as usize) * 3;
                        r += self.buf[i] as u32;
                        g += self.buf[i + 1] as u32;
                        b += self.buf[i + 2] as u32;
                    }
                }
                let (r, g, b) = ((r / 16) as f64, (g / 16) as f64, (b / 16) as f64);
                let lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
                let k = ((lum - 0.34) / 0.66).clamp(0.0, 1.0);
                let k = k * k * (3.0 - 2.0 * k);
                let i = (y * sw + x) * 3;
                self.small_a[i] = (r * k) as u8;
                self.small_a[i + 1] = (g * k) as u8;
                self.small_a[i + 2] = (b * k) as u8;
            }
        }
        // 2) two rounds of 3x3 box blur (a->b horizontal, b->a vertical)
        for _round in 0..2 {
            {
                let (src, dst) = (&self.small_a, &mut self.small_b);
                for y in 0..sh {
                    let row = y * sw * 3;
                    for x in 0..sw {
                        let (mut a0, mut a1, mut a2) = (0u32, 0u32, 0u32);
                        let mut n = 0u32;
                        for dx in -1i64..=1 {
                            let xx = x as i64 + dx;
                            if xx < 0 || xx >= sw as i64 {
                                continue;
                            }
                            let i = row + (xx as usize) * 3;
                            a0 += src[i] as u32;
                            a1 += src[i + 1] as u32;
                            a2 += src[i + 2] as u32;
                            n += 1;
                        }
                        let i = row + x * 3;
                        dst[i] = (a0 / n) as u8;
                        dst[i + 1] = (a1 / n) as u8;
                        dst[i + 2] = (a2 / n) as u8;
                    }
                }
            }
            {
                let (src, dst) = (&self.small_b, &mut self.small_a);
                for y in 0..sh {
                    for x in 0..sw {
                        let (mut a0, mut a1, mut a2) = (0u32, 0u32, 0u32);
                        let mut n = 0u32;
                        for dy in -1i64..=1 {
                            let yy = y as i64 + dy;
                            if yy < 0 || yy >= sh as i64 {
                                continue;
                            }
                            let i = ((yy as usize) * sw + x) * 3;
                            a0 += src[i] as u32;
                            a1 += src[i + 1] as u32;
                            a2 += src[i + 2] as u32;
                            n += 1;
                        }
                        let i = (y * sw + x) * 3;
                        dst[i] = (a0 / n) as u8;
                        dst[i + 1] = (a1 / n) as u8;
                        dst[i + 2] = (a2 / n) as u8;
                    }
                }
            }
        }
        // 3) upsample-add (nearest, integer 8.8 factor)
        let amt = ((amount.clamp(0.0, 1.0) * 0.85) * 256.0) as u32;
        for y in 0..H {
            let sy = (y / 4).min(sh - 1);
            let row = sy * sw;
            for x in 0..W {
                let sx = (x / 4).min(sw - 1);
                let i = (row + sx) * 3;
                let d = (y * W + x) * 3;
                for c in 0..3 {
                    let v = self.buf[d + c] as u32 + ((self.small_a[i + c] as u32 * amt) >> 8);
                    self.buf[d + c] = v.min(255) as u8;
                }
            }
        }
    }

    fn glitch(&mut self, intensity: f64, seed: u64) {
        let mut rng = crate::gfx::Rng::new(seed ^ 0x636f6465);
        let bands = (6.0 + intensity * 22.0) as usize;
        for _ in 0..bands {
            let y = rng.i64(0, H as i64 - 4) as usize;
            let bh = rng.i64(2, 3 + (intensity * 10.0) as i64) as usize;
            let bh = bh.min(H - y);
            let dx = (rng.range(-14.0, 14.0) * intensity).round() as i32;
            if dx == 0 {
                continue;
            }
            for yy in y..y + bh {
                let row = yy * W * 3;
                let src: Vec<u8> = self.buf[row..row + W * 3].to_vec();
                for x in 0..W {
                    let sx = x as i64 - dx as i64;
                    let si = if sx >= 0 && sx < W as i64 {
                        sx as usize * 3
                    } else {
                        x * 3
                    };
                    let di = row + x * 3;
                    self.buf[di] = src[si];
                    self.buf[di + 1] = src[si + 1];
                    self.buf[di + 2] = src[si + 2];
                }
                if rng.f64() < 0.5 {
                    for x in 0..W {
                        let i = row + x * 3;
                        self.buf[i] = self.buf[i].saturating_add(14);
                        self.buf[i + 2] = self.buf[i + 2].saturating_sub(10);
                    }
                }
            }
        }
    }

    fn aberrate(&mut self, amount: f64) {
        let dx = (amount * 3.2).round().max(1.0) as i64;
        let src = self.buf.clone();
        for y in 0..H {
            let row = y * W * 3;
            for x in 0..W {
                let i = row + x * 3;
                let xr = (x as i64 + dx).clamp(0, W as i64 - 1) as usize;
                let xb = (x as i64 - dx).clamp(0, W as i64 - 1) as usize;
                self.buf[i] = src[row + xr * 3];
                self.buf[i + 2] = src[row + xb * 3 + 2];
            }
        }
    }

    fn scanlines(&mut self, strength: f64) {
        if strength <= 0.01 {
            return;
        }
        let s = strength.clamp(0.0, 1.0);
        for y in (2usize..H).step_by(3) {
            let f = 1.0 - 0.13 * s;
            let row = y * W * 3;
            for i in (row..row + W * 3).step_by(3) {
                self.buf[i] = (self.buf[i] as f64 * f) as u8;
                self.buf[i + 1] = (self.buf[i + 1] as f64 * f) as u8;
                self.buf[i + 2] = (self.buf[i + 2] as f64 * f) as u8;
            }
        }
    }

    /// Vignette + grain + fade + flash in one integer pass.
    fn final_pass(&mut self, fx: &Fx) {
        let grain = fx.grain.clamp(0.0, 1.0);
        let fade = fx.fade.clamp(0.0, 1.0);
        let flash = fx.flash.clamp(0.0, 1.0);
        // fixed-point factors (8.8)
        let fade_m = ((1.0 - fade) * 256.0) as u32;
        let flash_a = (flash * 235.0) as u32;
        let grain_amp = (0.10 * grain * 256.0) as u32; // per-pixel noise gain (8.8)
        let do_grain = grain > 0.001;
        let do_flash = flash > 0.001;
        let base = (fx.seed as usize) & 0x3FFF;
        let noise_len = self.noise.len();
        for y in 0..H {
            let row = y * W * 3;
            let vrow = y * W;
            for x in 0..W {
                let i = row + x * 3;
                let mut m = self.vign[vrow + x] as u32; // 0..255
                if do_grain {
                    let n = self.noise[(base + ((x >> 1) + (y >> 1) * (W >> 1))) % noise_len] as i32;
                    // m' = m * (1 + n*amp/128/256) -> combine into one 8.8 factor
                    let g = 256 + (n * grain_amp as i32 >> 8);
                    m = ((m as i32 * g) >> 8).clamp(0, 255) as u32;
                }
                // combined factor: vign*fade in 16.16-ish
                let mf = (m * fade_m) >> 8; // 0..255
                for c in 0..3 {
                    let mut v = ((self.buf[i + c] as u32 * mf) >> 8) + flash_a * (do_flash as u32);
                    if v > 255 {
                        v = 255;
                    }
                    self.buf[i + c] = v as u8;
                }
            }
        }
    }

    // ------------------------------------------------------------- output ---

    pub fn write_stdout(&self) {
        use std::io::Write;
        let mut out = std::io::stdout().lock();
        let _ = out.write_all(&self.buf);
        let _ = out.flush();
    }

    pub fn write_png(&self, path: &std::path::Path) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        let w = std::io::BufWriter::new(file);
        let mut enc = png::Encoder::new(w, W as u32, H as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header()?;
        writer.write_image_data(&self.buf)?;
        Ok(())
    }
}

