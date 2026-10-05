//! 第五幕：处刑。
//!
//! Barrage   —— 12 拍里 12 行 `execute!(me);` 被砸进文件，每行一枚红章；
//!              六国计数是六行彩色注释；终章一声 full-width 红。
//! FinalExec —— 副歌二段变成一次 cargo build：警告 12 条未决处刑，
//!              `you` 仍然找不到，编译失败。

use crate::buf::{Rgb, sw};
use crate::code::{self, DocBuilder, Gutter, SrcLine, ViewOpts};
use crate::fx;
use crate::fx2::{self, TokenRain};
use crate::scenes::{Ctx, Scene};
use crate::theme::{self, *};

// ── Barrage ─────────────────────────────────────────────────
pub struct Barrage {
    rain: Option<TokenRain>,
}

/// 12 次 EXECUTION 的相对时刻
const STAMPS: [f32; 12] = [
    0.00, 0.94, 1.86, 2.88, 3.86, 4.62, 5.50, 6.32, 7.54, 8.42, 9.38, 10.34,
];
/// EIN DOS TROIS NE FEM LIU
const COUNT_IN: [(&str, f32); 6] = [
    ("ein", 11.24),
    ("dos", 11.66),
    ("trois", 12.00),
    ("ne", 12.58),
    ("fem", 13.03),
    ("liu", 13.46),
];
const FINAL: f32 = 13.92;

impl Barrage {
    pub fn new() -> Self {
        Barrage { rain: None }
    }
}

impl Scene for Barrage {
    fn name(&self) -> &'static str {
        "execute! ×12"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        if self.rain.is_none() {
            self.rain = Some(TokenRain::new(ctx.r.w, ctx.r.h, &mut ctx.rng));
        }
        let r = ctx.bounds();
        if let Some(rain) = &mut self.rain {
            rain.draw(ctx.c, r, ctx.dt, RED, 0.14 + 0.16 * ctx.bass());
        }

        let mut d = DocBuilder::new();
        d.fold_prev(117, "verdict · 12 diagnostics · build failed");
        d.dim_prev(&["panic!(ILLEGAL_ARGUMENTS);  // ×12"], 117);
        d.at(118);
        let mut stamp_rows: Vec<(usize, f32)> = Vec::new();
        for (k, &t0) in STAMPS.iter().enumerate() {
            let text = format!("execute!(me);  // EXECUTION x{}", k + 1);
            let i = d.code(lt, t0, &text, 110.0);
            stamp_rows.push((i, t0));
        }
        d.blank();
        let mut count_rows: Vec<(usize, f32, usize)> = Vec::new();
        for (k, (word, t0)) in COUNT_IN.iter().enumerate() {
            let s = format!("// {word}");
            let i = d.push(SrcLine::new(&s).rev(fx2::typed_char(lt, *t0, 26.0).min(sw(&s))));
            count_rows.push((i, *t0, k));
        }

        let mut lines = d.finish(ctx.frame);

        // 每枚印章：落下后红底脉冲（衰减为余温）
        for (i, t0) in &stamp_rows {
            let age = lt - t0;
            if age > 0.0 {
                if let Some(l) = lines.get_mut(*i) {
                    let a = fx::pulse(age, 0.04, 0.65);
                    l.bg = Some(ERR_BG.mul(0.45 + 0.55 * a + 0.25 * ctx.hit()));
                    let n = l.n_gutter();
                    l.gutter = Gutter::Del(n);
                }
            }
        }
        // 六国计数：各自一色（冷→暖）
        for (_i, t0, k) in &count_rows {
            if lt > *t0 {
                if let Some(l) = lines.get_mut(*_i) {
                    let col = theme::heat(*k as f32 / 5.0);
                    l.hl = Some((0, sw(&l.text), col.mul(0.5)));
                }
            }
        }

        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);

        // 每枚印章的小冲击波
        for (i, t0) in &stamp_rows {
            let p = fx::pulse(lt - *t0 - 0.03, 0.02, 0.30);
            if p > 0.0 {
                fx2::ripple(
                    ctx.c,
                    ctx.r.y + (*i as i32).clamp(1, ctx.r.h - 2),
                    2.0,
                    7.0,
                    3.0 * p,
                );
                ctx.flash(p * 0.07, RED);
            }
        }
        // 六国计数的轻脉冲
        for (_i, t0, k) in &count_rows {
            let p = fx::pulse(lt - *t0, 0.02, 0.30);
            if p > 0.0 {
                let col = theme::heat(*k as f32 / 5.0);
                ctx.flash(p * 0.05, col);
            }
        }

        // 终章：全宽红
        if lt > FINAL {
            let age = lt - FINAL;
            let s = "execute!(me);  // FINAL EXECUTION";
            code::banner(ctx, ctx.midy() + 2, s, RED, (age * 80.0) as i32, ctx.frame);
            let p = fx::pulse(age, 0.03, 0.6);
            if p > 0.0 {
                fx2::ripple(ctx.c, ctx.midy() + 2, 4.0, 14.0, 5.0 * p);
                ctx.flash(p * 0.22, RED);
            }
        }
    }
}

// ── FinalExec ───────────────────────────────────────────────
pub struct FinalExec;

impl FinalExec {
    pub fn new() -> Self {
        FinalExec
    }
}

impl Scene for FinalExec {
    fn name(&self) -> &'static str {
        "cargo build"
    }

    fn draw(&mut self, ctx: &mut Ctx) {
        let lt = ctx.lt;
        ctx.clear(BG);

        let mut d = DocBuilder::new();
        d.fold_prev(136, "barrage · execute! ×12 · ein dos trois ne fem liu");
        d.dim_prev(&["// liu", "execute!(me);  // EXECUTION x12"], 135);
        d.at(138);
        // If I can give them all the EXECUTION（163.32 / 165.17）
        d.cmt(lt, 0.10, "// if I can give them all the EXECUTION", 30.0);
        let i138 = d.code(lt, 0.90, "if me.give_them_all(EXECUTION) {", 30.0);
        // Then I can be your only EXECUTION（167.02 / 168.91）
        d.cmt(lt, 3.40, "    // then I can be your only EXECUTION", 30.0);
        let i140 = d.code(lt, 4.90, "    me.become(EXECUTION);", 30.0);
        d.code(lt, 6.30, "}", 40.0);
        // If I can have you back（169.82）
        d.cmt(lt, 7.20, "// if I can have you back", 30.0);
        let i_match = d.code(lt, 8.00, "match you.back() {", 30.0);
        let i_some = d.code(lt, 8.90, "    Some(you) => run(EXECUTION),", 30.0);
        let i_none = d.code(lt, 9.60, "    None      => me.wait(),", 30.0);
        d.code(lt, 10.20, "}", 40.0);
        // Though we are trapped（173.64 / 174.98）
        let i143 = d.cmt(
            lt,
            11.05,
            "// though we are trapped — we are trapped, ah",
            30.0,
        );
        d.cmt(lt, 12.60, "// (the cage compiles fine)", 26.0);

        let mut lines = d.finish(ctx.frame);

        // EXECUTION 分支被求值为真：整块随低频呼吸
        for &i in &[i138, i140] {
            if let Some(l) = lines.get_mut(i) {
                l.bg = Some(PANEL_HI.mul(0.25 + 0.45 * ctx.bass()));
            }
        }
        // 求值回显：become 的返回值 / you.back() 的结果
        if lt > 6.60 {
            if let Some(l) = lines.get_mut(i140) {
                l.ghost = Some(("// → Ok(EXECUTION)".to_string(), GREEN.mul(0.8)));
            }
        }
        if lt > 10.45 {
            // Some 臂永远走不到：字面意义上的"灰下去"
            if let Some(l) = lines.get_mut(i_some) {
                l.dim = 0.40;
            }
            if let Some(l) = lines.get_mut(i_none) {
                l.ghost = Some(("// → None".to_string(), ERR_SQ));
                l.bg = Some(ERR_BG.mul(0.55));
            }
        }
        // `you` 仍然找不到（E0425 的回声，171.2 起）
        if lt > 9.40 {
            if let Some(l) = lines.get_mut(i_match) {
                l.err = Some((6, 9));
            }
        }
        // trapped 唱点：整段红脉冲
        let tp = fx::pulse(lt - 11.01, 0.05, 1.2);
        if tp > 0.0 {
            if let Some(l) = lines.get_mut(i143) {
                l.bg = Some(ERR_BG.mul(0.35 * tp));
            }
        }

        let opts = ViewOpts::default();
        code::render(ctx, &lines, &opts);

        // ── cargo build 面板 ──
        let rows: [(&str, Rgb, f32); 5] = [
            ("$ cargo build --features love", SYN_DOC, 1.00),
            ("   Compiling me v0.0.1 (~/me)", GUTTER, 2.00),
            ("warning: 12 executions remain unresolved", AMBER, 6.40),
            ("error: could not compile `me`", ERR_SQ, 9.60),
            (
                "         `you` not found in this scope",
                ERR_SQ.mul(0.8),
                10.30,
            ),
        ];
        let py = ctx.r.h - 7;
        let panel = crate::buf::Rect::new(2, py, ctx.r.w - 5, 6);
        ctx.fill(panel, ' ', TEXT_DIM, PANEL);
        ctx.frame(panel, crate::buf::Frame::Rounded, RED.mul(0.45));
        for (k, (s, col, t0)) in rows.iter().enumerate() {
            let shown = fx2::typed_str(s, lt, *t0, 60.0);
            if shown.is_empty() {
                continue;
            }
            ctx.textb(panel.x + 3, panel.y + 1 + k as i32, &shown, *col, PANEL);
            let p = fx::pulse(lt - *t0, 0.02, 0.3);
            if p > 0.0 && s.starts_with("error") {
                ctx.flash(p * 0.10, RED);
            }
        }
    }
}
