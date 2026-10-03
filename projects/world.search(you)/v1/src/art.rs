//! ASCII art assets: pixel font for big text + hand-drawn sprites with palettes.

use crate::gfx::{Color, Grid};

// ------------------------------------------------------------------ font ---

const FONT: &[(&str, [&str; 5])] = &[
    // lowercase
    ("a", [".##.", "...#", ".###", "#..#", ".###"]),
    ("b", ["##.", "#.#", "#.#", "#.#", "##."]),
    ("c", [".##.", "#...", "#...", "#...", ".##."]),
    ("d", ["...#", ".##.", "#..#", "#..#", ".##."]),
    ("e", [".##.", "#..#", "####", "#...", ".##."]),
    ("f", [".##", "#..", "###", "#..", "#.."]),
    ("g", [".##.", "#...", "#.##", "#..#", ".##."]),
    ("h", ["#..#", "#..#", "####", "#..#", "#..#"]),
    ("i", ["#.", "...", "#.", "#.", "#."]),
    ("j", ["..#", "...", "..#", "#.#", ".#."]),
    ("k", ["#.#", "#.#", "##.", "#.#", "#.#"]),
    ("l", ["##", "#.", "#.", "#.", "##"]),
    ("m", ["#...#", "##.##", "#.#.#", "#...#", "#...#"]),
    ("n", ["#...#", "##..#", "#.#.#", "#..##", "#...#"]),
    ("o", [".##.", "#..#", "#..#", "#..#", ".##."]),
    ("p", ["##.", "#.#", "#.#", "##.", "#.."]),
    ("q", [".##.", "#..#", "#.##", "..#.", "...#"]),
    ("r", [".##", "#.#", "#..", "#..", "#.."]),
    ("s", [".##.", "#...", ".##.", "...#", ".##."]),
    ("t", ["#.#", "#.#", ".#.", ".#.", ".##"]),
    ("u", ["#..#", "#..#", "#..#", "#..#", ".##."]),
    ("v", ["#..#", "#..#", "#..#", ".##.", ".##."]),
    ("w", ["#...#", "#...#", "#.#.#", "#.#.#", ".#.#."]),
    ("x", ["#..#", ".##.", ".##.", ".##.", "#..#"]),
    ("y", ["#..#", "#..#", ".###", "...#", ".##."]),
    ("z", ["###", "..#", ".#.", "#..", "###"]),
    // uppercase
    ("A", [".#.", "#.#", "###", "#.#", "#.#"]),
    ("B", ["##.", "#.#", "##.", "#.#", "##."]),
    ("C", [".##", "#..", "#..", "#..", ".##"]),
    ("D", ["##.", "#.#", "#.#", "#.#", "##."]),
    ("E", ["###", "#..", "##.", "#..", "###"]),
    ("F", ["###", "#..", "##.", "#..", "#.."]),
    ("G", [".##", "#..", "#.#", "#.#", ".##"]),
    ("H", ["#.#", "#.#", "###", "#.#", "#.#"]),
    ("I", ["###", ".#.", ".#.", ".#.", "###"]),
    ("J", ["..#", "..#", "..#", "#.#", ".#."]),
    ("K", ["#.#", "#.#", "##.", "#.#", "#.#"]),
    ("L", ["#..", "#..", "#..", "#..", "###"]),
    ("M", ["#...#", "##.##", "#.#.#", "#...#", "#...#"]),
    ("N", ["#...#", "##..#", "#.#.#", "#..##", "#...#"]),
    ("O", [".##.", "#..#", "#..#", "#..#", ".##."]),
    ("P", ["##.", "#.#", "##.", "#..", "#.."]),
    ("Q", [".##.", "#..#", "#.##", "..#.", "...#"]),
    ("R", ["##.", "#.#", "##.", "#.#", "#.#"]),
    ("S", [".##", "#..", ".#.", "..#", "##."]),
    ("T", ["###", ".#.", ".#.", ".#.", ".#."]),
    ("U", ["#..#", "#..#", "#..#", "#..#", ".##."]),
    ("V", ["#..#", "#..#", "#..#", ".##.", ".##."]),
    ("W", ["#...#", "#...#", "#.#.#", "#.#.#", ".#.#."]),
    ("X", ["#..#", ".##.", ".##.", ".##.", "#..#"]),
    ("Y", ["#.#", "#.#", "###", ".#.", ".#."]),
    ("Z", ["###", "..#", ".#.", "#..", "###"]),
    // digits
    ("0", ["###", "#.#", "#.#", "#.#", "###"]),
    ("1", [".#.", "##.", ".#.", ".#.", "###"]),
    ("2", ["##.", "..#", ".#.", "#..", "###"]),
    ("3", ["###", "..#", ".##", "..#", "###"]),
    ("4", ["#.#", "#.#", "###", "..#", "..#"]),
    ("5", ["###", "#..", "###", "..#", "###"]),
    ("6", [".##", "#..", "###", "#.#", "###"]),
    ("7", ["###", "..#", ".#.", ".#.", ".#."]),
    ("8", ["###", "#.#", "###", "#.#", "###"]),
    ("9", ["###", "#.#", "###", "..#", "##."]),
    // symbols
    (" ", ["..", "..", "..", "..", ".."]),
    (".", [".", ".", ".", ".", "#"]),
    (",", [".", ".", ".", ".", "*"]),
    (":", [".", "#", ".", "#", "."]),
    (";", ["#", ".", "#", "#", "."]),
    ("(", [".#", "#.", "#.", "#.", ".#"]),
    (")", ["#.", ".#", ".#", ".#", "#."]),
    ("-", ["...", "...", "###", "...", "..."]),
    ("+", ["...", ".#.", "###", ".#.", "..."]),
    ("_", ["...", "...", "...", "...", "###"]),
    ("/", ["..#", "..#", ".#.", "#..", "#.."]),
    ("!", ["#", "#", "#", ".", "#"]),
    ("?", ["##.", "..#", ".#.", "...", ".#."]),
    ("'", ["#", "#", ".", ".", "."]),
    ("♥", [".#.#.", "#####", "#####", ".###.", "..#.."]),
    ("♪", ["..##", "..#.", ".##.", "..#.", ".#.."]),
    ("%", [".#..", "#..#", ".#.", "#..#", ".#.."]),
    ("★", [".#.", "###", "###", "###", ".#."]),
];

pub fn glyph(ch: char) -> Option<&'static [&'static str; 5]> {
    let needle: String = ch.to_string();
    FONT.iter()
        .find(|(k, _)| *k == needle)
        .map(|(_, rows)| rows)
}

/// Big 5-row pixel text. `scale` = pixel stretch per axis (1 or 2). Returns width used.
pub fn big_text(g: &mut Grid, x0: i64, y0: i64, s: &str, fg: Color, scale: i64, alpha: f64) -> i64 {
    let mut x = x0;
    for ch in s.chars() {
        if let Some(rows) = glyph(ch) {
            let gw = rows[0].chars().count() as i64;
            for (ry, row) in rows.iter().enumerate() {
                for (rx, pc) in row.chars().enumerate() {
                    if pc != '#' {
                        continue;
                    }
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = x + rx as i64 * scale + sx;
                            let py = y0 + ry as i64 * scale + sy;
                            if alpha >= 1.0 {
                                g.put_bold(px, py, '█', fg);
                            } else {
                                g.put_alpha(px, py, '█', fg, alpha);
                            }
                        }
                    }
                }
            }
            x += gw * scale;
        }
    }
    x
}

#[allow(dead_code)]
pub fn big_text_center(g: &mut Grid, y0: i64, s: &str, fg: Color, scale: i64, alpha: f64) {
    let w = big_text_width(s, scale);
    let x = (g.w as i64 - w) / 2;
    big_text(g, x, y0, s, fg, scale, alpha);
}

pub fn big_text_width(s: &str, scale: i64) -> i64 {
    s.chars()
        .map(|c| glyph(c).map(|r| r[0].chars().count() as i64).unwrap_or(2))
        .sum::<i64>()
        * scale
}

// ---------------------------------------------------------------- sprites ---

pub struct Sprite {
    pub rows: &'static [&'static str],
    /// char -> color (hex). Unmapped chars are skipped (treated as transparent).
    pub pal: &'static [(char, u32)],
}

/// Draw a sprite. `scale` = horizontal stretch (1 or 2); `solid` duplicates the char
/// into the neighbour column for a chunky look.
pub fn draw(
    g: &mut Grid,
    spr: &Sprite,
    ox: i64,
    oy: i64,
    scale: i64,
    solid: bool,
    alpha: f64,
    tint: Option<Color>,
) {
    for (ry, row) in spr.rows.iter().enumerate() {
        let y = oy + ry as i64;
        for (rx, ch) in row.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let color = match tint {
                Some(c) => c,
                None => match spr.pal.iter().find(|(k, _)| *k == ch) {
                    Some((_, hex)) => Color::hex(*hex),
                    None => continue,
                },
            };
            let x = ox + rx as i64 * scale;
            for k in 0..scale.max(solid as i64) {
                if alpha >= 1.0 {
                    g.put(x + k, y, ch, color);
                } else {
                    g.put_alpha(x + k, y, ch, color, alpha);
                }
            }
        }
    }
}

pub fn sprite_size(spr: &Sprite, scale: i64) -> (i64, i64) {
    let w = spr
        .rows
        .iter()
        .map(|r| r.chars().count() as i64)
        .max()
        .unwrap_or(0)
        * scale;
    (w, spr.rows.len() as i64)
}

// ------------------------------------------------------------ the assets ---

pub const TABLE_TOP: Sprite = Sprite {
    rows: &[
        " ╭───────────────╮",
        " ╰───────────────╯",
    ],
    pal: &[
        ('╭', 0xe0aa72),
        ('╮', 0xe0aa72),
        ('╰', 0x8a6242),
        ('╯', 0x8a6242),
        ('─', 0xd9a066),
    ],
};

pub const EGGPLANT: Sprite = Sprite {
    rows: &[
        "      |     ",
        "     \\|/    ",
        "   \\  |  /  ",
        " ,%%%%%%%,  ",
        " (%%%%%%%%%)",
        "(%%%%%%%%%%)",
        "(%%%%%%%%%%)",
        " `%%%%%%%,' ",
        "   `----'   ",
    ],
    pal: &[
        ('|', 0x5f9e57),
        ('(', 0x4a2d73),
        (')', 0x4a2d73),
        ('\\', 0x5f9e57),
        ('/', 0x5f9e57),
        ('%', 0x9a6dd7),
        ('\'', 0xe8a7c3),
        ('`', 0x4a2d73),
        (',', 0x4a2d73),
        ('-', 0x4a2d73),
    ],
};

pub const CAT_A: Sprite = Sprite {
    rows: &[
        " /\\___/\\    ",
        "(  o o  )~   ",
        "(   ω   )    ",
        " \\_____//    ",
        " /      \\    ",
        "|         |  ",
        "|  *   *  |  ",
        "|         |  ",
        "(__)  (__)   ",
    ],
    pal: &[
        ('/', 0x6f87b8),
        ('\\', 0x6f87b8),
        ('(', 0xcfd8e6),
        (')', 0xcfd8e6),
        ('o', 0x2b3a55),
        ('ω', 0xe89aa4),
        ('~', 0x6f87b8),
        ('*', 0x6f87b8),
        ('|', 0xcfd8e6),
        ('_', 0x9fb2cf),
    ],
};

pub const CAT_B: Sprite = Sprite {
    rows: &[
        " /\\___/\\    ",
        "(  o o  )    ",
        "(   ω   )    ",
        " \\_____/     ",
        " /      \\    ",
        "|         |~ ",
        "|  *   *  |  ",
        "|         |  ",
        "(__)  (__)   ",
    ],
    pal: &[
        ('/', 0x6f87b8),
        ('\\', 0x6f87b8),
        ('(', 0xcfd8e6),
        (')', 0xcfd8e6),
        ('o', 0x2b3a55),
        ('ω', 0xe89aa4),
        ('~', 0x6f87b8),
        ('*', 0x6f87b8),
        ('|', 0xcfd8e6),
        ('_', 0x9fb2cf),
    ],
};

/// cat with open mouth (meow) — row2 ω→O
pub const CAT_MEOW: Sprite = Sprite {
    rows: &[
        " /\\___/\\    ",
        "(  o o  )    ",
        "(   O   )    ",
        " \\_____/     ",
        " /      \\    ",
        "|         |  ",
        "|  *   *  |~ ",
        "|         |  ",
        "(__)  (__)   ",
    ],
    pal: &[
        ('/', 0x6f87b8),
        ('\\', 0x6f87b8),
        ('(', 0xcfd8e6),
        (')', 0xcfd8e6),
        ('o', 0x2b3a55),
        ('O', 0xe89aa4),
        ('~', 0x6f87b8),
        ('*', 0x6f87b8),
        ('|', 0xcfd8e6),
        ('_', 0x9fb2cf),
    ],
};

/// hissing cat — eyes squeezed
pub const CAT_HISS: Sprite = Sprite {
    rows: &[
        " /\\___/\\    ",
        "(  > <  )    ",
        "(   ω   )    ",
        " \\_____/     ",
        " /      \\    ",
        "|         |~ ",
        "|  *   *  |  ",
        "|         |  ",
        "(__)  (__)   ",
    ],
    pal: &[
        ('/', 0x6f87b8),
        ('\\', 0x6f87b8),
        ('(', 0xcfd8e6),
        (')', 0xcfd8e6),
        ('>', 0x2b3a55),
        ('<', 0x2b3a55),
        ('ω', 0xe89aa4),
        ('~', 0x6f87b8),
        ('*', 0x6f87b8),
        ('|', 0xcfd8e6),
        ('_', 0x9fb2cf),
    ],
};

/// asleep cat — purr? never! (only used for the joke)
pub const CAT_SLEEP: Sprite = Sprite {
    rows: &[
        " /\\___/\\    ",
        "(  - -  )    ",
        "(   ω   )    ",
        " \\_____/     ",
        " /      \\    ",
        "|         |~ ",
        "|  *   *  |  ",
        "|         |  ",
        "(__)  (__)   ",
    ],
    pal: &[
        ('/', 0x6f87b8),
        ('\\', 0x6f87b8),
        ('(', 0xcfd8e6),
        (')', 0xcfd8e6),
        ('-', 0x2b3a55),
        ('ω', 0xe89aa4),
        ('~', 0x6f87b8),
        ('*', 0x6f87b8),
        ('|', 0xcfd8e6),
        ('_', 0x9fb2cf),
    ],
};

pub const STEAK: Sprite = Sprite {
    rows: &[
    "    _,-----------._    ",
    "  ,'                 ', ",
    " /    ~o~      o~      \\",
    "|   o   ≡≡≡≡    ~o~     |",
    "|     ≡≡≡≡≡≡   o        |",
    "|  ~o    ≡≡≡≡     ~o~   |",
    " \\        ≡≡≡          /",
    "  ',                 ,'  ",
    "    '-----------´---'    ",
    "  ,--------------------, ",
    " (                      )",
    "  `--------------------' ",
    ],
    pal: &[
        ('_', 0x8a5a3a),
        (',', 0x8a5a3a),
        ('\'', 0x8a5a3a),
        ('/', 0x8a5a3a),
        ('\\', 0x8a5a3a),
        ('|', 0x8a5a3a),
        ('≈', 0xb0563e),
        ('≡', 0x6b3b28),
        ('o', 0xc9a86a),
        ('~', 0xa8825a),
        ('(', 0xd8d8de),
        (')', 0xd8d8de),
        ('`', 0xd8d8de),
        ('-', 0xb9b9c4),
        ('´', 0x8a5a3a),
    ],
};

// flower frames: bud -> half -> full bloom
pub const FLOWER_0: Sprite = Sprite {
    rows: &[
        "      |      ",
        "    , | ,    ",
        "   (  |  )   ",
        "   (  |  )   ",
        "    ( | )    ",
        "     ( )     ",
        "     \\ | /    ",
        "      |      ",
        "     _|_     ",
    ],
    pal: &[
        ('|', 0x5a9e6e),
        ('(', 0xd88aa8),
        (')', 0xd88aa8),
        (',', 0xe89ab8),
        ('\\', 0x5a9e6e),
        ('/', 0x5a9e6e),
        ('_', 0x5a9e6e),
    ],
};

pub const FLOWER_1: Sprite = Sprite {
    rows: &[
        "  .   *   .  ",
        " (  .---.  ) ",
        " ( (  @  ) ) ",
        "  (  '---'  ) ",
        "   `  *  '   ",
        "     \\ |/    ",
        "      |      ",
        "     _|_     ",
    ],
    pal: &[
        ('|', 0x5a9e6e),
        ('(', 0xf2a7c3),
        (')', 0xf2a7c3),
        ('.', 0xf2a7c3),
        ('-', 0xe89ab8),
        ('\'', 0xd88aa8),
        ('`', 0xd88aa8),
        ('\\', 0x5a9e6e),
        ('/', 0x5a9e6e),
        ('_', 0x5a9e6e),
        ('*', 0xf7c2d6),
        ('@', 0xf2c14e),
    ],
};

pub const FLOWER_2: Sprite = Sprite {
    rows: &[
        "   _  *  _   ",
        "  ( \\   / )  ",
        " *  \\ /  . * ",
        " *  ( @ )  * ",
        " .  /   \\  . ",
        "  ( /   \\ )  ",
        "   `  *  '   ",
        "     \\ |/    ",
        "      |      ",
        "     _|_     ",
    ],
    pal: &[
        ('|', 0x5a9e6e),
        ('(', 0xf2a7c3),
        (')', 0xf2a7c3),
        ('.', 0xe89ab8),
        ('-', 0xeda3bf),
        ('\'', 0xd88aa8),
        ('`', 0xd88aa8),
        ('\\', 0x5a9e6e),
        ('/', 0x5a9e6e),
        ('_', 0x5a9e6e),
        ('*', 0xf7c2d6),
        ('@', 0xf2c14e),
    ],
};

pub const HUMAN: Sprite = Sprite {
    rows: &[
        "   ,,^~~^,,   ",
        "  (  ^ ^  )   ",
        "  (   ~   )   ",
        "   `  u  `    ",
        "  __|   |__   ",
        " o |     | o ",
        " o |     | o ",
        "  \\|_____|/   ",
        "   |     |    ",
        "   |     |    ",
        "  _|     |_   ",
    ],
    pal: &[
        ('(', 0xe8c39e),
        (')', 0xe8c39e),
        ('|', 0xe8c39e),
        ('_', 0xc9a06a),
        ('-', 0xc9a06a),
        ('~', 0xd9a86a),
        ('^', 0xd9a86a),
        (',', 0xd9a86a),
        ('u', 0xc9526a),
        ('`', 0xe8c39e),
        ('o', 0xff9e5e),
        ('/', 0xe8c39e),
        ('\\', 0xe8c39e),
    ],
};

pub const DOG_A: Sprite = Sprite {
    rows: &[
        " .\\   /. ",
        " ( o o )  ",
        " (  ω  )  ",
        "  `---'   ",
        " /     \\  ",
        "|       |~",
        "| *   * | ",
        "(__) (__) ",
    ],
    pal: &[
        ('.', 0x8a6242),
        ('\\', 0x8a6242),
        ('/', 0x8a6242),
        ('(', 0xd9b98a),
        (')', 0xd9b98a),
        ('o', 0x3a2c1e),
        ('ω', 0xe89aa4),
        ('`', 0xb99a6a),
        ('-', 0xb99a6a),
        ('|', 0xd9b98a),
        ('~', 0xb99a6a),
        ('*', 0xb99a6a),
        ('_', 0x9fb2cf),
    ],
};

pub const DOG_B: Sprite = Sprite {
    rows: &[
        " .\\   /. ",
        " ( o o )  ",
        " (  ω  )  ",
        "  `---'   ",
        " /     \\  ",
        "|       | ",
        "| *   * |~",
        "(__) (__) ",
    ],
    pal: &[
        ('.', 0x8a6242),
        ('\\', 0x8a6242),
        ('/', 0x8a6242),
        ('(', 0xd9b98a),
        (')', 0xd9b98a),
        ('o', 0x3a2c1e),
        ('ω', 0xe89aa4),
        ('`', 0xb99a6a),
        ('-', 0xb99a6a),
        ('|', 0xd9b98a),
        ('~', 0xb99a6a),
        ('*', 0xb99a6a),
        ('_', 0x9fb2cf),
    ],
};

pub const TOMATO: Sprite = Sprite {
    rows: &[
        "    ,-,    ",
        "   /|x|\\   ",
        "  /x|x|x\\  ",
        " (   ~   ) ",
        "(   ~~~   )",
        "(  ~~~~~  )",
        " ( ~~~~~ ) ",
        "  `~~~~~'  ",
    ],
    pal: &[
        (',', 0x5f9e57),
        ('/', 0x5f9e57),
        ('\\', 0x5f9e57),
        ('|', 0x5f9e57),
        ('x', 0x5f9e57),
        ('(', 0xc94436),
        (')', 0xc94436),
        ('~', 0xe86a5e),
        ('`', 0xc94436),
        ('\'', 0xc94436),
    ],
};

/// bucket figure for the "bucket of love" verse; fill drawn procedurally
pub const BUCKET: Sprite = Sprite {
    rows: &[
        " ______ ",
        "(      )",
        " `----' ",
        "   ||   ",
        "   ||   ",
        "  (oo)  ",
        " /|  |\\ ",
        "  |  |  ",
        "  d  b  ",
    ],
    pal: &[
        ('_', 0x8a93a6),
        ('(', 0x8a93a6),
        (')', 0x8a93a6),
        ('`', 0x8a93a6),
        ('-', 0x8a93a6),
        ('|', 0x8a93a6),
        ('o', 0xe8c39e),
        ('\\', 0x8a93a6),
        ('/', 0x8a93a6),
        ('d', 0x8a93a6),
        ('b', 0x8a93a6),
    ],
};

/// merged creature: cat + flower + eggplant soul
pub const CHIMERA: Sprite = Sprite {
    rows: &[
        "    _ * _    ",
        "  .*  @  *.  ",
        "  ( * | * )  ",
        "   `._|_.'   ",
        "  /\\___/\\    ",
        " (  o o  )   ",
        " (   ω   )   ",
        "  \\_____//   ",
        " /       \\   ",
        "|  %   %  |~ ",
        "|         |  ",
        "(__)  (__)   ",
    ],
    pal: &[
        ('*', 0xf5c6d8),
        ('.', 0xf2a7c3),
        ('(', 0x9fb2cf),
        (')', 0x9fb2cf),
        ('_', 0x9fb2cf),
        ('\\', 0x6f87b8),
        ('/', 0x6f87b8),
        ('o', 0x2b3a55),
        ('ω', 0xe89aa4),
        ('%', 0x9a6dd7),
        ('|', 0xcfd8e6),
        ('@', 0xf2c14e),
        ('~', 0x6f87b8),
        ('`', 0xd88aa8),
    ],
};

// ------------------------------------------------------------- helpers ----

/// bubble ring by radius (1..=3)
pub fn bubble(g: &mut Grid, cx: i64, cy: i64, r: i64, color: Color, alpha: f64) {
    match r {
        1 => {
            g.put_alpha(cx, cy, 'o', color, alpha);
        }
        2 => {
            g.put_alpha(cx - 1, cy - 1, '_', color, alpha);
            g.put_alpha(cx - 1, cy, '(', color, alpha);
            g.put_alpha(cx, cy, '.', color, alpha * 0.8);
            g.put_alpha(cx + 1, cy, ')', color, alpha);
            g.put_alpha(cx, cy + 1, '`', color, alpha);
            g.put_alpha(cx + 1, cy + 1, '´', color, alpha);
        }
        _ => {
            g.put_alpha(cx - 2, cy - 2, ',', color, alpha);
            g.put_alpha(cx - 1, cy - 2, '-', color, alpha);
            g.put_alpha(cx, cy - 2, '.', color, alpha);
            g.put_alpha(cx - 2, cy - 1, '/', color, alpha);
            g.put_alpha(cx + 1, cy - 1, '\\', color, alpha);
            g.put_alpha(cx - 3, cy, '(', color, alpha);
            g.put_alpha(cx + 2, cy, ')', color, alpha);
            g.put_alpha(cx - 3, cy + 1, '(', color, alpha);
            g.put_alpha(cx + 2, cy + 1, ')', color, alpha);
            g.put_alpha(cx - 2, cy + 2, '`', color, alpha);
            g.put_alpha(cx - 1, cy + 2, '-', color, alpha);
            g.put_alpha(cx, cy + 2, '´', color, alpha);
        }
    }
}

/// pop flash
pub fn pop_burst(g: &mut Grid, cx: i64, cy: i64, t: f64, color: Color) {
    let chars = ['*', '°', 'º', '\'', '·'];
    for k in 0..8 {
        let a = k as f64 * std::f64::consts::PI / 4.0;
        let d = 1.0 + t * 5.0;
        let x = cx + (a.cos() * d).round() as i64;
        let y = cy + (a.sin() * d * 0.6).round() as i64;
        g.put_alpha(x, y, chars[(k as usize) % chars.len()], color, (1.0 - t).max(0.0));
    }
}
