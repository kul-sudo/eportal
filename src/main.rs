#![feature(core_intrinsics)]

mod constants;

use constants::*;

use std::{
    collections::HashMap,
    intrinsics::unlikely,
    thread::sleep,
    time::{Duration, Instant},
};

use macroquad::{
    camera::{set_camera, Camera2D},
    color::{Color, GREEN, WHITE},
    input::{is_mouse_button_pressed, mouse_position},
    math::{vec2, Rect, Vec2},
    miniquad::{window::set_fullscreen, MouseButton},
    rand::gen_range,
    shapes::{draw_circle, draw_line, draw_rectangle_lines, draw_triangle},
    text::{draw_text, measure_text},
    window::{next_frame, screen_height, screen_width, Conf},
};
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Status<'a> {
    FollowingBody(&'a Body<'a>),
    FollowingPlant(&'a Plant),
    EscapingBody(&'a Body<'a>),
    Sleeping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IQStage {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EatingStrategy {
    Bodies,
    Plants,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Body<'a> {
    x: f32,
    y: f32,
    energy: f32,
    speed: f32,
    vision_distance: f32,
    eating_strategy: EatingStrategy,
    divison_threshold: f32,
    mass: f32,
    iq: IQStage,
    color: Color,
    status: Status<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Plant {
    x: f32,
    y: f32,
}

macro_rules! distance {
    ($components1:expr, $components2:expr) => {
        f32::sqrt(
            $components1
                .iter()
                .enumerate()
                .map(|(index, component)| (component - $components2[index]).powf(2.0))
                .sum(),
        )
    };
}

// fn distance(components1: Vec<f32>, components2: Vec<f32>) -> f32 {
//     f32::sqrt(
//         components1
//             .iter()
//             .enumerate()
//             .map(|(index, component)| (component - components2[index]).powf(2.0))
//             .sum(),
//     )
// }

fn draw_body(x: f32, y: f32, color: Color) {
    draw_circle(x, y, OBJECT_RADIUS, color);
}

fn draw_plant(plant: &Plant) {
    draw_triangle(
        Vec2 {
            x: plant.x,
            y: plant.y - OBJECT_RADIUS,
        },
        Vec2 {
            x: plant.x + OBJECT_RADIUS * (COSINE_OF_30_DEGREES),
            y: plant.y + OBJECT_RADIUS / 2.0,
        },
        Vec2 {
            x: plant.x - OBJECT_RADIUS * (COSINE_OF_30_DEGREES),
            y: plant.y + OBJECT_RADIUS / 2.0,
        },
        GREEN,
    );
}

fn get_zoom_target(camera: &mut Camera2D, area_size: (f32, f32)) {
    let mouse_position = mouse_position();
    let (mut target_x, mut target_y) = (mouse_position.0 * MAX_ZOOM, mouse_position.1 * MAX_ZOOM);

    (target_x, target_y) = (
        target_x
            .max(area_size.0 / MAX_ZOOM / 2.0)
            .min(area_size.0 * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
        target_y
            .max(area_size.1 / MAX_ZOOM / 2.0)
            .min(area_size.1 * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
    );

    camera.target = vec2(target_x, target_y);
    camera.zoom = vec2(MAX_ZOOM / area_size.0 * 2.0, MAX_ZOOM / area_size.1 * 2.0);
    set_camera(camera);
}

fn default_camera(camera: &mut Camera2D, area_size: (f32, f32)) {
    camera.target = vec2(area_size.0 / 2.0, area_size.1 / 2.0);
    camera.zoom = vec2(MIN_ZOOM / area_size.0 * 2.0, MIN_ZOOM / area_size.1 * 2.0);
    set_camera(camera);
}

fn get_nearest_plant_for_body(plants: &[Plant], body: &Body) -> Option<(f32, (usize, Plant))> {
    let (plant_id, plant) = plants
        .iter()
        .enumerate()
        .min_by_key(|(_, plant)| distance!([plant.x, plant.y], [body.x, body.y]) as isize)?;
    Some((
        distance!([plant.x, plant.y], [body.x, body.y]),
        (plant_id, *plant),
    ))
}

fn get_with_deviation(value: f32, rng: &mut ThreadRng) -> f32 {
    let part = value * DEVIATION;
    rng.gen_range(value - part..value + part)
}

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
fn randomly_spawn_plant(
    bodies: &mut HashMap<usize, Body>,
    plants: &mut Vec<Plant>,
    rng: &mut ThreadRng,
    area_size: (f32, f32),
) {
    let starting_point = Instant::now();

    let mut x;
    let mut y;

    while {
        if starting_point.elapsed().as_nanos()
            >= Duration::from_millis(PLANT_SPAWN_TIME_LIMIT).as_nanos()
        {
            return;
        }
        x = rng.gen_range(0.0..area_size.0);
        y = rng.gen_range(0.0..area_size.1);
        (x <= OBJECT_RADIUS + MIN_GAP || x >= area_size.0 - OBJECT_RADIUS - MIN_GAP)
            || (y <= OBJECT_RADIUS + MIN_GAP || y >= area_size.1 - OBJECT_RADIUS - MIN_GAP)
            || bodies
                .values()
                .any(|body| distance!([body.x, body.y], [x, y]) <= OBJECT_RADIUS * 2.0 + MIN_GAP)
            || plants
                .iter()
                .any(|plant| distance!([plant.x, plant.y], [x, y]) <= OBJECT_RADIUS * 2.0 + MIN_GAP)
    } {}

    plants.push(Plant { x, y });
}

fn spawn_body(
    bodies: &mut HashMap<usize, Body>,
    x: f32,
    y: f32,
    rng: &mut ThreadRng,
    energy: f32,
    speed: f32,
    vision_distance: f32,
    eating_strategy: EatingStrategy,
    division_threshold: f32,
    mass: f32,
    iq: IQStage,
    color: Color,
) {
    bodies.insert(
        bodies.len() + 1,
        Body {
            x,
            y,
            energy: get_with_deviation(AVERAGE_ENERGY, rng),
            speed: get_with_deviation(AVERAGE_SPEED, rng),
            vision_distance: get_with_deviation(AVERAGE_VISION_DISTANCE, rng),
            eating_strategy,
            divison_threshold: get_with_deviation(AVERAGE_DIVISION_THRESHOLD, rng),
            mass: get_with_deviation(AVERAGE_MASS, rng),
            iq: IQStage::Zero,
            color,
            status: Status::Sleeping,
        },
    );
}

fn randomly_spawn_body(
    bodies: &mut HashMap<usize, Body>,
    area_size: (f32, f32),
    energy: f32,
    speed: f32,
    vision_distance: f32,
    eating_strategy: EatingStrategy,
    division_threshold: f32,
    mass: f32,
    iq: IQStage,
) {
    let rng = &mut thread_rng();

    let mut x;
    let mut y;

    while {
        x = rng.gen_range(0.0..area_size.0);
        y = rng.gen_range(0.0..area_size.1);
        (x <= OBJECT_RADIUS + MIN_GAP || x >= area_size.0 - OBJECT_RADIUS - MIN_GAP)
            || (y <= OBJECT_RADIUS + MIN_GAP || y >= area_size.1 - OBJECT_RADIUS - MIN_GAP)
            || bodies
                .values()
                .any(|body| distance!([body.x, body.y], [x, y]) < OBJECT_RADIUS * 2.0 + MIN_GAP)
    } {}

    let real_color_gap = COLOR_GAP / ((BODIES_N + 1) as f32).powf(1.0 / 3.0);

    let mut color = Color::from_rgba(
        gen_range(50, 250),
        gen_range(50, 250),
        gen_range(50, 250),
        255,
    );

    while bodies.values().any(|body| {
        let current_body_rgb = [body.color.r, body.color.g, body.color.b];
        let green_rgb = [GREEN.r, GREEN.g, GREEN.b];
        distance!(current_body_rgb.clone(), vec![color.r, color.g, color.b]) < real_color_gap
            || distance!(current_body_rgb, green_rgb) < real_color_gap
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
        x,
        y,
        rng,
        energy,
        speed,
        vision_distance,
        eating_strategy,
        division_threshold,
        mass,
        iq,
        color,
    );
}

fn window_conf() -> Conf {
    Conf {
        window_title: "My game".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Make the window fullscreen for Linux
    set_fullscreen(true);
    sleep(Duration::from_secs(1));
    next_frame().await;

    let screen_size = (screen_width(), screen_height());
    let area_size = (screen_size.0 * OBJECT_RADIUS, screen_size.1 * OBJECT_RADIUS);

    let mut bodies: HashMap<usize, Body> = HashMap::with_capacity(BODIES_N);
    let mut plants: Vec<Plant> = Vec::with_capacity(PLANTS_N);

    let rng = &mut thread_rng();

    for i in 1..BODIES_N {
        let eating_strategy = if i >= BODY_EATERS_N {
            EatingStrategy::Plants
        } else {
            EatingStrategy::Bodies
        };

        randomly_spawn_body(
            &mut bodies,
            area_size,
            get_with_deviation(AVERAGE_ENERGY, rng),
            get_with_deviation(AVERAGE_SPEED, rng),
            get_with_deviation(AVERAGE_VISION_DISTANCE, rng),
            eating_strategy,
            get_with_deviation(AVERAGE_DIVISION_THRESHOLD, rng),
            get_with_deviation(AVERAGE_MASS, rng),
            IQStage::Zero,
        );
    }

    // Spawn the plants
    for _ in 0..PLANTS_N {
        randomly_spawn_plant(&mut bodies, &mut plants, rng, area_size)
    }

    // // Get the nearest plant for each spawned body
    // for body in bodies.values_mut() {
    //     body.nearest_plant = get_nearest_plant_for_body(&plants, body);
    //     // body.nearest_body = get_nearest_body_for_body(&bodies, body)
    // }

    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, area_size.0, area_size.1));

    default_camera(&mut camera, area_size);

    let mut zoom_mode = false;

    let mut last_updated = Instant::now();

    loop {
        if is_mouse_button_pressed(MouseButton::Left) {
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

        // if last_left_right_sleep.elapsed().as_millis()
        //     >= Duration::from_millis(BODY_JUMP_DELAY).as_millis()
        // {
        //     if is_key_down(KeyCode::Left) {
        //         match body_to_inspect {
        //             Some(index) => {
        //                 if bodies.get(&(index - 1)).is_some() {
        //                     body_to_inspect = Some(index - 1)
        //                 }
        //             }
        //             None => body_to_inspect = Some(0),
        //         }
        //
        //         last_left_right_sleep = Instant::now();
        //     } else if is_key_down(KeyCode::Right) {
        //         match body_to_inspect {
        //             Some(index) => {
        //                 if bodies.get(&(index + 1)).is_some() {
        //                     body_to_inspect = Some(index + 1)
        //                 }
        //             }
        //             None => body_to_inspect = Some(bodies.len() - 1),
        //         }
        //
        //         last_left_right_sleep = Instant::now();
        //     }
        // }
        //
        // {
        //     if body_to_inspect.is_some() {
        //         let body = bodies.get(&body_to_inspect.unwrap()).unwrap();
        //         camera.target = vec2(body.x, body.y);
        //         // camera.target = vec2(area_size.0 / 2.0, area_size.1 / 2.0);
        //         // println!("{:?}", camera.zoom);
        //         // println!("a = {:?}", 1.0 / area_size.0);
        //         camera.zoom = vec2(zoom / area_size.0 * 2.0, zoom / area_size.1 * 2.0);
        //         // camera.zoom = vec2(1.0 / area_size.0 * zoom, -1.0 / area_size.1 * zoom);
        //         set_camera(&camera);
        //     }
        // }

        if unlikely(rng.gen_range(0.0..1.0) > 1.0 - PLANT_SPAWN_CHANCE) {
            randomly_spawn_plant(&mut bodies, &mut plants, rng, area_size)
        }

        let mut bodies_to_iter = bodies.iter_mut().collect::<Vec<_>>();
        bodies_to_iter.shuffle(rng);

        let is_draw_mode =
            last_updated.elapsed().as_millis() >= Duration::from_secs(1 / FPS).as_millis();

        for (body_id, body) in bodies_to_iter {
            // body.nearest_plant = get_nearest_plant_for_body(&plants, body);
            // body.nearest_body = get_nearest_body_for_body(&mut bodies, body);
            // update_nearest_body(body, &bodies);

            // body.energy -= ENERGY_FOR_WALKING;
            // match body.preference {
            //     Preference::Plants => {
            //         // Move towards the nearest plant
            //         let (distance_to_plant, (plant_id, nearest_plant)) =
            //             body.nearest_plant.unwrap();
            //         let (dx, dy) = (nearest_plant.x - body.x, nearest_plant.y - body.y);
            //         let coeff = body.speed / distance_to_plant;
            //
            //         body.x += coeff * dx;
            //         body.y += coeff * dy;
            //         body.status = Status::FollowingPlant(nearest_plant);
            //
            //         if zoom_mode && is_draw_mode {
            //             draw_line(body.x, body.y, nearest_plant.x, nearest_plant.y, 5.0, WHITE);
            //         }
            //
            //         // If there's been a contact between the body and a plant, handle it
            //         if distance!([body.x, body.y], [nearest_plant.x, nearest_plant.y])
            //             <= OBJECT_RADIUS
            //         {
            //             body.energy += PLANT_HP;
            //             plants.remove(plant_id);
            //             body.nearest_plant = None;
            //             body.status = Status::Sleeping
            //         }
            //     }
            //     Preference::Bodies => {
            //         //     for (enemy_id, mut enemy_body) in bodies.clone().into_iter() {
            //         //         if (enemy_body.x, enemy_body.y) == (body.x, body.y) {
            //         //             bodies.remove(match body.hp.cmp(&enemy_body.hp) {
            //         //                 Ordering::Less => {
            //         //                     enemy_body.hp += body.hp.min(MAX_HP - enemy_body.hp);
            //         //                     body_id
            //         //                 }
            //         //                 Ordering::Equal => {
            //         //                     let options = [body_id, &enemy_id];
            //         //                     let chosen = options.choose(rng).unwrap();
            //         //
            //         //                     if *chosen == body_id {
            //         //                         enemy_body.hp += body.hp.min(MAX_HP - enemy_body.hp);
            //         //                     } else {
            //         //                         body.hp += enemy_body.hp.min(MAX_HP - body.hp);
            //         //                     }
            //         //
            //         //                     chosen
            //         //                 }
            //         //                 Ordering::Greater => {
            //         //                     enemy_body.hp += body.hp.min(MAX_HP - enemy_body.hp);
            //         //                     &enemy_id
            //         //                 }
            //         //             });
            //         //         }
            //         //     }
            //     }
            // }
            //
            if is_draw_mode {
                draw_text(
                    &body_id.to_string(),
                    body.x
                        - measure_text(&body_id.to_string(), None, BODY_INFO_FONT_SIZE, 1.0).width
                            / 2.0,
                    body.y - OBJECT_RADIUS - MIN_GAP,
                    BODY_INFO_FONT_SIZE as f32,
                    WHITE,
                );

                draw_body(body.x, body.y, body.color);
                // draw_text(&format!("zoom {}", zoom), 10.0, 20.0, 20.0, WHITE);
            }
        }

        if is_draw_mode {
            draw_rectangle_lines(0.0, 0.0, area_size.0, area_size.1, 30.0, WHITE);
            plants.iter().for_each(draw_plant);
        }

        if is_draw_mode {
            last_updated = Instant::now();
            next_frame().await;
        }
    }
}
