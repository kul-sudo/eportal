use std::{collections::HashMap, time::Instant};

use macroquad::{
    color::{Color, GREEN, RED},
    math::{Vec2, Vec3},
    rand::gen_range,
};
use rand::{thread_rng, Rng};

use crate::{
    get_with_deviation, plant::Plant, AVERAGE_DIVISION_THRESHOLD, AVERAGE_ENERGY, AVERAGE_SPEED,
    AVERAGE_VISION_DISTANCE, BODIES_N, COLOR_GAP, MIN_GAP, OBJECT_RADIUS,
};

#[derive(Clone, Copy, PartialEq)]
pub enum Status<'a> {
    FollowingBody(&'a Body<'a>),
    FollowingPlant(&'a Plant),
    EscapingBody(&'a Body<'a>),
    Sleeping,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EatingStrategy {
    Bodies,
    Plants,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Body<'a> {
    pub pos: Vec2,
    pub energy: f32,
    pub speed: f32,
    pub vision_distance: f32,
    pub eating_strategy: EatingStrategy,
    pub division_threshold: f32,
    pub iq: f32,
    pub color: Color,
    pub status: Status<'a>,
    pub death_time: Option<Instant>,
}

#[allow(clippy::too_many_arguments)]
impl Body<'_> {
    pub fn new(
        pos: Vec2,
        energy: f32,
        speed: f32,
        vision_distance: f32,
        eating_strategy: EatingStrategy,
        division_threshold: f32,
        iq: f32,
        color: Color,
        status: Status<'static>,
    ) -> Self {
        Body {
            pos,
            energy,
            speed,
            vision_distance,
            eating_strategy,
            division_threshold,
            iq,
            color,
            status,
            death_time: None,
        }
    }
}

pub fn spawn_body<'a>(bodies: &mut HashMap<usize, Body<'a>>, body: Body<'a>) {
    bodies.insert(bodies.len() + 1, body);
}

pub fn randomly_spawn_body(
    bodies: &mut HashMap<usize, Body>,
    area_size: Vec2,
    eating_strategy: EatingStrategy,
) {
    let rng = &mut thread_rng();

    let mut pos = Vec2::default();

    while {
        pos.x = rng.gen_range(0.0..area_size.x);
        pos.y = rng.gen_range(0.0..area_size.y);
        (pos.x <= OBJECT_RADIUS + MIN_GAP || pos.x >= area_size.x - OBJECT_RADIUS - MIN_GAP)
            || (pos.y <= OBJECT_RADIUS + MIN_GAP || pos.y >= area_size.y - OBJECT_RADIUS - MIN_GAP)
            || bodies
                .values()
                .any(|body| body.pos.distance(pos) < OBJECT_RADIUS * 2.0 + MIN_GAP)
    } {}

    let real_color_gap = COLOR_GAP / ((BODIES_N + 2) as f32).powf(1.0 / 3.0);

    let mut color = Color::from_rgba(
        gen_range(50, 250),
        gen_range(50, 250),
        gen_range(50, 250),
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
            gen_range(50, 250),
            gen_range(50, 250),
            gen_range(50, 250),
            255,
        )
    }

    spawn_body(
        bodies,
        Body::new(
            pos,
            get_with_deviation(AVERAGE_ENERGY, rng),
            get_with_deviation(AVERAGE_SPEED, rng),
            get_with_deviation(AVERAGE_VISION_DISTANCE, rng),
            eating_strategy,
            get_with_deviation(AVERAGE_DIVISION_THRESHOLD, rng),
            0.0,
            color,
            Status::Sleeping,
        ),
    );
}

#[macro_export]
macro_rules! draw_body {
    ($body:expr) => {
        let side_length_half = OBJECT_RADIUS / SQRT_2;

        match $body.death_time {
            Some(_) => {
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
            None => match $body.eating_strategy {
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
            },
        }
    };
}
