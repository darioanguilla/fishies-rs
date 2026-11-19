//! Pixel frames for ANSI mode. Every terminal cell holds two pixels stacked
//! vertically (drawn with half-block characters), so a sprite that is `w x h`
//! cells is `w x 2h` pixels. `None` pixels are transparent.

use crate::color::{Rgb, shade};

#[derive(Clone, Debug, PartialEq)]
pub struct PixFrame {
    pub width: usize,
    pub height: usize,
    pub px: Vec<Option<Rgb>>,
}

impl PixFrame {
    pub fn new(width: usize, height: usize) -> Self {
        PixFrame {
            width,
            height,
            px: vec![None; width * height],
        }
    }

    /// Build a frame from rows of characters, mapping each character to a
    /// color (or `None` for transparent). Short rows are padded.
    pub fn from_grid<S: AsRef<str>>(rows: &[S], color: impl Fn(char) -> Option<Rgb>) -> Self {
        let width = rows
            .iter()
            .map(|r| r.as_ref().chars().count())
            .max()
            .unwrap_or(0);
        let mut f = PixFrame::new(width, rows.len());
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.as_ref().chars().enumerate() {
                f.set(x, y, color(ch));
            }
        }
        f
    }

    pub fn get(&self, x: usize, y: usize) -> Option<Rgb> {
        self.px[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, c: Option<Rgb>) {
        self.px[y * self.width + x] = c;
    }

    /// Flip horizontally.
    pub fn mirrored(&self) -> Self {
        let mut f = PixFrame::new(self.width, self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                f.set(self.width - 1 - x, y, self.get(x, y));
            }
        }
        f
    }

    /// Vertical light: brightness goes linearly from `top` to `bottom`.
    pub fn shaded(mut self, top: f32, bottom: f32) -> Self {
        let span = (self.height.max(2) - 1) as f32;
        for y in 0..self.height {
            let f = top + (bottom - top) * y as f32 / span;
            for x in 0..self.width {
                let c = self.get(x, y).map(|c| shade(c, f));
                self.set(x, y, c);
            }
        }
        self
    }

    /// Darken the pixels on the silhouette's edge, for a crisper outline.
    pub fn outlined(mut self, factor: f32) -> Self {
        let src = self.clone();
        let empty = |x: isize, y: isize| {
            x < 0
                || y < 0
                || x >= src.width as isize
                || y >= src.height as isize
                || src.get(x as usize, y as usize).is_none()
        };
        for y in 0..self.height {
            for x in 0..self.width {
                let (xi, yi) = (x as isize, y as isize);
                if let Some(c) = src.get(x, y)
                    && (empty(xi - 1, yi)
                        || empty(xi + 1, yi)
                        || empty(xi, yi - 1)
                        || empty(xi, yi + 1))
                {
                    self.set(x, y, Some(shade(c, factor)));
                }
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> PixFrame {
        PixFrame::from_grid(&["ab", "c"], |c| match c {
            'a' => Some((10, 10, 10)),
            'b' => Some((20, 20, 20)),
            'c' => Some((30, 30, 30)),
            _ => None,
        })
    }

    #[test]
    fn grid_pads_short_rows() {
        let f = grid();
        assert_eq!((f.width, f.height), (2, 2));
        assert_eq!(f.get(1, 1), None);
    }

    #[test]
    fn mirror() {
        let m = grid().mirrored();
        assert_eq!(m.get(0, 0), Some((20, 20, 20)));
        assert_eq!(m.get(1, 1), Some((30, 30, 30)));
    }

    #[test]
    fn shading_and_outline() {
        let f = grid().shaded(2.0, 1.0);
        assert_eq!(f.get(0, 0), Some((20, 20, 20)));
        assert_eq!(f.get(0, 1), Some((30, 30, 30)));
        let o =
            PixFrame::from_grid(&["xxx", "xxx", "xxx"], |_| Some((100, 100, 100))).outlined(0.5);
        assert_eq!(o.get(1, 1), Some((100, 100, 100)));
        assert_eq!(o.get(0, 1), Some((50, 50, 50)));
    }
}
