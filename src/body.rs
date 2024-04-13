use std::{
    collections::HashMap,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use macroquad::{
    color::{Color, GREEN, RED},
    math::{Vec2, Vec3},
    rand::gen_range,
};
use rand::{rngs::StdRng, seq::IteratorRandom, Rng};

use crate::{constants::*, get_with_deviation, time_since_unix_epoch};

#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    FollowingTarget((u128, Vec2)),
    EscapingBody((u128, u16)),
    Sleeping,
    Dead(Instant),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EatingStrategy {
    Bodies,
    Plants,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Body {
    pub pos: Vec2,
    pub energy: f32,
    pub speed: f32,
    pub vision_distance: f32,
    pub eating_strategy: EatingStrategy,
    /// The body procreates after a specific level of energy has been reached.
    pub division_threshold: f32,
    pub iq: f32,
    pub color: Color,
    pub status: Status,
    /// When the body died due to a lack of energy if it did die in the first place.
    pub body_type: u16,
    pub lifespan: u64,
}

#[allow(clippy::too_many_arguments)]
impl Body {
    pub fn new(
        pos: Vec2,
        energy: f32,
        speed: f32,
        vision_distance: f32,
        eating_strategy: EatingStrategy,
        division_threshold: f32,
        iq: f32,
        color: Color,
        is_first_generation: bool,
        rng: &mut StdRng,
        body_type: u16,
        lifespan: u64,
    ) -> Self {
        Body {
            pos,
            energy: if is_first_generation {
                get_with_deviation!(energy, rng)
            } else {
                energy / 2.0
            } - VISION_DISTANCE_BIRTH_ENERGY_SPENT * vision_distance
                - SPEED_BIRTH_ENERGY_SPENT * speed,
            speed: get_with_deviation!(speed, rng),
            vision_distance: get_with_deviation!(vision_distance, rng),
            eating_strategy,
            division_threshold: get_with_deviation!(division_threshold, rng),
            iq,
            color,
            status: Status::Sleeping,
            body_type,
            lifespan: if is_first_generation {
                unsafe { LIFESPAN_RANGE.choose(rng).unwrap_unchecked() }
            } else {
                lifespan
            },
        }
    }

    pub fn is_alive(&self) -> bool {
        !matches!(self.status, Status::Dead(_))
    }
}

/// Generate a random position until it suits certain creteria.
pub fn randomly_spawn_body(
    bodies: &mut HashMap<u128, Body>,
    area_size: Vec2,
    eating_strategy: EatingStrategy,
    rng: &mut StdRng,
    body_type: usize,
) {
    let mut pos = Vec2::default();

    // Make sure the position is far enough from the rest of the bodies and the borders of the area
    while {
        pos.x = rng.gen_range(0.0..area_size.x);
        pos.y = rng.gen_range(0.0..area_size.y);
        (pos.x <= OBJECT_RADIUS + MIN_GAP || pos.x >= area_size.x - OBJECT_RADIUS - MIN_GAP)
            || (pos.y <= OBJECT_RADIUS + MIN_GAP || pos.y >= area_size.y - OBJECT_RADIUS - MIN_GAP)
            || bodies
                .values()
                .any(|body| body.pos.distance(pos) < OBJECT_RADIUS * 2.0 + MIN_GAP)
    } {}

    // Make sure the color is different enough
    let real_color_gap = COLOR_GAP / ((BODIES_N + 2) as f32).powf(1.0 / 3.0);

    let mut color = Color::from_rgba(
        gen_range(COLOR_MIN, COLOR_MAX),
        gen_range(COLOR_MIN, COLOR_MAX),
        gen_range(COLOR_MIN, COLOR_MAX),
        255,
    );

    let green_rgb = Vec3 {
        x: GREEN.r,
        y: GREEN.g,
        z: GREEN.b,
    };

    let red_rgb = Vec3 {
        x: RED.r,
        y: RED.g,
        z: RED.b,
    };

    while bodies.values().any(|body| {
        let current_body_rgb = Vec3 {
            x: body.color.r,
            y: body.color.g,
            z: body.color.b,
        };
        current_body_rgb.distance(green_rgb) < real_color_gap
            || current_body_rgb.distance(red_rgb) < real_color_gap
            || current_body_rgb.distance(Vec3 {
                x: color.r,
                y: color.g,
                z: color.b,
            }) < real_color_gap
    }) {
        color = Color::from_rgba(
            gen_range(COLOR_MIN, COLOR_MAX),
            gen_range(COLOR_MIN, COLOR_MAX),
            gen_range(COLOR_MIN, COLOR_MAX),
            255,
        )
    }

    let body_eater = eating_strategy == EatingStrategy::Bodies;

    bodies.insert(
        time_since_unix_epoch!(),
        Body::new(
            pos,
            if body_eater {
                AVERAGE_ENERGY
            } else {
                AVERAGE_ENERGY
            },
            if body_eater {
                AVERAGE_SPEED
            } else {
                AVERAGE_SPEED
            },
            if body_eater {
                AVERAGE_VISION_DISTANCE
            } else {
                AVERAGE_VISION_DISTANCE
            },
            eating_strategy,
            if body_eater {
                AVERAGE_DIVISION_THRESHOLD
            } else {
                AVERAGE_DIVISION_THRESHOLD
            },
            0.0,
            color,
            true,
            rng,
            body_type as u16,
            0,
        ),
    );
}

#[macro_export]
macro_rules! draw_body {
    ($body:expr) => {
        let side_length_half = OBJECT_RADIUS / SQRT_2;

        if $body.is_alive() {
            match $body.eating_strategy {
                EatingStrategy::Bodies => {
                    let side_length = side_length_half * 2.0;
                    draw_rectangle(
                        $body.pos.x - side_length_half,
                        $body.pos.y - side_length_half,
                        side_length,
                        side_length,
                        $body.color,
                    )
                }

                EatingStrategy::Plants => {
                    draw_circle($body.pos.x, $body.pos.y, OBJECT_RADIUS, $body.color)
                }
            }
        } else {
            draw_line(
                $body.pos.x - side_length_half,
                $body.pos.y - side_length_half,
                $body.pos.x + side_length_half,
                $body.pos.y + side_length_half,
                2.0,
                RED,
            );

            draw_line(
                $body.pos.x + side_length_half,
                $body.pos.y - side_length_half,
                $body.pos.x - side_length_half,
                $body.pos.y + side_length_half,
                2.0,
                RED,
            )
        }
    };
}
