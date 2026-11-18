//! Aquarium logic: what the Perl script does on top of Term::Animation.

mod environment;
mod fish;
mod random;

use std::sync::OnceLock;

use rand::Rng;

use crate::engine::{Animation, Behavior, CollHandler, DeathAction, Entity, Id, Kind};

pub use random::OBJECT_NAMES;

/// The Z depth at which certain items occur (lower = closer to the viewer).
pub mod depth {
    pub const SHARK: f64 = 2.0;
    pub const FISH_START: f64 = 3.0;
    pub const FISH_END: f64 = 20.0;
    pub const SEAWEED: f64 = 21.0;
    pub const CASTLE: f64 = 22.0;
    /// `water_line0..3`
    pub const WATER_LINE: [f64; 4] = [8.0, 6.0, 4.0, 2.0];
    /// `water_gap0..3`
    pub const WATER_GAP: [f64; 4] = [9.0, 7.0, 5.0, 3.0];
}

static FORCED_OBJECT: OnceLock<Option<usize>> = OnceLock::new();

/// Always spawn the named random object (debugging aid). Unknown names are
/// ignored.
pub fn force_object(name: Option<&str>) {
    let idx = name.and_then(|n| OBJECT_NAMES.iter().position(|o| o.eq_ignore_ascii_case(n)));
    let _ = FORCED_OBJECT.set(idx);
}

/// Perl `int(rand($n))`.
pub(crate) fn rand_int(n: f64) -> f64 {
    rand_f(n).trunc()
}

/// Perl `rand($n)`; like Perl, `rand(0)` behaves as `rand(1)`.
pub(crate) fn rand_f(n: f64) -> f64 {
    let n = if n == 0.0 { 1.0 } else { n };
    rand::thread_rng().gen_range(0.0..1.0) * n
}

/// Build the whole scene for the current screen size.
pub fn populate(anim: &mut Animation) {
    environment::add_environment(anim);
    environment::add_castle(anim);
    environment::add_all_seaweed(anim);
    fish::add_all_fish(anim);
    random_object(anim);
    anim.flush();
}

/// Add one of the random objects to the screen.
pub fn random_object(anim: &mut Animation) {
    let idx = FORCED_OBJECT
        .get()
        .copied()
        .flatten()
        .unwrap_or_else(|| rand_int(OBJECT_NAMES.len() as f64) as usize);
    random::spawn(idx, anim);
}

/// Advance the animation by one tick.
pub fn tick(anim: &mut Animation) {
    let (w, h) = (anim.width as f64, anim.height as f64);

    // callbacks / movement
    for i in 0..anim.entities.len() {
        let e = &mut anim.entities[i];
        e.age += 1;
        match e.behavior {
            Behavior::Move => e.apply_motion(),
            Behavior::Fish => {
                if rand_int(100.0) > 97.0 {
                    let bubble = fish::bubble(e);
                    anim.add(bubble);
                }
                anim.entities[i].apply_motion();
            }
            Behavior::FishHook { hooked } => {
                if hooked {
                    e.y -= 1.0;
                } else if e.y + e.height() < h * 0.75 {
                    e.y += 1.0;
                }
            }
        }
    }

    // collisions
    for (id, hits) in anim.collisions() {
        let Some(handler) = anim.get_mut(id).map(|e| e.on_collision) else {
            continue;
        };
        let kinds: Vec<(Id, Kind)> = hits
            .into_iter()
            .filter_map(|h| anim.kind_of(h).map(|k| (h, k)))
            .collect();
        match handler {
            CollHandler::None => {}
            CollHandler::Bubble => {
                if kinds.iter().any(|&(_, k)| k == Kind::Waterline) {
                    anim.kill(id);
                }
            }
            CollHandler::Fish => fish::fish_collision(anim, id, &kinds),
        }
    }

    // automatic deaths
    let now = std::time::Instant::now();
    for e in anim.entities.iter_mut().filter(|e| e.alive) {
        if (e.die_offscreen && e.is_offscreen(w, h))
            || e.die_time.is_some_and(|t| now >= t)
            || e.die_frame.is_some_and(|f| e.age >= f)
        {
            e.alive = false;
        }
    }

    // death callbacks (they may kill more entities, so repeat)
    loop {
        let dead = anim.take_dead();
        if dead.is_empty() {
            break;
        }
        for e in dead {
            on_death(anim, e);
        }
    }

    anim.flush();
}

fn on_death(anim: &mut Animation, e: Entity) {
    match e.on_death {
        DeathAction::None => {}
        DeathAction::RespawnSeaweed => environment::add_seaweed(anim),
        DeathAction::RespawnFish => fish::add_fish(anim),
        DeathAction::RandomObject => random_object(anim),
        DeathAction::GroupDeath(kinds) => {
            // when a shark (or the fishhook) dies, kill its bound entities too
            for &k in kinds {
                for id in anim.ids_of_kind(k) {
                    anim.kill(id);
                }
            }
            random_object(anim);
        }
        DeathAction::LeadDolphin(others) => {
            // the followers started offscreen, so they only die offscreen now
            for id in others {
                if let Some(d) = anim.get_mut(id) {
                    d.die_offscreen = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Step;

    #[test]
    fn runs_many_ticks_without_panicking() {
        for (w, h) in [(120, 40), (80, 24), (20, 10), (1, 1)] {
            let mut anim = Animation::new(w, h);
            populate(&mut anim);
            let mut buf = crate::engine::Buffer::new(w, h);
            for _ in 0..3000 {
                tick(&mut anim);
                anim.render(&mut buf);
            }
        }
    }

    #[test]
    fn every_random_object_spawns_and_eventually_respawns() {
        for (idx, name) in OBJECT_NAMES.iter().enumerate() {
            let mut anim = Animation::new(100, 40);
            random::spawn(idx, &mut anim);
            anim.flush();
            let first: Vec<Id> = anim.entities.iter().map(|e| e.id).collect();
            assert!(!first.is_empty(), "{name} spawned nothing");
            if *name == "fishhook" {
                continue; // only leaves when it catches a fish
            }
            let mut replaced = false;
            for _ in 0..2000 {
                tick(&mut anim);
                if anim.entities.iter().all(|e| !first.contains(&e.id)) && !anim.entities.is_empty()
                {
                    replaced = true;
                    break;
                }
            }
            assert!(replaced, "{name} never left the screen");
        }
    }

    #[test]
    fn hooked_fish_reels_everything_up() {
        let mut anim = Animation::new(100, 40);
        random::spawn(
            OBJECT_NAMES.iter().position(|&n| n == "fishhook").unwrap(),
            &mut anim,
        );
        anim.flush();
        let point = anim.ids_of_kind(Kind::HookPoint)[0];
        let gear: Vec<Id> = anim.entities.iter().map(|e| e.id).collect();
        // drop a fish right where the hook point will be
        for _ in 0..60 {
            tick(&mut anim);
        }
        let (px, py, pz) = {
            let p = anim.get_mut(point).unwrap();
            (p.x, p.y, p.z)
        };
        let mut f = fish::new_fish(0);
        (f.x, f.y, f.z) = (px - 1.0, py - 1.0, pz);
        f.motion = crate::engine::entity::Motion::Linear(Step::new(0.0, 0.0, 0.0, 0.0));
        let fid = anim.add(f);
        anim.flush();
        tick(&mut anim);
        assert!(matches!(
            anim.get_mut(fid).unwrap().behavior,
            Behavior::FishHook { hooked: true }
        ));
        for _ in 0..200 {
            tick(&mut anim);
        }
        // hook, line and point are gone (a new random object may have spawned)
        for id in gear {
            assert!(anim.get_mut(id).is_none(), "fishing gear {id} left behind");
        }
    }
}
