//! 第一幕：开机。
//!
//! Boot  —— 一个空文件被打开，`me.rs` 逐行写出来：歌词是注释，代码是回应。
//! Title —— 全屏只剩一行正在被写下的调用：world.execute(me);

use crate::bigfont;
use crate::buf::{Rgb, sw};
use crate::code::{self, DocBuilder, Gutter, SrcLine, ViewOpts};
use crate::fx;
use crate::fx2::{self, TokenRain};
use crate::scenes::{Ctx, Scene};
use crate::theme::*;

// ── Boot：新文件 ────────────────────────────────────────────
pub struct Boot {
    rain: Option<TokenRain>,
}

impl Boot {
    pub fn new() -> Self {
        Boot { rain: None }
    }
}

impl Scene for Boot {
    fn name(&self) -> &'static str {
        "NEW FILE"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        let frame = ctx.frame;
        ctx.clear(BG);

        // 背景里极暗的 token 雨（"机器在等你打字"）
        if self.rain.is_none() {
            self.rain = Some(TokenRain::new(ctx.r.w, ctx.r.h, &mut ctx.rng));
        }
        let r = ctx.bounds();
        if let Some(rain) = &mut self.rain {
            rain.draw(ctx.c, r, ctx.dt, CYAN.mul(0.5), 0.10 + 0.08 * ctx.bass());
        }

        let mut d = DocBuilder::new();
        d.at(1);
        d.code(lt, 0.60, "use world::prelude::*;", 30.0);
        d.blank();

        // Switch on the power line
        d.cmt(lt, 2.00, "// switch on the power line", 30.0);
        let i_pw = d.code(lt, 2.45, "let power = Power::switch_on();", 32.0);
        d.blank();

        // Remember to put on PROTECTION（大写词落在 2.92 的唱点上）
        d.cmt(lt, 3.30, "// remember to put on PROTECTION", 9.0);
        d.code(lt, 4.55, "#![deny(unsafe_code)]", 26.0);
        d.blank();

        // Lay down your pieces — OBJECT CREATION（唱点 3.87 / 6.38）
        d.cmt(lt, 5.35, "// lay down your pieces — OBJECT CREATION", 8.5);
        d.code(lt, 6.90, "let me = Me::builder()", 26.0);
        d.code(lt, 7.40, "    .pieces(7)", 24.0);
        d.code_cmt(
            lt,
            7.95,
            "    .data(PARAMETERS)",
            30.0,
            9.20,
            "// fill in my data parameters",
        );
        d.code_cmt(lt, 10.15, "    .init()", 24.0, 10.15, "// INITIALIZATION");
        let i_build = d.code(lt, 10.80, "    .build()?;", 26.0);
        d.blank();

        // Set up our new world（唱点 11.10 / 12.91 / 13.89）
        d.cmt(lt, 11.15, "// set up our new world", 26.0);
        d.code(lt, 11.75, "let mut world = World::spawn(me.clone());", 30.0);
        d.code_cmt(
            lt,
            13.30,
            "world.begin(Simulation::new());",
            26.0,
            13.90,
            "// SIMULATION",
        );

        let mut lines = d.finish(frame);

        // 最早的两条运行时回显：这台机器的每一行都有结果
        if lt > 3.70 {
            if let Some(l) = lines.get_mut(i_pw) {
                l.ghost = Some(("// → Ok(Power)".to_string(), GREEN.mul(0.85)));
            }
        }
        if lt > 11.70 {
            if let Some(l) = lines.get_mut(i_build) {
                l.ghost = Some(("// → Me { pieces: 7 }".to_string(), GREEN.mul(0.85)));
            }
        }

        // 唱到 SIMULATION 时给注释芯片一点脉冲
        if ctx.t > 13.9 {
            if let Some(l) = lines.last_mut() {
                l.hl = Some((31, 41, BLUE.mul(0.4 + 0.4 * ctx.bass())));
            }
        }

        // 开场 0.5s：光标独自在空文件上闪烁（"它醒了"）
        if ctx.t < 0.05 {
            lines.clear();
            lines.push(SrcLine::new("").gut(Gutter::Num(1)).caret(true).rev(0));
        }

        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);
    }
}

// ── Title：world.execute(me); ───────────────────────────────
pub struct Title {
    rain: Option<TokenRain>,
}

impl Title {
    pub fn new() -> Self {
        Title { rain: None }
    }
}

impl Scene for Title {
    fn name(&self) -> &'static str {
        "world.execute(me);"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        if self.rain.is_none() {
            self.rain = Some(TokenRain::new(ctx.r.w, ctx.r.h, &mut ctx.rng));
        }
        let r = ctx.bounds();
        if let Some(rain) = &mut self.rain {
            rain.draw(ctx.c, r, ctx.dt, CYAN, 0.16 + 0.14 * ctx.bass());
        }

        // 旧文档的幽灵：整体压暗
        let ghost = 0.16 + 0.06 * (lt * 0.7).sin();
        let mut d = DocBuilder::new();
        d.push(SrcLine::num(1, "use world::prelude::*;").dimf(ghost));
        d.push(SrcLine::num(2, "let me = Me::builder()").dimf(ghost));
        d.push(SrcLine::num(3, "    .pieces(7)").dimf(ghost));
        d.push(SrcLine::num(4, "    .data(PARAMETERS)").dimf(ghost));
        d.push(SrcLine::num(5, "    .init()").dimf(ghost));
        d.push(SrcLine::num(6, "    .build()?;").dimf(ghost));
        d.push(SrcLine::num(7, "let mut world = World::spawn(me.clone());").dimf(ghost));
        d.push(SrcLine::num(8, "world.begin(Simulation::new());").dimf(ghost));
        let lines = d.finish(ctx.frame);
        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);

        let w = ctx.r.w;
        let cx = ctx.midx();

        // 大字标题：把那行调用写出来（16.3s 起逐字点亮）
        let title = "world.execute(me);";
        let rev = code::typed(lt, 0.30, 7.5) as f32;
        let big_w = bigfont::width(title, 1);
        let two_row = big_w > w - 6;
        // 结束段（27.4s 起）整体上收淡出
        let lift = ((lt - 11.4) / 2.4).clamp(0.0, 1.0);
        let fade = 1.0 - lift;
        if fade > 0.01 && !two_row {
            let y = ctx.midy() - 3 - (lift * 5.0) as i32;
            let fg = CYAN.mix(WHITE, 0.25 + 0.35 * ctx.bass()).mul(fade);
            let glow = PURPLE.mul(fade);
            bigfont::draw_reveal(
                ctx.c,
                ctx.r.x + cx - big_w / 2,
                ctx.r.y + y,
                title,
                rev,
                fg,
                glow,
                BG,
                1,
            );
        } else if fade > 0.01 {
            // 窄终端：两行堆叠
            for (i, part) in ["world.", "execute(me);"].iter().enumerate() {
                let pw = bigfont::width(part, 1);
                let y = ctx.midy() - 5 + i as i32 * 6 - (lift * 5.0) as i32;
                let fg = CYAN.mix(WHITE, 0.25).mul(fade);
                bigfont::draw_reveal(
                    ctx.c,
                    ctx.r.x + cx - pw / 2,
                    ctx.r.y + y,
                    part,
                    (rev - i as f32 * 7.0).max(0.0),
                    fg,
                    PURPLE.mul(fade),
                    BG,
                    1,
                );
            }
        }

        // "运行"输出：分号落下之后，绿色 stdout 逐行打出
        let outs: [(&str, f32, Rgb); 3] = [
            ("> thread 'me' started", 4.30, GREEN),
            ("> i am a set of points. i am ready.", 6.10, GREEN.mul(0.8)),
            ("> awaiting caller …", 8.20, GREEN.mul(0.6)),
        ];
        for (i, (s, t0, col)) in outs.iter().enumerate() {
            let shown = fx2::typed_str(s, lt, *t0, 26.0);
            if shown.is_empty() {
                continue;
            }
            let a = 1.0 - lift;
            ctx.text(
                cx - sw(s) / 2,
                ctx.midy() + 4 + i as i32,
                &shown,
                col.mul(a.max(0.15)),
            );
        }

        // 27.4s 起：它落回文件里成为第 1 行
        if lt > 11.4 {
            let mut d = DocBuilder::new();
            d.at(1);
            d.code(lt, 11.6, "world.execute(me);", 22.0);
            let l2 = d.finish(ctx.frame);
            let mut opts = ViewOpts::default();
            opts.minimap = false;
            let mut sub = crate::scenes::Ctx {
                c: ctx.c,
                r: crate::buf::Rect::new(ctx.r.x, ctx.r.y, ctx.r.w, 3),
                t: ctx.t,
                lt: ctx.lt,
                dt: ctx.dt,
                frame: ctx.frame,
                rng: fx::Rng::new(ctx.frame),
                v: ctx.v,
                ly: ctx.ly,
            };
            code::render(&mut sub, &l2, &opts);
        }

        // 结尾闪白：调用完成
        let f = fx::pulse(lt - 12.62, 0.04, 0.30);
        if f > 0.0 {
            ctx.flash(f * 0.35, WHITE);
        }
    }
}
