#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(more_float_constants)]
#![feature(variant_count)]

mod body;
mod cells;
mod condition;
mod constants;
mod plant;
mod smart_drawing;
mod user_constants;
mod utils;
mod zoom;

use body::*;
use cells::*;
use condition::*;
use constants::*;
use plant::*;
use user_constants::*;
use utils::*;
use zoom::*;

use std::{
    collections::{HashMap, HashSet},
    intrinsics::unlikely,
    mem::variant_count,
    process::exit,
    thread::sleep,
    time::{Duration, Instant},
};

use macroquad::{
    camera::Camera2D,
    color::WHITE,
    input::{
        is_key_down, is_key_pressed, is_mouse_button_pressed,
        mouse_position, KeyCode,
    },
    math::{Rect, Vec2},
    miniquad::{window::set_fullscreen, MouseButton},
    prelude::vec2,
    shapes::{draw_circle_lines, draw_line},
    window::{next_frame, screen_height, screen_width, Conf},
};
use rand::{rngs::StdRng, seq::IteratorRandom, Rng, SeedableRng};

pub static mut TOTAL_SKILLS_COUNT: usize = 0;
pub static mut VIRUSES_COUNT: usize = 0;
pub static mut UI_SHOW_PROPERTIES_N: usize = 0;

fn window_conf() -> Conf {
    Conf {
        window_title: "eportal".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    assert_eq!(Condition::ALL.len(), variant_count::<Condition>());
    assert_eq!(Virus::ALL.len(), variant_count::<Virus>());
    assert_eq!(Skill::ALL.len(), variant_count::<Skill>());
    assert_eq!(PlantKind::ALL.len(), variant_count::<PlantKind>());

    config_setup();

    // A workaround for Linux
    if cfg!(target_os = "linux") {
        set_fullscreen(true);
        sleep(Duration::from_secs(1));
        next_frame().await;
    }

    let area_size = vec2(
        // OBJECT_RADIUS is equal to one pixel when unzoomed
        screen_width() * OBJECT_RADIUS,
        screen_height() * OBJECT_RADIUS,
    );

    // Needed for randomness
    let mut rng = StdRng::from_rng(&mut rand::thread_rng()).unwrap();

    // Calculations
    let mut plants_cells = Cells::default();
    let area_space = area_size.x * area_size.y;

    unsafe {
        PLANTS_N = (PLANTS_DENSITY * area_space).round() as usize;
        PLANTS_N_FOR_ONE_STEP =
            (PLANT_SPAWN_CHANCE * area_space).round() as usize;
    }

    let area_size_ratio = area_size.x / area_size.y;

    // Get `k` out of PLANTS_N/k = DEFAULT_PLANTS/p
    // where `k` is the real number of cells
    // and `p` is the default number of cells.
    plants_cells.rows = ((DEFAULT_CELL_ROWS as f32
        * (DEFAULT_AREA_SIZE_RATIO * unsafe { PLANTS_N } as f32
            / (area_size_ratio * DEFAULT_PLANTS_N as f32))
            .sqrt())
    .round() as usize)
        .clamp(50, 200);
    plants_cells.columns =
        (plants_cells.rows as f32 * area_size_ratio).round() as usize;

    plants_cells.cell_width = area_size.x / plants_cells.columns as f32;
    plants_cells.cell_height = area_size.y / plants_cells.rows as f32;

    let mut bodies_cells = Cells::default();
    
    bodies_cells.rows = 

    // Camera
    let mut camera = Camera2D::from_display_rect(Rect::new(
        0.0,
        0.0,
        area_size.x,
        area_size.y,
    ));

    default_camera(&mut camera, &area_size);

    // Info
    let mut info = Info {
        body_info:      true,
        evolution_info: EvolutionInfo {
            show:         false,
            last_updated: None,
            last_info:    None,
        },
    };

    // Evolution stuff
    let mut condition: Option<(Condition, (Instant, Duration))> =
        None;

    let mut bodies: HashMap<Cell, HashMap<Instant, Body>> =
        HashMap::with_capacity(unsafe { BODIES_N });
    let mut plants: HashMap<Cell, HashMap<Instant, Plant>> =
        HashMap::with_capacity(plants_cells.rows * plants_cells.columns);

    for i in 0..plants_cells.rows {
        for j in 0..plants_cells.columns {
            plants.insert(
                Cell { i, j },
                HashMap::with_capacity(
                    AVERAGE_MAX_PLANTS_IN_ONE_CELL,
                ),
            );
        }
    }

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
            &mut rng,
        );
    }

    // Needs to be handled manually to avoid extracting all plants out of the cells
    let mut plants_n = 0;

    // Spawn the plants
    for _ in 0..unsafe { PLANTS_N } {
        Plant::randomly_spawn_plant(
            &bodies,
            &mut plants,
            &area_size,
            &plants_cells,
            &mut rng,
        );

        plants_n += 1;
    }

    // Zoom
    let rect_size = vec2(
        screen_width() / MAX_ZOOM * OBJECT_RADIUS,
        screen_height() / MAX_ZOOM * OBJECT_RADIUS,
    );

    let mut zoom = generate_zoom_struct(&area_size);

    // Needed for the FPS
    let mut last_updated = Instant::now();

    loop {
        // Handle interactions
        if unlikely(is_key_pressed(KeyCode::Escape)) {
            exit(0);
        }

        if unlikely(is_mouse_button_pressed(MouseButton::Left)) {
            if zoom.zoomed {
                default_camera(&mut camera, &area_size);
                zoom.mouse_pos = None;
            } else {
                zoom.rect = None;
                zoom.extended_rect = None;
                zoom.rect = None;
            }

            zoom.zoomed = !zoom.zoomed
        }

        if unlikely(is_key_pressed(KeyCode::Key1)) {
            info.body_info = !info.body_info
        }

        if unlikely(is_key_pressed(KeyCode::Key2)) {
            info.evolution_info.show = !info.evolution_info.show;
            info.evolution_info.last_updated = Some(Instant::now());
        }

        if zoom.zoomed {
            // There's no reason to zoom in again if the mouse position hasn't been changed
            let current_mouse_pos = Vec2::from(mouse_position());
            match zoom.mouse_pos {
                Some(mouse_pos) => {
                    if mouse_pos != current_mouse_pos {
                        zoom.mouse_pos = Some(current_mouse_pos);
                        get_zoom_target(
                            &mut camera,
                            &area_size,
                            &mut zoom,
                            &rect_size,
                        );
                    }
                }
                None => {
                    zoom.mouse_pos = Some(current_mouse_pos);
                    get_zoom_target(
                        &mut camera,
                        &area_size,
                        &mut zoom,
                        &rect_size,
                    );
                }
            }
        }

        let mut new_bodies: HashMap<Instant, Body> =
            HashMap::with_capacity(AVERAGE_MAX_NEW_BODIES);

        let mut removed_plants: HashMap<Instant, Vec2> =
            HashMap::with_capacity(AVERAGE_MAX_PLANTS_REMOVED);
        let mut removed_bodies: HashSet<Instant> =
            HashSet::with_capacity(AVERAGE_MAX_BODIES_REMOVED);

        update_condition(&mut condition, &mut rng);

        // Remove plants
        let n_to_remove = (plants_n as f32
            * (unsafe { PLANT_DIE_CHANCE }
                + if condition.is_some_and(|(condition, _)| {
                    condition == Condition::Drought
                }) {
                    (unsafe { PLANT_DIE_CHANCE })
                        * DROUGHT_PLANT_DIE_CHANCE_MULTIPLIER
                } else {
                    0.0
                })) as usize;

        for _ in 0..n_to_remove {
            loop {
                // Pick a random cell and remove a random plant from it
                let random_cell =
                    plants.iter().choose(&mut rng).unwrap().0;

                if let Some((random_plant_id, random_plant)) = plants
                    .get(random_cell)
                    .unwrap()
                    .iter()
                    .choose(&mut rng)
                {
                    if !removed_plants.contains_key(random_plant_id) {
                        removed_plants.insert(
                            *random_plant_id,
                            random_plant.pos,
                        );
                        plants_n -= 1;
                        break;
                    }
                }
            }
        }

        // Spawn a plant in a random place with a specific chance
        let n_to_add = unsafe { PLANTS_N_FOR_ONE_STEP }
            + if condition.is_some_and(|(condition, _)| {
                condition == Condition::Rain
            }) {
                (unsafe { PLANTS_N_FOR_ONE_STEP } as f32
                    * RAIN_PLANTS_N_FOR_ONE_STEP_MULTIPLIER)
                    as usize
            } else {
                0
            };

        for _ in 0..n_to_add {
            Plant::randomly_spawn_plant(
                &bodies,
                &mut plants,
                &area_size,
                &plants_cells,
                &mut rng,
            );

            plants_n += 1;
        }

        // Whether enough time has passed to draw a new frame
        let is_draw_mode = last_updated.elapsed().as_millis()
            >= Duration::from_secs(1 / FPS).as_millis();

        let bodies_mut = unsafe {
            &mut (*(&mut bodies as *mut HashMap<Cell, HashMap<Instant, Body>>))
        };

        let bodies_const =
            unsafe { &(*(&bodies as *const HashMap<Cell, HashMap<Instant, Body>>)) };

        for (body_id, body) in bodies_mut {
            // Handle if the body was eaten earlier
            if removed_bodies.contains(body_id) {
                continue;
            }

            // Handle if completely dead
            if let Status::Dead(death_time) = body.status {
                if death_time.elapsed().as_secs()
                    >= unsafe { CROSS_LIFESPAN }
                {
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
            let bodies_within_vision_distance = bodies_const
                .iter()
                .filter(|(other_body_id, other_body)| {
                    &body_id != other_body_id
                        && body.pos.distance(other_body.pos)
                            <= body.vision_distance
                        && !removed_bodies.contains(other_body_id)
                })
                .collect::<Vec<_>>();

            let mut chasers = bodies_within_vision_distance
                .iter()
                .filter(|(other_body_id, _)| {
                    if let Status::FollowingTarget(
                        other_body_target_id,
                        _,
                    ) = bodies.get(other_body_id).unwrap().status
                    {
                        &other_body_target_id == body_id
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>();

            if body.skills.contains(&Skill::PrioritizeFasterChasers)
                && chasers.iter().any(|(_, other_body)| {
                    other_body.speed > body.speed
                })
            {
                chasers.retain(|(_, other_body)| {
                    other_body.speed > body.speed
                })
            }

            if let Some((
                closest_chasing_body_id,
                closest_chasing_body,
            )) = chasers.iter().min_by(|(_, a), (_, b)| {
                body.pos
                    .distance(a.pos)
                    .total_cmp(&body.pos.distance(b.pos))
            }) {
                body.status = Status::EscapingBody(
                    **closest_chasing_body_id,
                    closest_chasing_body.body_type,
                );

                let distance_to_closest_chasing_body =
                    body.pos.distance(closest_chasing_body.pos);

                body.pos.x -= (closest_chasing_body.pos.x
                    - body.pos.x)
                    * (body.speed / distance_to_closest_chasing_body);
                body.pos.y -= (closest_chasing_body.pos.y
                    - body.pos.y)
                    * (body.speed / distance_to_closest_chasing_body);

                body.wrap(&area_size);

                continue;
            }

            // Eating
            let bodies_within_vision_distance_of_my_type =
                bodies_within_vision_distance
                    .iter()
                    .filter(|(_, other_body)| {
                        other_body.body_type == body.body_type
                    })
                    .collect::<Vec<_>>();

            let mut food: Option<FoodInfo> = None;

            // Find the closest cross
            match bodies_within_vision_distance
                .iter()
                .filter(|(cross_id, cross)| {
                    !cross.is_alive()
                        && body.handle_eat_crosses_of_my_type(cross)
                        && body.handle_alive_when_arrived_body(
                            cross, true,
                        )
                        && body.handle_profitable_when_arrived_body(
                            cross, true,
                        )
                        && body.handle_avoid_new_viruses(cross)
                        && body.handle_will_arive_first_body(
                            cross_id,
                            cross,
                            &bodies_within_vision_distance,
                        )
                        && body.handle_do_not_complete_with_relatives(
                            cross_id,
                            &cross.pos,
                            &bodies,
                            &bodies_within_vision_distance_of_my_type,
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
                        id:        **closest_cross_id,
                        food_type: FoodType::Body(
                            closest_cross.viruses.clone(),
                        ),
                        pos:       closest_cross.pos,
                        energy:    closest_cross.energy,
                    })
                }
                None => {
                    // Find the closest plant
                    let mut visible_plants: HashMap<
                        &Instant,
                        &Plant,
                    > = HashMap::with_capacity(
                        (plants_n as f32
                            * AVERAGE_PLANTS_PART_VISIBLE)
                            as usize,
                    );

                    // Using these for ease of development
                    let (a, b) = (body.pos.x, body.pos.y);
                    let r = body.vision_distance;
                    let (w, h) =
                        (plants_cells.cell_width, plants_cells.cell_height);
                    let (m, n) = (plants_cells.columns, plants_cells.rows);

                    // Get the bottommost, topmost, leftmost, and rightmost rows/columns.
                    // If the cell is within the circle or the circle touches the cell, it is
                    // within the rectangle around the circle. Some of those cells are unneeded.
                    let i_min =
                        ((b - r) / h).floor().max(0.0) as usize;
                    let i_max =
                        ((b + r) / h).floor().min(n as f32 - 1.0)
                            as usize;
                    let j_min =
                        ((a - r) / w).floor().max(0.0) as usize;
                    let j_max =
                        ((a + r) / w).floor().min(m as f32 - 1.0)
                            as usize;

                    // Ditch the unneeded cells
                    let Cell {
                        i: circle_center_i, ..
                    } = plants_cells.get_cell_by_pos(&body.pos);

                    for i in i_min..=i_max {
                        let (
                            // Get the min/max j we have to care about for i
                            j_min_for_i,
                            j_max_for_i,
                        );

                        if i == circle_center_i {
                            (j_min_for_i, j_max_for_i) =
                                (j_min, j_max);
                        } else {
                            let i_for_line = if i < circle_center_i {
                                i + 1
                            } else {
                                i
                            };

                            let delta = r
                                * (1.0
                                    - ((i_for_line as f32 * h - b)
                                        / r)
                                        .powi(2))
                                .sqrt();
                            (j_min_for_i, j_max_for_i) = (
                                ((a - delta) / w).floor().max(0.0)
                                    as usize,
                                ((a + delta) / w)
                                    .floor()
                                    .min((m - 1) as f32)
                                    as usize,
                            )
                        }

                        for j in j_min_for_i..=j_max_for_i {
                            // Center of the cell
                            let (center_x, center_y) = (
                                j as f32 * w + w / 2.0,
                                i as f32 * h + h / 2.0,
                            );

                            // true as usize = 1
                            // false as usize = 0
                            let (i_delta, j_delta) = (
                                (center_y > b) as usize, // If the cell is in the 1st or 2nd quadrant
                                (center_x > a) as usize, // If the cell is in the 1st or 4th quadrant
                            );

                            let fully_covered =
                                (((j + j_delta) as f32) * w - a)
                                    .powi(2)
                                    + (((i + i_delta) as f32) * h
                                        - b)
                                        .powi(2)
                                    < r.powi(2);

                            for (plant_id, plant) in
                                plants.get(&Cell { i, j }).unwrap()
                            {
                                if fully_covered
                                    || body.pos.distance(plant.pos)
                                        <= body.vision_distance
                                {
                                    visible_plants
                                        .insert(plant_id, plant);
                                }
                            }
                        }
                    }

                    let filtered_visible_plants = visible_plants
                        .iter()
                        .filter(|(plant_id, plant)| {
                            !removed_plants.contains_key(plant_id)
                            && body.handle_alive_when_arrived_plant(plant)
                            && body.handle_profitable_when_arrived_plant(plant)
                            && body.handle_do_not_complete_with_relatives(
                                plant_id,
                                &plant.pos,
                                &bodies,
                                &bodies_within_vision_distance_of_my_type,
                            )
                            && body.handle_will_arive_first_plant(
                                plant_id,
                                plant,
                                &bodies_within_vision_distance,
                            )
                        }).collect::<Vec<_>>();

                    let mut closest_plant = body.find_closest_plant(
                        &filtered_visible_plants,
                        PlantKind::Banana,
                    );

                    if closest_plant.is_none() {
                        closest_plant = body.find_closest_plant(
                            &filtered_visible_plants,
                            PlantKind::Grass,
                        );
                    }

                    match closest_plant {
                        Some((closest_plant_id, closest_plant)) => {
                            food = Some(FoodInfo {
                                id:        ***closest_plant_id,
                                food_type: FoodType::Plant,
                                pos:       closest_plant.pos,
                                energy:    closest_plant
                                    .get_contained_energy(),
                            })
                        }
                        None => {
                            // Find the closest body
                            if let Some((closest_body_id, closest_body)) = bodies_within_vision_distance
                                .iter()
                                .filter(|(other_body_id, other_body)| {
                                    body.body_type != other_body.body_type
                                    && body.energy > other_body.energy
                                    && other_body.is_alive()
                                    && body.handle_alive_when_arrived_body(
                                        other_body, false,
                                    )
                                    && body.handle_profitable_when_arrived_body(
                                        other_body, false,
                                    )
                                    && body.handle_avoid_new_viruses(other_body)
                                    && body.handle_will_arive_first_body(
                                        other_body_id,
                                        other_body,
                                        &bodies_within_vision_distance,
                                    )
                                    && body.handle_do_not_complete_with_relatives(
                                        other_body_id,
                                        &other_body.pos,
                                        &bodies,
                                        &bodies_within_vision_distance_of_my_type,
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
                                    id:        **closest_body_id,
                                    food_type: FoodType::Body(closest_body.viruses.clone()),
                                    pos:       closest_body.pos,
                                    energy:    closest_body.energy,
                                })
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
                            removed_plants.insert(food.id, food.pos);
                            plants_n -= 1;
                        }
                    }
                } else {
                    body.status =
                        Status::FollowingTarget(food.id, food.pos);
                    bodies.get_mut(body_id).unwrap().status =
                        body.status;

                    body.pos.x += (food.pos.x - body.pos.x)
                        * (body.speed / distance_to_food);
                    body.pos.y += (food.pos.y - body.pos.y)
                        * (body.speed / distance_to_food);

                    continue;
                }
            }

            // Procreate
            if body.handle_procreation(
                body_id,
                &mut new_bodies,
                &mut removed_bodies,
                &mut rng,
            ) {
                continue;
            }

            body.handle_walking_idle(&area_size, &mut rng);
        }

        for (plant_id, plant_pos) in &removed_plants {
            plants
                .get_mut(&plants_cells.get_cell_by_pos(plant_pos))
                .unwrap()
                .remove(plant_id);
        }

        for body_id in &removed_bodies {
            bodies.remove(body_id);
        }

        for (new_body_id, new_body) in new_bodies {
            bodies.insert(new_body_id, new_body);
        }

        if is_draw_mode {
            if !is_key_down(KeyCode::Space) {
                if zoom.zoomed {
                    for plant in Plant::get_plants_to_draw(
                        &plants_cells,
                        &zoom,
                        &plants,
                        &removed_plants,
                        plants_n,
                    ) {
                        plant.draw();
                    }

                    for body in bodies.values() {
                        let drawing_strategy =
                            body.get_drawing_strategy(&zoom);

                        if info.body_info {
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
                                if let Status::FollowingTarget(
                                    _,
                                    target_pos,
                                ) = body.status
                                {
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
                        }

                        if drawing_strategy.body {
                            body.draw();
                        }

                        if drawing_strategy.vision_distance
                            && info.body_info
                            && body.is_alive()
                        {
                            body.draw_info();
                        }
                    }
                } else {
                    for body in bodies.values() {
                        body.draw();
                    }

                    for cell in plants.values() {
                        for plant in cell.values() {
                            plant.draw();
                        }
                    }
                }

                last_updated = Instant::now();
            }

            if info.evolution_info.show {
                show_evolution_info(
                    &zoom,
                    &area_size,
                    &mut info,
                    plants_n,
                    bodies.len(),
                    &condition,
                );
            }

            if unsafe { SHOW_FPS } {
                show_fps(&zoom);
            }

            next_frame().await;
        }
    }
}
