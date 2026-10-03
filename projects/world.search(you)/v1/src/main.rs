//! world.search (you) ; — a terminal MV for Mili.

mod art;
mod audio;
mod beats;
mod capture;
mod fx;
mod gfx;
mod hud;
mod lyrics;
mod scenes;
mod term;

use anyhow::{anyhow, Context, Result};
use gfx::Grid;
use scenes::{Ctx, Ui};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

struct Args {
    audio: Option<PathBuf>,
    no_audio: bool,
    capture: Option<String>,
    cols: Option<usize>,
    rows: Option<usize>,
    seek: f64,
    fps_cap: f64,
    help: bool,
    dev_beats: bool,
    dev_audio: bool,
    bench: Option<f64>,
    video: Option<String>,
}

fn parse_args() -> Result<Args> {
    let mut a = Args {
        audio: None,
        no_audio: false,
        capture: None,
        cols: None,
        rows: None,
        seek: 0.0,
        fps_cap: 0.0,
        help: false,
        dev_beats: false,
        dev_audio: false,
        bench: None,
        video: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--audio" | "-a" => a.audio = it.next().map(PathBuf::from),
            "--no-audio" => a.no_audio = true,
            "--capture" | "-c" => a.capture = it.next(),
            "--cols" => a.cols = it.next().and_then(|s| s.parse().ok()),
            "--rows" => a.rows = it.next().and_then(|s| s.parse().ok()),
            "--seek" => a.seek = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0),
            "--fps" => a.fps_cap = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0),
            "--dev" => match it.next().as_deref() {
                Some("beats") => a.dev_beats = true,
                Some("audio") => a.dev_audio = true,
                other => return Err(anyhow!("unknown --dev target: {:?}", other)),
            },
            "--bench" => a.bench = it.next().and_then(|s| s.parse().ok()).or(Some(300.0)),
            "--video" => a.video = it.next(),
            "--help" | "-h" => a.help = true,
            other => return Err(anyhow!("unknown argument: {other} (try --help)")),
        }
    }
    Ok(a)
}

fn find_audio() -> Result<PathBuf> {
    let mut cands: Vec<PathBuf> = Vec::new();
    for dir in ["reference", "assets", "."] {
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

fn help_text() -> &'static str {
    "world.search (you) ;  —  a terminal MV for Mili\n\n\
     USAGE:\n  world-search-you [OPTIONS]\n\n\
     OPTIONS:\n\
     \x20 -a, --audio <FILE>   mp3 to sync with (default: any mp3 in reference/)\n\
     \x20     --no-audio        run on a wall clock (no sound)\n\
     \x20 -c, --capture <LIST>  offline frames for design review\n\
     \x20                       times: \"12,45.5,90\" or a range \"0:30\" or grid \"0:300:10\"\n\
     \x20     --cols <N>        force terminal width  (preview only)\n\
     \x20     --rows <N>        force terminal height (preview only)\n\
     \x20     --seek <T>        start at T seconds\n\
     \x20     --fps <N>         cap the frame rate\n\
     \x20 -h, --help            this text\n\n\
     KEYS (while playing):\n\
     \x20 space pause · ←/→ seek 5s · +/- volume · f fps · h help · q quit\n"
}

enum Loaded {
    Decoded(audio::Decoded),
    Beats(beats::BeatGrid),
}

fn main() -> Result<()> {
    let args = parse_args()?;
    if args.help {
        print!("{}", help_text());
        return Ok(());
    }

    let audio_path = match &args.audio {
        Some(p) => p.clone(),
        None => find_audio()?,
    };
    if !audio_path.exists() {
        return Err(anyhow!("audio file not found: {}", audio_path.display()));
    }

    // ---- dev: print beat analysis vs lyric anchors, then exit
    if args.dev_beats {        let dec = audio::decode(&audio_path, |_| {})?;
        let mono: Vec<f32> = dec
            .samples
            .chunks(dec.channels)
            .map(|c| {
                let s: i32 = c.iter().map(|&v| v as i32).sum();
                s as f32 / (c.len() as f32 * 32768.0)
            })
            .collect();
        let anchors: Vec<f64> = lyrics::LINES
            .iter()
            .filter(|l| !l.en.is_empty())
            .map(|l| l.t)
            .collect();
        let g = beats::analyze(&mono, dec.rate, dec.duration, &anchors);
        println!(
            "detected: {:.2} BPM · period {:.3}s · t0 {:.3}s · duration {:.1}s",
            g.bpm, g.period, g.t0, dec.duration
        );
        println!("{:>8}  {:>8}  {:>7}", "anchor", "nearest", "d(ms)");
        let mut worst = 0.0f64;
        for &a in anchors.iter().step_by(4) {
            let k = ((a - g.t0) / g.period).round();
            let near = g.beat_time(k as i64);
            let d = (a - near).abs() * 1000.0;
            worst = worst.max(d);
            println!("{:8.2}  {:8.2}  {:7.1}", a, near, d);
        }
        println!("worst offset: {:.0} ms", worst);
        return Ok(());
    }

    // ---- dev: play 1.5 s through the real output device, then exit
    if args.dev_audio {
        let dec = audio::decode(&audio_path, |_| {})?;
        let out = audio::AudioOut::start(&dec)?;
        let t0 = std::time::Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(1500));
        println!(
            "audio ok: device running, clock at {:.2}s after 1.5s · {:?} · {}ch {}Hz · {:.1}s total",
            out.time(),
            cpal::default_host().id(),
            dec.channels,
            dec.rate,
            dec.duration
        );
        let _ = t0;
        drop(out);
        return Ok(());
    }

    // ---- background loader: decode, then beat analysis
    let (tx, rx) = mpsc::channel::<Loaded>();
    let prog = Arc::new(AtomicU64::new(0));
    let path_bg = audio_path.clone();
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
        // mono mixdown for beat analysis (reuses the same PCM)
        let samples = dec.samples.clone();
        let mono: Vec<f32> = samples
            .chunks(dec.channels)
            .map(|c| {
                let s: i32 = c.iter().map(|&v| v as i32).sum();
                s as f32 / (c.len() as f32 * 32768.0)
            })
            .collect();
        let (rate, duration) = (dec.rate, dec.duration);
        if tx_bg.send(Loaded::Decoded(dec)).is_err() {
            return;
        }
        let anchors: Vec<f64> = lyrics::LINES
            .iter()
            .filter(|l| !l.en.is_empty())
            .map(|l| l.t)
            .collect();
        let fallback = beats::BeatGrid { t0: 0.0, period: 0.5, bpm: 120.0 };
        let grid = beats::analyze_cached(&path_bg, &mono, rate, duration, &anchors).unwrap_or(fallback);
        let _ = tx_bg.send(Loaded::Beats(grid));
    });
    drop(tx);

    // ---- dev: render N frames across the song and report timing (no terminal)
    if let Some(n) = args.bench {
        let dec = audio::decode(&audio_path, |_| {})?;
        let mono: Vec<f32> = dec
            .samples
            .chunks(dec.channels)
            .map(|c| {
                let s: i32 = c.iter().map(|&v| v as i32).sum();
                s as f32 / (c.len() as f32 * 32768.0)
            })
            .collect();
        let anchors: Vec<f64> = lyrics::LINES
            .iter()
            .filter(|l| !l.en.is_empty())
            .map(|l| l.t)
            .collect();
        let grid = beats::analyze(&mono, dec.rate, dec.duration, &anchors);
        let cols = args.cols.unwrap_or(120);
        let rows = args.rows.unwrap_or(36);
        let mut g = Grid::new(cols, rows, gfx::Color::hex(0x0a0d13));
        let mut buf = String::with_capacity(1 << 18);
        let ctx = Ctx { beats: &grid, dur: dec.duration };
        let ui = Ui { paused: false, volume: 0.85, anim: 0.0, finished: false, show_fps: false, fps: 60.0 };
        let n = n as usize;
        let t0 = std::time::Instant::now();
        let mut bytes: u64 = 0;
        for i in 0..n {
            let t = (i as f64 / n as f64) * dec.duration;
            g.clear();
            scenes::render(&mut g, t, &ui, &ctx);
            scenes::render_lyrics(&mut g, t, &ctx);
            hud::render(&mut g, t, &ui, &ctx);
            g.serialize(&mut buf);
            bytes += buf.len() as u64;
        }
        let el = t0.elapsed().as_secs_f64();
        println!(
            "bench: {} frames in {:.2}s -> {:.1} fps ({}x{}, avg frame {:.2} ms, {:.0} KB/frame)",
            n,
            el,
            n as f64 / el,
            cols,
            rows,
            el * 1000.0 / n as f64,
            bytes as f64 / n as f64 / 1024.0
        );
        return Ok(());
    }

    // ---- video render: stream packed frames to stdout for the Python encoder
    if let Some(spec) = &args.video {
        let dec = audio::decode(&audio_path, |_| {})?;
        let mono: Vec<f32> = dec
            .samples
            .chunks(dec.channels)
            .map(|c| {
                let s: i32 = c.iter().map(|&v| v as i32).sum();
                s as f32 / (c.len() as f32 * 32768.0)
            })
            .collect();
        let anchors: Vec<f64> = lyrics::LINES
            .iter()
            .filter(|l| !l.en.is_empty())
            .map(|l| l.t)
            .collect();
        let grid = beats::analyze(&mono, dec.rate, dec.duration, &anchors);
        let cols = args.cols.unwrap_or(120);
        let rows = args.rows.unwrap_or(36);
        // "start:end:fps" (end may exceed the song so the outro completes)
        let mut parts = spec.split(':');
        let start: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let end: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(dec.duration);
        let fps: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(60.0);
        capture::stream_video(&mut Grid::new(cols, rows, gfx::Color::hex(0x0a0d13)), &grid, dec.duration, start, end, fps);
        return Ok(());
    }

    // ---- capture mode?
    if let Some(spec) = &args.capture {
        return run_capture(&args, &audio_path, &rx, &prog, spec);
    }

    // ---- interactive
    let mut term = term::Term::init().context("cannot init terminal (raw mode)")?;
    if let Some(c) = args.cols { term.w = c; }
    if let Some(r) = args.rows { term.h = r; }
    let cols = term.w.max(20);
    let rows = term.h.max(10);
    let mut g = Grid::new(cols, rows, gfx::Color::hex(0x0a0d13));
    let mut frame_buf = String::with_capacity(1 << 18);

    let mut clock: Option<audio::Clock> = None;
    let mut beats = Arc::new(beats::BeatGrid { t0: 0.0, period: 0.5, bpm: 120.0 });
    let mut dur = 292.5f64;
    let mut ui = Ui { paused: false, volume: 0.85, anim: 0.0, finished: false, show_fps: false, fps: 0.0 };
    let mut show_help = false;
    let mut last = std::time::Instant::now();
    let start = std::time::Instant::now();
    #[allow(unused_assignments)]
    let mut t = 0.0f64;
    let mut fps_ema = 60.0f64;

    // loading phase
    loop {
        // drain loader
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
                        if let Some(c) = clock.as_mut() { c.seek(args.seek, dur); }
                    }
                }
                Loaded::Beats(b) => beats = Arc::new(b),
            }
        }

        if let Some(c) = &clock {
            t = c.time();
        } else {
            t = start.elapsed().as_secs_f64();
        }

        // ---- draw
        g.clear();
        let ctx = Ctx { beats: &beats, dur };
        scenes::render(&mut g, t, &ui, &ctx);
        scenes::render_lyrics(&mut g, t, &ctx);
        hud::render(&mut g, t, &ui, &ctx);
        if show_help { hud::help(&mut g); }
        if clock.is_none() {
            draw_loading(&mut g, prog.load(Ordering::Relaxed) as f64 / 1000.0, t);
        }
        g.serialize(&mut frame_buf);
        term.write_frame(&frame_buf);

        // ---- input
        match term::poll_input(std::time::Duration::from_millis(0)) {
            term::Input::Quit => break,
            term::Input::TogglePause => {
                if let Some(c) = clock.as_mut() { c.set_paused(!ui.paused); }
                ui.paused = !ui.paused;
            }
            term::Input::Seek(d) => {
                if let Some(c) = clock.as_mut() { c.seek((t + d).clamp(0.0, dur), dur); }
            }
            term::Input::VolUp => {
                ui.volume = (ui.volume + 0.1).min(1.0);
                if let Some(audio::Clock::Audio(a)) = &clock { a.set_volume(ui.volume); }
            }
            term::Input::VolDown => {
                ui.volume = (ui.volume - 0.1).max(0.0);
                if let Some(audio::Clock::Audio(a)) = &clock { a.set_volume(ui.volume); }
            }
            term::Input::ToggleFps => ui.show_fps = !ui.show_fps,
            term::Input::Help => show_help = !show_help,
            term::Input::Resize => {}
            _ => {}
        }

        // finished?
        if let Some(audio::Clock::Audio(a)) = &clock {
            if a.finished() && !ui.finished {
                ui.finished = true;
            }
        }

        // ---- pacing
        let dt = last.elapsed().as_secs_f64();
        last = std::time::Instant::now();
        ui.anim += dt;
        if dt > 0.0 {
            fps_ema = fps_ema * 0.92 + (1.0 / dt) * 0.08;
        }
        ui.fps = fps_ema;

        if let Some(c) = &clock {
            if c.is_paused() {
                std::thread::sleep(std::time::Duration::from_millis(30));
            } else {
                std::thread::sleep(std::time::Duration::from_millis(12));
            }
        } else {
            std::thread::sleep(std::time::Duration::from_millis(12));
        }
    }

    term.restore();
    println!("world.search(you); — session over. 再见。");
    Ok(())
}

fn draw_loading(g: &mut Grid, p: f64, t: f64) {
    let w = g.w as i64;
    let h = g.h as i64;
    let y = h / 2;
    g.text_center(y - 2, "world.search (you) ;", gfx::Color::hex(0xff6d8a));
    g.text_center(y, &format!("loading audio… {:>3.0}%", p * 100.0), gfx::Color::hex(0x8a93a6));
    let bw = 40.min(w - 10);
    let bx = (w - bw) / 2;
    g.hline(bx, bx + bw - 1, y + 2, '░', gfx::Color::hex(0x334));
    let filled = (bw as f64 * p) as i64;
    for k in 0..filled {
        g.put(bx + k, y + 2, '▓', gfx::Color::hex(0xff6d8a));
    }
    if (t * 2.0) as i64 % 2 == 0 {
        g.put(bx + filled, y + 2, '▌', gfx::Color::hex(0xff6d8a));
    }
    g.text_center(y + 4, "calibrating the clock…", gfx::Color::hex(0x55607a));
}

fn run_capture(
    args: &Args,
    audio_path: &std::path::Path,
    rx: &mpsc::Receiver<Loaded>,
    prog: &Arc<AtomicU64>,
    spec: &str,
) -> Result<()> {
    // wait for decode + beats
    #[allow(unused_assignments)]
    let mut beats = beats::BeatGrid { t0: 0.0, period: 0.5, bpm: 120.0 };
    let mut dur = 292.5f64;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
    loop {
        match rx.recv_timeout(std::time::Duration::from_millis(500)) {
            Ok(Loaded::Decoded(d)) => dur = d.duration,
            Ok(Loaded::Beats(b)) => {
                beats = b;
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if std::time::Instant::now() > deadline {
                    return Err(anyhow!("timed out loading audio"));
                }
                eprint!("\rdecoding… {:>3.0}%", prog.load(Ordering::Relaxed) as f64 / 10.0);
            }
            Err(_) => return Err(anyhow!("loader died")),
        }
    }
    eprintln!();

    let shots = parse_shots(spec, dur);
    let cols = args.cols.unwrap_or(100);
    let rows = args.rows.unwrap_or(32);
    let mut g = Grid::new(cols, rows, gfx::Color::hex(0x0a0d13));
    let base = std::path::Path::new(".preview");
    let stem = std::env::var("WSY_OUT").unwrap_or_else(|_| "frames".into());
    let html = base.join(format!("{stem}.html"));
    capture::write_html(&mut g, &beats, dur, &shots, &html)?;
    let dump = base.join(format!("{stem}.txt"));
    capture::write_dump(&mut g, &beats, dur, &shots, &dump)?;
    println!("wrote {} + {} ({} frames, {}x{})", html.display(), dump.display(), shots.len(), cols, rows);
    let _ = audio_path;
    Ok(())
}

fn parse_shots(spec: &str, dur: f64) -> Vec<capture::Shot> {
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
                v.push(shot(t));
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
            v.push(shot(t));
        }
        return v;
    }
    for part in spec.split(',') {
        if let Ok(t) = part.trim().parse::<f64>() {
            v.push(shot(t));
        }
    }
    v
}

fn shot(t: f64) -> capture::Shot {
    let sec = scenes::section_at(t);
    capture::Shot { t, title: format!("{} ({:.1}s)", sec.label, t) }
}
