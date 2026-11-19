//! Color handling compatible with Term::Animation color masks.
//!
//! Mask letters: `r g b c m y w k` (red, green, blue, cyan, magenta, yellow,
//! white, black). Lowercase is the normal color, uppercase is the bright/bold
//! variant, just like curses `A_BOLD` on most terminals.

use crossterm::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub color: Color,
    pub bold: bool,
}

impl Style {
    pub const WHITE: Style = Style {
        color: Color::Grey,
        bold: false,
    };
}

/// Map a single mask letter to a style. Returns `None` for anything that is
/// not a color letter (spaces, digits, ...), meaning "use the default color".
pub fn from_letter(c: char) -> Option<Style> {
    let bold = c.is_ascii_uppercase();
    let color = match (c.to_ascii_lowercase(), bold) {
        ('r', false) => Color::DarkRed,
        ('r', true) => Color::Red,
        ('g', false) => Color::DarkGreen,
        ('g', true) => Color::Green,
        ('b', false) => Color::DarkBlue,
        ('b', true) => Color::Blue,
        ('c', false) => Color::DarkCyan,
        ('c', true) => Color::Cyan,
        ('m', false) => Color::DarkMagenta,
        ('m', true) => Color::Magenta,
        ('y', false) => Color::DarkYellow,
        ('y', true) => Color::Yellow,
        ('w', false) => Color::Grey,
        ('w', true) => Color::White,
        ('k', false) => Color::Black,
        ('k', true) => Color::DarkGrey,
        _ => return None,
    };
    Some(Style { color, bold })
}

/// Map a color name such as `"cyan"` or `"BLACK"` to a style. An all-uppercase
/// name selects the bright variant. Unknown names fall back to white.
pub fn named(name: &str) -> Style {
    let lower = name.to_ascii_lowercase();
    let letter = match lower.as_str() {
        "black" => 'k',
        other => other.chars().next().unwrap_or('w'),
    };
    let letter = if name.chars().all(|c| c.is_ascii_uppercase()) {
        letter.to_ascii_uppercase()
    } else {
        letter
    };
    from_letter(letter).unwrap_or(Style::WHITE)
}

/// A 24-bit color.
pub type Rgb = (u8, u8, u8);

pub fn rgb(c: Rgb) -> Color {
    Color::Rgb {
        r: c.0,
        g: c.1,
        b: c.2,
    }
}

/// The softer truecolor palette used in ANSI mode.
pub fn ansi_rgb(color: Color) -> Rgb {
    match color {
        Color::Black => (55, 55, 65),
        Color::DarkGrey => (125, 125, 135),
        Color::DarkRed => (205, 70, 60),
        Color::Red => (255, 105, 90),
        Color::DarkGreen => (60, 165, 75),
        Color::Green => (125, 225, 110),
        Color::DarkYellow => (220, 165, 40),
        Color::Yellow => (255, 220, 95),
        Color::DarkBlue => (60, 100, 210),
        Color::Blue => (110, 155, 255),
        Color::DarkMagenta => (175, 80, 185),
        Color::Magenta => (240, 130, 235),
        Color::DarkCyan => (40, 170, 195),
        Color::Cyan => (120, 230, 245),
        Color::Grey => (200, 200, 205),
        Color::White => (250, 250, 250),
        Color::Rgb { r, g, b } => (r, g, b),
        _ => (200, 200, 205),
    }
}

/// Truecolor value of a mask letter, see [`from_letter`].
pub fn letter_rgb(c: char) -> Option<Rgb> {
    from_letter(c).map(|s| ansi_rgb(s.color))
}

/// Scale a color's brightness; `f > 1` lightens, `f < 1` darkens.
pub fn shade(c: Rgb, f: f32) -> Rgb {
    let s = |v: u8| (v as f32 * f).round().clamp(0.0, 255.0) as u8;
    (s(c.0), s(c.1), s(c.2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shading() {
        assert_eq!(shade((100, 200, 250), 0.5), (50, 100, 125));
        assert_eq!(shade((100, 200, 250), 2.0), (200, 255, 255));
        assert_eq!(letter_rgb('W'), Some((250, 250, 250)));
        assert_eq!(letter_rgb('.'), None);
    }

    #[test]
    fn letters() {
        assert_eq!(from_letter('r').unwrap().color, Color::DarkRed);
        assert!(from_letter('R').unwrap().bold);
        assert_eq!(from_letter('W').unwrap().color, Color::White);
        assert_eq!(from_letter(' '), None);
        assert_eq!(from_letter('1'), None);
    }

    #[test]
    fn names() {
        assert_eq!(named("cyan").color, Color::DarkCyan);
        assert_eq!(named("CYAN").color, Color::Cyan);
        assert_eq!(named("BLACK").color, Color::DarkGrey);
        assert_eq!(named("blue").color, Color::DarkBlue);
    }
}
