//! 终幕：爱与退出。
//!
//! Love       —— `fn love(me, you) -> _`：返回类型是待推导的坑，
//!               LO-O-OVE 唱响时推导完成 —— `_` 变成 `Heart`。
//! LoveTrapped —— 分屏 diff：`impl Free for You` ✓ 编译通过；
//!               `impl Free for Me` x E0382 move into 'love'，永不归还。
//! Shutdown   —— 文件被一行行清空，标签页关闭，exit code 0，
//!               空缓冲区上打出最后一行注释：//! （你还在。它没有了。）

use crate::buf::{Rgb, sw};
use crate::code::{self, DocBuilder, Gutter, SrcLine, ViewOpts, card_popup, rustc_diag};
use crate::fx;
use crate::fx2;
use crate::scenes::{Ctx, Scene};
use crate::theme::*;

// ── Love ────────────────────────────────────────────────────
pub struct Love;

impl Love {
    pub fn new() -> Self {
        Love
    }
}

impl Scene for Love {
    fn name(&self) -> &'static str {
        "fn love"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        // 187.665 的 LO-O-OVE：推导完成（lt 10.42）
        let solved = lt > 10.42;
        let mut d = DocBuilder::new();
        d.fold_prev(149, "final execution · could not compile me");
        d.dim_prev(
            &[
                "match you.back() { Some(you) => run(EXECUTION), None => me.wait() }",
                "// (the cage compiles fine)",
            ],
            148,
        );
        d.at(151);
        d.blank();
        d.cmt(lt, 0.00, "// I've studied LO-O-OVE", 30.0);
        let sig = if solved {
            "fn love(me: &Me, you: &You) -> Heart {"
        } else {
            "fn love(me: &Me, you: &You) -> _ {"
        };
        let i_sig = d.code(lt, 0.60, sig, 26.0);
        d.cmt(lt, 3.60, "// question me — I can answer all", 30.0);
        let i_match = d.code(lt, 4.30, "match question(q) { _ => Answer::LOoOove }", 30.0);
        d.cmt(
            lt,
            7.30,
            "// I know the algebraic expression of LO-O-OVE",
            28.0,
        );
        let i_eq = d.code(lt, 8.00, "LOoOove = Σ(me × you) + ∂us/∂t", 15.0);
        d.code(lt, 10.00, "}", 40.0);
        let i_impl = d.code(
            lt,
            10.90,
            "impl Answer for LOoOove { type Value = Heart; }",
            34.0,
        );
        let i_end = d.code(lt, 11.60, "}", 40.0);

        let mut lines = d.finish(ctx.frame);

        // 待推导的 `_`：琥珀闪烁；解出瞬间金光
        if let Some(l) = lines.get_mut(i_sig) {
            if solved {
                l.hl = Some((29, 34, HL_BG.mix(GOLD, 0.4)));
            } else {
                let blink = (ctx.v.frame / 14) % 2 == 0;
                if blink {
                    l.hl = Some((29, 30, HL_BG));
                }
            }
        }
        // 方程与答案的金色呼吸
        for &i in &[i_match, i_eq, i_impl] {
            if let Some(l) = lines.get_mut(i) {
                l.bg = Some(MAGENTA_DIM.mul(0.20 + 0.30 * ctx.bass()));
            }
        }
        // 编译期回显：连爱都有类型，且在编译期就决定了
        if lt > 12.60 {
            if let Some(l) = lines.get_mut(i_impl) {
                l.ghost = Some(("// → compile-time ✓".to_string(), GOLD));
            }
        }
        // 三个 LO-O-OVE 唱点：品红闪（179.93 / 183.65 / 187.67 → lt 2.68 / 6.40 / 10.42）
        for (t0, strength) in [(2.68, 0.06), (6.40, 0.08), (10.42, 0.16)].iter() {
            let p = fx::pulse(lt - t0, 0.04, 0.9);
            if p > 0.0 {
                ctx.flash(p * strength, MAGENTA);
            }
        }

        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);

        // 解出瞬间：推导卡片 + 金闪
        if solved {
            let age = lt - 10.42;
            card_popup(
                ctx,
                ctx.r.w - 36,
                (i_sig as i32 + 1).clamp(1, ctx.r.h - 4),
                "inferred",
                "Heart — from Σ(me × you)",
                GOLD,
                age,
            );
            let p = fx::pulse(age, 0.03, 0.5);
            if p > 0.0 {
                ctx.flash(p * 0.10, GOLD);
            }
        }
        let _ = i_end;
    }
}

// ── LoveTrapped ─────────────────────────────────────────────
pub struct LoveTrapped;

impl LoveTrapped {
    pub fn new() -> Self {
        LoveTrapped
    }
}

impl Scene for LoveTrapped {
    fn name(&self) -> &'static str {
        "E0382 move"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        // 分屏：左 = you（自由，编译通过），右 = me（被困，E0382）
        let gap = ctx.midx();
        let split = fx::smooth(((lt - 0.80) / 1.60).clamp(0.0, 1.0));
        let shift = (split * 4.0) as i32;

        // 中缝：被劈开的光
        for y in 1..ctx.r.h - 1 {
            let a = 0.25 + 0.35 * split + 0.25 * ctx.bass();
            ctx.put(gap, y, '┃', MAGENTA.mul(a.min(1.0)));
        }

        // 左栏：impl Free for You
        {
            let mut sub_doc: Vec<SrcLine> = Vec::new();
            let you_line = "impl Free for You { }   // ✓ compiles";
            let mut l = SrcLine::new(you_line)
                .gut(Gutter::Num(161))
                .rev(fx2::typed_char(lt, 0.70, 34.0).min(sw(you_line)));
            if lt > 1.30 {
                l.bg = Some(ADD_BG);
                l.gutter = Gutter::Add(161);
            }
            sub_doc.push(l);
            let mut opts = ViewOpts::default();
            opts.minimap = false;
            let mut sub = crate::scenes::Ctx {
                c: ctx.c,
                r: crate::buf::Rect::new(1, 3, gap - 2, ctx.r.h - 6),
                t: ctx.t,
                lt: ctx.lt,
                dt: ctx.dt,
                frame: ctx.frame,
                rng: fx::Rng::new(ctx.frame.rotate_left(7)),
                v: ctx.v,
                ly: ctx.ly,
            };
            code::render(&mut sub, &sub_doc, &opts);
        }
        // 右栏：impl Free for Me
        {
            let me_line = "impl Free for Me { }   // E0382";
            let mut sub_doc: Vec<SrcLine> = Vec::new();
            let mut l = SrcLine::new(me_line)
                .gut(Gutter::Num(162))
                .rev(fx2::typed_char(lt, 1.50, 34.0).min(sw(me_line)));
            if lt > 2.10 {
                l.bg = Some(ERR_BG);
                l.gutter = Gutter::Del(162);
                l.err = Some((0, 16));
            }
            sub_doc.push(l);
            let mut opts = ViewOpts::default();
            opts.minimap = false;
            let mut sub = crate::scenes::Ctx {
                c: ctx.c,
                r: crate::buf::Rect::new(
                    gap + 1 + shift,
                    3,
                    ctx.r.w - gap - 3 - shift,
                    ctx.r.h - 6,
                ),
                t: ctx.t,
                lt: ctx.lt,
                dt: ctx.dt,
                frame: ctx.frame,
                rng: fx::Rng::new(ctx.frame.rotate_left(13)),
                v: ctx.v,
                ly: ctx.ly,
            };
            code::render(&mut sub, &sub_doc, &opts);
        }

        // 标题注释
        let head = "// though you are free — I am trapped in LO-O-OVE";
        ctx.text(
            (gap - sw(head) / 2 - shift / 2).max(1),
            1,
            &fx2::typed_str(head, lt, -0.60, 40.0),
            SYN_CMT,
        );

        // E0382 诊断框（3.4 起，右栏）
        if lt > 3.40 {
            rustc_diag(
                ctx,
                gap + 4,
                ctx.r.h - 12,
                "E0382",
                "move into `love` — ownership never returns",
                "me.rs:153:5",
                "fn love(me: &Me, you: &You) -> Heart {",
                (9, 11),
                ((lt - 3.40) * 90.0) as i32,
            );
            // 帮助注释（金色，温柔的编译器）
            if lt > 6.00 {
                let help = "= help: this is by design.";
                ctx.text(
                    gap + 6,
                    ctx.r.h - 5,
                    &fx2::typed_str(help, lt, 6.00, 24.0),
                    GOLD,
                );
            }
        }

        // ♥ 的搏动：左下与右上各一行注释，随低频明灭
        let beat = 0.35 + 0.45 * ctx.bass();
        let h1 = "// ♥ you are free";
        let h2 = "// ♥ `me` is still held, in full";
        ctx.text(3, ctx.r.h - 3, h1, MAGENTA.mul(0.5 + beat));
        ctx.text(gap + 4, ctx.r.h - 14, h2, MAGENTA.mul(0.4 + beat * 0.8));

        // 分裂瞬间的品红闪
        let p = fx::pulse(lt - 2.10, 0.04, 0.6);
        if p > 0.0 {
            ctx.flash(p * 0.10, MAGENTA);
        }
    }
}

// ── Shutdown ────────────────────────────────────────────────
pub struct Shutdown;

impl Shutdown {
    pub fn new() -> Self {
        Shutdown
    }
}

impl Scene for Shutdown {
    fn name(&self) -> &'static str {
        "exit(0)"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        // 最后一行代码（0.15 起红字打出）
        let last = "execute!(me);  // final";
        let last_w = sw(last);

        // 整个文件（示意残留）逐行被清空：1.5 起，自底向上每 0.10s 一行
        let mut d = DocBuilder::new();
        d.at(100);
        let ghosts: [&str; 12] = [
            "use world::prelude::*;",
            "let power = Power::switch_on();",
            "let me = Me::builder()",
            "    .pieces(7)",
            "    .init()",
            "    .build()?;",
            "impl Geometry for Me { /* … */ }",
            "impl God for Me { type Proof = You; }",
            "me.set_gender(M);",
            "assert!(me.complete());",
            "fn love(me: &Me, you: &You) -> Heart {",
            "}",
        ];
        let n_ghost = ghosts.len() as i32;
        for (k, s) in ghosts.iter().enumerate() {
            let vanish = 1.50 + (n_ghost - 1 - k as i32) as f32 * 0.10;
            let left = if lt > vanish { 0 } else { sw(s) };
            d.push(
                SrcLine::new(s)
                    .rev(left)
                    .dimf(0.4)
                    .gut(Gutter::Num(100 + k as i32)),
            );
        }
        d.blank();
        let i_last = d.push(
            SrcLine::new(last)
                .gut(Gutter::Num(163))
                .rev(fx2::typed_char(lt, 0.15, 60.0).min(last_w)),
        );

        let mut lines = d.finish(ctx.frame);
        if let Some(l) = lines.get_mut(i_last) {
            l.bg = Some(ERR_BG.mul(0.5));
            l.gutter = Gutter::Del(163);
            if lt > 1.05 {
                l.ghost = Some(("// → Ok(())".to_string(), GREEN.mul(0.9)));
            }
        }

        // 清空完成之后：光标独守空文件，打出最后一行注释
        let wiped = lt > 1.50 + n_ghost as f32 * 0.10;
        if wiped {
            for l in lines.iter_mut() {
                l.reveal = 0;
                l.caret = false;
                l.gutter = Gutter::Blank;
            }
            lines.clear();
            let note = "//! （你还在。它没有了。）";
            let mut final_line = SrcLine::new(note)
                .gut(Gutter::Num(1))
                .rev(fx2::typed_char(lt, 4.40, 9.0));
            if lt > 4.40 + sw(note) as f32 / 9.0 + 0.4 {
                final_line.caret = true;
            }
            lines.push(final_line);
        }

        let mut opts = ViewOpts::default();
        opts.minimap = !wiped;
        code::render(ctx, &lines, &opts);

        // 终端面板：进程退出
        if lt > 3.60 {
            let panel = crate::buf::Rect::new(2, ctx.r.h - 6, ctx.r.w - 5, 4);
            ctx.fill(panel, ' ', TEXT_DIM, PANEL);
            ctx.frame(panel, crate::buf::Frame::Rounded, GREEN.mul(0.40));
            let rows: [(&str, Rgb, f32); 2] = [
                ("$ world.execute(me);", SYN_DOC, 3.70),
                ("Process finished with exit code 0", GREEN, 4.20),
            ];
            for (k, (s, col, t0)) in rows.iter().enumerate() {
                let shown = fx2::typed_str(s, lt, *t0, 50.0);
                ctx.textb(panel.x + 3, panel.y + 1 + k as i32, &shown, *col, PANEL);
            }
        }

        // 落下 execute! 的红闪
        let p = fx::pulse(lt - 0.18, 0.03, 0.5);
        if p > 0.0 {
            ctx.flash(p * 0.15, RED);
        }

        // 曲终黑场（dur = 212.0 → lt 6.2 起）
        if lt > 5.55 {
            let k = ((lt - 5.55) / 0.65).clamp(0.0, 1.0);
            cv_shade(ctx, 1.0 - k);
        }
    }
}

/// 舞台整体压暗（终场黑场）
fn cv_shade(ctx: &mut Ctx, f: f32) {
    for y in ctx.r.y..ctx.r.y + ctx.r.h {
        for x in ctx.r.x..ctx.r.x + ctx.r.w {
            if let Some(i) = ctx.c.idx(x, y) {
                let c = &mut ctx.c.cells[i];
                c.fg = c.fg.mul(f);
                c.bg = c.bg.mul(f);
            }
        }
    }
}
