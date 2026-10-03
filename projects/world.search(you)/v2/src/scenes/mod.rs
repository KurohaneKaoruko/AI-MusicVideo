//! The film's timeline. Each act owns a stretch of the song, its own
//! environment, its own camera and — this is the point of v2 — its own
//! composition. Nothing is shared except the reticle and the rose.

pub mod bloom;
pub mod bridge;
pub mod cells;
pub mod chorus;
pub mod ending;
pub mod index;
pub mod memory;
pub mod query;
pub mod radar;
pub mod specimen;
pub mod ui;

use crate::beats::BeatGrid;
use crate::cam::Cam;
use crate::gfx::{pal, Color, Grid};
use crate::pix::Canvas;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sc {
    Query,
    Sort,
    Table,
    Eggplant,
    Radar,
    Cat,
    Steak,
    Flower,
    Hands,
    RadarGlitch,
    Bridge,
    Chorus,
    Merge,
    MemFlower,
    MemHands,
    RadarDim,
    Outro,
    Fin,
}

impl Sc {
    pub fn label(self) -> &'static str {
        match self {
            Sc::Query => "world.search --target you",
            Sc::Sort => "memory.sort",
            Sc::Table => "you.table",
            Sc::Eggplant => "you.eggplant",
            Sc::Radar => "searching",
            Sc::Cat => "you.cat",
            Sc::Steak => "you.steak",
            Sc::Flower => "you.flower",
            Sc::Hands => "you.human",
            Sc::RadarGlitch => "searching (unstable)",
            Sc::Bridge => "u-um…",
            Sc::Chorus => "everywhere",
            Sc::Merge => "merge / evolve",
            Sc::MemFlower => "memory: flower",
            Sc::MemHands => "memory: human",
            Sc::RadarDim => "searching (dim)",
            Sc::Outro => "you.dog / you.tomato",
            Sc::Fin => "exit code 0",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Section {
    pub t0: f64,
    pub t1: f64,
    pub sc: Sc,
    pub accent: Color,
}

/// The spine of the film — times follow the song's LRC exactly.
pub static SECTIONS: &[Section] = &[
    Section { t0: 0.0, t1: 14.36, sc: Sc::Query, accent: Color::hex(0x8a93a6) },
    Section { t0: 14.36, t1: 40.90, sc: Sc::Sort, accent: Color::hex(0xe8c39e) },
    Section { t0: 40.90, t1: 54.46, sc: Sc::Table, accent: Color::hex(0xd9a066) },
    Section { t0: 54.46, t1: 68.17, sc: Sc::Eggplant, accent: Color::hex(0x9a6dd7) },
    Section { t0: 68.17, t1: 82.89, sc: Sc::Radar, accent: Color::hex(0xff6d8a) },
    Section { t0: 82.89, t1: 96.64, sc: Sc::Cat, accent: Color::hex(0x9fb2cf) },
    Section { t0: 96.64, t1: 109.72, sc: Sc::Steak, accent: Color::hex(0xc9a86a) },
    Section { t0: 109.72, t1: 123.44, sc: Sc::Flower, accent: Color::hex(0xf2a7c3) },
    Section { t0: 123.44, t1: 137.09, sc: Sc::Hands, accent: Color::hex(0xff9e5e) },
    Section { t0: 137.09, t1: 151.84, sc: Sc::RadarGlitch, accent: Color::hex(0xff6d8a) },
    Section { t0: 151.84, t1: 179.47, sc: Sc::Bridge, accent: Color::hex(0x8a93a6) },
    Section { t0: 179.47, t1: 207.14, sc: Sc::Chorus, accent: Color::hex(0xff6d8a) },
    Section { t0: 207.14, t1: 233.77, sc: Sc::Merge, accent: Color::hex(0x6fc7b8) },
    Section { t0: 233.77, t1: 247.37, sc: Sc::MemFlower, accent: Color::hex(0xd9c3a5) },
    Section { t0: 247.37, t1: 261.06, sc: Sc::MemHands, accent: Color::hex(0xd9c3a5) },
    Section { t0: 261.06, t1: 275.75, sc: Sc::RadarDim, accent: Color::hex(0x55607a) },
    Section { t0: 275.75, t1: 290.20, sc: Sc::Outro, accent: Color::hex(0xe8c39e) },
    Section { t0: 290.20, t1: 1e9, sc: Sc::Fin, accent: Color::hex(0xff6d8a) },
];

pub struct Ctx<'a> {
    pub beats: &'a BeatGrid,
    pub energy: &'a crate::audio::Energy,
    pub dur: f64,
}

pub fn section_at(t: f64) -> &'static Section {
    for s in SECTIONS {
        if t < s.t1 {
            return s;
        }
    }
    SECTIONS.last().unwrap()
}

/// Base background of the frame at `t` (the void behind everything).
pub fn bg_at(t: f64) -> Color {
    let s = section_at(t);
    match s.sc {
        Sc::MemFlower | Sc::MemHands => Color::hex(0x100c08),
        Sc::Outro => Color::hex(0x0e1016),
        Sc::Merge => Color::hex(0x06100f),
        Sc::Chorus => Color::hex(0x0a0710),
        Sc::Bridge => Color::hex(0x07080c),
        _ => pal::c(pal::BG0),
    }
}

/// Should the diegetic search readout be on screen?
pub fn chrome_at(t: f64) -> bool {
    let s = section_at(t);
    let tl = t - s.t0;
    match s.sc {
        Sc::Query => tl > 5.2,
        Sc::Sort | Sc::Table | Sc::Eggplant | Sc::Cat | Sc::Steak => true,
        Sc::Radar | Sc::RadarGlitch => true,
        Sc::Flower => tl < 4.0,
        Sc::Hands => false,
        Sc::Bridge => tl > 24.0,
        Sc::Chorus => false,
        Sc::Merge => tl < 8.0,
        Sc::MemFlower | Sc::MemHands => false,
        Sc::RadarDim => true,
        Sc::Outro => false,
        Sc::Fin => tl > 4.5,
    }
}

/// Camera for the current moment. Every act gets its own move.
pub fn cam_at(t: f64) -> Cam {
    let s = section_at(t);
    let tl = t - s.t0;
    let span = (s.t1 - s.t0).min(30.0).max(1.0);
    let u = (tl / span).clamp(0.0, 1.2) as f32;
    let mut c = Cam::default();
    match s.sc {
        // a slow push in as the program wakes
        Sc::Query => {
            c.zoom = 1.0 + 0.10 * u;
            c.fy = crate::pix::H as f32 * 0.5;
        }
        // drifting across the sorting grid
        Sc::Sort => {
            c.zoom = 1.04;
            c.x = -30.0 + 60.0 * u;
        }
        // a wide room, slowly dollying right
        Sc::Table => {
            c.zoom = 1.02;
            c.x = -40.0 + 80.0 * u;
        }
        // a little handheld sway in the kitchen
        Sc::Eggplant => {
            c.zoom = 1.06;
            c.x = (tl as f32 * 0.35).sin() * 10.0;
            c.y = (tl as f32 * 0.27).cos() * 7.0;
        }
        // radar pulls in on every pass
        Sc::Radar => {
            let cyc = ((tl / 12.0) % 1.0) as f32;
            c.zoom = 1.0 + 0.16 * cyc;
        }
        Sc::RadarGlitch => {
            c.zoom = 1.10;
            c.shake = 2.4;
        }
        Sc::RadarDim => {
            c.zoom = 0.94 + 0.05 * u;
        }
        // the cat: a still, low frame, barely breathing
        Sc::Cat => {
            c.zoom = 1.12 - 0.05 * u;
            c.y = -20.0;
        }
        // top-down on the steak, a slow rotation-ish drift
        Sc::Steak => {
            c.zoom = 1.18 - 0.10 * u;
        }
        // the flower grows: the camera rises with it
        Sc::Flower => {
            c.zoom = 1.22 - 0.12 * u;
            c.y = 26.0 - 46.0 * u;
        }
        // the hands cradle the frame, very close and very still
        Sc::Hands => {
            c.zoom = 1.16 + 0.05 * u;
            c.y = 12.0;
        }
        // the bridge falls apart
        Sc::Bridge => {
            c.zoom = 1.0 + 0.05 * u;
            c.x = (tl as f32 * 0.9).sin() * 4.0;
            c.shake = if tl > 20.0 { 3.0 } else { 0.6 };
        }
        // the chorus is wide, then the world opens
        Sc::Chorus => {
            c.zoom = 0.96 + 0.12 * u;
        }
        Sc::Merge => {
            c.zoom = 1.0 + 0.22 * u;
            c.fy = crate::pix::H as f32 * 0.5;
        }
        Sc::MemFlower | Sc::MemHands => {
            c.zoom = 1.0 + 0.02 * ((tl as f32) * 0.4).sin();
        }
        Sc::Outro => {
            c.zoom = 1.02;
            c.x = (tl as f32 * 0.45).sin() * 14.0;
            c.y = (tl as f32 * 0.3).cos() * 6.0;
        }
        Sc::Fin => {
            c.zoom = 1.18 - 0.16 * u.min(1.0);
        }
    }
    c
}

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, ctx: &Ctx) {
    let s = section_at(t);
    let tl = t - s.t0;
    // per-scene global post-FX budget
    cv.fx.seed = ((t * 60.0) as u64) | 1;
    match s.sc {
        Sc::Query => query::render(g, cv, tl, t, ctx),
        Sc::Sort => index::render(g, cv, tl, t, ctx),
        Sc::Table => specimen::table(g, cv, tl, t, ctx),
        Sc::Eggplant => specimen::eggplant(g, cv, tl, t, ctx),
        Sc::Cat => specimen::cat(g, cv, tl, t, ctx),
        Sc::Steak => specimen::steak(g, cv, tl, t, ctx),
        Sc::Radar => radar::render(g, cv, tl, t, ctx, radar::RadarCfg::calm()),
        Sc::RadarGlitch => radar::render(g, cv, tl, t, ctx, radar::RadarCfg::glitch()),
        Sc::RadarDim => radar::render(g, cv, tl, t, ctx, radar::RadarCfg::dim()),
        Sc::Flower => bloom::flower(g, cv, tl, t, ctx),
        Sc::Hands => bloom::hands(g, cv, tl, t, ctx),
        Sc::Bridge => bridge::render(g, cv, tl, t, ctx),
        Sc::Chorus => chorus::render(g, cv, tl, t, ctx),
        Sc::Merge => cells::render(g, cv, tl, t, ctx),
        Sc::MemFlower => memory::flower(g, cv, tl, t, ctx),
        Sc::MemHands => memory::hands(g, cv, tl, t, ctx),
        Sc::Outro => ending::outro(g, cv, tl, t, ctx),
        Sc::Fin => ending::fin(g, cv, tl, t, ctx),
    }
}
