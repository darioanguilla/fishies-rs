//! A minimal replacement for the parts of Perl's Term::Animation used by the
//! aquarium: an entity store, collision detection and z-ordered rendering.

pub mod entity;
pub mod pixel;
pub mod render;
pub mod shape;

pub use entity::{Behavior, CollHandler, DeathAction, Entity, Id, Kind, Step};
pub use pixel::PixFrame;
pub use render::{Buffer, Mode};
pub use shape::Sprite;

use crate::color::{self, Style, shade};
use render::Glyph;
pub struct Animation {
    pub width: i32,
    pub height: i32,
    pub entities: Vec<Entity>,
    pending: Vec<Entity>,
    next_id: Id,
}

impl Animation {
    pub fn new(width: i32, height: i32) -> Self {
        Animation {
            width,
            height,
            entities: Vec::new(),
            pending: Vec::new(),
            next_id: 1,
        }
    }

    /// Queue an entity; it becomes active at the next [`Animation::flush`].
    pub fn add(&mut self, mut e: Entity) -> Id {
        e.id = self.next_id;
        self.next_id += 1;
        let id = e.id;
        self.pending.push(e);
        id
    }

    /// Move queued entities into the active set.
    pub fn flush(&mut self) {
        self.entities.append(&mut self.pending);
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Entity> {
        self.entities
            .iter_mut()
            .chain(self.pending.iter_mut())
            .find(|e| e.id == id)
    }

    pub fn ids_of_kind(&self, kind: Kind) -> Vec<Id> {
        self.entities
            .iter()
            .chain(self.pending.iter())
            .filter(|e| e.kind == kind)
            .map(|e| e.id)
            .collect()
    }

    pub fn kill(&mut self, id: Id) {
        if let Some(e) = self.get_mut(id) {
            e.alive = false;
        }
    }

    /// Remove and return all dead entities.
    pub fn take_dead(&mut self) -> Vec<Entity> {
        let (dead, alive) = std::mem::take(&mut self.entities)
            .into_iter()
            .partition(|e| !e.alive);
        self.entities = alive;
        dead
    }

    /// For each living physical entity, the ids of the living physical
    /// entities it collides with.
    pub fn collisions(&self) -> Vec<(Id, Vec<Id>)> {
        let phys: Vec<&Entity> = self
            .entities
            .iter()
            .filter(|e| e.physical && e.alive)
            .collect();
        phys.iter()
            .filter_map(|a| {
                let hits: Vec<Id> = phys
                    .iter()
                    .filter(|b| b.id != a.id && a.collides_with(b))
                    .map(|b| b.id)
                    .collect();
                (!hits.is_empty()).then_some((a.id, hits))
            })
            .collect()
    }

    pub fn kind_of(&self, id: Id) -> Option<Kind> {
        self.entities.iter().find(|e| e.id == id).map(|e| e.kind)
    }

    /// Draw all entities, back (high z) to front (low z).
    pub fn render(&self, buf: &mut Buffer, mode: Mode) {
        buf.clear();
        let mut order: Vec<&Entity> = self.entities.iter().filter(|e| e.alive).collect();
        order.sort_by(|a, b| b.z.total_cmp(&a.z));
        for e in order {
            let (x, y) = (e.x.trunc() as i32, e.y.trunc() as i32);
            match (mode, e.pixel_frame()) {
                (Mode::Ascii, _) => buf.blit(e.frame(), x, y, ascii_glyph),
                (Mode::Ansi, Some(px)) => buf.blit_pixels(px, x, y),
                (Mode::Ansi, None) => buf.blit(e.frame(), x, y, |ch, st, row, h| {
                    ansi_glyph(e.kind, ch, st, row, h)
                }),
            }
        }
    }
}

fn ascii_glyph(ch: char, st: Style, _row: usize, _height: usize) -> Glyph {
    Glyph {
        ch,
        fg: st.color,
        bold: st.bold,
    }
}

/// ANSI mode for sprites without pixel art: truecolor with a soft vertical
/// light, and a few Unicode replacements.
fn ansi_glyph(kind: Kind, ch: char, st: Style, row: usize, height: usize) -> Glyph {
    let ch = match (kind, ch) {
        (Kind::Bubble, '.') => '·',
        (Kind::Bubble, 'o') => '◦',
        (Kind::Bubble, 'O') => '○',
        (Kind::FishLine, '|') => '│',
        (_, c) => c,
    };
    let t = if height > 1 {
        row as f32 / (height - 1) as f32
    } else {
        0.0
    };
    // seaweed is lit from the surface, everything else gets a gentle gradient
    let (top, bottom) = if kind == Kind::Seaweed {
        (1.35, 0.75)
    } else {
        (1.12, 0.85)
    };
    Glyph {
        ch,
        fg: color::rgb(shade(color::ansi_rgb(st.color), top + (bottom - top) * t)),
        bold: st.bold,
    }
}
