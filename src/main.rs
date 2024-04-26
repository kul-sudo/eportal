#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(more_float_constants)]
#![feature(exclusive_range_pattern)]

mod body;
mod constants;
mod plant;

use body::*;
use constants::*;
use plant::{randomly_spawn_plant, Plant};

use std::{
    collections::{HashMap, HashSet},
    env::consts::OS,
    f32::consts::PI,
    intrinsics::unlikely,
    io::Result,
    thread::sleep,
    time::{Duration, Instant},
};

use macroquad::{
    camera::{set_camera, Camera2D},
    color::WHITE,
    input::{is_key_down, is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode},
    math::{vec2, Rect, Vec2},
    miniquad::{window::set_fullscreen, MouseButton},
    shapes::{draw_circle_lines, draw_line},
    text::{draw_text, measure_text},
    window::{next_frame, screen_height, screen_width, Conf},
};
use rand::{rngs::StdRng, seq::IteratorRandom, Rng, SeedableRng};

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

fn window_conf() -> Conf {
    Conf {
        window_title: "eportal".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<()> {
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
    let mut show_info = true;

    // 'main_evolution_loop: loop {
    let mut bodies: HashMap<Instant, Body> = HashMap::with_capacity(BODIES_N);
    let mut plants: HashMap<Instant, Plant> = HashMap::new();
    let mut removed_plants: HashSet<Instant> = HashSet::new();
    let mut removed_bodies: HashSet<Instant> = HashSet::with_capacity(bodies.len());

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
            i + 1,
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

        if unlikely(is_key_pressed(KeyCode::Key1)) {
            show_info = !show_info
        }

        let is_draw_prevented = is_key_down(KeyCode::Space);

        if zoom_mode {
            get_zoom_target(&mut camera, area_size);
        }

        // Remove plants
        let n_to_remove =
            ((plants.len() - removed_plants.len()) as f32 * PART_OF_PLANTS_TO_REMOVE) as usize;

        for _ in 0..n_to_remove {
            loop {
                let random_plant_id = unsafe { plants.iter().choose(rng).unwrap_unchecked() }.0;
                if !removed_plants.contains(random_plant_id) {
                    removed_plants.insert(*random_plant_id);
                    break;
                }
            }
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
        let mut new_bodies: HashMap<Instant, Body> = HashMap::with_capacity(bodies.len() * 2);
        let bodies_shot = bodies.clone();
        let mut bodies_shot_for_statuses = bodies.clone();

        let plants_shot = plants.clone();

        for (body_id, body) in &mut bodies {
            // Handle if the body was eaten earlier
            if removed_bodies.contains(body_id) {
                continue;
            }

            // Handle if completely dead
            if let Status::Dead(death_time) = body.status {
                if death_time.elapsed().as_secs() >= CROSS_LIFESPAN {
                    removed_bodies.insert(*body_id);
                }
                continue;
            }

            // Handle lifespan
            if body.status != Status::Idle {
                body.lifespan = (body.lifespan
                    - CONST_FOR_LIFESPAN
                        * unsafe { body.speed.unwrap_unchecked() }
                        * unsafe { body.energy.unwrap_unchecked() })
                .max(0.0)
            }

            // Handle if dead to become a cross
            if unsafe { body.energy.unwrap_unchecked() } < MIN_ENERGY
                || body_id.elapsed().as_secs_f32() > body.lifespan
            {
                body.status = Status::Dead(Instant::now());
                continue;
            }

            // Handle the energy
            // The mass is proportional to the energy; to keep the mass up, energy is spent
            body.energy = Some(
                unsafe { body.energy.unwrap_unchecked() }
                    - ENERGY_SPENT_CONST_FOR_MASS * unsafe { body.energy.unwrap_unchecked() }
                    - ENERGY_SPENT_CONST_FOR_IQ * unsafe { body.iq.unwrap_unchecked() } as f32
                    - ENERGY_SPENT_CONST_FOR_VISION_DISTANCE
                        * unsafe { body.vision_distance.unwrap_unchecked() }.powi(2),
            );

            if body.status != Status::Idle {
                body.energy = Some(
                    unsafe { body.energy.unwrap_unchecked() }
                        - match body.eating_strategy {
                            EatingStrategy::Bodies => BODY_EATER_ENERGY_SPENT_CONST_FOR_MOVEMENT,
                            EatingStrategy::Plants => PLANT_EATER_ENERGY_SPENT_CONST_FOR_MOVEMENT,
                        } * unsafe { body.speed.unwrap_unchecked() }
                            * unsafe { body.energy.unwrap_unchecked() },
                )
            }

            if body.energy <= Some(0.0) {
                removed_bodies.insert(*body_id);
                continue;
            }

            // Escape
            let bodies_within_vision_distance = bodies_shot
                .iter()
                .filter(|(other_body_id, other_body)| {
                    body_id != *other_body_id
                        && !removed_bodies.contains(other_body_id)
                        && body.pos.distance(other_body.pos)
                            <= unsafe { body.vision_distance.unwrap_unchecked() }
                })
                .collect::<Vec<_>>();

            if let Some((closest_chasing_body_id, closest_chasing_body)) =
                bodies_within_vision_distance
                    .iter()
                    .filter(|(other_body_id, _)| {
                        if let Status::FollowingTarget(other_body_target) = unsafe {
                            bodies_shot_for_statuses
                                .get(other_body_id)
                                .unwrap_unchecked()
                        }
                        .status
                        {
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
                body.status = Status::EscapingBody((
                    **closest_chasing_body_id,
                    closest_chasing_body.body_type,
                ));
                unsafe { bodies_shot_for_statuses.get_mut(body_id).unwrap_unchecked() }.status =
                    body.status;

                let distance_to_closest_chasing_body = body.pos.distance(closest_chasing_body.pos);
                body.pos = Vec2 {
                    x: body.pos.x
                        - ((closest_chasing_body.pos.x - body.pos.x)
                            * unsafe { body.speed.unwrap_unchecked() })
                            / distance_to_closest_chasing_body,
                    y: body.pos.y
                        - ((closest_chasing_body.pos.y - body.pos.y)
                            * unsafe { body.speed.unwrap_unchecked() })
                            / distance_to_closest_chasing_body,
                };

                body.wrap(area_size);

                continue;
            }

            // Find food according to body.eating_strategy
            match body.eating_strategy {
                EatingStrategy::Bodies => {
                    if let Some((prey_id, prey)) = bodies_within_vision_distance
                        .iter()
                        .filter(|(other_body_id, other_body)| {
                            other_body.body_type != body.body_type
                                && match other_body.eating_strategy {
                                    EatingStrategy::Bodies => {
                                        if other_body.is_alive() {
                                            body.energy > other_body.energy
                                        } else {
                                            true
                                        }
                                    }
                                    EatingStrategy::Plants => true,
                                }
                                && match unsafe { body.iq.unwrap_unchecked() } {
                                    1..7 => {
                                        if let Status::EscapingBody((
                                            chasing_body_id,
                                            chasing_body_type,
                                        )) = unsafe {
                                            bodies_shot_for_statuses
                                                .get_mut(other_body_id)
                                                .unwrap_unchecked()
                                        }
                                        .status
                                        {
                                            // if chasing_body_type == body.body_type {
                                            //     *body_id == chasing_body_id
                                            // } else {
                                            //     true
                                            // }
                                            chasing_body_type != body.body_type
                                                || *body_id == chasing_body_id
                                        } else {
                                            true
                                        }
                                    }
                                    _ => true,
                                }
                        })
                        .min_by(|(_, a), (_, b)| unsafe {
                            body.pos
                                .distance(a.pos)
                                .partial_cmp(&body.pos.distance(b.pos))
                                .unwrap_unchecked()
                        })
                    {
                        let distance_to_prey = body.pos.distance(prey.pos);
                        if distance_to_prey <= unsafe { body.speed.unwrap_unchecked() } {
                            body.energy = Some(
                                unsafe { body.energy.unwrap_unchecked() }
                                    + unsafe { prey.energy.unwrap_unchecked() },
                            );
                            body.pos = prey.pos;
                            removed_bodies.insert(**prey_id);
                        } else {
                            body.status = Status::FollowingTarget((**prey_id, prey.pos));
                            unsafe {
                                bodies_shot_for_statuses.get_mut(body_id).unwrap_unchecked()
                            }
                            .status = body.status;

                            body.pos.x += ((prey.pos.x - body.pos.x)
                                * unsafe { body.speed.unwrap_unchecked() })
                                / distance_to_prey;
                            body.pos.y += ((prey.pos.y - body.pos.y)
                                * unsafe { body.speed.unwrap_unchecked() })
                                / distance_to_prey;

                            continue;
                        }
                    }
                }
                EatingStrategy::Plants => {
                    if let Some((closest_plant_index, closest_plant)) = plants_shot
                        .iter()
                        .filter(|(plant_id, plant)| {
                            !removed_plants.contains(plant_id)
                                && body.pos.distance(plant.pos)
                                    <= unsafe { body.vision_distance.unwrap_unchecked() }
                                && match unsafe { body.iq.unwrap_unchecked() } {
                                    1..7 => bodies_within_vision_distance.iter().all(
                                        |(other_body_id, other_body)| {
                                            if other_body.body_type == body.body_type
                                                && *other_body_id != body_id
                                            {
                                                if let Status::FollowingTarget((
                                                    other_body_chasing_plant_id,
                                                    _,
                                                )) = unsafe {
                                                    bodies_shot_for_statuses
                                                        .get_mut(other_body_id)
                                                        .unwrap_unchecked()
                                                }
                                                .status
                                                {
                                                    other_body_chasing_plant_id != **plant_id
                                                } else {
                                                    true
                                                }
                                            } else {
                                                true
                                            }
                                        },
                                    ),
                                    _ => true,
                                }
                        })
                        .min_by(|(_, a), (_, b)| unsafe {
                            body.pos
                                .distance(a.pos)
                                .partial_cmp(&body.pos.distance(b.pos))
                                .unwrap_unchecked()
                        })
                    {
                        let distance_to_closest_plant = body.pos.distance(closest_plant.pos);
                        if distance_to_closest_plant <= unsafe { body.speed.unwrap_unchecked() } {
                            body.energy =
                                Some(unsafe { body.energy.unwrap_unchecked() } + PLANT_HP);
                            body.pos = closest_plant.pos;
                            removed_plants.insert(*closest_plant_index);
                        } else {
                            body.status =
                                Status::FollowingTarget((*closest_plant_index, closest_plant.pos));
                            unsafe {
                                bodies_shot_for_statuses.get_mut(body_id).unwrap_unchecked()
                            }
                            .status = body.status;

                            body.pos.x += ((closest_plant.pos.x - body.pos.x)
                                * unsafe { body.speed.unwrap_unchecked() })
                                / distance_to_closest_plant;
                            body.pos.y += ((closest_plant.pos.y - body.pos.y)
                                * unsafe { body.speed.unwrap_unchecked() })
                                / distance_to_closest_plant;

                            continue;
                        }
                    }
                }
            }

            // Procreate
            if body.energy > body.division_threshold {
                for _ in 0..2 {
                    new_bodies.insert(
                        Instant::now(),
                        Body::new(
                            Vec2 {
                                x: body.pos.x + OBJECT_RADIUS,
                                y: body.pos.y,
                            },
                            body.energy,
                            body.speed,
                            body.vision_distance,
                            body.eating_strategy,
                            body.division_threshold,
                            body.iq,
                            body.max_iq,
                            body.color,
                            rng,
                            body.body_type,
                        ),
                    );
                }

                removed_bodies.insert(*body_id);

                continue;
            }

            // Handle body-eaters walking & plant-eaters idle
            match body.eating_strategy {
                EatingStrategy::Bodies => {
                    if !matches!(body.status, Status::Walking(..)) {
                        let walking_angle: f32 = rng.gen_range(0.0..2.0 * PI);
                        let pos_deviation = Vec2 {
                            x: unsafe { body.speed.unwrap_unchecked() } * walking_angle.cos(),
                            y: unsafe { body.speed.unwrap_unchecked() } * walking_angle.sin(),
                        };

                        body.status = Status::Walking(pos_deviation);
                    }

                    if let Status::Walking(pos_deviation) = body.status {
                        body.pos.x += pos_deviation.x;
                        body.pos.y += pos_deviation.y;
                    }

                    body.wrap(area_size);
                }
                EatingStrategy::Plants => body.status = Status::Idle,
            }
        }

        for (new_body_id, new_body) in new_bodies {
            bodies.insert(new_body_id, new_body);
        }

        if is_draw_mode {
            if !is_draw_prevented {
                for (plant_id, plant) in &plants {
                    if !removed_plants.contains(plant_id) {
                        plant.draw();
                    }
                }

                for (body_id, body) in &bodies {
                    if !removed_bodies.contains(body_id) {
                        if zoom_mode && body.is_alive() {
                            if let Status::FollowingTarget((_, target_pos)) = body.status {
                                draw_line(
                                    body.pos.x,
                                    body.pos.y,
                                    target_pos.x,
                                    target_pos.y,
                                    2.0,
                                    WHITE,
                                );
                            }
                            if show_info {
                                draw_circle_lines(
                                    body.pos.x,
                                    body.pos.y,
                                    unsafe { body.vision_distance.unwrap_unchecked() },
                                    2.0,
                                    body.color,
                                );

                                let to_display = format!(
                                    "iq = {:?} \n max_iq = {:?} \n energy = {:?}",
                                    unsafe { body.iq.unwrap_unchecked() },
                                    unsafe { body.max_iq.unwrap_unchecked() },
                                    unsafe { body.energy.unwrap_unchecked() }.round()
                                );
                                draw_text(
                                    &to_display,
                                    body.pos.x
                                        - measure_text(&to_display, None, BODY_INFO_FONT_SIZE, 1.0)
                                            .width
                                            / 2.0,
                                    body.pos.y - OBJECT_RADIUS - MIN_GAP,
                                    BODY_INFO_FONT_SIZE as f32,
                                    WHITE,
                                );
                            }
                        }

                        body.draw();
                    }
                }

                // draw_text(
                //     &format!("Bodies alive {}", bodies.len()),
                //     10.0,
                //     20.0,
                //     20.0,
                //     WHITE,
                // );

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
            }

            next_frame().await;

            // Removing by a key takes too long, so it's better to do it once
            // but more rarely
            if removed_plants.len() > MIN_TO_REMOVE {
                for plant_id in &removed_plants {
                    plants.remove(plant_id);
                }
                removed_plants.clear();
            }

            if removed_bodies.len() > MIN_TO_REMOVE {
                for body_id in &removed_bodies {
                    bodies.remove(body_id);
                }
                removed_bodies.clear();
            }
        }
    }
    // }
}
