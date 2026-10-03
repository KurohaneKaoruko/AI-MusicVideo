//! S00 BOOT — black screen, the machine goddess's OS wakes up, title stamps in.

use crate::art;
use crate::gfx::{pal, Grid};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

const BOOT_LOG: &[&str] = &[
    "DEUS EX MACHINA OS  v8.2",
    "seeding structure \"EDEN\" — dyson lattice / G2V star",
    "",
    "> boot kernel .............. OK",
    "> soul forge ............... IDLE",
    "> personality vault ........ 8,214,006 lives simulated",
    "> observer link ............ SYNCED",
    "> emotion daemon ........... [not loaded]",
];

const MACHINES: &[(&str, &str, &str)] = &[
    ("[01] PROMETOR ", "神機統括", "LOST"),
    ("[02] ECCLESIA ", "秩序維持", "IDLE"),
    ("[03] NOEIN    ", "機密事項", "SEALED"),
    ("[04] ANTHROPOS", "人間定義", "WATCHING"),
    ("[05] RETIA    ", "観測収集", "SYNCED"),
    ("[06] LOGOS    ", "文明発展", "HALTED"),
    ("[07] ZOE      ", "身体再生", "STANDBY"),
    ("[08] ENOA     ", "精神再生", "◄ ACTIVE"),
];

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, _sd: f64, _ctx: &Ctx) {
    let w = g.w as i64;

    // ---- phase 1: boot log (0 ~ 5.5)
    let mut y = 5;
    for (i, line) in BOOT_LOG.iter().enumerate() {
        let t0 = 0.9 + i as f64 * 0.55;
        if lt > t0 {
            let col = if line.starts_with('>') {
                if line.ends_with("OK") || line.ends_with("SYNCED") {
                    pal::c(pal::GREEN).scale(0.8)
                } else if line.contains("emotion") {
                    pal::c(pal::DIM)
                } else {
                    pal::c(pal::MID)
                }
            } else if i == 0 {
                pal::c(pal::BRIGHT)
            } else {
                pal::c(pal::DIM)
            };
            let shown = lt - t0;
            let n = crate::fx::typed(line, shown, 60.0);
            let s: String = line.chars().take(n).collect();
            let cur = n < line.chars().count();
            g.text(4, y, &s, col);
            if cur && (shown * 3.0).fract() < 0.6 {
                let x = 4 + Grid::measure(&s);
                g.put(x, y, '▌', pal::c(pal::ICE));
            }
        }
        y += 1;
    }

    // ---- phase 2: the eight divine machines (3.2 ~ 8.2)
    let mt0 = 3.6;
    let rows_y = 15;
    if lt > mt0 {
        // panel frame
        let ph = MACHINES.len() as i64 + 2;
        g.box_rounded(
            w - 62,
            rows_y - 1,
            56,
            ph,
            pal::c(pal::DIM),
            Some(pal::c(pal::BG1)),
            Some(("DEI EX MACHINA — 8 UNITS", pal::c(pal::MID))),
        );
        for (i, (name, duty, status)) in MACHINES.iter().enumerate() {
            let t0 = mt0 + i as f64 * 0.42;
            if lt < t0 {
                continue;
            }
            let active = i == 7;
            let status_col = match *status {
                "LOST" | "HALTED" | "SEALED" => pal::c(pal::RED).scale(0.85),
                "◄ ACTIVE" => pal::c(pal::ICE).glow(0.3 + 0.2 * (t * 2.0).sin()),
                _ => pal::c(pal::MID),
            };
            let name_col = if active {
                pal::c(pal::ICE)
            } else {
                pal::c(pal::BRIGHT).scale(0.8)
            };
            let rev = crate::gfx::clamp01((lt - t0) / 0.25);
            let x0 = w - 60;
            let yy = rows_y + i as i64;
            g.text_alpha(x0, yy, name, name_col, rev);
            g.text_alpha(x0 + 14, yy, duty, if active { pal::c(pal::ICE_PALE) } else { pal::c(pal::MID) }, rev);
            g.text_alpha(x0 + 24, yy, status, status_col, rev);
        }
    }

    // ---- phase 3: credits (5.5 ~ end, dim, bottom-left)
    if lt > 5.2 {
        for (i, (en, _zh)) in crate::lyrics::CREDITS.iter().enumerate() {
            let t0 = 5.2 + i as f64 * 0.5;
            if lt > t0 {
                g.text_alpha(
                    4,
                    32 + i as i64,
                    en,
                    pal::c(pal::MID).scale(0.8),
                    crate::gfx::clamp01((lt - t0) / 0.4) * 0.9,
                );
            }
        }
    }

    // ---- phase 4: title stamp (8.3 ~ end)
    let st0 = 8.35;
    if lt >= st0 {
        // wipe the stage: the title takes over
        for yy in STAGE_Y0..STAGE_Y1 {
            for xx in 0..w {
                let i2 = yy as usize * g.w + xx as usize;
                let c = &mut g.cells[i2];
                c.ch = ' ';
                c.fg = g.bg;
                c.bg = g.bg;
            }
        }
        let p = crate::gfx::clamp01((lt - st0) / 0.5);
        let e = crate::gfx::ease_out_back(p);
        let big = 1.0 - (1.0 - e) * 0.5;
        // scale-in via alpha + size steps on the pixel font
        let title = "NotToNotice();";
        let tw = art::big_text_width(title, 2);
        let x = (w - tw) / 2;
        let y0 = 11 + ((1.0 - big) * 4.0) as i64;
        // shadow pass
        art::big_text(g, x + 1, y0 + 1, title, pal::c(pal::ICE_DEEP).scale(0.5), 2, 0.5 * p);
        art::big_text(g, x, y0, title, pal::c(pal::WHITE), 2, p);

        // subtitle (below the 10-row title block)
        if lt > st0 + 0.5 {
            caption(g, y0 + 12, "— 第八神機エノア · 精神再生担当 —", pal::c(pal::ICE).scale(0.9), p * 0.95);
        }
        if lt > st0 + 0.9 {
            caption(g, y0 + 14, "a fan-made terminal tribute to CRYMACHINA", pal::c(pal::DIM), p * 0.8);
        }
        // impact flash + glitch tick
        let hit = hit_envelope(lt, st0, 0.05, 0.5);
        cv.fx.flash = hit * 0.5;
        cv.fx.bloom = 0.4 + hit * 0.5;
        if lt - st0 < 0.12 {
            cv.fx.glitch = 0.5 * (1.0 - (lt - st0) / 0.12);
            cv.fx.aberration = 0.7 * (1.0 - (lt - st0) / 0.12);
        }
    } else {
        // faint dust while booting
        motes(g, t, 24, pal::c(pal::MID), 77, 0.25);
    }

    // fade out into the fairy scene
    if lt > _sd - 0.8 {
        cv.fx.fade = ((lt - (_sd - 0.8)) / 0.8).min(1.0) * 0.9;
    }
    cv.fx.scanline = 0.55;
    cv.fx.grain = 0.07;
}
