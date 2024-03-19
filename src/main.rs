#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(more_float_constants)]

mod body;
mod constants;
mod plant;

use body::*;
use constants::*;
use plant::{randomly_spawn_plant, Plant};

use std::{
    collections::{HashMap, HashSet},
    env::consts::OS,
    f32::consts::SQRT_2,
    intrinsics::unlikely,
    process::exit,
    thread::sleep,
    time::{Duration, Instant},
};

use macroquad::{
    camera::{set_camera, Camera2D},
    color::{Color, GREEN, RED, WHITE},
    input::{is_mouse_button_pressed, mouse_position},
    math::{vec2, Rect, Vec2},
    miniquad::{window::set_fullscreen, MouseButton},
    shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_triangle},
    text::{draw_text, measure_text},
    window::{next_frame, screen_height, screen_width, Conf},
};
use rand::{rngs::StdRng, Rng, SeedableRng};

/// Adjust the coordinates according to the borders.
macro_rules! adjusted_coordinates {
    ($pos:expr, $area_size:expr) => {
        (
            ($pos.x * MAX_ZOOM)
                .max($area_size.x / MAX_ZOOM / 2.0)
                .min($area_size.x * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
            ($pos.y * MAX_ZOOM)
                .max($area_size.y / MAX_ZOOM / 2.0)
                .min($area_size.y * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
        )
    };
}

/// Used for getting specific values with deviations.
#[macro_export]
macro_rules! get_with_deviation {
    ($value:expr, $rng:expr) => {{
        let part = $value * DEVIATION;
        $rng.gen_range($value - part..$value + part)
    }};
}

#[macro_export]
macro_rules! time_since_unix_epoch {
    () => {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    };
}

/// Set the camera zoom to where the mouse cursor is.
fn get_zoom_target(camera: &mut Camera2D, area_size: Vec2) {
    let (x, y) = mouse_position();
    let (target_x, target_y) = adjusted_coordinates!(Vec2 { x, y }, area_size);

    camera.target = vec2(target_x, target_y);
    camera.zoom = vec2(MAX_ZOOM / area_size.x * 2.0, MAX_ZOOM / area_size.y * 2.0);
    set_camera(camera);
}

/// Reset the camera zoom.
fn default_camera(camera: &mut Camera2D, area_size: Vec2) {
    camera.target = vec2(area_size.x / 2.0, area_size.y / 2.0);
    camera.zoom = vec2(MIN_ZOOM / area_size.x * 2.0, MIN_ZOOM / area_size.y * 2.0);
    set_camera(camera);
}

// fn get_nearest_plant_for_body(plants: &[Plant], body: &Body) -> Option<(f32, (usize, Plant))> {
//     let (plant_id, plant) = plants
//         .iter()
//         .enumerate()
//         .min_by_key(|(_, plant)| plant.pos.distance(body.pos) as i16)?;
//     Some((plant.pos.distance(body.pos), (plant_id, *plant)))
// }

// fn get_nearest_body_for_body<'a>(
//     bodies: &'a HashMap<usize, Body<'a>>,
//     body: &Body,
// ) -> Option<(f32, (usize, &'a Body<'a>))> {
//     let (body_id, closest_body) = bodies.iter().min_by_key(|(_, enemy_body)| {
//         distance(vec![enemy_body.x, enemy_body.y], vec![body.x, body.y]) as isize
//     })?;
//     Some((
//         distance(vec![closest_body.x, closest_body.y], vec![body.x, body.y]),
//         (*body_id, closest_body),
//     ))
// }

fn window_conf() -> Conf {
    Conf {
        window_title: "eportal".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Make the window fullscreen on Linux: for some reason, when the application has been built,
    // Arch Linux apparently doesn't have enough time to make it fullscreen
    if OS == "linux" {
        set_fullscreen(true);
        sleep(Duration::from_secs(1));
        next_frame().await;
    }

    let area_size = Vec2 {
        x: screen_width() * OBJECT_RADIUS,
        y: screen_height() * OBJECT_RADIUS,
    };
    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, area_size.x, area_size.y));
    default_camera(&mut camera, area_size);

    let rng = &mut StdRng::from_entropy();

    let mut zoom_mode = false;

    'main_evolution_loop: loop {
        let mut bodies: HashMap<u128, Body> = HashMap::with_capacity(BODIES_N);
        let mut plants: HashMap<u128, Plant> = HashMap::new();

        // Spawn the bodies
        for i in 0..BODIES_N {
            randomly_spawn_body(
                &mut bodies,
                area_size,
                if i >= BODY_EATERS_N {
                    EatingStrategy::Plants
                } else {
                    EatingStrategy::Bodies
                },
                rng,
            );
        }

        // Spawn the plants
        for _ in 0..PLANTS_N {
            randomly_spawn_plant(&bodies, &mut plants, rng, area_size)
        }

        // The timer needed for the FPS
        let mut last_updated = Instant::now();

        loop {
            // Handle the left mouse button click for zooming in/out
            if unlikely(is_mouse_button_pressed(MouseButton::Left)) {
                if zoom_mode {
                    default_camera(&mut camera, area_size);
                } else {
                    get_zoom_target(&mut camera, area_size);
                }

                zoom_mode = !zoom_mode
            }

            if zoom_mode {
                get_zoom_target(&mut camera, area_size);
            }

            // Spawn a plant in a random place with a specific chance
            if rng.gen_range(0.0..1.0) < PLANT_SPAWN_CHANCE {
                randomly_spawn_plant(&bodies, &mut plants, rng, area_size)
            }

            // Whether enough time has passed to draw a new frame
            let is_draw_mode =
                last_updated.elapsed().as_millis() >= Duration::from_secs(1 / FPS).as_millis();

            let bodies_clone = bodies.clone();
            let plants_clone = plants.clone();

            // Due to certain borrowing rules, it's impossible to modify these during the loop,
            // so it'll be done after it
            let mut bodies_to_remove: HashSet<u128> = HashSet::with_capacity(bodies.len());
            let mut bodies_to_spawn: Vec<Body> = Vec::with_capacity(bodies.len());
            let mut body_colors: Vec<Color> = Vec::with_capacity(bodies.len());

            for (body_id, body) in &mut bodies {
                // Handle if dead
                if body.status == Status::Dead {
                    // If body.status == Status::Dead, body.death_time is definitely Some
                    if body.death_time.unwrap().elapsed().as_secs() >= CROSS_LIFESPAN {
                        bodies_to_remove.insert(*body_id);
                    }
                    body.target = None;

                    continue;
                }

                // Check if the body should be dead
                if body.energy.is_sign_negative() {
                    body.death_time = Some(Instant::now());
                    body.status = Status::Dead;

                    continue;
                }

                // Handle the energy
                // The mass is proportional to the energy; to keep the mass up, energy is spent
                body.energy -= ENERGY_SPEND_CONST_FOR_MASS * body.energy
                    + ENERGY_SPEND_CONST_FOR_IQ * body.iq
                    + ENERGY_SPEND_CONST_FOR_VISION * body.vision_distance;
                if body.status != Status::Sleeping {
                    body.energy -= ENERGY_SPEND_CONST_FOR_MOVEMENT * body.speed * body.energy
                }

                body.status = Status::Sleeping;
                body.target = None;

                // Escape
                body.just_wrapped = false;
                let bodies_within_vision_distance = bodies_clone
                    .iter()
                    .filter(|(_, other_body)| {
                        body.pos.distance(other_body.pos) <= body.vision_distance
                    })
                    .collect::<Vec<_>>();

                for (_, other_body) in bodies_within_vision_distance
                    .iter()
                    .filter(|(_, other_body)| {
                        other_body.target.is_some() && other_body.target.unwrap() == *body_id
                    })
                    .collect::<Vec<_>>()
                {
                    body.status = Status::EscapingBody;
                    let distance_to_pefect_body = body.pos.distance(other_body.pos);
                    body.pos = Vec2 {
                        x: body.pos.x
                            - ((other_body.pos.x - body.pos.x) * body.speed)
                                / distance_to_pefect_body,
                        y: body.pos.y
                            - ((other_body.pos.y - body.pos.y) * body.speed)
                                / distance_to_pefect_body,
                    };

                    // Wrap
                    let old_pos = body.pos;
                    if body.pos.x > area_size.x {
                        body.pos.x = MIN_GAP;
                    } else if body.pos.x < 0.0 {
                        body.pos.x = area_size.x - MIN_GAP;
                    }

                    if body.pos.y > area_size.y {
                        body.pos.y = MIN_GAP;
                    } else if body.pos.y < 0.0 {
                        body.pos.y = area_size.y - MIN_GAP;
                    }

                    if old_pos != body.pos {
                        body.just_wrapped = true
                    }

                    continue;
                }

                // Find food according to body.eating_strategy
                match body.eating_strategy {
                    EatingStrategy::Bodies => {
                        if let Some((perfect_body_to_follow_index, perfect_body_to_follow)) =
                            bodies_within_vision_distance
                                .iter()
                                .filter(|(_, other_body)| {
                                    body.energy > other_body.energy
                                        && other_body.status != Status::Dead
                                        && body.color != other_body.color
                                })
                                .min_by(|(_, a), (_, b)| {
                                    body.pos
                                        .distance(a.pos)
                                        .partial_cmp(&body.pos.distance(b.pos))
                                        .unwrap()
                                })
                        {
                            let distance_to_pefect_body =
                                body.pos.distance(perfect_body_to_follow.pos);
                            body.pos = Vec2 {
                                x: body.pos.x
                                    + ((perfect_body_to_follow.pos.x - body.pos.x) * body.speed)
                                        / distance_to_pefect_body,
                                y: body.pos.y
                                    + ((perfect_body_to_follow.pos.y - body.pos.y) * body.speed)
                                        / distance_to_pefect_body,
                            };

                            if body.pos.distance(perfect_body_to_follow.pos) <= body.speed {
                                bodies_to_remove.insert(**perfect_body_to_follow_index);
                                body.energy += perfect_body_to_follow.energy;
                            } else {
                                body.status = Status::FollowingTarget;
                                body.target = Some(**perfect_body_to_follow_index);
                            }
                        }
                    }
                    EatingStrategy::Plants => {
                        if let Some((closest_plant_index, closest_plant)) = plants_clone
                            .iter()
                            .filter(|(_, plant)| {
                                body.pos.distance(plant.pos) <= body.vision_distance
                            })
                            .min_by(|(_, a), (_, b)| {
                                body.pos
                                    .distance(a.pos)
                                    .partial_cmp(&body.pos.distance(b.pos))
                                    .unwrap()
                            })
                        {
                            let distance_to_closest_plant = body.pos.distance(closest_plant.pos);
                            body.pos = Vec2 {
                                x: body.pos.x
                                    + ((closest_plant.pos.x - body.pos.x) * body.speed)
                                        / distance_to_closest_plant,
                                y: body.pos.y
                                    + ((closest_plant.pos.y - body.pos.y) * body.speed)
                                        / distance_to_closest_plant,
                            };

                            if body.pos.distance(closest_plant.pos) <= body.speed {
                                plants.remove(closest_plant_index);
                                body.energy += PLANT_HP;
                            } else {
                                body.status = Status::FollowingTarget;
                                body.target = Some(*closest_plant_index);
                            }
                        }
                    }
                }

                body_colors.push(body.color);

                // Procreate
                if body.energy > body.division_threshold
                // && bodies_clone
                //     .values()
                //     .all(|other_body| body.pos.distance(other_body.pos) > MIN_GAP)
                {
                    for min_gap in [MIN_GAP, -MIN_GAP] {
                        bodies_to_spawn.push(Body::new(
                            Vec2 {
                                x: body.pos.x + min_gap,
                                y: body.pos.y,
                            },
                            body.energy,
                            body.speed,
                            body.vision_distance,
                            body.eating_strategy,
                            body.division_threshold,
                            body.iq,
                            body.color,
                            false,
                            rng,
                        ));
                    }
                    bodies_to_remove.insert(*body_id);
                }
            }

            body_colors.dedup();
            if body_colors.len() == 1 {
                continue 'main_evolution_loop;
            }

            if is_draw_mode {
                for plant in plants.values() {
                    draw_plant!(plant);
                }

                for (body_id, body) in &bodies {
                    if zoom_mode && body.status != Status::Dead {
                        if let Some(target) = body.target {
                            let target_pos_from_hashmap = match body.eating_strategy {
                                EatingStrategy::Bodies => {
                                    let target = bodies.get(&target).unwrap();
                                    match target.just_wrapped {
                                        true => None,
                                        false => Some(target.pos),
                                    }
                                }
                                EatingStrategy::Plants => {
                                    Some(plants_clone.get(&target).unwrap().pos)
                                }
                            };

                            if let Some(target_pos) = target_pos_from_hashmap {
                                draw_line(
                                    body.pos.x,
                                    body.pos.y,
                                    target_pos.x,
                                    target_pos.y,
                                    2.0,
                                    WHITE,
                                );
                            }
                        }
                        draw_circle_lines(
                            body.pos.x,
                            body.pos.y,
                            body.vision_distance,
                            4.0,
                            body.color,
                        );
                        let to_display = format!("{:?} {:?}", body.division_threshold, body.energy);
                        draw_text(
                            &to_display.to_string(),
                            body.pos.x
                                - measure_text(
                                    &to_display.to_string(),
                                    None,
                                    BODY_INFO_FONT_SIZE,
                                    1.0,
                                )
                                .width
                                    / 2.0,
                            body.pos.y - OBJECT_RADIUS - MIN_GAP,
                            BODY_INFO_FONT_SIZE as f32,
                            WHITE,
                        );
                    }

                    draw_body!(body);
                }
                // draw_text(&format!("zoom {}", zoom), 10.0, 20.0, 20.0, WHITE);

                // if zoom_mode {
                //     let mouse_position = mouse_position();
                //     let (x, y) = adjusted_coordinates!(
                //         mouse_position.0 + 25.0,
                //         mouse_position.1 - 25.0,
                //         area_size
                //     );
                //     draw_text("zoomed in", x, y, 10.0 * MAX_ZOOM, WHITE)
                // }

                last_updated = Instant::now();
                next_frame().await;
            }

            for body_index in &bodies_to_remove {
                bodies.remove(body_index);
            }

            for body in &bodies_to_spawn {
                spawn_body(&mut bodies, *body)
            }
        }
    }
}
