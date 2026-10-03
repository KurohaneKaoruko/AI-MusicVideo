//! Pixel-art specimens. Each sprite is a coarse char map with its own palette,
//! drawn at an arbitrary pixel scale — the machine's own resolution for "you".
//!
//! Convention: ' ' transparent · 'K' outline · 'B' body · 'M' shadow
//!             'L' light · 'o' eye · 'p' blush · 'w' white · 'g' glow

use crate::gfx::Color;
use crate::pix::Canvas;

pub struct Spr {
    pub rows: &'static [&'static str],
    pub pal: &'static [(char, u32)],
}

/// Draw a sprite centred on (cx, cy) in pixels at the given scale.
pub fn draw_c(cv: &mut Canvas, s: &Spr, cx: f32, cy: f32, scale: i32, alpha: f64, tint: Option<Color>) {
    let (w, h) = size(s, scale);
    let x = (cx - w as f32 / 2.0).round() as i32;
    let y = (cy - h as f32 / 2.0).round() as i32;
    cv.bitmap(x, y, s.rows, s.pal, scale, alpha, tint);
}

/// Draw anchored at the top-left.
pub fn draw_at(cv: &mut Canvas, s: &Spr, x: i32, y: i32, scale: i32, alpha: f64, tint: Option<Color>) {
    cv.bitmap(x, y, s.rows, s.pal, scale, alpha, tint);
}

pub fn size(s: &Spr, scale: i32) -> (i32, i32) {
    Canvas::bitmap_size(s.rows, scale)
}

// ------------------------------------------------------------------ table ---

pub const TABLE: Spr = Spr {
    rows: &[
        "  ╔══════════════════════════════════════════╗  ",
        "  ╠══════════════════════════════════════════╣  ",
        "  ╚══════════════════════════════════════════╝  ",
        "                                                ",
        "   ║                ║                ║          ",
        "   ║                ║                ║          ",
        "   ║                ║                ║          ",
        "   ║                ║                ║          ",
        "   ║                ║                ║          ",
        "   ║                ║                ║          ",
        "   ║                ║                ║          ",
        "  ███              ║                ║          ",
        "   ║                ║             ██████        ",
        "   ▔▔▔              ▔▔▔              ▔▔▔        ",
    ],
    pal: &[
        ('═', 0xd9a066),
        ('║', 0x8a6242),
        ('╔', 0xe0aa72),
        ('╗', 0xe0aa72),
        ('╚', 0x6f4c34),
        ('╝', 0x6f4c34),
        ('╠', 0xc08a55),
        ('╣', 0xc08a55),
        ('█', 0x8a6242),
        ('▔', 0x3a2a1e),
    ],
};

// --------------------------------------------------------------- eggplant ---

pub const EGGPLANT: Spr = Spr {
    rows: &[
        "         g              ",
        "        ggg             ",
        "       gMgM             ",
        "      g   g             ",
        "       KKKKK            ",
        "      KBBBBBK           ",
        "     KBBLBBBBK          ",
        "    KBBLBBBBBBK         ",
        "   KBBLBBBBBBBBK        ",
        "   KBLBBBBBBBBBBK       ",
        "  KBLLBBBBBBBBBBK       ",
        "  KBLBBBBBBBBBBBK       ",
        "  KBBBBBBBBBBBBBK       ",
        "  KBBBBBBBBBBBBBK       ",
        "  KBBBBBBBBBBBBK        ",
        "   KBBBBBBBBBBK         ",
        "   KBBBBBBBBBK          ",
        "    KBBBBBBK            ",
        "     KBBBBK             ",
        "      KKKK              ",
    ],
    pal: &[
        ('K', 0x3a2055),
        ('B', 0x8b5cc9),
        ('L', 0xc7a3ee),
        ('M', 0x2b1b45),
        ('g', 0x5f9e57),
    ],
};

// -------------------------------------------------------------------- cat ---

pub const CAT: Spr = Spr {
    rows: &[
        "       K           K       ",
        "      KKK         KKK      ",
        "      KKKK       KKKK      ",
        "      KKKKKKKKKKKKKKK      ",
        "     KKKKKKKKKKKKKKKKK     ",
        "     KKBBBBBBBBBBBBBKK     ",
        "     KKBwwBBBBBBBwwBKK     ",
        "     KKBwoBBBBBBBowBKK     ",
        "     KKBBBBBpppBBBBBKK     ",
        "     KKBBBBBBBBBBBBBKK     ",
        "     KKKBBBBBBBBBBBKKK     ",
        "      KKKBBBBBBBBBKKK      ",
        "      KKKBBBBBBBBBKKK      ",
        "     KKKKBBBBBBBBBKKKK     ",
        "    KKLLKBBBBBBBBBKLLKK    ",
        "    KKLLKBBBBBBBBBKLLKK    ",
        "    KKKKKBBBBBBBBBKKKKK    ",
        "     KKKKKBBBBBBBKKKKK     ",
        "      KKKKKKKKKKKKKKK      ",
        "       KKK K   K KKK       ",
        "        KK K   K KK        ",
    ],
    pal: &[
        ('K', 0x7d8ba3),
        ('B', 0xc3cad6),
        ('w', 0x2b3a55),
        ('o', 0x1b2433),
        ('p', 0xe89aa4),
        ('L', 0xe8eef7),
    ],
};

// ------------------------------------------------------------------ steak ---

pub const STEAK: Spr = Spr {
    rows: &[
        "      KKKKKKKKKKKKKKKK      ",
        "     KeeeeeeeeeeeeeeeeK     ",
        "   KKeeeeeeeeeeeeeeeeeeKK   ",
        "  KeeeeeeeeeeeeeeeeeeeeeeK  ",
        " KeeeeeeeeeeeeeeeeeeeeeeeeK ",
        " KeeKKeeeeeeeeeeeeeeeeKKeeeK",
        " KeeKppppppppppppppppKpeeeeK",
        " KeKppppppppppppppppppKeeeeK",
        " KeKppppppppppppppppppKeeeeK",
        " KeKppppppppppppppppppKeeeeK",
        " KeeeeKppppppppppppKeeeeeeK ",
        " KeeKKeeeeeeeeeeeeeeKKeeeeeK ",
        " KeeeeeeeeeeeeeeeeeeeeeeeeK ",
        "  KeeeeeeeeeeeeeeeeeeeeeeK  ",
        "   KKeeeeeeeeeeeeeeeeeeKK   ",
        "     KeeeeeeeeeeeeeeeeK     ",
        "      KKKKKKKKKKKKKKKK      ",
    ],
    pal: &[
        ('K', 0x4a2a18),
        ('e', 0x8a4f30),
        ('p', 0xd98a8a),
    ],
};

// ----------------------------------------------------------------- flower ---

pub const FLOWER_BUD: Spr = Spr {
    rows: &[
        "      KKK      ",
        "     KpppK     ",
        "    KppLppK    ",
        "    KpLLLpK    ",
        "    KppLppK    ",
        "     KpppK     ",
        "      KgK      ",
        "      KgK      ",
        "      KgK      ",
        "    g KgK g    ",
        "   gg KgK gg   ",
        "    g KgK g    ",
        "      KgK      ",
        "      KgK      ",
        "      KgK      ",
        "      KgK      ",
        "      KgK      ",
        "     KKgKK     ",
        "    KgggggK    ",
        "   KgggggggK   ",
        "  KgggggggggK  ",
    ],
    pal: &[
        ('K', 0x3f6b3a),
        ('p', 0xe98cae),
        ('L', 0xffc2d6),
        ('g', 0x5f9e57),
    ],
};

pub const FLOWER_BLOOM: Spr = Spr {
    rows: &[
        "    KKK   KKK    ",
        "   KpppK KpppK   ",
        "  KppLppKppLppK  ",
        "  KpLLLpppLLLpK  ",
        "  KppLLLwLLLppK  ",
        "   KppLwLwLppK   ",
        "  KppLLLwLLLppK  ",
        "  KpLLLpppLLLpK  ",
        "  KppLppKppLppK  ",
        "   KpppK KpppK   ",
        "    KKK   KKK    ",
        "        KgK      ",
        "      g KgK g    ",
        "     gg KgK gg   ",
        "      g KgK g    ",
        "        KgK      ",
        "        KgK      ",
        "        KgK      ",
        "       KKgKK     ",
        "      KgggggK    ",
        "     KgggggggK   ",
    ],
    pal: &[
        ('K', 0x3f6b3a),
        ('p', 0xf2a7c3),
        ('L', 0xffd8e6),
        ('g', 0x5f9e57),
        ('w', 0xf2c14e),
    ],
};

// ------------------------------------------------------------------ hands ---

pub const HANDS: Spr = Spr {
    rows: &[
        "    KK                        KK    ",
        "   KBBK                      KBBK   ",
        "   KBBK                      KBBK   ",
        "   KBBK     KKKKKKKK         KBBK   ",
        "   KBBK    KBBBBBBBBK       KBBK    ",
        "   KBBK   KBBBBBBBBBBK     KBBK     ",
        "   KBBK  KBBBBBBBBBBBBK   KBBK      ",
        "   KBBK KBBBBBBBBBBBBBBK KBBK       ",
        "   KBBKKBBBBBBBBBBBBBBBBKKBBK       ",
        "   KBBBBBBBBBBBBBBBBBBBBBBBK        ",
        "   KBBBBBBBBBBBBBBBBBBBBBBK         ",
        "   KBBBBBBBBBBBBBBBBBBBBBBK         ",
        "    KBBBBBBBBBBBBBBBBBBBBK          ",
        "    KBBBBBBBBBBBBBBBBBBBBK          ",
        "     KBBBBBBBBBBBBBBBBBBK           ",
        "      KBBBBBBBBBBBBBBBBK            ",
        "       KBBBBBBBBBBBBBBK             ",
        "        KBBBBBBBBBBBBK              ",
        "         KBBBBBBBBBBK               ",
        "          KKKKKKKKKK                ",
    ],
    pal: &[
        ('K', 0xa8724c),
        ('B', 0xe8c39e),
    ],
};

// -------------------------------------------------------------------- dog ---

pub const DOG: Spr = Spr {
    rows: &[
        "     KKK         KKK     ",
        "    KKMMK       KMMKK    ",
        "    KKKKKKKKKKKKKKKKK    ",
        "   KKKKKKKKKKKKKKKKKKK   ",
        "   KKKBBBBBBBBBBBBBKKK   ",
        "   KKBBwBBBBBBBwBBBBKK   ",
        "   KKBBBBBBBBBBBBBBBKK   ",
        "   KKBBBBBBKKBBBBBBBKK   ",
        "   KKKBBBKKKKKKBBBKKKK   ",
        "    KKBBBBBBBBBBBBBKK    ",
        "     KKBBBBBBBBBBBKK     ",
        "   KKLLKBBBBBBBBBKLLKK   ",
        "   KKLLKBBBBBBBBBKLLKK   ",
        "   KKKKKBBBBBBBBBKKKKK   ",
        "    KKKKBBBBBBBBBKKKK    ",
        "     KKKKKBBBBBKKKKK     ",
        "       KKK BBB KKK       ",
        "       KKK BBB KKK       ",
        "       KKKKKKKKKKK       ",
    ],
    pal: &[
        ('K', 0x8a6242),
        ('B', 0xd9b98a),
        ('M', 0x6f4c34),
        ('w', 0x2b2010),
        ('L', 0xf0dcb8),
    ],
};

// ----------------------------------------------------------------- tomato ---

pub const TOMATO: Spr = Spr {
    rows: &[
        "        ggg        ",
        "      ggggggg      ",
        "     ggggggggg     ",
        "      gKgggKg      ",
        "       KKKKK       ",
        "     KKBBBBBKK     ",
        "   KKBBBLBBBBBKK   ",
        "  KBBBLLBBBBBBBBK  ",
        " KBBLBBBBBBBBBBBBK ",
        " KBLBBBBBBBBBBBBBK ",
        " KBLLBBBBBBBBBBBBK ",
        " KBBBBBBBBBBBBBBBK ",
        " KBBBBBBBBBBBBBBBK ",
        "  KBBBBBBBBBBBBBK  ",
        "   KKBBBBBBBBBKK   ",
        "     KKBBBBBKK     ",
        "       KKKKK       ",
    ],
    pal: &[
        ('K', 0x7a2418),
        ('B', 0xc94436),
        ('L', 0xf07a68),
        ('g', 0x5f9e57),
    ],
};

// ----------------------------------------------------------------- bucket ---

pub const BUCKET: Spr = Spr {
    rows: &[
        "  KKKKKKKKKKKKKKKKKKKK  ",
        " K                    K ",
        " K                    K ",
        "  K                  K  ",
        "  K                  K  ",
        "  K                  K  ",
        "  K                  K  ",
        "  K                  K  ",
        "  K                  K  ",
        "  K                  K  ",
        "  K                  K  ",
        "  K                  K  ",
        "  K                  K  ",
        "   K                K   ",
        "   K                K   ",
        "   KKKKKKKKKKKKKKKKKK   ",
    ],
    pal: &[('K', 0x9aa4b4)],
};

// ---------------------------------------------------------------- chimera ---

pub const CHIMERA: Spr = Spr {
    rows: &[
        "    KKK   KKK   KKK    ",
        "   KMMMK KMMMK KMMMK   ",
        "    KKKKKKKKKKKKKKK    ",
        "   KKBBBBBBBBBBBBBKK   ",
        "   KKBwBBBBBBBBwBBKK   ",
        "   KKBBBBBBBBBBBBBKK   ",
        "   KKBBBBBppBBBBBBKK   ",
        "   KKKBBBBBBBBBBKKKK   ",
        "    KKBBBBBBBBBBKK     ",
        "   KKLLKBBBBBBBKLLKK   ",
        "   KKLLKBBBBBBBKLLKK   ",
        "   KKKKKBBBBBBBKKKKK   ",
        "    KKKKKKKKKKKKKKK    ",
        "      g KgK g          ",
        "     gg KgK gg         ",
        "      g KgK g          ",
        "        KgK            ",
    ],
    pal: &[
        ('K', 0x7d8ba3),
        ('B', 0xc3cad6),
        ('M', 0x5a6a86),
        ('w', 0x2b3a55),
        ('p', 0xe89aa4),
        ('L', 0xe8eef7),
        ('g', 0x6fc7b8),
    ],
};

// ------------------------------------------------------------------ heart ---

pub const HEART: Spr = Spr {
    rows: &[
        "  KKKK     KKKK  ",
        " KBBBBK   KBBBBK ",
        "KBLBBBBK KBBBBBBK",
        "KBLLBBBBKBBBBBBBK",
        "KBLBBBBBBBBBBBBBK",
        "KBBBBBBBBBBBBBBBK",
        " KBBBBBBBBBBBBBK ",
        " KBBBBBBBBBBBBBK ",
        "  KBBBBBBBBBBBK  ",
        "   KBBBBBBBBBK   ",
        "    KBBBBBBBK    ",
        "     KBBBBBK     ",
        "      KBBBK      ",
        "       KBK       ",
        "        K        ",
    ],
    pal: &[
        ('K', 0xa0495f),
        ('B', 0xff6d8a),
        ('L', 0xffc2d0),
    ],
};

// ---------------------------------------------------------------- figure ---

/// A distant person at the end of the corridor — deliberately tiny.
pub const FIGURE: Spr = Spr {
    rows: &[
        "  KKK  ",
        " KBBBK ",
        " KBoBK ",
        "  KKK  ",
        " KK KK ",
        "KBBBBK ",
        "KBBBBK ",
        " KKBKK ",
        "  K K  ",
        "  K K  ",
        "  K K  ",
        " KKKKK ",
    ],
    pal: &[
        ('K', 0x2b3547),
        ('B', 0x66748c),
        ('o', 0xff6d8a),
    ],
};
