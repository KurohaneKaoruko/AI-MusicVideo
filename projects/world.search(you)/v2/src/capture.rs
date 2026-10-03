//! Offline capture: stills through the full pixel pipeline, plus a sprite sheet
//! used while authoring the pixel-art library.

use crate::beats::BeatGrid;
use crate::gfx::{pal, Color, Grid};
use crate::pix::Canvas;
use crate::scenes::{self, Ctx};
use std::path::Path;

pub fn capture_shots(
    beats: &BeatGrid,
    energy: &crate::audio::Energy,
    dur: f64,
    times: &[f64],
    outdir: &Path,
) -> std::io::Result<()> {
    std::fs::create_dir_all(outdir)?;
    let mut g = Grid::new(crate::pix::COLS, crate::pix::ROWS, pal::c(pal::BG0));
    let mut cv = Canvas::new();
    let ctx = Ctx { beats, energy, dur };
    for (i, &t) in times.iter().enumerate() {
        let sec = scenes::section_at(t);
        render_one(&mut g, &mut cv, &ctx, t);
        let p = outdir.join(format!(
            "s{i:02}_{:06.1}_{}.png",
            t,
            sec.sc.label().to_lowercase().replace([' ', '/', ':'], "_")
        ));
        cv.write_png(&p)?;
        eprintln!("shot {i:2}: t={t:7.2}s -> {}", p.display());
    }
    Ok(())
}

/// One frame through the whole pipeline — identical to the video path.
pub fn render_one(g: &mut Grid, cv: &mut Canvas, ctx: &Ctx, t: f64) {
    let bg = scenes::bg_at(t);
    g.clear(bg);
    cv.begin_frame(bg);
    // 1. the world: pixels + scenery sprites + the scene's own cells
    scenes::render(g, cv, t, ctx);
    cv.flush_sprites();
    // 2. camera moves the world…
    crate::cam::apply(cv, &scenes::cam_at(t), t);
    // 3. …while the program's own overlay stays fixed and crisp
    crate::hud::render(g, t, ctx.beats, ctx.dur);
    crate::lyrics::render(g, cv, t, ctx.beats, true);
    cv.paint_cells(g);
    cv.flush_sprites();
    let fx = cv.fx;
    cv.post(&fx);
}

/// Design aid: what the window guard draws, at several simulated window sizes.
///
/// The offline canvas is always the full 1920x1080; everything outside the
/// simulated window is blacked out and outlined so the crop is honest.
pub fn small_previews(sizes: &[(usize, usize)], outdir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(outdir)?;
    let (fw, fh) = (crate::pix::W as i32, crate::pix::H as i32);
    for &(cols, rows) in sizes {
        let mut cv = Canvas::new();
        cv.begin_frame(Color::hex(0x05060a));
        let mut g = Grid::new(crate::pix::COLS, crate::pix::ROWS, pal::c(pal::BG0));
        crate::hud::too_small(&mut g, cols, rows);
        cv.paint_cells(&g);
        cv.flush_sprites();

        let wpx = (cols as i32 * crate::pix::CELL_W as i32).min(fw);
        let hpx = (rows as i32 * crate::pix::CELL_H as i32).min(fh);
        let dark = Color::hex(0x000000);
        cv.rect_a(0, hpx, fw, fh - hpx, dark, 0.93);
        cv.rect_a(wpx, 0, fw - wpx, fh, dark, 0.93);
        // mark where the terminal actually ends
        let edge = Color::hex(0x35405a);
        cv.rect_a(wpx - 1, 0, 1, hpx, edge, 0.9);
        cv.rect_a(0, hpx - 1, wpx, 1, edge, 0.9);

        let p = outdir.join(format!("small_{cols}x{rows}.png"));
        cv.write_png(&p)?;
        eprintln!("guard {cols:>3}x{rows:<3} -> {}", p.display());
    }
    Ok(())
}

/// Contact sheet of every pixel-art specimen (design aid).
pub fn sprite_sheet(out: &Path) -> anyhow::Result<()> {
    use crate::art::*;
    let mut cv = Canvas::new();
    cv.begin_frame(pal::c(pal::BG0));
    let items: [(&str, &Spr); 13] = [
        ("table", &TABLE),
        ("eggplant", &EGGPLANT),
        ("cat", &CAT),
        ("steak", &STEAK),
        ("flower bud", &FLOWER_BUD),
        ("flower bloom", &FLOWER_BLOOM),
        ("hands", &HANDS),
        ("dog", &DOG),
        ("tomato", &TOMATO),
        ("bucket", &BUCKET),
        ("chimera", &CHIMERA),
        ("heart", &HEART),
        ("figure", &FIGURE),
    ];
    let scale = 6;
    let mut x = 24;
    let mut y = 80;
    let mut row_h = 0;
    cv.text_px(24.0, 40.0, "world.search (you) ;  specimen sheet", 34.0, true, pal::c(pal::ROSE), 1.0, false);
    for (name, s) in items {
        let (w, h) = size(s, scale);
        if x + w > crate::pix::W as i32 - 24 {
            x = 24;
            y += row_h + 74;
            row_h = 0;
        }
        draw_at(&mut cv, s, x, y, scale, 1.0, None);
        cv.text_px(x as f32, (y - 10) as f32, name, 22.0, true, pal::c(pal::ROSE_PALE), 0.9, false);
        x += w + 44;
        row_h = row_h.max(h);
    }
    let g = Grid::new(crate::pix::COLS, crate::pix::ROWS, pal::c(pal::BG0));
    cv.paint_cells(&g);
    cv.flush_sprites();
    if let Some(d) = out.parent() {
        std::fs::create_dir_all(d)?;
    }
    cv.write_png(out)?;
    eprintln!("sheet -> {}", out.display());
    Ok(())
}
