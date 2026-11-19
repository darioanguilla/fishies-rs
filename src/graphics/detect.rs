//! Find out whether, and how, the terminal can show real pixels.
//!
//! The terminal is asked rather than guessed from its name, because the
//! answer depends on version and settings (Windows Terminal has Sixel only
//! from 1.22, xterm only when built and configured for it, VTE behind a
//! flag...). One write sends four queries:
//!
//! 1. a kitty graphics query (`ESC _ G i=31,a=q ... ESC \`): terminals with
//!    the kitty graphics protocol answer `ESC _ G i=31;OK ESC \`, the
//!    others ignore it;
//! 2. `CSI 16 t`: the size of a cell in pixels (`CSI 6 ; h ; w t`);
//! 3. `CSI 14 t`: the size of the text area in pixels (`CSI 4 ; h ; w t`);
//! 4. `CSI c` (DA1, primary device attributes): every terminal answers it,
//!    and attribute `4` means Sixel. Terminals answer in order, so this
//!    reply also marks the end of the others.
//!
//! The kitty protocol is preferred: it has a real alpha channel and needs
//! less bandwidth. `FISHIES_GRAPHICS` (`auto`, `kitty`, `sixel`, `off`) and
//! `FISHIES_CELL` (e.g. `10x20`) override the detection.

use std::env;
use std::io::IsTerminal;
use std::time::Duration;

use super::query;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    Kitty,
    Sixel,
}

impl Protocol {
    pub fn name(self) -> &'static str {
        match self {
            Protocol::Kitty => "kitty graphics protocol",
            Protocol::Sixel => "Sixel",
        }
    }
}

/// Where the cell size came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellSource {
    Terminal,
    TextArea,
    System,
    Setting,
    Default,
}

#[derive(Clone, Debug)]
pub struct Support {
    /// `None` when graphics are not available; `reason` says why.
    pub protocol: Option<Protocol>,
    pub reason: String,
    /// Cell size in pixels.
    pub cell: (u32, u32),
    pub cell_source: CellSource,
    pub replies: Replies,
}

/// What the terminal answered.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Replies {
    pub kitty: bool,
    /// DA1 attributes; `None` if the terminal did not answer.
    pub attributes: Option<Vec<u32>>,
    pub cell: Option<(u32, u32)>,
    pub text_area: Option<(u32, u32)>,
    /// The raw reply, escaped for display.
    pub raw: String,
}

const QUERY: &str = "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[16t\x1b[14t\x1b[c";
const TIMEOUT: Duration = Duration::from_millis(500);
/// The cell size Sixel assumes on VT340-compatible terminals, including
/// Windows Terminal, which scales Sixel images from it.
const DEFAULT_CELL: (u32, u32) = (10, 20);

/// The DA1 reply is complete: `ESC [ ? ... c`.
fn has_attributes(reply: &[u8]) -> bool {
    find_csi(reply, b'?', b'c').is_some()
}

/// The parameters of the first `ESC [ <lead> params <end>` sequence.
fn find_csi(reply: &[u8], lead: u8, end: u8) -> Option<Vec<u32>> {
    let mut i = 0;
    while i + 2 < reply.len() {
        if reply[i] == 0x1b && reply[i + 1] == b'[' && reply[i + 2] == lead {
            let start = i + 3;
            let len = reply[start..]
                .iter()
                .position(|&b| !(b.is_ascii_digit() || b == b';'))?;
            if reply[start + len] == end {
                let params = std::str::from_utf8(&reply[start..start + len]).ok()?;
                return Some(params.split(';').filter_map(|p| p.parse().ok()).collect());
            }
        }
        i += 1;
    }
    None
}

/// `ESC [ <kind> ; height ; width t` as `(width, height)`.
fn size_reply(reply: &[u8], kind: u8) -> Option<(u32, u32)> {
    // the kind digit is the lead byte, so the parameters are `;h;w`
    match find_csi(reply, b'0' + kind, b't')?[..] {
        [h, w] if h > 0 && w > 0 => Some((w, h)),
        _ => None,
    }
}

pub fn parse(reply: &[u8]) -> Replies {
    let raw = reply
        .iter()
        .map(|&b| match b {
            0x1b => "ESC".to_string(),
            b if b.is_ascii_graphic() || b == b' ' => (b as char).to_string(),
            b => format!("\\x{b:02x}"),
        })
        .collect();
    Replies {
        kitty: reply.windows(9).any(|w| w == b"_Gi=31;OK"),
        attributes: find_csi(reply, b'?', b'c'),
        cell: size_reply(reply, 6),
        text_area: size_reply(reply, 4),
        raw,
    }
}

fn setting(name: &str) -> Option<String> {
    env::var(name).ok().map(|v| v.trim().to_ascii_lowercase())
}

/// Detect graphics support for a terminal of `cols` x `rows` cells. The
/// terminal must be in raw mode.
pub fn detect(cols: u16, rows: u16) -> Support {
    let forced = match setting("FISHIES_GRAPHICS").as_deref() {
        Some("off" | "none" | "no" | "0") => {
            return unavailable("disabled with FISHIES_GRAPHICS=off", Replies::default());
        }
        Some("kitty") => Some(Protocol::Kitty),
        Some("sixel") => Some(Protocol::Sixel),
        _ => None,
    };
    if !std::io::stdout().is_terminal() || !std::io::stdin().is_terminal() {
        return unavailable("not running in a terminal", Replies::default());
    }
    let in_multiplexer = env::var_os("TMUX").is_some()
        || env::var("TERM").is_ok_and(|t| t.starts_with("screen") || t.starts_with("tmux"));
    if in_multiplexer && forced.is_none() {
        return unavailable(
            "running inside tmux or screen, which need extra setup for graphics \
             (set FISHIES_GRAPHICS=sixel or kitty to try anyway)",
            Replies::default(),
        );
    }

    let replies = match query::ask(QUERY.as_bytes(), TIMEOUT, has_attributes) {
        Ok(reply) => parse(&reply),
        Err(e) => {
            return unavailable(
                &format!("could not ask the terminal: {e}"),
                Replies::default(),
            );
        }
    };
    let protocol = match (forced, &replies.attributes) {
        (Some(p), _) => p,
        (None, None) => {
            return unavailable("the terminal did not answer the capability query", replies);
        }
        (None, Some(_)) if replies.kitty => Protocol::Kitty,
        (None, Some(attrs)) if attrs.contains(&4) => Protocol::Sixel,
        (None, Some(_)) => {
            return unavailable(
                "this terminal supports neither Sixel nor the kitty graphics protocol",
                replies,
            );
        }
    };
    let reason = match forced {
        Some(_) => "chosen with FISHIES_GRAPHICS".to_string(),
        None => "reported by the terminal".to_string(),
    };
    let (cell, cell_source) = cell_size(&replies, cols, rows);
    Support {
        protocol: Some(protocol),
        reason,
        cell,
        cell_source,
        replies,
    }
}

fn unavailable(reason: &str, replies: Replies) -> Support {
    Support {
        protocol: None,
        reason: reason.to_string(),
        cell: DEFAULT_CELL,
        cell_source: CellSource::Default,
        replies,
    }
}

fn cell_size(replies: &Replies, cols: u16, rows: u16) -> ((u32, u32), CellSource) {
    if let Some(v) = setting("FISHIES_CELL")
        && let Some((w, h)) = v.split_once('x')
        && let (Ok(w), Ok(h)) = (w.parse::<u32>(), h.parse::<u32>())
        && w > 0
        && h > 0
    {
        return ((w, h), CellSource::Setting);
    }
    if let Some(c) = replies.cell {
        return (c, CellSource::Terminal);
    }
    let (cols, rows) = (u32::from(cols.max(1)), u32::from(rows.max(1)));
    if let Some((w, h)) = replies.text_area {
        return ((w / cols, h / rows), CellSource::TextArea);
    }
    if let Ok(ws) = crossterm::terminal::window_size()
        && ws.width > 0
        && ws.height > 0
    {
        let (w, h) = (u32::from(ws.width) / cols, u32::from(ws.height) / rows);
        if w > 0 && h > 0 {
            return ((w, h), CellSource::System);
        }
    }
    (DEFAULT_CELL, CellSource::Default)
}

/// A human-readable report, for `fishies --detect`.
pub fn report(s: &Support) -> String {
    let yes_no = |b: bool| if b { "yes" } else { "no" };
    let r = &s.replies;
    let sixel = r.attributes.as_ref().map(|a| a.contains(&4));
    let source = match s.cell_source {
        CellSource::Terminal => "reported by the terminal",
        CellSource::TextArea => "text area size / cells",
        CellSource::System => "window size from the system / cells",
        CellSource::Setting => "set with FISHIES_CELL",
        CellSource::Default => "default, not reported",
    };
    let hint = |name: &str| env::var(name).unwrap_or_else(|_| "-".into());
    format!(
        "terminal          TERM={} TERM_PROGRAM={} WT_SESSION={}\n\
         reply             {}\n\
         kitty graphics    {}\n\
         Sixel (DA1 has 4) {}\n\
         cell size         {}x{} px ({source})\n\
         graphics mode     {} ({})",
        hint("TERM"),
        hint("TERM_PROGRAM"),
        if env::var_os("WT_SESSION").is_some() {
            "set"
        } else {
            "-"
        },
        if r.raw.is_empty() { "(none)" } else { &r.raw },
        yes_no(r.kitty),
        sixel.map_or("no answer", yes_no),
        s.cell.0,
        s.cell.1,
        s.protocol.map_or("not available", Protocol::name),
        s.reason,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_terminal_reply() {
        // Windows Terminal 1.25: no kitty, Sixel (4), cell size reported
        let r = parse(b"\x1b[6;20;10t\x1b[4;800;1200t\x1b[?61;4;6;7;14;21;22;23;24;28;32;42;52c");
        assert!(!r.kitty);
        assert!(r.attributes.unwrap().contains(&4));
        assert_eq!(r.cell, Some((10, 20)));
        assert_eq!(r.text_area, Some((1200, 800)));
    }

    #[test]
    fn kitty_reply() {
        let r = parse(b"\x1b_Gi=31;OK\x1b\\\x1b[6;18;9t\x1b[?62;c");
        assert!(r.kitty);
        assert_eq!(r.cell, Some((9, 18)));
        assert!(!r.attributes.unwrap().contains(&4));
    }

    #[test]
    fn plain_terminal_reply() {
        // e.g. macOS Terminal.app or the old Windows console: only DA1
        let r = parse(b"\x1b[?1;2c");
        assert!(!r.kitty && r.cell.is_none());
        assert_eq!(r.attributes, Some(vec![1, 2]));
        assert!(has_attributes(b"\x1b[?1;2c"));
        assert!(!has_attributes(b"\x1b[?1;2"));
    }

    #[test]
    fn raw_reply_is_printable() {
        assert_eq!(parse(b"\x1b[?4c").raw, "ESC[?4c");
    }
}
