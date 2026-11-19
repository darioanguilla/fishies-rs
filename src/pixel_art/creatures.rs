//! Pixel art for the random objects. Everything is drawn facing right; use
//! [`super::facing`] for the other direction.

use std::f32::consts::TAU;

use super::{BLACK, WHITE, finish, grid};
use crate::color::{self, Rgb, shade};
use crate::engine::PixFrame;
use crate::engine::shape::perl_lines;

/// 25 x 12
pub fn ship() -> PixFrame {
    let wood = (125, 78, 40);
    let sail = (240, 236, 222);
    let mast = (95, 62, 32);
    let red = (215, 50, 45);
    let mut f = PixFrame::new(25, 12);
    f.poly(&[(0.0, 8.0), (25.0, 8.0), (21.0, 12.0), (2.0, 12.0)], wood);
    f.line((0.0, 8.0), (24.0, 8.0), (225, 180, 60));
    for x in [5, 9, 13, 17] {
        f.dot(x, 10, (45, 30, 20));
    }
    // sails billowing towards the bow, and a jib
    for m in [5.0, 10.0, 15.0] {
        f.poly(
            &[
                (m - 2.0, 2.0),
                (m + 2.0, 2.0),
                (m + 3.2, 5.0),
                (m + 2.0, 8.0),
                (m - 2.0, 8.0),
                (m - 1.0, 5.0),
            ],
            sail,
        );
        f.line((m - 1.0, 5.0), (m + 2.0, 5.0), shade(sail, 0.85));
    }
    f.poly(&[(16.5, 2.0), (23.5, 8.0), (17.5, 8.0)], sail);
    for m in [5, 10, 15] {
        f.line((m as f32, 0.0), (m as f32, 7.0), mast);
        f.dot(m + 1, 0, red);
        f.dot(m + 2, 0, red);
        f.dot(m + 1, 1, red);
    }
    finish(f)
}

/// The whale's frames: 5 without water spout, then the 7 spout frames.
/// 19 x 14, the body in the bottom 8 rows.
pub fn whale() -> Vec<PixFrame> {
    let mut body = PixFrame::new(19, 14);
    let blue = (52, 84, 160);
    let oy = 6.0;
    body.poly(
        &[
            (5.0, oy + 7.9),
            (5.0, oy + 4.0),
            (8.0, oy + 1.0),
            (12.0, oy + 0.2),
            (15.5, oy + 0.8),
            (18.0, oy + 3.0),
            (19.0, oy + 6.0),
            (18.0, oy + 7.9),
        ],
        blue,
    );
    body.poly(
        &[
            (6.0, oy + 5.0),
            (1.0, oy + 2.5),
            (0.0, oy + 4.0),
            (2.0, oy + 7.9),
            (6.5, oy + 7.6),
        ],
        blue,
    );
    body.poly(
        &[
            (8.0, oy + 7.9),
            (10.0, oy + 6.3),
            (18.5, oy + 6.0),
            (18.0, oy + 7.9),
        ],
        (160, 185, 215),
    );
    body.line((12.0, oy + 5.5), (18.0, oy + 5.5), (30, 50, 110));
    body.dot(14, 10, WHITE);
    body.dot(15, 10, BLACK);
    let body = finish(body);

    let water = (150, 215, 250);
    let spray = (225, 245, 255);
    let x = 14;
    // (x offset, y, is spray) for each spout frame
    let spouts: [&[(i32, i32, bool)]; 7] = [
        &[(0, 5, false)],
        &[(0, 3, false), (0, 4, false), (0, 5, false)],
        &[
            (0, 1, true),
            (-1, 1, true),
            (1, 1, true),
            (0, 2, false),
            (0, 3, false),
            (0, 4, false),
            (0, 5, false),
        ],
        &[
            (-2, 0, true),
            (2, 0, true),
            (-1, 1, true),
            (1, 1, true),
            (0, 1, false),
            (0, 2, false),
            (0, 3, false),
            (0, 4, false),
            (0, 5, false),
        ],
        &[
            (-3, 1, true),
            (3, 1, true),
            (-2, 0, true),
            (2, 0, true),
            (-1, 2, true),
            (1, 2, true),
            (0, 1, true),
            (0, 3, false),
            (0, 4, false),
            (0, 5, false),
        ],
        &[
            (-3, 2, true),
            (3, 2, true),
            (-2, 3, true),
            (2, 3, true),
            (0, 4, false),
            (0, 5, false),
        ],
        &[(-3, 4, true), (3, 4, true), (-2, 5, true), (2, 5, true)],
    ];
    let mut frames = vec![body.clone(); 5];
    for drops in spouts {
        let mut f = body.clone();
        for &(dx, y, is_spray) in drops {
            f.dot(x + dx, y, if is_spray { spray } else { water });
        }
        frames.push(f);
    }
    frames
}

/// One of the 4 sea monster frames, 66 x 10: humps rolling along its body.
pub fn monster(frame: usize) -> PixFrame {
    let green = (70, 165, 80);
    let dark = (40, 110, 55);
    let mut f = PixFrame::new(66, 10);
    for x in 4..54 {
        let s = (x as f32 * TAU / 22.0 - frame as f32 * TAU / 4.0).sin();
        if s <= 0.0 {
            continue; // under water
        }
        let top = 9.5 - 8.0 * s;
        for y in top.floor() as i32..(top + 3.0) as i32 {
            f.dot(x, y, green);
        }
        if x % 4 == 0 && s > 0.3 {
            f.dot(x, top.floor() as i32 - 1, dark); // spines
        }
    }
    f.poly(
        &[
            (52.0, 10.0),
            (55.0, 3.0),
            (58.0, 2.0),
            (58.0, 5.0),
            (56.5, 10.0),
        ],
        green,
    );
    f.ellipse(60.5, 2.8, 4.5, 2.4, green);
    f.poly(&[(60.0, 2.0), (66.0, 3.0), (66.0, 5.2), (60.0, 5.2)], green);
    f.line((61.0, 4.0), (65.0, 4.0), dark);
    f.dot(61, 1, WHITE);
    f.dot(62, 1, BLACK);
    f.poly(&[(1.0, 10.0), (0.0, 5.0), (2.0, 4.0), (4.0, 10.0)], green);
    finish(f)
}

/// 34 x 28. Colors come from the fish's palette; the spots sit where the
/// ASCII color mask has its `2`s.
pub fn big_fish(palette: &[char; 9], ascii_mask: &str) -> PixFrame {
    let body = color::letter_rgb(palette[0]).unwrap_or((220, 180, 60));
    let spot = color::letter_rgb(palette[1]).unwrap_or(WHITE);
    let fin = shade(body, 0.75);
    let mut f = PixFrame::new(34, 28);
    // a forked tail
    f.poly(&[(11.0, 14.0), (0.0, 2.0), (6.0, 3.0), (13.0, 9.0)], fin);
    f.poly(&[(11.0, 14.0), (0.0, 26.0), (6.0, 25.0), (13.0, 19.0)], fin);
    f.poly(&[(6.0, 10.0), (12.0, 9.0), (12.0, 19.0), (6.0, 18.0)], fin);
    f.poly(&[(9.0, 7.0), (15.0, 0.5), (27.0, 1.5), (30.0, 5.5)], fin);
    f.poly(&[(14.0, 23.0), (22.0, 24.0), (12.0, 27.5)], fin);
    f.ellipse(20.0, 14.0, 13.5, 10.5, body);
    for (row, line) in perl_lines(ascii_mask.strip_prefix('\n').unwrap_or(ascii_mask))
        .iter()
        .enumerate()
    {
        for (col, c) in line.chars().enumerate() {
            let y = row * 2 + (col + row) % 2;
            if c == '2' && f.get(col, y) == Some(body) {
                f.set(col, y, Some(spot));
            }
        }
    }
    f.poly(
        &[(20.0, 15.0), (25.0, 17.0), (19.0, 20.0)],
        shade(body, 0.8),
    );
    for y in 8..21 {
        let x = 23.0 + (y as f32 - 14.0).powi(2) / 25.0;
        f.dot(x as i32, y, shade(body, 0.6));
    }
    f.ellipse(28.0, 10.0, 1.6, 1.6, WHITE);
    f.dot(29, 10, BLACK);
    f.line((31.0, 15.0), (33.0, 14.0), shade(body, 0.45));
    finish(f)
}

/// 53 x 20
pub fn shark() -> PixFrame {
    let grey = (110, 130, 150);
    let belly = (205, 210, 215);
    let dark = (60, 70, 85);
    let mut f = PixFrame::new(53, 20);
    // tail and fins
    f.poly(&[(8.0, 12.0), (0.0, 3.0), (3.0, 3.5), (11.0, 10.5)], grey);
    f.poly(&[(8.0, 13.0), (2.0, 19.5), (5.0, 19.5), (11.0, 14.0)], grey);
    f.poly(&[(27.0, 8.5), (31.0, 0.0), (34.0, 1.0), (38.0, 8.0)], grey);
    f.poly(&[(23.0, 16.0), (31.0, 16.0), (24.0, 19.9)], dark);
    // body, pale belly
    f.poly(
        &[
            (6.0, 12.0),
            (14.0, 9.0),
            (25.0, 8.0),
            (38.0, 7.5),
            (46.0, 9.0),
            (52.5, 12.5),
            (50.0, 15.0),
            (42.0, 17.0),
            (28.0, 17.5),
            (16.0, 16.0),
            (7.0, 14.0),
        ],
        grey,
    );
    f.poly(
        &[
            (14.0, 15.0),
            (30.0, 14.5),
            (44.0, 14.0),
            (51.0, 13.5),
            (48.0, 16.0),
            (40.0, 17.0),
            (28.0, 17.5),
            (16.0, 16.0),
        ],
        belly,
    );
    for x in [36.0, 38.0, 40.0] {
        f.line((x, 11.0), (x - 0.5, 13.0), dark);
    }
    f.dot(45, 11, (220, 40, 40));
    f.dot(46, 11, BLACK);
    // jaw and teeth
    f.line((40.0, 15.0), (51.0, 13.5), (70, 30, 35));
    for x in (41..51).step_by(2) {
        f.dot(x, 14, WHITE);
    }
    finish(f)
}

const DUCK: [&str; 6] = [
    "......gg.",
    ".....gkgY",
    "w.....gw.",
    "ww..wwww.",
    "wwWWWwww.",
    ".wwwwww..",
];
/// The quacking duck has its beak open.
const DUCK_QUACK: [&str; 6] = [
    "......gg.",
    ".....gkgY",
    "w.....gwY",
    "ww..wwww.",
    "wwWWWwww.",
    ".wwwwww..",
];

/// One of the 3 frames of three ducks, 31 x 6: a different duck quacks in
/// each frame, like in the ASCII art.
pub fn ducks(frame: usize) -> PixFrame {
    let color = |c| match c {
        'w' => Some((240, 240, 235)),
        'W' => Some((200, 200, 195)),
        'g' => Some((40, 140, 70)),
        'k' => Some(BLACK),
        'c' => Some((40, 140, 70)),
        'Y' => Some((240, 180, 40)),
        _ => None,
    };
    let mut f = PixFrame::new(31, 6);
    for (i, x0) in [0usize, 11, 22].into_iter().enumerate() {
        let rows = if i == 2 - frame { DUCK_QUACK } else { DUCK };
        let duck = grid(&rows, color);
        for y in 0..duck.height {
            for x in 0..duck.width {
                if let Some(c) = duck.get(x, y) {
                    f.set(x0 + x, y, Some(c));
                }
            }
        }
    }
    // small and white: a lighter touch than [`finish`]
    f.outlined(0.85).shaded(1.1, 0.9)
}

/// Dolphin frames (tail down, tail up), 14 x 8, in the dolphin's color.
pub fn dolphin(base: Rgb) -> Vec<PixFrame> {
    const FRAMES: [[&str; 8]; 2] = [
        [
            "........b.....",
            ".......bb.....",
            "....bbbbbbb...",
            "...bbbbbbbbkb.",
            "..bbbbbbbbbbbb",
            ".bbblllllllb..",
            "bb...lll......",
            "b.............",
        ],
        [
            "b.......b.....",
            "bb.....bb.....",
            ".bb.bbbbbbb...",
            "..bbbbbbbbbkb.",
            "..bbbbbbbbbbbb",
            "...blllllllb..",
            ".....lll......",
            "..............",
        ],
    ];
    let belly = shade(base, 1.7);
    FRAMES
        .iter()
        .map(|rows| {
            finish(grid(rows, |c| match c {
                'b' => Some(base),
                'l' => Some(belly),
                'k' => Some(BLACK),
                _ => None,
            }))
        })
        .collect()
}

/// 11 x 14
pub fn swan() -> PixFrame {
    const ROWS: [&str; 14] = [
        ".......www.",
        "......wwkYY",
        "w.....ww...",
        "ww....ww...",
        "ww....ww...",
        "ww...ww....",
        "www..ww....",
        "wwww.www...",
        "wwwwwwwwww.",
        "wwwWWWwwwww",
        "wwWWWWWwwww",
        "wwwWWWwwwww",
        ".wwwwwwwwww",
        "..wwwwwwww.",
    ];
    finish(grid(&ROWS, |c| match c {
        'w' => Some((245, 245, 240)),
        'W' => Some((205, 205, 212)),
        'k' => Some(BLACK),
        'Y' => Some((235, 140, 40)),
        _ => None,
    }))
}
