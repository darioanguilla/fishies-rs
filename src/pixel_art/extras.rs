//! Pixel art for the fishhook, seaweed and the shark's splat.

use super::{finish, grid, noise};
use crate::color::shade;
use crate::engine::PixFrame;

/// 8 x 12
pub fn fishhook() -> PixFrame {
    const ROWS: [&str; 12] = [
        ".....ggg", ".....g.g", ".....ggg", "......g.", "......g.", "......g.", "g.....g.",
        "gg....g.", "g.....g.", "g.....g.", ".g...g..", "..ggg...",
    ];
    finish(grid(&ROWS, |c| (c == 'g').then_some((190, 196, 208))))
}

/// A transparent frame, for entities that only exist to collide (the
/// shark's teeth, the hook's point): the visible sprite already shows them.
pub fn invisible(width: usize, height: usize) -> PixFrame {
    PixFrame::new(width, 2 * height)
}

/// The two swaying frames of a seaweed `height` cells tall, 2 x 2*height.
pub fn seaweed(height: usize) -> Vec<PixFrame> {
    (0..2)
        .map(|frame| {
            let mut f = PixFrame::new(2, 2 * height);
            for y in 0..f.height {
                // zig-zag every 2 pixels, like the ASCII `(` and `)`
                let x = (y / 2 + frame) % 2;
                f.set(x, y, Some((70, 170, 80)));
                // a leaf now and then on the other side
                if y % 5 == 1 {
                    f.set(1 - x, y, Some((95, 190, 95)));
                }
            }
            // lit from the surface
            f.shaded(1.5, 0.7)
        })
        .collect()
}

/// The shark's splat: droplets spreading out, one frame per ASCII frame
/// (`sizes` are their sizes in cells).
pub fn splat(sizes: &[(usize, usize)]) -> Vec<PixFrame> {
    let n = sizes.len();
    sizes
        .iter()
        .enumerate()
        .map(|(k, &(w, h))| {
            let mut f = PixFrame::new(w, 2 * h);
            let (cx, cy) = (w as f32 / 2.0, h as f32);
            let spread = (k + 1) as f32 / n as f32;
            for y in 0..f.height {
                for x in 0..f.width {
                    let (dx, dy) = ((x as f32 + 0.5 - cx) / cx, (y as f32 + 0.5 - cy) / cy);
                    let d = (dx * dx + dy * dy).sqrt();
                    // a dense blob that breaks into fewer, farther drops
                    let keep = noise(x + 31 * k, y) > 0.25 + 0.6 * spread;
                    if d <= spread && (d < 0.35 * (1.0 - spread) + 0.15 || keep) {
                        let c = shade((215, 35, 40), 0.75 + 0.5 * noise(y, x + k));
                        f.set(x, y, Some(c));
                    }
                }
            }
            f
        })
        .collect()
}
