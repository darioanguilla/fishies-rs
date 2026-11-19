//! Animated entities (the Rust counterpart of Term::Animation::Entity).

use std::rc::Rc;
use std::time::Instant;

use super::pixel::PixFrame;
use super::shape::Frame;

/// High-resolution pixels per cell column (and per half cell row).
pub const HIRES: usize = 4;

pub type Id = u64;

/// Entity "type", used by collision handlers and group deaths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Other,
    Seaweed,
    Fish,
    Bubble,
    Waterline,
    Teeth,
    Shark,
    FishHook,
    FishLine,
    HookPoint,
}

/// One step of a movement path: `(dx, dy, dz, frame increment)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Step {
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
    pub frame: f64,
}

impl Step {
    pub const fn new(dx: f64, dy: f64, dz: f64, frame: f64) -> Self {
        Step { dx, dy, dz, frame }
    }
}

/// Default movement (Term::Animation `move_entity`).
#[derive(Clone, Debug)]
pub enum Motion {
    /// Constant speed per tick.
    Linear(Step),
    /// Follow a list of steps, wrapping around.
    Path { index: usize, steps: Rc<[Step]> },
}

/// What the entity does each tick (the Perl `callback`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Behavior {
    /// Just apply [`Motion`].
    Move,
    /// Apply [`Motion`] and sometimes emit a bubble.
    Fish,
    /// Lower to 3/4 of the screen, or reel up once something is hooked.
    FishHook { hooked: bool },
}

/// What happens when the entity dies (the Perl `death_cb`).
#[derive(Clone, Debug, PartialEq)]
pub enum DeathAction {
    None,
    RespawnSeaweed,
    RespawnFish,
    RandomObject,
    /// Kill every entity of these kinds, then add a random object.
    GroupDeath(&'static [Kind]),
    /// The lead dolphin tells the other two to die offscreen.
    LeadDolphin([Id; 2]),
}

/// Reaction to collisions (the Perl `coll_handler`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollHandler {
    None,
    Bubble,
    Fish,
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub id: Id,
    pub kind: Kind,
    pub frames: Vec<Frame>,
    /// Pixel-art frames for ANSI mode; empty means "draw the ASCII frames".
    pub pixels: Vec<PixFrame>,
    /// Finer pixel art for graphics mode, [`HIRES`] pixels per cell column.
    pub hires: Vec<PixFrame>,
    /// Pixel frames advance this many times faster than the ASCII frames,
    /// for smoother animation with more pixel frames.
    pub pixel_rate: f64,
    pub frame_pos: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub motion: Motion,
    pub behavior: Behavior,
    pub physical: bool,
    /// Number of z layers this entity spans for collision purposes.
    pub coll_depth: f64,
    pub die_offscreen: bool,
    pub die_time: Option<Instant>,
    /// Die after this many ticks.
    pub die_frame: Option<u32>,
    pub age: u32,
    pub on_death: DeathAction,
    pub on_collision: CollHandler,
    pub alive: bool,
}

impl Entity {
    pub fn new(kind: Kind, frames: Vec<Frame>) -> Self {
        assert!(!frames.is_empty(), "entity needs at least one frame");
        Entity {
            id: 0,
            kind,
            frames,
            pixels: Vec::new(),
            hires: Vec::new(),
            pixel_rate: 1.0,
            frame_pos: 0.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            motion: Motion::Linear(Step::new(0.0, 0.0, 0.0, 0.0)),
            behavior: Behavior::Move,
            physical: false,
            coll_depth: 1.0,
            die_offscreen: false,
            die_time: None,
            die_frame: None,
            age: 0,
            on_death: DeathAction::None,
            on_collision: CollHandler::None,
            alive: true,
        }
    }

    pub fn at(mut self, x: f64, y: f64, z: f64) -> Self {
        (self.x, self.y, self.z) = (x, y, z);
        self
    }

    /// Constant movement, like Perl `callback_args => [dx, dy, dz, frame]`.
    pub fn speed(mut self, dx: f64, dy: f64, dz: f64, frame: f64) -> Self {
        self.motion = Motion::Linear(Step::new(dx, dy, dz, frame));
        self
    }

    pub fn path(mut self, index: usize, steps: Rc<[Step]>) -> Self {
        self.motion = Motion::Path { index, steps };
        self
    }

    pub fn behavior(mut self, b: Behavior) -> Self {
        self.behavior = b;
        self
    }

    pub fn physical(mut self, coll_depth: f64) -> Self {
        self.physical = true;
        self.coll_depth = coll_depth;
        self
    }

    pub fn die_offscreen(mut self) -> Self {
        self.die_offscreen = true;
        self
    }

    pub fn die_time(mut self, t: Instant) -> Self {
        self.die_time = Some(t);
        self
    }

    pub fn die_frame(mut self, n: u32) -> Self {
        self.die_frame = Some(n);
        self
    }

    pub fn on_death(mut self, a: DeathAction) -> Self {
        self.on_death = a;
        self
    }

    pub fn on_collision(mut self, h: CollHandler) -> Self {
        self.on_collision = h;
        self
    }

    /// Pixel art for ANSI mode. Each frame must be as wide as the ASCII
    /// frames and twice as tall, so the footprint (and gameplay) is the same.
    pub fn pixels(mut self, frames: Vec<PixFrame>) -> Self {
        self.pixels = frames;
        self
    }

    pub fn frame(&self) -> &Frame {
        let i = self.frame_pos.max(0.0) as usize % self.frames.len();
        &self.frames[i]
    }

    /// Finer pixel art for graphics mode: [`HIRES`] times as many pixels in
    /// each direction as [`Entity::pixels`].
    pub fn hires(mut self, frames: Vec<PixFrame>) -> Self {
        self.hires = frames;
        self
    }

    /// Make the pixel frames advance `rate` times faster than the ASCII ones.
    pub fn pixel_rate(mut self, rate: f64) -> Self {
        self.pixel_rate = rate;
        self
    }

    fn pick<'a>(&self, frames: &'a [PixFrame]) -> Option<&'a PixFrame> {
        if frames.is_empty() {
            return None;
        }
        let i = (self.frame_pos.max(0.0) * self.pixel_rate) as usize % frames.len();
        Some(&frames[i])
    }

    /// Current pixel frame, if the entity has pixel art.
    pub fn pixel_frame(&self) -> Option<&PixFrame> {
        self.pick(&self.pixels)
    }

    /// Current high-resolution frame, if the entity has one.
    pub fn hires_frame(&self) -> Option<&PixFrame> {
        self.pick(&self.hires)
    }

    pub fn width(&self) -> f64 {
        self.frame().width as f64
    }

    pub fn height(&self) -> f64 {
        self.frame().height as f64
    }

    /// Horizontal speed (first element of Perl `callback_args`).
    pub fn dx(&self) -> f64 {
        match &self.motion {
            Motion::Linear(s) => s.dx,
            Motion::Path { index, steps } => steps.get(*index).map_or(0.0, |s| s.dx),
        }
    }

    /// Apply the default movement for one tick.
    pub fn apply_motion(&mut self) {
        let step = match &mut self.motion {
            Motion::Linear(s) => *s,
            Motion::Path { index, steps } => {
                if steps.is_empty() {
                    return;
                }
                let s = steps[*index % steps.len()];
                *index = (*index + 1) % steps.len();
                s
            }
        };
        self.x += step.dx;
        self.y += step.dy;
        self.z += step.dz;
        self.frame_pos += step.frame;
    }

    /// Completely outside a `width` x `height` screen.
    pub fn is_offscreen(&self, width: f64, height: f64) -> bool {
        self.x < -self.width() || self.y < -self.height() || self.x >= width || self.y >= height
    }

    /// Bounding boxes and z ranges overlap.
    pub fn collides_with(&self, o: &Entity) -> bool {
        let (ax, ay) = (self.x.trunc(), self.y.trunc());
        let (bx, by) = (o.x.trunc(), o.y.trunc());
        ax < bx + o.width()
            && bx < ax + self.width()
            && ay < by + o.height()
            && by < ay + self.height()
            && self.z < o.z + o.coll_depth
            && o.z < self.z + self.coll_depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::shape::Sprite;

    fn block(w: usize, h: usize) -> Entity {
        let line = "#".repeat(w);
        let shape = vec![line; h].join("\n");
        Entity::new(Kind::Other, Sprite::single(&shape).build())
    }

    #[test]
    fn path_wraps() {
        let steps: Rc<[Step]> =
            vec![Step::new(1.0, 0.0, 0.0, 0.0), Step::new(0.0, 1.0, 0.0, 0.0)].into();
        let mut e = block(1, 1).path(1, steps);
        e.apply_motion();
        e.apply_motion();
        e.apply_motion();
        assert_eq!((e.x, e.y), (1.0, 2.0));
    }

    #[test]
    fn frames_advance_and_wrap() {
        let mut e =
            Entity::new(Kind::Other, Sprite::new(&["a", "b"]).build()).speed(0.0, 0.0, 0.0, 0.5);
        let seen: String = (0..5)
            .map(|_| {
                let c = e.frame().get(0, 0).unwrap().0;
                e.apply_motion();
                c
            })
            .collect();
        assert_eq!(seen, "aabba");
    }

    #[test]
    fn offscreen() {
        let e = block(3, 2).at(-3.0, 0.0, 0.0);
        assert!(!e.is_offscreen(10.0, 10.0));
        assert!(e.clone().at(-3.5, 0.0, 0.0).is_offscreen(10.0, 10.0));
        assert!(e.clone().at(10.0, 0.0, 0.0).is_offscreen(10.0, 10.0));
        assert!(e.at(0.0, -2.5, 0.0).is_offscreen(10.0, 10.0));
    }

    #[test]
    fn collision_needs_overlap_in_space_and_depth() {
        let a = block(2, 2).at(0.0, 0.0, 3.0);
        let teeth = block(1, 1).at(1.0, 1.0, 3.0).physical(17.0);
        assert!(a.collides_with(&teeth) && teeth.collides_with(&a));
        let deep = a.clone().at(0.0, 0.0, 19.0);
        assert!(deep.collides_with(&teeth));
        let behind = a.clone().at(0.0, 0.0, 20.0);
        assert!(!behind.collides_with(&teeth));
        let apart = a.at(2.0, 0.0, 3.0);
        assert!(!apart.collides_with(&teeth));
    }
}
