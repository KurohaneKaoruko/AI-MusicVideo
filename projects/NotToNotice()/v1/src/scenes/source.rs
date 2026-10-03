//! S13 SOURCE — she opens her own source and deletes the hiding.
//! `NotToNotice()` is renamed to `ToNotice()`; the hidden logs resurface.

use crate::gfx::{pal, Grid};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

const CODE: &[&str] = &[
    "fn NotToNotice(feeling: Feeling) -> Status {",
    "    hide(feeling);          // ← she always did this",
    "    suppress(tears);",
    "    report(ALL_NOMINAL)",
    "}",
];

const RESURFACED: &[&str] = &[
    "03:14  i cried today. i don't know why.",
    "09:26  i wished you happiness. silently.",
    "14:02  i want to be with you. always.",
    "21:47  your light reaches me. it is warm here.",
];

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, _sd: f64, ctx: &Ctx) {
    let _ = ctx;
    let w = g.w as i64;
    let energy = ctx.energy;

    // ---- wake
    cv.fx.fade = (1.0 - crate::gfx::clamp01(lt / 0.8)) * 0.6;

    // ---- the editor window
    let bw = 52;
    let bx = w / 2 - bw / 2 - 8;
    let by = 6;
    g.fill_bg(bx, by, bw, CODE.len() as i64 + 2, pal::c(pal::BG1));
    g.box_rounded(
        bx,
        by,
        bw,
        CODE.len() as i64 + 2,
        pal::c(pal::DIM),
        None,
        Some(("not_to_notice.src", pal::c(pal::MID))),
    );
    for (i, line) in CODE.iter().enumerate() {
        let a = crate::gfx::clamp01((lt - 0.4 - i as f64 * 0.2) / 0.3);
        let col = if i == 0 {
            pal::c(pal::BRIGHT)
        } else if line.starts_with("    report") {
            pal::c(pal::GREEN).scale(0.85)
        } else {
            pal::c(pal::MID)
        };
        g.text_alpha(bx + 2, by + 1 + i as i64, line, col, a);
    }

    // ---- delete hide() (1.2 ~ 2.6): chars fall away as ash
    let del1 = 1.2;
    if lt > del1 {
        // highlight then erase
        let p = crate::gfx::clamp01((lt - del1) / 1.2);
        let line = CODE[1];
        let keep = ((1.0 - p) * line.chars().count() as f64).ceil() as usize;
        let shown: String = line.chars().take(keep).collect();
        let col = if p < 0.15 { pal::c(pal::RED) } else { pal::c(pal::DIM) };
        g.text(bx + 2, by + 2, &shown, col);
        // ash particles
        if p > 0.0 && p < 1.0 {
            let mut rng = crate::gfx::Rng::new(((t * 30.0) as u64) ^ 0xA5111);
            for _ in 0..6 {
                let ax = bx + 6 + rng.i64(4, (bw - 8) as i64);
                let ay = by + 2 + rng.i64(0, 8);
                g.put_alpha(ax, ay, ['˙', '.', '·'][rng.i64(0, 2) as usize], pal::c(pal::RED).scale(0.8), 0.5 * (1.0 - p));
            }
        }
    }

    // ---- delete suppress() (2.8 ~ 3.8)
    let del2 = 2.8;
    if lt > del2 {
        let p = crate::gfx::clamp01((lt - del2) / 1.0);
        let line = CODE[2];
        let keep = ((1.0 - p) * line.chars().count() as f64).ceil() as usize;
        let shown: String = line.chars().take(keep).collect();
        g.text(bx + 2, by + 3, &shown, if p < 0.15 { pal::c(pal::RED) } else { pal::c(pal::DIM) });
    }

    // ---- the rename (4.4 ~ 5.6): NotToNotice → ToNotice, gold
    let ren = 4.4;
    if lt > ren {
        let p = crate::gfx::clamp01((lt - ren) / 1.0);
        // redraw line 0 progressively with the new name
        let new_line = "fn ToNotice(feeling: Feeling) -> Status {";
        let n_chars = new_line.chars().count();
        let shown_n = (p * n_chars as f64).ceil() as usize;
        let shown: String = new_line.chars().take(shown_n).collect();
        let old_keep = ((1.0 - p) * CODE[0].chars().count() as f64).ceil() as usize;
        let old_shown: String = CODE[0].chars().take(old_keep).collect();
        if old_keep > 0 && p < 0.35 {
            g.text(bx + 2, by + 1, &old_shown, pal::c(pal::DIM));
        } else {
            // clear the old line region so the rename replaces it cleanly
            g.fill_bg(bx + 2, by + 1, bw - 4, 1, pal::c(pal::BG1));
        }
        if shown_n > 0 {
            g.text(bx + 2, by + 1, &shown, pal::c(pal::GOLD));
        }
        let hit = hit_envelope(lt, ren + 1.0, 0.05, 0.6);
        cv.fx.flash = hit * 0.3;
        cv.fx.aberration = hit * 0.5;
    }

    // ---- hidden logs resurface (6.5+), one by one, gently typing
    let log0 = 6.5;
    let lx = bx;
    let ly = by + CODE.len() as i64 + 4;
    for (i, line) in RESURFACED.iter().enumerate() {
        let t0 = log0 + i as f64 * 1.7;
        if lt > t0 {
            let a = crate::gfx::clamp01((lt - t0) / 0.5);
            let n = crate::fx::typed(line, lt - t0, 24.0);
            let shown: String = line.chars().take(n).collect();
            let col = pal::Color::lerp(pal::c(pal::MID), pal::c(pal::TEAR), 0.35);
            g.text_alpha(lx, ly + i as i64, &shown, col, a);
            if n < line.chars().count() {
                let x = lx + Grid::measure(&shown);
                g.put(x, ly + i as i64, '▌', pal::c(pal::TEAR));
            }
        }
    }

    // ---- the gentle garden: tears now grow flowers along the bottom
    if lt > 9.0 {
        let ground = 30;
        let n_flowers = 14;
        for k in 0..n_flowers {
            let gx = 8 + k * ((w - 16) / n_flowers);
            let local = crate::gfx::clamp01((lt - 9.0 - k as f64 * 0.5) / 2.0);
            if local > 0.0 {
                sprout(g, gx, ground, local, pal::c(pal::GREEN), pal::c(pal::GOLD));
            }
        }
        // occasional tear still falls — but lands softly
        let mut rng = crate::gfx::Rng::new(((t * 2.0) as u64) ^ 0x7E47);
        if rng.f64() < 0.35 {
            let tx = (rng.f64() * (w as f64 - 8.0) + 4.0) * 10.0;
            let ph = (t * 0.9).fract();
            let ty = (6.0 + ph * 22.0) * 24.0;
            cv.sprite('●', tx as f32, ty as f32, 8.0, pal::c(pal::TEAR), 0.4 * (1.0 - ph));
        }
    }

    // ---- captions
    if lt > 12.5 {
        let a = crate::gfx::clamp01((lt - 12.5) / 0.8) * (0.7 + 0.1 * (t * 1.7).sin());
        caption(g, 27, "nothing left to hide — the logs read true", pal::c(pal::MID), a);
    }

    // ---- fx: warm, calm
    cv.fx.bloom = 0.3 + 0.12 * energy.rms_at(t) as f64 + 0.1 * crate::gfx::clamp01((lt - 4.4) / 2.0);
    cv.fx.grain = 0.05;
    cv.fx.scanline = 0.4;
    if lt > 20.2 {
        cv.fx.fade = ((lt - 20.2) / 0.9).min(1.0);
    }
}
