//! 第四幕：失去。
//!
//! Abandon   —— 六次 "You have left"，`impl Life for Us` 的六行一次被
//!              diff 删除（红装订线 + `// you have left`），ISOLATION 落章。
//! Isolation —— 退格风暴把死掉的代码逐字符擦掉；`me.erase(Fragments)`。
//! Fragments —— 挑战神明得到 warning，ILLEGAL ARGUMENTS 得到 E0004。
//! Verdict   —— PROBLEMS 面板升起，rustc 式输出逐行宣判，最终 EXECUTE。

use crate::bigfont;
use crate::buf::{Rect, Rgb, sw};
use crate::code::{self, DocBuilder, Gutter, SrcLine, ViewOpts, rustc_diag};
use crate::fx;
use crate::fx2::{self, TokenRain};
use crate::scenes::{Ctx, Scene};
use crate::theme::*;

// ── Abandon ─────────────────────────────────────────────────
pub struct Abandon;

/// 六次 "You have left" 的相对时刻（110.90 / 112.20 / 113.10 / 114.18 / 114.92 / 115.78）
const LEFT: [f32; 6] = [0.05, 1.30, 2.20, 3.28, 4.02, 4.88];
/// ISOLATION（117.27）
const ISO: f32 = 6.37;

impl Abandon {
    pub fn new() -> Self {
        Abandon
    }
}

impl Scene for Abandon {
    fn name(&self) -> &'static str {
        "you have left"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        let body = [
            "    you.hold(&me.hand);",
            "    you.stay(me.side);",
            "    you.wait(me.return);",
            "    you.believe(me.words);",
            "    you.remind(me.name);",
            "    you.remember(me);",
        ];

        let mut d = DocBuilder::new();
        d.at(98);
        d.blank();
        let i_impl = d.code(lt, -0.5, "impl Life for Us {", 60.0);
        // 六行 you.*：从下往上依次被删除
        let mut body_idx = Vec::new();
        for (k, s) in body.iter().enumerate() {
            // 第 k 行的死亡时刻：LEFT 从后往前
            let die = LEFT[5 - k];
            let gone = lt > die;
            let text = if gone {
                format!("{s:<28}// you have left")
            } else {
                s.to_string()
            };
            let i = d.push(
                SrcLine::new(&text)
                    .rev(sw(&text))
                    .dimf(if gone { 0.55 } else { 1.0 }),
            );
            body_idx.push(i);
        }
        let i_close = d.code(lt, -0.3, "}", 60.0);
        let i_state = d.code_cmt(
            lt,
            6.50,
            "let state = State::Isolation;",
            24.0,
            6.50,
            "// …",
        );

        let mut lines = d.finish(ctx.frame);

        // 应用删除样式
        for (k, &i) in body_idx.iter().enumerate() {
            let die = LEFT[5 - k];
            if lt > die {
                if let Some(l) = lines.get_mut(i) {
                    l.bg = Some(DEL_BG);
                    l.gutter = Gutter::Del(l.n_gutter());
                }
            }
        }
        // ISOLATION：impl 壳也死了
        if lt > ISO {
            for &i in &[i_impl, i_close] {
                if let Some(l) = lines.get_mut(i) {
                    l.bg = Some(DEL_BG);
                    l.gutter = Gutter::Del(l.n_gutter());
                }
            }
            if let Some(l) = lines.get_mut(i_state) {
                l.kw = Some((19, 28, Kw::Negative));
                l.bg = Some(ERR_BG.mul(0.7));
            }
        }

        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);

        // 每次删除：红闪 + 行涟漪
        for (j, &die) in LEFT.iter().enumerate() {
            let p = fx::pulse(lt - die - 0.02, 0.03, 0.45);
            if p > 0.0 {
                ctx.flash(p * (0.10 + 0.015 * j as f32), RED);
                fx2::ripple(ctx.c, ctx.r.y + 4 + j as i32, 2.0, 6.0, 2.0 * p);
            }
        }
        let fp = fx::pulse(lt - ISO, 0.04, 0.7);
        if fp > 0.0 {
            ctx.flash(fp * 0.16, RED);
        }
    }
}

// SrcLine 的小工具：读出装订线里的行号
impl SrcLine {
    pub fn n_gutter(&self) -> i32 {
        match self.gutter {
            Gutter::Num(v) | Gutter::Dirty(v) | Gutter::Add(v) | Gutter::Del(v) => v,
            _ => 0,
        }
    }
}

// ── Isolation ───────────────────────────────────────────────
pub struct Isolation;

impl Isolation {
    pub fn new() -> Self {
        Isolation
    }
}

impl Scene for Isolation {
    fn name(&self) -> &'static str {
        "E0425 'you'"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        let body = [
            "    you.hold(&me.hand);",
            "    you.stay(me.side);",
            "    you.wait(me.return);",
            "    you.believe(me.words);",
            "    you.remind(me.name);",
            "    you.remember(me);",
        ];

        let mut d = DocBuilder::new();
        d.at(98);
        d.blank();
        d.code(lt, -1.0, "impl Life for Us {", 60.0);
        // 退格风暴：每行从 0.3 + k*0.35 开始被逐字符擦掉
        for (k, s) in body.iter().enumerate() {
            let t0 = 0.30 + k as f32 * 0.35;
            let left = (sw(s) - fx2::typed_char(lt, t0, 42.0)).max(0);
            let i = d.push(
                SrcLine::new(s)
                    .rev(left)
                    .dimf(if left == 0 { 0.25 } else { 0.6 })
                    .gut(Gutter::Del(100 + k as i32)),
            );
            // 擦除时红 everywhere
            if left > 0 && lt > t0 {
                if let Some(l) = d.lines.get_mut(i) {
                    l.err = Some((0, left));
                }
            }
        }
        d.code(lt, -1.0, "}", 60.0);
        d.blank();

        // If I can erase all the pointless FRAGMENTS（118.98 / 120.86）
        d.cmt(
            lt,
            0.65,
            "// if I can erase all the pointless FRAGMENTS",
            30.0,
        );
        let i_er = d.code(
            lt,
            1.85,
            "let erased = me.erase_all(Fragments::pointless());",
            30.0,
        );
        let _i_if6 = d.code(lt, 4.10, "if erased == 6 {", 30.0);
        // Then maybe you won't leave me so DISHEARTENED（122.71 / 124.89）
        d.cmt(
            lt,
            4.55,
            "    // then maybe — you won't leave me so DISHEARTENED",
            28.0,
        );
        d.code(lt, 6.00, "    me.mood = Mood::Disheartened;", 26.0);
        d.code(lt, 7.00, "}", 40.0);

        let mut lines = d.finish(ctx.frame);
        // 擦除进度回显：退格风暴每吞掉一行，计数 +1（与上方死块严格同步）
        if lt > 2.00 {
            let k = ((((lt - 0.30) / 0.35).floor() + 1.0).clamp(0.0, 6.0)) as i32;
            if let Some(l) = lines.get_mut(i_er) {
                let col = if k >= 6 {
                    GREEN.mul(0.9)
                } else {
                    AMBER.mul(0.9)
                };
                l.ghost = Some((format!("// erased = {k}/6"), col));
            }
        }
        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);

        // FRAGMENTS 唱点：白噪一下（代码被擦除的静电）
        let f = fx::pulse(lt - 2.53, 0.05, 0.8);
        if f > 0.0 {
            ctx.flash(f * 0.08, TEXT_DIM);
        }
        let f2 = fx::pulse(lt - 6.56, 0.05, 0.6);
        if f2 > 0.0 {
            ctx.flash(f2 * 0.08, RED);
        }
    }
}

// ── Fragments ───────────────────────────────────────────────
pub struct Fragments {
    rain: Option<TokenRain>,
}

impl Fragments {
    pub fn new() -> Self {
        Fragments { rain: None }
    }
}

impl Scene for Fragments {
    fn name(&self) -> &'static str {
        "illegal arguments"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        // 被擦下来的碎片：暗红 token 雨
        if self.rain.is_none() {
            self.rain = Some(TokenRain::new(ctx.r.w, ctx.r.h, &mut ctx.rng));
        }
        let r = ctx.bounds();
        if let Some(rain) = &mut self.rain {
            rain.draw(ctx.c, r, ctx.dt, RED.mul(0.7), 0.10 + 0.10 * ctx.bass());
        }

        let mut d = DocBuilder::new();
        d.fold_prev(110, "isolation · me.erase(Fragments)");
        d.dim_prev(&["me.mood = Mood::Disheartened;"], 110);
        d.at(112);
        d.blank();
        // Challenging your god（125.71）
        d.cmt(lt, 0.10, "// challenging your god", 30.0);
        let i_ch = d.code(lt, 0.45, "me.challenge(god);", 28.0);
        d.cmt(
            lt,
            1.70,
            "// warning: no god found in scope — challenge unsafe",
            26.0,
        );
        // You have made some ILLEGAL ARGUMENTS（128.66 / 131.22）
        d.cmt(lt, 3.30, "// you have made some ILLEGAL ARGUMENTS", 28.0);
        let i_ill = d.code(lt, 3.95, "you.make(IllegalArguments::new());", 28.0);
        d.code_cmt(lt, 5.55, "panic!(ILLEGAL_ARGUMENTS);", 26.0, 5.55, "// ×12");

        let mut lines = d.finish(ctx.frame);

        // challenge 行：琥珀警告（125.7+1.2 起）
        if lt > 1.20 {
            if let Some(l) = lines.get_mut(i_ch) {
                l.hl = Some((3, 12, HL_BG));
            }
        }
        // ILLEGAL ARGUMENTS：红错（131.22 → lt 5.51）
        if lt > 5.51 {
            if let Some(l) = lines.get_mut(i_ill) {
                l.err = Some((9, 27));
            }
        }

        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);

        // E0004 诊断框
        if lt > 5.80 {
            rustc_diag(
                ctx,
                ctx.r.w - 50,
                ctx.r.h - 9,
                "E0004",
                "illegal arguments: 12 violations",
                "me.rs:115:10",
                "you.make(IllegalArguments::new());",
                (9, 27),
                ((lt - 5.80) * 90.0) as i32,
            );
        }

        // 唱点闪
        let f1 = fx::pulse(lt - 0.02, 0.04, 0.4);
        if f1 > 0.0 {
            ctx.flash(f1 * 0.08, AMBER);
        }
        let f2 = fx::pulse(lt - 5.51, 0.04, 0.6);
        if f2 > 0.0 {
            ctx.flash(f2 * 0.14, RED);
        }
    }
}

// ── Verdict ─────────────────────────────────────────────────
pub struct Verdict;

impl Verdict {
    pub fn new() -> Self {
        Verdict
    }
}

impl Scene for Verdict {
    fn name(&self) -> &'static str {
        "DIAGNOSTICS"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        // 冻结的旧文档（压暗 0.32）：上一幕的三行 + 死亡的 impl 壳
        let mut d = DocBuilder::new();
        d.at(111);
        d.push(SrcLine::new("").dimf(0.3));
        d.push(
            SrcLine::new("me.challenge(god);")
                .dimf(0.3)
                .gut(Gutter::Num(113)),
        );
        d.push(
            SrcLine::new("you.make(IllegalArguments::new());")
                .dimf(0.3)
                .gut(Gutter::Num(115)),
        );
        d.push(
            SrcLine::new("panic!(ILLEGAL_ARGUMENTS);  // ×12")
                .dimf(0.3)
                .gut(Gutter::Num(116)),
        );
        d.push(SrcLine::new("").dimf(0.3));
        let doc = d.finish(ctx.frame);
        let opts = ViewOpts::default();
        code::render(ctx, &doc, &opts);

        // ── PROBLEMS / cargo check 面板（下半）──
        let py = ctx.r.h / 2 - 1;
        let panel = Rect::new(2, py, ctx.r.w - 5, ctx.r.h - py - 2);
        ctx.fill(panel, ' ', TEXT_DIM, PANEL);
        ctx.frame(panel, crate::buf::Frame::Rounded, RED.mul(0.55));
        ctx.textb(panel.x + 3, panel.y, " PROBLEMS ", RED, PANEL);

        let rows: [(&str, Rgb, f32); 10] = [
            ("$ cargo check --me", SYN_DOC, 0.50),
            ("    Checking me v0.0.1 (~/me)", GUTTER, 1.30),
            (
                "error[E0277]: the trait bound `Me: God` is not satisfied",
                ERR_SQ,
                2.30,
            ),
            ("   --> me.rs:76:5", SYN_DOC, 3.05),
            ("warning: unused variable: `heart`", AMBER, 4.10),
            ("   --> me.rs:1:9", SYN_DOC, 4.75),
            (
                "error[E0909]: `you` has left; `me` cannot recover",
                ERR_SQ,
                5.90,
            ),
            ("   = help: there is no recovery path", SYN_DOC, 6.70),
            ("verdict: 12 × EXECUTION approved", RED, 8.20),
            ("build failed. awaiting sentence…", TEXT_DIM, 9.30),
        ];
        for (k, (s, col, t0)) in rows.iter().enumerate() {
            let shown = fx2::typed_str(s, lt, *t0, 60.0);
            if shown.is_empty() {
                continue;
            }
            let y = panel.y + 1 + k as i32;
            if y >= panel.bottom() {
                break;
            }
            // 错误行落下的瞬间：整面板抖一下
            let sh = fx::pulse(lt - *t0, 0.02, 0.18);
            let dx = if sh > 0.3 {
                ((ctx.frame % 2) as i32) - 1
            } else {
                0
            };
            ctx.textb(panel.x + 4 + dx, y, &shown, *col, PANEL);
            if s.starts_with("error") {
                let p = fx::pulse(lt - *t0 - 0.05, 0.02, 0.35);
                if p > 0.0 {
                    ctx.flash(p * 0.10, RED);
                }
            }
        }

        // 终审落章：EXECUTE（146.4 → lt 13.10）
        let stamp = lt > 13.10;
        if stamp {
            let age = lt - 13.10;
            let a = fx::pulse(age, 0.05, 2.0).clamp(0.15, 1.0);
            let s = "EXECUTE";
            let _w = bigfont::width(s, 1);
            bigfont::center_glow(
                ctx.c,
                ctx.r.x + ctx.midx(),
                ctx.r.y + 3,
                s,
                RED.mul(a),
                RED.mul(0.5 * a),
                BG,
                1,
                0.8,
            );
            let sub = "// by the court of rustc — effective immediately";
            let shown = fx2::typed_str(sub, age, 0.25, 50.0);
            ctx.text(
                ctx.midx() - sw(sub) / 2,
                ctx.r.y + 10,
                &shown,
                RED.mul(0.85),
            );
            // 落章冲击
            let p = fx::pulse(age, 0.04, 0.5);
            if p > 0.0 {
                fx2::ripple(ctx.c, ctx.r.y + 5, 3.0, 10.0, 4.0 * p);
                ctx.flash(p * 0.20, RED);
            }
        }
        let _ = sw("");
    }
}
