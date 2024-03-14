use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use macroquad::math::Vec2;
use rand::{rngs::ThreadRng, Rng};

use crate::{Body, MIN_GAP, OBJECT_RADIUS, PLANT_SPAWN_TIME_LIMIT};

#[derive(Clone, Copy, PartialEq)]
pub struct Plant {
    pub pos: Vec2,
}

pub fn randomly_spawn_plant(
    bodies: &mut HashMap<usize, Body>,
    plants: &mut Vec<Plant>,
    rng: &mut ThreadRng,
    area_size: Vec2,
) {
    let starting_point = Instant::now();

    let mut pos = Vec2::default();

    while {
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
                .iter()
                .any(|plant| plant.pos.distance(pos) <= OBJECT_RADIUS * 2.0 + MIN_GAP)
    } {}

    plants.push(Plant { pos });
}

#[macro_export]
macro_rules! draw_plant {
    ($plant:expr) => {
        draw_triangle(
            Vec2 {
                x: $plant.pos.x,
                y: $plant.pos.y - OBJECT_RADIUS,
            },
            Vec2 {
                x: $plant.pos.x + OBJECT_RADIUS * (COSINE_OF_30_DEGREES),
                y: $plant.pos.y + OBJECT_RADIUS / 2.0,
            },
            Vec2 {
                x: $plant.pos.x - OBJECT_RADIUS * (COSINE_OF_30_DEGREES),
                y: $plant.pos.y + OBJECT_RADIUS / 2.0,
            },
            GREEN,
        );
    };
}
