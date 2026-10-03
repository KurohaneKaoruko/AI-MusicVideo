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

/// Which physical face a glyph is rasterized from. Part of the raster cache key,
/// so a fallback never reuses another face's bitmap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    Latin = 0,
    LatinBold = 1,
    Cjk = 2,
}

impl Fonts {
    /// Pick the face that actually *has* this glyph.
    ///
    /// Width alone is not enough: several symbols the film draws (`○ ● ˙ ★ ♪ ♥ √`)
    /// are absent from Consolas, and several range-classified "wide" codepoints
    /// have no CJK outline either. Rasterizing a missing glyph yields an empty or
    /// notdef bitmap, which reads on screen as a misplaced blob, so probe the
    /// cmap and fall through to the other face.
    pub fn pick(&self, ch: char, bold: bool) -> Face {
        let want_cjk = crate::gfx::char_w(ch) >= 2;
        let primary = match (want_cjk, bold) {
            (true, _) => Face::Cjk,
            (false, true) => Face::LatinBold,
            (false, false) => Face::Latin,
        };
        let order = match primary {
            Face::Cjk => [Face::Cjk, Face::Latin, Face::LatinBold],
            Face::LatinBold | Face::Latin => [primary, Face::Cjk, Face::Latin],
        };
        for f in order {
            if self.has_glyph(f, ch) {
                return f;
            }
        }
        primary
    }

    #[inline]
    fn has_glyph(&self, face: Face, ch: char) -> bool {
        // glyph index 0 is .notdef
        let idx = match face {
            Face::Latin | Face::LatinBold => self.latin.lookup_glyph_index(ch),
            Face::Cjk => match &self.cjk {
                Some(f) => f.lookup_glyph_index(ch),
                None => return false,
            },
        };
        idx != 0
    }

    #[inline]
    pub fn font(&self, face: Face) -> &fontdue::Font {
        match face {
            Face::Latin => &self.latin,
            Face::LatinBold => &self.latin_b,
            Face::Cjk => self.cjk.as_ref().unwrap_or(&self.latin),
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

fn key(cp: u32, size: u16, face: Face) -> u64 {
    (cp as u64) | ((size as u64) << 24) | ((face as u64) << 40)
}

/// Get (or rasterize) a glyph. Free fn so callers can borrow cache & buf separately.
fn raster_cached<'a>(
    cache: &'a mut HashMap<u64, Raster>,
    fonts: &Fonts,
    ch: char,
    size: u16,
    bold: bool,
) -> &'a Raster {
    let face = fonts.pick(ch, bold);
    let k = key(ch as u32, size, face);
    if !cache.contains_key(&k) {
        if cache.len() > 40000 {
            cache.clear();
        }
        let (m, data) = fonts.font(face).rasterize(ch, size as f32);
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

/// Rasterize a glyph so its **advance** fits a `cw`-cell slot at `BASE_PX`.
///
/// Ink overhang is deliberate and must be preserved: line-drawing glyphs are
/// designed to spill ~0.5px past their advance so consecutive cells join without
/// a seam (`─` in Consolas: 10px of ink on a 9.9px advance). What must not happen
/// is a glyph *claiming* more advance than its slot. A handful of symbols the
/// film uses are missing from Consolas and land on a CJK face with a wider
/// advance (`▔` 10.8px, `★` 14.7px, in a 10px cell); unscaled they push their
/// neighbours sideways, which shows up as an uneven row. Scale those down.
///
/// The fitted size is a pure function of (char, cw, bold), so the shared raster
/// cache stays valid.
fn cell_raster_cached<'a>(
    cache: &'a mut HashMap<u64, Raster>,
    fonts: &Fonts,
    ch: char,
    cw: usize,
    bold: bool,
) -> &'a Raster {
    let span = (cw.max(1) * CELL_W) as f32;
    let advance = {
        let b = raster_cached(cache, fonts, ch, BASE_PX as u16, bold);
        b.advance
    };
    if advance <= span + 0.01 {
        return raster_cached(cache, fonts, ch, BASE_PX as u16, bold);
    }
    let fit = (BASE_PX * span / advance).floor().clamp(4.0, BASE_PX);
    raster_cached(cache, fonts, ch, fit as u16, bold)
}

/// Top-left pixel to blit a glyph at, given the pen position it was laid out for.
#[inline]
fn blit_origin(pen_x: f32, baseline_y: f32, r: &Raster) -> (i32, i32) {
    (
        pen_x.round() as i32 + r.xmin,
        baseline_y.round() as i32 - (r.ymin + r.h as i32),
    )
}

/// Left edge of the pen box for a glyph occupying `cw` cells starting at `cell`.
///
/// A terminal is a rigid grid: every narrow glyph shares one pen origin (the
/// cell's left edge) and one advance, so a column of `i`, `M` and `|` lines up.
/// This used to centre each glyph by its own *ink* width instead, which moved
/// every character by `(CELL_W - ink_w) / 2` — narrow glyphs drifted right and
/// wide ones left, so mixed text and box art visibly wobbled.
///
/// Full-width glyphs are the exception: their advance covers two cells and may
/// not fill it, so their advance box is centred inside the 2-cell span.
#[inline]
fn cell_pen_x(cell: i64, cw: usize, r: &Raster) -> f32 {
    let left = (cell * CELL_W as i64) as f32;
    if cw >= 2 {
        let span = (cw * CELL_W) as f32;
        left + ((span - r.advance) * 0.5).round() + r.xmin as f32
    } else {
        left + r.xmin as f32
    }
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
    /// pen x — or, when `center` is set, the x the glyph's *ink box* is centred on
    x: f32,
    /// baseline y — or, when `center` is set, the y the ink box is centred on
    y: f32,
    size: u16,
    bold: bool,
    color: Color,
    alpha: f64,
    center: bool,
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
            let baseline = baseline_of(&self.fonts, BASE_PX, top);
            for cx in 0..g.w {
                let c = g.cells[cy * g.w + cx];
                if c.ch == ' ' || c.ch == '\0' {
                    continue;
                }
                let cw = crate::gfx::char_w(c.ch);
                let r = cell_raster_cached(&mut self.cache, &self.fonts, c.ch, cw, c.bold);
                let (px, py) = blit_origin(cell_pen_x(cx as i64, cw, r), baseline, r);
                blit_raster(&mut self.buf, r, px, py, c.fg, if c.bold { 1.0 } else { 0.92 });
            }
        }
    }

    pub fn fill_rect(&mut self, x0: i32, y0: i32, w: usize, h: usize, color: Color) {
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

    /// Queue a glyph whose *ink box* is centred on (x, y).
    ///
    /// Sprites are sub-cell, so there is no pen grid to snap to; what we want is
    /// the drawn shape centred. Using the raster's real ink box (xmin/ymin/w/h)
    /// keeps a halo and its core concentric, which a fixed baseline fudge
    /// (`y + size * 0.36`) could not do across different glyphs and sizes.
    pub fn sprite_c(&mut self, ch: char, x: f32, y: f32, size: f32, color: Color, alpha: f64) {
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
            center: true,
        });
    }

    /// Queue a glyph at a pixel position (baseline anchored). Drawn over cells.
    pub fn sprite(&mut self, ch: char, x: f32, y: f32, size: f32, color: Color, alpha: f64) {
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
            center: false,
        });
    }

    /// Queue a whole string laid out on one baseline. `center=true` centres the
    /// string's **ink** box on x (not its advance box, which is off by the side
    /// bearings and makes headlines look shifted). Returns the end pen x.
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
        let size_r = size.round().clamp(4.0, 220.0) as u16;
        if alpha <= 0.01 {
            return x;
        }
        let chars: Vec<char> = s.chars().collect();
        let mut total = 0.0f32;
        for &ch in &chars {
            total += raster_cached(&mut self.cache, &self.fonts, ch, size_r, bold).advance;
        }
        let start = if center {
            let mut pen = 0.0f32;
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for &ch in &chars {
                let r = raster_cached(&mut self.cache, &self.fonts, ch, size_r, bold);
                if r.w > 0 {
                    lo = lo.min(pen + r.xmin as f32);
                    hi = hi.max(pen + r.xmin as f32 + r.w as f32);
                }
                pen += r.advance;
            }
            if lo <= hi {
                x - (lo + hi) * 0.5
            } else {
                x - total * 0.5
            }
        } else {
            x
        };
        let mut pen = start;
        for &ch in &chars {
            self.sprites.push(Blit {
                cp: ch as u32,
                x: pen,
                y,
                size: size_r,
                bold,
                color,
                alpha,
                center: false,
            });
            pen += raster_cached(&mut self.cache, &self.fonts, ch, size_r, bold).advance;
        }
        pen
    }

    pub fn flush_sprites(&mut self) {
        let sprites = std::mem::take(&mut self.sprites);
        for b in sprites {
            let ch = char::from_u32(b.cp).unwrap_or(' ');
            let r = raster_cached(&mut self.cache, &self.fonts, ch, b.size, b.bold);
            let (pen_x, baseline) = if b.center {
                let h = r.h as f32;
                (
                    b.x - r.xmin as f32 - r.w as f32 * 0.5,
                    b.y + r.ymin as f32 + h * 0.5,
                )
            } else {
                (b.x, b.y)
            };
            let (px, py) = blit_origin(pen_x, baseline, r);
            blit_raster(&mut self.buf, r, px, py, b.color, b.alpha);
        }
    }

    // --------------------------------------------------------------- post ---

    pub fn post(&mut self, fx: &Fx) {
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
}

// ------------------------------------------------------------- dev report ---

/// Dev self-test: dump the cell-typography metrics.
///
/// A skewed grid is otherwise only judgeable by eye on a 1920px frame. As numbers
/// it is three checks per glyph: which face it resolves to, whether its advance
/// matches the cell, and where its ink lands relative to the cell edge.
pub fn type_report() {
    let fonts = load_fonts();
    let mut cache: HashMap<u64, Raster> = HashMap::new();

    println!(
        "grid {}x{} cells of {}x{} px  ->  {}x{} px · base glyph size {} px",
        COLS, ROWS, CELL_W, CELL_H, W, H, BASE_PX
    );
    if fonts.cjk.is_none() {
        println!("!! no CJK face found (msyh.ttc / SIMHEI.TTF) — wide glyphs will be blank");
    }
    println!();

    // advance vs the slot it occupies: any mismatch accumulates across a row
    for (name, probe, cw) in [("latin", 'M', 1usize), ("cjk", '漢', 2)] {
        if name == "cjk" && fonts.cjk.is_none() {
            continue;
        }
        let r = cell_raster_cached(&mut cache, &fonts, probe, cw, false);
        let slot = (cw * CELL_W) as f32;
        let err = r.advance - slot;
        let cells = COLS / cw;
        println!(
            "{:<5} probe '{}'  advance {:6.2} px  slot {:5.1} px ({} cell)  err {:+5.2} px  \
             (row drift {:+6.1} px over {} glyphs)",
            name, probe, r.advance, slot, cw, err, err * cells as f32, cells
        );
    }
    println!();

    println!(
        "{:<3} {:>2} {:>10} {:>8} {:>5} {:>5} {:>7} {:>8} {:>7} {:>7}",
        "ch", "cw", "face", "advance", "xmin", "inkW", "pen_x", "ink_l", "vs cell", "was"
    );
    // every glyph class the film actually draws
    let sample = "ilMW.|;:!'(),-+_/[]{}=*%&#@abcXYZ0189\
                  ─│┌┐└┘╭╮╰╯═║█▌▔▁┊┃━╸\
                  ●○◆▼■★☆♪♫♥√×·°˙\
                  精 神 再 生 · 人 間 は 、 な ま る 。";
    let mut missing = Vec::new();
    let mut max_off = 0.0f32;
    let mut worst_shift = 0.0f32;
    for ch in sample.chars() {
        if ch == ' ' {
            continue;
        }
        let r = cell_raster_cached(&mut cache, &fonts, ch, crate::gfx::char_w(ch), false);
        let cw = crate::gfx::char_w(ch);
        let face = fonts.pick(ch, false);
        // lay the glyph out in cell #1 so "vs cell" is directly comparable
        let pen = cell_pen_x(1, cw, &r);
        let ink_l = pen + r.xmin as f32;
        let off = ink_l - CELL_W as f32;
        // what the previous ink-width centring would have produced
        let span = (cw.max(1) * CELL_W) as f32;
        let was = CELL_W as f32 + (span - r.w as f32) * 0.5;
        let shift = ink_l - was;
        if r.w == 0 {
            missing.push(ch);
        } else if cw == 1 {
            max_off = max_off.max(off.abs());
            worst_shift = worst_shift.max(shift.abs());
        }
        println!(
            "{:<3} {:>2} {:>10} {:>8.2} {:>5} {:>5} {:>7.1} {:>8.1} {:>+7.1} {:>+7.1}",
            ch, cw, format!("{face:?}"), r.advance, r.xmin, r.w, pen, ink_l, off, shift
        );
    }
    println!();
    if missing.is_empty() {
        println!("all sample glyphs resolved to a real outline");
    } else {
        println!(
            "!! {} glyph(s) have no outline in any face: {}",
            missing.len(),
            missing.iter().collect::<String>()
        );
    }
    println!("max |ink_l - cell_edge| for 1-cell glyphs: {max_off:.2} px");
    println!("max horizontal correction vs the old ink-centred layout: {worst_shift:.2} px");
}

// ------------------------------------------------------------- output ---

impl Canvas {
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

