//! Sixel encoder.
//!
//! A Sixel image is drawn in bands 6 pixels tall; in each band, every color
//! is a run of characters whose 6 bits say which of the 6 pixels of that
//! column have the color. Pixels no color sets stay transparent (`P2 = 1`),
//! so the terminal's background shows through the water.

use std::collections::HashMap;
use std::fmt::Write as _;

use super::canvas::{Canvas, unpack};

/// Sixel palettes have up to 256 registers; one is never used for clarity.
const MAX_COLORS: usize = 255;

/// The palette: exact colors when there are few enough, otherwise a
/// 6 x 7 x 6 color cube.
fn palette(canvas: &Canvas) -> (Vec<u32>, HashMap<u32, u8>) {
    let mut index = HashMap::new();
    let mut colors = Vec::new();
    for &p in &canvas.px {
        if p != 0 && !index.contains_key(&p) {
            if colors.len() == MAX_COLORS {
                return cube(canvas);
            }
            index.insert(p, colors.len() as u8);
            colors.push(p);
        }
    }
    (colors, index)
}

fn cube(canvas: &Canvas) -> (Vec<u32>, HashMap<u32, u8>) {
    let level = |v: u8, n: u32| (u32::from(v) * (n - 1) + 127) / 255;
    let value = |l: u32, n: u32| (l * 255 / (n - 1)) as u8;
    let mut colors = Vec::new();
    for r in 0..6 {
        for g in 0..7 {
            for b in 0..6 {
                colors.push(super::canvas::pack((value(r, 6), value(g, 7), value(b, 6))));
            }
        }
    }
    let mut index = HashMap::new();
    for &p in &canvas.px {
        if p != 0 {
            index.entry(p).or_insert_with(|| {
                let (r, g, b) = unpack(p);
                (level(r, 6) * 42 + level(g, 7) * 6 + level(b, 6)) as u8
            });
        }
    }
    (colors, index)
}

/// Append `ch` repeated `n` times, run-length encoded.
fn run(out: &mut String, ch: u8, n: usize) {
    match n {
        0 => {}
        1..=3 => (0..n).for_each(|_| out.push(ch as char)),
        _ => {
            let _ = write!(out, "!{n}{}", ch as char);
        }
    }
}

pub fn encode(canvas: &Canvas) -> Vec<u8> {
    let (w, h) = (canvas.width, canvas.height);
    let (colors, index) = palette(canvas);
    let mut out = String::with_capacity(w * h / 8);
    // DCS P1=0 (aspect from raster) P2=1 (transparent) q, 1:1 pixels
    let _ = write!(out, "\x1bP0;1;0q\"1;1;{w};{h}");
    for (i, &c) in colors.iter().enumerate() {
        let (r, g, b) = unpack(c);
        let pct = |v: u8| (u32::from(v) * 100 + 127) / 255;
        let _ = write!(out, "#{i};2;{};{};{}", pct(r), pct(g), pct(b));
    }

    // per band: the sixel bits of each column, for each color in the band
    let mut bits = vec![0u8; colors.len() * w];
    let mut used: Vec<usize> = Vec::new();
    let mut in_band = vec![false; colors.len()];
    for band in (0..h).step_by(6) {
        used.clear();
        for dy in 0..6.min(h - band) {
            let row = &canvas.px[(band + dy) * w..(band + dy + 1) * w];
            for (x, &p) in row.iter().enumerate() {
                if p == 0 {
                    continue;
                }
                let ci = usize::from(index[&p]);
                if !in_band[ci] {
                    in_band[ci] = true;
                    used.push(ci);
                }
                bits[ci * w + x] |= 1 << dy;
            }
        }
        for (n, &ci) in used.iter().enumerate() {
            if n > 0 {
                out.push('$'); // back to the start of the band
            }
            let _ = write!(out, "#{ci}");
            let slot = &mut bits[ci * w..(ci + 1) * w];
            // trailing empty columns need not be sent
            let end = slot.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
            let mut x = 0;
            while x < end {
                let b = slot[x];
                let len = slot[x..end].iter().take_while(|&&v| v == b).count();
                run(&mut out, 63 + b, len);
                x += len;
            }
            slot.fill(0);
            in_band[ci] = false;
        }
        out.push('-'); // next band
    }
    out.push_str("\x1b\\");
    out.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::canvas::pack;

    #[test]
    fn encodes_bands_colors_and_transparency() {
        let mut c = Canvas::new(8, 7);
        c.rect(0, 0, 8, 6, pack((255, 0, 0)));
        c.rect(2, 6, 3, 7, pack((0, 0, 255)));
        let s = String::from_utf8(encode(&c)).unwrap();
        assert!(s.starts_with("\x1bP0;1;0q\"1;1;8;7"));
        assert!(s.contains("#0;2;100;0;0") && s.contains("#1;2;0;0;100"));
        // band 1: 8 full red columns; band 2: 2 empty columns, one blue pixel
        assert!(s.contains("#0!8~-"));
        assert!(s.contains("#1??@-"));
        assert!(s.ends_with("\x1b\\"));
    }

    #[test]
    fn many_colors_fall_back_to_a_color_cube() {
        let mut c = Canvas::new(300, 1);
        for x in 0..300i32 {
            c.rect(x, 0, x + 1, 1, pack((x as u8, (x / 2) as u8, 7)));
        }
        let (colors, index) = palette(&c);
        assert_eq!(colors.len(), 252);
        assert!(index.values().all(|&i| usize::from(i) < 252));
    }
}
