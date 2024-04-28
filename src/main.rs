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

enum FoodType {
    Body,
    Plant,
}

struct FoodInfo {
    id: Instant,
    food_type: FoodType,
    pos: Vec2,
    energy: f32,
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct CellPos {
    /// Row number (0..).
    i: usize,
    /// Column number (0..).
    j: usize,
}

#[derive(Default, Debug)]
pub struct Cells {
    rows: usize,
    columns: usize,
    cell_width: f32,
    cell_height: f32,
}

impl Cells {
    fn get_cell_by_pos(&self, pos: Vec2) -> CellPos {
        CellPos {
            i: (pos.y / self.cell_height) as usize,
            j: (pos.x / self.cell_width) as usize,
        }
    }
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

    let mut cells = Cells::default();

    cells.rows = CELL_ROWS;
    cells.columns = ((area_size.x * CELL_ROWS as f32) / area_size.y).round() as usize;
    cells.cell_width = area_size.x / cells.columns as f32;
    cells.cell_height = area_size.y / cells.rows as f32;

    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, area_size.x, area_size.y));
    default_camera(&mut camera, area_size);

    let rng = &mut StdRng::from_entropy();

    let mut zoom_mode = false;
    let mut show_info = true;

    // 'main_evolution_loop: loop {
    let mut bodies: HashMap<Instant, Body> = HashMap::with_capacity(BODIES_N);
    let mut plants: HashMap<CellPos, HashMap<Instant, Plant>> =
        HashMap::with_capacity(cells.rows * cells.columns);
    for i in 0..cells.rows {
        for j in 0..cells.columns {
            plants.insert(CellPos { i, j }, HashMap::new());
        }
    }

    let mut removed_plants: Vec<(Instant, Vec2)> = Vec::new();
    let mut removed_bodies: HashSet<Instant> = HashSet::with_capacity(bodies.len());

    // Spawn the bodies
    for i in 0..BODIES_N {
        randomly_spawn_body(
            &mut bodies,
            area_size,
            if rng.gen_range(0.0..1.0) <= PASSIVE_CHANCE {
                EatingStrategy::Passive
            } else {
                EatingStrategy::Active
            },
            rng,
            i + 1,
        );
    }

    // Spawn the plants
    for _ in 0..PLANTS_N {
        randomly_spawn_plant(&bodies, &mut plants, rng, area_size, &cells)
    }

    // The timer needed for the FPS
    let mut last_updated = Instant::now();

    loop {
        // Handle the left mouse button click for zooming in/out
        if unlikely(is_mouse_button_pressed(MouseButton::Left)) {
            if zoom_mode {
                default_camera(&mut camera, area_size);
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

        let mut only_plants: HashMap<&Instant, &Plant> = HashMap::new();
        for cell in plants.values() {
            for (plant_id, plant) in cell {
                only_plants.insert(plant_id, plant);
            }
        }

        // Remove plants
        let n_to_remove =
            ((only_plants.len() - removed_plants.len()) as f32 * PART_OF_PLANTS_TO_REMOVE) as usize;

        for _ in 0..n_to_remove {
            loop {
                let random_cell = plants.iter().choose(rng).unwrap().0;

                if let Some((random_plant_id, random_plant)) =
                    plants.get(random_cell).unwrap().iter().choose(rng)
                {
                    if !removed_plants.contains(&(*random_plant_id, random_plant.pos)) {
                        removed_plants.push((*random_plant_id, random_plant.pos));
                        // if random_plant.pos.x > area_size.x || random_plant.pos.y > area_size.y {
                        //     println!("{:?}", random_plant.pos);
                        // }
                        break;
                    }
                }
            }
        }

        // Spawn a plant in a random place with a specific chance
        for _ in 0..PLANTS_N_FOR_ONE_STEP {
            randomly_spawn_plant(&bodies, &mut plants, rng, area_size, &cells)
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
                    - CONST_FOR_LIFESPAN * body.speed.unwrap() * body.energy.unwrap())
                .max(0.0)
            }

            // Handle if dead to become a cross
            if body.energy.unwrap() < MIN_ENERGY || body_id.elapsed().as_secs_f32() > body.lifespan
            {
                body.status = Status::Dead(Instant::now());
                continue;
            }

            // Handle the energy
            // The mass is proportional to the energy; to keep the mass up, energy is spent
            body.energy = Some(
                body.energy.unwrap()
                    - ENERGY_SPENT_CONST_FOR_MASS * body.energy.unwrap()
                    - ENERGY_SPENT_CONST_FOR_IQ * body.iq.unwrap() as f32
                    - ENERGY_SPENT_CONST_FOR_VISION_DISTANCE
                        * body.vision_distance.unwrap().powi(2),
            );

            if body.status != Status::Idle {
                body.energy = Some(
                    body.energy.unwrap()
                        - ENERGY_SPENT_CONST_FOR_MOVEMENT
                            * body.speed.unwrap()
                            * body.energy.unwrap(),
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
                        && body.pos.distance(other_body.pos) <= body.vision_distance.unwrap()
                })
                .collect::<Vec<_>>();

            if let Some((closest_chasing_body_id, closest_chasing_body)) =
                bodies_within_vision_distance
                    .iter()
                    .filter(|(other_body_id, _)| {
                        if let Status::FollowingTarget(other_body_target) =
                            bodies_shot_for_statuses.get(other_body_id).unwrap().status
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
                bodies_shot_for_statuses.get_mut(body_id).unwrap().status = body.status;

                let distance_to_closest_chasing_body = body.pos.distance(closest_chasing_body.pos);

                body.pos.x -= ((closest_chasing_body.pos.x - body.pos.x) * body.speed.unwrap())
                    / distance_to_closest_chasing_body;
                body.pos.y -= ((closest_chasing_body.pos.y - body.pos.y) * body.speed.unwrap())
                    / distance_to_closest_chasing_body;

                body.wrap(area_size);

                continue;
            }

            // Eating
            let bodies_within_vision_distance_of_my_type = bodies_within_vision_distance
                .iter()
                .filter(|(_, other_body)| other_body.body_type == body.body_type)
                .collect::<Vec<_>>();

            let mut food: Option<FoodInfo> = None;

            // Find the closest cross
            match bodies_within_vision_distance
                .iter()
                .filter(|(cross_id, cross)| {
                    body.body_type != cross.body_type && !cross.is_alive() && {
                        let iq = body.iq.unwrap();

                        if (1..7).contains(&iq) {
                            bodies_within_vision_distance_of_my_type.iter().all(
                                |(other_body_id, _)| {
                                    bodies_shot_for_statuses.get(other_body_id).unwrap().status
                                        != Status::FollowingTarget((**cross_id, cross.pos))
                                },
                            )
                        } else {
                            true
                        }
                    }
                })
                .min_by(|(_, a), (_, b)| {
                    body.pos
                        .distance(a.pos)
                        .partial_cmp(&body.pos.distance(b.pos))
                        .unwrap()
                }) {
                Some(closest_cross) => {
                    food = Some(FoodInfo {
                        id: *closest_cross.0,
                        food_type: FoodType::Body,
                        pos: closest_cross.1.pos,
                        energy: closest_cross.1.energy.unwrap(),
                    })
                }
                None => {
                    // Find the closest plant
                    let mut only_plants: HashMap<&Instant, &Plant> = HashMap::new();
                    let body_cell = cells.get_cell_by_pos(body.pos);

                    for (plant_id, plant) in plants_shot.get(&body_cell).unwrap() {
                        only_plants.insert(plant_id, plant);
                    }
                    match only_plants
                        .iter()
                        .filter(|(plant_id, plant)| {
                            body.pos.distance(plant.pos) <= body.vision_distance.unwrap()
                                && !removed_plants.contains(&(***plant_id, plant.pos))
                                && {
                                    let iq = body.iq.unwrap();

                                    if (1..7).contains(&iq) {
                                        bodies_within_vision_distance_of_my_type.iter().all(
                                            |(other_body_id, _)| {
                                                bodies_shot_for_statuses
                                                    .get(other_body_id)
                                                    .unwrap()
                                                    .status
                                                    != Status::FollowingTarget((
                                                        ***plant_id,
                                                        plant.pos,
                                                    ))
                                            },
                                        )
                                    } else {
                                        true
                                    }
                                }
                        })
                        .min_by(|(_, a), (_, b)| {
                            body.pos
                                .distance(a.pos)
                                .partial_cmp(&body.pos.distance(b.pos))
                                .unwrap()
                        }) {
                        Some(closest_plant) => {
                            food = Some(FoodInfo {
                                id: **closest_plant.0,
                                food_type: FoodType::Plant,
                                pos: closest_plant.1.pos,
                                energy: PLANT_ENERGY,
                            })
                        }
                        None => {
                            // Find the closest body
                            if let Some(closest_body) = bodies_within_vision_distance
                                .iter()
                                .filter(|(other_body_id, other_body)| {
                                    body.body_type != other_body.body_type
                                        && body.energy > other_body.energy
                                        && other_body.is_alive()
                                        && {
                                            let iq = body.iq.unwrap();

                                            if (1..7).contains(&iq) {
                                                bodies_within_vision_distance_of_my_type.iter().all(
                                                    |(_, other_body)| {
                                                        other_body.status
                                                            != Status::FollowingTarget((
                                                                **other_body_id,
                                                                other_body.pos,
                                                            ))
                                                    },
                                                )
                                            } else {
                                                true
                                            }
                                        }
                                })
                                .min_by(|(_, a), (_, b)| {
                                    body.pos
                                        .distance(a.pos)
                                        .partial_cmp(&body.pos.distance(b.pos))
                                        .unwrap()
                                })
                            {
                                food = Some(FoodInfo {
                                    id: *closest_body.0,
                                    food_type: FoodType::Body,
                                    pos: closest_body.1.pos,
                                    energy: closest_body.1.energy.unwrap(),
                                })
                            }
                        }
                    }
                }
            }

            if let Some(food) = food {
                let distance_to_food = body.pos.distance(food.pos);
                if distance_to_food <= body.speed.unwrap() {
                    body.energy = Some(body.energy.unwrap() + food.energy);
                    body.pos = food.pos;

                    match food.food_type {
                        FoodType::Body => {
                            removed_bodies.insert(food.id);
                        }
                        FoodType::Plant => {
                            removed_plants.push((food.id, food.pos));

                            if food.pos.x > area_size.x || food.pos.y > area_size.y {
                                println!("{:?}", food.pos);
                            }
                        }
                    }
                } else {
                    body.status = Status::FollowingTarget((food.id, food.pos));
                    bodies_shot_for_statuses.get_mut(body_id).unwrap().status = body.status;

                    body.pos.x +=
                        ((food.pos.x - body.pos.x) * body.speed.unwrap()) / distance_to_food;
                    body.pos.y +=
                        ((food.pos.y - body.pos.y) * body.speed.unwrap()) / distance_to_food;

                    continue;
                }
            }

            // Procreate
            if body.energy > body.division_threshold {
                for _ in 0..2 {
                    new_bodies.insert(
                        Instant::now(),
                        Body::new(
                            body.pos,
                            body.energy,
                            body.speed,
                            body.vision_distance,
                            body.eating_strategy,
                            body.division_threshold,
                            body.iq,
                            body.max_iq,
                            body.color,
                            body.body_type,
                            rng,
                        ),
                    );
                }

                removed_bodies.insert(*body_id);

                continue;
            }

            // Handle body-eaters walking & plant-eaters idle
            match body.eating_strategy {
                EatingStrategy::Active => {
                    if !matches!(body.status, Status::Walking(..)) {
                        let walking_angle: f32 = rng.gen_range(0.0..2.0 * PI);
                        let pos_deviation = Vec2 {
                            x: body.speed.unwrap() * walking_angle.cos(),
                            y: body.speed.unwrap() * walking_angle.sin(),
                        };

                        body.status = Status::Walking(pos_deviation);
                    }

                    if let Status::Walking(pos_deviation) = body.status {
                        body.pos.x += pos_deviation.x;
                        body.pos.y += pos_deviation.y;
                    }

                    body.wrap(area_size);
                }
                EatingStrategy::Passive => body.status = Status::Idle,
            }
        }

        for (new_body_id, new_body) in new_bodies {
            bodies.insert(new_body_id, new_body);
        }

        if is_draw_mode {
            if !is_draw_prevented {
                for cell in plants.values() {
                    for (plant_id, plant) in cell {
                        if !removed_plants.contains(&(*plant_id, plant.pos)) {
                            plant.draw();
                        }
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
                                    body.vision_distance.unwrap(),
                                    2.0,
                                    body.color,
                                );

                                let to_display = format!(
                                    "iq = {:?} \n max_iq = {:?} \n energy = {:?}",
                                    body.iq.unwrap(),
                                    body.max_iq.unwrap(),
                                    body.energy.unwrap().round()
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
                for (plant_id, plant_pos) in &removed_plants {
                    // println!(
                    //     "{:?} {:?} {:?}",
                    //     &cells.get_cell_by_pos(*plant_pos),
                    //     plant_pos,
                    //     cells
                    // );
                    let plants_in_cell =
                        plants.get_mut(&cells.get_cell_by_pos(*plant_pos)).unwrap();
                    plants_in_cell.remove(plant_id);
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
