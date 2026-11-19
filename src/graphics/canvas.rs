//! An RGBA image of the scene at the terminal's real pixel resolution.

use crate::color::Rgb;
use crate::engine::{Animation, Entity, Kind};

/// Pixels are `0` (transparent) or `0xFF_RR_GG_BB`.
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub px: Vec<u32>,
}

pub fn pack((r, g, b): Rgb) -> u32 {
    0xFF00_0000 | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

pub fn unpack(p: u32) -> Rgb {
    ((p >> 16) as u8, (p >> 8) as u8, p as u8)
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Canvas {
            width,
            height,
            px: vec![0; width * height],
        }
    }

    pub fn clear(&mut self) {
        self.px.fill(0);
    }

    #[cfg(test)]
    pub fn get(&self, x: usize, y: usize) -> u32 {
        self.px[y * self.width + x]
    }

    /// Fill `[x0, x1) x [y0, y1)`, clipped.
    pub fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: u32) {
        let (x0, x1) = (x0.max(0) as usize, (x1.max(0) as usize).min(self.width));
        let (y0, y1) = (y0.max(0) as usize, (y1.max(0) as usize).min(self.height));
        for y in y0..y1 {
            self.px[y * self.width + x0.min(x1)..y * self.width + x1].fill(c);
        }
    }
}

/// Maps scene coordinates (cells, as floats) to canvas pixels.
#[derive(Clone, Copy)]
pub struct Cells {
    pub w: f64,
    pub h: f64,
}

impl Cells {
    fn x(self, cells: f64) -> i32 {
        (cells * self.w).round() as i32
    }

    fn y(self, cells: f64) -> i32 {
        (cells * self.h).round() as i32
    }
}

/// Draw the scene, back to front. Positions keep their fractional part, so
/// sprites move pixel by pixel instead of cell by cell.
pub fn compose(anim: &Animation, canvas: &mut Canvas, cells: Cells) {
    canvas.clear();
    let mut order: Vec<&Entity> = anim.entities.iter().filter(|e| e.alive).collect();
    order.sort_by(|a, b| b.z.total_cmp(&a.z));
    for e in order {
        if let Some(f) = e.pixel_frame() {
            // a pixel-art pixel is one cell wide and half a cell tall
            for py in 0..f.height {
                let (y0, y1) = (
                    cells.y(e.y + py as f64 / 2.0),
                    cells.y(e.y + (py + 1) as f64 / 2.0),
                );
                for px in 0..f.width {
                    if let Some(c) = f.get(px, py) {
                        let (x0, x1) = (cells.x(e.x + px as f64), cells.x(e.x + px as f64 + 1.0));
                        canvas.rect(x0, y0, x1, y1, pack(c));
                    }
                }
            }
        } else {
            match e.kind {
                Kind::Bubble => bubble(e, canvas, cells),
                Kind::FishLine => fishing_line(e, canvas, cells),
                _ => {}
            }
        }
    }
}

/// A ring that grows as the bubble rises (the ASCII `.`, `o`, `O`).
fn bubble(e: &Entity, canvas: &mut Canvas, cells: Cells) {
    let Some((ch, _)) = e.frame().get(0, 0) else {
        return;
    };
    let (cx, cy) = ((e.x + 0.5) * cells.w, (e.y + 0.5) * cells.h);
    let r = match ch {
        '.' => 0.12,
        'o' => 0.28,
        _ => 0.42,
    } * cells.w;
    let (ring, shine) = (pack((150, 215, 250)), pack((235, 250, 255)));
    let (x0, y0) = ((cx - r - 1.0) as i32, (cy - r - 1.0) as i32);
    for y in y0..=(cy + r + 1.0) as i32 {
        for x in x0..=(cx + r + 1.0) as i32 {
            let d = ((x as f64 + 0.5 - cx).powi(2) + (y as f64 + 0.5 - cy).powi(2)).sqrt();
            // a ring, or a filled speck while the bubble is tiny
            let c = if (r < 1.5 && d <= r + 0.5) || (d - r).abs() <= 0.6 {
                ring
            } else if d < r * 0.5 && x as f64 + 0.5 < cx && (y as f64 + 0.5) < cy - r * 0.2 {
                shine
            } else {
                continue;
            };
            canvas.rect(x, y, x + 1, y + 1, c);
        }
    }
}

/// One thin line where the ASCII line has `|`.
fn fishing_line(e: &Entity, canvas: &mut Canvas, cells: Cells) {
    let f = e.frame();
    let x = cells.x(e.x + 0.5);
    for row in 0..f.height {
        if f.get(0, row).is_some_and(|(c, _)| c == '|') {
            let (y0, y1) = (cells.y(e.y + row as f64), cells.y(e.y + row as f64 + 1.0));
            canvas.rect(x, y0, x + 1, y1, pack((200, 200, 205)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{PixFrame, Sprite};

    const CELLS: Cells = Cells { w: 10.0, h: 20.0 };

    #[test]
    fn a_pixel_is_a_cell_wide_and_half_a_cell_tall() {
        let mut anim = Animation::new(4, 2);
        let px = PixFrame::from_grid(&["r"], |_| Some((255, 0, 0)));
        anim.add(Entity::new(Kind::Other, Sprite::single("x").build()).pixels(vec![px]));
        anim.flush();
        let mut c = Canvas::new(40, 40);
        compose(&anim, &mut c, CELLS);
        assert_ne!(c.get(0, 0), 0);
        assert_ne!(c.get(9, 9), 0);
        assert_eq!(c.get(10, 0), 0);
        assert_eq!(c.get(0, 10), 0);
    }

    #[test]
    fn fractional_positions_move_by_pixels() {
        let mut anim = Animation::new(4, 2);
        let px = PixFrame::from_grid(&["r"], |_| Some((255, 0, 0)));
        anim.add(
            Entity::new(Kind::Other, Sprite::single("x").build())
                .pixels(vec![px])
                .at(0.35, 0.0, 0.0),
        );
        anim.flush();
        let mut c = Canvas::new(40, 40);
        compose(&anim, &mut c, CELLS);
        assert_eq!(c.get(3, 0), 0);
        assert_ne!(c.get(4, 0), 0);
        assert_ne!(c.get(13, 0), 0);
    }

    #[test]
    fn rect_is_clipped() {
        let mut c = Canvas::new(4, 4);
        c.rect(-5, -5, 2, 2, 1);
        c.rect(3, 3, 99, 99, 2);
        assert_eq!((c.get(1, 1), c.get(2, 2), c.get(3, 3)), (1, 0, 2));
    }
}
