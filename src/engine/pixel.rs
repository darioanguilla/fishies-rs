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

    /// Set a pixel, ignoring coordinates outside the frame.
    pub fn dot(&mut self, x: i32, y: i32, c: Rgb) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.set(x as usize, y as usize, Some(c));
        }
    }

    /// Paint every pixel whose center satisfies `inside`.
    pub fn fill(&mut self, c: Rgb, inside: impl Fn(f32, f32) -> bool) {
        for y in 0..self.height {
            for x in 0..self.width {
                if inside(x as f32 + 0.5, y as f32 + 0.5) {
                    self.set(x, y, Some(c));
                }
            }
        }
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: Rgb) {
        self.fill(c, |x, y| {
            let (dx, dy) = ((x - cx) / rx, (y - cy) / ry);
            dx * dx + dy * dy <= 1.0
        });
    }

    /// Fill a polygon (even-odd rule).
    pub fn poly(&mut self, pts: &[(f32, f32)], c: Rgb) {
        self.fill(c, |x, y| {
            let mut inside = false;
            let mut j = pts.len() - 1;
            for i in 0..pts.len() {
                let ((xi, yi), (xj, yj)) = (pts[i], pts[j]);
                if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
                    inside = !inside;
                }
                j = i;
            }
            inside
        });
    }

    /// A one-pixel line between two pixel coordinates.
    pub fn line(&mut self, (x0, y0): (f32, f32), (x1, y1): (f32, f32), c: Rgb) {
        let steps = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as i32;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let (x, y) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
            self.dot(x.round() as i32, y.round() as i32, c);
        }
    }

    /// Double the size with Scale2x (EPX): like nearest-neighbor, but a
    /// pixel's corner takes its neighbors' color where they meet, so
    /// diagonal edges come out smooth instead of staircased.
    pub fn scale2x(&self) -> Self {
        let mut f = PixFrame::new(self.width * 2, self.height * 2);
        let at = |x: isize, y: isize| {
            if x < 0 || y < 0 || x >= self.width as isize || y >= self.height as isize {
                None
            } else {
                self.get(x as usize, y as usize)
            }
        };
        for y in 0..self.height {
            for x in 0..self.width {
                let (xi, yi) = (x as isize, y as isize);
                let p = self.get(x, y);
                let (a, b, c, d) = (
                    at(xi, yi - 1),
                    at(xi + 1, yi),
                    at(xi - 1, yi),
                    at(xi, yi + 1),
                );
                let tl = if c == a && c != d && a != b { a } else { p };
                let tr = if a == b && a != c && b != d { b } else { p };
                let bl = if d == c && d != b && c != a { c } else { p };
                let br = if b == d && b != a && d != c { d } else { p };
                f.set(2 * x, 2 * y, tl);
                f.set(2 * x + 1, 2 * y, tr);
                f.set(2 * x, 2 * y + 1, bl);
                f.set(2 * x + 1, 2 * y + 1, br);
            }
        }
        f
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
    fn scale2x_smooths_diagonals() {
        let c = Some((9, 9, 9));
        // a diagonal staircase
        let f = PixFrame::from_grid(&["x.", "xx"], |ch| (ch == 'x').then_some((9, 9, 9)));
        let s = f.scale2x();
        assert_eq!((s.width, s.height), (4, 4));
        // the empty corner gets its bottom-left quarter filled, on the diagonal
        assert_eq!(s.get(2, 1), c);
        assert_eq!(s.get(3, 0), None);
        assert_eq!(s.get(2, 0), None);
        // solid areas stay solid
        assert_eq!(s.get(0, 0), c);
        assert_eq!(s.get(3, 3), c);
    }

    #[test]
    fn shapes() {
        let c = (1, 2, 3);
        let mut f = PixFrame::new(5, 5);
        f.poly(&[(0.0, 0.0), (5.0, 0.0), (0.0, 5.0)], c);
        assert!(f.get(0, 0).is_some() && f.get(4, 4).is_none());
        let mut e = PixFrame::new(5, 5);
        e.ellipse(2.5, 2.5, 2.5, 2.5, c);
        assert!(e.get(2, 2).is_some() && e.get(0, 0).is_none());
        let mut l = PixFrame::new(4, 4);
        l.line((0.0, 0.0), (3.0, 3.0), c);
        assert!((0..4).all(|i| l.get(i, i).is_some()));
        l.dot(-1, 9, c); // out of bounds is ignored
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
