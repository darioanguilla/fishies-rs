//! Character framebuffer and incremental terminal output.

use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Attribute, Print, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use super::shape::{Cell, Frame};
use crate::color::Style;

#[derive(Clone, PartialEq)]
pub struct Buffer {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

impl Buffer {
    pub fn new(width: i32, height: i32) -> Self {
        let (width, height) = (width.max(0) as usize, height.max(0) as usize);
        Buffer {
            width,
            height,
            cells: vec![None; width * height],
        }
    }

    pub fn clear(&mut self) {
        self.cells.fill(None);
    }

    pub fn get(&self, col: usize, row: usize) -> Cell {
        self.cells[row * self.width + col]
    }

    /// Draw a frame with its top-left corner at `(x, y)`, clipping to the
    /// buffer and skipping transparent cells.
    pub fn blit(&mut self, frame: &Frame, x: i32, y: i32) {
        for row in 0..frame.height {
            let by = y + row as i32;
            if by < 0 || by >= self.height as i32 {
                continue;
            }
            for col in 0..frame.width {
                let bx = x + col as i32;
                if bx < 0 || bx >= self.width as i32 {
                    continue;
                }
                if let Some(cell) = frame.get(col, row) {
                    self.cells[by as usize * self.width + bx as usize] = Some(cell);
                }
            }
        }
    }
}

/// Writes buffers to the terminal, only sending cells that changed.
#[derive(Default)]
pub struct Screen {
    prev: Option<Buffer>,
}

impl Screen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, out: &mut impl Write, buf: &Buffer) -> io::Result<()> {
        let full = self
            .prev
            .as_ref()
            .is_none_or(|p| p.width != buf.width || p.height != buf.height);
        if full {
            queue!(out, SetAttribute(Attribute::Reset), Clear(ClearType::All))?;
        }
        let mut style: Option<Style> = None;
        let mut cursor: Option<(usize, usize)> = None;
        for row in 0..buf.height {
            for col in 0..buf.width {
                // Writing the bottom-right cell can scroll some terminals.
                if row + 1 == buf.height && col + 1 == buf.width {
                    continue;
                }
                let cell = buf.get(col, row);
                let changed = match &self.prev {
                    Some(p) if !full => p.get(col, row) != cell,
                    _ => cell.is_some(),
                };
                if !changed {
                    continue;
                }
                if cursor != Some((col, row)) {
                    queue!(out, MoveTo(col as u16, row as u16))?;
                }
                let (ch, st) = match cell {
                    Some((ch, st)) => (ch, Some(st)),
                    None => (' ', None),
                };
                if st != style {
                    queue!(out, SetAttribute(Attribute::Reset))?;
                    if let Some(s) = st {
                        if s.bold {
                            queue!(out, SetAttribute(Attribute::Bold))?;
                        }
                        queue!(out, SetForegroundColor(s.color))?;
                    }
                    style = st;
                }
                queue!(out, Print(ch))?;
                cursor = Some((col + 1, row));
            }
        }
        queue!(out, SetAttribute(Attribute::Reset))?;
        out.flush()?;
        self.prev = Some(buf.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::shape::Sprite;

    #[test]
    fn blit_clips_and_respects_transparency() {
        let mut buf = Buffer::new(3, 2);
        let back = &Sprite::single("xxx\nxxx").build()[0];
        let front = &Sprite::single("a?b").build()[0];
        buf.blit(back, 0, 0);
        buf.blit(front, -1, 1);
        assert_eq!(buf.get(0, 1).unwrap().0, 'x');
        assert_eq!(buf.get(1, 1).unwrap().0, 'b');
        assert_eq!(buf.get(2, 1).unwrap().0, 'x');
    }

    #[test]
    fn only_changes_are_written() {
        let mut screen = Screen::new();
        let mut buf = Buffer::new(4, 2);
        let mut out = Vec::new();
        screen.draw(&mut out, &buf).unwrap();
        out.clear();
        screen.draw(&mut out, &buf).unwrap();
        let unchanged = out.len();
        buf.blit(&Sprite::single("z").build()[0], 1, 0);
        out.clear();
        screen.draw(&mut out, &buf).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains('z'));
        assert!(s.len() > unchanged);
    }
}
