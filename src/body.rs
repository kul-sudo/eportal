use std::{collections::HashMap, f32::consts::SQRT_2, time::Instant};

use macroquad::{
    color::{Color, GREEN},
    math::{Vec2, Vec3},
    rand::gen_range,
    shapes::{draw_circle, draw_line, draw_rectangle},
};
use rand::{rngs::StdRng, seq::IteratorRandom, Rng};

use crate::{constants::*, get_with_deviation};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Status {
    FollowingTarget((Instant, Vec2)),
    EscapingBody((Instant, u16)),
    Dead(Instant),
    Walking(Vec2),
    Idle,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EatingStrategy {
    Passive,
    Active,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Body {
    pub pos: Vec2,
    pub energy: Option<f32>,
    pub speed: Option<f32>,
    pub vision_distance: Option<f32>,
    pub eating_strategy: EatingStrategy,
    /// The body procreates after a specific level of energy has been reached.
    pub division_threshold: Option<f32>,
    pub iq: Option<u8>,
    pub max_iq: Option<u8>,
    pub color: Color,
    pub status: Status,
    /// When the body died due to a lack of energy if it did die in the first place.
    pub body_type: u16,
    pub lifespan: f32,
}

#[allow(clippy::too_many_arguments)]
impl Body {
    pub fn new(
        pos: Vec2,
        energy: Option<f32>,
        speed: Option<f32>,
        vision_distance: Option<f32>,
        eating_strategy: EatingStrategy,
        division_threshold: Option<f32>,
        iq: Option<u8>,
        max_iq: Option<u8>,
        color: Color,
        body_type: u16,
        rng: &mut StdRng,
    ) -> Self {
        Body {
            pos,
            energy: Some(match energy {
                Some(energy) => energy / 2.0,
                None => get_with_deviation!(AVERAGE_ENERGY, rng),
            }),
            speed: Some(get_with_deviation!(
                match speed {
                    Some(speed) => speed,
                    None => AVERAGE_SPEED,
                },
                rng
            )),
            vision_distance: Some(get_with_deviation!(
                match vision_distance {
                    Some(vision_distance) => vision_distance,
                    None => AVERAGE_VISION_DISTANCE,
                },
                rng
            )),
            eating_strategy,
            division_threshold: Some(get_with_deviation!(
                match division_threshold {
                    Some(division_threshold) => division_threshold,
                    None => AVERAGE_DIVISION_THRESHOLD,
                },
                rng
            )),
            iq: Some(match iq {
                Some(iq) => {
                    if rng.gen_range(0.0..1.0) < IQ_INCREASE_CHANCE {
                        if Some(iq) == max_iq {
                            iq
                        } else {
                            iq + 1
                        }
                    } else {
                        iq
                    }
                }
                None => 0,
            }),
            max_iq: Some(match max_iq {
                Some(max_iq) => max_iq,
                None => unsafe { (0..MAX_IQ + 1).choose(rng).unwrap_unchecked() },
            }),
            color,
            status: Status::Idle,
            body_type,
            lifespan: LIFESPAN,
        }
    }

    pub fn is_alive(&self) -> bool {
        !matches!(self.status, Status::Dead(..))
    }

    pub fn wrap(&mut self, area_size: Vec2) {
        if self.pos.x >= area_size.x {
            self.pos.x = MIN_GAP;
        } else if self.pos.x <= 0.0 {
            self.pos.x = area_size.x - MIN_GAP;
        }

        if self.pos.y >= area_size.y {
            self.pos.y = MIN_GAP;
        } else if self.pos.y <= 0.0 {
            self.pos.y = area_size.y - MIN_GAP;
        }
    }

    pub fn draw(self) {
        let side_length_half = OBJECT_RADIUS / SQRT_2;

        if self.is_alive() {
            match self.eating_strategy {
                EatingStrategy::Active => {
                    let side_length = side_length_half * 2.0;
                    draw_rectangle(
                        self.pos.x - side_length_half,
                        self.pos.y - side_length_half,
                        side_length,
                        side_length,
                        self.color,
                    )
                }
                EatingStrategy::Passive => {
                    draw_circle(self.pos.x, self.pos.y, OBJECT_RADIUS, self.color)
                }
            }
        } else {
            draw_line(
                self.pos.x - side_length_half,
                self.pos.y - side_length_half,
                self.pos.x + side_length_half,
                self.pos.y + side_length_half,
                2.0,
                self.color,
            );

            draw_line(
                self.pos.x + side_length_half,
                self.pos.y - side_length_half,
                self.pos.x - side_length_half,
                self.pos.y + side_length_half,
                2.0,
                self.color,
            )
        }
    }
}

/// Generate a random position until it suits certain creteria.
pub fn randomly_spawn_body(
    bodies: &mut HashMap<Instant, Body>,
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
    let real_color_gap = COLOR_GAP / ((BODIES_N + 1) as f32).powf(1.0 / 3.0);

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

    while bodies.values().any(|body| {
        let current_body_rgb = Vec3 {
            x: body.color.r,
            y: body.color.g,
            z: body.color.b,
        };
        current_body_rgb.distance(green_rgb) < real_color_gap
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

    bodies.insert(
        Instant::now(),
        Body::new(
            pos,
            None,
            None,
            None,
            eating_strategy,
            None,
            None,
            None,
            color,
            body_type as u16,
            rng,
        ),
    );
}
