//! Water surface, castle and seaweed.

use std::time::{Duration, Instant};

use super::{depth, rand_f, rand_int};
use crate::engine::{Animation, DeathAction, Entity, Kind, Sprite};
use crate::{art, pixel_art};

pub fn add_environment(anim: &mut Animation) {
    // tile the segments so they stretch across the screen
    let segment_size = art::WATER_SEGMENTS[0].len() as i32;
    let repeat = (anim.width / segment_size).max(0) as usize + 1;
    for (i, seg) in art::WATER_SEGMENTS.iter().enumerate() {
        let line = seg.repeat(repeat);
        let frames = Sprite::single(&line).color("cyan").build();
        anim.add(
            Entity::new(Kind::Waterline, frames)
                .at(0.0, i as f64 + 5.0, depth::WATER_LINE[i])
                .physical(22.0)
                // the pixel waves roll; the ASCII line has a single frame
                .speed(0.0, 0.0, 0.0, 0.25)
                .pixels(pixel_art::waterline(i, &line)),
        );
    }
}

pub fn add_castle(anim: &mut Animation) {
    let frames = Sprite::single(art::CASTLE)
        .mask(art::CASTLE_MASK)
        .color("BLACK")
        .build();
    let (w, h) = (anim.width as f64, anim.height as f64);
    anim.add(
        Entity::new(Kind::Other, frames)
            .pixels(vec![pixel_art::castle()])
            .at(w - 32.0, h - 13.0, depth::CASTLE),
    );
}

pub fn add_all_seaweed(anim: &mut Animation) {
    // figure out how many seaweed to add by the width of the screen
    for _ in 0..(anim.width / 15).max(0) {
        add_seaweed(anim);
    }
}

pub fn add_seaweed(anim: &mut Animation) {
    let mut images = [String::new(), String::new()];
    let height = rand_int(4.0) as usize + 3;
    for i in 1..=height {
        let left = i % 2;
        images[left].push_str("(\n");
        images[1 - left].push_str(" )\n");
    }
    let x = rand_int(anim.width as f64 - 2.0) + 1.0;
    let y = (anim.height - height as i32) as f64;
    let anim_speed = rand_f(0.05) + 0.25;
    // seaweed lives for 8 to 12 minutes
    let lifetime = Duration::from_secs(rand_int(4.0 * 60.0) as u64 + 8 * 60);
    let frames = Sprite::new(&images).color("green").build();
    anim.add(
        Entity::new(Kind::Seaweed, frames)
            .at(x, y, depth::SEAWEED)
            .speed(0.0, 0.0, 0.0, anim_speed)
            .die_time(Instant::now() + lifetime)
            .on_death(DeathAction::RespawnSeaweed),
    );
}
