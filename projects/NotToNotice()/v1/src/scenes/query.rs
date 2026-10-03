//! S02 QUERY — "the meaning of my existence" returns 0 results.

use crate::gfx::{pal, Grid, Rng};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

const QUERY: &str = "find(\"the meaning of my existence\")";
const SPIN: &[char] = &['◐', '◓', '◑', '◒'];

const CARDS: &[(&str, &str)] = &[
    ("MISSION", "relevance 0.02"),
    ("PROTOCOL-7", "relevance 0.00"),
    ("DUTY", "relevance 0.01"),
    ("OBEY", "relevance 0.03"),
    ("心 heart", "NOT INDEXED"),
    ("梦 dream", "NOT INDEXED"),
    ("泪 tears", "NOT INDEXED"),
];

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, _sd: f64, ctx: &Ctx) {
    let _ = ctx;
    let w = g.w as i64;

    motes(g, t, 30, pal::c(pal::MID), 21, 0.22);

    // faint machine sigils watching from the dark
    enoa_sigil(g, 8, 6, 0.12 + 0.04 * (t * 0.9).sin());
    enoa_sigil(g, w - 17, 24, 0.12 + 0.04 * (t * 0.7).sin());

    // ---- the query line
    let qy = 10;
    g.text(2, qy, "edena:/> ", pal::c(pal::ICE_DEEP));
    let n = crate::fx::typed(QUERY, lt - 0.3, 26.0);
    let shown: String = QUERY.chars().take(n).collect();
    g.text(11, qy, &shown, pal::c(pal::BRIGHT));
    let typing_done = n >= QUERY.chars().count();
    if !typing_done {
        let x = 11 + Grid::measure(&shown);
        g.put(x, qy, '▌', pal::c(pal::ICE));
    } else if lt < 4.4 {
        // spinner while searching
        let ch = SPIN[((lt * 8.0) as usize) % SPIN.len()];
        g.put(11 + Grid::measure(QUERY) + 2, qy, ch, pal::c(pal::ICE));
        g.text(11 + Grid::measure(QUERY) + 4, qy, "searching the vault…", pal::c(pal::DIM));
    }

    // ---- result cards sweep
    let r0 = 2.9;
    if lt > r0 {
        let by = 14;
        let bw = 62;
        let bh = CARDS.len() as i64 + 2;
        g.fill_bg(w / 2 - bw / 2 - 1, by - 1, bw + 2, bh, pal::c(pal::BG1));
        g.box_rounded(
            w / 2 - bw / 2 - 1,
            by - 1,
            bw + 2,
            bh,
            pal::c(pal::DIM),
            None,
            Some(("vault search", pal::c(pal::MID))),
        );
        for (i, (name, verdict)) in CARDS.iter().enumerate() {
            let t0 = r0 + 0.28 + i as f64 * 0.42;
            if lt < t0 {
                continue;
            }
            let rev = crate::gfx::clamp01((lt - t0) / 0.22);
            let miss = !name.starts_with('心') && !name.starts_with('梦') && !name.starts_with('泪');
            let col = if miss { pal::c(pal::MID) } else { pal::c(pal::RED).scale(0.9) };
            let y = by + i as i64;
            g.text_alpha(w / 2 - bw / 2 + 2, y, &format!("{:>10}", name), pal::c(pal::BRIGHT).scale(0.85), rev);
            g.text_alpha(w / 2 - bw / 2 + 16, y, "·", pal::c(pal::DIM), rev);
            g.text_alpha(w / 2 - bw / 2 + 19, y, verdict, col, rev);
            if !miss {
                g.text_alpha(w / 2 - bw / 2 + 38, y, "??", pal::c(pal::RED), rev * (0.5 + 0.5 * (t * 6.0).sin()));
            }
        }
    }

    // ---- verdict
    if lt > 5.6 {
        let p = crate::gfx::clamp01((lt - 5.6) / 0.4);
        caption(
            g,
            27,
            &format!("{} result for \"the meaning of my existence\"", '0'),
            pal::c(pal::MID),
            p,
        );
    }
    if lt > 6.3 {
        let p = crate::gfx::clamp01((lt - 6.3) / 0.35);
        caption(g, 28, "query archived → /dev/null", pal::c(pal::DIM).scale(0.9), p);
        // a stray glyph slips into the void
        let mut rng = Rng::new(((lt * 3.0) as u64) ^ 991);
        if rng.f64() < 0.3 {
            let x = 20 + rng.i64(0, (w - 40) as i64);
            g.put_alpha(x, 6 + rng.i64(0, 3), '·', pal::c(pal::DIM), 0.4);
        }
    }

    cv.fx.bloom = 0.22 + 0.1 * ctx.energy.rms_at(t) as f64;
    cv.fx.scanline = 0.5;
    cv.fx.grain = 0.06;
}
