//! NotToNotice(); — a terminal MV for CRYMACHINA's opening theme.
//!
//! The whole video is one console session: the Eighth Deus Ex Machina, ENOA,
//! runs her mission (mind restoration), forges E.V.E souls, discovers tears in
//! her logs, hides them behind `NotToNotice();` … and finally stops pretending.

mod art;
mod audio;
mod beats;
mod capture;
mod fx;
mod gfx;
mod hud;
mod lyrics;
mod pix;
mod scenes;
mod term;

use anyhow::{anyhow, Context, Result};
use gfx::{pal, Grid};
use pix::Canvas;
use scenes::Ctx;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};

struct Args {
    audio: Option<PathBuf>,
    no_audio: bool,
    capture: Option<String>,
    video: Option<String>,
    out: Option<PathBuf>,
    cols: Option<usize>,
    rows: Option<usize>,
    seek: f64,
    fps_cap: f64,
    dev_beats: bool,
    dev_audio: bool,
    dev_type: bool,
    geometry: bool,
    bench: Option<usize>,
    help: bool,
}

fn parse_args() -> Result<Args> {
    let mut a = Args {
        audio: None,
        no_audio: false,
        capture: None,
        video: None,
        out: None,
        cols: None,
        rows: None,
        seek: 0.0,
        fps_cap: 0.0,
        dev_beats: false,
        dev_audio: false,
        dev_type: false,
        geometry: false,
        bench: None,
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
            "--cols" => a.cols = it.next().and_then(|s| s.parse().ok()),
            "--rows" => a.rows = it.next().and_then(|s| s.parse().ok()),
            "--seek" => a.seek = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0),
            "--fps" => a.fps_cap = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0),
            "--dev" => match it.next().as_deref() {
                Some("beats") => a.dev_beats = true,
                Some("audio") => a.dev_audio = true,
                Some("type") => a.dev_type = true,
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
    for dir in [".", "reference", "assets", "..", "../reference"] {
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
        .ok_or_else(|| anyhow!("no .mp3 found - put the song beside the exe or pass --audio"))
}

fn help_text() -> &'static str {
    "NotToNotice(); - a terminal MV for CRYMACHINA\n\n\
     USAGE:\n  not-to-notice [OPTIONS]\n\n\
     OPTIONS:\n\
     \x20 -a, --audio <FILE>   mp3 to sync with (default: any mp3 in ./ or reference/)\n\
     \x20     --no-audio        run on a wall clock (no sound)\n\
     \x20 -c, --capture <LIST>  render frames to .preview/*.png (full video pipeline)\n\
     \x20                       times: \"12,45.5,90\" or \"a..b\" (24 steps) or \"a:b:step\"\n\
     \x20     --video <SPEC>    offline render \"start:end:fps\" (deterministic)\n\
     \x20     --out <FILE>      video output path (mp4; ffmpeg via NTN_FFMPEG or PATH)\n\
     \x20     --geometry        print the terminal grid geometry and exit\n\
     \x20     --seek <T>        start at T seconds (interactive)\n\
     \x20     --fps <N>         cap the frame rate (interactive)\n\
     \x20     --bench <N>       render N frames across the song, report timing\n\
     \x20     --dev beats|audio|type  analysis self-test; `type` dumps cell typography\n\
     \x20 -h, --help            this text\n\n\
     KEYS (interactive): space pause / arrows seek / +- volume / f fps / q quit\n"
}

/// What the loader thread hands back to the main loop.
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
    // cell typography self-test — needs no audio
    if args.dev_type {
        pix::type_report();
        return Ok(());
    }

    let audio_path = match &args.audio {
        Some(p) => p.clone(),
        None => find_audio()?,
    };
    if !audio_path.exists() {
        return Err(anyhow!("audio file not found: {}", audio_path.display()));
    }

    // ---- dev: beat analysis report
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

    // ---- dev: audio output self-test
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

    // ---- video render: deterministic frames -> ffmpeg
    if let Some(spec) = &args.video {
        return run_video(&args, &audio_path, spec);
    }

    // ---- capture: png stills for design review
    if let Some(spec) = &args.capture {
        return run_capture(&audio_path, spec);
    }

    // ---- bench
    if let Some(n) = args.bench {
        let dec = audio::decode(&audio_path, |_| {})?;
        let mono = mono_mix(&dec);
        let grid = beats::analyze(&mono, dec.rate, dec.duration, &lyrics::anchors());
        let energy = audio::energy(&mono, dec.rate);
        let ctx = Ctx { beats: &grid, energy: &energy, dur: dec.duration };
        let mut g = Grid::new(pix::COLS, pix::ROWS, pal::c(pal::BG0));
        let mut cv = Canvas::new();
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
        return Ok(());
    }

    // ---- interactive player (cells only; the full pixel FX live in the video)
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

// ---------------------------------------------------------------- video ---

fn find_ffmpeg() -> PathBuf {
    if let Ok(p) = std::env::var("NTN_FFMPEG") {
        let p = PathBuf::from(p);
        if p.exists() {
            return p;
        }
    }
    if let Ok(p) = std::env::var("MV_FFMPEG") {
        let p = PathBuf::from(p);
        if p.exists() {
            return p;
        }
    }
    // imageio-ffmpeg's bundled binary (installed with the python tools)
    if let Ok(appdata) = std::env::var("APPDATA") {
        let pattern = Path::new(&appdata).join("Python").join("Python39").join("site-packages").join("imageio_ffmpeg").join("binaries");
        if let Ok(rd) = std::fs::read_dir(&pattern) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) == Some("exe") && p.to_string_lossy().contains("ffmpeg") {
                    return p;
                }
            }
        }
    }
    PathBuf::from("ffmpeg")
}

fn run_video(args: &Args, audio_path: &Path, spec: &str) -> Result<()> {
    let mut parts = spec.split(':');
    let start: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let end: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(183.2);
    let fps: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(60.0);

    eprintln!("decoding + analyzing audio…");
    let dec = audio::decode(audio_path, |_| {})?;
    let mono = mono_mix(&dec);
    let grid = beats::analyze(&mono, dec.rate, dec.duration, &lyrics::anchors());
    let energy = audio::energy(&mono, dec.rate);
    eprintln!("bpm {:.2} · rendering {:.2}..{:.2}s @ {}fps", grid.bpm, start, end, fps);

    let ctx = Ctx { beats: &grid, energy: &energy, dur: dec.duration };
    let mut g = Grid::new(pix::COLS, pix::ROWS, pal::c(pal::BG0));
    let mut cv = Canvas::new();

    // round, not ceil: a frame-aligned segment boundary such as
    // (3664 - 1832) / 60 * 60 lands on ...0000002 and ceil would add a phantom
    // frame to every segment, stretching the film.
    let total = ((end - start) * fps).round().max(0.0) as u64;
    let out_path = args
        .out
        .clone()
        .unwrap_or_else(|| PathBuf::from(format!("ntn_{}_{}.mp4", start as i64, end as i64)));

    // spawn ffmpeg reading rawvideo on stdin
    let ff = find_ffmpeg();
    let mut cmd = std::process::Command::new(&ff);
    cmd.args([
        "-y",
        "-f", "rawvideo",
        "-pix_fmt", "rgb24",
        "-s", &format!("{}x{}", pix::W, pix::H),
        "-r", &format!("{}", fps as i64),
        "-i", "-",
        "-c:v", "libx264",
        "-preset", "fast",
        "-crf", "15",
        "-pix_fmt", "yuv420p",
    ]);
    cmd.arg(&out_path);
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());
    let mut child = cmd
        .spawn()
        .with_context(|| format!("cannot spawn ffmpeg at {}", ff.display()))?;
    let mut stdin = child.stdin.take().context("no stdin to ffmpeg")?;

    let t0 = std::time::Instant::now();
    for i in 0..total {
        let t = start + i as f64 / fps;
        capture::render_one(&mut g, &mut cv, &ctx, t);
        use std::io::Write;
        stdin.write_all(&cv.buf)?;
        if i % 300 == 0 {
            let el = t0.elapsed().as_secs_f64();
            eprint!("\rframe {i}/{total} ({:.0}%) · {:.1} fps ", i as f64 / total as f64 * 100.0, i as f64 / el.max(1e-6));
        }
    }
    drop(stdin);
    let _ = child.wait();
    eprintln!("\nvideo segment done -> {}", out_path.display());
    Ok(())
}

// -------------------------------------------------------------- capture ---

fn run_capture(audio_path: &Path, spec: &str) -> Result<()> {
    let dec = audio::decode(audio_path, |_| {})?;
    let mono = mono_mix(&dec);
    let grid = beats::analyze(&mono, dec.rate, dec.duration, &lyrics::anchors());
    let energy = audio::energy(&mono, dec.rate);

    let times = parse_times(spec, dec.duration);
    let outdir = std::env::var("NTN_OUT").unwrap_or_else(|_| ".preview".into());
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
        let g = beats::analyze(&mono, rate, dur, &lyrics::anchors());
        let en = audio::energy(&mono, rate);
        let _ = tx_bg.send(Loaded::Analysis(Arc::new(g), Arc::new(en)));
    });
    drop(tx);

    let mut term = term::Term::init()?;

    // interactive always draws the full 192x45 board (clipped on small terminals)
    let mut g = Grid::new(pix::COLS, pix::ROWS, pal::c(pal::BG0));
    let mut sink = Canvas::new(); // sprites are dropped in terminal mode

    let mut clock: Option<audio::Clock> = None;
    let mut beats = Arc::new(beats::BeatGrid { t0: 0.0, period: 0.5, bpm: 120.0 });
    let mut energy = Arc::new(audio::Energy {
        rate: 90.0,
        rms: vec![],
        bass: vec![],
        high: vec![],
    });
    let mut dur = 183.0f64;
    let mut paused = false;
    let mut finished = false;
    let start = std::time::Instant::now();
    let mut frame_buf = String::with_capacity(1 << 18);

    loop {
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

        let t = match &clock {
            Some(c) => c.time(),
            None => start.elapsed().as_secs_f64(),
        };

        // ---- draw (cells; the pixel layer is a video-only upgrade)
        let bg = scenes::bg_at(t);
        g.clear(bg);
        let ctx = Ctx { beats: &beats, energy: &energy, dur };
        scenes::render(&mut g, &mut sink, t, &ctx);
        lyrics::render(&mut g, &mut sink, t, &beats);
        hud::render(&mut g, t, &beats, dur);

        g.serialize(&mut frame_buf);
        term.write_frame(&frame_buf);

        // ---- input
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
            _ => {}
        }

        if let Some(audio::Clock::Audio(a)) = &clock {
            if a.finished() {
                finished = true;
            }
        }
        if finished && clock.is_some() {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(if paused { 30 } else { 12 }));
    }

    term.restore();
    println!("NotToNotice(); — session closed. またね。");
    Ok(())
}
