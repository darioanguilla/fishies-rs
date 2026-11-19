//! Fish, their bubbles, and what happens when they meet sharks or hooks.

use super::{depth, rand_int};
use crate::engine::{Animation, Behavior, CollHandler, DeathAction, Entity, Id, Kind, Sprite};
use crate::{art, pixel_art};

const COLORS: [char; 12] = ['c', 'C', 'r', 'R', 'y', 'Y', 'b', 'B', 'g', 'G', 'm', 'M'];

/// A random color letter for each mask digit 1-9.
pub fn rand_palette() -> [char; 9] {
    std::array::from_fn(|_| COLORS[rand_int(COLORS.len() as f64) as usize])
}

/// Replace the digits 1-9 of a color mask with the palette's colors.
pub(super) fn apply_palette(mask: &str, palette: &[char; 9]) -> String {
    mask.chars()
        .map(|c| match c {
            '1'..='9' => palette[c as usize - '1' as usize],
            c => c,
        })
        .collect()
}

pub fn add_all_fish(anim: &mut Animation) {
    // figure out how many fish to add by the size of the screen,
    // minus the stuff above the water
    let screen_size = (anim.height - 9) * anim.width;
    for _ in 0..(screen_size / 350).max(0) {
        add_fish(anim);
    }
}

/// Build fish number `fish_num` (0..16); odd numbers swim to the left.
pub fn new_fish(fish_num: usize) -> Entity {
    // mask digits: 1 body, 2 dorsal fin, 3 flippers, 4 eye, 5 mouth, 6 tailfin, 7 gills
    let (image, mask) = art::FISH[fish_num];
    // the same colors in both modes
    let palette = rand_palette();
    let mask = apply_palette(&mask.replace('4', "W"), &palette);
    let mut speed = super::rand_f(2.0) + 0.25;
    if fish_num % 2 == 1 {
        speed = -speed;
    }
    let z = rand_int(depth::FISH_END - depth::FISH_START) + depth::FISH_START;
    let frames = Sprite::single(image).mask(&mask).auto_trans().build();
    Entity::new(Kind::Fish, frames)
        .pixels(vec![pixel_art::fish(fish_num, &palette)])
        .at(0.0, 0.0, z)
        .speed(speed, 0.0, 0.0, 0.0)
        .behavior(Behavior::Fish)
        .die_offscreen()
        .on_death(DeathAction::RespawnFish)
        .physical(1.0)
        .on_collision(CollHandler::Fish)
}

pub fn add_fish(anim: &mut Animation) {
    let fish_num = rand_int(art::FISH.len() as f64) as usize;
    let mut fish = new_fish(fish_num);
    let max_height = 9.0;
    let min_height = anim.height as f64 - fish.height();
    fish.y = rand_int(min_height - max_height) + max_height;
    fish.x = if fish_num % 2 == 1 {
        anim.width as f64 - 2.0
    } else {
        1.0 - fish.width()
    };
    anim.add(fish);
}

/// An air bubble coming out of `fish`'s mouth.
pub fn bubble(fish: &Entity) -> Entity {
    let (mut x, mut y, mut z) = (fish.x, fish.y, fish.z);
    if fish.dx() > 0.0 {
        x += fish.width();
    }
    y += (fish.height() / 2.0).trunc();
    // bubble always goes on top of the fish
    z -= 1.0;
    let frames = Sprite::new(&[".", "o", "O", "O", "O"])
        .color("CYAN")
        .build();
    Entity::new(Kind::Bubble, frames)
        .at(x, y, z)
        .speed(0.0, -1.0, 0.0, 0.1)
        .die_offscreen()
        .physical(1.0)
        .on_collision(CollHandler::Bubble)
}

pub fn fish_collision(anim: &mut Animation, fish: Id, hits: &[(Id, Kind)]) {
    for &(other, kind) in hits {
        match kind {
            Kind::Teeth => {
                if let Some(t) = anim.get_mut(other) {
                    let (x, y, z) = (t.x, t.y, t.z);
                    add_splat(anim, x, y, z);
                }
                anim.kill(fish);
                return;
            }
            Kind::HookPoint => {
                retract(anim, other);
                retract(anim, fish);
                // get the hook and line
                for k in [Kind::FishHook, Kind::FishLine] {
                    if let Some(&id) = anim.ids_of_kind(k).first() {
                        retract(anim, id);
                    }
                }
                return;
            }
            _ => {}
        }
    }
}

/// Pull the fishhook, line and whatever got caught back to the surface.
fn retract(anim: &mut Animation, id: Id) {
    if let Some(e) = anim.get_mut(id) {
        e.physical = false;
        if e.kind == Kind::Fish {
            e.z = depth::WATER_GAP[2];
        }
        e.behavior = Behavior::FishHook { hooked: true };
    }
}

fn add_splat(anim: &mut Animation, x: f64, y: f64, z: f64) {
    let frames = Sprite::new(&art::SPLAT)
        .color("RED")
        .transparent(' ')
        .build();
    let sizes: Vec<(usize, usize)> = frames.iter().map(|f| (f.width, f.height)).collect();
    anim.add(
        Entity::new(Kind::Other, frames)
            .pixels(pixel_art::extras::splat(&sizes))
            .at(x - 4.0, y - 2.0, z - 2.0)
            .speed(0.0, 0.0, 0.0, 0.25)
            .die_frame(15),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_replaces_all_digits() {
        let m = apply_palette("123456789 W", &rand_palette());
        assert!(!m.chars().any(|c| c.is_ascii_digit()));
        assert!(m.ends_with(" W"));
    }

    #[test]
    fn all_fish_build_with_matching_masks() {
        for n in 0..art::FISH.len() {
            let f = new_fish(n);
            assert!(f.width() > 0.0 && f.height() > 0.0);
            assert_eq!(f.dx() < 0.0, n % 2 == 1);
        }
    }

    #[test]
    fn shark_teeth_eat_fish() {
        let mut anim = Animation::new(80, 30);
        // a still fish, with no random bubbles
        let mut f = new_fish(0).speed(0.0, 0.0, 0.0, 0.0);
        (f.x, f.y, f.z) = (10.0, 10.0, 5.0);
        f.behavior = Behavior::Move;
        let fid = anim.add(f);
        let teeth = Entity::new(Kind::Teeth, Sprite::single("*").build())
            .at(11.0, 11.0, 3.0)
            .physical(17.0);
        anim.add(teeth);
        anim.flush();
        let before = anim.entities.len();
        super::super::tick(&mut anim);
        assert!(anim.get_mut(fid).is_none(), "fish should be eaten");
        // splat + respawned fish replace the eaten one
        assert_eq!(anim.entities.len(), before + 1);
    }
}
