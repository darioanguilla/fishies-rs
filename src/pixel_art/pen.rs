//! Drawing at two resolutions with the same code.
//!
//! Sprites are described in ANSI-mode pixel coordinates (one pixel per cell
//! column, two per cell row). A [`Pen`] scales them: at [`Res::Low`] it
//! draws exactly those pixels, at [`Res::High`] the same shapes with
//! [`HIRES`] times as many pixels each way, so curves come out smooth.

use crate::color::Rgb;
use crate::engine::{HIRES, PixFrame};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res {
    /// ANSI mode: half blocks.
    Low,
    /// Graphics mode.
    High,
}

impl Res {
    pub fn scale(self) -> usize {
        match self {
            Res::Low => 1,
            Res::High => HIRES,
        }
    }

    /// Enlarge low-resolution grid art for this resolution, smoothing its
    /// diagonals (Scale2x, repeated).
    pub fn upscale(self, mut f: PixFrame) -> PixFrame {
        let mut s = 1;
        while s < self.scale() {
            f = f.scale2x();
            s *= 2;
        }
        f
    }
}

pub struct Pen {
    pub f: PixFrame,
    s: f32,
}

impl Pen {
    /// A canvas of `width` x `height` low-resolution pixels.
    pub fn new(width: usize, height: usize, res: Res) -> Self {
        let s = res.scale();
        Pen {
            f: PixFrame::new(width * s, height * s),
            s: s as f32,
        }
    }

    pub fn high(&self) -> bool {
        self.s > 1.0
    }

    /// Paint where `inside(x, y)` holds, in low-resolution coordinates.
    pub fn fill(&mut self, c: Rgb, inside: impl Fn(f32, f32) -> bool) {
        let s = self.s;
        self.f.fill(c, |x, y| inside(x / s, y / s));
    }

    pub fn poly(&mut self, pts: &[(f32, f32)], c: Rgb) {
        let s = self.s;
        let pts: Vec<(f32, f32)> = pts.iter().map(|&(x, y)| (x * s, y * s)).collect();
        self.f.poly(&pts, c);
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: Rgb) {
        let s = self.s;
        self.f.ellipse(cx * s, cy * s, rx * s, ry * s, c);
    }

    /// A disc of radius `r`, for round details at high resolution.
    pub fn disc(&mut self, cx: f32, cy: f32, r: f32, c: Rgb) {
        self.ellipse(cx, cy, r, r, c);
    }

    /// A line one low-resolution pixel thick.
    pub fn line(&mut self, a: (f32, f32), b: (f32, f32), c: Rgb) {
        if !self.high() {
            self.f.line(a, b, c);
            return;
        }
        let s = self.s;
        let steps = ((b.0 - a.0).abs().max((b.1 - a.1).abs()) * s * 2.0)
            .ceil()
            .max(1.0) as i32;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let (x, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            self.square(x, y, c);
        }
    }

    /// A line one high-resolution pixel thin, for details that only exist at
    /// high resolution (rigging, fin rays).
    pub fn hairline(&mut self, a: (f32, f32), b: (f32, f32), c: Rgb) {
        let s = self.s;
        self.f.line((a.0 * s, a.1 * s), (b.0 * s, b.1 * s), c);
    }

    /// One low-resolution pixel.
    pub fn dot(&mut self, x: i32, y: i32, c: Rgb) {
        self.square(x as f32, y as f32, c);
    }

    fn square(&mut self, x: f32, y: f32, c: Rgb) {
        let s = self.s as i32;
        let (x0, y0) = ((x * self.s) as i32, (y * self.s) as i32);
        for dy in 0..s {
            for dx in 0..s {
                self.f.dot(x0 + dx, y0 + dy, c);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_shape_at_both_resolutions() {
        let draw = |res| {
            let mut p = Pen::new(4, 4, res);
            p.poly(&[(0.0, 0.0), (4.0, 0.0), (0.0, 4.0)], (1, 1, 1));
            p.dot(3, 3, (2, 2, 2));
            p.f
        };
        let (lo, hi) = (draw(Res::Low), draw(Res::High));
        assert_eq!((lo.width, hi.width), (4, 4 * HIRES));
        assert_eq!(lo.get(0, 0), Some((1, 1, 1)));
        assert_eq!(hi.get(0, 0), Some((1, 1, 1)));
        assert_eq!(lo.get(3, 3), Some((2, 2, 2)));
        assert_eq!(hi.get(4 * HIRES - 1, 4 * HIRES - 1), Some((2, 2, 2)));
    }

    #[test]
    fn upscale_multiplies_the_size() {
        let f = PixFrame::new(3, 2);
        let up = Res::High.upscale(f.clone());
        assert_eq!((up.width, up.height), (3 * HIRES, 2 * HIRES));
        assert_eq!(Res::Low.upscale(f).width, 3);
    }
}
