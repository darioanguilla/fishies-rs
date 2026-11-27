//! Pixel art for the seasonal visitors (see `objects::season`). Everything
//! is drawn facing right; use [`super::facing`] for the other direction.

use std::f32::consts::TAU;

use super::pen::{Pen, Res};
use super::{BLACK, WHITE, finish, grid};
use crate::color::{Rgb, shade};
use crate::engine::PixFrame;

/// Santa in his sleigh behind a red-nosed reindeer, 29 x 8; `frame` 0 or 1
/// moves the reindeer's legs, like the ASCII frames.
pub fn santa(frame: usize, res: Res) -> PixFrame {
    const BODY: [&str; 6] = [
        "....RRW..................a.a.",
        "...RRRR..................aaa.",
        "...WWWW..................yky.",
        "...ssks.............yyyyyyyyn",
        "r..WWWW.rllllllllllyyyyyyyy..",
        "rrrRRRRrr..........yyyyyyy...",
    ];
    const LEGS: [[&str; 2]; 2] = [
        [
            ".rrrrrrr............yy..yy...",
            ".gggggggg..........y..yy..y..",
        ],
        [
            ".rrrrrrr............yy..yy...",
            ".gggggggg...........yy..yy...",
        ],
    ];
    let rows: Vec<&str> = BODY.iter().chain(&LEGS[frame % 2]).copied().collect();
    finish(res.upscale(grid(&rows, |c| match c {
        'R' => Some((215, 40, 40)),
        'r' => Some((150, 30, 35)),
        'W' => Some(WHITE),
        's' => Some((240, 190, 160)),
        'k' => Some(BLACK),
        'g' => Some((230, 185, 60)),
        'l' => Some((140, 90, 50)),
        'y' => Some((150, 100, 60)),
        'a' => Some((205, 175, 130)),
        'n' => Some((255, 60, 50)),
        _ => None,
    })))
}

/// Number of pixel frames of the ghost (its hem flutters).
pub const GHOST_FRAMES: usize = 4;

/// The ghost, 8 x 10, looking right; its hem ripples over the frames.
pub fn ghost(res: Res) -> Vec<PixFrame> {
    const BODY: [&str; 9] = [
        "..wwww..", ".wwwwww.", "wwwwwwww", "wwwwkwkw", "wwwwkwkw", "wwwwwwww", "wwwwwkww",
        "wwwwwwww", "wwwwwwww",
    ];
    const HEM: [&str; GHOST_FRAMES] = ["ww..ww..", ".ww..ww.", "..ww..ww", "w..ww..w"];
    HEM.iter()
        .map(|hem| {
            let rows: Vec<&str> = BODY.iter().copied().chain([*hem]).collect();
            let f = grid(&rows, |c| match c {
                'w' => Some((232, 238, 255)),
                'k' => Some(BLACK),
                _ => None,
            });
            // pale and glowing: a lighter touch than [`finish`]
            res.upscale(f).outlined(0.8).shaded(1.1, 0.75)
        })
        .collect()
}

/// One firework, one frame per ASCII frame (`sizes` are their sizes in
/// cells): the rocket rises, bursts into rays of `base` color and the
/// sparks fall and fade. The burst is centered on cell column 5, row 2.
pub fn fireworks(sizes: &[(usize, usize)], base: Rgb, res: Res) -> Vec<PixFrame> {
    let hot = (255, 245, 215);
    sizes
        .iter()
        .enumerate()
        .map(|(k, &(w, h))| {
            let mut p = Pen::new(w, 2 * h, res);
            let high = p.high();
            // pixel coordinates at low resolution (rounded to whole pixels),
            // continuous at high resolution, where there are more rays
            let (cx, cy, rays) = if high { (5.5, 5.0, 12) } else { (5.0, 4.0, 8) };
            let angle = |i: usize| i as f32 * TAU / rays as f32;
            let spark = |p: &mut Pen, x: f32, y: f32, c: Rgb| {
                if high {
                    p.disc(x, y, 0.45, c);
                } else {
                    p.dot(x.round() as i32, y.round() as i32, c);
                }
            };
            let line = |p: &mut Pen, a: (f32, f32), b: (f32, f32), c: Rgb| {
                if high {
                    p.hairline(a, b, c);
                } else {
                    p.line(a, b, c);
                }
            };
            let ray = |p: &mut Pen, i: usize, r0: f32, r1: f32, c: Rgb| {
                let (dx, dy) = (angle(i).cos(), angle(i).sin());
                line(
                    p,
                    (cx + dx * r0, cy + dy * r0),
                    (cx + dx * r1, cy + dy * r1),
                    c,
                );
            };
            match k {
                // the rocket climbs with a fading trail
                0 | 1 => {
                    let head = if k == 0 { 6.0 } else { 4.0 } + cy - 4.0;
                    line(
                        &mut p,
                        (cx, head + 1.0),
                        (cx, 9.0 + cy - 4.0),
                        shade(base, 0.6),
                    );
                    spark(&mut p, cx, head, hot);
                }
                // it bursts
                2 => {
                    for i in 0..rays {
                        ray(&mut p, i, 1.0, 2.2, base);
                    }
                    spark(&mut p, cx, cy, hot);
                }
                // the rays fly out
                3 => {
                    for i in 0..rays {
                        ray(&mut p, i, 2.0, 3.2, shade(base, 0.7));
                        ray(&mut p, i, 3.2, 4.6, base);
                    }
                }
                // and leave sparks behind, which fall and fade
                _ => {
                    let age = (k - 4) as f32;
                    let dim = 1.0 - 0.25 * age;
                    for i in 0..rays {
                        if k > 5 && i % 2 == 1 {
                            continue;
                        }
                        let (dx, dy) = (angle(i).cos(), angle(i).sin());
                        let r = 4.6 + 0.3 * age;
                        let (x, y) = (cx + dx * r, cy + dy * r + 1.5 * age);
                        let c = if k == 4 && i % 2 == 0 { hot } else { base };
                        spark(&mut p, x, y, shade(c, dim));
                        if k == 4 {
                            spark(&mut p, cx + dx * 2.6, cy + dy * 2.6, shade(base, 0.7));
                        }
                    }
                }
            }
            p.f
        })
        .collect()
}

/// The Easter bunny floating in half a painted eggshell, 13 x 10.
pub fn bunny(res: Res) -> PixFrame {
    const ROWS: [&str; 10] = [
        "....w..w.....",
        "....wp.wp....",
        "....wwwwwwww.",
        "....wwwwwkwp.",
        "e.e.wwwwwww.e",
        "eeeeWwwwwweee",
        "eegeeegeeegee",
        "eddddddddddde",
        ".eeeeeeeeeee.",
        "...eeeeeee...",
    ];
    finish(res.upscale(grid(&ROWS, |c| match c {
        'w' => Some((245, 245, 245)),
        'W' => Some((215, 215, 222)),
        'p' => Some((240, 150, 180)),
        'k' => Some(BLACK),
        'e' => Some((250, 238, 210)),
        'd' => Some((175, 140, 230)),
        'g' => Some((120, 200, 140)),
        _ => None,
    })))
}
