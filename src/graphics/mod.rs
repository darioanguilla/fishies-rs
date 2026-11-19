//! Graphics mode: the scene as real pixels, sent with Sixel or the kitty
//! graphics protocol.
//!
//! The image covers every row but the last one: a Sixel image that reaches
//! the bottom of the screen makes the terminal scroll. The last row is drawn
//! as half blocks, like ANSI mode, and holds the status messages.

pub mod canvas;
pub mod detect;
mod kitty;
#[cfg(test)]
pub mod png;
mod query;
mod sixel;

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::{cursor::MoveTo, queue};

use crate::engine::Animation;
use canvas::{Canvas, Cells, compose};
pub use detect::{Protocol, Support};

/// Begin / end synchronized output: the terminal shows the frame at once.
const SYNC_BEGIN: &[u8] = b"\x1b[?2026h";
const SYNC_END: &[u8] = b"\x1b[?2026l";

pub struct Graphics {
    protocol: Protocol,
    cell: (u32, u32),
    cols: u16,
    image_rows: u16,
    canvas: Canvas,
    /// Size and encoding time of the last frame.
    pub last: Option<(usize, Duration)>,
}

impl Graphics {
    pub fn new(protocol: Protocol, cell: (u32, u32), cols: u16, rows: u16) -> Self {
        let image_rows = rows.saturating_sub(1);
        Graphics {
            protocol,
            cell,
            cols,
            image_rows,
            canvas: Canvas::new(
                usize::from(cols) * cell.0 as usize,
                usize::from(image_rows) * cell.1 as usize,
            ),
            last: None,
        }
    }

    pub fn draw(&mut self, out: &mut impl Write, anim: &Animation) -> io::Result<()> {
        if self.image_rows == 0 || self.cols == 0 {
            return Ok(());
        }
        let start = Instant::now();
        let cells = Cells {
            w: f64::from(self.cell.0),
            h: f64::from(self.cell.1),
        };
        compose(anim, &mut self.canvas, cells);
        let data = match self.protocol {
            Protocol::Sixel => sixel::encode(&self.canvas),
            Protocol::Kitty => kitty::encode(&self.canvas),
        };
        self.last = Some((data.len(), start.elapsed()));

        out.write_all(SYNC_BEGIN)?;
        if self.protocol == Protocol::Sixel {
            // transparent pixels don't erase, so remove the previous frame
            self.erase(out)?;
        }
        queue!(out, MoveTo(0, 0))?;
        out.write_all(&data)?;
        out.write_all(SYNC_END)?;
        out.flush()
    }

    /// Remove the image, when leaving graphics mode.
    pub fn clear(&self, out: &mut impl Write) -> io::Result<()> {
        match self.protocol {
            Protocol::Kitty => out.write_all(kitty::DELETE)?,
            Protocol::Sixel => self.erase(out)?,
        }
        out.flush()
    }

    /// Erase every cell covered by the image.
    fn erase(&self, out: &mut impl Write) -> io::Result<()> {
        if self.image_rows > 0 && self.cols > 0 {
            queue!(out, MoveTo(self.cols - 1, self.image_rows - 1))?;
            out.write_all(b"\x1b[1J")?;
        }
        Ok(())
    }

    /// e.g. `Sixel 1100x660 px, 84 KB/frame, 6.1 ms`
    pub fn stats(&self) -> Option<String> {
        self.last.map(|(bytes, t)| {
            format!(
                "{} {}x{} px, {} KB/frame, {:.1} ms",
                self.protocol.name(),
                self.canvas.width,
                self.canvas.height,
                bytes.div_ceil(1024),
                t.as_secs_f64() * 1000.0
            )
        })
    }
}

#[cfg(test)]
pub use kitty::encode as kitty_encode;
#[cfg(test)]
pub use sixel::encode as sixel_encode;
