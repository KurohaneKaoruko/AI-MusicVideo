//! 第三幕：有机段。
//!
//! Organic —— 三块 `impl Give<…>` 像 diff 新增行一样长出来（茄子/番茄/猫）。
//! Deity   —— `trait God`：`type Proof = You;` 在 EXISTENCE 唱点上被金光
//!            点亮，附带一次补全幽灵文本的"顿悟"。
//! Morph   —— 三次查找替换：F→M、AM→PM、S→M，替换条与扫过高亮。
//! Trance  —— 代码块随节拍折叠/展开，缩进参考线像催眠波。

use crate::code::{self, DocBuilder, Gutter, SrcLine, ViewOpts, card_popup, find_bar};
use crate::fx;
use crate::scenes::{Ctx, Scene};
use crate::theme::*;

// ── Organic ─────────────────────────────────────────────────
pub struct Organic;

impl Organic {
    pub fn new() -> Self {
        Organic
    }
}

impl Scene for Organic {
    fn name(&self) -> &'static str {
        "impl Give"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        let mut d = DocBuilder::new();
        d.fold_prev(54, "trapped · E0499 · the cage");
        d.dim_prev(
            &[
                "let cage = world.cage(&mut me);",
                "loop { cage.tighten(); }",
            ],
            55,
        );
        d.at(57);
        d.blank();

        // If I'm an eggplant — NUTRIENTS（74.05 / 75.42 / 76.96）
        d.cmt(
            lt,
            0.15,
            "// if I'm an eggplant — I will give you my NUTRIENTS",
            32.0,
        );
        let b0 = d.code(lt, 0.55, "impl Give<Nutrients> for Eggplant {", 36.0);
        d.code(
            lt,
            1.10,
            "    fn give(&self) -> Nutrients { ripen(0.87) }",
            40.0,
        );
        let e0 = d.code(lt, 1.50, "}", 40.0);

        // If I'm a tomato — ANTIOXIDANTS（77.58 / 79.23 / 80.62）
        d.cmt(
            lt,
            3.55,
            "// if I'm a tomato — I will give you ANTIOXIDANTS",
            32.0,
        );
        let b1 = d.code(lt, 3.95, "impl Give<Antioxidants> for Tomato {", 36.0);
        d.code(
            lt,
            4.50,
            "    fn give(&self) -> Antioxidants { ripen(0.92) }",
            40.0,
        );
        let e1 = d.code(lt, 4.90, "}", 40.0);

        // If I'm a tabby cat — ENJOYMENT（81.35 / 82.83 / 84.27 → 后半落在 Deity 段）
        d.cmt(
            lt,
            7.35,
            "// if I'm a tabby cat — I will purr for your ENJOYMENT",
            30.0,
        );
        let b2 = d.code(lt, 7.85, "impl Purr for TabbyCat {", 32.0);
        d.code(lt, 8.15, "    fn purr(&self) -> Sound { purr(8.5) }", 34.0);
        let e2 = d.code(lt, 8.45, "}", 34.0);

        let mut lines = d.finish(ctx.frame);

        // 每块完成即短暂绿底（像被 git 收下的新代码）
        for (b, e, t_done) in [(b0, e0, 2.10), (b1, e1, 5.50), (b2, e2, 8.50)] {
            let age = lt - t_done;
            if age > 0.0 && age < 0.9 {
                let a = fx::pulse(age, 0.05, 0.85);
                for i in b..=e {
                    if let Some(l) = lines.get_mut(i) {
                        l.bg = Some(ADD_BG.mul(0.4 + a));
                    }
                }
            }
        }
        // 全部带 + 装订线
        for i in [b0, b0 + 1, b0 + 2, b1, b1 + 1, b1 + 2, b2, b2 + 1, b2 + 2] {
            if let Some(l) = lines.get_mut(i) {
                let ln = match l.gutter {
                    Gutter::Num(v) => v,
                    _ => 0,
                };
                l.gutter = Gutter::Add(ln);
            }
        }

        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);
    }
}

// ── Deity ───────────────────────────────────────────────────
pub struct Deity;

impl Deity {
    pub fn new() -> Self {
        Deity
    }
}

impl Scene for Deity {
    fn name(&self) -> &'static str {
        "trait God"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        let mut d = DocBuilder::new();
        d.at(66);

        // 猫的尾巴：上一段的最后一块在这里打完，ENJOYMENT 唱点脉冲
        let cat_c = d.cmt(
            lt,
            -1.20,
            "// if I'm a tabby cat — I will purr for your ENJOYMENT",
            30.0,
        );
        d.code(lt, -0.55, "impl Purr for TabbyCat {", 32.0);
        d.code(lt, -0.15, "    fn purr(&self) -> Sound { purr(8.5) }", 34.0);
        let cat_e = d.code(lt, 0.25, "}", 34.0);

        let mut lines = d.finish(ctx.frame);
        // ENJOYMENT（84.27 → lt 1.68）脉冲
        let ep = fx::pulse(lt - 1.68, 0.05, 0.9);
        if ep > 0.0 {
            for i in cat_c..=cat_e {
                if let Some(l) = lines.get_mut(i) {
                    l.bg = Some(ADD_BG.mul(0.35 + ep * 0.6));
                }
            }
        }

        // ── trait God ──
        let mut d = DocBuilder {
            lines,
            caret_at: None,
            n: 70,
        };
        d.fold_prev(69, "organic · eggplant / tomato / tabby cat");
        d.at(70);
        d.blank();
        d.cmt(
            lt,
            2.50,
            "// if I'm the only god — then you're the proof of my EXISTENCE",
            30.0,
        );
        let t0 = d.code(lt, 0.35, "pub trait God {", 30.0);
        d.code(lt, 0.75, "    type Proof;", 30.0);
        d.code(lt, 1.15, "    fn exist(&self) -> Self::Proof;", 30.0);
        let t4 = d.code(lt, 1.45, "}", 40.0);
        let t5 = d.code(lt, 2.80, "impl God for Me {", 30.0);
        // 自动补全幽灵：先打 you，幽灵提示 .clone()，86.9 接受
        let acc = lt > 4.30;
        let line78_text = if acc {
            "    fn exist(&self) -> You { you.clone() }"
        } else {
            "    fn exist(&self) -> You { you"
        };
        let t6 = d.code(lt, 3.55, line78_text, 30.0);
        // type Proof = You; 在 EXISTENCE（87.92 → lt 5.33）落定
        let t7 = d.code(lt, 5.20, "    type Proof = You;", 34.0);
        let t8 = d.code(lt, 5.90, "}", 40.0);

        let mut lines = d.finish(ctx.frame);

        // 未接受补全时的幽灵文本
        if !acc && lt > 4.05 {
            if let Some(l) = lines.get_mut(t6) {
                l.ghost = Some((
                    ".clone()  puny_devotion()  worship()".to_string(),
                    SYN_GHOST,
                ));
            }
        }
        // 接受瞬间：白闪一下该行
        let accf = fx::pulse(lt - 4.30, 0.03, 0.4);
        if accf > 0.0 {
            if let Some(l) = lines.get_mut(t6) {
                l.bg = Some(PANEL_HI.mul(0.5 + accf));
            }
        }
        // EXISTENCE：`You` 金色点亮（span 指向 `= You;`）
        if lt > 5.28 {
            if let Some(l) = lines.get_mut(t7) {
                l.hl = Some((16, 20, HL_BG));
            }
        }
        // 编译期证明：神性在 const 求值里成立
        if lt > 6.00 {
            if let Some(l) = lines.get_mut(t7) {
                l.ghost = Some(("// ✓ compile-time".to_string(), GOLD));
            }
        }

        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);

        // q.e.d. 悬停卡
        if ctx.t > 88.05 {
            card_popup(
                ctx,
                ctx.r.w - 30,
                (t7 as i32 - opts.scroll).clamp(1, ctx.r.h - 4),
                "proof of existence",
                "You  ✓  q.e.d.",
                GOLD,
                ctx.t - 88.05,
            );
        }

        // EXISTENCE 唱点：金色脉冲
        let f = fx::pulse(lt - 5.33, 0.05, 0.8);
        if f > 0.0 {
            ctx.flash(f * 0.12, GOLD);
        }
        let _ = (t0, t4, t5, t8);
    }
}

// ── Morph ───────────────────────────────────────────────────
pub struct Morph;

impl Morph {
    pub fn new() -> Self {
        Morph
    }
}

impl Scene for Morph {
    fn name(&self) -> &'static str {
        "F → M"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        // 三次替换：(行文本, 替换时刻, 列区间)
        let g_swapped = lt > 1.35; // To F to M @ 90.197 → lt 0.97, 挥下 1.35
        let s_swapped = lt > 8.85; // To S to M @ 97.739 → lt 8.52
        let mut d = DocBuilder::new();
        d.fold_prev(79, "deity · type Proof = You");
        d.dim_prev(&["impl God for Me {", "    type Proof = You;", "}"], 76);
        d.at(80);
        d.blank();
        d.cmt(lt, -0.70, "// switch my gender — to F to M", 30.0);
        let i83 = d.code(
            lt,
            0.10,
            if g_swapped {
                "me.set_gender(M);"
            } else {
                "me.set_gender(F);"
            },
            30.0,
        );
        d.cmt(lt, 2.75, "// and then do whatever — from AM to PM", 30.0);
        let i85 = d.code(lt, 3.30, "schedule(AM, PM);", 30.0);
        d.cmt(lt, 6.25, "// oh switch my role — to S to M", 30.0);
        let i87 = d.code(
            lt,
            6.80,
            if s_swapped {
                "me.set_role(M);"
            } else {
                "me.set_role(S);"
            },
            30.0,
        );
        d.cmt(
            lt,
            10.20,
            "// so we can enter — the trance, the trance",
            30.0,
        );
        let i89 = d.code(lt, 10.90, "me.enter(Trance);", 30.0);

        let mut lines = d.finish(ctx.frame);

        // 替换扫过：目标字符琥珀 → 替换后绿
        if let Some(l) = lines.get_mut(i83) {
            if g_swapped {
                if lt < 1.80 {
                    l.hl = Some((13, 14, ADD_BG));
                }
            } else if lt > 0.90 {
                l.hl = Some((13, 14, HL_BG));
            }
        }
        if let Some(l) = lines.get_mut(i85) {
            // From AM to PM @93.953 → lt 4.73：两个 token 依次亮起
            if lt > 4.70 && lt < 5.60 {
                l.hl = Some((9, 15, HL_BG));
            } else if lt >= 5.60 {
                l.hl = Some((9, 15, SEL_BG));
            }
        }
        if let Some(l) = lines.get_mut(i87) {
            if s_swapped {
                if lt < 9.30 {
                    l.hl = Some((12, 13, ADD_BG));
                }
            } else if lt > 8.50 {
                l.hl = Some((12, 13, HL_BG));
            }
        }
        // trance 行呼应下一段
        if let Some(l) = lines.get_mut(i89) {
            l.bg = Some(PANEL_HI.mul(0.30 + 0.4 * ctx.bass()));
        }

        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);

        // 查找替换条随每次替换出现
        if lt > 0.90 && lt < 2.60 {
            find_bar(
                ctx,
                ctx.r.h - 2,
                "F",
                "M",
                (1, 1),
                ((lt - 0.90) * 40.0) as i32,
            );
        } else if lt > 8.50 && lt < 10.10 {
            find_bar(
                ctx,
                ctx.r.h - 2,
                "S",
                "M",
                (1, 1),
                ((lt - 8.50) * 40.0) as i32,
            );
        }

        // 替换落定的白闪
        let f1 = fx::pulse(lt - 1.35, 0.03, 0.30);
        let f2 = fx::pulse(lt - 8.85, 0.03, 0.30);
        let f = (f1 + f2).clamp(0.0, 1.0);
        if f > 0.0 {
            ctx.flash(f * 0.14, WHITE);
        }
    }
}

// ── Trance ──────────────────────────────────────────────────
pub struct Trance;

impl Trance {
    pub fn new() -> Self {
        Trance
    }
}

impl Scene for Trance {
    fn name(&self) -> &'static str {
        "trance{}"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        // 折叠催眠：2.6s 之后随慢正弦反复折叠/展开
        let folded = lt > 2.60 && ((lt * 0.9).sin() > 0.0);

        let mut d = DocBuilder::new();
        d.fold_prev(88, "morph · F to M · AM to PM · S to M");
        d.dim_prev(&["me.set_role(M);", "me.enter(Trance);"], 87);
        d.at(90);
        d.blank();
        if folded {
            d.push(
                SrcLine::new("while trance.depth < MAX { if me.feel(…) { depth += 1 } }")
                    .gut(Gutter::Fold)
                    .folded(6),
            );
        } else {
            let l1 = d.code(lt, 0.30, "while trance.depth < MAX {", 34.0);
            let _l2 = d.cmt(lt, 0.85, "    // if I can feel your VIBRATIONS", 34.0);
            let _l3 = d.code(lt, 2.00, "    if me.feel(you.vibrations()) {", 34.0);
            let _l4 = d.code_cmt(
                lt,
                4.90,
                "        trance.depth += 1;",
                34.0,
                5.30,
                &format!("// depth = {:.2}", 0.60 + 0.35 * ctx.bass()),
            );
            let l5 = d.code(lt, 5.60, "    }", 40.0);
            let l6 = d.code(lt, 5.80, "}", 40.0);
            let _ = (l1, l5, l6);
        }
        d.blank();
        let i95 = d.cmt(lt, 6.45, "// then I can finally be COMPLETION", 30.0);
        let assert_s = if lt > 8.72 {
            "assert!(me.complete());  // ✓"
        } else {
            "assert!(me.complete());"
        };
        let i96 = d.code(lt, 6.95, assert_s, 26.0);

        let mut lines = d.finish(ctx.frame);
        // 断言行随低频呼吸
        if let Some(l) = lines.get_mut(i96) {
            l.bg = Some(PANEL_HI.mul(0.22 + 0.5 * ctx.bass()));
            if lt > 8.72 {
                l.hl = Some((25, 27, ADD_BG));
            }
        }
        let _ = i95;

        let mut opts = ViewOpts::default();
        opts.minimap = !folded; // 折叠时让画面更空灵
        code::render(ctx, &lines, &opts);

        // COMPLETION：绿色确认闪
        let f = fx::pulse(lt - 8.72, 0.04, 0.6);
        if f > 0.0 {
            ctx.flash(f * 0.14, GREEN);
        }
    }
}
