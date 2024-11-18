#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(more_float_constants)]
#![feature(variant_count)]
#![feature(let_chains)]

mod body;
mod cells;
mod condition;
mod constants;
mod cross;
mod plant;
mod smart_drawing;
mod user_constants;
mod utils;
mod zoom;

use body::*;
use cells::*;
use condition::*;
use constants::*;
use cross::*;
use plant::*;
use smart_drawing::*;
use user_constants::*;
use utils::*;
use zoom::*;

use macroquad::prelude::{
    draw_circle_lines, draw_line, is_key_pressed,
    is_mouse_button_pressed, mouse_position, next_frame,
    screen_height, screen_width, set_fullscreen, vec2, Camera2D,
    Conf, KeyCode, MouseButton, Rect, Vec2, WHITE,
};
use rand::{rngs::StdRng, seq::IteratorRandom, SeedableRng};
use std::{
    collections::HashMap,
    intrinsics::unlikely,
    mem::variant_count,
    sync::LazyLock,
    time::{Duration, Instant},
};

fn window_conf() -> Conf {
    Conf {
        window_title: "eportal".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

static FPS_DURATION: LazyLock<u128> =
    LazyLock::new(|| Duration::from_secs(1 / FPS).as_millis());
pub static AREA_SIZE: LazyLock<Vec2> = LazyLock::new(|| {
    vec2(
        // OBJECT_RADIUS is equal to one pixel when unzoomed
        screen_width() * OBJECT_RADIUS,
        screen_height() * OBJECT_RADIUS,
    )
});
pub static CELLS: LazyLock<Cells> = LazyLock::new(|| {
    let mut cells = Cells::default();

    let area_size_ratio = AREA_SIZE.x / AREA_SIZE.y;

    // Get `k` out of PLANTS_N/k = DEFAULT_PLANTS/p
    // where `k` is the real number of cells
    // and `p` is the default number of cells.
    cells.rows = ((DEFAULT_CELL_ROWS as f32
        * (DEFAULT_AREA_SIZE_RATIO * unsafe { PLANTS_N } as f32
            / (area_size_ratio * DEFAULT_PLANTS_N as f32))
            .sqrt())
    .round() as usize)
        .clamp(50, 200);
    cells.columns =
        (cells.rows as f32 * area_size_ratio).round() as usize;

    cells.cell_width = AREA_SIZE.x / cells.columns as f32;
    cells.cell_height = AREA_SIZE.y / cells.rows as f32;

    cells
});

#[macroquad::main(window_conf)]
async fn main() {
    assert_eq!(Condition::ALL.len(), variant_count::<Condition>());
    assert_eq!(Virus::ALL.len(), variant_count::<Virus>());
    assert_eq!(Skill::ALL.len(), variant_count::<Skill>());
    assert_eq!(PlantKind::ALL.len(), variant_count::<PlantKind>());

    config_setup(true);

    set_fullscreen(true);
    next_frame().await;

    // Needed for randomness
    let mut rng = StdRng::from_rng(&mut rand::thread_rng()).unwrap();

    // Calculations
    let area_space = AREA_SIZE.x * AREA_SIZE.y;

    unsafe {
        PLANTS_N = (PLANTS_DENSITY * area_space).round() as usize;
        PLANTS_N_FOR_ONE_STEP =
            (PLANT_SPAWN_CHANCE * area_space).round() as usize;
    }

    // Camera
    let mut camera = Camera2D::from_display_rect(Rect::new(
        0.0,
        0.0,
        AREA_SIZE.x,
        AREA_SIZE.y,
    ));

    default_camera(&mut camera);

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

    let mut bodies: Vec<Vec<HashMap<BodyId, Body>>> =
        vec![vec![HashMap::new(); CELLS.columns]; CELLS.rows];
    let mut plants: Vec<Vec<HashMap<PlantId, Plant>>> =
        vec![vec![HashMap::new(); CELLS.columns]; CELLS.rows];
    let mut crosses: Vec<Vec<HashMap<CrossId, Cross>>> =
        vec![vec![HashMap::new(); CELLS.columns]; CELLS.rows];

    // Need to be handled manually to avoid extracting all out of the cells
    let mut plants_n = 0;
    let mut bodies_n = 0;

    // Spawn the bodies
    for i in
        0..unsafe { OMNIVOROUS_N + HERBIVOROUS_N + CARNIVOROUS_N }
    {
        Body::randomly_spawn_body(
            &mut bodies,
            unsafe {
                match i {
                    _ if (0..OMNIVOROUS_N).contains(&i) => {
                        EatingStrategy::Omnivorous
                    }
                    _ if (OMNIVOROUS_N
                        ..OMNIVOROUS_N + HERBIVOROUS_N)
                        .contains(&i) =>
                    {
                        EatingStrategy::Herbivorous
                    }
                    _ if (OMNIVOROUS_N + HERBIVOROUS_N
                        ..OMNIVOROUS_N
                            + HERBIVOROUS_N
                            + CARNIVOROUS_N)
                        .contains(&i) =>
                    {
                        EatingStrategy::Carnivorous
                    }
                    _ => unreachable!(),
                }
            },
            i + 1,
            &mut rng,
        );

        bodies_n += 1;
    }

    // Spawn the plants
    for _ in 0..unsafe { PLANTS_N } {
        Plant::randomly_spawn_plant(&bodies, &mut plants, &mut rng);

        plants_n += 1;
    }

    // Zoom
    let mut zoom = generate_zoom_struct();

    // Needed for the FPS
    let mut last_updated = Instant::now();

    let mut is_draw_prevented = false;

    loop {
        // Handle interactions
        if unlikely(is_key_pressed(KeyCode::Escape)) {
            std::process::exit(0);
        }

        if unlikely(is_key_pressed(KeyCode::Space)) {
            is_draw_prevented = !is_draw_prevented;
        }

        if unlikely(is_mouse_button_pressed(MouseButton::Left)) {
            if zoom.zoomed {
                default_camera(&mut camera);
                zoom.mouse_pos = None;
            } else {
                zoom.rect = None;
                zoom.extended_rect = None;
                zoom.rect = None;
            }

            zoom.zoomed = !zoom.zoomed
        }

        if zoom.zoomed && unlikely(is_key_pressed(KeyCode::Key1)) {
            info.body_info = !info.body_info;
        }

        if unlikely(is_key_pressed(KeyCode::Key2)) {
            info.evolution_info.show = !info.evolution_info.show;
            info.evolution_info.last_updated = Some(Instant::now());
        }

        if unlikely(is_key_pressed(KeyCode::Key3)) {
            config_setup(false);
        }

        if zoom.zoomed {
            // There's no reason to zoom in again if the mouse position hasn't been changed
            let current_mouse_pos = Vec2::from(mouse_position());
            match zoom.mouse_pos {
                Some(mouse_pos) => {
                    if mouse_pos != current_mouse_pos {
                        zoom.mouse_pos = Some(current_mouse_pos);
                        get_zoom_target(&mut camera, &mut zoom);
                    }
                }
                None => {
                    zoom.mouse_pos = Some(current_mouse_pos);
                    get_zoom_target(&mut camera, &mut zoom);
                }
            }
        }

        let mut new_bodies: HashMap<BodyId, Body> = HashMap::new();

        let mut removed_plants: HashMap<PlantId, Vec2> =
            HashMap::new();
        let mut removed_bodies: HashMap<BodyId, Vec2> =
            HashMap::new();

        Condition::update_condition(&mut condition, &mut rng);

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
                let random_row =
                    plants.iter().choose(&mut rng).unwrap();
                let random_column =
                    random_row.iter().choose(&mut rng).unwrap();

                if let Some((random_plant_id, random_plant)) =
                    random_column.iter().choose(&mut rng)
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
                &mut rng,
            );

            plants_n += 1;
        }

        let is_draw_mode = !is_draw_prevented
            && last_updated.elapsed().as_millis() >= *FPS_DURATION;

        // Whether enough time has passed to draw a new frame
        for row in unsafe {
            &mut (*(&mut bodies
                as *mut Vec<Vec<HashMap<BodyId, Body>>>))
        } {
            for column in row {
                for (body_id, body) in column {
                    if removed_bodies.contains_key(body_id) {
                        continue;
                    }

                    body.handle_viruses();
                    body.handle_lifespan();

                    // Handle if dead to become a cross
                    if body.energy < unsafe { MIN_ENERGY }
                        || body_id.elapsed().as_secs_f32()
                            > body.lifespan
                    {
                        body.status = Status::Cross;
                        //body.set_status(
                        //    Status::Cross,
                        //    body_id,
                        //    &mut bodies,
                        //    unsafe {
                        //        &mut (*(&mut crosses
                        //            as *mut Vec<
                        //                Vec<HashMap<Instant, Cross>>,
                        //            >))
                        //    },
                        //    &mut plants,
                        //);

                        removed_bodies.insert(*body_id, body.pos);

                        continue;
                    }

                    if body
                        .handle_energy(body_id, &mut removed_bodies)
                    {
                        continue;
                    }

                    // Escape
                    let mut visible_bodies = HashMap::new();

                    get_visible!(body, bodies, visible_bodies);

                    visible_bodies.remove(body_id);

                    let chasers = &mut visible_bodies;

                    chasers.retain(|other_body_id, other_body| {
                        !removed_bodies.contains_key(other_body_id)
                            && if let Status::FollowingTarget(
                                target_id,
                                _,
                                _,
                            ) = other_body.status
                            {
                                &target_id == body_id
                            } else {
                                false
                            }
                    });

                    if !chasers.is_empty() {
                        if body
                            .skills
                            .contains(&Skill::PrioritizeFasterChasers)
                            && chasers.iter().any(
                                |(_, other_body)| {
                                    other_body.speed > body.speed
                                },
                            )
                        {
                            chasers.retain(|_, other_body| {
                                other_body.speed > body.speed
                            })
                        }

                        if let Some((
                            closest_chasing_body_id,
                            closest_chasing_body,
                        )) =
                            chasers.iter().min_by(|(_, a), (_, b)| {
                                body.pos.distance(a.pos).total_cmp(
                                    &body.pos.distance(b.pos),
                                )
                            })
                        {
                            body.status = Status::EscapingBody(
                                **closest_chasing_body_id,
                                closest_chasing_body.body_type,
                            );

                            let distance_to_closest_chasing_body =
                                body.pos.distance(
                                    closest_chasing_body.pos,
                                );

                            body.last_pos.x -= (closest_chasing_body.last_pos.x
                            - body.last_pos.x)
                            * (body.speed
                            / distance_to_closest_chasing_body);
                            body.last_pos.y -= (closest_chasing_body.last_pos.y
                            - body.last_pos.y)
                            * (body.speed
                            / distance_to_closest_chasing_body);

                            body.wrap();

                            continue;
                        }
                    }

                    // Eating
                    let food = body.find_food(
                        body_id,
                        unsafe {
                            &(*(&bodies
                                as *const Vec<
                                    Vec<HashMap<BodyId, Body>>,
                                >))
                        },
                        unsafe {
                            &(*(&plants
                                as *const Vec<
                                    Vec<HashMap<PlantId, Plant>>,
                                >))
                        },
                        unsafe {
                            &(*(&crosses
                                as *const Vec<
                                    Vec<HashMap<CrossId, Cross>>,
                                >))
                        },
                        &removed_bodies,
                        &removed_plants,
                    );

                    if let Some(food) = food {
                        let distance_to_food =
                            body.pos.distance(food.pos);
                        if distance_to_food <= body.speed {
                            body.energy += match body.eating_strategy
                            {
                                EatingStrategy::Omnivorous => {
                                    food.energy
                                        * unsafe {
                                            OMNIVOROUS_FOOD_PART
                                        }
                                }
                                EatingStrategy::Herbivorous
                                | EatingStrategy::Carnivorous => {
                                    food.energy
                                }
                            };

                            body.last_pos = food.pos;

                            match food.food_type {
                                ObjectType::Body => {
                                    body.get_viruses(
                                        food.viruses.unwrap(),
                                    );
                                    removed_bodies
                                        .insert(food.id, food.pos);
                                }
                                ObjectType::Cross => {
                                    body.get_viruses(
                                        food.viruses.unwrap(),
                                    );

                                    let Cell { i, j } = CELLS
                                        .get_cell_by_pos(food.pos);
                                    crosses[i][j].remove(&food.id);
                                }
                                ObjectType::Plant => {
                                    removed_plants
                                        .insert(food.id, food.pos);
                                    plants_n -= 1;
                                }
                            }

                            body.status = Status::Undefined;
                        } else {
                            body.status = Status::FollowingTarget(
                                food.id,
                                food.pos,
                                food.food_type,
                            );
                            body.last_pos.x += (food.pos.x
                                - body.last_pos.x)
                                * (body.speed / distance_to_food);
                            body.last_pos.y += (food.pos.y
                                - body.last_pos.y)
                                * (body.speed / distance_to_food);
                        }

                        continue;
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

                    body.handle_walking_or_idle(&mut rng);
                }
            }
        }

        for row in &mut crosses {
            for column in row {
                column.retain(|_, cross| {
                    cross.timestamp.elapsed().as_secs()
                        <= unsafe { CROSS_LIFESPAN }
                })
            }
        }

        for (body_id, body_pos) in &removed_bodies {
            let Cell { i, j } = CELLS.get_cell_by_pos(*body_pos);
            let body = bodies[i][j].get(body_id).unwrap();

            if let Status::Cross = body.status {
                crosses[i][j].insert(*body_id, Cross::new(body));
            }

            bodies[i][j].remove(body_id);
            bodies_n -= 1;
        }

        let mut changed: Vec<(Instant, Vec2)> = Vec::new();

        for row in &mut bodies {
            for column in row {
                for (body_id, body) in column.iter_mut() {
                    if body.pos != body.last_pos {
                        changed.push((*body_id, body.pos));
                    }
                }
            }
        }

        for (body_id, body_pos) in &changed {
            let Cell { i: old_i, j: old_j } =
                CELLS.get_cell_by_pos(*body_pos);
            let mut body =
                bodies[old_i][old_j].get(body_id).unwrap().clone();

            bodies[old_i][old_j].remove(body_id);

            body.pos = body.last_pos;

            let Cell { i: new_i, j: new_j } =
                CELLS.get_cell_by_pos(body.pos);
            bodies[new_i][new_j].insert(*body_id, body);
        }

        for (new_body_id, new_body) in new_bodies {
            let Cell { i, j } = CELLS.get_cell_by_pos(new_body.pos);

            bodies[i][j].insert(new_body_id, new_body);
            bodies_n += 1;
        }

        for (plant_id, plant_pos) in &removed_plants {
            let Cell { i, j } = CELLS.get_cell_by_pos(*plant_pos);
            plants[i][j].remove(plant_id);
        }

        if is_draw_mode {
            for row in &crosses {
                for column in row {
                    for cross in column.values() {
                        cross.draw(&zoom);
                    }
                }
            }

            if zoom.zoomed {
                for plant in Plant::get_plants_to_draw(
                    &zoom,
                    &plants,
                    &removed_plants,
                    plants_n,
                ) {
                    plant.draw();
                }

                for row in &bodies {
                    for column in row {
                        for body in column.values() {
                            let DrawingStrategy {
                                body: draw_body,
                                vision_distance: draw_vision_distance,
                                target_line: draw_target_line,
                            } = body.get_drawing_strategy(&zoom);

                            if info.body_info {
                                if draw_vision_distance {
                                    draw_circle_lines(
                                        body.pos.x,
                                        body.pos.y,
                                        body.vision_distance,
                                        2.0,
                                        body.color,
                                    );
                                }

                                if draw_target_line {
                                    if let Status::FollowingTarget(
                                        _,
                                        target_pos,
                                        _,
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

                            if draw_body {
                                body.draw();
                            }

                            if draw_vision_distance && info.body_info
                            {
                                body.draw_info();
                            }
                        }
                    }
                }
            } else {
                for row in &bodies {
                    for column in row {
                        for body in column.values() {
                            body.draw();
                        }
                    }
                }

                for row in &plants {
                    for column in row {
                        for plant in column.values() {
                            plant.draw();
                        }
                    }
                }

                last_updated = Instant::now();
            }

            if info.evolution_info.show {
                show_evolution_info(
                    &zoom, &mut info, plants_n, bodies_n, &condition,
                );
            }

            if unsafe { SHOW_FPS } {
                show_fps(&zoom);
            }
        }

        next_frame().await;
    }
}
