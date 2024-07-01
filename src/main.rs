#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(more_float_constants)]
#![feature(variant_count)]

mod body;
mod cells;
mod constants;
mod plant;
mod smart_drawing;
mod user_constants;
mod utils;
mod zoom;

use body::*;
use cells::{Cell, Cells};
use constants::*;
use plant::Plant;
use user_constants::*;
use zoom::{default_camera, get_zoom_target, Zoom};

use body::Skill;
use std::mem::size_of;
use std::{
    collections::{HashMap, HashSet},
    env::consts::OS,
    intrinsics::unlikely,
    process::exit,
    thread::sleep,
    time::{Duration, Instant},
};
use utils::*;

use macroquad::{
    camera::Camera2D,
    color::WHITE,
    input::{is_key_down, is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode},
    math::{Rect, Vec2},
    miniquad::{window::set_fullscreen, MouseButton},
    shapes::{draw_circle_lines, draw_line},
    text::{draw_text, measure_text},
    window::{next_frame, screen_height, screen_width, Conf},
};
use rand::{rngs::StdRng, seq::IteratorRandom, Rng, SeedableRng};

/// Adjust the coordinates according to the borders.
#[macro_export]
macro_rules! adjusted_pos {
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
        let part = $value * unsafe { DEVIATION };
        $rng.gen_range($value - part..$value + part)
    }};
}

enum FoodType {
    Body(HashMap<usize, f32>),
    Plant,
}

struct FoodInfo {
    id: Instant,
    food_type: FoodType,
    pos: Vec2,
    energy: f32,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "eportal".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

pub static mut TOTAL_SKILLS_COUNT: usize = 0;
pub static mut VIRUSES_COUNT: usize = 0;

#[macroquad::main(window_conf)]
async fn main() {
    config_setup();
    // Get all variants of enums (needed somewhere in the code)
    let (all_skills, all_viruses) = enum_consts();
    let ui_show_properties_n = (size_of::<UIField>() - size_of::<u16>()) / size_of::<bool>();

    // Workaround
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
    let area_space = area_size.x * area_size.y;

    unsafe {
        PLANTS_N = (PLANTS_DENSITY * area_space).round() as usize;
        PLANTS_N_FOR_ONE_STEP = (PLANT_SPAWN_CHANCE * area_space).round() as usize;
    }

    let area_size_ratio = area_size.x / area_size.y;

    cells.rows = (DEFAULT_CELL_ROWS as f32
        * (DEFAULT_AREA_SIZE_RATIO * unsafe { PLANTS_N } as f32
            / (area_size_ratio * DEFAULT_PLANTS_N as f32))
            .sqrt())
    .round() as usize;

    cells.columns = (cells.rows as f32 * area_size_ratio).round() as usize;
    cells.cell_width = area_size.x / cells.columns as f32;
    cells.cell_height = area_size.y / cells.rows as f32;

    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, area_size.x, area_size.y));
    default_camera(&mut camera, &area_size);

    let mut rng = StdRng::from_entropy();

    let mut zoom_mode = false; // Whether we're zoomed in
    let mut show_info = true; // Whether the info over the bodies has to be shown

    let mut bodies: HashMap<Instant, Body> = HashMap::with_capacity(unsafe { BODIES_N });
    let mut plants: HashMap<Cell, HashMap<Instant, Plant>> =
        HashMap::with_capacity(cells.rows * cells.columns);
    for i in 0..cells.rows {
        for j in 0..cells.columns {
            plants.insert(
                Cell { i, j },
                HashMap::with_capacity(MAX_PLANTS_IN_ONE_CELL),
            );
        }
    }

    let mut removed_plants: Vec<(Instant, Vec2)> = Vec::new();
    let mut removed_bodies: HashSet<Instant> = HashSet::with_capacity(bodies.len());

    // Spawn the bodies
    for i in 0..unsafe { BODIES_N } {
        Body::randomly_spawn_body(
            &mut bodies,
            &area_size,
            if rng.gen_range(0.0..1.0) <= unsafe { PASSIVE_CHANCE } {
                EatingStrategy::Passive
            } else {
                EatingStrategy::Active
            },
            i + 1,
            &all_skills,
            &all_viruses,
            &mut rng,
        );
    }

    // Needs to be handled manually to avoid extracting all plants out of the cells
    let mut plants_n = 0;

    // Spawn the plants
    for _ in 0..unsafe { PLANTS_N } {
        Plant::randomly_spawn_plant(&bodies, &mut plants, &area_size, &cells, &mut rng);
        plants_n += 1;
    }

    // The timer needed for the FPS
    let mut last_updated = Instant::now();

    let scaling_width = MAX_ZOOM / area_size.x * 2.0;
    let scaling_height = MAX_ZOOM / area_size.y * 2.0;
    let rect_width = screen_width() / MAX_ZOOM * OBJECT_RADIUS;
    let rect_height = screen_height() / MAX_ZOOM * OBJECT_RADIUS;

    let extended_rect_width = rect_width + OBJECT_RADIUS * 2.0;
    let extended_rect_height = rect_height + OBJECT_RADIUS * 2.0;

    // All the info about the zoom
    let mut zoom = Zoom {
        scaling_width,
        scaling_height,
        width: rect_width,
        height: rect_height,
        center_pos: None,
        mouse_pos: None,
        rect: None,
        extended_rect: None,
        diagonal_rect: (rect_width.powi(2) + rect_height.powi(2)).sqrt(),
        diagonal_extended_rect: (extended_rect_width.powi(2) + extended_rect_height.powi(2)).sqrt(),
    };

    loop {
        // Handle interactions
        if unlikely(is_key_pressed(KeyCode::Escape)) {
            exit(0);
        }

        if unlikely(is_mouse_button_pressed(MouseButton::Left)) {
            if zoom_mode {
                default_camera(&mut camera, &area_size);
                zoom.mouse_pos = None;
            } else {
                zoom.rect = None;
                zoom.extended_rect = None;
                zoom.rect = None;
            }

            zoom_mode = !zoom_mode
        }

        if unlikely(is_key_pressed(KeyCode::Key1)) {
            show_info = !show_info
        }

        let is_draw_prevented = is_key_down(KeyCode::Space);

        if zoom_mode {
            // There's no reason to zoom in again if the mouse position hasn't been changed
            let current_mouse_pos = Vec2::from(mouse_position());
            match zoom.mouse_pos {
                Some(mouse_pos) => {
                    if mouse_pos != current_mouse_pos {
                        zoom.mouse_pos = Some(current_mouse_pos);
                        get_zoom_target(&mut camera, &area_size, &mut zoom);
                    }
                }
                None => {
                    zoom.mouse_pos = Some(current_mouse_pos);
                    get_zoom_target(&mut camera, &area_size, &mut zoom);
                }
            }
        }

        // Remove plants
        let n_to_remove = (plants_n as f32 * unsafe { PLANT_DIE_CHANCE }) as usize;

        for _ in 0..n_to_remove {
            loop {
                // Pick a random cell and remove a random plant from it
                let random_cell = plants.iter().choose(&mut rng).unwrap().0;

                if let Some((random_plant_id, random_plant)) =
                    plants.get(random_cell).unwrap().iter().choose(&mut rng)
                {
                    if !removed_plants.contains(&(*random_plant_id, random_plant.pos)) {
                        removed_plants.push((*random_plant_id, random_plant.pos));
                        plants_n -= 1;
                        break;
                    }
                }
            }
        }

        // Spawn a plant in a random place with a specific chance
        for _ in 0..unsafe { PLANTS_N_FOR_ONE_STEP } {
            Plant::randomly_spawn_plant(&bodies, &mut plants, &area_size, &cells, &mut rng);
            plants_n += 1;
        }

        // Whether enough time has passed to draw a new frame
        let is_draw_mode =
            last_updated.elapsed().as_millis() >= Duration::from_secs(1 / FPS).as_millis();

        // Due to certain borrowing rules, it's impossible to modify these during the loop,
        // so it'll be done after it
        let mut new_bodies: HashMap<Instant, Body> =
            HashMap::with_capacity((bodies.len() - removed_bodies.len()) * 2);
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
                if death_time.elapsed().as_secs() >= unsafe { CROSS_LIFESPAN } {
                    removed_bodies.insert(*body_id);
                }
                continue;
            }

            body.handle_viruses();
            body.handle_lifespan();

            // Handle if dead to become a cross
            if body.energy < unsafe { MIN_ENERGY }
                || body_id.elapsed().as_secs_f32() > body.lifespan
            {
                body.status = Status::Dead(Instant::now());
                continue;
            }

            if body.handle_energy(body_id, &mut removed_bodies) {
                continue;
            }

            // Escape
            let bodies_within_vision_distance = bodies_shot
                .iter()
                .filter(|(other_body_id, other_body)| {
                    &body_id != other_body_id
                        && !removed_bodies.contains(other_body_id)
                        && body.pos.distance(other_body.pos) <= body.vision_distance
                })
                .collect::<Vec<_>>();

            if let Some((closest_chasing_body_id, closest_chasing_body)) = {
                let mut chasers = bodies_within_vision_distance
                    .iter()
                    .filter(|(other_body_id, _)| {
                        if let Status::FollowingTarget(other_body_target) =
                            bodies_shot_for_statuses.get(other_body_id).unwrap().status
                        {
                            &other_body_target.0 == body_id
                        } else {
                            false
                        }
                    })
                    .collect::<Vec<_>>();

                if body
                    .skills
                    .contains(&(Skill::PrioritizeFasterChasers as usize))
                    && chasers
                        .iter()
                        .any(|(_, other_body)| other_body.speed > body.speed)
                {
                    chasers.retain(|(_, other_body)| other_body.speed > body.speed)
                }

                chasers
            }
            .iter()
            .min_by(|(_, a), (_, b)| {
                body.pos
                    .distance(a.pos)
                    .total_cmp(&body.pos.distance(b.pos))
            }) {
                body.status = Status::EscapingBody((
                    **closest_chasing_body_id,
                    closest_chasing_body.body_type,
                ));
                bodies_shot_for_statuses.get_mut(body_id).unwrap().status = body.status;

                let distance_to_closest_chasing_body = body.pos.distance(closest_chasing_body.pos);

                body.pos.x -= (closest_chasing_body.pos.x - body.pos.x)
                    * (body.speed / distance_to_closest_chasing_body);
                body.pos.y -= (closest_chasing_body.pos.y - body.pos.y)
                    * (body.speed / distance_to_closest_chasing_body);

                body.wrap(&area_size);

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
                    !cross.is_alive()
                        && body.handle_avoid_new_viruses(cross)
                        && body.handle_eat_crosses_of_my_type(cross)
                        && body.handle_do_not_complete_with_relatives(
                            cross_id,
                            &cross.pos,
                            &bodies_shot_for_statuses,
                            &bodies_within_vision_distance_of_my_type,
                        )
                        && body.handle_alive_when_arrived_body(cross, true)
                        && body.handle_profitable_when_arrived_body(cross, true)
                        && body.handle_will_arive_first_body(
                            cross_id,
                            cross,
                            &bodies_within_vision_distance,
                        )
                })
                .min_by(|(_, a), (_, b)| {
                    body.pos
                        .distance(a.pos)
                        .partial_cmp(&body.pos.distance(b.pos))
                        .unwrap()
                }) {
                Some((closest_cross_id, closest_cross)) => {
                    food = Some(FoodInfo {
                        id: **closest_cross_id,
                        food_type: FoodType::Body(closest_cross.viruses.clone()),
                        pos: closest_cross.pos,
                        energy: closest_cross.energy,
                    })
                }
                None => {
                    // Find the closest plant
                    let mut visible_plants: HashMap<&Instant, &Plant> = HashMap::new();

                    // Using these for ease of development
                    let (a, b) = (body.pos.x, body.pos.y);
                    let r = body.vision_distance;
                    let (w, h) = (cells.cell_width, cells.cell_height);
                    let (m, n) = (cells.columns, cells.rows);

                    // Get the bottommost, topmost, leftmost, and rightmost rows/columns
                    let i_min = ((b - r) / h).floor().max(0.0) as usize;
                    let i_max = ((b + r) / h).floor().min(n as f32 - 1.0) as usize;
                    let j_min = ((a - r) / w).floor().max(0.0) as usize;
                    let j_max = ((a + r) / w).floor().min(m as f32 - 1.0) as usize;

                    // Get the row going through the center of the body
                    let body_i = cells.get_cell_by_pos(&body.pos).i;

                    let (
                        (
                            // Get the min/max j we have to care about for i
                            mut j_min_for_i,
                            mut j_max_for_i,
                        ),
                        mut i_for_line,
                        mut delta,
                    );

                    for i in i_min..=i_max {
                        if i == body_i {
                            (j_min_for_i, j_max_for_i) = (j_min, j_max);
                        } else {
                            i_for_line = if i < body_i { i + 1 } else { i };

                            delta = r * (1.0 - ((i_for_line as f32 * h - b) / r).powi(2)).sqrt();
                            (j_min_for_i, j_max_for_i) = (
                                ((a - delta) / w).floor().max(0.0) as usize,
                                ((a + delta) / w).floor().min(m as f32 - 1.0) as usize,
                            )
                        }

                        for j in j_min_for_i..=j_max_for_i {
                            // Center of the cell
                            let (center_x, center_y) =
                                (j as f32 * w + w / 2.0, i as f32 * h + h / 2.0);

                            // true as usize = 1
                            // false as usize = 0
                            let (i_adjustment, j_adjustment) =
                                ((center_y > b) as usize, (center_x > a) as usize);

                            let fully_covered = (((j + j_adjustment) as f32) * w - a).powi(2)
                                + (((i + i_adjustment) as f32) * h - b).powi(2)
                                < r.powi(2);

                            for (plant_id, plant) in plants_shot.get(&Cell { i, j }).unwrap() {
                                if fully_covered
                                    || body.pos.distance(plant.pos) <= body.vision_distance
                                {
                                    visible_plants.insert(plant_id, plant);
                                }
                            }
                        }

                        match visible_plants
                            .iter()
                            .filter(|(plant_id, plant)| {
                                !removed_plants.contains(&(***plant_id, plant.pos))
                                    && body.handle_alive_when_arrived_plant(plant)
                                    && body.handle_do_not_complete_with_relatives(
                                        plant_id,
                                        &plant.pos,
                                        &bodies_shot_for_statuses,
                                        &bodies_within_vision_distance_of_my_type,
                                    )
                                    && body.handle_profitable_when_arrived_plant(plant)
                                    && body.handle_will_arive_first_plant(
                                        plant_id,
                                        plant,
                                        &bodies_within_vision_distance,
                                    )
                            })
                            .min_by(|(_, a), (_, b)| {
                                body.pos
                                    .distance(a.pos)
                                    .partial_cmp(&body.pos.distance(b.pos))
                                    .unwrap()
                            }) {
                            Some((closest_plant_id, closest_plant)) => {
                                food = Some(FoodInfo {
                                    id: **closest_plant_id,
                                    food_type: FoodType::Plant,
                                    pos: closest_plant.pos,
                                    energy: PLANT_ENERGY,
                                })
                            }
                            None => {
                                // Find the closest body
                                if let Some((closest_body_id, closest_body)) =
                                    bodies_within_vision_distance
                                        .iter()
                                        .filter(|(other_body_id, other_body)| {
                                            body.body_type != other_body.body_type
                                                && body.energy > other_body.energy
                                                && other_body.is_alive()
                                                && body.handle_avoid_new_viruses(other_body)
                                                && body.handle_do_not_complete_with_relatives(
                                                    other_body_id,
                                                    &other_body.pos,
                                                    &bodies_shot_for_statuses,
                                                    &bodies_within_vision_distance_of_my_type,
                                                )
                                                && body.handle_alive_when_arrived_body(
                                                    other_body, false,
                                                )
                                                && body.handle_profitable_when_arrived_body(
                                                    other_body, false,
                                                )
                                                && body.handle_will_arive_first_body(
                                                    other_body_id,
                                                    other_body,
                                                    &bodies_within_vision_distance,
                                                )
                                        })
                                        .min_by(|(_, a), (_, b)| {
                                            body.pos
                                                .distance(a.pos)
                                                .partial_cmp(&body.pos.distance(b.pos))
                                                .unwrap()
                                        })
                                {
                                    food = Some(FoodInfo {
                                        id: **closest_body_id,
                                        food_type: FoodType::Body(closest_body.viruses.clone()),
                                        pos: closest_body.pos,
                                        energy: closest_body.energy,
                                    })
                                }
                            }
                        }
                    }
                }
            }

            if let Some(food) = food {
                let distance_to_food = body.pos.distance(food.pos);
                if distance_to_food <= body.speed {
                    body.energy += food.energy;
                    body.pos = food.pos;

                    match food.food_type {
                        FoodType::Body(viruses) => {
                            body.get_viruses(&viruses);
                            removed_bodies.insert(food.id);
                        }
                        FoodType::Plant => {
                            removed_plants.push((food.id, food.pos));
                            plants_n -= 1;
                        }
                    }
                } else {
                    body.status = Status::FollowingTarget((food.id, food.pos));
                    bodies_shot_for_statuses.get_mut(body_id).unwrap().status = body.status;

                    body.pos.x += (food.pos.x - body.pos.x) * (body.speed / distance_to_food);
                    body.pos.y += (food.pos.y - body.pos.y) * (body.speed / distance_to_food);

                    continue;
                }
            }

            // Procreate
            if body.handle_procreation(
                body_id,
                &mut new_bodies,
                &mut removed_bodies,
                &all_skills,
                &all_viruses,
                &mut rng,
            ) {
                continue;
            }

            body.handle_walking_idle(&area_size, &mut rng);
        }

        // average_performance += timestamp.elapsed().as_nanos() as f32 / (bodies.len() - removed_bodies.len()).pow(2) as f32;

        for (new_body_id, new_body) in new_bodies {
            bodies.insert(new_body_id, new_body);
        }

        if is_draw_mode {
            if !is_draw_prevented {
                if zoom_mode {
                    for plant in Plant::get_plants_to_draw(&cells, &zoom, &plants, &removed_plants)
                    {
                        plant.draw();
                    }

                    for (body_id, body) in &bodies {
                        if !removed_bodies.contains(body_id) {
                            let drawing_strategy = body.get_drawing_strategy(&zoom);

                            if show_info {
                                if drawing_strategy.vision_distance {
                                    draw_circle_lines(
                                        body.pos.x,
                                        body.pos.y,
                                        body.vision_distance,
                                        2.0,
                                        body.color,
                                    );
                                }

                                if drawing_strategy.target_line {
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
                                }

                                if body.is_alive() {
                                    let mut to_display_components =
                                        Vec::with_capacity(ui_show_properties_n);

                                    if unsafe { SHOW_ENERGY } {
                                        to_display_components
                                            .push(format!("energy = {}", body.energy as usize));
                                    }

                                    if unsafe { SHOW_DIVISION_THRESHOLD } {
                                        to_display_components.push(format!(
                                            "dt = {}",
                                            body.division_threshold as usize
                                        ));
                                    }

                                    if unsafe { SHOW_BODY_TYPE } {
                                        to_display_components
                                            .push(format!("body_type = {}", body.body_type));
                                    }

                                    if unsafe { SHOW_LIFESPAN } {
                                        to_display_components
                                            .push(format!("lifespan = {}", body.lifespan as usize));
                                    }

                                    if unsafe { SHOW_SKILLS } {
                                        to_display_components
                                            .push(format!("skills = {:?}", body.skills));
                                    }

                                    if unsafe { SHOW_VIRUSES } {
                                        to_display_components
                                            .push(format!("viruses = {:?}", body.viruses.keys()));
                                    }

                                    if !to_display_components.is_empty() {
                                        let to_display = to_display_components.join(" | ");
                                        draw_text(
                                            &to_display,
                                            body.pos.x
                                                - measure_text(
                                                    &to_display,
                                                    None,
                                                    unsafe { BODY_INFO_FONT_SIZE },
                                                    1.0,
                                                )
                                                .width
                                                    / 2.0,
                                            body.pos.y - OBJECT_RADIUS - MIN_GAP,
                                            unsafe { BODY_INFO_FONT_SIZE } as f32,
                                            WHITE,
                                        );
                                    }
                                }
                            }

                            if drawing_strategy.body {
                                body.draw(&zoom, zoom_mode);
                            }
                        }
                    }
                } else {
                    for cell in plants.values() {
                        for (plant_id, plant) in cell {
                            if !removed_plants.contains(&(*plant_id, plant.pos)) {
                                plant.draw();
                            }
                        }
                    }

                    for (body_id, body) in &bodies {
                        if !removed_bodies.contains(body_id) {
                            body.draw(&zoom, zoom_mode);
                        }
                    }
                }

                last_updated = Instant::now();
            }

            next_frame().await;

            for (plant_id, plant_pos) in &removed_plants {
                plants
                    .get_mut(&cells.get_cell_by_pos(plant_pos))
                    .unwrap()
                    .remove(plant_id);
            }
            removed_plants.clear();

            // If all the bodies that need to be removed were removed evenly throughout the program,
            // reallocations would happen constantly. It turns out it's nicer
            // when it's done more rarely but all at once.
            if removed_bodies.len() > MIN_TO_REMOVE {
                for body_id in &removed_bodies {
                    bodies.remove(body_id);
                }
                removed_bodies.clear();
            }
        }
    }
}
