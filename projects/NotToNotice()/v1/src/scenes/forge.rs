//! S03 SOUL FORGE — personality data rains down and becomes a human shape.

use crate::art;
use crate::gfx::{pal, Grid};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

const SOUL_ROWS: i64 = 14;

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, _sd: f64, ctx: &Ctx) {
    let w = g.w as i64;
    let beat = ctx.beats;
    let energy = ctx.energy;

    // ---- palette shifts to ice after 30.66 (6.82 local)
    let ice_t = crate::gfx::clamp01((lt - 6.82) / 1.2);
    let data_col = pal::Color::lerp(pal::c(pal::GOLD), pal::c(pal::ICE), ice_t);

    // ---- glyph rain (personality data pouring into the forge)
    let density = 0.30 + 0.40 * energy.rms_at(t) as f64;
    crate::fx::glyph_rain(g, t, 2, STAGE_Y0, w - 4, STAGE_H, density, data_col.scale(0.85), 4242);

    // ---- forge frame
    let fw = 30;
    let fx0 = w / 2 - fw / 2;
    let fy0 = 6;
    let fh = SOUL_ROWS + 3;
    let forge_open = crate::gfx::clamp01(lt / 0.5);
    if forge_open > 0.0 {
        g.box_rounded(
            fx0 - 2,
            fy0 - 1,
            fw + 4,
            fh + 1,
            pal::c(pal::ICE_DEEP).scale(0.9),
            Some(pal::c(pal::BG1)),
            Some(("soul forge", data_col.scale(0.95))),
        );
    }

    // ---- assemble the silhouette row by row on the beats
    let asm_start = 1.0;
    let asm_end = 6.6;
    let ap = crate::gfx::clamp01((lt - asm_start) / (asm_end - asm_start));
    // quantize to beats for a stitched, mechanical feel
    let steps = 14.0;
    let bstep = (ap * steps).floor() / steps;
    let shown_rows = (bstep * SOUL_ROWS as f64).ceil() as i64;
    let sprite_x = fx0 + (fw - 13) / 2;
    let sprite_y = fy0 + 1;
    if shown_rows > 0 {
        // draw the sprite clipped to shown_rows
        let spr = &art::SOUL;
        let rows = &spr.rows[..shown_rows.min(SOUL_ROWS) as usize];
        let clipped = art::Sprite { rows, pal: spr.pal };
        // thread: latest row shimmers
        art::draw(g, &clipped, sprite_x, sprite_y, 1.0, None);
        let cur_row = sprite_y + shown_rows - 1;
        for dx in 0..13 {
            g.put_alpha(sprite_x + dx, cur_row, '▒', data_col.glow(0.3), 0.25 + 0.2 * beat.pulse(t));
        }
    }

    // ---- "Eve become human" (lt 6.82): eyes open bright
    if lt > 6.82 {
        let e = hit_envelope(lt, 6.82, 0.1, 1.4);
        let ex = sprite_x + 4;
        let ey = sprite_y + 2;
        g.put(ex, ey, '@', pal::c(pal::WHITE).glow(0.4 * e));
        g.put(ex + 3, ey, '@', pal::c(pal::WHITE).glow(0.4 * e));
        cv.fx.bloom += 0.12 * e;
    }

    // ---- "Create soul for eve" (lt 9.34): completion stamp + heartbeat
    if lt > 9.34 {
        let p = crate::gfx::clamp01((lt - 9.34) / 0.4);
        let stamp = "E.V.E — REGISTERED";
        let sx = (w - Grid::measure(stamp)) / 2;
        g.text_alpha(sx, fy0 + fh + 1, stamp, pal::c(pal::GOLD).glow(0.3), p);
        // expanding heartbeat rings on downbeats
        let hb = beat.beat_index(t);
        if hb.rem_euclid(4) == 0 {
            let ph = beat.beat_phase(t);
            let r = (ph * 7.0) as i64 + 1;
            ring(g, sprite_x + 6, sprite_y + 6, r, data_col.glow(0.25), 0.5 * (1.0 - ph));
        }
    }

    // ---- progress + caption
    if lt > 1.0 && lt < 9.34 {
        let pp = ap;
        let label = format!("weaving personality … {:>3.0}%", pp * 100.0);
        caption(g, fy0 + fh + 1, &label, pal::c(pal::MID), 0.85);
    } else if lt > 9.34 {
        caption(g, fy0 + fh + 3, "a soul, newly made — warm to the touch", pal::c(pal::GOLD_DEEP), crate::gfx::clamp01((lt - 9.8) / 0.5) * 0.85);
    }

    // ---- rising soul-lights near the end
    if lt > 10.0 {
        rising_lights(g, cv, t, 14, 313, pal::c(pal::GOLD), 0.5);
    }

    // snow of ice phase
    if ice_t > 0.0 {
        motes(g, t, 26, pal::c(pal::ICE_PALE), 88, 0.35 * ice_t);
    }

    // ---- fx
    cv.fx.bloom = 0.32 + 0.25 * energy.rms_at(t) as f64 + 0.2 * ice_t;
    cv.fx.grain = 0.06;
    cv.fx.scanline = 0.4;
    let stamp_hit = hit_envelope(lt, 9.34, 0.05, 0.6);
    cv.fx.flash = stamp_hit * 0.35;
    cv.fx.aberration = stamp_hit * 0.5;
}

/// expanding circle ring of dots
fn ring(g: &mut Grid, cx: i64, cy: i64, r: i64, col: pal::Color, alpha: f64) {
    if r <= 0 || alpha <= 0.02 {
        return;
    }
    let n = (r * 6).max(8);
    for k in 0..n {
        let a = k as f64 / n as f64 * std::f64::consts::PI * 2.0;
        let x = cx + (a.cos() * r as f64 * 2.1).round() as i64;
        let y = cy + (a.sin() * r as f64).round() as i64;
        g.put_alpha(x, y, '·', col, alpha);
    }
}
