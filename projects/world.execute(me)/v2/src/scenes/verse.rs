//! 第二幕：主歌。
//!
//! Geometry —— `impl Geometry for Me`：每个"如果我是…"唱句落下一条 match 臂，
//!             关键词唱到时右侧弹出悬停卡给出答案。
//! Electric  —— 查找替换把 AC 换成 DC；vision.blind() 让整个编辑器暗掉；
//!             年份倒流；两块 impl 合并成一个 `impl Me`。
//! Stimulus  —— 一个真的在跑的 loop，tick 计数与满足度仪表随低频攀升。
//! Trapped   —— borrow checker 出场：`&mut me` 被世界借走，笼子是一对
//!             正在合拢的花括号。

use crate::buf::{Rgb, sw};
use crate::code::{self, DocBuilder, Gutter, SrcLine, ViewOpts, find_bar, rustc_diag};
use crate::fx::{self};
use crate::fx2;
use crate::scenes::{Ctx, Scene};
use crate::theme::*;

// ── Geometry ────────────────────────────────────────────────
pub struct Geometry;

impl Geometry {
    pub fn new() -> Self {
        Geometry
    }
}

impl Scene for Geometry {
    fn name(&self) -> &'static str {
        "impl Geometry"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        let mut d = DocBuilder::new();
        d.fold_prev(18, "boot · object creation · simulation");
        d.at(22);
        d.blank();
        d.code(lt, 0.20, "impl Geometry for Me {", 30.0);
        d.code(lt, 0.90, "    fn describe(&self) -> Answer {", 30.0);
        d.code(lt, 1.50, "        match self.kind {", 30.0);

        // 四条 match 臂：歌词"如果我是…"逐条落下
        let arm0 = d.code_cmt(
            lt,
            2.30,
            "            Kind::Points(p) => give(p.dimension()),",
            34.0,
            2.97,
            "",
        );
        let arm1 = d.code_cmt(
            lt,
            3.90,
            "            Kind::Circle(c) => give(c.circumference()),",
            34.0,
            6.58,
            "",
        );
        let arm2 = d.code_cmt(
            lt,
            7.60,
            "            Kind::Sine(s)   => give(s.tangents()),",
            34.0,
            10.34,
            "",
        );
        let arm3 = d.code_cmt(
            lt,
            11.00,
            "            Kind::Infty(i)  => you.be_my_limit(),",
            34.0,
            13.80,
            "",
        );
        d.code(lt, 12.40, "        }", 20.0);
        d.code(lt, 12.60, "    }", 20.0);
        d.code(lt, 12.80, "}", 20.0);

        let mut lines = d.finish(ctx.frame);

        // 当前被唱到的臂：整行点亮
        let focus = if ctx.t < 33.41 {
            Some(arm0)
        } else if ctx.t < 37.07 {
            Some(arm1)
        } else if ctx.t < 40.71 {
            Some(arm2)
        } else {
            Some(arm3)
        };
        if let Some(i) = focus {
            if let Some(l) = lines.get_mut(i) {
                l.bg = Some(PANEL_HI.mul(0.55 + 0.25 * ctx.bass()));
            }
        }

        let opts = ViewOpts::default();
        // 悬停卡要画在代码右侧，先渲染文档再叠卡片
        code::render(ctx, &lines, &opts);

        // 关键词唱点 → 右侧悬停卡（答案）
        let cards: [(usize, f32, &str, &str, Rgb); 4] = [
            (arm0, 32.85, "DIMENSION", "= 3   // 一组点的维度", CYAN),
            (arm1, 36.45, "CIRCUMFERENCE", "= 2πr // 圆的周长", CYAN),
            (arm2, 40.20, "TANGENTS", "= tanθ × ∞ // 你可以坐上来", CYAN),
            (arm3, 43.65, "LIMITATIONS", "= ∞   // 你是我的极限", AMBER),
        ];
        for (row, t0, title, body, col) in cards {
            if ctx.t > t0 {
                let y = (row as i32 - opts.scroll).clamp(1, ctx.r.h - 4);
                let x = (ctx.r.w - 36).max(code::GUT_W + 46);
                code::card_popup(ctx, x, y, title, body, col, ctx.t - t0);
            }
        }
    }
}

// ── Electric ────────────────────────────────────────────────
pub struct Electric;

impl Electric {
    pub fn new() -> Self {
        Electric
    }
}

impl Scene for Electric {
    fn name(&self) -> &'static str {
        "AC → DC"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        // vision.blind()：49.2–50.6 整个编辑器暗下来
        let blind = fx::pulse(lt - 4.75, 0.25, 1.55).clamp(0.0, 1.0);
        let gdim = 1.0 - 0.62 * blind;

        let mut d = DocBuilder::new();
        d.fold_prev(30, "geometry of me · four answers");
        d.dim_prev(&["    }", "}", ""], 31);
        d.at(34);
        d.blank();

        // Switch my current — AC to DC（唱点 44.45 / 45.85）
        d.cmt(lt, 0.15, "// switch my current — AC to DC", 34.0);
        // 45.9 之前是 AC，之后被"全部替换"成 DC
        let swapped = lt > 1.45;
        let cur = if swapped {
            "let current = Current::DC;"
        } else {
            "let current = Current::AC;"
        };
        let i35 = d.code(lt, 0.60, cur, 30.0);

        // And then blind my vision（47.67）
        d.cmt(lt, 3.45, "// and then blind my vision", 30.0);
        let i_blind = d.code(lt, 4.05, "vision.blind();", 30.0);

        // So dizzy so dizzy（49.53）
        d.cmt(lt, 5.35, "// so dizzy, so dizzy", 30.0);
        d.code(lt, 6.10, "for spin in (0..).cycle() {", 30.0);
        let i40 = d.code_cmt(
            lt,
            6.50,
            "    me.dizzy(spin);",
            30.0,
            6.95,
            &format!(
                "// spin = {:04}",
                ((ctx.t - 50.8) * 31.0).max(0.0) as i32 % 10000
            ),
        );
        d.code(lt, 7.00, "}", 30.0);

        // We can travel — to A.D. to B.C.（51.36 / 53.23）
        d.cmt(lt, 7.35, "// we can travel — to A.D. to B.C.", 30.0);
        // 年份倒流：2026 → -300
        let tp = ((lt - 9.35) / 2.6).clamp(0.0, 1.0);
        let year = 2026.0 - 2326.0 * fx::smooth(tp);
        let year_s = format!("let year = {};", year.round() as i64);
        let i43 = d.code(lt, 9.05, &year_s, 40.0);

        // And we can unite（55.08 / 56.92）
        d.cmt(
            lt,
            11.00,
            "// and we can unite — so deeply, so deeply",
            30.0,
        );
        let ln45 = d.n;
        let i45 = d.code(lt, 12.95, "impl Me { /* human + machine */ }", 34.0);

        let mut lines = d.finish(ctx.frame);

        // AC/DC 替换动画：替换前琥珀高亮，替换后绿色
        if let Some(l) = lines.get_mut(i35) {
            if swapped {
                if lt < 1.95 {
                    l.hl = Some((23, 25, ADD_BG));
                }
            } else if lt > 1.45 {
                l.hl = Some((23, 25, HL_BG));
            }
        }
        // dizzy 行随节奏抖动
        if let Some(l) = lines.get_mut(i40) {
            l.dim = gdim * (0.85 + 0.3 * ctx.high());
        }
        // 年份行：倒流时白色发烫
        if let Some(l) = lines.get_mut(i43) {
            if tp > 0.0 && tp < 1.0 {
                l.hl = Some((11, sw(&year_s) - 1, SEL_BG));
            }
        }
        // vision.blind() 的运行时回显（49.65，正好在暗场里）
        if lt > 5.20 {
            if let Some(l) = lines.get_mut(i_blind) {
                l.ghost = Some(("// → brightness: 100% → 7%".to_string(), AMBER.mul(0.9)));
            }
        }
        // 合并行绿色 diff 底 + 回显
        if let Some(l) = lines.get_mut(i45) {
            if lt > 13.35 {
                l.bg = Some(ADD_BG);
                l.gutter = Gutter::Add(ln45);
            }
            if lt > 14.05 {
                l.ghost = Some(("// → Me(human + machine)".to_string(), GREEN.mul(0.8)));
            }
        }

        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);

        // 查找替换条（AC → DC）
        if lt > 1.45 && lt < 3.30 {
            find_bar(
                ctx,
                ctx.r.h - 2,
                "AC",
                "DC",
                (1, 1),
                ((lt - 1.45) * 40.0) as i32,
            );
        }

        // 合并动画：两行 impl 从两侧向中间滑（12.35–12.95）
        let mp = ((lt - 12.35) / 0.6).clamp(0.0, 1.0);
        if mp > 0.0 && mp < 1.0 {
            let cx = ctx.midx();
            let y1 = ctx.r.y + 12;
            let y2 = ctx.r.y + 13;
            let x1 = cx - 4 - ((1.0 - mp) * 22.0) as i32;
            let x2 = cx - 4 + ((1.0 - mp) * 22.0) as i32;
            ctx.text(
                x1 - sw("impl Human") - 2,
                y1,
                "impl Human ─┐",
                GREEN.mul(0.4 + 0.6 * mp),
            );
            ctx.text(x2 + 4, y2, "┌─ impl Machine", GREEN.mul(0.4 + 0.6 * mp));
        }
        let mf = fx::pulse(lt - 13.30, 0.04, 0.45);
        if mf > 0.0 {
            ctx.flash(mf * 0.18, GREEN);
        }

        // blind 的黑场闪
        if blind > 0.5 {
            ctx.flash(blind * 0.10, VOID);
        }
    }
}

// ── Stimulus ────────────────────────────────────────────────
pub struct Stimulus;

impl Stimulus {
    pub fn new() -> Self {
        Stimulus
    }
}

impl Scene for Stimulus {
    fn name(&self) -> &'static str {
        "stimulate()"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        let mut d = DocBuilder::new();
        d.fold_prev(44, "electric · AC to DC · unite");
        d.dim_prev(&["impl Me { /* human + machine */ }", ""], 45);
        d.at(47);
        d.blank();
        // If I can give you all the STIMULATIONS（59.22 / 59.69 / 61.96）
        d.cmt(lt, 0.20, "// if I can give you all the STIMULATIONS", 30.0);
        let i_if = d.code(lt, 1.00, "if me.give_all(&STIMULATIONS) {", 30.0);
        let i_body = d.code_cmt(
            lt,
            1.90,
            "    you.receive(me.stimulate());",
            30.0,
            2.45,
            &format!(
                "// tick {:04}",
                ((ctx.t - 61.9) * 24.0).max(0.0) as i32 % 10000
            ),
        );
        let i_close = d.code(lt, 2.90, "}", 40.0);
        // Then I can be your only SATISFACTION（62.59 / 63.54 / 65.40）
        d.cmt(lt, 3.40, "// then I can be your only SATISFACTION", 30.0);
        // 满足度仪表：63.2 → 64.6 冲到 100%
        let ratio = fx::smooth(((lt - 3.95) / 1.40).clamp(0.0, 1.0));
        let (g, _gc) = fx2::gauge(ratio, 14);
        let gauge_s = format!("// satisfaction: [{g}] {:02}%", (ratio * 100.0) as i32);
        let ln50 = d.n;
        let i50 = d.push(
            SrcLine::new(&gauge_s)
                .gut(Gutter::Num(ln50))
                .rev(sw(&gauge_s))
                .kwin(17, 30, Kw::Machine),
        );

        let mut lines = d.finish(ctx.frame);
        // 条件求值：→ true（61.6，比 "Then I can" 唱点早一拍——因果先于歌词发生）
        if lt > 2.40 {
            if let Some(l) = lines.get_mut(i_if) {
                l.ghost = Some(("// → true".to_string(), GREEN.mul(0.9)));
                l.bg = Some(ADD_BG.mul(0.30 + 0.30 * ctx.bass()));
            }
        }
        // 循环体行：低频脉冲底色
        if let Some(l) = lines.get_mut(i_body) {
            l.bg = Some(PANEL_HI.mul(0.30 + 0.55 * ctx.bass()));
        }
        let _ = i_close;
        // 满行时仪表变绿闪
        if ratio >= 0.999 {
            if let Some(l) = lines.get_mut(i50) {
                l.bg = Some(ADD_BG);
            }
        }

        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);

        // 100% 的一瞬：绿闪 + "ready"提示
        let f = fx::pulse(lt - 5.35, 0.04, 0.5);
        if f > 0.0 {
            ctx.flash(f * 0.15, GREEN);
            code::tooltip(
                ctx,
                ctx.midx() - 12,
                ctx.r.h - 2,
                "you.happy() == true ✓",
                GREEN,
                ((lt - 5.35) * 40.0) as i32,
            );
        }
    }
}

// ── Trapped ─────────────────────────────────────────────────
pub struct Trapped;

impl Trapped {
    pub fn new() -> Self {
        Trapped
    }
}

impl Scene for Trapped {
    fn name(&self) -> &'static str {
        "E0499 borrow"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        let mut d = DocBuilder::new();
        d.fold_prev(48, "stimulus · satisfaction 100%");
        d.dim_prev(&["// satisfaction: [▓▓▓▓▓▓▓▓▓▓▓▓▓▓] 100%", ""], 49);
        d.at(51);
        // If I can make you happy — I will run the EXECUTION（66.60 / 68.25 / 69.26）
        d.cmt(
            lt,
            0.30,
            "// if I can make you happy — I will run the EXECUTION",
            30.0,
        );
        let i52 = d.code(lt, 1.10, "if you.happy() { run(EXECUTION); }", 30.0);
        d.blank();

        // Though we are trapped — in this strange strange SIMULATION（70.08 / 71.76 / 73.17）
        d.cmt(
            lt,
            6.15,
            "// though we are trapped — in this strange strange SIMULATION",
            30.0,
        );
        let cage_src = "let cage = world.cage(&mut me);";
        let i55 = d.code(lt, 6.65, cage_src, 30.0);
        let _i56 = d.code(lt, 8.10, "loop { cage.tighten(); }", 30.0);

        let mut lines = d.finish(ctx.frame);

        // EXECUTION 芯片脉冲
        if let Some(l) = lines.get_mut(i52) {
            l.kw = Some((18, 27, Kw::Machine));
        }
        // 条件求值：→ true（68.9）——"你快乐"为真，所以处决执行
        if lt > 2.60 {
            if let Some(l) = lines.get_mut(i52) {
                l.ghost = Some(("// → true".to_string(), GREEN.mul(0.85)));
            }
        }
        // 71.4 起：&mut me 报错（借出去就收不回来）
        if ctx.t > 71.45 {
            if let Some(l) = lines.get_mut(i55) {
                l.err = Some((22, 30));
            }
        }

        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);

        // rustc 诊断框（71.7 起）
        if ctx.t > 71.70 {
            let w = ctx.r.w;
            rustc_diag(
                ctx,
                w.saturating_sub(64).max(2),
                ctx.r.h - 8,
                "E0499",
                "cannot borrow `me` as mutable twice",
                "me.rs:55:23",
                cage_src,
                (22, 30),
                ((ctx.t - 71.70) * 130.0) as i32,
            );
        }

        // 花括号笼子：从两侧合拢（72.6 → 74.0）
        let cp = ((lt - 8.55) / 1.40).clamp(0.0, 1.0);
        if cp > 0.0 {
            let inset = (1.0 + (1.0 - fx::smooth(cp)) * (ctx.r.w / 2 - 10) as f32) as i32;
            let a = 0.30 + 0.55 * cp + 0.2 * ctx.hit();
            let top = 1;
            let bot = ctx.r.h - 2;
            for y in top..=bot {
                ctx.put(inset, y, '{', RED.mul(a.min(1.0)));
                ctx.put(ctx.r.w - 1 - inset, y, '}', RED.mul(a.min(1.0)));
            }
            // 笼中提示
            if cp > 0.55 {
                let s = "the world still holds `me`";
                ctx.text(
                    ctx.midx() - sw(s) / 2,
                    ctx.r.h - 2,
                    s,
                    RED.mul(0.75 + 0.25 * ctx.bass()),
                );
            }
        }

        // SIMULATION 唱点：红脉冲
        let f = fx::pulse(lt - 9.12, 0.05, 0.55);
        if f > 0.0 {
            ctx.flash(f * 0.10, RED);
        }
    }
}
