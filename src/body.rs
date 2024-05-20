use std::{collections::HashMap, f32::consts::SQRT_2, intrinsics::unlikely, time::Instant};

use macroquad::{
    color::{Color, GREEN, PURPLE, RED},
    math::{vec2, Circle, Rect, Vec2, Vec3},
    rand::gen_range,
    shapes::{draw_circle, draw_line, draw_rectangle, draw_rectangle_lines},
    window::{screen_height, screen_width},
};
use rand::{rngs::StdRng, seq::IteratorRandom, Rng};

use crate::{
    constants::*,
    get_with_deviation,
    smart_drawing::{DrawingStrategy, RectangleCorner, SEARCH_SEQUENCE},
    zoom::Zoom,
};

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Virus {
    SpeedVirus,
    VisionVirus,
}

#[derive(Clone, PartialEq)]
pub struct Body {
    pub pos: Vec2,
    pub energy: f32,
    pub speed: f32,
    pub vision_distance: f32,
    pub eating_strategy: EatingStrategy,
    /// The body procreates after a specific level of energy has been reached.
    pub division_threshold: f32,
    pub iq: u8,
    pub max_iq: u8,
    pub color: Color,
    pub status: Status,
    /// When the body died due to a lack of energy if it did die in the first place.
    pub body_type: u16,
    pub lifespan: f32,
    pub viruses: HashMap<Virus, f32>,
    pub initial_speed: f32,
    pub initial_vision_distance: f32,
}

#[allow(clippy::too_many_arguments)]
impl Body {
    pub fn new(
        pos: Vec2,
        energy: Option<f32>,
        eating_strategy: EatingStrategy,
        division_threshold: Option<f32>,
        iq: Option<u8>,
        max_iq: Option<u8>,
        color: Color,
        body_type: u16,
        viruses: Option<HashMap<Virus, f32>>,
        initial_speed: Option<f32>,
        initial_vision_distance: Option<f32>,
        rng: &mut StdRng,
    ) -> Self {
        let speed = get_with_deviation!(
            match initial_speed {
                Some(initial_speed) => initial_speed,
                None => AVERAGE_SPEED,
            },
            rng
        );

        let vision_distance = get_with_deviation!(
            match initial_vision_distance {
                Some(initial_vision_distance) => initial_vision_distance,
                None => AVERAGE_VISION_DISTANCE,
            },
            rng
        );

        let mut body = Body {
            pos,
            energy: match energy {
                Some(energy) => energy / 2.0,
                None => get_with_deviation!(AVERAGE_ENERGY, rng),
            },
            speed,
            initial_speed: speed,
            vision_distance,
            initial_vision_distance: vision_distance,
            eating_strategy,
            division_threshold: get_with_deviation!(
                match division_threshold {
                    Some(division_threshold) => division_threshold,
                    None => AVERAGE_DIVISION_THRESHOLD,
                },
                rng
            ),
            iq: match iq {
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
            },
            max_iq: match max_iq {
                Some(max_iq) => max_iq,
                None => (0..=MAX_IQ).choose(rng).unwrap(),
            },
            color,
            status: Status::Idle,
            body_type,
            lifespan: LIFESPAN,
            viruses: match viruses {
                Some(mut viruses) => {
                    for energy_spent_for_healing in viruses.values_mut() {
                        *energy_spent_for_healing = 0.0
                    }

                    viruses
                }
                None => {
                    let mut viruses = HashMap::new();
                    for virus in [Virus::SpeedVirus, Virus::VisionVirus] {
                        if rng.gen_range(0.0..1.0)
                            <= match virus {
                                Virus::SpeedVirus => SPEEDVIRUS_FIRST_GENERATION_INFECTION_CHANCE,
                                Virus::VisionVirus => VISIONVIRUS_FIRST_GENERATION_INFECTION_CHANCE,
                            }
                        {
                            viruses.insert(
                                virus,
                                rng.gen_range(
                                    0.0..match virus {
                                        Virus::SpeedVirus => SPEEDVIRUS_HEAL_ENERGY,
                                        Virus::VisionVirus => VISIONVIRUS_HEAL_ENERGY,
                                    },
                                ), // Assuming the evolution
                                   // theoretically starts before it starts being shown
                            );
                        }
                    }

                    viruses
                }
            },
        };

        body.get_viruses(body.viruses.clone());
        body
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

    pub fn draw(&self) {
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
                    );
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

        if !self.viruses.is_empty() {
            draw_circle(self.pos.x, self.pos.y, 5.0, RED)
        }
    }

    pub fn get_viruses(&mut self, viruses: HashMap<Virus, f32>) {
        for virus in viruses.keys() {
            if !self.viruses.contains_key(virus) {
                self.viruses.insert(*virus, 0.0);
                match virus {
                    Virus::SpeedVirus => self.speed -= self.speed * SPEEDVIRUS_SPEED_DECREASE,
                    Virus::VisionVirus => {
                        self.vision_distance -=
                            self.vision_distance * VISIONVIRUS_VISION_DISTANCE_DECREASE
                    }
                }
            }
        }
    }

    pub fn get_drawing_strategy(&self, zoom: Zoom) -> DrawingStrategy {
        let mut drawing_strategy = DrawingStrategy::default();
        let mut target_line = None; // It hasn't been decided yet whether it's needed to draw a
                                    // line

        let mut rectangle_corners = HashMap::with_capacity(4);
        for corner in [
            RectangleCorner::TopRight,
            RectangleCorner::TopLeft,
            RectangleCorner::BottomRight,
            RectangleCorner::BottomLeft,
        ] {
            let (i, j) = match corner {
                RectangleCorner::TopRight => (1.0, 1.0),
                RectangleCorner::TopLeft => (-1.0, 1.0),
                RectangleCorner::BottomRight => (1.0, -1.0),
                RectangleCorner::BottomLeft => (-1.0, -1.0),
            };

            rectangle_corners.insert(
                corner,
                vec2(
                    zoom.center_pos.unwrap().x + i * zoom.scaling_width / 2.0,
                    zoom.center_pos.unwrap().y + j * zoom.scaling_height / 2.0,
                ),
            );
        }

        // Step 1
        draw_rectangle_lines(
            zoom.center_pos.unwrap().x - zoom.width / 2.0,
            zoom.center_pos.unwrap().y - zoom.height / 2.0,
            zoom.width,
            zoom.height,
            50.0,
            PURPLE,
        );

        if zoom.extended_rect.unwrap().contains(self.pos) {
            // The body can be partially
            // visible/hidden or completely visible.
            drawing_strategy.body = true;
            drawing_strategy.vision_distance = true;

            if let Status::FollowingTarget((_, target_pos)) = self.status {
                if zoom.rect.unwrap().contains(target_pos) {
                    target_line = Some(true);
                }
            }
        } else {
            drawing_strategy.body = false;
            drawing_strategy.vision_distance =
                Circle::new(self.pos.x, self.pos.y, self.vision_distance)
                    .overlaps_rect(&zoom.rect.unwrap());
        }

        // Step 3
        match target_line {
            Some(_) => {
                if !drawing_strategy.body {
                    if drawing_strategy.vision_distance {
                        target_line = None;
                    } else {
                        target_line = if unlikely(self.vision_distance > zoom.diagonal)
                        // It is very unlikely for the vision distance to be big enough to cover the
                        // whole diagonal of the zoom area
                        {
                            None
                        } else {
                            Some(false)
                        }
                    }
                }
            }
            None => {
                for (i, j) in SEARCH_SEQUENCE {
                    if let Status::FollowingTarget((_, target_pos)) = self.status {
                        target_line = Some(false);
                        if DrawingStrategy::segments_intersect(
                            self.pos,
                            target_pos,
                            *rectangle_corners.get(&i).unwrap(),
                            *rectangle_corners.get(&j).unwrap(),
                        ) {
                            target_line = Some(true);
                            break;
                        }
                    }
                }
            }
        }

        if let Status::FollowingTarget(_) = self.status {
            drawing_strategy.target_line = target_line.unwrap();
        } else {
            drawing_strategy.target_line = false;
        }

        drawing_strategy
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

    bodies.insert(
        Instant::now(),
        Body::new(
            pos,
            None,
            eating_strategy,
            None,
            None,
            None,
            color,
            body_type as u16,
            None,
            None,
            None,
            rng,
        ),
    );
}
