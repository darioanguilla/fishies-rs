//! Scenes rendered to HTML for the screenshots in `docs/screenshots`.
//!
//! Run `scripts/screenshots.sh`, which calls
//! `cargo test screenshots -- --ignored` and turns the pages into PNGs.

use std::fs;
use std::path::{Path, PathBuf};

use super::{environment, fish, random, tick};
use crate::engine::render::snapshot::{CELL_H, CELL_W, to_html};
use std::time::{Duration, Instant};

use crate::color::Rgb;
use crate::engine::{Animation, Buffer, Mode};
use crate::graphics::canvas::{Canvas, Cells, compose, unpack};
use crate::graphics::png;
use crate::graphics::{kitty_encode, sixel_encode};

const W: i32 = 110;
const H: i32 = 34;
/// A dark terminal background.
const DARK: &str = "#0c0c0c";
/// A colorful "wallpaper", to show that the water is transparent.
const WALLPAPER: &str = "linear-gradient(135deg,#2b1055 0%,#7597de 55%,#f5c26b 100%)";

fn scene(object: &str, ticks: usize) -> Animation {
    scene_sized(W, H, object, ticks)
}

fn scene_sized(w: i32, h: i32, object: &str, ticks: usize) -> Animation {
    let mut anim = Animation::new(w, h);
    environment::add_environment(&mut anim);
    environment::add_castle(&mut anim);
    environment::add_all_seaweed(&mut anim);
    fish::add_all_fish(&mut anim);
    let idx = random::OBJECT_NAMES
        .iter()
        .position(|&n| n == object)
        .unwrap();
    random::spawn(idx, &mut anim);
    anim.flush();
    for _ in 0..ticks {
        tick(&mut anim);
    }
    anim
}

/// Like [`scene`], but the object is moved to the middle of the screen
/// right away (the fishhook only needs time to sink).
fn centered(object: &str, ticks: usize) -> Animation {
    let mut anim = scene(object, 0);
    let ids: Vec<_> = anim
        .entities
        .iter()
        .filter(|e| {
            e.z < 20.0
                && e.kind != crate::engine::Kind::Fish
                && e.kind != crate::engine::Kind::Waterline
        })
        .map(|e| e.id)
        .collect();
    let group = anim.entities.iter().filter(|e| ids.contains(&e.id));
    let (left, right) = group.fold((f64::MAX, f64::MIN), |(l, r), e| {
        (l.min(e.x), r.max(e.x + e.width()))
    });
    if object != "fishhook" {
        let dx = (W as f64 - (right - left)) / 2.0 - left;
        for e in anim.entities.iter_mut().filter(|e| ids.contains(&e.id)) {
            e.x += dx;
        }
    }
    for _ in 0..ticks {
        tick(&mut anim);
    }
    anim
}

fn write(dir: &Path, name: &str, anim: &Animation, mode: Mode, background: &str) {
    let mut buf = Buffer::new(W, H);
    anim.render(&mut buf, mode);
    fs::write(dir.join(format!("{name}.html")), to_html(&buf, background)).unwrap();
}

#[test]
#[ignore = "writes screenshot pages to target/screenshots"]
fn screenshots() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/screenshots");
    fs::create_dir_all(&dir).unwrap();
    // page size for the browser: cells plus the 9px padding on each side
    fs::write(
        dir.join("size.txt"),
        format!("{},{}", W as usize * CELL_W + 18, H as usize * CELL_H + 18),
    )
    .unwrap();

    // the same moment in both modes
    for (name, object, ticks) in [("whale", "whale", 32), ("shark", "shark", 36)] {
        let anim = scene(object, ticks);
        write(&dir, &format!("{name}-ascii"), &anim, Mode::Ascii, DARK);
        write(&dir, &format!("{name}-ansi"), &anim, Mode::Ansi, DARK);
    }
    let anim = scene("ship", 40);
    write(&dir, "transparent-ascii", &anim, Mode::Ascii, WALLPAPER);
    write(&dir, "transparent-ansi", &anim, Mode::Ansi, WALLPAPER);

    // every random object in ANSI mode
    for object in random::OBJECT_NAMES {
        let ticks = if object == "fishhook" { 30 } else { 4 };
        write(
            &dir,
            &format!("gallery-{object}"),
            &centered(object, ticks),
            Mode::Ansi,
            DARK,
        );
    }
}

/// Composite a graphics-mode canvas over a background and save it as PNG.
fn write_png(dir: &Path, name: &str, canvas: &Canvas, background: impl Fn(usize, usize) -> Rgb) {
    let mut rgb = Vec::with_capacity(canvas.width * canvas.height * 3);
    for y in 0..canvas.height {
        for x in 0..canvas.width {
            let p = canvas.px[y * canvas.width + x];
            let (r, g, b) = if p == 0 { background(x, y) } else { unpack(p) };
            rgb.extend_from_slice(&[r, g, b]);
        }
    }
    fs::write(
        dir.join(format!("{name}.png")),
        png::encode(canvas.width, canvas.height, &rgb),
    )
    .unwrap();
}

/// Graphics mode: screenshots, plus the size and encoding time of frames.
/// Run with `cargo test --release graphics_frames -- --ignored --nocapture`.
#[test]
#[ignore = "writes docs/screenshots/graphics-*.png and prints frame statistics"]
fn graphics_frames() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/screenshots");
    fs::create_dir_all(&dir).unwrap();
    let cells = Cells { w: 10.0, h: 20.0 };

    let anim = scene("shark", 36);
    let mut canvas = Canvas::new(W as usize * 10, H as usize * 20);
    compose(&anim, &mut canvas, cells);
    write_png(&dir, "graphics-shark", &canvas, |_, _| (12, 12, 12));
    let anim = scene("ship", 40);
    compose(&anim, &mut canvas, cells);
    let (cw, ch) = (canvas.width as f32, canvas.height as f32);
    write_png(&dir, "graphics-transparent", &canvas, |x, y| {
        // the same wallpaper as the ANSI screenshot
        let t = (x as f32 / cw + y as f32 / ch) / 2.0;
        let mix = |a: f32, b: f32, c: f32| {
            (if t < 0.55 {
                a + (b - a) * t / 0.55
            } else {
                b + (c - b) * (t - 0.55) / 0.45
            }) as u8
        };
        (
            mix(43.0, 117.0, 245.0),
            mix(16.0, 151.0, 194.0),
            mix(85.0, 222.0, 107.0),
        )
    });

    for (w, h) in [(120, 40), (200, 60)] {
        let mut anim = scene_sized(w, h, "whale", 30);
        let mut canvas = Canvas::new(w as usize * 10, (h - 1) as usize * 20);
        let (mut sixel_bytes, mut kitty_bytes) = (0, 0);
        let (mut sixel_time, mut kitty_time) = (Duration::ZERO, Duration::ZERO);
        let frames = 30;
        for _ in 0..frames {
            tick(&mut anim);
            compose(&anim, &mut canvas, cells);
            let t = Instant::now();
            sixel_bytes += sixel_encode(&canvas).len();
            sixel_time += t.elapsed();
            let t = Instant::now();
            kitty_bytes += kitty_encode(&canvas).len();
            kitty_time += t.elapsed();
        }
        println!(
            "{w}x{h} cells, {}x{} px: Sixel {} KB/frame {:.1} ms, kitty {} KB/frame {:.1} ms",
            canvas.width,
            canvas.height,
            sixel_bytes / frames / 1024,
            sixel_time.as_secs_f64() * 1000.0 / frames as f64,
            kitty_bytes / frames / 1024,
            kitty_time.as_secs_f64() * 1000.0 / frames as f64,
        );
    }
}
