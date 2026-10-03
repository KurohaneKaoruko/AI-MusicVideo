//! Offline capture: render shots through the full pixel pipeline to PNG files.

use crate::beats::BeatGrid;
use crate::gfx::{Color, Grid};
use crate::pix::Canvas;
use crate::scenes::{self, Ctx};
use std::path::Path;

pub fn capture_shots(beats: &BeatGrid, energy: &crate::audio::Energy, dur: f64, times: &[f64], outdir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(outdir)?;
    let mut g = Grid::new(crate::pix::COLS, crate::pix::ROWS, Color::hex(crate::gfx::pal::BG0));
    let mut cv = Canvas::new();
    let ctx = Ctx { beats, energy, dur };
    for (i, &t) in times.iter().enumerate() {
        let sec = scenes::section_at(t);
        render_one(&mut g, &mut cv, &ctx, t);
        let p = outdir.join(format!("t{:07.2}_{}_{}s.png", t, sec.label.to_lowercase().replace(' ', "_"), i));
        cv.write_png(&p)?;
        eprintln!("shot {:2}: t={:7.2}s -> {}", i, t, p.display());
    }
    Ok(())
}

/// One frame through the whole pipeline (identical to the video path).
pub fn render_one(g: &mut Grid, cv: &mut Canvas, ctx: &Ctx, t: f64) {
    let bg = scenes::bg_at(t);
    g.clear(bg);
    cv.begin_frame(bg);
    scenes::render(g, cv, t, ctx);
    crate::lyrics::render(g, cv, t, ctx.beats);
    crate::hud::render(g, t, ctx.beats, ctx.dur);
    cv.paint_cells(g);
    cv.flush_sprites();
    let fx = cv.fx;
    cv.post(&fx);
}
