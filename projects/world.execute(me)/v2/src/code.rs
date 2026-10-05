//! 源码文档引擎：v2 的"演员"。
//!
//! 场景不再画图，而是编排一个 `.rs` 文件：每帧把 `Vec<SrcLine>` 交给
//! `render`，由它完成语法高亮 / 打字光标 / 行号装订线 / 错误与查找高亮 /
//! 折叠 / 补全幽灵文本。再配合 `rustc_diag`（rustc 诊断框）、`card`（悬停
//! 卡）、`find_bar`（查找替换条）等编辑器部件，整部 MV 就是一次编码现场。
//!
//! 所有动画都是 (lt, t0) 的纯函数——离屏渲染零预热、可复现。

// 编辑器部件按幕取用，保留完整 API 面
#![allow(dead_code)]

use crate::buf::{Rgb, cw, sw};
use crate::scenes::Ctx;
use crate::theme::{self, *};

// ── 语法高亮 ────────────────────────────────────────────────
// 一个只为本 MV 服务的 mini Rust 高亮器：不追求完备，追求"一眼就是 IDE"。

pub struct Run {
    pub s: String,
    pub col: Rgb,
}

const KW1: &[&str] = &[
    "fn", "let", "mut", "struct", "impl", "trait", "for", "while", "loop", "if", "else", "match",
    "return", "pub", "use", "mod", "in", "as", "where", "ref", "move", "dyn", "unsafe", "const",
    "static", "type", "enum", "break", "continue", "async", "await", "true", "false",
];
const KW2: &[&str] = &["self", "Self", "crate", "super"];

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// 高亮一行 Rust 源码。注释里的中文等宽字符不受影响（按字符切 run）。
pub fn highlight(src: &str) -> Vec<Run> {
    let cs: Vec<char> = src.chars().collect();
    let n = cs.len();
    let mut runs: Vec<Run> = Vec::new();
    let push = |runs: &mut Vec<Run>, s: String, col: Rgb| {
        if s.is_empty() {
            return;
        }
        if let Some(last) = runs.last_mut() {
            if last.col == col {
                last.s.push_str(&s);
                return;
            }
        }
        runs.push(Run { s, col });
    };

    // 注释：本行内第一个不在字符串里的 `//` 之后全部是注释
    let mut in_str = false;
    let mut cmt_at: Option<usize> = None;
    let mut k = 0;
    while k < n {
        match cs[k] {
            '"' => in_str = !in_str,
            '\\' if in_str => k += 1,
            '/' if !in_str && k + 1 < n && cs[k + 1] == '/' => {
                cmt_at = Some(k);
                break;
            }
            _ => {}
        }
        k += 1;
    }

    let mut i = 0;
    let code_end = cmt_at.unwrap_or(n);
    while i < n {
        if i == code_end {
            let rest: String = cs[i..].iter().collect();
            let col = if rest.starts_with("///") || rest.starts_with("//!") {
                SYN_DOC
            } else {
                SYN_CMT
            };
            push(&mut runs, rest, col);
            break;
        }
        let c = cs[i];
        match c {
            '"' => {
                let mut j = i + 1;
                while j < code_end {
                    if cs[j] == '\\' {
                        j += 2;
                        continue;
                    }
                    if cs[j] == '"' {
                        j += 1;
                        break;
                    }
                    j += 1;
                }
                let s: String = cs[i..j.min(n)].iter().collect();
                push(&mut runs, s, SYN_STR);
                i = j;
            }
            '#' => {
                // #[…] / #![…] 属性
                let mut j = i + 1;
                if j < code_end && cs[j] == '!' {
                    j += 1;
                }
                if j < code_end && cs[j] == '[' {
                    while j < code_end && cs[j] != ']' {
                        j += 1;
                    }
                    j = (j + 1).min(code_end);
                }
                let s: String = cs[i..j].iter().collect();
                push(&mut runs, s, SYN_ATTR);
                i = j;
            }
            '\'' => {
                // 生命周期 'a（本 MV 不用字符字面量）
                let mut j = i + 1;
                while j < code_end && is_word(cs[j]) {
                    j += 1;
                }
                if j == i + 1 {
                    j += 1; // 孤立引号，按标点处理
                    push(&mut runs, "'".into(), SYN_PUNCT);
                } else {
                    let s: String = cs[i..j].iter().collect();
                    push(&mut runs, s, SYN_LIFE);
                }
                i = j;
            }
            c if c.is_ascii_digit() => {
                let mut j = i;
                while j < code_end
                    && (cs[j].is_ascii_alphanumeric() || cs[j] == '.' || cs[j] == '_')
                {
                    j += 1;
                }
                let s: String = cs[i..j].iter().collect();
                push(&mut runs, s, SYN_NUM);
                i = j;
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut j = i;
                while j < code_end && is_word(cs[j]) {
                    j += 1;
                }
                let w: String = cs[i..j].iter().collect();
                let col = if KW2.contains(&w.as_str()) {
                    SYN_KW2
                } else if KW1.contains(&w.as_str()) {
                    SYN_KW
                } else if j < code_end && cs[j] == '!' {
                    SYN_MACRO
                } else if w.chars().next().unwrap().is_ascii_uppercase() {
                    SYN_TY
                } else if cs[j..code_end].iter().any(|&c| c != ' ') == false {
                    SYN_IDENT
                } else {
                    let rest: String = cs[j..code_end].iter().collect();
                    if rest.trim_start().starts_with('(') {
                        SYN_FN
                    } else {
                        SYN_IDENT
                    }
                };
                push(&mut runs, w, col);
                i = j;
            }
            _ => {
                // 运算符 / 标点
                let two: String = cs[i..(i + 2).min(code_end)].iter().collect();
                let op2 = [
                    "->", "=>", "::", "==", "!=", "<=", ">=", "+=", "-=", "..", "&&", "||", "..=",
                    "*=", "/=",
                ]
                .contains(&two.as_str());
                let len = if op2 { 2 } else { 1 };
                let s: String = cs[i..(i + len).min(code_end)].iter().collect();
                let col = if op2 {
                    SYN_OP
                } else if "+-*/%=<>!&|^".contains(c) {
                    SYN_OP
                } else {
                    SYN_PUNCT
                };
                push(&mut runs, s, col);
                i += len;
            }
        }
    }
    runs
}

// ── 行模型 ──────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Gutter {
    /// 普通行号
    Num(i32),
    /// 行号 + 未保存圆点（amber）
    Dirty(i32),
    /// diff 新增 + 行号
    Add(i32),
    /// diff 删除 + 行号（整行建议配 DEL_BG）
    Del(i32),
    /// 折叠占位（无行号）
    Fold,
    /// 空装订线（无行号的续行/输出行）
    Blank,
}

#[derive(Clone)]
pub struct SrcLine {
    pub text: String,
    /// 可见的显示宽度；i32::MAX = 全部；≤0 只画行号
    pub reveal: i32,
    /// 在已显示文本末尾画光标
    pub caret: bool,
    pub gutter: Gutter,
    /// 整行底色
    pub bg: Option<Rgb>,
    /// 整行变暗（0..1，1 = 正常）
    pub dim: f32,
    /// 错误区间（显示列 [a,b)）：红字 + 红底
    pub err: Option<(i32, i32)>,
    /// 查找/替换高亮区间
    pub hl: Option<(i32, i32, Rgb)>,
    /// 光标后的补全幽灵文本
    pub ghost: Option<(String, Rgb)>,
    /// Some(n) = 折叠行，显示 "… n lines"
    pub fold: Option<i32>,
    /// 注释里的歌词关键词芯片（显示列 [a,b) 上色块）
    pub kw: Option<(i32, i32, Kw)>,
}

impl SrcLine {
    pub fn new(text: &str) -> Self {
        SrcLine {
            text: text.to_string(),
            reveal: i32::MAX,
            caret: false,
            gutter: Gutter::Blank,
            bg: None,
            dim: 1.0,
            err: None,
            hl: None,
            ghost: None,
            fold: None,
            kw: None,
        }
    }
    pub fn num(n: i32, text: &str) -> Self {
        SrcLine::new(text).gut(Gutter::Num(n))
    }
    pub fn kwin(mut self, a: i32, b: i32, k: Kw) -> Self {
        self.kw = Some((a, b, k));
        self
    }
    pub fn gut(mut self, g: Gutter) -> Self {
        self.gutter = g;
        self
    }
    pub fn rev(mut self, r: i32) -> Self {
        self.reveal = r;
        self
    }
    pub fn caret(mut self, on: bool) -> Self {
        self.caret = on;
        self
    }
    pub fn bgc(mut self, c: Rgb) -> Self {
        self.bg = Some(c);
        self
    }
    pub fn dimf(mut self, f: f32) -> Self {
        self.dim = f;
        self
    }
    pub fn errin(mut self, a: i32, b: i32) -> Self {
        self.err = Some((a, b));
        self
    }
    pub fn hlin(mut self, a: i32, b: i32, c: Rgb) -> Self {
        self.hl = Some((a, b, c));
        self
    }
    pub fn ghostin(mut self, s: &str, c: Rgb) -> Self {
        self.ghost = Some((s.to_string(), c));
        self
    }
    pub fn folded(mut self, n: i32) -> Self {
        self.fold = Some(n);
        self
    }
}

pub struct ViewOpts {
    /// 首个可见的文档行下标（滚动）
    pub scroll: i32,
    /// 右缘迷你地图
    pub minimap: bool,
    /// 缩进参考线
    pub guides: bool,
}

impl Default for ViewOpts {
    fn default() -> Self {
        ViewOpts {
            scroll: 0,
            minimap: true,
            guides: true,
        }
    }
}

pub const GUT_W: i32 = 7; // 装订线总宽：3 行号 + 1 空 + 1 分隔 + 2 空

// ── 时间 → 打字进度（纯函数）────────────────────────────────
/// 自 t0 起按 cps 打字，当前应显示的宽度
#[inline]
pub fn typed(lt: f32, t0: f32, cps: f32) -> i32 {
    ((lt - t0).max(0.0) * cps) as i32
}
/// 是否已打完
#[inline]
pub fn done(lt: f32, t0: f32, cps: f32, len: i32) -> bool {
    typed(lt, t0, cps) >= len
}
/// 光标闪烁（供离屏渲染可复现，frame 驱动）
#[inline]
pub fn blink(frame: u64, active: bool) -> bool {
    if active { true } else { (frame / 26) % 2 == 0 }
}

// ── 渲染 ────────────────────────────────────────────────────
/// 把一个源码文档画进舞台。 lines 可能多于可见行数，用 opts.scroll 滚动。
/// 注意：本函数全部使用舞台相对坐标（putb/text 都以舞台左上角为原点），
/// 因此可以嵌进任意子视口（分屏 diff 用）。
pub fn render(ctx: &mut Ctx, lines: &[SrcLine], opts: &ViewOpts) {
    let r_w = ctx.r.w;
    let r_h = ctx.r.h;
    let x0 = 1;
    let code_x = x0 + GUT_W;
    let blink_on = blink(ctx.v.frame, false);
    let pulse = ctx.bass();

    for row in 0..r_h {
        let li = opts.scroll + row;
        if li < 0 {
            continue;
        }
        let li = li as usize;
        if li >= lines.len() {
            break;
        }
        let l = &lines[li];
        let y = ctx.r.y + row;
        let gy = y - ctx.r.y;
        let dim = l.dim.clamp(0.0, 1.0);

        // 行底色（铺满视口宽）
        if let Some(bg) = l.bg {
            for x in 0..r_w {
                let bg = bg.mul(dim);
                ctx.putb(x, gy, ' ', bg, bg);
            }
        }

        // 装订线
        match l.gutter {
            Gutter::Num(n) | Gutter::Dirty(n) | Gutter::Add(n) | Gutter::Del(n) => {
                let dirty = matches!(l.gutter, Gutter::Dirty(_));
                let col = if dirty { GUTTER_DIRTY } else { GUTTER }.mul(dim);
                let s = format!("{n:>3}");
                ctx.text(1, gy, &s, col);
                let mark = match l.gutter {
                    Gutter::Add(_) => Some((ADD_GUT, '+')),
                    Gutter::Del(_) => Some((DEL_GUT, '-')),
                    Gutter::Dirty(_) => Some((GUTTER_DIRTY, '●')),
                    _ => None,
                };
                if let Some((mc, ch)) = mark {
                    ctx.put(4, gy, ch, mc.mul(dim));
                }
            }
            Gutter::Fold => {
                ctx.put(2, gy, '▾', GUTTER.mul(dim));
            }
            Gutter::Blank => {}
        }
        // 装订线分隔
        ctx.put(5, gy, '│', TEXT_FAINT.mul(0.55).mul(dim));

        if l.reveal <= 0 {
            continue;
        }

        // 折叠行
        if let Some(n) = l.fold {
            let s = if l.text.is_empty() {
                format!("┄┄┄  ▾ {n} lines ┄┄┄")
            } else {
                format!("┄┄┄  ▾ {n} lines · {} ┄┄┄", l.text)
            };
            ctx.text(code_x, gy, &s, TEXT_FAINT.mul(dim));
            continue;
        }

        // 缩进参考线（随低频呼吸）
        if opts.guides {
            let lead = l.text.len() - l.text.trim_start().len();
            let lead = lead as i32;
            let mut g = 0;
            while g + 4 <= lead {
                let col = GUIDE.mul(0.65 + 0.5 * pulse).mul(dim);
                ctx.put(code_x + g, gy, '│', col);
                g += 4;
            }
        }

        // 正文（逐字符着色）
        let runs = highlight(&l.text);
        let mut px = code_x; // 视口相对列
        let mut revealed = 0i32;
        let base_bg = l.bg.unwrap_or(BG);
        for run in &runs {
            for ch in run.s.chars() {
                let cwid = cw(ch);
                if cwid == 0 {
                    continue;
                }
                let start = revealed;
                if start < l.reveal {
                    if px < r_w - if opts.minimap { 3 } else { 1 } {
                        let mut col = run.col;
                        let mut bg = base_bg;
                        // 查找高亮
                        if let Some((a, b, hc)) = l.hl {
                            if start >= a && start < b {
                                bg = bg.mix(hc, 0.72);
                                col = col.mix(WHITE, 0.30);
                            }
                        }
                        // 注释里的歌词关键词芯片
                        if let Some((a, b, k)) = l.kw {
                            if start >= a && start < b {
                                let (kfg, kbg) = k.colors();
                                col = kfg;
                                bg = kbg.mul(0.85 + 0.30 * pulse);
                            }
                        }
                        // 错误区间
                        if let Some((a, b)) = l.err {
                            if start >= a && start < b {
                                col = col.mix(ERR_SQ, 0.85);
                                bg = bg.mix(ERR_BG, 0.60);
                            }
                        }
                        ctx.putb(px, gy, ch, col.mul(dim), bg.mul(dim));
                    }
                }
                revealed = start + cwid;
                px += cwid;
                if revealed >= l.reveal {
                    break;
                }
            }
            if revealed >= l.reveal {
                break;
            }
        }

        // 光标 + 幽灵补全
        if l.caret && blink_on {
            let caret_x = (code_x + l.reveal.min(r_w)).min(r_w - 2);
            let cw2 = CARET.mul(0.8 + 0.35 * pulse);
            ctx.putb(caret_x, gy, '▌', VOID, cw2);
            ctx.glow(caret_x, gy, CARET, 0.35);
        }
        if let Some((g, gc)) = &l.ghost {
            // 运行时求值回显：条件结果、返回值、计数器——"这段代码真的在跑"
            let caret_x = (code_x + l.reveal.min(r_w)).min(r_w - 2);
            ctx.text((caret_x + 1).min(r_w - 2), gy, g, gc.mul(dim));
        }
    }

    // 迷你地图：右缘一列，每个文档行一个字符 + 视口标记
    if opts.minimap {
        let mx = r_w - 2;
        let h = r_h;
        let total = lines.len() as i32;
        let scroll = opts.scroll.max(0);
        for row in 0..h {
            let li = scroll + row;
            if li < 0 || li as usize >= lines.len() {
                ctx.put(mx, row, '│', theme::PANEL_HI);
                continue;
            }
            let l = &lines[li as usize];
            let in_view = li >= scroll && li < scroll + h;
            let ch = l.text.trim_start().chars().next().unwrap_or('·');
            let col = if l.fold.is_some() {
                TEXT_FAINT
            } else if matches!(l.gutter, Gutter::Del(_)) {
                DEL_GUT.mul(0.5)
            } else if matches!(l.gutter, Gutter::Add(_)) {
                ADD_GUT.mul(0.5)
            } else if l.err.is_some() {
                ERR_SQ.mul(0.6)
            } else if in_view {
                GUTTER.mul(0.85)
            } else {
                TEXT_FAINT.mul(0.7)
            };
            let col = col.mul(0.7 + 0.3 * ctx.high());
            ctx.put(mx, row, ch, col);
        }
        // 视口把手
        let vy = (scroll as f32 / total.max(1) as f32 * h as f32) as i32;
        let vy = vy.clamp(0, h - 1);
        ctx.put(mx, vy, '◀', GUTTER.mul(1.4));
    }
}

// ── 编辑器部件 ──────────────────────────────────────────────
fn card_frame(ctx: &mut Ctx, r: crate::buf::Rect, col: Rgb, bg: Rgb) {
    ctx.fill(r, ' ', col, bg);
    ctx.frame(r, crate::buf::Frame::Rounded, col);
}

/// 悬停卡：`ⓘ` 信息 + 若干行彩色文本，按 reveal 列数打字。
pub fn card(ctx: &mut Ctx, x: i32, y: i32, title: &str, body: &[(&str, Rgb)], reveal: i32) {
    let w = body
        .iter()
        .map(|(s, _)| sw(s))
        .max()
        .unwrap_or(0)
        .max(sw(title))
        + 6;
    let h = body.len() as i32 + 3;
    let r = crate::buf::Rect::new(x, y, w, h);
    card_frame(ctx, r, CYAN.mul(0.75), theme::PANEL_HI);
    ctx.textb(r.x + 2, r.y, " INFO ", CYAN, theme::PANEL_HI);
    ctx.textb(r.x + 2, r.y + 1, title, SYN_DOC, theme::PANEL_HI);
    let mut acc = 0i32;
    for (i, (s, col)) in body.iter().enumerate() {
        let yy = r.y + 2 + i as i32;
        let ww = sw(s);
        if acc >= reveal {
            break;
        }
        let vis = (reveal - acc).min(ww);
        let shown: String = {
            let mut o = String::new();
            let mut w2 = 0;
            for ch in s.chars() {
                let c = cw(ch);
                if w2 + c <= vis {
                    o.push(ch);
                    w2 += c;
                } else {
                    break;
                }
            }
            o
        };
        ctx.textb(r.x + 3, yy, &shown, *col, theme::PANEL_HI);
        acc += ww + 2;
    }
}

/// 单行悬停卡（标题 + 一行内容，打字 + 呼吸）
pub fn card_popup(ctx: &mut Ctx, x: i32, y: i32, title: &str, body: &str, col: Rgb, age: f32) {
    let w = crate::buf::sw(title).max(crate::buf::sw(body)) + 5;
    let w = w.min(ctx.r.w - 2);
    let r = crate::buf::Rect::new(x.clamp(1, ctx.r.w - w - 1), y.clamp(0, ctx.r.h - 3), w, 3);
    let breathe = 0.75 + 0.25 * (age * 3.0).sin();
    ctx.fill(r, ' ', col, theme::PANEL_HI);
    ctx.frame(r, crate::buf::Frame::Rounded, col.mul(breathe));
    ctx.textb(r.x + 2, r.y, title, col, theme::PANEL_HI);
    let shown = crate::fx2::typed_str(body, age, 0.0, 60.0);
    ctx.textb(
        r.x + 3,
        r.y + 1,
        &shown,
        theme::TEXT.mul(0.95),
        theme::PANEL_HI,
    );
}

/// rustc 诊断框：错误 + 位置 + 源码行 + ^^^^ 指示，typed 列数打字。
pub fn rustc_diag(
    ctx: &mut Ctx,
    x: i32,
    y: i32,
    code: &str,
    msg: &str,
    at: &str,
    src: &str,
    span: (i32, i32),
    reveal: i32,
) {
    let head = format!("error[{code}]: {msg}");
    let l2 = format!("  --> {at}");
    let bar = "   |";
    let l4 = format!("{bar} {}", src.trim_start());
    let indent = 4 + src.len() - src.trim_start().len();
    let mut marks = String::new();
    for i in 0..(src.trim_start().len() as i32) {
        marks.push(if i >= span.0 && i < span.1 { '^' } else { ' ' });
    }
    let l5 = format!(
        "{bar} {}{}",
        " ".repeat((indent - 4).max(0) as usize),
        marks
    );
    let w = [sw(&head), sw(&l2), sw(&l4), sw(&l5)]
        .into_iter()
        .max()
        .unwrap_or(20)
        + 5;
    let h = 6;
    let r = crate::buf::Rect::new(x, y, w.min(ctx.r.w - x - 1).max(12), h);
    card_frame(ctx, r, RED.mul(0.8), theme::PANEL);
    let mut acc = 0;
    let put = |ctx: &mut Ctx, x: i32, yy: i32, s: &str, col: Rgb, acc: &mut i32| {
        let ww = sw(s);
        if *acc >= reveal {
            return;
        }
        let vis = (reveal - *acc).min(ww);
        let mut o = String::new();
        let mut w2 = 0;
        for ch in s.chars() {
            let c = cw(ch);
            if w2 + c <= vis {
                o.push(ch);
                w2 += c;
            } else {
                break;
            }
        }
        ctx.textb(x, yy, &o, col, theme::PANEL);
        *acc += ww + 2;
    };
    put(ctx, r.x + 2, r.y + 1, &head, ERR_SQ, &mut acc);
    put(ctx, r.x + 2, r.y + 2, &l2, SYN_DOC, &mut acc);
    put(ctx, r.x + 2, r.y + 3, bar, GUTTER, &mut acc);
    put(ctx, r.x + 2, r.y + 4, &l4, SYN_IDENT, &mut acc);
    put(ctx, r.x + 2, r.y + 5, &l5, ERR_SQ, &mut acc);
}

/// 查找 / 替换条
pub fn find_bar(ctx: &mut Ctx, y: i32, find: &str, rep: &str, hit: (i32, i32), reveal: i32) {
    let s1 = "Find: ";
    let s2 = format!(
        "{find}  -> replace: {rep}   {}/{}   [Aa] [.*] [Replace All]",
        hit.0, hit.1
    );
    let w = sw(s1) + sw(&s2) + 4;
    let r = crate::buf::Rect::new(ctx.r.w - w - 2, y, w, 1);
    ctx.fill(r, ' ', SYN_IDENT, theme::PANEL_HI);
    let mut acc = 0;
    let put = |ctx: &mut Ctx, x: &mut i32, s: &str, col: Rgb, acc: &mut i32| {
        let ww = sw(s);
        if *acc < reveal {
            let vis = (reveal - *acc).min(ww);
            let mut o = String::new();
            let mut w2 = 0;
            for ch in s.chars() {
                let c = cw(ch);
                if w2 + c <= vis {
                    o.push(ch);
                    w2 += c;
                } else {
                    break;
                }
            }
            ctx.textb(*x, y, &o, col, theme::PANEL_HI);
        }
        *acc += ww;
        *x += ww;
    };
    let mut x = r.x + 2;
    put(ctx, &mut x, s1, SYN_DOC, &mut acc);
    put(
        ctx,
        &mut x,
        &format!("{find}"),
        HL_BG.mix(WHITE, 0.6),
        &mut acc,
    );
    put(ctx, &mut x, "  -> ", SYN_PUNCT, &mut acc);
    put(ctx, &mut x, "replace: ", SYN_DOC, &mut acc);
    put(ctx, &mut x, rep, ADD_GUT, &mut acc);
    put(
        ctx,
        &mut x,
        &format!("   {}/{}   [Aa] [.*] [Replace All]", hit.0, hit.1),
        GUTTER,
        &mut acc,
    );
}

/// 行内悬停提示（单行）
pub fn tooltip(ctx: &mut Ctx, x: i32, y: i32, text: &str, col: Rgb, reveal: i32) {
    let w = sw(text) + 4;
    let r = crate::buf::Rect::new(x, y, w, 1);
    ctx.fill(r, ' ', col, theme::PANEL_HI);
    let vis = reveal.min(sw(text));
    let mut o = String::new();
    let mut w2 = 0;
    for ch in text.chars() {
        let c = cw(ch);
        if w2 + c <= vis {
            o.push(ch);
            w2 += c;
        } else {
            break;
        }
    }
    ctx.textb(x + 2, y, &o, col, theme::PANEL_HI);
}

/// 全宽注释横幅（verdict / 大事件）
pub fn banner(ctx: &mut Ctx, y: i32, s: &str, col: Rgb, reveal: i32, frame: u64) {
    let w = ctx.r.w;
    let swd = sw(s);
    let pad = ((w - swd - 12) / 2).max(2);
    let text = format!(
        "{}  {s}  {}",
        "/".repeat(pad as usize),
        "/".repeat(pad as usize)
    );
    let vis = reveal.min(sw(&text));
    let mut o = String::new();
    let mut w2 = 0;
    for ch in text.chars() {
        let c = cw(ch);
        if w2 + c <= vis {
            o.push(ch);
            w2 += c;
        } else {
            break;
        }
    }
    let r = crate::buf::Rect::new(0, y, w, 1);
    ctx.fill(r, ' ', col.mul(0.22), BG);
    ctx.textb(0, y, &o, col, BG);
    // 打完后的呼吸
    if vis >= sw(&text) {
        let a = 0.10 + 0.12 * ((frame as f32 * 0.25).sin() * 0.5 + 0.5);
        for x in 0..w {
            ctx.glow(x, y, col, a);
        }
    }
}

// ── 文档构建器 ──────────────────────────────────────────────
/// 场景按"哪一行、什么时刻开始打字"声明文档，构建器负责算 reveal、
/// 分配行号（自动递增，注释也有行号——真实编辑器就是这样）与光标归属。
pub struct DocBuilder {
    pub lines: Vec<SrcLine>,
    /// 强制把光标放在这一行（下标）；None = 自动放在打字中的最后一行
    pub caret_at: Option<usize>,
    /// 下一行的行号
    pub n: i32,
}

impl DocBuilder {
    pub fn new() -> Self {
        DocBuilder {
            lines: Vec::new(),
            caret_at: None,
            n: 1,
        }
    }

    /// 设定起始行号
    pub fn at(&mut self, n: i32) -> &mut Self {
        self.n = n;
        self
    }

    /// 消耗一个行号
    #[inline]
    pub fn take(&mut self) -> i32 {
        let v = self.n;
        self.n += 1;
        v
    }

    /// 空行（有行号）
    pub fn blank(&mut self) {
        let n = self.take();
        self.lines.push(SrcLine::new("").gut(Gutter::Num(n)));
    }

    /// 注释行（歌词的形态）。注释里的全大写词自动上关键词芯片。
    pub fn cmt(&mut self, lt: f32, t0: f32, s: &str, cps: f32) -> usize {
        let w = sw(s);
        let n = self.take();
        let mut l = SrcLine::new(s)
            .gut(Gutter::Num(n))
            .rev(typed(lt, t0, cps).min(w));
        let mut a = 0i32;
        for word in s.split(' ') {
            let ww = sw(word);
            if let Some(k) = theme::classify(word) {
                if a < w {
                    l.kw = Some((a, (a + ww).min(w), k));
                }
            }
            a += ww + 1;
        }
        self.lines.push(l);
        self.lines.len() - 1
    }

    /// 代码行
    pub fn code(&mut self, lt: f32, t0: f32, s: &str, cps: f32) -> usize {
        let w = sw(s);
        let n = self.take();
        let l = SrcLine::new(s)
            .gut(Gutter::Num(n))
            .rev(typed(lt, t0, cps).min(w));
        self.lines.push(l);
        self.lines.len() - 1
    }

    /// 代码行 + 行尾注释（注释从 t1 开始打）
    pub fn code_cmt(&mut self, lt: f32, t0: f32, s: &str, cps: f32, t1: f32, cmt: &str) -> usize {
        let text = format!("{s}  {cmt}");
        let w1 = sw(s);
        let w = sw(&text);
        let r = if lt < t1 {
            typed(lt, t0, cps).min(w1)
        } else {
            w1 + 2 + typed(lt, t1, cps).min(w - w1 - 2)
        };
        let n = self.take();
        let l = SrcLine::new(&text).gut(Gutter::Num(n)).rev(r.min(w));
        self.lines.push(l);
        self.lines.len() - 1
    }

    /// 自定义行：装订线为 Blank 时自动补行号
    pub fn push(&mut self, mut l: SrcLine) -> usize {
        if l.gutter == Gutter::Blank {
            let n = self.take();
            l.gutter = Gutter::Num(n);
        }
        self.lines.push(l);
        self.lines.len() - 1
    }

    /// 完全自定义行（不动行号）
    pub fn push_raw(&mut self, l: SrcLine) -> usize {
        self.lines.push(l);
        self.lines.len() - 1
    }

    /// 折叠掉的前文（编辑器 fold 标记）：让每一幕都像同一个文件在生长
    pub fn fold_prev(&mut self, n_lines: i32, label: &str) {
        let l = SrcLine::new(label).gut(Gutter::Fold).folded(n_lines);
        self.push_raw(l);
    }

    /// 前文上下文（压暗的真实行）
    pub fn dim_prev(&mut self, lines: &[&str], from: i32) {
        for (k, s) in lines.iter().enumerate() {
            let l = SrcLine::new(s).gut(Gutter::Num(from + k as i32)).dimf(0.20);
            self.push_raw(l);
        }
    }

    /// 已显示宽度（供场景做追加判断）
    pub fn shown(&self, i: usize, lt: f32, t0: f32, cps: f32) -> i32 {
        self.lines
            .get(i)
            .map(|l| l.reveal)
            .unwrap_or_else(|| typed(lt, t0, cps))
    }

    /// 收尾：确定光标位置。frame 用于闪烁相位。
    pub fn finish(mut self, frame: u64) -> Vec<SrcLine> {
        let mut active: Option<usize> = None;
        for (i, l) in self.lines.iter().enumerate().rev() {
            if l.fold.is_some() {
                continue;
            }
            if l.reveal < sw(&l.text) && l.reveal > 0 {
                active = Some(i);
                break;
            }
        }
        let at = self.caret_at.or(active);
        if let Some(i) = at {
            if let Some(l) = self.lines.get_mut(i) {
                l.caret = blink(frame, true);
            }
        }
        self.lines
    }
}

impl Default for DocBuilder {
    fn default() -> Self {
        Self::new()
    }
}
