//! world.search (you) ; — a terminal music video for Mili.
//!
//! The whole film is one run of a search program: it wakes, indexes the world,
//! opens every candidate for "you" (a table, an eggplant, a cat, a steak, a
//! flower, a human…), loses the word, searches everywhere, rewrites the query
//! into `my old self`, merges what is left, and finally answers itself.
//!
//! Rendering: a 192x45 character grid (10x24px cells = 1920x1080) painted as
//! real pixels (glyphs rasterised with fontdue) plus a free pixel layer and a
//! deterministic post chain.

// The rendering modules deliberately expose a superset of the primitives the
// current cut happens to use, so unused helpers are expected during authoring.
#![allow(dead_code)]

mod art;
mod audio;
mod beats;
mod cam;
mod capture;
mod env;
mod gfx;
mod hud;
mod lyrics;
mod pix;
mod scenes;
mod term;
#[cfg(windows)]
mod win_pipe;

use anyhow::{anyhow, Context, Result};
use gfx::{pal, Grid};
use pix::Canvas;
use scenes::Ctx;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};

const SONG: &str = "world.search (you) ;";

struct Args {
    audio: Option<PathBuf>,
    no_audio: bool,
    capture: Option<String>,
    video: Option<String>,
    out: Option<PathBuf>,
    seek: f64,
    fps_cap: f64,
    dev_beats: bool,
    dev_audio: bool,
    geometry: bool,
    bench: Option<usize>,
    sheet: bool,
    small: bool,
    help: bool,
}

fn parse_args() -> Result<Args> {
    let mut a = Args {
        audio: None,
        no_audio: false,
        capture: None,
        video: None,
        out: None,
        seek: 0.0,
        fps_cap: 0.0,
        dev_beats: false,
        dev_audio: false,
        geometry: false,
        bench: None,
        sheet: false,
        small: false,
        help: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--audio" | "-a" => a.audio = it.next().map(PathBuf::from),
            "--no-audio" => a.no_audio = true,
            "--capture" | "-c" => a.capture = it.next(),
            "--video" => a.video = it.next(),
            "--out" => a.out = it.next().map(PathBuf::from),
            "--seek" => a.seek = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0),
            "--fps" => a.fps_cap = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0),
            "--dev" => match it.next().as_deref() {
                Some("beats") => a.dev_beats = true,
                Some("audio") => a.dev_audio = true,
                Some("sheet") => a.sheet = true,
                Some("small") => a.small = true,
                other => return Err(anyhow!("unknown --dev target: {other:?}")),
            },
            "--geometry" => a.geometry = true,
            "--bench" => a.bench = it.next().and_then(|s| s.parse().ok()).or(Some(300)),
            "--help" | "-h" => a.help = true,
            other => return Err(anyhow!("unknown argument: {other} (try --help)")),
        }
    }
    Ok(a)
}

fn find_audio() -> Result<PathBuf> {
    let mut cands: Vec<PathBuf> = Vec::new();
    for dir in ["reference", "assets", ".", "../reference", "../assets", ".."] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) == Some("mp3") {
                    cands.push(p);
                }
            }
        }
    }
    cands.sort();
    cands
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no .mp3 found — put the song in reference/ or pass --audio <file.mp3>"))
}

fn help_text() -> String {
    format!(
        "{SONG} — a terminal MV for Mili\n\n\
     USAGE:\n  world-search-you [OPTIONS]\n\n\
     OPTIONS:\n\
     \x20 -a, --audio <FILE>   mp3 to sync with (default: any mp3 in reference/)\n\
     \x20     --no-audio        run on a wall clock (no sound)\n\
     \x20 -c, --capture <LIST>  render stills to .preview/*.png (full pixel pipeline)\n\
     \x20                       times: \"12,45.5,90\" | range \"a..b\" | grid \"a:b:step\"\n\
     \x20     --video <SPEC>    offline render \"start:end:fps\" (deterministic)\n\
     \x20     --out <FILE>      mp4 output path (ffmpeg via MV_FFMPEG or PATH)\n\
     \x20     --geometry        print the render geometry and exit\n\
     \x20     --seek <T>        start at T seconds (interactive)\n\
     \x20     --fps <N>         cap the frame rate, 0 = unlimited (interactive)\n\
     \x20     --bench <N>       render N full-pipeline frames, report timing\n\
     \x20     --dev beats|audio|sheet|small   self-tests / sprite sheet / small-window card\n\
     \x20 -h, --help            this text\n\n\
     The film is a fixed 192 x 45 character grid: the window must be at least\n\
     192 columns by 45 rows (maximise it, or shrink the font).\n\n\
     KEYS (interactive): space pause · ←/→ seek · +/- volume · f fps cap · h help · q quit\n"
    )
}

enum Loaded {
    Decoded(Arc<audio::Decoded>),
    Analysis(Arc<beats::BeatGrid>, Arc<audio::Energy>),
}

fn main() -> Result<()> {
    let args = parse_args()?;
    if args.help {
        print!("{}", help_text());
        return Ok(());
    }
    if args.geometry {
        println!("{} {} {} {} {} {}", pix::COLS, pix::ROWS, pix::CELL_W, pix::CELL_H, pix::W, pix::H);
        return Ok(());
    }
    if args.sheet {
        return capture::sprite_sheet(Path::new(".preview/sheet.png"));
    }
    if args.small {
        return capture::small_previews(
            &[(100, 30), (80, 24), (60, 20), (40, 12), (24, 6), (16, 4)],
            Path::new(".preview"),
        );
    }

    let audio_path = match &args.audio {
        Some(p) => p.clone(),
        None => find_audio()?,
    };
    if !audio_path.exists() {
        return Err(anyhow!("audio file not found: {}", audio_path.display()));
    }

    if args.dev_beats {
        let dec = audio::decode(&audio_path, |_| {})?;
        let mono = mono_mix(&dec);
        let g = beats::analyze(&mono, dec.rate, dec.duration, &lyrics::anchors());
        println!(
            "detected: {:.2} BPM · period {:.3}s · t0 {:.3}s · duration {:.1}s",
            g.bpm, g.period, g.t0, dec.duration
        );
        println!("{:>8}  {:>8}  {:>7}", "anchor", "nearest", "d(ms)");
        let mut worst = 0.0f64;
        for &a in lyrics::anchors().iter().step_by(3) {
            let k = ((a - g.t0) / g.period).round();
            let near = g.beat_time(k as i64);
            let d = (a - near).abs() * 1000.0;
            worst = worst.max(d);
            println!("{:8.2}  {:8.2}  {:7.1}", a, near, d);
        }
        println!("worst offset: {:.0} ms", worst);
        return Ok(());
    }

    if args.dev_audio {
        let dec = audio::decode(&audio_path, |_| {})?;
        let out = audio::AudioOut::start(&dec)?;
        std::thread::sleep(std::time::Duration::from_millis(1500));
        println!(
            "audio ok: clock at {:.2}s after 1.5s · {}ch {}Hz · {:.1}s total",
            out.time(),
            dec.channels,
            dec.rate,
            dec.duration
        );
        return Ok(());
    }

    if let Some(spec) = &args.video {
        return run_video(&args, &audio_path, spec);
    }

    if let Some(spec) = &args.capture {
        return run_capture(&audio_path, spec);
    }

    if let Some(n) = args.bench {
        let dec = audio::decode(&audio_path, |_| {})?;
        let mono = mono_mix(&dec);
        let grid = beat_grid(&audio_path, &mono, dec.rate, dec.duration);
        let energy = audio::energy(&mono, dec.rate);
        let ctx = Ctx { beats: &grid, energy: &energy, dur: dec.duration };
        let mut g = Grid::new(pix::COLS, pix::ROWS, pal::c(pal::BG0));
        let mut cv = Canvas::new();
        // warm the glyph cache
        capture::render_one(&mut g, &mut cv, &ctx, 0.0);
        let t0 = std::time::Instant::now();
        for i in 0..n {
            let t = (i as f64 / n as f64) * dec.duration;
            capture::render_one(&mut g, &mut cv, &ctx, t);
        }
        let el = t0.elapsed().as_secs_f64();
        println!(
            "bench: {} full-pipeline frames in {:.2}s -> {:.1} fps ({:.2} ms/frame)",
            n,
            el,
            n as f64 / el,
            el * 1000.0 / n as f64
        );
        // The player's frame has a different shape: no cell pass, no post.
        // Report both so the cost of the pixel layer stays visible.
        for (label, mut cv) in [
            ("pixel layer on ", Canvas::new()),
            ("pixel layer off", Canvas::terminal_sink()),
        ] {
            for i in 0..(n / 4).max(1) {
                terminal_frame(&mut g, &mut cv, &ctx, i as f64 * 0.5, dec.duration);
            }
            let t0 = std::time::Instant::now();
            for i in 0..n {
                let t = (i as f64 / n as f64) * dec.duration;
                terminal_frame(&mut g, &mut cv, &ctx, t, dec.duration);
            }
            let el = t0.elapsed().as_secs_f64();
            println!(
                "bench: terminal frame, {}: {:.2} ms/frame -> {:.1} fps",
                label,
                el * 1000.0 / n as f64,
                n as f64 / el
            );
        }
        return Ok(());
    }

    run_interactive(&args, &audio_path)
}

fn mono_mix(dec: &audio::Decoded) -> Vec<f32> {
    dec.samples
        .chunks(dec.channels)
        .map(|c| {
            let s: i32 = c.iter().map(|&v| v as i32).sum();
            s as f32 / (c.len() as f32 * 32768.0)
        })
        .collect()
}

/// Beat grid for an audio file, through the on-disk cache beside it.
///
/// Analysis costs ~1.8s and every entry point runs it on the same file — the
/// player, each capture, and each of the eight parallel video segments. The
/// cache key covers rate, sample count, duration and anchor count, so a stale
/// file can never be mistaken for a hit. Anything unexpected falls back to a
/// fresh analysis.
fn beat_grid(path: &Path, mono: &[f32], rate: u32, dur: f64) -> beats::BeatGrid {
    beats::analyze_cached(path, mono, rate, dur, &lyrics::anchors())
        .unwrap_or_else(|_| beats::analyze(mono, rate, dur, &lyrics::anchors()))
}

// ---------------------------------------------------------------- video ---

fn find_ffmpeg() -> PathBuf {
    for key in ["MV_FFMPEG", "WSY_FFMPEG"] {
        if let Ok(p) = std::env::var(key) {
            let p = PathBuf::from(p);
            if p.exists() {
                return p;
            }
        }
    }
    // imageio-ffmpeg's bundled binary (installed with the shared python tools)
    if let Ok(appdata) = std::env::var("APPDATA") {
        for py in ["Python39", "Python313", "Python312", "Python311", "Python310"] {
            let dir = Path::new(&appdata).join("Python").join(py).join("site-packages").join("imageio_ffmpeg").join("binaries");
            if let Ok(rd) = std::fs::read_dir(&dir) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.extension().and_then(|s| s.to_str()) == Some("exe")
                        && p.to_string_lossy().contains("ffmpeg")
                    {
                        return p;
                    }
                }
            }
        }
    }
    PathBuf::from("ffmpeg")
}

fn run_video(args: &Args, audio_path: &Path, spec: &str) -> Result<()> {
    let mut parts = spec.split(':');
    let start: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let end: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(296.0);
    let fps: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(60.0);

    eprintln!("decoding + analysing audio…");
    let dec = audio::decode(audio_path, |_| {})?;
    let mono = mono_mix(&dec);
    let grid = beat_grid(audio_path, &mono, dec.rate, dec.duration);
    let energy = audio::energy(&mono, dec.rate);
    if std::env::var("WSY_QUIET").is_err() {
        eprintln!("bpm {:.2} · rendering {:.2}..{:.2}s @ {}fps", grid.bpm, start, end, fps);
    }

    let ctx = Ctx { beats: &grid, energy: &energy, dur: dec.duration };
    let mut g = Grid::new(pix::COLS, pix::ROWS, pal::c(pal::BG0));
    let mut cv = Canvas::new();

    let total = ((end - start).max(0.0) * fps).ceil() as u64;
    let out_path = args
        .out
        .clone()
        .unwrap_or_else(|| PathBuf::from(format!("wsy_{}_{}.mp4", start as i64, end as i64)));

    let ff = find_ffmpeg();
    let mut cmd = std::process::Command::new(&ff);
    cmd.args([
        "-y", "-f", "rawvideo", "-pix_fmt", "rgb24",
        "-s", &format!("{}x{}", pix::W, pix::H),
        "-r", &format!("{}", fps as i64),
        "-i", "-",
        "-c:v", "libx264", "-preset", "faster", "-crf", "18", "-pix_fmt", "yuv420p",
    ]);
    cmd.arg(&out_path);
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());

    // The child's stdin: on Windows Rust's `Stdio::piped()` goes through a
    // named-pipe path that this machine rejects with ERROR_PIPE_BUSY, so the
    // pipe is created directly and its read end handed to the child.
    #[cfg(windows)]
    let mut stdin = {
        let (cfg, file) = win_pipe::stdin_pipe()?;
        cmd.stdin(cfg);
        file
    };
    #[cfg(not(windows))]
    cmd.stdin(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .with_context(|| format!("cannot spawn ffmpeg at {}", ff.display()))?;
    #[cfg(not(windows))]
    let mut stdin = child.stdin.take().context("no stdin to ffmpeg")?;

    let t0 = std::time::Instant::now();
    for i in 0..total {
        let t = start + i as f64 / fps;
        capture::render_one(&mut g, &mut cv, &ctx, t);
        use std::io::Write;
        if stdin.write_all(&cv.buf).is_err() {
            break;
        }
        if i % 300 == 0 {
            let el = t0.elapsed().as_secs_f64();
            eprint!(
                "\rframe {i}/{total} ({:.0}%) · {:.1} fps ",
                i as f64 / total as f64 * 100.0,
                i as f64 / el.max(1e-6)
            );
        }
    }
    drop(stdin);
    let _ = child.wait();
    eprintln!("\nsegment done -> {}", out_path.display());
    Ok(())
}

// -------------------------------------------------------------- capture ---

fn run_capture(audio_path: &Path, spec: &str) -> Result<()> {
    let dec = audio::decode(audio_path, |_| {})?;
    let mono = mono_mix(&dec);
    let grid = beat_grid(audio_path, &mono, dec.rate, dec.duration);
    let energy = audio::energy(&mono, dec.rate);
    let times = parse_times(spec, dec.duration);
    let outdir = std::env::var("WSY_OUT").unwrap_or_else(|_| ".preview".into());
    capture::capture_shots(&grid, &energy, dec.duration, &times, Path::new(&outdir))?;
    Ok(())
}

fn parse_times(spec: &str, dur: f64) -> Vec<f64> {
    let mut v = Vec::new();
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() == 3 {
        if let (Ok(a), Ok(b), Ok(st)) = (
            parts[0].trim().parse::<f64>(),
            parts[1].trim().parse::<f64>(),
            parts[2].trim().parse::<f64>(),
        ) {
            let mut t = a;
            while t <= b.max(a) {
                v.push(t);
                t += st.max(0.5);
            }
            return v;
        }
    }
    if let Some((a, b)) = spec.split_once("..") {
        let pa: f64 = a.trim().parse().unwrap_or(0.0);
        let pb: f64 = b.trim().parse().unwrap_or(dur);
        let n = 24;
        for i in 0..n {
            let t = pa + (pb - pa) * i as f64 / (n - 1).max(1) as f64;
            v.push(t);
        }
        return v;
    }
    for part in spec.split(',') {
        if let Ok(t) = part.trim().parse::<f64>() {
            v.push(t);
        }
    }
    v
}

// ----------------------------------------------------------- interactive ---

/// One terminal frame: background, scenes, lyrics, readout.
///
/// Shared by the player and by `--bench` so that what gets measured is exactly
/// what gets shown. The optional overlays (help, pause, loading) sit outside.
fn terminal_frame(g: &mut Grid, cv: &mut Canvas, ctx: &Ctx, t: f64, dur: f64) {
    g.clear(scenes::bg_at(t));
    scenes::render(g, cv, t, ctx);
    lyrics::render(g, cv, t, ctx.beats, false);
    hud::render(g, t, ctx.beats, dur);
}

/// True when the window can hold the film's fixed grid.
fn window_fits(term: &term::Term) -> bool {
    term.w >= pix::COLS && term.h >= pix::ROWS
}

/// Write the "window too small" card, clamped to the region the window shows.
///
/// The card is a static picture: it is only repainted when the size changes, so
/// it does not flicker while the user is dragging the window edge.
fn paint_too_small(term: &mut term::Term, g: &mut Grid, buf: &mut String) {
    hud::too_small(g, term.w, term.h);
    g.serialize_view(term.w, term.h, buf);
    term.write_frame(buf);
}

fn run_interactive(args: &Args, audio_path: &Path) -> Result<()> {
    let (tx, rx) = mpsc::channel::<Loaded>();
    let prog = Arc::new(AtomicU64::new(0));
    let path_bg = audio_path.to_path_buf();
    let prog_bg = prog.clone();
    let tx_bg = tx.clone();
    std::thread::spawn(move || {
        let dec = match audio::decode(&path_bg, |p| {
            prog_bg.store((p * 1000.0) as u64, Ordering::Relaxed);
        }) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("decode failed: {e:#}");
                std::process::exit(1);
            }
        };
        let mono = mono_mix(&dec);
        let (rate, dur) = (dec.rate, dec.duration);
        let arc = Arc::new(dec);
        if tx_bg.send(Loaded::Decoded(arc.clone())).is_err() {
            return;
        }
        let g = beat_grid(&path_bg, &mono, rate, dur);
        let en = audio::energy(&mono, rate);
        let _ = tx_bg.send(Loaded::Analysis(Arc::new(g), Arc::new(en)));
    });
    drop(tx);

    let mut term = match term::Term::init() {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "cannot take over this terminal: {e:#}\n\
                 playback needs a real console.\n\
                 \x20 Windows: run it from Windows Terminal, cmd.exe or PowerShell.\n\
                 \x20 If you started it from Git Bash / mintty and ended up here, try `winpty`\n\
                 \x20 or open Windows Terminal instead.\n\
                 \x20 (--capture, --video and --bench need no console at all.)"
            );
            return Ok(());
        }
    };
    let mut g = Grid::new(pix::COLS, pix::ROWS, pal::c(pal::BG0));
    // The player paints the character grid only. The pixel canvas is still
    // passed to the scenes (they share one code path with the renderer), but
    // with the pixel layer switched off so the work is not done and thrown away.
    let mut sink = Canvas::terminal_sink();

    let mut clock: Option<audio::Clock> = None;
    let mut beats = Arc::new(beats::BeatGrid { t0: 0.0, period: 0.5, bpm: 120.0 });
    let mut energy = Arc::new(audio::Energy { rate: 90.0, rms: vec![], bass: vec![], mid: vec![], high: vec![] });
    let mut dur = 296.0f64;
    let mut paused = false;
    let mut show_help = false;
    let mut finished = false;
    let mut fps_cap = args.fps_cap;
    let mut t: f64;
    let mut frame_buf = String::with_capacity(1 << 19);

    // The grid is a fixed 192x45 written with absolute cursor moves, so a
    // smaller window folds every row into the next. Hold here until the window
    // fits, rather than painting something unreadable. The card is painted once
    // per size change — a card repainted on a timer is a card that flickers.
    let mut card_painted = false;
    loop {
        let resized = term.check_resize();
        if window_fits(&term) {
            break;
        }
        if resized || !card_painted {
            paint_too_small(&mut term, &mut g, &mut frame_buf);
            card_painted = true;
        }
        if matches!(term::poll_input(std::time::Duration::from_millis(120)), term::Input::Quit) {
            term.restore();
            return Ok(());
        }
    }

    loop {
        let frame_t0 = std::time::Instant::now();

        // Shrinking the window mid-film has the same failure mode as starting in
        // a small one, so swap in the notice card and leave the clock running:
        // the film picks up where it should be once the window grows back.
        let resized = term.check_resize();
        if !window_fits(&term) {
            if resized || !card_painted {
                paint_too_small(&mut term, &mut g, &mut frame_buf);
                card_painted = true;
            }
            if matches!(term::poll_input(std::time::Duration::from_millis(120)), term::Input::Quit) {
                break;
            }
            if let Some(audio::Clock::Audio(a)) = &clock {
                if a.finished() {
                    finished = true;
                }
            }
            if finished {
                break;
            }
            continue;
        }
        card_painted = false;

        while let Ok(msg) = rx.try_recv() {
            match msg {
                Loaded::Decoded(d) => {
                    dur = d.duration;
                    if args.no_audio {
                        clock = Some(audio::Clock::wall());
                    } else {
                        match audio::AudioOut::start(&d) {
                            Ok(a) => clock = Some(audio::Clock::Audio(Box::new(a))),
                            Err(e) => {
                                term.restore();
                                return Err(anyhow!("audio output failed: {e}"));
                            }
                        }
                    }
                    if args.seek > 0.0 {
                        if let Some(c) = clock.as_mut() {
                            c.seek(args.seek, dur);
                        }
                    }
                }
                Loaded::Analysis(a, e) => {
                    beats = a;
                    energy = e;
                }
            }
        }

        t = match &clock {
            Some(c) => c.time(),
            None => 0.0,
        };

        let ctx = Ctx { beats: &beats, energy: &energy, dur };
        terminal_frame(&mut g, &mut sink, &ctx, t, dur);
        hud::fps_badge(&mut g, fps_cap);
        if show_help {
            hud::help(&mut g);
        }
        if paused {
            hud::paused_overlay(&mut g);
        }
        if clock.is_none() {
            hud::loading(&mut g, prog.load(Ordering::Relaxed) as f64 / 1000.0);
        }

        g.serialize(&mut frame_buf);
        term.write_frame(&frame_buf);

        match term::poll_input(std::time::Duration::from_millis(0)) {
            term::Input::Quit => break,
            term::Input::TogglePause => {
                if let Some(c) = clock.as_mut() {
                    c.set_paused(!paused);
                }
                paused = !paused;
            }
            term::Input::Seek(d) => {
                if let Some(c) = clock.as_mut() {
                    c.seek((t + d).clamp(0.0, dur), dur);
                }
            }
            term::Input::VolUp => {
                if let Some(audio::Clock::Audio(a)) = &clock {
                    a.set_volume((a.volume() + 0.1).clamp(0.0, 1.0));
                }
            }
            term::Input::VolDown => {
                if let Some(audio::Clock::Audio(a)) = &clock {
                    a.set_volume((a.volume() - 0.1).clamp(0.0, 1.0));
                }
            }
            term::Input::ToggleFps => {
                fps_cap = match fps_cap as i64 {
                    0 => 60.0,
                    60 => 30.0,
                    _ => 0.0,
                };
            }
            term::Input::Help => show_help = !show_help,
            term::Input::Resize => {}
            term::Input::Any | term::Input::None_ => {}
        }

        if let Some(audio::Clock::Audio(a)) = &clock {
            if a.finished() {
                finished = true;
            }
        }
        if finished {
            break;
        }

        // Frame pacing only decides how often we repaint — the film's time comes
        // from the audio clock, so dropping frames never desyncs the picture.
        // `--fps` / [f] set a ceiling; with no cap we hold a 12 ms floor.
        let budget_ms = if paused {
            30.0
        } else if fps_cap > 0.0 {
            1000.0 / fps_cap
        } else {
            12.0
        };
        let spent_ms = frame_t0.elapsed().as_secs_f64() * 1000.0;
        if spent_ms < budget_ms {
            std::thread::sleep(std::time::Duration::from_micros(
                ((budget_ms - spent_ms) * 1000.0) as u64,
            ));
        }
    }

    term.restore();
    println!("{SONG} — session over. 再见。");
    Ok(())
}
