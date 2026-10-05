//! 调色板：v2「SOURCE」版。
//!
//! v1 的色彩语言是"机器 / 爱 / 死亡"，v2 的舞台是一个代码编辑器，
//! 所以整套颜色换成**语法高亮**的语言：
//!   关键词紫 · 类型青 · 字符串绿 · 宏品红 · 注释灰 · 诊断红 · 补全幽灵灰
//! 剧情色只保留三种：金 = 神性/解答，品红 = 爱，正红 = 执行/报错。

// 调色板按需取用，保留完整 API 面
#![allow(dead_code)]

use crate::buf::Rgb;

// ── 基础背景（编辑器底色）────────────────────────────────────
pub const VOID: Rgb = Rgb::new(4, 6, 10);
pub const BG: Rgb = Rgb::new(8, 11, 18);
pub const PANEL: Rgb = Rgb::new(13, 18, 28);
pub const PANEL_HI: Rgb = Rgb::new(20, 28, 42);

// ── 前景文本 ────────────────────────────────────────────────
pub const TEXT: Rgb = Rgb::new(198, 214, 234);
pub const TEXT_DIM: Rgb = Rgb::new(96, 114, 140);
pub const TEXT_FAINT: Rgb = Rgb::new(48, 60, 78);

// ── 主题色（剧情用）─────────────────────────────────────────
pub const CYAN: Rgb = Rgb::new(58, 214, 226);
pub const CYAN_DIM: Rgb = Rgb::new(24, 92, 104);
pub const BLUE: Rgb = Rgb::new(96, 146, 255);
pub const BLUE_DIM: Rgb = Rgb::new(28, 48, 104);
pub const MAGENTA: Rgb = Rgb::new(255, 78, 158);
pub const MAGENTA_DIM: Rgb = Rgb::new(112, 26, 68);
pub const RED: Rgb = Rgb::new(255, 74, 64);
pub const RED_DIM: Rgb = Rgb::new(104, 20, 18);
pub const AMBER: Rgb = Rgb::new(255, 180, 84);
pub const AMBER_DIM: Rgb = Rgb::new(104, 70, 28);
pub const GREEN: Rgb = Rgb::new(96, 228, 122);
pub const GREEN_DIM: Rgb = Rgb::new(28, 92, 48);
pub const PURPLE: Rgb = Rgb::new(178, 118, 255);
pub const PURPLE_DIM: Rgb = Rgb::new(62, 38, 104);
pub const WHITE: Rgb = Rgb::new(240, 246, 255);
/// 神性 / 解答 / 推导成功的金色
pub const GOLD: Rgb = Rgb::new(255, 214, 120);

// ── 语法高亮色（代码的正装）─────────────────────────────────
pub const SYN_KW: Rgb = Rgb::new(198, 130, 255); // fn let mut impl trait …
pub const SYN_KW2: Rgb = Rgb::new(255, 120, 190); // self Self crate
pub const SYN_TY: Rgb = Rgb::new(94, 220, 228); // World Me Power
pub const SYN_FN: Rgb = Rgb::new(118, 158, 255); // execute build give
pub const SYN_MACRO: Rgb = Rgb::new(255, 110, 178); // execute! assert!
pub const SYN_STR: Rgb = Rgb::new(158, 226, 158); // "…"
pub const SYN_NUM: Rgb = Rgb::new(255, 186, 108); // 0 3 2π
pub const SYN_CMT: Rgb = Rgb::new(94, 122, 118); // // …
pub const SYN_DOC: Rgb = Rgb::new(110, 148, 156); // /// … 与 //!
pub const SYN_ATTR: Rgb = Rgb::new(255, 200, 140); // #[…] #![…]
pub const SYN_LIFE: Rgb = Rgb::new(178, 130, 255); // 'a
pub const SYN_PUNCT: Rgb = Rgb::new(120, 138, 164); // ( ) ; : , . =
pub const SYN_OP: Rgb = Rgb::new(150, 200, 220); // -> + * ::
pub const SYN_IDENT: Rgb = Rgb::new(214, 226, 242); // 普通标识符
pub const SYN_GHOST: Rgb = Rgb::new(88, 100, 122); // 补全幽灵文本

// ── 编辑器部件色 ────────────────────────────────────────────
pub const GUTTER: Rgb = Rgb::new(74, 88, 110); // 行号
pub const GUTTER_DIRTY: Rgb = Rgb::new(255, 180, 84); // 未保存行号（amber）
pub const GUIDE: Rgb = Rgb::new(38, 50, 68); // 缩进参考线
pub const SEL_BG: Rgb = Rgb::new(38, 74, 120); // 查找高亮底
pub const HL_BG: Rgb = Rgb::new(96, 60, 16); // 命中底（amber 深）
pub const ERR_BG: Rgb = Rgb::new(96, 24, 22); // 错误行底
pub const ERR_SQ: Rgb = Rgb::new(255, 84, 74); // 错误波浪线（红）
pub const ADD_BG: Rgb = Rgb::new(18, 56, 32); // diff 新增行底
pub const ADD_GUT: Rgb = Rgb::new(96, 228, 122);
pub const DEL_BG: Rgb = Rgb::new(70, 22, 22); // diff 删除行底
pub const DEL_GUT: Rgb = Rgb::new(255, 96, 86);
pub const CARET: Rgb = Rgb::new(120, 236, 255); // 光标

// ── 关键词配色（歌词条芯片，沿用 v1 的分类）─────────────────
/// 关键词的视觉分类，决定它在歌词条里被渲染成什么颜色的"数据块"。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kw {
    /// 数学：次元、圆周、切线、极限
    Math,
    /// 机器指令：执行、初始化、模拟
    Machine,
    /// 生物：营养、抗氧化物、咕噜声
    Organic,
    /// 情感：爱
    Love,
    /// 神性/存在
    Divine,
    /// 负面：孤独、失望、非法参数
    Negative,
}

impl Kw {
    /// (前景亮色, 背景底色)
    pub fn colors(self) -> (Rgb, Rgb) {
        match self {
            Kw::Math => (Rgb::new(150, 240, 250), Rgb::new(14, 74, 86)),
            Kw::Machine => (Rgb::new(160, 200, 255), Rgb::new(20, 52, 104)),
            Kw::Organic => (Rgb::new(180, 246, 160), Rgb::new(30, 78, 34)),
            Kw::Love => (Rgb::new(255, 168, 210), Rgb::new(110, 22, 66)),
            Kw::Divine => (Rgb::new(255, 232, 168), Rgb::new(96, 68, 20)),
            Kw::Negative => (Rgb::new(255, 158, 150), Rgb::new(96, 22, 20)),
        }
    }
    /// 该类关键词的"主题色"（用于粒子、光晕）
    pub fn accent(self) -> Rgb {
        match self {
            Kw::Math => CYAN,
            Kw::Machine => BLUE,
            Kw::Organic => GREEN,
            Kw::Love => MAGENTA,
            Kw::Divine => AMBER,
            Kw::Negative => RED,
        }
    }
}

/// 根据大写关键词判断视觉分类。返回 None 表示这是普通词。
pub fn classify(word: &str) -> Option<Kw> {
    let w: String = word
        .chars()
        .filter(|c| c.is_ascii_alphabetic() || *c == '-' || *c == ' ')
        .collect::<String>()
        .to_ascii_uppercase();
    let w = w.trim();
    Some(match w {
        "DIMENSION" | "CIRCUMFERENCE" | "TANGENTS" | "LIMITATIONS" | "VIBRATIONS" => Kw::Math,
        "PROTECTION" | "OBJECT CREATION" | "OBJECT" | "CREATION" | "INITIALIZATION"
        | "SIMULATION" | "EXECUTION" | "STIMULATIONS" | "SATISFACTION" | "COMPLETION"
        | "FRAGMENTS" => Kw::Machine,
        "NUTRIENTS" | "ANTIOXIDANTS" | "ENJOYMENT" => Kw::Organic,
        "LO-O-OVE" | "LOVE" => Kw::Love,
        "EXISTENCE" => Kw::Divine,
        "ISOLATION" | "DISHEARTENED" | "ILLEGAL ARGUMENTS" | "ILLEGAL" | "ARGUMENTS" => {
            Kw::Negative
        }
        _ => return None,
    })
}

/// 判断一个词是否是"全大写关键词"（含连字符 / 空格）。
pub fn is_keyword(word: &str) -> bool {
    let letters = word.chars().filter(|c| c.is_ascii_alphabetic()).count();
    letters >= 2
        && word
            .chars()
            .all(|c| !c.is_ascii_lowercase() && (c.is_ascii_alphabetic() || c == '-' || c == ' '))
}

/// 冷 → 暖的连续色带，用于进度条。
pub fn heat(t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    if t < 0.34 {
        CYAN.mix(BLUE, t / 0.34)
    } else if t < 0.67 {
        BLUE.mix(MAGENTA, (t - 0.34) / 0.33)
    } else {
        MAGENTA.mix(AMBER, (t - 0.67) / 0.33)
    }
}
