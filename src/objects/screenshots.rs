//! Scenes rendered to HTML for the screenshots in `docs/screenshots`.
//!
//! Run `scripts/screenshots.sh`, which calls
//! `cargo test screenshots -- --ignored` and turns the pages into PNGs.

use std::fs;
use std::path::{Path, PathBuf};

use super::{environment, fish, random, tick};
use crate::engine::render::snapshot::{CELL_H, CELL_W, to_html};
use crate::engine::{Animation, Buffer, Mode};

const W: i32 = 110;
const H: i32 = 34;
/// A dark terminal background.
const DARK: &str = "#0c0c0c";
/// A colorful "wallpaper", to show that the water is transparent.
const WALLPAPER: &str = "linear-gradient(135deg,#2b1055 0%,#7597de 55%,#f5c26b 100%)";

fn scene(object: &str, ticks: usize) -> Animation {
    let mut anim = Animation::new(W, H);
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
}
