//! Pixel art for the random objects, at both resolutions (see [`Pen`]).
//! Everything is drawn facing right; use [`super::facing`] for the other
//! direction.

use std::f32::consts::TAU;

use super::pen::{Pen, Res};
use super::{BLACK, WHITE, finish, grid};
use crate::color::{self, Rgb, shade};
use crate::engine::PixFrame;
use crate::engine::shape::perl_lines;

/// 25 x 12
pub fn ship(res: Res) -> PixFrame {
    let wood = (125, 78, 40);
    let sail = (240, 236, 222);
    let mast = (95, 62, 32);
    let red = (215, 50, 45);
    let mut p = Pen::new(25, 12, res);
    p.poly(&[(0.0, 8.0), (25.0, 8.0), (21.0, 12.0), (2.0, 12.0)], wood);
    if p.high() {
        // planks
        for y in [9.5, 10.75] {
            p.hairline((1.0, y), (23.5, y), shade(wood, 0.8));
        }
    }
    p.line((0.0, 8.0), (24.0, 8.0), (225, 180, 60));
    for x in [5, 9, 13, 17] {
        if p.high() {
            p.disc(x as f32 + 0.5, 10.4, 0.45, (45, 30, 20));
        } else {
            p.dot(x, 10, (45, 30, 20));
        }
    }
    // sails billowing towards the bow, and a jib; each sail's back edge
    // follows the front edge of the one before, so there are no gaps
    for m in [5.0, 10.0, 15.0] {
        p.poly(
            &[
                (m - 3.0, 2.0),
                (m + 2.0, 2.0),
                (m + 3.2, 5.0),
                (m + 2.0, 8.0),
                (m - 3.0, 8.0),
                (m - 1.8, 5.0),
            ],
            sail,
        );
        p.line((m - 1.0, 5.0), (m + 2.0, 5.0), shade(sail, 0.85));
    }
    p.poly(&[(16.5, 2.0), (23.5, 8.0), (17.5, 8.0)], sail);
    if p.high() {
        // rigging
        p.hairline((15.5, 0.2), (24.5, 8.0), (90, 70, 50));
        p.hairline((5.5, 0.2), (0.5, 8.0), (90, 70, 50));
    }
    for m in [5, 10, 15] {
        p.line((m as f32, 0.0), (m as f32, 7.0), mast);
        p.dot(m + 1, 0, red);
        p.dot(m + 2, 0, red);
        p.dot(m + 1, 1, red);
    }
    finish(p.f)
}

/// The whale's frames: 5 without water spout, then the 7 spout frames.
/// 19 x 14, the body in the bottom 8 rows.
pub fn whale(res: Res) -> Vec<PixFrame> {
    let blue = (52, 84, 160);
    let oy = 6.0;
    let mut p = Pen::new(19, 14, res);
    p.poly(
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
    p.poly(
        &[
            (6.0, oy + 5.0),
            (1.0, oy + 2.5),
            (0.0, oy + 4.0),
            (2.0, oy + 7.9),
            (6.5, oy + 7.6),
        ],
        blue,
    );
    p.poly(
        &[
            (8.0, oy + 7.9),
            (10.0, oy + 6.3),
            (18.5, oy + 6.0),
            (18.0, oy + 7.9),
        ],
        (160, 185, 215),
    );
    if p.high() {
        // throat grooves
        for i in 0..3 {
            let y = oy + 6.6 + i as f32 * 0.4;
            p.hairline((11.0, y), (17.5, y - 0.2), (130, 155, 190));
        }
        p.disc(14.5, oy + 4.5, 0.6, WHITE);
        p.disc(14.7, oy + 4.5, 0.3, BLACK);
    } else {
        p.dot(14, 10, WHITE);
        p.dot(15, 10, BLACK);
    }
    p.line((12.0, oy + 5.5), (18.0, oy + 5.5), (30, 50, 110));
    let body = finish(p.f);

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
        let mut p = Pen::new(19, 14, res);
        p.f = body.clone();
        for &(dx, y, is_spray) in drops {
            let c = if is_spray { spray } else { water };
            if p.high() && is_spray {
                p.disc((x + dx) as f32 + 0.5, y as f32 + 0.5, 0.4, c);
            } else if p.high() {
                // the jet: a column, narrower at the top
                let w = 0.25 + 0.06 * y as f32;
                p.poly(
                    &[
                        ((x + dx) as f32 + 0.5 - w, y as f32),
                        ((x + dx) as f32 + 0.5 + w, y as f32),
                        ((x + dx) as f32 + 0.5 + w, y as f32 + 1.0),
                        ((x + dx) as f32 + 0.5 - w, y as f32 + 1.0),
                    ],
                    c,
                );
            } else {
                p.dot(x + dx, y, c);
            }
        }
        frames.push(p.f);
    }
    frames
}

/// Number of sea monster frames (the pixel frames run 4 times as fast as
/// its 4 ASCII frames).
pub const MONSTER_FRAMES: usize = 16;

/// Sea monster frame `frame` of [`MONSTER_FRAMES`], 66 x 10: a serpent whose
/// body swims in a sine wave travelling from the neck to the tail, swinging
/// wider towards the tail; the head rides gently above the water.
pub fn monster(frame: usize, res: Res) -> PixFrame {
    let green = (70, 165, 80);
    let belly = (150, 210, 120);
    let dark = (40, 110, 55);
    let phase = frame as f32 / MONSTER_FRAMES as f32 * TAU;
    let (tail, neck) = (2.0f32, 50.0f32);
    // 0 at the neck, 1 at the tail
    let along = move |x: f32| ((neck - x) / (neck - tail)).clamp(0.0, 1.0);
    let wave = move |x: f32| 6.0 + (0.6 + 2.4 * along(x)) * (x * TAU / 20.0 + phase).sin();
    let half = move |x: f32| 0.6 + 1.3 * (1.0 - along(x));

    let mut p = Pen::new(66, 10, res);
    p.fill(green, |x, y| {
        x >= tail && x <= neck + 0.5 && (y - wave(x)).abs() <= half(x)
    });
    p.fill(belly, |x, y| {
        let d = y - wave(x);
        x >= tail && x <= neck && d > half(x) * 0.3 && d <= half(x)
    });
    // spines along the back
    for i in 0..11 {
        let x = 7.0 + i as f32 * 4.0;
        let top = wave(x) - half(x);
        p.poly(
            &[(x - 0.8, top + 0.4), (x, top - 1.0), (x + 0.8, top + 0.4)],
            dark,
        );
    }
    // the neck curves up from the end of the body to the head
    let head_y = 2.8 + 0.3 * phase.sin();
    let (p0, p1, p2) = (
        (neck - 1.0, wave(neck - 1.0)),
        (55.5, wave(neck)),
        (57.5, head_y + 0.6),
    );
    for i in 0..=24 {
        let t = i as f32 / 24.0;
        let u = 1.0 - t;
        let x = u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0;
        let y = u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1;
        p.disc(x, y, 1.9 - 0.4 * t, green);
    }
    // head, snout, crest, mouth and eye
    p.ellipse(60.5, head_y, 4.2, 2.1, green);
    p.poly(
        &[
            (60.0, head_y - 0.8),
            (66.0, head_y + 0.3),
            (66.0, head_y + 2.0),
            (60.0, head_y + 2.0),
        ],
        green,
    );
    p.poly(
        &[
            (57.0, head_y - 1.4),
            (56.5, head_y - 2.7),
            (59.5, head_y - 1.9),
        ],
        dark,
    );
    p.line((61.0, head_y + 1.2), (65.5, head_y + 1.2), dark);
    if p.high() {
        p.disc(61.6, head_y - 0.6, 0.6, WHITE);
        p.disc(61.8, head_y - 0.6, 0.3, BLACK);
    } else {
        p.dot(61, (head_y - 1.0) as i32, WHITE);
        p.dot(62, (head_y - 1.0) as i32, BLACK);
    }
    finish(p.f)
}

/// 34 x 28. Colors come from the fish's palette; the spots sit where the
/// ASCII color mask has its `2`s.
pub fn big_fish(palette: &[char; 9], ascii_mask: &str, res: Res) -> PixFrame {
    let body = color::letter_rgb(palette[0]).unwrap_or((220, 180, 60));
    let spot = color::letter_rgb(palette[1]).unwrap_or(WHITE);
    let fin = shade(body, 0.75);
    let mut p = Pen::new(34, 28, res);
    // a forked tail
    p.poly(&[(11.0, 14.0), (0.0, 2.0), (6.0, 3.0), (13.0, 9.0)], fin);
    p.poly(&[(11.0, 14.0), (0.0, 26.0), (6.0, 25.0), (13.0, 19.0)], fin);
    p.poly(&[(6.0, 10.0), (12.0, 9.0), (12.0, 19.0), (6.0, 18.0)], fin);
    p.poly(&[(9.0, 7.0), (15.0, 0.5), (27.0, 1.5), (30.0, 5.5)], fin);
    p.poly(&[(14.0, 23.0), (22.0, 24.0), (12.0, 27.5)], fin);
    if p.high() {
        // fin rays
        for i in 0..6 {
            let t = i as f32 / 5.0;
            p.hairline(
                (11.0, 14.0),
                (0.5 + 5.0 * t, 2.5 + 0.5 * t),
                shade(fin, 0.8),
            );
            p.hairline(
                (11.0, 14.0),
                (0.5 + 5.0 * t, 25.5 - 0.5 * t),
                shade(fin, 0.8),
            );
        }
    }
    p.ellipse(20.0, 14.0, 13.5, 10.5, body);
    for (row, line) in perl_lines(ascii_mask.strip_prefix('\n').unwrap_or(ascii_mask))
        .iter()
        .enumerate()
    {
        for (col, c) in line.chars().enumerate() {
            let y = row * 2 + (col + row) % 2;
            if c == '2' {
                if p.high() {
                    p.disc(col as f32 + 0.5, y as f32 + 0.5, 0.45, spot);
                } else if p.f.get(col, y) == Some(body) {
                    p.dot(col as i32, y as i32, spot);
                }
            }
        }
    }
    p.poly(
        &[(20.0, 15.0), (25.0, 17.0), (19.0, 20.0)],
        shade(body, 0.8),
    );
    // the gill arc
    let gill = |y: f32| (23.0 + (y - 14.0).powi(2) / 25.0, y);
    for y in 8..20 {
        p.line(gill(y as f32), gill(y as f32 + 1.0), shade(body, 0.6));
    }
    p.ellipse(28.0, 10.0, 1.6, 1.6, WHITE);
    if p.high() {
        p.disc(28.6, 10.2, 0.8, BLACK);
        p.disc(28.2, 9.6, 0.25, WHITE);
    } else {
        p.dot(29, 10, BLACK);
    }
    p.line((31.0, 15.0), (33.0, 14.0), shade(body, 0.45));
    finish(p.f)
}

/// 53 x 20
pub fn shark(res: Res) -> PixFrame {
    let grey = (110, 130, 150);
    let belly = (205, 210, 215);
    let dark = (60, 70, 85);
    let mut p = Pen::new(53, 20, res);
    // tail and fins
    p.poly(&[(8.0, 12.0), (0.0, 3.0), (3.0, 3.5), (11.0, 10.5)], grey);
    p.poly(&[(8.0, 13.0), (2.0, 19.5), (5.0, 19.5), (11.0, 14.0)], grey);
    p.poly(&[(27.0, 8.5), (31.0, 0.0), (34.0, 1.0), (38.0, 8.0)], grey);
    p.poly(&[(23.0, 16.0), (31.0, 16.0), (24.0, 19.9)], dark);
    // body, pale belly
    p.poly(
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
    p.poly(
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
        if p.high() {
            p.hairline((x, 10.8), (x - 0.6, 13.2), dark);
        } else {
            p.line((x, 11.0), (x - 0.5, 13.0), dark);
        }
    }
    if p.high() {
        p.disc(45.5, 11.3, 0.55, (220, 40, 40));
        p.disc(45.7, 11.3, 0.3, BLACK);
    } else {
        p.dot(45, 11, (220, 40, 40));
        p.dot(46, 11, BLACK);
    }
    // jaw and teeth
    p.line((40.0, 15.0), (51.0, 13.5), (70, 30, 35));
    for x in (41..51).step_by(2) {
        if p.high() {
            let y = 15.0 - (x as f32 - 40.0) * 1.5 / 11.0;
            p.poly(
                &[
                    (x as f32, y - 0.3),
                    (x as f32 + 1.0, y - 0.3),
                    (x as f32 + 0.5, y - 1.2),
                ],
                WHITE,
            );
        } else {
            p.dot(x, 14, WHITE);
        }
    }
    finish(p.f)
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
pub fn ducks(frame: usize, res: Res) -> PixFrame {
    let color = |c| match c {
        'w' => Some((240, 240, 235)),
        'W' => Some((200, 200, 195)),
        'g' => Some((40, 140, 70)),
        'k' => Some(BLACK),
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
    res.upscale(f).outlined(0.85).shaded(1.1, 0.9)
}

/// Dolphin frames (tail down, tail up), 14 x 8, in the dolphin's color.
pub fn dolphin(base: Rgb, res: Res) -> Vec<PixFrame> {
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
            finish(res.upscale(grid(rows, |c| match c {
                'b' => Some(base),
                'l' => Some(belly),
                'k' => Some(BLACK),
                _ => None,
            })))
        })
        .collect()
}

/// 11 x 14
pub fn swan(res: Res) -> PixFrame {
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
    finish(res.upscale(grid(&ROWS, |c| match c {
        'w' => Some((245, 245, 240)),
        'W' => Some((205, 205, 212)),
        'k' => Some(BLACK),
        'Y' => Some((235, 140, 40)),
        _ => None,
    })))
}
