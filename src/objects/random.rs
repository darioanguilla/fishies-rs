//! The "random objects": one of them is always on screen, and when it leaves
//! another one is picked.

use std::rc::Rc;

use super::Art;
use super::fish::{apply_palette, rand_palette};
use super::{depth, rand_int};
use crate::art;
use crate::color;
use crate::engine::shape::perl_lines;
use crate::engine::{Animation, Behavior, DeathAction, Entity, Kind, Sprite, Step};
use crate::pixel_art::creatures::{self, MONSTER_FRAMES};
use crate::pixel_art::{extras, facing};

pub const OBJECT_NAMES: [&str; 9] = [
    "ship", "whale", "monster", "big_fish", "shark", "fishhook", "swan", "ducks", "dolphins",
];

pub fn spawn(idx: usize, anim: &mut Animation) {
    match idx {
        0 => add_ship(anim),
        1 => add_whale(anim),
        2 => add_monster(anim),
        3 => add_big_fish(anim),
        4 => add_shark(anim),
        5 => add_fishhook(anim),
        6 => add_swan(anim),
        7 => add_ducks(anim),
        _ => add_dolphins(anim),
    }
}

/// Random direction: `(dir, speed, x)`. `dir == 1` moves right to left,
/// starting at `width - 2`; otherwise it starts at `left_x`.
fn direction(anim: &Animation, speed: f64, left_x: f64) -> (usize, f64, f64) {
    let dir = rand_int(2.0) as usize;
    if dir == 1 {
        (dir, -speed, anim.width as f64 - 2.0)
    } else {
        (dir, speed, left_x)
    }
}

fn add_shark(anim: &mut Animation) {
    let (dir, speed, x) = direction(anim, 2.0, -53.0);
    let y = rand_int(anim.height as f64 - (10.0 + 9.0)) + 9.0;
    let teeth_x = if dir == 1 { x + 9.0 } else { -9.0 };
    let teeth_y = y + 7.0;

    // the "teeth" is the entity that actually collides with fish
    anim.add(
        Entity::new(Kind::Teeth, Sprite::single("*").build())
            .pixels(vec![extras::invisible(1, 1)])
            .at(teeth_x, teeth_y, depth::SHARK + 1.0)
            .physical(depth::FISH_END - depth::FISH_START)
            .speed(speed, 0.0, 0.0, 0.0),
    );

    let frames = Sprite::single(art::SHARK[dir])
        .mask(art::SHARK_MASK[dir])
        .auto_trans()
        .color("CYAN")
        .build();
    anim.add(
        Entity::new(Kind::Shark, frames)
            .art(|r| vec![facing(creatures::shark(r), dir)])
            .at(x, y, depth::SHARK)
            .speed(speed, 0.0, 0.0, 0.0)
            .die_offscreen()
            .on_death(DeathAction::GroupDeath(&[Kind::Teeth])),
    );
}

fn add_fishhook(anim: &mut Animation) {
    let x = 10.0 + rand_int(anim.width as f64 - 20.0);
    let y = -4.0;
    let line_image = "|\n".repeat(50) + &" \n".repeat(6);
    let hook = Behavior::FishHook { hooked: false };

    anim.add(
        Entity::new(
            Kind::FishLine,
            Sprite::single(&line_image).auto_trans().build(),
        )
        .at(x + 7.0, y - 50.0, depth::WATER_LINE[1])
        .behavior(hook),
    );

    let frames = Sprite::single(art::FISHHOOK)
        .auto_trans()
        .color("GREEN")
        .build();
    anim.add(
        Entity::new(Kind::FishHook, frames)
            .art(|r| vec![extras::fishhook(r)])
            .at(x, y, depth::WATER_LINE[1])
            .behavior(hook)
            .die_offscreen()
            // the Perl original forgot the hook point, leaving it behind
            .on_death(DeathAction::GroupDeath(&[
                Kind::Teeth,
                Kind::FishLine,
                Kind::HookPoint,
            ])),
    );

    let frames = Sprite::single(art::HOOK_POINT).color("GREEN").build();
    anim.add(
        Entity::new(Kind::HookPoint, frames)
            .pixels(vec![extras::invisible(1, 3)])
            .at(x + 1.0, y + 2.0, depth::SHARK + 1.0)
            .physical(depth::FISH_END - depth::FISH_START)
            .behavior(hook),
    );
}

fn add_ship(anim: &mut Animation) {
    let (dir, speed, x) = direction(anim, 1.0, -24.0);
    let frames = Sprite::single(art::SHIP[dir])
        .mask(art::SHIP_MASK[dir])
        .auto_trans()
        .color("WHITE")
        .build();
    anim.add(
        Entity::new(Kind::Other, frames)
            .art(|r| vec![facing(creatures::ship(r), dir)])
            .at(x, 0.0, depth::WATER_GAP[1])
            .speed(speed, 0.0, 0.0, 0.0)
            .die_offscreen()
            .on_death(DeathAction::RandomObject),
    );
}

fn add_whale(anim: &mut Animation) {
    let (dir, speed, x) = direction(anim, 1.0, -18.0);
    let spout_align = if dir == 1 { 1 } else { 11 };
    let whale = art::WHALE[dir];

    // no water spout
    let mut frames: Vec<String> = (0..5).map(|_| format!("\n\n\n{whale}")).collect();
    // animate water spout
    let sep = format!("\n{}", " ".repeat(spout_align));
    for spout in art::WATER_SPOUT {
        frames.push(perl_lines(spout).join(&sep) + whale);
    }
    let masks = vec![art::WHALE_MASK[dir]; frames.len()];

    let frames = Sprite::new(&frames)
        .masks(&masks)
        .auto_trans()
        .color("WHITE")
        .build();
    anim.add(
        Entity::new(Kind::Other, frames)
            .art(|r| {
                creatures::whale(r)
                    .into_iter()
                    .map(|f| facing(f, dir))
                    .collect()
            })
            .at(x, 0.0, depth::WATER_GAP[2])
            .speed(speed, 0.0, 0.0, 1.0)
            .die_offscreen()
            .on_death(DeathAction::RandomObject),
    );
}

fn add_monster(anim: &mut Animation) {
    let (dir, speed, x) = direction(anim, 2.0, -64.0);
    let frames = Sprite::new(&art::MONSTER[dir])
        .mask(art::MONSTER_MASK[dir])
        .auto_trans()
        .color("GREEN")
        .build();
    anim.add(
        Entity::new(Kind::Other, frames)
            .art(|r| {
                (0..MONSTER_FRAMES)
                    .map(|k| facing(creatures::monster(k, r), dir))
                    .collect()
            })
            .pixel_rate((MONSTER_FRAMES / 4) as f64)
            .at(x, 2.0, depth::WATER_GAP[2])
            .speed(speed, 0.0, 0.0, 0.25)
            .die_offscreen()
            .on_death(DeathAction::RandomObject),
    );
}

fn add_big_fish(anim: &mut Animation) {
    let dir = rand_int(2.0) as usize;
    let (speed, x) = if dir == 1 {
        (-3.0, anim.width as f64 - 1.0)
    } else {
        (3.0, -34.0)
    };
    let max_height = 9.0;
    let min_height = anim.height as f64 - 15.0;
    let y = rand_int(min_height - max_height) + max_height;
    let palette = rand_palette();
    let mask = apply_palette(art::BIG_FISH_MASK[dir], &palette);
    let frames = Sprite::single(art::BIG_FISH[dir])
        .mask(&mask)
        .auto_trans()
        .color("YELLOW")
        .build();
    anim.add(
        Entity::new(Kind::Other, frames)
            .art(|r| {
                vec![facing(
                    creatures::big_fish(&palette, art::BIG_FISH_MASK[0], r),
                    dir,
                )]
            })
            .at(x, y, depth::SHARK)
            .speed(speed, 0.0, 0.0, 0.0)
            .die_offscreen()
            .on_death(DeathAction::RandomObject),
    );
}

fn add_ducks(anim: &mut Animation) {
    let (dir, speed, x) = direction(anim, 1.0, -30.0);
    let frames = Sprite::new(&art::DUCKS[dir])
        .mask(art::DUCKS_MASK[dir])
        .auto_trans()
        .color("WHITE")
        .build();
    anim.add(
        Entity::new(Kind::Other, frames)
            .art(|r| {
                (0..3)
                    .map(|k| facing(creatures::ducks(k, r), dir))
                    .collect()
            })
            .at(x, 5.0, depth::WATER_GAP[3])
            .speed(speed, 0.0, 0.0, 0.25)
            .die_offscreen()
            .on_death(DeathAction::RandomObject),
    );
}

fn add_dolphins(anim: &mut Animation) {
    let (dir, speed, x) = direction(anim, 1.0, -13.0);
    // how far apart the dolphins are
    let distance = if dir == 1 { -15.0 } else { 15.0 };

    let up = Step::new(speed, -0.5, 0.0, 0.5);
    let down = Step::new(speed, 0.5, 0.0, 0.5);
    let glide = Step::new(speed, 0.0, 0.0, 0.5);
    let path: Rc<[Step]> = std::iter::repeat_n(up, 14)
        .chain(std::iter::repeat_n(glide, 2))
        .chain(std::iter::repeat_n(down, 14))
        .chain(std::iter::repeat_n(glide, 6))
        .collect();

    let dolphin = |color: &str| {
        let frames = Sprite::new(&art::DOLPHIN[dir])
            .mask(art::DOLPHIN_MASK[dir])
            .auto_trans()
            .color(color)
            .build();
        let base = color::ansi_rgb(color::named(color).color);
        Entity::new(Kind::Other, frames).art(|r| {
            creatures::dolphin(base, r)
                .into_iter()
                .map(|f| facing(f, dir))
                .collect()
        })
    };

    let d3 = anim.add(
        dolphin("blue")
            .at(x - distance * 2.0, 8.0, depth::WATER_GAP[3])
            .path(0, path.clone())
            .on_death(DeathAction::RandomObject),
    );
    let d2 = anim.add(
        dolphin("BLUE")
            .at(x - distance, 2.0, depth::WATER_GAP[3])
            .path(12, path.clone()),
    );
    // the lead dolphin tells the others to die offscreen, since they start offscreen
    anim.add(
        dolphin("CYAN")
            .at(x, 5.0, depth::WATER_GAP[3])
            .path(24, path)
            .die_offscreen()
            .on_death(DeathAction::LeadDolphin([d2, d3])),
    );
}

fn add_swan(anim: &mut Animation) {
    let (dir, speed, x) = direction(anim, 1.0, -10.0);
    let frames = Sprite::new(&[art::SWAN[dir]])
        .mask(art::SWAN_MASK[dir])
        .auto_trans()
        .color("WHITE")
        .build();
    anim.add(
        Entity::new(Kind::Other, frames)
            .art(|r| vec![facing(creatures::swan(r), dir)])
            .at(x, 1.0, depth::WATER_GAP[3])
            .speed(speed, 0.0, 0.0, 0.25)
            .die_offscreen()
            .on_death(DeathAction::RandomObject),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whale_frames_have_same_height() {
        let mut anim = Animation::new(100, 40);
        add_whale(&mut anim);
        anim.flush();
        let e = &anim.entities[0];
        assert_eq!(e.frames.len(), 12);
        assert!(e.frames.iter().all(|f| f.height == 7));
        // the spout is drawn above the whale in the last frames
        let spout = &e.frames[5];
        assert!((0..spout.width).any(|c| spout.get(c, 2).is_some_and(|(ch, _)| ch == ':')));
    }
}
