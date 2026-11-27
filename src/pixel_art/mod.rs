//! Pixel art for ANSI mode (original artwork for this port).
//!
//! Every sprite has the same footprint as its ASCII counterpart: as many
//! pixels wide as the ASCII art has columns, and twice as many pixels tall
//! as it has rows.

pub mod creatures;
pub mod extras;
pub mod pen;
pub mod seasonal;

use std::f32::consts::PI;

use crate::color::{self, Rgb, shade};
use crate::engine::PixFrame;
use pen::Res;

const WHITE: Rgb = (250, 250, 250);
const BLACK: Rgb = (15, 15, 25);

/// The common look of every sprite: a darker outline and light from above.
pub(super) fn finish(f: PixFrame) -> PixFrame {
    f.outlined(0.7).shaded(1.2, 0.8)
}

pub(super) fn grid<S: AsRef<str>>(rows: &[S], color: impl Fn(char) -> Option<Rgb>) -> PixFrame {
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
pub fn fish(fish_num: usize, palette: &[char; 9], res: Res) -> PixFrame {
    let color = |c: char| match c {
        'W' => Some(WHITE),
        'k' => Some(BLACK),
        d @ '1'..='9' => color::letter_rgb(palette[d as usize - '1' as usize]),
        _ => None,
    };
    facing(
        finish(res.upscale(grid(FISH[fish_num / 2], color))),
        fish_num % 2,
    )
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

pub fn castle(res: Res) -> PixFrame {
    if res == Res::High {
        return castle_high();
    }
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

/// The castle at high resolution: the layout smoothed with Scale2x (round
/// merlons, arched windows, a smooth roof), then textured pixel by pixel:
/// smaller bricks with their own tint, roof shingles, gate planks.
fn castle_high() -> PixFrame {
    let layout = PixFrame::from_grid(&CASTLE, |c| (c != '.').then_some((c as u8, 0, 0)));
    let up = Res::High.upscale(layout);
    let mut f = PixFrame::new(up.width, up.height);
    for y in 0..up.height {
        for x in 0..up.width {
            let Some((region, _, _)) = up.get(x, y) else {
                continue;
            };
            let n = noise(x, y);
            let brick = |base: Rgb| {
                // courses 6 pixels tall, bricks 12 long, staggered
                let course = y / 6;
                let shifted = x + 6 * (course % 2);
                let mortar = y % 6 == 5 || shifted % 12 == 0;
                let tint = 0.88 + 0.22 * noise(shifted / 12, course) + 0.05 * n;
                shade(base, if mortar { 0.68 } else { tint })
            };
            let px = match region as char {
                'T' | 'm' => brick((150, 146, 140)),
                'K' => brick((178, 172, 162)),
                'r' => {
                    let row = y / 3;
                    let edge = y % 3 == 2 || (x + 3 * (row % 2)) % 6 == 0;
                    shade((74, 94, 155), if edge { 0.75 } else { 0.95 + 0.1 * n })
                }
                'p' => (95, 95, 100),
                'f' => shade((225, 50, 50), 1.0 - 0.15 * (x % 8) as f32 / 8.0),
                'w' => (28, 28, 40),
                'y' => shade((250, 205, 90), 0.85 + 0.25 * n),
                'g' => {
                    let band = y % 12 == 3;
                    let seam = x % 4 == 0;
                    match (band, seam) {
                        (true, _) => (70, 66, 64),
                        (false, true) => (92, 57, 28),
                        _ => shade((125, 80, 40), 0.92 + 0.12 * n),
                    }
                }
                _ => continue,
            };
            f.set(x, y, Some(px));
        }
    }
    f
}

/// Number of animation frames of the water surface.
pub const WATER_FRAMES: usize = 8;

/// Animated pixel frames for water line `row` (0 = surface), drawn only
/// where the ASCII line has a character, so the rest stays transparent.
pub fn waterline(row: usize, line: &str, res: Res) -> Vec<PixFrame> {
    if res == Res::High {
        return waterline_high(row, line);
    }
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

/// The water at high resolution: the surface is a smooth rolling wave two
/// pixels thick with foam on the crests; the ripples are thin streaks with
/// tapered ends and a glint drifting along them.
fn waterline_high(row: usize, line: &str) -> Vec<PixFrame> {
    const DEEP: [Rgb; 3] = [(95, 180, 240), (65, 145, 225), (45, 115, 205)];
    let s = Res::High.scale();
    let chars: Vec<char> = line.chars().collect();
    (0..WATER_FRAMES)
        .map(|p| {
            let mut f = PixFrame::new(chars.len() * s, 2 * s);
            if row == 0 {
                for x in 0..f.width {
                    if chars[x / s] == ' ' {
                        continue;
                    }
                    let w = (x as f32 / s as f32 * 0.55 - p as f32 * PI / 4.0).sin();
                    let yc = (s as f32 - 0.5) - (s as f32 - 1.4) * w;
                    for y in 0..f.height {
                        if (y as f32 + 0.5 - yc).abs() <= 1.0 {
                            let t = (w + 1.0) / 2.0;
                            let c = (
                                (115.0 + 75.0 * t) as u8,
                                (205.0 + 35.0 * t) as u8,
                                (245.0 + 10.0 * t) as u8,
                            );
                            f.set(x, y, Some(c));
                        }
                    }
                    if w > 0.92 {
                        f.dot(x as i32, (yc - 1.5) as i32, (235, 250, 255));
                    }
                }
            } else {
                let mut run = 0;
                let mut x = 0;
                while x < chars.len() {
                    if chars[x] == ' ' {
                        x += 1;
                        continue;
                    }
                    let end = x + chars[x..].iter().take_while(|&&c| c != ' ').count();
                    run += 1;
                    // the streak bobs between the two halves of the cell
                    let y = (run + row + p / 4) % 2 * s + s / 2;
                    for hx in x * s + 1..end * s - 1 {
                        let glow =
                            0.85 + 0.3 * ((hx as f32 / s as f32 * 0.7 - p as f32) * PI / 4.0).sin();
                        f.set(hx, y, Some(shade(DEEP[row - 1], glow)));
                        if glow > 1.1 {
                            f.set(hx, y - 1, Some(shade(DEEP[row - 1], glow * 0.85)));
                        }
                    }
                    x = end;
                }
            }
            f
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::creatures::*;
    use super::extras::*;
    use super::seasonal::*;
    use super::*;
    use crate::art;
    use crate::engine::{HIRES, Sprite};

    fn ascii_size(shape: &str) -> (usize, usize) {
        let f = &Sprite::single(shape).build()[0];
        (f.width, f.height)
    }

    /// The sprite takes up the cells of its ASCII art: `w` pixels per cell
    /// column and two per row at low resolution, `HIRES` times more at high.
    fn assert_fits(name: &str, ascii: &str, make: impl Fn(Res) -> PixFrame, slack: usize) {
        let (w, h) = ascii_size(ascii);
        for res in [Res::Low, Res::High] {
            let s = res.scale();
            let px = make(res);
            assert!(
                px.width.abs_diff(w * s) <= slack * s && px.height == 2 * h * s,
                "{name} at {res:?}: pixels {}x{}, ASCII {w}x{h} cells",
                px.width,
                px.height
            );
        }
    }

    #[test]
    fn fish_footprints_match_ascii() {
        let palette = ['c'; 9];
        for n in 0..art::FISH.len() {
            assert_fits("fish", art::FISH[n].0, |r| fish(n, &palette, r), 0);
            for row in FISH[n / 2] {
                assert_eq!(
                    row.chars().count(),
                    ascii_size(art::FISH[n].0).0,
                    "fish {n}"
                );
            }
        }
    }

    #[test]
    fn castle_footprint_matches_ascii() {
        assert_fits("castle", art::CASTLE, castle, 0);
        assert!(CASTLE.iter().all(|r| r.len() == ascii_size(art::CASTLE).0));
    }

    #[test]
    fn left_facing_fish_is_mirrored() {
        let palette = ['r'; 9];
        for res in [Res::Low, Res::High] {
            assert_eq!(fish(1, &palette, res), fish(0, &palette, res).mirrored());
        }
    }

    #[test]
    fn object_footprints_match_ascii() {
        for dir in 0..2 {
            assert_fits("ship", art::SHIP[dir], |r| facing(ship(r), dir), 0);
            assert_fits("shark", art::SHARK[dir], shark, 0);
            let palette = ['y'; 9];
            let fish = |r| big_fish(&palette, art::BIG_FISH_MASK[0], r);
            assert_fits("big fish", art::BIG_FISH[dir], fish, 0);
            assert_fits("swan", art::SWAN[dir], swan, 0);
            for (k, f) in art::DUCKS[dir].iter().enumerate() {
                assert_fits("ducks", f, |r| ducks(k, r), 0);
            }
            for k in 0..2 {
                let d = |r| dolphin((0, 0, 255), r).swap_remove(k);
                assert_fits("dolphin", art::DOLPHIN[dir][k], d, 0);
            }
            // the left-facing ASCII monster is 1-2 columns narrower in some frames
            for (k, f) in art::MONSTER[dir].iter().enumerate() {
                assert_fits("monster", f, |r| monster(k * MONSTER_FRAMES / 4, r), 2);
            }
        }
        for res in [Res::Low, Res::High] {
            let s = res.scale();
            let w = whale(res);
            assert_eq!(w.len(), 12);
            assert!(w.iter().all(|f| (f.width, f.height) == (19 * s, 14 * s)));
            assert!(
                seaweed(5, res)
                    .iter()
                    .all(|f| (f.width, f.height) == (2 * s, 10 * s))
            );
        }
        assert_fits("fishhook", art::FISHHOOK, fishhook, 0);
        let sizes: Vec<_> = art::SPLAT.iter().map(|s| ascii_size(s)).collect();
        for k in 0..sizes.len() {
            assert_fits(
                "splat",
                art::SPLAT[k],
                |r| splat(&sizes, r).swap_remove(k),
                0,
            );
        }
    }

    #[test]
    fn seasonal_footprints_match_ascii() {
        for dir in 0..2 {
            for k in 0..2 {
                let s = |r| facing(santa(k, r), dir);
                assert_fits("santa", art::SANTA[dir][k], s, 0);
            }
            assert_fits("bunny", art::BUNNY[dir], |r| facing(bunny(r), dir), 0);
        }
        for f in art::GHOST {
            for k in 0..GHOST_FRAMES {
                assert_fits("ghost", f, |r| ghost(r).swap_remove(k), 0);
            }
        }
        let sizes: Vec<_> = art::FIREWORKS.iter().map(|s| ascii_size(s)).collect();
        for k in 0..sizes.len() {
            let f = |r| fireworks(&sizes, (255, 0, 0), r).swap_remove(k);
            assert_fits("fireworks", art::FIREWORKS[k], f, 0);
            assert!(
                f(Res::Low).px.iter().any(Option::is_some),
                "fireworks frame {k} is empty"
            );
        }
    }

    #[test]
    fn seasonal_masks_match_their_art() {
        let pairs = [
            (art::SANTA[0][0], art::SANTA_MASK[0]),
            (art::SANTA[0][1], art::SANTA_MASK[0]),
            (art::SANTA[1][0], art::SANTA_MASK[1]),
            (art::SANTA[1][1], art::SANTA_MASK[1]),
            (art::GHOST[0], art::GHOST_MASK),
            (art::GHOST[1], art::GHOST_MASK),
            (art::BUNNY[0], art::BUNNY_MASK[0]),
            (art::BUNNY[1], art::BUNNY_MASK[1]),
        ];
        for (shape, mask) in pairs {
            let (shape, mask): (Vec<_>, Vec<_>) = (shape.lines().collect(), mask.lines().collect());
            assert_eq!(shape.len(), mask.len());
            for (s, m) in shape.iter().zip(&mask) {
                // every visible character has a color and nothing else does
                let s: Vec<char> = s.chars().collect();
                for (x, c) in m.chars().enumerate() {
                    assert_eq!(
                        c == ' ',
                        s.get(x).is_none_or(|&s| s == ' '),
                        "{s:?} / {m:?}"
                    );
                }
                assert!(m.len() >= s.iter().rposition(|&c| c != ' ').map_or(0, |i| i + 1));
            }
        }
    }

    #[test]
    fn monster_swims_in_a_wave() {
        // the body moves from frame to frame and the wave loops
        let a = monster(0, Res::Low);
        let b = monster(MONSTER_FRAMES / 2, Res::Low);
        assert_ne!(a, b);
        assert_eq!(monster(MONSTER_FRAMES, Res::Low), a, "the wave loops");
    }

    #[test]
    fn monster_body_stays_above_the_water() {
        use crate::pixel_art::creatures::MONSTER_SURFACE;
        for res in [Res::Low, Res::High] {
            let surface = MONSTER_SURFACE * res.scale();
            for k in 0..MONSTER_FRAMES {
                let f = monster(k, res);
                let row = |y| (0..f.width).filter(|&x| f.get(x, y).is_some()).count();
                assert!(
                    (surface..f.height).all(|y| row(y) == 0),
                    "frame {k} {res:?}: pixels under the water"
                );
                assert!(
                    row(surface - 1) > 0,
                    "frame {k} {res:?}: humps reach the surface"
                );
            }
        }
    }

    #[test]
    fn waterline_keeps_gaps_transparent() {
        for res in [Res::Low, Res::High] {
            let s = res.scale();
            let frames = waterline(1, "^^  ^", res);
            assert_eq!(frames.len(), WATER_FRAMES);
            for f in &frames {
                assert_eq!((f.width, f.height), (5 * s, 2 * s));
                for x in 2 * s..4 * s {
                    assert!((0..f.height).all(|y| f.get(x, y).is_none()));
                }
            }
        }
        // at low resolution, exactly one pixel per character
        for f in &waterline(1, "^^  ^", Res::Low) {
            assert!(f.get(0, 0).is_some() != f.get(0, 1).is_some());
        }
        assert_eq!(HIRES, Res::High.scale());
    }
}
