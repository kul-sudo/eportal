use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use macroquad::{color::GREEN, math::Vec2, shapes::draw_triangle};
use rand::{rngs::StdRng, Rng};

use crate::{Body, COSINE_OF_30_DEGREES, MIN_GAP, OBJECT_RADIUS, PLANT_SPAWN_TIME_LIMIT};

#[derive(Debug, Clone, PartialEq)]
pub struct Plant {
    pub pos: Vec2,
}

impl Plant {
    pub fn draw(&self) {
        draw_triangle(
            Vec2 {
                x: self.pos.x,
                y: self.pos.y - OBJECT_RADIUS,
            },
            Vec2 {
                x: self.pos.x + OBJECT_RADIUS * (COSINE_OF_30_DEGREES),
                y: self.pos.y + OBJECT_RADIUS / 2.0,
            },
            Vec2 {
                x: self.pos.x - OBJECT_RADIUS * (COSINE_OF_30_DEGREES),
                y: self.pos.y + OBJECT_RADIUS / 2.0,
            },
            GREEN,
        );
    }
}

pub fn randomly_spawn_plant(
    bodies: &HashMap<u128, Body>,
    plants: &mut HashMap<u128, Plant>,
    rng: &mut StdRng,
    area_size: Vec2,
    epoch_start: Instant,
) {
    let starting_point = Instant::now();

    let mut pos = Vec2::default();

    // Make sure the position is far enough from the rest of the plants and bodies and the borders of the area
    while {
        // Make sure finding a suitable position doesn't exceed a specific time limit
        if starting_point.elapsed().as_nanos()
            >= Duration::from_millis(PLANT_SPAWN_TIME_LIMIT).as_nanos()
        {
            return;
        }
        pos.x = rng.gen_range(0.0..area_size.x);
        pos.y = rng.gen_range(0.0..area_size.y);
        (pos.x <= OBJECT_RADIUS + MIN_GAP || pos.x >= area_size.x - OBJECT_RADIUS - MIN_GAP)
            || (pos.y <= OBJECT_RADIUS + MIN_GAP || pos.y >= area_size.y - OBJECT_RADIUS - MIN_GAP)
            || bodies
                .values()
                .any(|body| body.pos.distance(pos) <= OBJECT_RADIUS * 2.0 + MIN_GAP)
            || plants
                .values()
                .any(|plant| plant.pos.distance(pos) <= OBJECT_RADIUS * 2.0 + MIN_GAP)
    } {}

    let mut key;

    // Make sure the position is far enough from the rest of the bodies and the borders of the area
    while {
        key = epoch_start.elapsed().as_nanos();
        plants.contains_key(&key)
    } {}
    plants.insert(key, Plant { pos });
}
