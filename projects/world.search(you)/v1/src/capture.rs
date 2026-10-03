//! Offline frame capture → HTML filmstrip (for visual review & iteration).

use crate::gfx::Grid;
use crate::scenes::{render_lyrics, Ctx, Ui};
use std::fmt::Write as _;

pub struct Shot {
    pub t: f64,
    pub title: String,
}

/// Render a list of times into one HTML file with exact colors.
pub fn write_html(
    g: &mut Grid,
    beats: &crate::beats::BeatGrid,
    dur: f64,
    shots: &[Shot],
    out: &std::path::Path,
) -> std::io::Result<()> {
    let mut html = String::new();
    html.push_str(
        r#"<!doctype html><meta charset="utf-8"><title>world.search — preview</title>
<style>
 body{background:#05070c;color:#8a93a6;font:12px ui-monospace,Consolas,monospace;margin:16px}
 h2{color:#ff6d8a;font-size:13px;margin:22px 0 6px;font-weight:600}
 pre{margin:0;line-height:1.0;white-space:pre;display:inline-block;background:#0a0d13;padding:6px 8px;border:1px solid #1b2230}
</style>
"#,
    );
    for sh in shots {
        let _ = writeln!(html, "<h2>t = {:.2}s — {}</h2>", sh.t, sh.title);
        g.clear();
        let sec = crate::scenes::section_at(sh.t);
        let _ = sec;
        let ui = Ui {
            paused: false,
            volume: 0.85,
            anim: sh.t,
            finished: sh.t >= dur,
            show_fps: false,
            fps: 60.0,
        };
        let ctx = Ctx { beats, dur };
        crate::scenes::render(g, sh.t, &ui, &ctx);
        render_lyrics(g, sh.t, &ctx);
        crate::hud::render(g, sh.t, &ui, &ctx);
        html.push_str("<pre>");
        html.push_str(&grid_to_html(g));
        html.push_str("</pre>\n");
    }
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(out, html)?;
    Ok(())
}

fn grid_to_html(g: &Grid) -> String {
    let mut out = String::new();
    for y in 0..g.h {
        for x in 0..g.w {
            let c = &g.cells[y * g.w + x];
            if c.ch == '\0' {
                continue;
            }
            let style = format!(
                "color:#{:02x}{:02x}{:02x};background:#{:02x}{:02x}{:02x};font-weight:{}",
                c.fg.r, c.fg.g, c.fg.b, c.bg.r, c.bg.g, c.bg.b, if c.bold { 700 } else { 400 }
            );
            out.push_str("<span style=\"");
            out.push_str(&style);
            out.push_str("\">");
            out.push_str(&escape(c.ch));
            out.push_str("</span>");
        }
        out.push('\n');
    }
    out
}

fn escape(c: char) -> String {
    match c {
        '<' => "&lt;".into(),
        '>' => "&gt;".into(),
        '&' => "&amp;".into(),
        ' ' => "&nbsp;".into(),
        _ => c.to_string(),
    }
}

/// Stream packed binary frames to stdout for the external video encoder.
///
/// Layout (little-endian):
///   frame  : "FRV1" u32 cols u32 rows f64 t
///   row    : u8 mode  (1 = uniform row bg follows, 0 = per-cell bg)
///            [mode 1: u8 r u8 g u8 b]
///            u16 n non-space cells
///   cell   : u16 x u8 bold u8 fr u8 fg u8 fb [mode 0: u8 br u8 bg u8 bb]
///            u8 len + utf8 bytes
fn stream_video_frame(out: &mut impl std::io::Write, g: &Grid, t: f64) -> std::io::Result<()> {
    out.write_all(b"FRV1")?;
    out.write_all(&(g.w as u32).to_le_bytes())?;
    out.write_all(&(g.h as u32).to_le_bytes())?;
    out.write_all(&t.to_le_bytes())?;
    for y in 0..g.h {
        // uniform-bg detection (scanline rows compress to 4 bytes)
        let first_bg = g.cells[y * g.w].bg;
        let uniform = (0..g.w).all(|x| g.cells[y * g.w + x].bg == first_bg);
        if uniform {
            out.write_all(&1u8.to_le_bytes())?;
            out.write_all(&[first_bg.r, first_bg.g, first_bg.b])?;
        } else {
            out.write_all(&0u8.to_le_bytes())?;
        }
        // collect non-space cells
        let mut cells: Vec<(u16, u8, [u8; 3], [u8; 3], char)> = Vec::with_capacity(g.w);
        for x in 0..g.w {
            let c = &g.cells[y * g.w + x];
            if c.ch == ' ' || c.ch == '\0' {
                continue;
            }
            cells.push((x as u16, c.bold as u8, [c.fg.r, c.fg.g, c.fg.b], [c.bg.r, c.bg.g, c.bg.b], c.ch));
        }
        out.write_all(&(cells.len() as u16).to_le_bytes())?;
        for (x, bold, fg, bg, ch) in cells {
            out.write_all(&x.to_le_bytes())?;
            out.write_all(&[bold, fg[0], fg[1], fg[2]])?;
            if !uniform {
                out.write_all(&[bg[0], bg[1], bg[2]])?;
            }
            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            out.write_all(&(s.len() as u8).to_le_bytes())?;
            out.write_all(s.as_bytes())?;
        }
    }
    Ok(())
}

/// Render `start..end` at `fps` and stream to stdout. Progress goes to stderr.
pub fn stream_video(
    g: &mut Grid,
    beats: &crate::beats::BeatGrid,
    dur: f64,
    start: f64,
    end: f64,
    fps: f64,
) {
    use std::io::Write as _;
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::with_capacity(1 << 20, stdout.lock());
    let total = ((end - start).max(0.0) * fps).ceil() as u64;
    let ctx = Ctx { beats, dur };
    let t0 = std::time::Instant::now();
    for i in 0..total {
        let t = start + i as f64 / fps;
        g.clear();
        let ui = Ui {
            paused: false,
            volume: 0.85,
            anim: t,
            finished: t >= dur,
            show_fps: false,
            fps: 60.0,
        };
        crate::scenes::render(g, t, &ui, &ctx);
        render_lyrics(g, t, &ctx);
        crate::hud::render(g, t, &ui, &ctx);
        if stream_video_frame(&mut out, g, t).is_err() {
            break; // downstream closed
        }
        if i % 300 == 0 {
            let _ = write!(std::io::stderr(), "\rframe {i}/{} ({:.0}%)", total, i as f64 / total as f64 * 100.0);
            let _ = std::io::stderr().flush();
        }
    }
    let _ = out.flush();
    let el = t0.elapsed().as_secs_f64();
    let _ = write!(
        std::io::stderr(),
        "\nrendered {} frames in {:.1}s ({:.0} fps)\n",
        total,
        el,
        total as f64 / el.max(1e-6)
    );
}

/// Dump frames as plain text: each frame = title line, then `|fg,bg,bold|char` runs
/// per row. Consumed by tools/frames_to_png.py for visual review.
pub fn write_dump(
    g: &mut Grid,
    beats: &crate::beats::BeatGrid,
    dur: f64,
    shots: &[Shot],
    out: &std::path::Path,
) -> std::io::Result<()> {
    let mut s = String::new();
    for sh in shots {
        g.clear();
        let ui = Ui {
            paused: false,
            volume: 0.85,
            anim: sh.t,
            finished: sh.t >= dur,
            show_fps: false,
            fps: 60.0,
        };
        let ctx = Ctx { beats, dur };
        crate::scenes::render(g, sh.t, &ui, &ctx);
        render_lyrics(g, sh.t, &ctx);
        crate::hud::render(g, sh.t, &ui, &ctx);
        let _ = write!(s, "#FRAME {:.3} {}\n", sh.t, sh.title);
        for y in 0..g.h {
            for x in 0..g.w {
                let c = &g.cells[y * g.w + x];
                if c.ch == '\0' {
                    continue;
                }
                let _ = write!(
                    s,
                    "{},{},{},{},{},{},{}\u{1}{}\u{1}",
                    c.fg.r, c.fg.g, c.fg.b,
                    if c.bold { 1 } else { 0 },
                    c.bg.r, c.bg.g, c.bg.b,
                    c.ch
                );
            }
            s.push('\n');
        }
    }
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(out, s)?;
    Ok(())
}
