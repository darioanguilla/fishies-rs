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

#[cfg(test)]
mod tests {
    use super::*;

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
