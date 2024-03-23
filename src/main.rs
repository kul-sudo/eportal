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
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
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
        Vec2 {
            x: ($pos.x * MAX_ZOOM)
                .max($area_size.x / MAX_ZOOM / 2.0)
                .min($area_size.x * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
            y: ($pos.y * MAX_ZOOM)
                .max($area_size.y / MAX_ZOOM / 2.0)
                .min($area_size.y * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
        }
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
    let target = adjusted_coordinates!(Vec2 { x, y }, area_size);

    camera.target = target;
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
        // OBJECT_RADIUS is equal to one pixel when unzoomed
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
            for _ in 0..PLANTS_N_FOR_ONE_STEP {
                randomly_spawn_plant(&bodies, &mut plants, rng, area_size)
            }

            // Whether enough time has passed to draw a new frame
            let is_draw_mode =
                last_updated.elapsed().as_millis() >= Duration::from_secs(1 / FPS).as_millis();

            // Due to certain borrowing rules, it's impossible to modify these during the loop,
            // so it'll be done after it
            let mut body_colors: Vec<Color> = Vec::with_capacity(bodies.len());
            let mut new_bodies: HashMap<u128, Body> = HashMap::with_capacity(bodies.len() * 2);
            let mut removed_plants: HashSet<u128> = HashSet::with_capacity(bodies.len());
            let mut bodies_to_remove: HashSet<u128> = HashSet::with_capacity(bodies.len());
            let bodies_shot = bodies.clone();
            let plants_shot = plants.clone();

            for (body_id, body) in &mut bodies {
                // Handle if the body was eaten earlier
                if bodies_to_remove.contains(body_id) {
                    continue;
                }

                // Handle if dead
                if body.status == Status::Dead
                    && body.death_time.unwrap().elapsed().as_secs() >= CROSS_LIFESPAN
                {
                    bodies_to_remove.insert(*body_id);
                    continue;
                }

                // Handle the energy
                // The mass is proportional to the energy; to keep the mass up, energy is spent
                body.energy -= ENERGY_SPEND_CONST_FOR_MASS * body.energy
                    + ENERGY_SPEND_CONST_FOR_IQ * body.iq
                    + ENERGY_SPEND_CONST_FOR_VISION * body.vision_distance;
                if body.status == Status::FollowingTarget || body.status == Status::EscapingBody {
                    body.energy -= ENERGY_SPEND_CONST_FOR_MOVEMENT * body.speed * body.energy
                }

                // If needed, it's changed in the future
                body.target = None;

                // Check if the body should be dead
                if body.energy.is_sign_negative() {
                    body.death_time = Some(Instant::now());
                    body.status = Status::Dead;

                    continue;
                }

                // Escape
                let bodies_within_vision_distance = bodies_shot
                    .iter()
                    .filter(|(other_body_id, other_body)| {
                        !bodies_to_remove.contains(other_body_id)
                            && body.pos.distance(other_body.pos) <= body.vision_distance
                    })
                    .collect::<Vec<_>>();

                if let Some((_, closest_chasing_body)) = bodies_within_vision_distance
                    .iter()
                    .filter(|(_, other_body)| {
                        if let Some(other_body_target) = other_body.target {
                            other_body_target.0 == *body_id
                        } else {
                            false
                        }
                    })
                    .min_by(|(_, a), (_, b)| {
                        body.pos
                            .distance(a.pos)
                            .total_cmp(&body.pos.distance(b.pos))
                    })
                {
                    body.status = Status::EscapingBody;

                    let distance_to_closest_chasing_body =
                        body.pos.distance(closest_chasing_body.pos);
                    body.pos = Vec2 {
                        x: body.pos.x
                            - ((closest_chasing_body.pos.x - body.pos.x) * body.speed)
                                / distance_to_closest_chasing_body,
                        y: body.pos.y
                            - ((closest_chasing_body.pos.y - body.pos.y) * body.speed)
                                / distance_to_closest_chasing_body,
                    };

                    // Wrap
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

                    continue;
                }

                body.status = Status::Sleeping;

                // Find food according to body.eating_strategy
                match body.eating_strategy {
                    EatingStrategy::Bodies => {
                        if let Some((prey_id, prey)) = bodies_within_vision_distance
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
                            let distance_to_prey = body.pos.distance(prey.pos);
                            if distance_to_prey <= body.speed {
                                body.energy += prey.energy;
                                body.pos = prey.pos;
                                bodies_to_remove.insert(**prey_id);
                            } else {
                                body.status = Status::FollowingTarget;
                                body.target = Some((**prey_id, prey.pos));
                                body.pos = Vec2 {
                                    x: body.pos.x
                                        + ((prey.pos.x - body.pos.x) * body.speed)
                                            / distance_to_prey,
                                    y: body.pos.y
                                        + ((prey.pos.y - body.pos.y) * body.speed)
                                            / distance_to_prey,
                                };
                                continue;
                            }
                        }
                    }
                    EatingStrategy::Plants => {
                        if let Some((closest_plant_index, closest_plant)) = plants_shot
                            .iter()
                            .filter(|(plant_id, plant)| {
                                !removed_plants.contains(plant_id)
                                    && body.pos.distance(plant.pos) <= body.vision_distance
                            })
                            .min_by(|(_, a), (_, b)| {
                                body.pos
                                    .distance(a.pos)
                                    .partial_cmp(&body.pos.distance(b.pos))
                                    .unwrap()
                            })
                        {
                            let distance_to_closest_plant = body.pos.distance(closest_plant.pos);
                            if distance_to_closest_plant <= body.speed {
                                body.energy += PLANT_HP;
                                body.pos = closest_plant.pos;
                                plants.remove(closest_plant_index);
                                removed_plants.insert(*closest_plant_index);
                            } else {
                                body.status = Status::FollowingTarget;
                                body.target = Some((*closest_plant_index, closest_plant.pos));
                                body.pos = Vec2 {
                                    x: body.pos.x
                                        + ((closest_plant.pos.x - body.pos.x) * body.speed)
                                            / distance_to_closest_plant,
                                    y: body.pos.y
                                        + ((closest_plant.pos.y - body.pos.y) * body.speed)
                                            / distance_to_closest_plant,
                                };

                                continue;
                            }
                        }
                    }
                }

                body_colors.push(body.color);

                // Procreate
                if body.energy > body.division_threshold {
                    for lambda in [1.0, -1.0] {
                        spawn_body!(
                            &mut new_bodies,
                            &shot,
                            Body::new(
                                Vec2 {
                                    x: body.pos.x + OBJECT_RADIUS * lambda,
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
                            )
                        );
                    }
                    bodies_to_remove.insert(*body_id);
                }
            }

            for body_id in bodies_to_remove {
                bodies.remove(&body_id);
            }

            for (new_body_id, new_body) in new_bodies {
                bodies.insert(new_body_id, new_body);
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
                    if zoom_mode {
                        if let Some((_, target_pos)) = body.target {
                            draw_line(
                                body.pos.x,
                                body.pos.y,
                                target_pos.x,
                                target_pos.y,
                                2.0,
                                WHITE,
                            );
                        }
                        draw_circle_lines(
                            body.pos.x,
                            body.pos.y,
                            body.vision_distance,
                            2.0,
                            body.color,
                        );
                        let to_display = body_id.to_string();
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
        }
    }
}
