//! Conversion of ASCII art + color masks into drawable frames, following the
//! conventions of Perl's Term::Animation.

use crate::color::{self, Style};

/// A single drawable cell; `None` is transparent.
pub type Cell = Option<(char, Style)>;

#[derive(Clone, Debug)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

impl Frame {
    pub fn get(&self, col: usize, row: usize) -> Cell {
        self.cells[row * self.width + col]
    }
}

/// Apply Perl single-quote (`q{}`) semantics: a doubled backslash becomes one.
/// Carriage returns are dropped, in case the sources were checked out with
/// CRLF line endings.
pub fn perl_q(s: &str) -> String {
    s.replace('\r', "").replace("\\\\", "\\")
}

/// Split like Perl's `split("\n", ...)`: trailing empty fields are dropped.
pub fn perl_lines(s: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = s.split('\n').collect();
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines
}

/// Remove the first newline, as Term::Animation does, then split into lines.
fn shape_lines(s: &str) -> Vec<Vec<char>> {
    let s = s.strip_prefix('\n').unwrap_or(s);
    perl_lines(s).iter().map(|l| l.chars().collect()).collect()
}

/// Builder for the frames of an entity.
pub struct Sprite {
    shapes: Vec<String>,
    masks: Vec<String>,
    default: Style,
    auto_trans: bool,
    transparent: char,
}

impl Sprite {
    /// Shapes are given exactly as in the Perl source (they are passed
    /// through [`perl_q`]).
    pub fn new<S: AsRef<str>>(shapes: &[S]) -> Self {
        Sprite {
            shapes: shapes.iter().map(|s| perl_q(s.as_ref())).collect(),
            masks: Vec::new(),
            default: Style::WHITE,
            auto_trans: false,
            transparent: '?',
        }
    }

    pub fn single(shape: &str) -> Self {
        Self::new(&[shape])
    }

    /// One mask for every frame, or one mask per frame.
    pub fn masks<S: AsRef<str>>(mut self, masks: &[S]) -> Self {
        self.masks = masks.iter().map(|s| s.as_ref().to_string()).collect();
        self
    }

    pub fn mask(self, mask: &str) -> Self {
        self.masks(&[mask])
    }

    pub fn color(mut self, name: &str) -> Self {
        self.default = color::named(name);
        self
    }

    /// Make leading whitespace of every line transparent.
    pub fn auto_trans(mut self) -> Self {
        self.auto_trans = true;
        self
    }

    pub fn transparent(mut self, c: char) -> Self {
        self.transparent = c;
        self
    }

    pub fn build(self) -> Vec<Frame> {
        self.shapes
            .iter()
            .enumerate()
            .map(|(i, shape)| {
                let mask = match self.masks.len() {
                    0 => None,
                    1 => Some(&self.masks[0]),
                    _ => self.masks.get(i),
                };
                self.build_frame(shape, mask.map(String::as_str))
            })
            .collect()
    }

    fn build_frame(&self, shape: &str, mask: Option<&str>) -> Frame {
        let lines = shape_lines(shape);
        let mask_lines = mask.map(shape_lines).unwrap_or_default();
        let width = lines.iter().map(Vec::len).max().unwrap_or(0);
        let height = lines.len();
        let mut cells = vec![None; width * height];
        for (row, line) in lines.iter().enumerate() {
            let lead = if self.auto_trans {
                line.iter().take_while(|&&c| c == ' ').count()
            } else {
                0
            };
            for (col, &ch) in line.iter().enumerate() {
                if col < lead || ch == self.transparent {
                    continue;
                }
                let style = mask_lines
                    .get(row)
                    .and_then(|m| m.get(col))
                    .and_then(|&m| color::from_letter(m))
                    .unwrap_or(self.default);
                cells[row * width + col] = Some((ch, style));
            }
        }
        Frame {
            width,
            height,
            cells,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unescape_backslashes() {
        assert_eq!(perl_q(r",;\\\\\,,"), r",;\\\,,");
        assert_eq!(perl_q(r"''\\\\\\\\\'' ';\"), r"''\\\\\'' ';\");
        assert_eq!(perl_q(r"`'\'"), r"`'\'");
    }

    #[test]
    fn crlf_sources_are_handled() {
        let f = &Sprite::single("\r\nab\r\nc\r\n").build()[0];
        assert_eq!((f.width, f.height), (2, 2));
    }

    #[test]
    fn perl_split() {
        assert_eq!(perl_lines("a\n\nb\n\n"), vec!["a", "", "b"]);
        assert_eq!(perl_lines("\n\n:"), vec!["", "", ":"]);
    }

    #[test]
    fn strips_first_newline_and_measures() {
        let f = &Sprite::single("\nab\nc\n").build()[0];
        assert_eq!((f.width, f.height), (2, 2));
        assert_eq!(f.get(1, 1), None);
        assert_eq!(f.get(0, 1).unwrap().0, 'c');
    }

    #[test]
    fn question_mark_is_transparent_and_spaces_opaque() {
        let f = &Sprite::single("a? b").build()[0];
        assert!(f.get(1, 0).is_none());
        assert_eq!(f.get(2, 0).unwrap().0, ' ');
    }

    #[test]
    fn auto_trans_only_leading_spaces() {
        let f = &Sprite::single("  a b").auto_trans().build()[0];
        assert!(f.get(0, 0).is_none() && f.get(1, 0).is_none());
        assert_eq!(f.get(3, 0).unwrap().0, ' ');
    }

    #[test]
    fn transparent_space() {
        let f = &Sprite::single("* *").transparent(' ').build()[0];
        assert!(f.get(1, 0).is_none());
    }

    #[test]
    fn mask_colors_and_default() {
        let f = &Sprite::single("abc").mask("R w").color("green").build()[0];
        assert_eq!(f.get(0, 0).unwrap().1, color::from_letter('R').unwrap());
        assert_eq!(f.get(1, 0).unwrap().1, color::named("green"));
        assert_eq!(f.get(2, 0).unwrap().1, color::from_letter('w').unwrap());
    }
}
