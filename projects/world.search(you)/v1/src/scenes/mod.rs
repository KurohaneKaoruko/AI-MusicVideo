//! Scene graph: the MV timeline. Each section owns a time range and a renderer;
//! transitions blend between them.

pub mod boot;
pub mod bridge;
pub mod browse;
pub mod buckets;
pub mod chorus;
pub mod ending;
pub mod flower;
pub mod human;
pub mod outro;
pub mod radar;
pub mod sortui;
pub mod ui;

use crate::beats::BeatGrid;
use crate::fx;
use crate::gfx::{Color, Grid};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sc {
    Boot,
    SortUI,
    Table,
    Eggplant,
    Cat,
    Steak,
    Radar,
    Flower,
    Human,
    Bridge,
    Chorus,
    Buckets,
    MemFlower,
    MemHuman,
    Outro,
    Ending,
}

#[derive(Clone, Copy)]
pub struct Section {
    pub t0: f64,
    pub t1: f64,
    pub sc: Sc,
    pub label: &'static str,
    pub accent: Color,
    /// transition INTO this section
    pub trans: Trans,
    /// transition weight (0..1 extra intensity)
    pub trans_w: f64,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Trans {
    None,
    Wipe,
    Glitch,
    Crt,
    Curtain,
    Fade,
}

/// the spine of the MV — times follow the song's LRC exactly
pub static SECTIONS: &[Section] = &[
    Section { t0: 0.0,    t1: 14.36,  sc: Sc::Boot,      label: "boot",            accent: Color::hex(0x8a93a6), trans: Trans::Fade,   trans_w: 0.2 },
    Section { t0: 14.36,  t1: 40.90,  sc: Sc::SortUI,    label: "memory.sort",     accent: Color::hex(0xe8c39e), trans: Trans::Wipe,   trans_w: 1.0 },
    Section { t0: 40.90,  t1: 54.46,  sc: Sc::Table,     label: "you.table",       accent: Color::hex(0xd9a066), trans: Trans::Glitch, trans_w: 0.8 },
    Section { t0: 54.46,  t1: 68.17,  sc: Sc::Eggplant,  label: "you.eggplant",    accent: Color::hex(0x9a6dd7), trans: Trans::Glitch, trans_w: 0.8 },
    Section { t0: 68.17,  t1: 82.89,  sc: Sc::Radar,     label: "searching",       accent: Color::hex(0xff6d8a), trans: Trans::Wipe,   trans_w: 0.7 },
    Section { t0: 82.89,  t1: 96.64,  sc: Sc::Cat,       label: "you.cat",         accent: Color::hex(0x9fb2cf), trans: Trans::Glitch, trans_w: 0.7 },
    Section { t0: 96.64,  t1: 109.72, sc: Sc::Steak,     label: "you.steak",       accent: Color::hex(0xc9a86a), trans: Trans::Glitch, trans_w: 0.7 },
    Section { t0: 109.72, t1: 123.44, sc: Sc::Flower,    label: "you.flower",      accent: Color::hex(0xf2a7c3), trans: Trans::Crt,    trans_w: 0.9 },
    Section { t0: 123.44, t1: 137.09, sc: Sc::Human,     label: "you.human",       accent: Color::hex(0xff9e5e), trans: Trans::Crt,    trans_w: 0.9 },
    Section { t0: 137.09, t1: 151.84, sc: Sc::Radar,     label: "searching",       accent: Color::hex(0xff6d8a), trans: Trans::Glitch, trans_w: 1.0 },
    Section { t0: 151.84, t1: 179.47, sc: Sc::Bridge,    label: "u-um…",           accent: Color::hex(0x8a93a6), trans: Trans::Curtain,trans_w: 0.8 },
    Section { t0: 179.47, t1: 207.14, sc: Sc::Chorus,    label: "everywhere",      accent: Color::hex(0xff6d8a), trans: Trans::Glitch, trans_w: 1.0 },
    Section { t0: 207.14, t1: 233.77, sc: Sc::Buckets,   label: "merge/evolve",    accent: Color::hex(0x6fc7b8), trans: Trans::Wipe,   trans_w: 0.9 },
    Section { t0: 233.77, t1: 247.37, sc: Sc::MemFlower, label: "memory: flower",  accent: Color::hex(0xd9c3a5), trans: Trans::Curtain,trans_w: 0.7 },
    Section { t0: 247.37, t1: 261.06, sc: Sc::MemHuman,  label: "memory: human",   accent: Color::hex(0xd9c3a5), trans: Trans::Curtain,trans_w: 0.7 },
    Section { t0: 261.06, t1: 275.75, sc: Sc::Radar,     label: "searching (dim)", accent: Color::hex(0x55607a), trans: Trans::Fade,   trans_w: 0.5 },
    Section { t0: 275.75, t1: 290.20, sc: Sc::Outro,     label: "outro",           accent: Color::hex(0xe8c39e), trans: Trans::Wipe,   trans_w: 0.6 },
    Section { t0: 290.20, t1: 1e9,    sc: Sc::Ending,    label: "— fin —",         accent: Color::hex(0xff6d8a), trans: Trans::Fade,   trans_w: 0.3 },
];

pub struct Ctx<'a> {
    pub beats: &'a BeatGrid,
    pub dur: f64,
}

pub struct Ui {
    pub paused: bool,
    pub volume: f32,
    pub anim: f64, // free-running time (drives blinks even when audio is done)
    pub finished: bool,
    pub show_fps: bool,
    pub fps: f64,
}

pub fn section_at(t: f64) -> &'static Section {
    for s in SECTIONS {
        if t < s.t1 {
            return s;
        }
    }
    SECTIONS.last().unwrap()
}

fn radar_cfg(sec: &Section) -> radar::RadarCfg {
    if sec.t0 < 100.0 {
        radar::RadarCfg { sweep_speed: 2.2, blips: 9, dim: 0.9, glitch: false, heart: true }
    } else if sec.t0 < 200.0 {
        radar::RadarCfg { sweep_speed: 3.4, blips: 14, dim: 1.0, glitch: true, heart: true }
    } else {
        radar::RadarCfg { sweep_speed: 1.3, blips: 5, dim: 0.45, glitch: false, heart: false }
    }
}

pub fn render(g: &mut Grid, t: f64, ui: &Ui, ctx: &Ctx) {
    let sec = section_at(t);
    let tl = t - sec.t0;

    match sec.sc {
        Sc::Boot => boot::render(g, tl, t, ctx),
        Sc::SortUI => sortui::render(g, tl, t, ctx),
        Sc::Table => browse::render(g, tl, t, ctx, browse::Obj::Table),
        Sc::Eggplant => browse::render(g, tl, t, ctx, browse::Obj::Eggplant),
        Sc::Cat => browse::render(g, tl, t, ctx, browse::Obj::Cat),
        Sc::Steak => browse::render(g, tl, t, ctx, browse::Obj::Steak),
        Sc::Radar => radar::render(g, tl, t, ctx, radar_cfg(sec)),
        Sc::Flower => flower::render(g, tl, t, ctx, false),
        Sc::MemFlower => flower::render(g, tl, t, ctx, true),
        Sc::Human => human::render(g, tl, t, ctx, false),
        Sc::MemHuman => human::render(g, tl, t, ctx, true),
        Sc::Bridge => bridge::render(g, tl, t, ctx),
        Sc::Chorus => chorus::render(g, tl, t, ctx),
        Sc::Buckets => buckets::render(g, tl, t, ctx),
        Sc::Outro => outro::render(g, tl, t, ctx),
        Sc::Ending => ending::render(g, tl, ui.anim, ctx),
    }

    // transition into this section (first ~0.5s)
    apply_transition(g, t, sec, ui);
}

fn apply_transition(g: &mut Grid, t: f64, sec: &Section, ui: &Ui) {
    let dur = 0.55;
    let local = t - sec.t0;
    if local > dur || sec.trans == Trans::None {
        return;
    }
    let p = (local / dur).clamp(0.0, 1.0);
    let bg = g.bg;
    let intensity = sec.trans_w * ui.anim.sin(); // tiny variation
    match sec.trans {
        Trans::Wipe => {
            let frac = (1.0 - p).powi(2);
            fx::wipe(g, frac, bg, sec.accent);
        }
        Trans::Glitch => {
            let amt = (1.0 - p) * sec.trans_w * (0.6 + 0.4 * intensity);
            if amt > 0.02 {
                fx::glitch(g, amt, (ui.anim * 30.0) as u64 ^ 0x9E37);
            }
        }
        Trans::Crt => {
            fx::crt_collapse(g, 1.0 - p, bg);
        }
        Trans::Curtain => {
            let frac = (1.0 - p).powi(3);
            fx::curtain(g, frac, bg);
        }
        Trans::Fade => {
            fx::fade(g, (1.0 - p) * 0.9);
        }
        Trans::None => {}
    }
}

/// lyrics with the right anchor per section
pub fn render_lyrics(g: &mut Grid, t: f64, ctx: &Ctx) {
    let sec = section_at(t);
    let anchor = match sec.sc {
        Sc::Radar | Sc::Bridge => crate::lyrics::Anchor::Upper,
        Sc::Chorus => crate::lyrics::Anchor::Center,
        _ => crate::lyrics::Anchor::Bottom,
    };
    // the boot and ending scenes own their own words
    if sec.sc == Sc::Boot || sec.sc == Sc::Ending {
        return;
    }
    let style = crate::lyrics::LyricStyle { tint: sec.accent, anchor };
    crate::lyrics::render(g, t, ctx.beats, &style);
}
