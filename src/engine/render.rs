//! Character framebuffer and incremental terminal output.
//!
//! A cell holds either a glyph or two half-block pixels. The terminal's own
//! background shows through every empty cell and every empty half, so the
//! water stays transparent in both modes.

use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use super::pixel::PixFrame;
use super::shape::Frame;
use crate::color::{self, Rgb, Style};

/// How sprites are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    /// The original look: ASCII art in the 16 terminal colors.
    #[default]
    Ascii,
    /// Truecolor, Unicode glyphs and half-block pixel art where available.
    Ansi,
}

impl Mode {
    pub fn toggled(self) -> Self {
        match self {
            Mode::Ascii => Mode::Ansi,
            Mode::Ansi => Mode::Ascii,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Cell {
    #[default]
    Empty,
    Glyph {
        ch: char,
        fg: Color,
        bold: bool,
        bg: Option<Color>,
    },
    /// Upper and lower pixel of the cell.
    Halves {
        top: Option<Rgb>,
        bottom: Option<Rgb>,
    },
}

/// What a glyph looks like once a mode has styled it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub ch: char,
    pub fg: Color,
    pub bold: bool,
}

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
            cells: vec![Cell::Empty; width * height],
        }
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::Empty);
    }

    pub fn get(&self, col: usize, row: usize) -> Cell {
        self.cells[row * self.width + col]
    }

    fn cell_mut(&mut self, x: i32, y: i32) -> Option<&mut Cell> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        Some(&mut self.cells[y as usize * self.width + x as usize])
    }

    /// Draw an ASCII frame with its top-left corner at `(x, y)`, clipping
    /// to the buffer and skipping transparent cells. `style` turns each
    /// character into a glyph; it also gets the row and the frame height.
    pub fn blit(
        &mut self,
        frame: &Frame,
        x: i32,
        y: i32,
        style: impl Fn(char, Style, usize, usize) -> Glyph,
    ) {
        for row in 0..frame.height {
            for col in 0..frame.width {
                let Some((ch, st)) = frame.get(col, row) else {
                    continue;
                };
                let Some(cell) = self.cell_mut(x + col as i32, y + row as i32) else {
                    continue;
                };
                let g = style(ch, st, row, frame.height);
                // a glyph over a pixel-filled cell keeps that color behind it
                let bg = match *cell {
                    Cell::Halves {
                        top: Some(t),
                        bottom: Some(b),
                    } if t == b => Some(color::rgb(t)),
                    Cell::Glyph { bg, .. } => bg,
                    _ => None,
                };
                *cell = Cell::Glyph {
                    ch: g.ch,
                    fg: g.fg,
                    bold: g.bold,
                    bg,
                };
            }
        }
    }

    /// Draw a pixel frame with its top-left corner at cell `(x, y)`.
    pub fn blit_pixels(&mut self, frame: &PixFrame, x: i32, y: i32) {
        for py in 0..frame.height {
            for px in 0..frame.width {
                let Some(c) = frame.get(px, py) else {
                    continue;
                };
                let Some(cell) = self.cell_mut(x + px as i32, y + (py / 2) as i32) else {
                    continue;
                };
                let (mut top, mut bottom) = match *cell {
                    Cell::Halves { top, bottom } => (top, bottom),
                    _ => (None, None),
                };
                if py % 2 == 0 {
                    top = Some(c);
                } else {
                    bottom = Some(c);
                }
                *cell = Cell::Halves { top, bottom };
            }
        }
    }
}

/// The terminal output of a cell: character, colors and boldness.
#[derive(Clone, Copy, PartialEq)]
struct Out {
    ch: char,
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
}

fn output(cell: Cell) -> Out {
    let out = |ch, fg, bg| Out {
        ch,
        fg,
        bg,
        bold: false,
    };
    match cell {
        Cell::Empty => out(' ', None, None),
        Cell::Glyph { ch, fg, bold, bg } => Out {
            ch,
            fg: Some(fg),
            bg,
            bold,
        },
        Cell::Halves { top, bottom } => match (top, bottom) {
            (Some(t), Some(b)) if t == b => out('█', Some(color::rgb(t)), None),
            (Some(t), Some(b)) => out('▀', Some(color::rgb(t)), Some(color::rgb(b))),
            (Some(t), None) => out('▀', Some(color::rgb(t)), None),
            (None, Some(b)) => out('▄', Some(color::rgb(b)), None),
            (None, None) => out(' ', None, None),
        },
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
        let mut style: Option<(Option<Color>, Option<Color>, bool)> = None;
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
                    _ => cell != Cell::Empty,
                };
                if !changed {
                    continue;
                }
                if cursor != Some((col, row)) {
                    queue!(out, MoveTo(col as u16, row as u16))?;
                }
                let o = output(cell);
                let st = (o.fg, o.bg, o.bold);
                if style != Some(st) {
                    queue!(out, SetAttribute(Attribute::Reset))?;
                    if o.bold {
                        queue!(out, SetAttribute(Attribute::Bold))?;
                    }
                    if let Some(fg) = o.fg {
                        queue!(out, SetForegroundColor(fg))?;
                    }
                    if let Some(bg) = o.bg {
                        queue!(out, SetBackgroundColor(bg))?;
                    }
                    style = Some(st);
                }
                queue!(out, Print(o.ch))?;
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
pub mod snapshot {
    //! Render a buffer as a standalone HTML page, to make screenshots.

    use super::*;

    /// Windows Terminal's default ("Campbell") palette, for ASCII mode.
    fn campbell(c: Color) -> Rgb {
        match c {
            Color::Black => (12, 12, 12),
            Color::DarkRed => (197, 15, 31),
            Color::DarkGreen => (19, 161, 14),
            Color::DarkYellow => (193, 156, 0),
            Color::DarkBlue => (0, 55, 218),
            Color::DarkMagenta => (136, 23, 152),
            Color::DarkCyan => (58, 150, 221),
            Color::Grey => (204, 204, 204),
            Color::DarkGrey => (118, 118, 118),
            Color::Red => (231, 72, 86),
            Color::Green => (22, 198, 12),
            Color::Yellow => (249, 241, 165),
            Color::Blue => (59, 120, 255),
            Color::Magenta => (180, 0, 158),
            Color::Cyan => (97, 214, 214),
            Color::White => (242, 242, 242),
            Color::Rgb { r, g, b } => (r, g, b),
            _ => (204, 204, 204),
        }
    }

    fn css((r, g, b): Rgb) -> String {
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    /// Cell size in CSS pixels.
    pub const CELL_W: usize = 9;
    pub const CELL_H: usize = 18;

    /// `background` is any CSS background: the terminal's own background,
    /// which shows through every transparent cell.
    pub fn to_html(buf: &Buffer, background: &str) -> String {
        let mut s = format!(
            "<!doctype html><meta charset=utf-8><style>\
             body{{margin:0;background:{background}}}\
             .t{{font:15px 'Cascadia Mono',Consolas,monospace;color:#ccc;padding:9px}}\
             .r{{display:flex;height:{CELL_H}px}}\
             .r span{{display:block;flex:none;width:{CELL_W}px;height:{CELL_H}px;\
             line-height:{CELL_H}px;text-align:center;white-space:pre;overflow:hidden}}\
             </style><div class=t>"
        );
        for row in 0..buf.height {
            s.push_str("<div class=r>");
            for col in 0..buf.width {
                match buf.get(col, row) {
                    Cell::Empty => s.push_str("<span></span>"),
                    Cell::Glyph { ch, fg, bold, bg } => {
                        let ch = match ch {
                            '<' => "&lt;".to_string(),
                            '>' => "&gt;".to_string(),
                            '&' => "&amp;".to_string(),
                            c => c.to_string(),
                        };
                        let bg = bg.map_or(String::new(), |b| {
                            format!(";background:{}", css(campbell(b)))
                        });
                        let weight = if bold { ";font-weight:bold" } else { "" };
                        s.push_str(&format!(
                            "<span style=\"color:{}{bg}{weight}\">{ch}</span>",
                            css(campbell(fg))
                        ));
                    }
                    // half-blocks drawn as exact halves, like a terminal does
                    Cell::Halves { top, bottom } => {
                        let half = |c: Option<Rgb>| c.map_or("transparent".to_string(), css);
                        s.push_str(&format!(
                            "<span style=\"background:linear-gradient({} 50%,{} 50%)\"></span>",
                            half(top),
                            half(bottom)
                        ));
                    }
                }
            }
            s.push_str("</div>");
        }
        s.push_str("</div>");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::shape::Sprite;

    fn ascii(ch: char, st: Style, _: usize, _: usize) -> Glyph {
        Glyph {
            ch,
            fg: st.color,
            bold: st.bold,
        }
    }

    fn ch(buf: &Buffer, col: usize, row: usize) -> char {
        match buf.get(col, row) {
            Cell::Glyph { ch, .. } => ch,
            _ => ' ',
        }
    }

    #[test]
    fn blit_clips_and_respects_transparency() {
        let mut buf = Buffer::new(3, 2);
        let back = &Sprite::single("xxx\nxxx").build()[0];
        let front = &Sprite::single("a?b").build()[0];
        buf.blit(back, 0, 0, ascii);
        buf.blit(front, -1, 1, ascii);
        assert_eq!(ch(&buf, 0, 1), 'x');
        assert_eq!(ch(&buf, 1, 1), 'b');
        assert_eq!(ch(&buf, 2, 1), 'x');
    }

    #[test]
    fn pixels_fill_halves_and_keep_transparency() {
        let red = (255, 0, 0);
        let blue = (0, 0, 255);
        let f = PixFrame::from_grid(&["r.", "rb", ".b"], |c| match c {
            'r' => Some(red),
            'b' => Some(blue),
            _ => None,
        });
        let mut buf = Buffer::new(2, 2);
        buf.blit_pixels(&f, 0, 0);
        assert_eq!(
            buf.get(0, 0),
            Cell::Halves {
                top: Some(red),
                bottom: Some(red)
            }
        );
        assert_eq!(
            buf.get(1, 0),
            Cell::Halves {
                top: None,
                bottom: Some(blue)
            }
        );
        assert_eq!(output(buf.get(0, 0)).ch, '█');
        assert_eq!(output(buf.get(1, 0)).ch, '▄');
        assert_eq!(
            output(buf.get(1, 0)).bg,
            None,
            "empty half stays transparent"
        );
        assert_eq!(output(buf.get(1, 1)).ch, '▀');
        assert_eq!(buf.get(0, 1), Cell::Empty);
    }

    #[test]
    fn glyph_over_solid_pixels_keeps_background() {
        let f = PixFrame::from_grid(&["g", "g"], |_| Some((0, 200, 0)));
        let mut buf = Buffer::new(1, 1);
        buf.blit_pixels(&f, 0, 0);
        buf.blit(&Sprite::single("o").build()[0], 0, 0, ascii);
        assert!(matches!(
            buf.get(0, 0),
            Cell::Glyph {
                ch: 'o',
                bg: Some(_),
                ..
            }
        ));
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
        buf.blit(&Sprite::single("z").build()[0], 1, 0, ascii);
        out.clear();
        screen.draw(&mut out, &buf).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains('z'));
        assert!(s.len() > unchanged);
    }
}
