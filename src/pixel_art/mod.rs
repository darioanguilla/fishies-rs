//! Pixel art for ANSI mode (original artwork for this port).
//!
//! Every sprite has the same footprint as its ASCII counterpart: as many
//! pixels wide as the ASCII art has columns, and twice as many pixels tall
//! as it has rows.

pub mod creatures;
pub mod extras;

use std::f32::consts::PI;

use crate::color::{self, Rgb, shade};
use crate::engine::PixFrame;

const WHITE: Rgb = (250, 250, 250);
const BLACK: Rgb = (15, 15, 25);

/// The common look of every sprite: a darker outline and light from above.
fn finish(f: PixFrame) -> PixFrame {
    f.outlined(0.7).shaded(1.2, 0.8)
}

fn grid<S: AsRef<str>>(rows: &[S], color: impl Fn(char) -> Option<Rgb>) -> PixFrame {
    PixFrame::from_grid(rows, color)
}

/// Mirror right-facing art for objects moving left (`dir == 1`).
pub fn facing(f: PixFrame, dir: usize) -> PixFrame {
    if dir == 1 { f.mirrored() } else { f }
}

/// Fish species, facing right. Digits are the parts of the ASCII color
/// masks (1 body, 2 dorsal fin, 3 flippers, 5 mouth, 6 tail, 7 gills) and
/// get the fish's random colors; `W` and `k` are the eye, `.` is empty.
const FISH: [&[&str]; 8] = [
    // 14 x 12
    &[
        "..............",
        ".......2......",
        "......222.....",
        "....2222222...",
        "6..111111111..",
        "66111111111Wk.",
        "6611771111111.",
        "66117711111115",
        "6..1111111111.",
        "....11111111..",
        ".....333......",
        "......3.......",
    ],
    // 7 x 10
    &[
        "...2...", "..222..", "..1111.", "6.11111", "66111Wk", "6611115", "6.11111", "..1111.",
        "...3...", "...33..",
    ],
    // 16 x 10
    &[
        "................",
        ".......2222.....",
        "...22.222222....",
        "6...1111111111..",
        "66.111111111111.",
        "666117711111Wk1.",
        "6661177111111115",
        "66.111111111111.",
        "6...11333331....",
        "......33........",
    ],
    // 5 x 6
    &["..2..", ".111.", "611Wk", "61115", ".113.", "....."],
    // 9 x 6
    &[
        "...222...",
        "6.111111.",
        "6617111Wk",
        "661711115",
        "6.113311.",
        ".........",
    ],
    // 6 x 10
    &[
        "...2..", "..22..", "..111.", ".11111", "6111Wk", "661115", "6.1111", "..111.", "..33..",
        "...3..",
    ],
    // 5 x 6
    &["..2..", "6111.", "671Wk", "67115", "6113.", "....."],
    // 5 x 6
    &[".....", ".111.", "61Wk1", "61115", ".111.", "....."],
];

/// Pixel art for ASCII fish number `fish_num` (even = facing right), using
/// `palette[d - 1]` as the mask letter of part `d`.
pub fn fish(fish_num: usize, palette: &[char; 9]) -> PixFrame {
    let color = |c: char| match c {
        'W' => Some(WHITE),
        'k' => Some(BLACK),
        d @ '1'..='9' => color::letter_rgb(palette[d as usize - '1' as usize]),
        _ => None,
    };
    facing(finish(grid(FISH[fish_num / 2], color)), fish_num % 2)
}

/// Castle layout, 31 x 26: `T` tower wall, `K` keep wall, `m` merlon,
/// `r` roof, `p` flagpole, `f` flag, `w` window, `y` lit window, `g` gate.
const CASTLE: [&str; 26] = [
    "...............pfff............",
    "...............pff.............",
    "...............p...............",
    "..............rrr..............",
    ".............rrrrr.............",
    "............rrrrrrr............",
    "...........rrrrrrrrr...........",
    "..........rrrrrrrrrrr..........",
    "mm..mm..mmKKKKKKKKKKKmm..mm..mm",
    "mm..mm..mmKKKKKKKKKKKmm..mm..mm",
    "TTTTTTTTTTKKKKKKKKKKKTTTTTTTTTT",
    "TTTTTTTTTTKKwwKKKwwKKTTTTTTTTTT",
    "TTTTyyTTTTKKwwKKKwwKKTTTTyyTTTT",
    "TTTTyyTTTTKKwwKKKwwKKTTTTyyTTTT",
    "TTTTyyTTTTKKKKKKKKKKKTTTTyyTTTT",
    "TTTTTTTTTTKKKKKKKKKKKTTTTTTTTTT",
    "TTTTTTTTTTKKKKKKKKKKKTTTTTTTTTT",
    "TTTTTTTTTTKKKKgggKKKKTTTTTTTTTT",
    "TTTwwTTTTTKKKgggggKKKTTTTTwwTTT",
    "TTTwwTTTTTKKgggggggKKTTTTTwwTTT",
    "TTTwwTTTTTKKgggggggKKTTTTTwwTTT",
    "TTTTTTTTTTKKgggggggKKTTTTTTTTTT",
    "TTTTTTTTTTKKgggggggKKTTTTTTTTTT",
    "TTTTTTTTTTKKgggggggKKTTTTTTTTTT",
    "TTTTTTTTTTKKgggggggKKTTTTTTTTTT",
    "TTTTTTTTTTKKgggggggKKTTTTTTTTTT",
];

/// Small deterministic noise in `0.0..1.0`, for stone texture.
fn noise(x: usize, y: usize) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    (h >> 8) as f32 / (1u32 << 24) as f32
}

pub fn castle() -> PixFrame {
    let rows: Vec<Vec<char>> = CASTLE.iter().map(|r| r.chars().collect()).collect();
    let mut f = PixFrame::new(rows[0].len(), rows.len());
    for (y, row) in rows.iter().enumerate() {
        for (x, &c) in row.iter().enumerate() {
            let n = noise(x, y);
            let brick = |base: Rgb| {
                // courses of bricks 3 pixels tall, with staggered joints
                let course = y / 3;
                let mortar = y % 3 == 2 || (x + 2 * (course % 2)) % 5 == 0;
                let f = if mortar { 0.72 } else { 0.9 + 0.2 * n };
                shade(base, f)
            };
            let px = match c {
                'T' | 'm' => Some(brick((150, 146, 140))),
                'K' => Some(brick((178, 172, 162))),
                'r' => Some(if y % 2 == 0 {
                    (78, 98, 160)
                } else {
                    (62, 80, 140)
                }),
                'p' => Some((95, 95, 100)),
                'f' => Some((225, 50, 50)),
                'w' => Some((28, 28, 40)),
                'y' => Some(shade((250, 205, 90), 0.9 + 0.15 * n)),
                'g' => Some(if x % 2 == 0 {
                    (125, 80, 40)
                } else {
                    (100, 62, 30)
                }),
                _ => None,
            };
            f.set(x, y, px);
        }
    }
    f
}

/// Number of animation frames of the water surface.
pub const WATER_FRAMES: usize = 8;

/// Animated pixel frames for water line `row` (0 = surface), drawn only
/// where the ASCII line has a character, so the rest stays transparent.
pub fn waterline(row: usize, line: &str) -> Vec<PixFrame> {
    const DEEP: [Rgb; 3] = [(95, 180, 240), (65, 145, 225), (45, 115, 205)];
    let chars: Vec<char> = line.chars().collect();
    (0..WATER_FRAMES)
        .map(|p| {
            let mut f = PixFrame::new(chars.len(), 2);
            let mut run = 0;
            for (x, &c) in chars.iter().enumerate() {
                if c == ' ' {
                    continue;
                }
                // runs of characters are numbered, so each ripple is one streak
                if x > 0 && chars[x - 1] == ' ' {
                    run += 1;
                }
                if row == 0 {
                    // a continuous rolling wave
                    let s = (x as f32 * 0.55 - p as f32 * PI / 4.0).sin();
                    let (y, c) = if s > 0.0 {
                        (0, (175, 235, 255))
                    } else {
                        (1, (115, 205, 245))
                    };
                    f.set(x, y, Some(c));
                } else {
                    // horizontal streaks that bob up and down, with a glint
                    // drifting along them
                    let y = (run + row + p / 4) % 2;
                    let glow = 0.85 + 0.3 * ((x as f32 * 0.7 - p as f32) * PI / 4.0).sin();
                    f.set(x, y, Some(shade(DEEP[row - 1], glow)));
                }
            }
            f
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art;
    use crate::engine::Sprite;

    fn ascii_size(shape: &str) -> (usize, usize) {
        let f = &Sprite::single(shape).build()[0];
        (f.width, f.height)
    }

    #[test]
    fn fish_footprints_match_ascii() {
        let palette = ['c'; 9];
        for n in 0..art::FISH.len() {
            let (w, h) = ascii_size(art::FISH[n].0);
            let p = fish(n, &palette);
            assert_eq!((p.width, p.height), (w, 2 * h), "fish {n}");
            for row in FISH[n / 2] {
                assert_eq!(row.chars().count(), w, "fish {n}: row {row:?}");
            }
        }
    }

    #[test]
    fn castle_footprint_matches_ascii() {
        let (w, h) = ascii_size(art::CASTLE);
        let c = castle();
        assert_eq!((c.width, c.height), (w, 2 * h));
        assert!(CASTLE.iter().all(|r| r.len() == w));
    }

    #[test]
    fn left_facing_fish_is_mirrored() {
        let palette = ['r'; 9];
        assert_eq!(fish(1, &palette), fish(0, &palette).mirrored());
    }

    fn assert_fits(name: &str, ascii: &str, px: &PixFrame, slack: usize) {
        let (w, h) = ascii_size(ascii);
        assert!(
            px.width.abs_diff(w) <= slack && px.height == 2 * h,
            "{name}: pixels {}x{}, ASCII {w}x{h} cells",
            px.width,
            px.height
        );
    }

    #[test]
    fn object_footprints_match_ascii() {
        use super::creatures::*;
        use super::extras::*;
        for dir in 0..2 {
            assert_fits("ship", art::SHIP[dir], &facing(ship(), dir), 0);
            assert_fits("shark", art::SHARK[dir], &shark(), 0);
            assert_fits(
                "big fish",
                art::BIG_FISH[dir],
                &big_fish(&['y'; 9], art::BIG_FISH_MASK[0]),
                0,
            );
            assert_fits("swan", art::SWAN[dir], &swan(), 0);
            for (k, f) in art::DUCKS[dir].iter().enumerate() {
                assert_fits("ducks", f, &ducks(k), 0);
            }
            for (k, f) in dolphin((0, 0, 255)).iter().enumerate() {
                assert_fits("dolphin", art::DOLPHIN[dir][k], f, 0);
            }
            // the left-facing ASCII monster is 1-2 columns narrower in some frames
            for (k, f) in art::MONSTER[dir].iter().enumerate() {
                assert_fits("monster", f, &monster(k), 2);
            }
        }
        let whale = whale();
        assert_eq!(whale.len(), 12);
        assert!(whale.iter().all(|f| (f.width, f.height) == (19, 14)));
        assert_fits("fishhook", art::FISHHOOK, &fishhook(), 0);
        let sizes: Vec<_> = art::SPLAT.iter().map(|s| ascii_size(s)).collect();
        for (k, f) in splat(&sizes).iter().enumerate() {
            assert_fits("splat", art::SPLAT[k], f, 0);
        }
        assert!(seaweed(5).iter().all(|f| (f.width, f.height) == (2, 10)));
    }

    #[test]
    fn waterline_keeps_gaps_transparent() {
        let frames = waterline(1, "^^  ^");
        assert_eq!(frames.len(), WATER_FRAMES);
        for f in &frames {
            assert_eq!((f.width, f.height), (5, 2));
            assert!(f.get(2, 0).is_none() && f.get(2, 1).is_none());
            // exactly one pixel per character
            assert!(f.get(0, 0).is_some() != f.get(0, 1).is_some());
        }
    }
}
