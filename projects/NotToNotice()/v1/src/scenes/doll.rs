//! S09 DOLL — `while (be_a_doll) { focus(mission); suppress(tears); }`
//! A marionette on strings, jerking through its loop until it cracks.

use crate::art;
use crate::gfx::{pal, Grid};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, _sd: f64, ctx: &Ctx) {
    let w = g.w as i64;
    let beat = ctx.beats;
    let energy = ctx.energy;

    // ---- wake from the touch flash
    cv.fx.fade = (1.0 - crate::gfx::clamp01(lt / 0.5)) * 0.6;

    // ---- the loop, top-left, metronome-tight
    let lines = [
        "while (be_a_doll) {",
        "  focus(mission);",
        "  suppress(tears);",
        "}",
    ];
    let bx = 4;
    let by = 5;
    for (i, line) in lines.iter().enumerate() {
        let a = crate::gfx::clamp01((lt - i as f64 * 0.25) / 0.3);
        let col = match i {
            0 => pal::c(pal::ICE),
            3 => pal::c(pal::ICE_DEEP),
            _ => pal::c(pal::BRIGHT).scale(0.85),
        };
        g.text_alpha(bx, by + i as i64, line, col, a);
    }
    // iteration counter spinning
    let iters = 4_291_038u64 + ((t * 148.0) as u64);
    let counter = format!("loop × {:>12}", iters);
    g.text_alpha(bx, by + 5, &counter, pal::c(pal::DIM), crate::gfx::clamp01((lt - 1.2) / 0.4));

    // ---- marionette strings (sway on the beat)
    let doll_w = 11i64;
    let dx = w / 2 - doll_w / 2;
    let dy = 12;
    let sway = ((t * 1.4).sin() * 0.7
        + beat.pulse(t) * 0.9 * (if beat.beat_index(t).rem_euclid(2) == 0 { 1.0 } else { -1.0 }))
        as i64;
    let anchors = [(dx + 5, dy), (dx - 1, dy + 4), (dx + 11, dy + 4)];
    for (ax, ay) in anchors {
        let sx = ax + sway / 2;
        let steps = ay - STAGE_Y0;
        for k in 0..=steps {
            let y = STAGE_Y0 + k;
            let x = stage_x(sx, ax, STAGE_Y0, ay, y);
            if y < ay {
                g.put_alpha(x, y, if k % 3 == 0 { '¦' } else { '|' }, pal::c(pal::DIM), 0.55);
            }
        }
        g.put_alpha(ax + sway / 2, ay, '·', pal::c(pal::MID), 0.8);
    }
    // control bar
    g.text_alpha(dx - 3 + sway / 2, STAGE_Y0, "┌───────┐", pal::c(pal::DIM), 0.8);

    // ---- the doll: pose alternates with the beat, jerks on downbeats
    let pose = if beat.beat_index(t).rem_euclid(2) == 0 { &art::DOLL_A } else { &art::DOLL_B };
    let jerk = (beat.pulse(t) * 1.4) as i64;
    let alpha = crate::gfx::clamp01(lt / 0.6);
    art::draw(g, pose, dx + sway + jerk, dy, alpha, None);

    // ---- cracks accumulate (2.2+)
    if lt > 2.2 {
        let crack_n = (((lt - 2.2) / 0.9) as usize).min(6);
        let spots = [(3i64, 2), (7, 3), (5, 5), (2, 7), (8, 8), (5, 10)];
        let mut rng = crate::gfx::Rng::new(((t * 9.0) as u64) ^ 0xD0);
        for i in 0..crack_n {
            let (cx_, cy_) = spots[i];
            let ch = if rng.f64() < 0.5 { ',' } else { '\\' };
            g.put_alpha(dx + sway + jerk + cx_, dy + cy_, ch, pal::c(pal::RED), 0.75 + 0.2 * rng.f64());
        }
    }
    // ---- the fracture in the chest glows (4.5+)
    if lt > 4.5 {
        let gl = 0.4 + 0.35 * (t * 5.0).sin().max(0.0) + 0.3 * beat.pulse(t);
        g.put_alpha(dx + sway + jerk + 5, dy + 6, '◆', pal::c(pal::RED).glow(0.4), gl.min(1.0));
        if beat.pulse(t) > 0.85 {
            art::burst(g, dx + sway + jerk + 5, dy + 6, 0.3, pal::c(pal::RED));
        }
    }

    // ---- heartbeat caption on "Focus on the mission" (3.5 ~ 6)
    if lt > 3.5 && lt < 6.0 {
        let bpm_txt = format!("mission tempo: {} bpm — steady, steady", ctx.beats.bpm.round() as i64);
        caption(g, 29, &bpm_txt, pal::c(pal::DIM), 0.7);
    }
    // final fracture warning
    if lt > 5.9 {
        let p = crate::gfx::clamp01((lt - 5.9) / 0.4);
        caption(g, 28, "WARNING: vessel integrity 62% … 61% …", pal::c(pal::RED), p * 0.9);
    }

    // ---- fx
    cv.fx.bloom = 0.3 + 0.2 * energy.rms_at(t) as f64;
    cv.fx.grain = 0.06;
    cv.fx.scanline = 0.45;
    // glitch ticks on downbeats
    if beat.downbeat(t) && beat.beat_phase(t) < 0.1 {
        cv.fx.glitch = 0.3 * (1.0 - beat.beat_phase(t) * 10.0);
        cv.fx.aberration = 0.4;
    }
    // strings snap on the way out
    if lt > 6.2 {
        let p = ((lt - 6.2) / 0.5).min(1.0);
        cv.fx.glitch = 0.5 * p;
        cv.fx.aberration = 0.7 * p;
    }
}

fn stage_x(sx: i64, ax: i64, sy: i64, ay: i64, y: i64) -> i64 {
    if ay <= sy {
        return ax;
    }
    let p = (y - sy) as f64 / (ay - sy) as f64;
    (sx as f64 * (1.0 - p) + ax as f64 * p).round() as i64
}
