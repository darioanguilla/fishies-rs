//! A minimal replacement for the parts of Perl's Term::Animation used by the
//! aquarium: an entity store, collision detection and z-ordered rendering.

pub mod entity;
pub mod render;
pub mod shape;

pub use entity::{Behavior, CollHandler, DeathAction, Entity, Id, Kind, Step};
pub use render::Buffer;
pub use shape::Sprite;

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
    pub fn render(&self, buf: &mut Buffer) {
        buf.clear();
        let mut order: Vec<&Entity> = self.entities.iter().filter(|e| e.alive).collect();
        order.sort_by(|a, b| b.z.total_cmp(&a.z));
        for e in order {
            buf.blit(e.frame(), e.x.trunc() as i32, e.y.trunc() as i32);
        }
    }
}
