//! Scene timeline: 14 sections mapped to the song, plus the render dispatcher.

pub mod boot;
pub mod common;
pub mod doll;
pub mod eden;
pub mod fairy;
pub mod fight;
pub mod flight;
pub mod forge;
pub mod hakoniwa;
pub mod hanamaru;
pub mod light;
pub mod query;
pub mod redact;
pub mod source;
pub mod tears;

use crate::audio::Energy;
use crate::beats::BeatGrid;
use crate::gfx::{pal, Color, Grid};
use crate::pix::Canvas;

pub struct Section {
    pub t: f64,
    pub label: &'static str,
    pub col: u32,
}

impl Section {
    pub fn color(&self) -> Color {
        Color::hex(self.col)
    }
}

pub static SECTIONS: &[Section] = &[
    Section { t: 0.0, label: "BOOT", col: pal::ICE_DEEP },
    Section { t: 10.89, label: "BLUE FAIRY", col: pal::ICE },
    Section { t: 17.21, label: "QUERY", col: pal::MID },
    Section { t: 23.84, label: "SOUL FORGE", col: pal::GOLD },
    Section { t: 37.89, label: "TEARS", col: pal::TEAR },
    Section { t: 58.14, label: "REDACT", col: pal::GOLD_DEEP },
    Section { t: 66.0, label: "HAKONIWA", col: pal::GREEN },
    Section { t: 76.0, label: "EDEN FLIGHT", col: pal::STAR },
    Section { t: 87.62, label: "EDEN", col: pal::GOLD },
    Section { t: 100.96, label: "DOLL", col: pal::RED },
    Section { t: 107.68, label: "LIGHT", col: pal::AMBER },
    Section { t: 111.07, label: "FIGHT", col: pal::RED },
    Section { t: 115.03, label: "CHORUS", col: pal::TEAR },
    Section { t: 135.73, label: "SOURCE", col: pal::GOLD },
    Section { t: 157.0, label: "HANAMARU", col: pal::GOLD },
];

pub fn section_at(t: f64) -> &'static Section {
    let mut cur = &SECTIONS[0];
    for s in SECTIONS {
        if t >= s.t {
            cur = s;
        } else {
            break;
        }
    }
    cur
}

pub struct Ctx<'a> {
    pub beats: &'a BeatGrid,
    pub energy: &'a Energy,
    pub dur: f64,
}

/// Base background color per section (very dark, subtly tinted).
pub fn bg_at(t: f64) -> Color {
    use pal::*;
    let sec = section_at(t);
    let base = match sec.label {
        "BLUE FAIRY" => Color::hex(0x030509),
        "SOUL FORGE" => Color::hex(0x060710),
        "TEARS" => Color::hex(0x040710),
        "REDACT" => Color::hex(0x05070c),
        "EDEN FLIGHT" => Color::hex(0x02030a),
        "EDEN" => Color::hex(0x070806),
        "LIGHT" => Color::hex(0x020308),
        "FIGHT" => Color::hex(0x070406),
        "CHORUS" => Color::hex(0x040812),
        "SOURCE" => Color::hex(0x050709),
        "HANAMARU" => Color::hex(0x080705),
        _ => Color::hex(BG0),
    };
    // gentle breathing with the music is added per-scene; keep base static here
    base
}

/// Render the stage (scenes draw under lyrics & HUD chrome).
pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, ctx: &Ctx) {
    let idx = SECTIONS
        .iter()
        .rposition(|s| t >= s.t)
        .unwrap_or(0);
    let sec = &SECTIONS[idx];
    let t1 = SECTIONS.get(idx + 1).map(|s| s.t).unwrap_or(ctx.dur + 1.0);
    let lt = t - sec.t; // local time
    let sd = t1 - sec.t; // section duration

    match sec.label {
        "BOOT" => boot::render(g, cv, t, lt, sd, ctx),
        "BLUE FAIRY" => fairy::render(g, cv, t, lt, sd, ctx),
        "QUERY" => query::render(g, cv, t, lt, sd, ctx),
        "SOUL FORGE" => forge::render(g, cv, t, lt, sd, ctx),
        "TEARS" => tears::render(g, cv, t, lt, sd, ctx),
        "REDACT" => redact::render(g, cv, t, lt, sd, ctx),
        "HAKONIWA" => hakoniwa::render(g, cv, t, lt, sd, ctx),
        "EDEN FLIGHT" => flight::render(g, cv, t, lt, sd, ctx),
        "EDEN" => eden::render(g, cv, t, lt, sd, ctx),
        "DOLL" => doll::render(g, cv, t, lt, sd, ctx),
        "LIGHT" => light::render(g, cv, t, lt, sd, ctx),
        "FIGHT" => fight::render(g, cv, t, lt, sd, ctx),
        "CHORUS" => tears::render_chorus(g, cv, t, lt, sd, ctx),
        "SOURCE" => source::render(g, cv, t, lt, sd, ctx),
        "HANAMARU" => hanamaru::render(g, cv, t, lt, sd, ctx),
        _ => {}
    }
}
