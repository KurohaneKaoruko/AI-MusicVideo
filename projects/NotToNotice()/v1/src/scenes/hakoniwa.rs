//! S06 HAKONIWA — the virtual world: a living globe Enoa tends, season to season,
//! searching simulated lives for humanlike souls (E.V.E).

use crate::gfx::{hash01, pal, Color, Grid, Rng};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

/// sphere land/sea cell render
fn globe(g: &mut Grid, cx: i64, cy: i64, r: i64, spin: f64, season: usize, t: f64) {
    let rx = r * 2; // terminal cells are ~2:1, so stretch x for a round look
    for dy in -r..=r {
        for dx in -rx..=rx {
            let nx = dx as f64 / rx as f64;
            let ny = dy as f64 / r as f64;
            let d2 = nx * nx + ny * ny;
            if d2 > 1.0 {
                continue;
            }
            let z = (1.0 - d2).sqrt();
            // limb darkening
            let shade = 0.35 + 0.65 * z;
            // sphere coords
            let lat = ny.asin(); // -pi/2..pi/2
            let lon = nx.atan2(z) + spin;
            // land blobs via hashed lat/lon lattice
            let la = (lat * 3.2).floor() as i64;
            let lo = (lon * 3.2).floor() as i64;
            let land = hash01(((la as u64) << 20) ^ (lo as u64).wrapping_mul(0x9E37)) > 0.52;
            let x = cx + dx;
            let y = cy + dy;
            if land {
                let (col, ch) = match season {
                    0 => (Color::rgb(94, 190, 130), '#'),  // spring
                    1 => (Color::rgb(210, 180, 84), '#'),  // summer
                    2 => (Color::rgb(206, 120, 84), '#'),  // autumn
                    _ => (Color::rgb(158, 196, 216), '#'), // winter
                };
                let c = col.scale(shade);
                g.put(x, y, ch, c);
            } else {
                let sea = pal::c(pal::ICE_DEEP).scale(0.5 * shade);
                let ch = if z > 0.93 { '·' } else { '.' };
                g.put(x, y, ch, sea);
            }
            // ice caps
            if lat.abs() > 1.12 {
                g.put(x, y, if (x + y) % 3 == 0 { '*' } else { '·' }, pal::c(pal::ICE_PALE).scale(shade));
            }
        }
    }
    // a tiny observer satellite dot orbiting
    let ang = t * 0.9;
    let ox = cx + (ang.cos() * rx as f64 * 1.18).round() as i64;
    let oy = cy + (ang.sin() * r as f64 * 1.18 * 0.8).round() as i64;
    g.put(ox, oy, '·', pal::c(pal::MID));
}

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, _sd: f64, ctx: &Ctx) {
    let w = g.w as i64;
    let energy = ctx.energy;

    // ---- seasons cycle across the scene
    let season_f = lt / 2.45;
    let season = (season_f as usize) % 4;
    let spin = t * 0.22;

    // ---- seasonal ambient particles
    let rng = Rng::new(((t * 6.0) as u64) ^ 0x51E5);
    match season {
        0 => {
            // spring petals (pixel sprites drifting)
            for i in 0..14 {
                let s0 = hash01(i as u64 * 31 + 7);
                let s1 = hash01(i as u64 * 63 + 11);
                let x = ((s0 * 1.3 * W_PX as f64 - t * 26.0 * (0.5 + s1)).rem_euclid(W_PX as f64 + 80.0)) - 40.0;
                let y = (s1 * STAGE_H_PX as f64 + (t * 30.0 * (0.4 + s0)).rem_euclid(STAGE_H_PX as f64 + 40.0)) - 20.0;
                cv.sprite('*', x as f32, y as f32, 9.0 + 5.0 * s0 as f32, Color::rgb(244, 170, 196), 0.6);
            }
        }
        1 => {
            motes(g, t, 30, Color::rgb(240, 220, 140), 3, 0.3);
        }
        2 => {
            // autumn leaves
            for i in 0..12 {
                let s0 = hash01(i as u64 * 41 + 3);
                let s1 = hash01(i as u64 * 87 + 5);
                let x = ((s0 * 1.2 * W_PX as f64 + t * 34.0 * (0.5 + s1)).rem_euclid(W_PX as f64 + 80.0)) - 40.0;
                let y = (s1 * STAGE_H_PX as f64 + (t * 24.0).rem_euclid(STAGE_H_PX as f64 + 40.0)) - 20.0;
                cv.sprite('*', x as f32, y as f32, 10.0 + 5.0 * s0 as f32, Color::rgb(226, 130, 84), 0.6);
            }
        }
        _ => {
            // winter snow
            motes(g, t, 44, pal::c(pal::ICE_PALE), 9, 0.4);
        }
    }
    let _ = rng;

    // ---- the globe
    let gcx = 46;
    let gcy = 18;
    let gr = 11;
    // fade in
    let p0 = crate::gfx::clamp01(lt / 0.7);
    if p0 > 0.0 {
        // dim the area behind
        globe(g, gcx, gcy, gr, spin, season, t);
        // stand line
        for k in 0..7 {
            g.put(gcx - 3 + k * 1, gcy + gr + 2 + (k as f64 * 0.4).sin().abs() as i64, if k % 2 == 0 { '─' } else { '╌' }, pal::c(pal::DIM));
        }
        g.put(gcx, gcy + gr + 1, '|', pal::c(pal::DIM));
    }

    // ---- simulation panel
    let bx = w - 66;
    let by = 6;
    let panel = [
        ("SIMULATION · 仮想世界", pal::MID),
        ("iteration  8,214,006+", pal::BRIGHT),
        ("seeds      2,003,111 live", pal::MID),
        ("season     spring → winter", pal::DIM),
    ];
    for (i, (txt, col)) in panel.iter().enumerate() {
        let a = crate::gfx::clamp01((lt - 0.6 - i as f64 * 0.3) / 0.3);
        if a > 0.0 {
            g.text_alpha(bx, by + i as i64, txt, pal::c(*col), a);
        }
    }

    // iteration ticker
    let it = 8_214_006 + ((t * 137.0) as u64);
    g.text(bx + 11, by + 1, &format!("{:>10}", it), pal::c(pal::BRIGHT));

    // ---- E.V.E candidates found (7.2+)
    let found_t = 7.2;
    if lt > found_t {
        let p = crate::gfx::clamp01((lt - found_t) / 0.5);
        g.text_alpha(bx, by + 6, "humanlike souls found:", pal::c(pal::MID), p);
        let names = [("LEBEN · 人間はクソ", pal::RED), ("MIKOTO · かっこよく", pal::AMBER), ("AMI · 家族がいてこそ", pal::GREEN)];
        for (i, (name, col)) in names.iter().enumerate() {
            let t0 = found_t + 0.5 + i as f64 * 0.55;
            if lt > t0 {
                let q = crate::gfx::clamp01((lt - t0) / 0.35);
                g.text_alpha(bx + 2, by + 7 + i as i64, &format!("E.V.E №{}", i + 1), pal::c(pal::GOLD), q);
                g.text_alpha(bx + 14, by + 7 + i as i64, name, pal::c(*col), q);
                g.put_alpha(bx + 40, by + 7 + i as i64, '●', pal::c(*col), q * (0.6 + 0.4 * (t * 3.0 + i as f64).sin()));
            }
        }
        if lt > found_t + 2.4 {
            let q = crate::gfx::clamp01((lt - found_t - 2.4) / 0.5);
            caption(g, 28, "「あなたは、選ばれた。」", pal::c(pal::GOLD).glow(0.15 * (t * 2.0).sin().max(0.0)), q);
            caption(g, 29, "— you have been chosen —", pal::c(pal::DIM), q * 0.8);
        }
    }

    // ---- the question this answers
    if lt > 1.4 && lt < 5.4 {
        caption(g, 29, "her answer: give them life, again and again", pal::c(pal::MID), 0.6);
    }

    // ---- fx per season
    cv.fx.bloom = 0.3 + 0.12 * energy.rms_at(t) as f64 + match season { 1 => 0.08, _ => 0.0 };
    cv.fx.grain = 0.055;
    cv.fx.scanline = 0.4;
    if lt > 9.3 {
        cv.fx.fade = ((lt - 9.3) / 0.85).min(1.0);
    }
}
